//! Statement type checking engine.

use super::analysis::AnalysisStatus;
use super::binding::{BindingContract, BindingContractOrigin, BindingSeed, reconcile_binding_relation};
use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::expression::{analyze_expression, synthesize_expr};
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{ContractAssumptionEligibility, DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};
use phalcom_ast::ast::{BindingKind, Pattern, Statement};
use phalcom_common::selector::Selector;

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
                if binding.kind == BindingKind::Const {
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ConstWithoutInitializer,
                        "const binding requires an initializer",
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
                causal_invalidity = causal_invalidity.join(crate::checker::causal::CausalInvalidity::One(cause));
                val_typed.status = AnalysisStatus::Invalid(cause);
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
            if let Some(expression_id) = val_typed.expression_id {
                if let Some(analysis) = ctx.expressions.get_mut(&expression_id) {
                    analysis.status = val_typed.status.clone();
                    analysis.causal_invalidity = val_typed.causal_invalidity;
                }
            }

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
                let iter_k = synthesize_expr(ctx, &lane.iter);
                let elem_knowledge = if let Some(iter_ty) = iter_k.ty() {
                    resolve_iteration_element(ctx, iter_ty)
                } else if iter_k.is_dynamic() {
                    TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::RuntimeReflection)
                } else {
                    TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration)
                };

                let elem_fact = ValueSemanticFact::new(elem_knowledge);
                lane_facts.push((&lane.pattern, elem_fact));
            }
            let before = ctx.flow.clone();
            ctx.push_loop_frame();
            ctx.push_scope();
            for (pat, fact) in lane_facts {
                bind_pattern(ctx, pat, fact);
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
            let component_facts = match fact.knowledge {
                TypeKnowledge::Known(evidence) => match ctx.store.get(evidence.ty()) {
                    crate::types::store::TypeData::Tuple(types) if types.len() == elements.len() => Some(
                        types
                            .iter()
                            .map(|element| {
                                ValueSemanticFact::new(
                                    TypeKnowledge::Known(evidence.clone()).derive_known_type(element.ty, EvidenceOrigin::PatternDecomposition),
                                )
                            })
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                },
                _ => None,
            };
            for (index, element) in elements.iter().enumerate() {
                let component = component_facts
                    .as_ref()
                    .and_then(|facts| facts.get(index))
                    .cloned()
                    .unwrap_or_else(|| ValueSemanticFact::new(TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)));
                bind_declaration_pattern(ctx, element, component, None, causal_invalidity, mutable, range);
            }
        }
        _ => {}
    }
}

fn bind_pattern(ctx: &mut CheckingContext<'_>, pattern: &Pattern, fact: ValueSemanticFact) {
    match pattern {
        Pattern::Name { name, range, .. } => {
            ctx.bind_pattern_binding(name.clone(), fact, *range);
        }
        Pattern::Tuple { elements, .. } => {
            let component_facts = match fact.knowledge {
                TypeKnowledge::Known(evidence) => match ctx.store.get(evidence.ty()) {
                    crate::types::store::TypeData::Tuple(types) if types.len() == elements.len() => Some(
                        types
                            .iter()
                            .map(|element| {
                                ValueSemanticFact::new(
                                    TypeKnowledge::Known(evidence.clone()).derive_known_type(element.ty, EvidenceOrigin::PatternDecomposition),
                                )
                            })
                            .collect::<Vec<_>>(),
                    ),
                    _ => None,
                },
                _ => None,
            };
            for (index, element) in elements.iter().enumerate() {
                let component = component_facts
                    .as_ref()
                    .and_then(|facts| facts.get(index))
                    .cloned()
                    .unwrap_or_else(|| ValueSemanticFact::new(TypeKnowledge::Unknown(UnknownReason::NoTypeEvidence)));
                bind_pattern(ctx, element, component);
            }
        }
        _ => {}
    }
}

/// Derives the iteration element type from the receiver via the `iterate(_)` / `iteratorValue(_)` protocol (F5 / DEC-IMPL-FOR-PROTOCOL-ONLY).
pub fn resolve_iteration_element(ctx: &mut CheckingContext<'_>, receiver_ty: TypeId) -> TypeKnowledge {
    // 1. Try 1-argument protocol selector `iteratorValue(_)`
    if let Ok(sel_1) = Selector::method("iteratorValue", vec![phalcom_common::selector::SelectorSlot::Positional]) {
        let dispatch_res = ctx.resolve_dispatch(receiver_ty, &sel_1, crate::dispatch::DispatchLookup::Normal);
        match dispatch_res {
            crate::dispatch::DispatchResult::Found(sig) => {
                if !sig.return_type.is_unknown() {
                    return sig.return_type;
                }
            }
            crate::dispatch::DispatchResult::Dynamic => {
                return TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::RuntimeReflection);
            }
            _ => {}
        }
    }

    // 2. Try 0-argument selector / getter `iteratorValue`
    if let Ok(sel_0) = Selector::method("iteratorValue", vec![]) {
        let dispatch_res = ctx.resolve_dispatch(receiver_ty, &sel_0, crate::dispatch::DispatchLookup::Normal);
        match dispatch_res {
            crate::dispatch::DispatchResult::Found(sig) => {
                if !sig.return_type.is_unknown() {
                    return sig.return_type;
                }
            }
            crate::dispatch::DispatchResult::Dynamic => {
                return TypeKnowledge::Dynamic(crate::types::evidence::DynamicReason::RuntimeReflection);
            }
            _ => {}
        }
    }

    // 3. Check iterate(_) protocol existence. Presence without a resolved
    // element result is not enough to invent one from generic arguments.
    if let Ok(iterate_sel) = Selector::method("iterate", vec![phalcom_common::selector::SelectorSlot::Positional]) {
        let dispatch_res = ctx.resolve_dispatch(receiver_ty, &iterate_sel, crate::dispatch::DispatchLookup::Normal);
        if dispatch_res.is_found() {
            return TypeKnowledge::Unknown(UnknownReason::UnannotatedDeclaration);
        }
    }

    TypeKnowledge::Unknown(UnknownReason::DynamicMessageSend)
}
