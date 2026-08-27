//! Full callable body type checking and CallableAnalysis generation (Spec 04.5 / Wave 5).

use crate::checker::analysis::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus};
use crate::checker::context::{CallableReturnContract, CheckerControl, CheckingContext};
use crate::checker::flow::graph::FlowGraph;
use crate::checker::statement::check_statement;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::declarations::DeclarationTypeTable;
use crate::identity::{CallableId, ModuleId};
use crate::types::annotation::TypeResolver;
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use phalcom_ast::ast::Statement;
use phalcom_common::range::SourceRange;
use std::sync::Arc;

fn stmt_range(stmt: &Statement) -> SourceRange {
    match stmt {
        Statement::Class(c) => c.range,
        Statement::TypeAlias(t) => t.range,
        Statement::Let(l) => l.range,
        Statement::Return(r) => r.range,
        Statement::Expr { range, .. } => *range,
        Statement::For(f) => f.range,
        Statement::Break { range } => *range,
        Statement::Continue { range } => *range,
        Statement::Throw { range, .. } => *range,
        Statement::Export(e) => e.range,
    }
}

use crate::dispatch::{CallableSignature, SurfaceDispatchResolver};

/// Returns the declaration signature consumed by a body, including the class-side
/// fallback used for constructor bodies represented on the instance side.
pub(crate) fn signature_consumed_by_body(dispatch: &SurfaceDispatchResolver, callable: &CallableId) -> Option<(CallableId, CallableSignature)> {
    let surface = dispatch.surfaces().get(&callable.owner)?;

    if let Some(signature) = surface.get_callable(callable.side, &callable.selector) {
        let signature_id = surface
            .get_callable_id(callable.side, &callable.selector)
            .cloned()
            .unwrap_or_else(|| callable.clone());
        return Some((signature_id, signature.clone()));
    }

    if callable.side == crate::identity::DispatchSide::Instance {
        let signature = surface.get_callable(crate::identity::DispatchSide::Class, &callable.selector)?;
        let signature_id = surface
            .get_callable_id(crate::identity::DispatchSide::Class, &callable.selector)
            .cloned()
            .unwrap_or_else(|| CallableId::new(callable.owner.clone(), callable.selector.clone(), crate::identity::DispatchSide::Class));
        return Some((signature_id, signature.clone()));
    }

    None
}

/// Context holding canonical published semantic inputs for callable body checking.
pub struct BodyAnalysisContext<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub dispatch: &'a SurfaceDispatchResolver,
    pub module: ModuleId,
}

/// Analyzes a single callable body and returns a complete [`CallableAnalysis`].
pub fn analyze_callable_body(
    callable: CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    dispatch: &SurfaceDispatchResolver,
    module: ModuleId,
    budget: QueryBudget,
    cancel: &CancellationToken,
) -> CallableAnalysis {
    analyze_callable_body_with_fields(
        callable,
        body,
        body_range,
        store,
        hierarchy,
        resolver,
        declarations,
        dispatch,
        module,
        budget,
        cancel,
        None,
    )
}

pub fn analyze_callable_body_with_fields(
    callable: CallableId,
    body: &[Statement],
    body_range: SourceRange,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    resolver: &dyn TypeResolver,
    declarations: &DeclarationTypeTable,
    dispatch: &SurfaceDispatchResolver,
    module: ModuleId,
    budget: QueryBudget,
    cancel: &CancellationToken,
    field_lifecycle: Option<&crate::checker::field_lifecycle::FieldLifecycleTable>,
) -> CallableAnalysis {
    let control = CheckerControl::new(budget, cancel);
    let mut ctx = CheckingContext::new_with_dispatch_ref_and_control(store, hierarchy, resolver, declarations, dispatch, module, control);
    ctx.current_callable = Some(callable.clone());
    ctx.current_class = Some(callable.owner.clone());
    ctx.current_side = callable.side;

    // 1. Build flow graph for the body statements
    let flow_graph = Arc::new(FlowGraph::from_statements(body));
    ctx.flow_graph = Some(flow_graph);

    // Bind parameters and expected return from the exact published signature consumed by this body.
    // Constructor bodies are represented as instance-side bodies, while their public constructor
    // signatures live on the class side; record the class-side identity in that fallback case.
    let sig_opt = signature_consumed_by_body(ctx.dispatch_ref(), &callable);
    let constructor_body = sig_opt
        .as_ref()
        .is_some_and(|(signature_id, _)| callable.side == crate::identity::DispatchSide::Instance && signature_id.side == crate::identity::DispatchSide::Class);
    let setter_body = matches!(
        callable.selector.kind,
        phalcom_common::selector::SelectorKind::Setter | phalcom_common::selector::SelectorKind::SubscriptSet
    );
    if let Some(field_lifecycle) = field_lifecycle {
        field_lifecycle.seed_flow_for_owner(&mut ctx.flow, &callable.owner, constructor_body);
    }

    if let Some((signature_id, sig)) = sig_opt {
        ctx.record_consumed_callable_signature(&signature_id, &sig);
        ctx.push_scope();
        for param in &sig.parameters {
            ctx.bind_callable_parameter(param.local_name.clone(), param.ty.clone(), body_range);
        }
        if let Some(ret_ty) = sig.return_type.ty() {
            ctx.expected_return = Some(CallableReturnContract {
                ty: ret_ty,
                origin: crate::types::evidence::EvidenceOrigin::CallableSignature,
                source: None,
            });
        }
    }

    // 2. Check each statement while charging budget and checking cancellation
    let mut status = CallableAnalysisStatus::Complete;
    let mut normal_return_values = Vec::new();
    let mut can_fall_through = true;

    for (statement_index, stmt) in body.iter().enumerate() {
        if ctx.is_cancelled() {
            status = CallableAnalysisStatus::Cancelled;
            break;
        }

        if let Err(report) = ctx.charge_step() {
            ctx.diagnostics.push(crate::diagnostic::SemanticDiagnostic::warning_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::AnalysisBudgetExceeded,
                format!("callable body analysis exceeded step budget ({}/{})", report.used, report.limit),
                stmt_range(stmt),
            ));
            status = CallableAnalysisStatus::BudgetExceeded;
            break;
        }

        let is_tail = statement_index + 1 == body.len();
        if is_tail {
            if let Statement::Expr { expr, range } = stmt {
                let expected = ctx
                    .expected_return
                    .as_ref()
                    .map(|contract| {
                        crate::checker::expected::ExpectedType::proper_from(contract.ty, crate::checker::expected::ExpectationOrigin::ReturnContract)
                    })
                    .unwrap_or_default();
                let typed = crate::checker::expression::analyze_expression(&mut ctx, expr, &expected);
                if !constructor_body && !setter_body {
                    if let Some(expected_return) = ctx.expected_return.clone() {
                        ctx.apply_knowledge_against_type(
                            &typed.knowledge,
                            expected_return.ty,
                            crate::diagnostic::DiagnosticCode::ReturnMismatch,
                            "tail expression result is not assignable to method's declared return type",
                            *range,
                        );
                    }
                }
                if let Some(AnalysisStatus::InternalFailure(incident)) = ctx.terminal_status.clone() {
                    status = CallableAnalysisStatus::InternalFailure(incident);
                    break;
                }
                if can_fall_through {
                    if typed.knowledge.ty() != Some(ctx.store.never()) {
                        normal_return_values.push(typed.knowledge);
                    }
                }
                can_fall_through = false;
                continue;
            }
        }

        let returned = check_statement(&mut ctx, stmt);
        if let Some(AnalysisStatus::InternalFailure(incident)) = ctx.terminal_status.clone() {
            status = CallableAnalysisStatus::InternalFailure(incident);
            break;
        }
        if can_fall_through {
            if let Some(value) = returned {
                if value.ty() != Some(ctx.store.never()) {
                    normal_return_values.push(value);
                }
                can_fall_through = false;
            } else if matches!(stmt, Statement::Throw { .. } | Statement::Break { .. } | Statement::Continue { .. }) {
                can_fall_through = false;
            } else if is_tail {
                // `let`/`const` and declaration-like statements complete with
                // Unit. Their initializer is checked for diagnostics and
                // binding facts above, but never becomes the callable result.
                let initializer_never = if let Statement::Let(binding) = stmt {
                    binding.value.as_ref().is_some_and(|expr| {
                        ctx.expressions
                            .values()
                            .any(|analysis| analysis.range == expr.range() && analysis.knowledge.ty() == Some(ctx.store.never()))
                    })
                } else {
                    false
                };
                if !initializer_never {
                    let unit = crate::types::evidence::TypeKnowledge::established(ctx.store.unit(), crate::types::evidence::EvidenceOrigin::Flow);
                    if !constructor_body && !setter_body {
                        if let Some(expected_return) = ctx.expected_return.clone() {
                            ctx.apply_knowledge_against_type(
                                &unit,
                                expected_return.ty,
                                crate::diagnostic::DiagnosticCode::ReturnMismatch,
                                "tail statement completes with Unit, which is not assignable to method's declared return type",
                                stmt_range(stmt),
                            );
                        }
                    }
                    normal_return_values.push(unit);
                }
                can_fall_through = false;
            }
        }
    }

    if body.is_empty() && can_fall_through {
        let unit = crate::types::evidence::TypeKnowledge::established(ctx.store.unit(), crate::types::evidence::EvidenceOrigin::Flow);
        if !constructor_body && !setter_body {
            if let Some(expected_return) = ctx.expected_return.clone() {
                ctx.apply_knowledge_against_type(
                    &unit,
                    expected_return.ty,
                    crate::diagnostic::DiagnosticCode::ReturnMismatch,
                    "empty callable body completes with Unit, which is not assignable to method's declared return type",
                    body_range,
                );
            }
        }
        normal_return_values.push(unit);
    }

    if let Some(AnalysisStatus::InternalFailure(incident)) = ctx.terminal_status {
        status = CallableAnalysisStatus::InternalFailure(incident);
    }
    ctx.finalize_with_normal_returns(callable, body_range, status, normal_return_values)
}
