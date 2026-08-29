//! Statement type checking engine.

use super::analysis::{AnalysisStatus, NormalReturnFact};
use super::binding::{BindingContract, BindingContractOrigin, BindingSeed, reconcile_binding_relation};
use super::causal::CausalInvalidity;
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
/// diagnostics. A direct `return` reports its structured normal-return fact to
/// callable-body analysis; all other statement forms return `None`.
pub fn check_statement(ctx: &mut CheckingContext<'_>, statement: &Statement) -> Option<NormalReturnFact> {
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
            let relation_explanation = contract.as_ref().map(|contract| {
                let parents = val_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id)).into_iter().collect();
                ctx.record_type_relation_with_parents(&val_typed.knowledge, contract.ty, &relation, binding.range, parents)
            });
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
                if let (Some(callable), Some(explanation)) = (ctx.current_callable.clone(), relation_explanation) {
                    diag = diag.with_explanation(crate::diagnostic::ExplanationRef::new(callable, explanation));
                }
                if let (Some(annotation), TypeKnowledge::Known(evidence)) = (&binding.annotation, &val_typed.knowledge) {
                    if evidence.status() == crate::types::evidence::EvidenceStatus::Established {
                        diag = diag.with_guidance(crate::diagnostic::DiagnosticGuidance::ChangeAnnotation {
                            range: annotation.range,
                            ty: evidence.ty(),
                        });
                    }
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
                val_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id)),
            );
            None
        }
        Statement::Return(ret) => {
            let expected_ret = ctx
                .expected_return
                .as_ref()
                .map(|contract| ExpectedType::proper_from(contract.ty, ExpectationOrigin::ReturnContract))
                .unwrap_or_default();
            let mut val_typed = if let Some(expr) = &ret.value {
                analyze_expression(ctx, expr, &expected_ret)
            } else {
                TypedExpression::established(ctx.store.unit(), EvidenceOrigin::DeclarationSemantics, ret.range)
            };

            let relation = if let Some(expected) = ctx.expected_return.clone() {
                Some(ctx.apply_knowledge_against_type(
                    &val_typed.knowledge,
                    expected.ty,
                    DiagnosticCode::ReturnMismatch,
                    "returned value is not assignable to method's declared return type",
                    ret.range,
                ))
            } else {
                None
            };
            if let Some(relation_application) = &relation {
                if let Some(cause) = relation_application.cause {
                    val_typed.status = AnalysisStatus::Invalid(cause);
                    val_typed.causal_invalidity = val_typed.causal_invalidity.join(CausalInvalidity::One(cause));
                } else {
                    val_typed.status = match &relation_application.outcome {
                        RelationOutcome::Blocked(reason) => AnalysisStatus::Blocked(reason.clone()),
                        RelationOutcome::Cancelled => AnalysisStatus::Cancelled,
                        RelationOutcome::BudgetExceeded(report) => AnalysisStatus::BudgetExceeded(report.clone()),
                        RelationOutcome::InternalFailure(message) => AnalysisStatus::InternalFailure(ctx.publish_analysis_incident(message)),
                        RelationOutcome::DynamicBoundary(_) => AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection),
                        _ => val_typed.status.clone(),
                    };
                }
            }
            ctx.sync_expression_outcome(&val_typed);

            let mut parents = val_typed
                .expression_id
                .and_then(|id| ctx.explanation_for_expression(id))
                .into_iter()
                .collect::<Vec<_>>();
            if let Some(explanation) = relation.as_ref().and_then(|application| application.explanation) {
                parents.push(explanation);
            }
            let return_explanation = ctx.record_derivation(
                crate::explain::ExplanationStep::ReturnCheck {
                    actual: val_typed.knowledge.clone(),
                    expected: ctx.expected_return.as_ref().map(|contract| contract.ty),
                },
                crate::explain::DerivationRule::ReturnTypeCheck,
                val_typed.knowledge.status().unwrap_or(crate::types::evidence::EvidenceStatus::Assumed),
                val_typed.knowledge.origin().unwrap_or(EvidenceOrigin::Flow),
                Vec::new(),
                parents,
            );
            if let Some(cause) = relation.as_ref().and_then(|application| application.cause) {
                ctx.attach_explanation_to_cause(cause, return_explanation);
            }
            ctx.record_call_dependency(val_typed.causal_invalidity, Some(return_explanation));
            let fact = NormalReturnFact {
                knowledge: val_typed.knowledge,
                flow: ctx.current_flow_summary(),
                status: val_typed.status,
                causal_invalidity: val_typed.causal_invalidity,
            };
            Some(fact)
        }
        Statement::Expr { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
            None
        }
        Statement::Throw { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
            ctx.record_throw_exit();
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

                let mut parents = iter_typed
                    .expression_id
                    .and_then(|id| ctx.explanation_for_expression(id))
                    .into_iter()
                    .collect::<Vec<_>>();
                let iteration = ctx.record_derivation(
                    crate::explain::ExplanationStep::IterationElement {
                        iterable: iter_typed.knowledge.clone(),
                        element: elem_knowledge.clone(),
                        callable: ctx.resolved_callable_for_current_expression(),
                    },
                    crate::explain::DerivationRule::IterationElementResolution,
                    elem_knowledge.status().unwrap_or(crate::types::evidence::EvidenceStatus::Assumed),
                    elem_knowledge.origin().unwrap_or(EvidenceOrigin::Flow),
                    Vec::new(),
                    std::mem::take(&mut parents),
                );
                let elem_fact = ValueSemanticFact::new(elem_knowledge);
                lane_facts.push((
                    &lane.pattern,
                    elem_fact,
                    iter_typed.causal_invalidity.join(iteration_causal_invalidity),
                    Some(iteration),
                ));
            }
            let before = ctx.flow.clone();
            ctx.push_loop_frame();
            ctx.push_scope();
            for (pat, fact, causal_invalidity, explanation) in lane_facts {
                bind_pattern(ctx, pat, fact, causal_invalidity, explanation);
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
    explanation: Option<crate::identity::ExplanationId>,
) {
    match pattern {
        Pattern::Name { name, range: name_range } => {
            let has_contract = contract.is_some();
            let result = ctx.declare_binding(BindingSeed {
                parameter: None,
                name: name.clone(),
                range: *name_range,
                contract,
                current: fact.knowledge,
                denotation: fact.denotation,
                causal_invalidity,
                mutable,
            });
            if !has_contract {
                if let (crate::checker::binding::BindingDeclarationResult::Inserted(binding), Some(explanation)) = (result, explanation) {
                    ctx.flow.set_binding_explanation(binding, explanation);
                }
            }
        }
        Pattern::Tuple { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                let knowledge = crate::checker::composition::decompose_tuple_component(ctx.store, &fact.knowledge, index, elements.len());
                let component_explanation = explanation.map(|source| {
                    ctx.record_derivation(
                        crate::explain::ExplanationStep::ProductComponent {
                            source,
                            index,
                            result: knowledge.clone(),
                        },
                        crate::explain::DerivationRule::ProductDecomposition,
                        knowledge.status().unwrap_or(crate::types::evidence::EvidenceStatus::Assumed),
                        knowledge.origin().unwrap_or(EvidenceOrigin::Flow),
                        Vec::new(),
                        vec![source],
                    )
                });
                let component = ValueSemanticFact::new(knowledge);
                bind_declaration_pattern(ctx, element, component, None, causal_invalidity, mutable, range, component_explanation);
            }
        }
        Pattern::List { elements, rest, .. } => {
            let list_origin = ctx.resolve_type_name("List").map(|decl| ctx.nominal_type_of(&decl));
            let element_knowledge = list_origin
                .map(|origin| crate::checker::composition::decompose_list_element(ctx.store, &fact.knowledge, origin))
                .unwrap_or_else(|| fact.knowledge.clone());
            for element in elements {
                bind_declaration_pattern(
                    ctx,
                    element,
                    ValueSemanticFact::new(element_knowledge.clone()),
                    None,
                    causal_invalidity,
                    mutable,
                    range,
                    explanation,
                );
            }
            if let Some(rest) = rest {
                let rest_knowledge = list_origin
                    .map(|origin| crate::checker::composition::decompose_list_rest(ctx.store, &fact.knowledge, origin))
                    .unwrap_or_else(|| fact.knowledge.clone());
                bind_declaration_pattern(
                    ctx,
                    rest,
                    ValueSemanticFact::new(rest_knowledge),
                    None,
                    causal_invalidity,
                    mutable,
                    range,
                    explanation,
                );
            }
        }
        _ => {}
    }
}

fn bind_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    fact: ValueSemanticFact,
    causal_invalidity: crate::checker::causal::CausalInvalidity,
    explanation: Option<crate::identity::ExplanationId>,
) {
    match pattern {
        Pattern::Name { name, range, .. } => {
            ctx.bind_pattern_binding_with_causal(name.clone(), fact, *range, causal_invalidity);
            if let (Some(binding), Some(explanation)) = (ctx.lookup_binding_info(name).map(|info| info.id), explanation) {
                ctx.flow.set_binding_explanation(binding, explanation);
            }
        }
        Pattern::Tuple { elements, .. } => {
            for (index, element) in elements.iter().enumerate() {
                let component = ValueSemanticFact::new(crate::checker::composition::decompose_tuple_component(
                    ctx.store,
                    &fact.knowledge,
                    index,
                    elements.len(),
                ));
                bind_pattern(ctx, element, component, causal_invalidity, explanation);
            }
        }
        Pattern::List { elements, rest, .. } => {
            let list_origin = ctx.resolve_type_name("List").map(|decl| ctx.nominal_type_of(&decl));
            let element_knowledge = list_origin
                .map(|origin| crate::checker::composition::decompose_list_element(ctx.store, &fact.knowledge, origin))
                .unwrap_or_else(|| fact.knowledge.clone());
            for element in elements {
                bind_pattern(ctx, element, ValueSemanticFact::new(element_knowledge.clone()), causal_invalidity, explanation);
            }
            if let Some(rest) = rest {
                let rest_knowledge = list_origin
                    .map(|origin| crate::checker::composition::decompose_list_rest(ctx.store, &fact.knowledge, origin))
                    .unwrap_or_else(|| fact.knowledge.clone());
                bind_pattern(ctx, rest, ValueSemanticFact::new(rest_knowledge), causal_invalidity, explanation);
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
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection));
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
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection));
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
                return analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::PremiseDynamic(DynamicReason::RuntimeReflection));
            }
            crate::dispatch::ResolvedDispatchResult::Missing { .. } => {}
        }
    }

    analyze_unresolved_application(ctx, premise, &[], UnresolvedApplicationReason::DispatchMissing)
}
