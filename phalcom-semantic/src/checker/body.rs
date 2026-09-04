//! Full callable body type checking and CallableAnalysis generation (Spec 04.5 / Wave 5).

use crate::checker::analysis::{AnalysisStatus, CallableAnalysis, CallableAnalysisStatus, NormalReturnFact};
use crate::checker::causal::CausalInvalidity;
use crate::checker::context::{CallableReturnContract, CheckerControl, CheckingContext};
use crate::checker::flow::graph::FlowGraph;
use crate::checker::statement::check_statement;
use crate::db::budget::{CancellationToken, QueryBudget};
use crate::declarations::DeclarationTypeTable;
use crate::identity::{CallableId, ModuleId};
use crate::types::annotation::{ScopedTypeResolver, TypeResolver, type_level_binding_for_parameter};
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
        Statement::Enum(e) => e.range,
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
    pub enum_semantics: Option<&'a crate::enum_semantics::EnumSemanticTable>,
    pub associated_families: Option<&'a crate::associated::AssociatedFamilyTable>,
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
        enum_semantics,
        associated_families,
    } = request;
    let control = CheckerControl::new(budget, cancel);
    // Body annotations are resolved in the callable's lexical type scope. The
    // canonical signature already owns the parameter identities; expose those
    // identities through the same resolver overlay used while lowering the
    // signature itself. Declaration parameters form the outer scope and
    // callable parameters shadow them (notably for constructors).
    let owner_parameters = if callable.side == crate::identity::DispatchSide::Instance {
        declarations
            .generic_signature(callable.declaration_owner())
            .map(|signature| signature.parameters.to_vec())
            .unwrap_or_default()
    } else {
        Vec::new()
    };
    let callable_parameters = declared_signature
        .and_then(|(_, signature)| signature.generics.as_ref())
        .map(|signature| signature.parameters.to_vec())
        .unwrap_or_default();
    let mut type_parameters = std::collections::HashMap::new();
    for parameter in owner_parameters.into_iter().chain(callable_parameters) {
        let name = store.type_parameter(parameter).name.to_string();
        let binding = type_level_binding_for_parameter(store, parameter);
        type_parameters.insert(name, binding);
    }
    let scoped_resolver = ScopedTypeResolver {
        parent: resolver,
        type_parameters,
    };
    let mut ctx = CheckingContext::new_with_dispatch_ref_and_control(store, hierarchy, &scoped_resolver, declarations, dispatch, module, control);
    if let Some(field_signatures) = field_signatures {
        ctx.attach_field_signatures(field_signatures);
    }
    if let Some(field_lifecycle) = field_lifecycle {
        ctx.attach_field_lifecycle(field_lifecycle);
    }
    if let Some(enum_semantics) = enum_semantics {
        ctx.attach_enum_semantics(enum_semantics);
    }
    if let Some(associated_families) = associated_families {
        ctx.attach_associated_families(associated_families);
    }
    ctx.current_callable = Some(callable.clone());
    ctx.current_class = Some(callable.declaration_owner().clone());
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
        ctx.attach_field_lifecycle(field_lifecycle);
        field_lifecycle.seed_flow_for_owner(&mut ctx.flow, callable.declaration_owner(), constructor_body);
    }

    if let Some((signature_id, signature)) = declared_signature {
        ctx.record_semantic_dependency(crate::checker::analysis::SemanticDependency::CallableSignature(signature_id.clone()));
        for parameter in signature.parameters.iter() {
            if let Some(ty) = parameter.declared_type.canonical_type() {
                ctx.seed_stable_record_row_lacks(crate::checker::row_inference::collect_stable_record_row_lacks(ctx.store, ty));
            }
        }
        if let Some(ty) = signature.declared_return.canonical_type() {
            ctx.seed_stable_record_row_lacks(crate::checker::row_inference::collect_stable_record_row_lacks(ctx.store, ty));
        }
        ctx.push_scope();
        for parameter in &signature.parameters {
            ctx.bind_canonical_callable_parameter(parameter, body_range);
        }
        if !signature.is_constructor() || constructor_body {
            let is_dynamic = signature.declared_return.is_dynamic();
            let declared_return = signature.declared_return.to_knowledge();
            if let Some(ret_ty) = declared_return.ty() {
                ctx.expected_return = Some(CallableReturnContract {
                    ty: ret_ty,
                    basis: signature.declared_return.basis,
                    origin: crate::types::evidence::EvidenceOrigin::CallableSignature,
                    is_dynamic: false,
                    source: None,
                });
            } else if is_dynamic {
                ctx.expected_return = Some(CallableReturnContract {
                    ty: ctx.store.unit(),
                    basis: signature.declared_return.basis,
                    origin: crate::types::evidence::EvidenceOrigin::CallableSignature,
                    is_dynamic: true,
                    source: None,
                });
            }
        }
    }

    // 2. Check each statement while charging budget and checking cancellation
    let mut status = CallableAnalysisStatus::Complete;

    for (statement_index, stmt) in body.iter().enumerate() {
        if ctx.is_cancelled() {
            status = CallableAnalysisStatus::Cancelled;
            break;
        }

        if !ctx.flow.is_reachable() {
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
                if ctx.flow.is_reachable() && typed.knowledge.ty() != Some(ctx.store.never()) {
                    let fact = NormalReturnFact {
                        knowledge: typed.knowledge,
                        flow: ctx.current_flow_summary(),
                        status: typed.status,
                        causal_invalidity: typed.causal_invalidity,
                    };
                    ctx.record_return_exit(fact);
                }
                continue;
            }
        }

        check_statement(&mut ctx, stmt);
        if let Some(AnalysisStatus::InternalFailure(incident)) = ctx.terminal_status.clone() {
            status = CallableAnalysisStatus::InternalFailure(incident);
            break;
        }
        if is_tail && ctx.flow.is_reachable() {
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
                let fact = NormalReturnFact {
                    knowledge: unit,
                    flow: ctx.current_flow_summary(),
                    status: exit_status,
                    causal_invalidity: exit_causal,
                };
                ctx.record_return_exit(fact);
            }
        }
    }

    if body.is_empty() && ctx.flow.is_reachable() {
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
        let fact = NormalReturnFact {
            knowledge: unit,
            flow: ctx.current_flow_summary(),
            status: exit_status,
            causal_invalidity: exit_causal,
        };
        ctx.record_return_exit(fact);
    }

    if let Some(AnalysisStatus::InternalFailure(incident)) = ctx.terminal_status {
        status = CallableAnalysisStatus::InternalFailure(incident);
    }
    ctx.finalize(callable, body_range, status)
}
