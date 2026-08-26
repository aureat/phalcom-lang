//! Statement type checking engine.

use super::analysis::AnalysisStatus;
use super::binding::{BindingContract, BindingContractOrigin, BindingSeed, reconcile_binding_relation};
use super::call::{CallPremise, CallableApplicationTarget, UnresolvedApplicationReason, analyze_unresolved_application, apply_resolved_callable};
use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::expression::analyze_expression;
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{ContractAssumptionEligibility, DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};
use phalcom_ast::ast::{BindingKind, Pattern, Statement};
use phalcom_common::range::SourceRange;
use phalcom_common::selector::{Selector, SelectorSlot};

/// Checks a single statement, updating context bindings and recording
/// diagnostics. A direct `return` reports its typed normal-return value to
/// callable-body analysis; all other statement forms return `None`.
pub fn check_statement(ctx: &mut CheckingContext<'_>, statement: &Statement) -> Option<TypeKnowledge> {
    match statement {
        Statement::Let(binding) => {
            let (declared_k, annotation_invalidity) = binding
                .annotation
                .as_ref()
                .map(|annotation| {
                    let resolver = ctx.resolver.clone();
                    let (knowledge, causal_invalidity) = ctx.resolve_type_annotation(&resolver, annotation);
                    (Some(knowledge), causal_invalidity)
                })
                .unwrap_or((None, crate::checker::causal::CausalInvalidity::Clean));

            let expected_init = declared_k
                .as_ref()
                .and_then(TypeKnowledge::ty)
                .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::DeclarationContract))
                .unwrap_or_default();
            let mut val_typed = if let Some(expr) = &binding.value {
                analyze_expression(ctx, expr, &expected_init)
            } else {
                if binding.kind == BindingKind::Const || !matches!(binding.pattern, Pattern::Name { .. }) {
                    let message = if matches!(binding.pattern, Pattern::Name { .. }) {
                        "const binding requires an initializer"
                    } else {
                        "a destructuring let/const pattern requires an initializer to unpack"
                    };
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ConstWithoutInitializer,
                        message,
                        binding.range,
                    ));
                }
                TypedExpression::unknown(UnknownReason::MissingInitializer)
            };

            let contract = if let Some(declared_k) = declared_k.as_ref().and_then(TypeKnowledge::ty) {
                Some(BindingContract {
                    ty: declared_k,
                    origin: BindingContractOrigin::SourceAnnotation,
                    source: binding.annotation.as_ref().map(|annotation| annotation.range),
                })
            } else {
                val_typed.knowledge.ty().map(|ty| BindingContract {
                    ty,
                    origin: BindingContractOrigin::InferredInitializer,
                    source: binding.value.as_ref().map(|value| value.range()),
                })
            };
            let relation = match contract.as_ref() {
                None => RelationOutcome::proven(()),
                Some(contract) => match &val_typed.knowledge {
                    TypeKnowledge::Unknown(reason)
                        if matches!(contract.origin, BindingContractOrigin::SourceAnnotation)
                            && reason.contract_assumption_eligibility() == ContractAssumptionEligibility::MaySupplyAssumption =>
                    {
                        RelationOutcome::proven(())
                    }
                    TypeKnowledge::Unknown(reason) => RelationOutcome::Blocked(crate::types::outcome::BlockReason::UnknownType(reason.clone())),
                    TypeKnowledge::Dynamic(_) => RelationOutcome::DynamicBoundary(DynamicBoundaryObligation {
                        reason: "binding contract crosses dynamic boundary".into(),
                    }),
                    TypeKnowledge::Known(_) => ctx.check_knowledge_against_type(&val_typed.knowledge, contract.ty),
                },
            };
            let reconciliation = reconcile_binding_relation(contract.as_ref(), &val_typed.knowledge, relation.clone());
            let mut causal_invalidity = val_typed.causal_invalidity.join(annotation_invalidity);
            if matches!(reconciliation.consistency, crate::checker::binding::BindingConsistency::Refuted { .. }) {
                let mut diag = SemanticDiagnostic::error_in(
                    ctx.current_module.clone(),
                    DiagnosticCode::BindingInitializerMismatch,
                    "initializer expression is not assignable to declared type",
                    binding.range,
                );
                if let Some(ann) = &binding.annotation {
                    diag = diag.with_label(ann.range, "declared type");
                }
                if let Some(val) = &binding.value {
                    diag = diag.with_label(val.range(), "inferred type");
                }
                let cause = ctx.emit_diagnostic(diag).expect("error diagnostic has cause");
                val_typed.invalidate(cause);
                causal_invalidity = val_typed.causal_invalidity.join(annotation_invalidity);
            } else {
                val_typed.status = match relation {
                    RelationOutcome::Blocked(reason) => AnalysisStatus::Blocked(reason),
                    RelationOutcome::Cancelled => AnalysisStatus::Cancelled,
                    RelationOutcome::BudgetExceeded(report) => AnalysisStatus::BudgetExceeded(report),
                    RelationOutcome::InternalFailure(message) => AnalysisStatus::InternalFailure(ctx.publish_analysis_incident(message)),
                    RelationOutcome::DynamicBoundary(_) => AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection),
                    _ => val_typed.status.clone(),
                };
            }
            ctx.sync_expression_outcome(&val_typed);

            bind_declaration_pattern(
                ctx,
                &binding.pattern,
                ValueSemanticFact {
                    knowledge: reconciliation.current,
                    denotation: val_typed.denotation,
                },
                contract,
                causal_invalidity,
                binding.kind == BindingKind::Let,
                binding.range,
            );
            None
        }
        Statement::Return(ret) => {
            let expected_ret = ctx
                .expected_return
                .as_ref()
                .map(|contract| ExpectedType::proper_from(contract.ty, ExpectationOrigin::ReturnContract))
                .unwrap_or_default();
            let val_typed = if let Some(expr) = &ret.value {
                analyze_expression(ctx, expr, &expected_ret)
            } else {
                TypedExpression::established(ctx.store.unit(), EvidenceOrigin::DeclarationSemantics, ret.range)
            };

            if let Some(expected) = ctx.expected_return.clone() {
                ctx.apply_knowledge_against_type(
                    &val_typed.knowledge,
                    expected.ty,
                    DiagnosticCode::ReturnMismatch,
                    "returned value is not assignable to method's declared return type",
                    ret.range,
                );
            }
            Some(val_typed.knowledge)
        }
        Statement::Expr { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
            None
        }
        Statement::Throw { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
            None
        }
        Statement::Break { .. } => {
            ctx.record_break();
            ctx.flow.mark_unreachable();
            None
        }
        Statement::Continue { .. } => {
            ctx.record_continue();
            ctx.flow.mark_unreachable();
            None
        }
        Statement::Class(class_def) => {
            super::declaration::check_class(ctx, class_def);
            None
        }
        Statement::For(for_stmt) => {
            let mut lane_facts = Vec::new();
            for lane in &for_stmt.lanes {
                let iter_typed = analyze_expression(ctx, &lane.iter, &ExpectedType::None);
                let premise = CallPremise::from_typed(ctx, &iter_typed);
                let (elem_knowledge, iteration_causal_invalidity) = match &iter_typed.knowledge {
                    TypeKnowledge::Known(evidence) => {
                        let result = resolve_iteration_element_application(ctx, &premise, evidence.ty(), lane.iter.range());
                        if !result.status.is_ready() {
                            ctx.record_call_status(result.status.clone());
                        }
                        (result.knowledge, result.causal_invalidity)
                    }
                    TypeKnowledge::Unknown(reason) => (TypeKnowledge::Unknown(reason.clone()), crate::checker::causal::CausalInvalidity::Clean),
                    TypeKnowledge::Dynamic(reason) => (TypeKnowledge::Dynamic(reason.clone()), crate::checker::causal::CausalInvalidity::Clean),
                };

                let elem_fact = ValueSemanticFact::new(elem_knowledge);
                lane_facts.push((&lane.pattern, elem_fact, iter_typed.causal_invalidity.join(iteration_causal_invalidity)));
            }
            let before = ctx.flow.clone();
            ctx.push_loop_frame();
            ctx.push_scope();
            for (pat, fact, causal_invalidity) in lane_facts {
                bind_pattern(ctx, pat, fact, causal_invalidity);
            }
            for s in &for_stmt.body {
                check_statement(ctx, s);
            }
            let body_flow = ctx.flow.clone();
            ctx.pop_scope();
            let loop_frame = ctx.pop_loop_frame();
            let mut loop_states = vec![before, body_flow];
            loop_states.extend(loop_frame.continues);
            loop_states.extend(loop_frame.breaks);
            ctx.flow = match ctx.join_flow_states(&loop_states) {
                Ok(flow) => flow,
                Err(failure) => {
                    ctx.publish_flow_join_failure(failure, for_stmt.range);
                    return None;
                }
            };
            None
        }
        _ => None,
    }
}

fn bind_declaration_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    fact: ValueSemanticFact,
    contract: Option<BindingContract>,
    causal_invalidity: crate::checker::causal::CausalInvalidity,
    mutable: bool,
    range: phalcom_common::range::SourceRange,
) {
    match pattern {
        Pattern::Name { name, .. } => {
            ctx.declare_binding(BindingSeed {
                name: name.clone(),
                range,
                contract,
                current: fact.knowledge,
                denotation: fact.denotation,
                causal_invalidity,
                mutable,
            });
        }
        Pattern::Tuple { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                let component = ValueSemanticFact::new(crate::checker::composition::decompose_tuple_component(
                    ctx.store,
                    &fact.knowledge,
                    index,
                    elements.len(),
                ));
                bind_declaration_pattern(ctx, element, component, None, causal_invalidity, mutable, range);
            }
        }
        _ => {}
    }
}

fn bind_pattern(ctx: &mut CheckingContext<'_>, pattern: &Pattern, fact: ValueSemanticFact, causal_invalidity: crate::checker::causal::CausalInvalidity) {
    match pattern {
        Pattern::Name { name, range, .. } => {
            ctx.bind_pattern_binding_with_causal(name.clone(), fact, *range, causal_invalidity);
        }
        Pattern::Tuple { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                let component = ValueSemanticFact::new(crate::checker::composition::decompose_tuple_component(
                    ctx.store,
                    &fact.knowledge,
                    index,
                    elements.len(),
                ));
                bind_pattern(ctx, element, component, causal_invalidity);
            }
        }
        _ => {}
    }
}

/// Compatibility wrapper for callers that only need element knowledge.
pub fn resolve_iteration_element(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> TypeKnowledge {
    let premise = CallPremise::established(TypeKnowledge::established(receiver_ty, EvidenceOrigin::DeclarationSemantics));
    resolve_iteration_element_application(ctx, &premise, receiver_ty, SourceRange::default()).knowledge
}

fn resolve_iteration_element_application(
    ctx: &mut CheckingContext<'_>,
    premise: &CallPremise,
    receiver_ty: TypeId,
    call_range: SourceRange,
) -> super::call::CallCheckResult {
    // Parameterized protocol application has no modeled cursor expression here.
    if let Ok(selector) = Selector::method("iteratorValue", vec![SelectorSlot::Positional]) {
        match ctx.resolve_dispatch_target(receiver_ty, &selector, crate::dispatch::DispatchLookup::Normal) {
            crate::dispatch::ResolvedDispatchResult::Found(_) | crate::dispatch::ResolvedDispatchResult::Ambiguous(_) => {
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::IterationArgumentUnavailable);
            }
            crate::dispatch::ResolvedDispatchResult::Dynamic => {
                return analyze_unresolved_application(
                    ctx,
                    premise,
                    &[],
                    UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
                );
            }
            crate::dispatch::ResolvedDispatchResult::Missing { .. } => {}
        }
    }

    // A zero-argument protocol getter is a real callable application.
    if let Ok(selector) = Selector::getter("iteratorValue") {
        match ctx.resolve_dispatch_target(receiver_ty, &selector, crate::dispatch::DispatchLookup::Normal) {
            crate::dispatch::ResolvedDispatchResult::Found(resolved) => {
                let target = CallableApplicationTarget::from_dispatch(resolved);
                return apply_resolved_callable(ctx, &target, premise, &[], &ExpectedType::None, call_range);
            }
            crate::dispatch::ResolvedDispatchResult::Ambiguous(_) => {
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::DispatchAmbiguous);
            }
            crate::dispatch::ResolvedDispatchResult::Dynamic => {
                return analyze_unresolved_application(
                    ctx,
                    premise,
                    &[],
                    UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
                );
            }
            crate::dispatch::ResolvedDispatchResult::Missing { .. } => {}
        }
    }

    // `iterate(_)` is also parameterized; do not consume its return contract.
    if let Ok(selector) = Selector::method("iterate", vec![SelectorSlot::Positional]) {
        match ctx.resolve_dispatch_target(receiver_ty, &selector, crate::dispatch::DispatchLookup::Normal) {
            crate::dispatch::ResolvedDispatchResult::Found(_) | crate::dispatch::ResolvedDispatchResult::Ambiguous(_) => {
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::IterationArgumentUnavailable);
            }
            crate::dispatch::ResolvedDispatchResult::Dynamic => {
                return analyze_unresolved_application(
                    ctx,
                    premise,
                    &[],
                    UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection),
                );
            }
            crate::dispatch::ResolvedDispatchResult::Missing { .. } => {}
        }
    }

    analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::DispatchMissing)
}
