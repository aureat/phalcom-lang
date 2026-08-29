//! Full callable body type checking and CallableAnalysis generation (Spec 04.5 / Wave 5).

use crate::checker::analysis::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus, NormalReturnFact};
use crate::checker::causal::CausalInvalidity;
use crate::checker::context::{CallableReturnContract, CheckerControl, CheckingContext};
use crate::checker::flow::graph::FlowGraph;
use crate::checker::statement::check_statement;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::declarations::DeclarationTypeTable;
use crate::identity::{CallableId, ModuleId};
use crate::types::annotation::TypeResolver;
use crate::types::evidence::DynamicReason;
use crate::types::outcome::RelationOutcome;
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

use crate::dispatch::SurfaceDispatchResolver;

/// Context holding canonical published semantic inputs for callable body checking.
pub struct BodyAnalysisContext<'a> {
    pub store: &'a mut TypeStore,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
    pub dispatch: &'a SurfaceDispatchResolver,
    pub module: ModuleId,
}

/// Inputs specific to one callable-body analysis.
pub struct CallableBodyRequest<'a> {
    pub callable: CallableId,
    pub body: &'a [Statement],
    pub body_range: SourceRange,
    pub declared_signature: Option<(&'a CallableId, &'a crate::signature::CallableSemanticSignature)>,
    pub budget: QueryBudget,
    pub cancel: &'a CancellationToken,
    pub field_signatures: Option<&'a crate::signature::FieldSignatureTable>,
    pub field_lifecycle: Option<&'a crate::checker::field_lifecycle::FieldLifecycleTable>,
}

/// Analyzes a single callable body and returns a complete [`CallableAnalysis`].
pub fn analyze_callable_body(context: BodyAnalysisContext<'_>, request: CallableBodyRequest<'_>) -> CallableAnalysis {
    let BodyAnalysisContext {
        store,
        hierarchy,
        resolver,
        declarations,
        dispatch,
        module,
    } = context;
    let CallableBodyRequest {
        callable,
        body,
        body_range,
        declared_signature,
        budget,
        cancel,
        field_signatures,
        field_lifecycle,
    } = request;
    let control = CheckerControl::new(budget, cancel);
    let mut ctx = CheckingContext::new_with_dispatch_ref_and_control(store, hierarchy, resolver, declarations, dispatch, module, control);
    if let Some(field_signatures) = field_signatures {
        ctx.attach_field_signatures(field_signatures);
    }
    ctx.current_callable = Some(callable.clone());
    ctx.current_class = Some(callable.owner.clone());
    ctx.current_side = callable.side;

    // 1. Build flow graph for the body statements
    let flow_graph = Arc::new(FlowGraph::from_statements(body));
    ctx.flow_graph = Some(flow_graph);

    // Bind parameters and the constraining return requirement from the exact
    // canonical declaration signature. `inferred_return` is deliberately not
    // consulted here; a body-derived result can never become its own premise.
    let constructor_body = declared_signature
        .is_some_and(|(signature_id, _)| callable.side == crate::identity::DispatchSide::Instance && signature_id.side == crate::identity::DispatchSide::Class);
    let setter_body = matches!(
        callable.selector.kind,
        phalcom_common::selector::SelectorKind::Setter | phalcom_common::selector::SelectorKind::SubscriptSet
    );
    if let Some(field_lifecycle) = field_lifecycle {
        field_lifecycle.seed_flow_for_owner(&mut ctx.flow, &callable.owner, constructor_body);
    }

    if let Some((signature_id, signature)) = declared_signature {
        ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::CallableSignature(signature_id.clone()));
        ctx.push_scope();
        for parameter in &signature.parameters {
            ctx.bind_canonical_callable_parameter(parameter, body_range);
        }
        if !signature.is_constructor() || constructor_body {
            let declared_return = signature.declared_return.to_knowledge();
            if let Some(ret_ty) = declared_return.ty() {
                ctx.expected_return = Some(CallableReturnContract {
                    ty: ret_ty,
                    basis: signature.declared_return.basis,
                    origin: crate::types::evidence::EvidenceOrigin::CallableSignature,
                    source: None,
                });
            }
        }
    }

    // 2. Check each statement while charging budget and checking cancellation
    let mut status = CallableAnalysisStatus::Complete;
    let mut normal_returns = Vec::new();
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
                let mut typed = crate::checker::expression::analyze_expression(&mut ctx, expr, &expected);
                if !constructor_body && !setter_body {
                    if let Some(expected_return) = ctx.expected_return.clone() {
                        let relation = ctx.apply_knowledge_against_type(
                            &typed.knowledge,
                            expected_return.ty,
                            crate::diagnostic::DiagnosticCode::ReturnMismatch,
                            "tail expression result is not assignable to method's declared return type",
                            *range,
                        );
                        if let Some(cause) = relation.cause {
                            typed.status = AnalysisStatus::Invalid(cause);
                            typed.causal_invalidity = typed.causal_invalidity.join(CausalInvalidity::One(cause));
                        } else {
                            typed.status = match &relation.outcome {
                                RelationOutcome::Blocked(reason) => AnalysisStatus::Blocked(reason.clone()),
                                RelationOutcome::Cancelled => AnalysisStatus::Cancelled,
                                RelationOutcome::BudgetExceeded(report) => AnalysisStatus::BudgetExceeded(report.clone()),
                                RelationOutcome::InternalFailure(message) => AnalysisStatus::InternalFailure(ctx.publish_analysis_incident(message)),
                                RelationOutcome::DynamicBoundary(_) => AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection),
                                _ => typed.status.clone(),
                            };
                        }
                        ctx.sync_expression_outcome(&typed);
                    }
                }
                if let Some(AnalysisStatus::InternalFailure(incident)) = ctx.terminal_status.clone() {
                    status = CallableAnalysisStatus::InternalFailure(incident);
                    break;
                }
                if can_fall_through && typed.knowledge.ty() != Some(ctx.store.never()) {
                    normal_returns.push(NormalReturnFact {
                        knowledge: typed.knowledge,
                        flow: ctx.current_flow_summary(),
                        status: typed.status,
                        causal_invalidity: typed.causal_invalidity,
                    });
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
            if let Some(fact) = returned {
                if fact.knowledge.ty() != Some(ctx.store.never()) {
                    normal_returns.push(fact);
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
                    let mut exit_status = AnalysisStatus::Ready;
                    let mut exit_causal = CausalInvalidity::Clean;
                    if !constructor_body && !setter_body {
                        if let Some(expected_return) = ctx.expected_return.clone() {
                            let relation = ctx.apply_knowledge_against_type(
                                &unit,
                                expected_return.ty,
                                crate::diagnostic::DiagnosticCode::ReturnMismatch,
                                "tail statement completes with Unit, which is not assignable to method's declared return type",
                                stmt_range(stmt),
                            );
                            if let Some(cause) = relation.cause {
                                exit_status = AnalysisStatus::Invalid(cause);
                                exit_causal = exit_causal.join(CausalInvalidity::One(cause));
                            }
                        }
                    }
                    normal_returns.push(NormalReturnFact {
                        knowledge: unit,
                        flow: ctx.current_flow_summary(),
                        status: exit_status,
                        causal_invalidity: exit_causal,
                    });
                }
                can_fall_through = false;
            }
        }
    }

    if body.is_empty() && can_fall_through {
        let unit = crate::types::evidence::TypeKnowledge::established(ctx.store.unit(), crate::types::evidence::EvidenceOrigin::Flow);
        let mut exit_status = AnalysisStatus::Ready;
        let mut exit_causal = CausalInvalidity::Clean;
        if !constructor_body && !setter_body {
            if let Some(expected_return) = ctx.expected_return.clone() {
                let relation = ctx.apply_knowledge_against_type(
                    &unit,
                    expected_return.ty,
                    crate::diagnostic::DiagnosticCode::ReturnMismatch,
                    "empty callable body completes with Unit, which is not assignable to method's declared return type",
                    body_range,
                );
                if let Some(cause) = relation.cause {
                    exit_status = AnalysisStatus::Invalid(cause);
                    exit_causal = exit_causal.join(CausalInvalidity::One(cause));
                }
            }
        }
        normal_returns.push(NormalReturnFact {
            knowledge: unit,
            flow: ctx.current_flow_summary(),
            status: exit_status,
            causal_invalidity: exit_causal,
        });
    }

    if let Some(AnalysisStatus::InternalFailure(incident)) = ctx.terminal_status {
        status = CallableAnalysisStatus::InternalFailure(incident);
    }
    ctx.finalize_with_normal_returns(callable, body_range, status, normal_returns)
}
