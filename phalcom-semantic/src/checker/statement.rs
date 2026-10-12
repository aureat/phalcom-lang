//! Statement type checking engine.

use super::analysis::AnalysisStatus;
use super::binding::{BindingContract, BindingContractOrigin, BindingSeed, reconcile_binding_relation};
use super::call::CallPremise;
use super::causal::CausalInvalidity;
use super::context::CheckingContext;
use super::control::StatementControl;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::expression::analyze_expression;
use super::flow::FlowState;
use super::iteration::resolve_for_iteration_lane;
use super::typed_expr::TypedExpression;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::types::denotation::ValueSemanticFact;
use crate::types::evidence::{ContractAssumptionEligibility, DynamicReason, EvidenceOrigin, TypeKnowledge, UnknownReason};
use crate::types::outcome::{DynamicBoundaryObligation, RelationOutcome};
use crate::types::rigid::LocalType;
use crate::types::store::TypeData;
use phalcom_ast::ast::{BindingKind, Pattern, Statement};

/// Checks a single statement, updating context bindings, recording exits, and
/// reporting the resulting [`StatementControl`].
pub fn check_statement(ctx: &mut CheckingContext<'_>, statement: &Statement) -> StatementControl {
    match statement {
        Statement::Let(binding) => {
            // `const` initializers are checked in the static phase. Keep that
            // phase visible to nested expression synthesis so phase-owned
            // defaults (including bare sequence literals) agree with the
            // canonical static-constant product. Runtime `let` bindings keep
            // the caller's phase, which also preserves static-definition
            // locals checked within an already-static body.
            let previous_phase = ctx.phase;
            if matches!(binding.kind, BindingKind::Const) {
                ctx.phase = super::context::EvaluationPhase::Static;
            }

            // A module static-constant owner forms annotations containing
            // computed Record keys only after its selected constants have
            // reduced. Resolving such an annotation in this generic pass
            // would create a competing literal-only interpretation before the
            // owner can supply its checked projection facts.
            let defer_computed_record_key_annotation = matches!(binding.kind, BindingKind::Const)
                && binding
                    .annotation
                    .as_ref()
                    .is_some_and(|annotation| !crate::static_key::type_record_key_expressions(annotation, 0).is_empty());

            let resolver = ctx.resolver.clone();
            let (elaboration, mut annotation_invalidity) = if !defer_computed_record_key_annotation {
                if let Some(annotation) = binding.annotation.as_ref() {
                    ctx.elaborate_type_annotation(&resolver, annotation)
                } else {
                    (None, crate::checker::causal::CausalInvalidity::Clean)
                }
            } else {
                (None, crate::checker::causal::CausalInvalidity::Clean)
            };

            let expected_init = match &elaboration {
                Some(crate::checker::annotation_elaboration::AnnotationElaboration::Canonical(ty)) => {
                    ExpectedType::proper_from(*ty, ExpectationOrigin::DeclarationContract)
                }
                Some(crate::checker::annotation_elaboration::AnnotationElaboration::Inference { context, term, .. }) => {
                    ExpectedType::inference_from(*context, term.clone(), ExpectationOrigin::DeclarationContract)
                }
                None => ExpectedType::None,
            };

            let (mut val_typed, init_state) = if let Some(expr) = &binding.value {
                (
                    analyze_expression(ctx, expr, &expected_init),
                    crate::checker::flow::InitializationState::DefinitelyInitialized,
                )
            } else {
                if binding.kind.requires_initializer() || !matches!(binding.pattern, Pattern::Name { .. }) {
                    let message = if matches!(binding.pattern, Pattern::Name { .. }) {
                        match binding.kind {
                            BindingKind::Const => "const binding requires an initializer",
                            BindingKind::Let => "let binding requires an initializer",
                            BindingKind::LetMut => "uninitialized binding requires an initializer",
                        }
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
                (
                    TypedExpression::unknown(UnknownReason::MissingInitializer),
                    crate::checker::flow::InitializationState::Uninitialized,
                )
            };
            ctx.phase = previous_phase;

            let annotation_owner = match &elaboration {
                Some(crate::checker::annotation_elaboration::AnnotationElaboration::Inference { context, frame, .. }) => {
                    Some(super::sequence_publication::PublicationOwner(*context, *frame))
                }
                _ => None,
            };
            let declared_k = match elaboration {
                Some(crate::checker::annotation_elaboration::AnnotationElaboration::Canonical(ty)) => {
                    Some(TypeKnowledge::established(ty, EvidenceOrigin::ContextualDerivation))
                }
                Some(crate::checker::annotation_elaboration::AnnotationElaboration::Inference { context, frame, term, holes }) => {
                    let outcome = if let Some(session) = ctx.inference_session(context) {
                        ctx.solve_inference_in_frame(&mut session.borrow_mut(), frame)
                    } else {
                        crate::checker::inference::InferenceOutcome::Cancelled
                    };

                    let solved_ty = if outcome.is_solved() {
                        if let Some(session) = ctx.inference_session(context) {
                            session.borrow().materialize(&term, ctx.store).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(ty) = solved_ty {
                        Some(TypeKnowledge::established(ty, EvidenceOrigin::ContextualDerivation))
                    } else {
                        for hole in holes.iter() {
                            let diag_code = match &outcome {
                                crate::checker::inference::InferenceOutcome::Conflicting(_) => DiagnosticCode::GenericInferenceConflict,
                                _ => DiagnosticCode::GenericInferenceUnderconstrained,
                            };
                            let msg = match &outcome {
                                crate::checker::inference::InferenceOutcome::Conflicting(_) => {
                                    "conflicting type constraints for inference hole `_`".to_string()
                                }
                                _ => {
                                    "inference hole `_` could not be solved; insufficient type evidence from initializer".to_string()
                                }
                            };
                            ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                diag_code,
                                msg,
                                hole.range,
                            ));
                        }
                        let cause = ctx.alloc_diagnostic_cause();
                        annotation_invalidity = annotation_invalidity.join(crate::checker::causal::CausalInvalidity::One(cause));
                        None
                    }
                }
                None => None,
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
                if let (Some(expected), TypeKnowledge::Known(actual)) = (contract.as_ref().map(|contract| contract.ty), &val_typed.knowledge) {
                    let expected_is_canonical_map = ctx
                        .store
                        .applied_nominal_static_parts(expected)
                        .is_some_and(|(declaration, arguments)| declaration == ctx.core_ids.map && arguments.len() == 2);
                    if expected_is_canonical_map && matches!(ctx.store.get(actual.ty()), TypeData::Record(_)) {
                        diag = diag.with_help("use `Map { ... }` for runtime-keyed associations; `{ ... }` constructs a structural Record");
                    }
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
                if !matches!(val_typed.status, AnalysisStatus::Invalid(_)) {
                    val_typed.status = match relation {
                        RelationOutcome::Blocked(reason) => AnalysisStatus::Blocked(reason),
                        RelationOutcome::Cancelled => AnalysisStatus::Cancelled,
                        RelationOutcome::BudgetExceeded(report) => AnalysisStatus::BudgetExceeded(report),
                        RelationOutcome::InternalFailure(message) => AnalysisStatus::InternalFailure(ctx.publish_analysis_incident(message)),
                        RelationOutcome::DynamicBoundary(_) => AnalysisStatus::DynamicBoundary(DynamicReason::RuntimeReflection),
                        _ => val_typed.status.clone(),
                    };
                }
            }
            ctx.sync_expression_outcome(&val_typed);
            if let Some(owner) = annotation_owner {
                if declared_k.is_some() && val_typed.status.is_ready() && matches!(causal_invalidity, CausalInvalidity::Clean) {
                    if let Err(status) = ctx.accept_sequence_owner(owner) {
                        val_typed.status = status;
                        ctx.sync_expression_outcome(&val_typed);
                    }
                } else {
                    ctx.reject_sequence_owner(owner);
                }
                ctx.pop_inference_frame(owner.0, owner.1);
                ctx.finish_inference_context(owner.0);
            }

            validate_declaration_pattern(ctx, &binding.pattern);
            bind_declaration_pattern(
                ctx,
                &binding.pattern,
                ValueSemanticFact {
                    knowledge: reconciliation.current,
                    denotation: val_typed.denotation,
                },
                contract,
                causal_invalidity,
                binding.kind.is_mutable(),
                binding.kind.phase(),
                init_state,
                binding.range,
                val_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id)),
                val_typed.local_type.clone(),
            );
            StatementControl::FallsThrough
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

            if !ctx.check_local_type_escape(
                val_typed.local_type.as_ref(),
                ctx.expected_return.as_ref().map(|contract| contract.ty),
                &[],
                ret.range,
            ) {
                val_typed.knowledge = TypeKnowledge::Unknown(UnknownReason::ExistentialEscape);
                val_typed.local_type = None;
            }

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
            if let Some(expected_kind) = ctx.expected_kinded_return {
                let actual_kind = match val_typed.denotation.as_ref() {
                    Some(crate::types::denotation::SemanticDenotation::TypeForm(form) | crate::types::denotation::SemanticDenotation::StaticTerm(form)) => {
                        match ctx.store.get(*form) {
                            crate::types::store::TypeData::Lambda(lambda) => Some(ctx.store.arena().get_lambda(*lambda).result_kind),
                            _ => Some(ctx.store.kind_of(*form)),
                        }
                    }
                    Some(crate::types::denotation::SemanticDenotation::StaticApplication(application)) => ctx
                        .checked_static_applications
                        .iter()
                        .find(|candidate| candidate.id == *application)
                        .and_then(|candidate| candidate.signature.result_domain.as_kind()),
                    _ => None,
                };
                if actual_kind != Some(expected_kind) {
                    let actual = actual_kind.map_or_else(|| "a non-kinded value".to_string(), |kind| ctx.store.format_kind(kind));
                    let expected = ctx.store.format_kind(expected_kind);
                    if let Some(cause) = ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::TypeMismatch,
                        format!("returned static term has kind {actual}, expected {expected}"),
                        ret.range,
                    )) {
                        val_typed.invalidate(cause);
                    }
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
            let fact = crate::checker::analysis::NormalReturnFact {
                direct_result_dependency: ret
                    .value
                    .as_ref()
                    .and_then(|expr| crate::checker::analysis::direct_result_dependency(expr, val_typed.callable.as_ref())),
                knowledge: val_typed.knowledge,
                flow: ctx.current_flow_summary(),
                status: val_typed.status,
                causal_invalidity: val_typed.causal_invalidity,
            };
            ctx.record_return_exit(fact);
            StatementControl::Return
        }
        Statement::Expr { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
            StatementControl::FallsThrough
        }
        Statement::Throw { expr, .. } => {
            analyze_expression(ctx, expr, &ExpectedType::None);
            ctx.record_throw_exit_and_terminate();
            StatementControl::Throw
        }
        Statement::Break { .. } => {
            ctx.record_break_and_terminate();
            StatementControl::Break
        }
        Statement::Continue { .. } => {
            ctx.record_continue_and_terminate();
            StatementControl::Continue
        }
        Statement::Class(class_def) => {
            super::declaration::check_class(ctx, class_def);
            StatementControl::FallsThrough
        }
        Statement::Data(_data_def) => StatementControl::FallsThrough,
        Statement::For(for_stmt) => {
            let mut lane_facts = Vec::new();
            for lane in &for_stmt.lanes {
                let iter_typed = analyze_expression(ctx, &lane.iter, &ExpectedType::None);
                let (elem_knowledge, iteration_causal_invalidity) = match ctx.phase {
                    crate::checker::context::EvaluationPhase::Runtime => match &iter_typed.knowledge {
                        TypeKnowledge::Known(evidence) => {
                            let premise = CallPremise::from_typed(ctx, &iter_typed);
                            let (resolution, result) = resolve_for_iteration_lane(ctx, &premise, iter_typed.expression_id, evidence.ty(), lane.iter.range());
                            if let Some(resolution) = resolution {
                                ctx.for_iteration_lanes.insert(resolution.source_expression, resolution);
                            }
                            if !result.status.is_ready() {
                                ctx.record_call_status(result.status.clone());
                            }
                            (result.knowledge, result.causal_invalidity)
                        }
                        TypeKnowledge::Unknown(reason) => (TypeKnowledge::Unknown(reason.clone()), crate::checker::causal::CausalInvalidity::Clean),
                        TypeKnowledge::Dynamic(reason) => (TypeKnowledge::Dynamic(reason.clone()), crate::checker::causal::CausalInvalidity::Clean),
                    },
                    crate::checker::context::EvaluationPhase::Static => {
                        let Some(source_expr_id) = iter_typed.expression_id else {
                            continue;
                        };
                        let resolution = crate::checker::static_iteration::resolve_static_iteration_lane(ctx, source_expr_id, lane.iter.range(), &iter_typed);
                        if let Some(resolution) = resolution {
                            let elem_knowledge = resolution.item_domain.consolidated_item_knowledge(ctx.store);
                            ctx.static_iteration_lanes.insert(resolution.source_expression, resolution);
                            (elem_knowledge, crate::checker::causal::CausalInvalidity::Clean)
                        } else {
                            (TypeKnowledge::Unknown(UnknownReason::SuppressedByInvalidCause), iter_typed.causal_invalidity)
                        }
                    }
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
                    lane.index.as_ref(),
                ));
            }
            let before = ctx.flow.clone();

            let evaluate_step = |step_ctx: &mut CheckingContext<'_>,
                                 current_header: &FlowState,
                                 is_real_pass: bool|
             -> (Option<FlowState>, FlowState, Vec<FlowState>, crate::checker::causal::CausalInvalidity) {
                step_ctx.flow = current_header.clone();
                let exit_flow = step_ctx.flow.clone();

                step_ctx.push_loop_frame();
                let body_res = super::control::analyze_executable_region_with_prelude(step_ctx, &for_stmt.body, for_stmt.range, &ExpectedType::None, |ctx| {
                    for (pat, fact, causal, explanation, index_binding) in &lane_facts {
                        if let Some(index) = index_binding {
                            let int_ty = ctx.core_type(&ctx.core_ids.int.clone()).unwrap_or_else(|| ctx.store.unit());
                            ctx.bind_pattern_binding_with_causal(
                                index.name.clone(),
                                ValueSemanticFact::new(TypeKnowledge::established(int_ty, EvidenceOrigin::Syntax)),
                                index.range,
                                crate::checker::causal::CausalInvalidity::Clean,
                            );
                        }
                        let elem_ty = fact.knowledge.ty().unwrap_or_else(|| ctx.store.unit());
                        let subject = crate::checker::coverage::CoverageSubject::from_parts(elem_ty, crate::types::LocalType::Canonical(elem_ty));
                        let mut engine = crate::checker::coverage::CoverageEngine::new(subject.clone(), ctx.control.clone());
                        let mut arm_bindings = Vec::new();
                        let pattern_res = crate::checker::pattern::resolve_pattern(ctx, pat, &subject, &mut arm_bindings);
                        if let Some(reason) = crate::checker::pattern::blocked_reason_for_resolution(&pattern_res) {
                            engine.retain_blocked(reason.clone());
                        }
                        let coverage_pat = crate::checker::pattern::coverage_pattern_for_resolution(engine.arena_mut(), &pattern_res);
                        let usefulness = engine.classify_arm(ctx.declarations, ctx.store, &ctx.hierarchy, &mut ctx.rigids, ctx.enum_table, coverage_pat);
                        if is_real_pass && usefulness == crate::match_semantics::PatternUsefulness::Impossible {
                            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                                ctx.current_module.clone(),
                                crate::diagnostic::DiagnosticCode::MatchPatternImpossible,
                                "impossible match pattern: it cannot match any value of the target type",
                                pat.range(),
                            ));
                        }
                        for b in arm_bindings.iter() {
                            ctx.attach_causal_invalidity_to_binding(b.binding, *causal);
                            if let Some(exp) = *explanation {
                                ctx.flow.set_binding_explanation(b.binding, exp);
                            }
                        }
                        if is_real_pass {
                            ctx.record_pattern_site_resolution(crate::match_semantics::PatternSiteResolution {
                                kind: crate::match_semantics::PatternSiteKind::ForLane,
                                range: pat.range(),
                                subject: fact.knowledge.clone(),
                                pattern: pattern_res,
                                bindings: arm_bindings.into_boxed_slice(),
                                usefulness,
                            });
                        }
                    }
                });
                let body_flow = step_ctx.flow.clone();
                let loop_frame = step_ctx.pop_loop_frame();
                let normal_backedge = if body_res.completes_normally() { Some(body_flow) } else { None };
                let mut continues = loop_frame.continues;
                let breaks = loop_frame.breaks;
                continues.extend(normal_backedge);
                let backedge = if continues.is_empty() {
                    None
                } else if continues.len() == 1 {
                    continues.pop()
                } else {
                    step_ctx.join_flow_states(&continues).ok()
                };

                (backedge, exit_flow, breaks, body_res.causal_invalidity)
            };

            let fixpoint = match super::loop_analysis::solve_loop_header(ctx, &before, |probe_ctx, current_header| {
                let (backedge, _exit_flow, breaks, _causal) = evaluate_step(probe_ctx, current_header, false);
                super::loop_analysis::LoopStepResult {
                    normal_backedge: backedge,
                    continues: Vec::new(),
                    breaks,
                }
            }) {
                Ok(fp) => fp,
                Err(failure) => {
                    ctx.publish_flow_join_failure(failure, for_stmt.range);
                    return StatementControl::FallsThrough;
                }
            };

            // Final real pass at stable header
            let (_backedge, exit_flow, breaks, _causal) = evaluate_step(ctx, &fixpoint.header, true);

            let mut exit_states = Vec::new();
            if exit_flow.is_reachable() {
                exit_states.push(exit_flow);
            }
            for brk in breaks {
                if brk.is_reachable() {
                    exit_states.push(brk);
                }
            }

            ctx.flow = if exit_states.is_empty() {
                FlowState::unreachable()
            } else if exit_states.len() == 1 {
                exit_states.pop().unwrap()
            } else {
                match ctx.join_flow_states(&exit_states) {
                    Ok(flow) => flow,
                    Err(failure) => {
                        ctx.publish_flow_join_failure(failure, for_stmt.range);
                        return StatementControl::FallsThrough;
                    }
                }
            };

            StatementControl::FallsThrough
        }
        _ => StatementControl::FallsThrough,
    }
}

#[allow(clippy::too_many_arguments, clippy::only_used_in_recursion)]
fn bind_declaration_pattern(
    ctx: &mut CheckingContext<'_>,
    pattern: &Pattern,
    fact: ValueSemanticFact,
    contract: Option<BindingContract>,
    causal_invalidity: crate::checker::causal::CausalInvalidity,
    mutable: bool,
    phase: phalcom_ast::ast::BindingPhase,
    initialization: crate::checker::flow::InitializationState,
    range: phalcom_common::range::SourceRange,
    explanation: Option<crate::identity::ExplanationId>,
    local_type: Option<LocalType>,
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
                initialization,
            });
            let (binding_id, inserted) = match result {
                crate::checker::binding::BindingDeclarationResult::Inserted(binding) => (binding, true),
                crate::checker::binding::BindingDeclarationResult::Redeclared(binding) => (binding, false),
            };
            if inserted {
                ctx.record_binding_phase(binding_id, phase);
            }
            if let Some(local_type) = local_type {
                ctx.set_local_binding_type(binding_id, local_type);
            }
            if !has_contract && inserted {
                if let Some(explanation) = explanation {
                    ctx.flow.set_binding_explanation(binding_id, explanation);
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
                bind_declaration_pattern(
                    ctx,
                    element,
                    component,
                    None,
                    causal_invalidity,
                    mutable,
                    phase,
                    initialization,
                    range,
                    component_explanation,
                    local_type.as_ref().and_then(|local| match local {
                        LocalType::Tuple(elements) => elements.get(index).map(|element| element.ty.clone()),
                        _ => None,
                    }),
                );
            }
        }
        Pattern::List { elements, rest, .. } => {
            let list_origin = ctx.core_type(&ctx.core_ids.list.clone());
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
                    phase,
                    initialization,
                    range,
                    explanation,
                    local_type.as_ref().and_then(local_list_element),
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
                    phase,
                    initialization,
                    range,
                    explanation,
                    local_type.clone(),
                );
            }
        }
        Pattern::Record { entries, rest, .. } => {
            let source_row = fact.knowledge.ty().and_then(|source| match ctx.store.get(source) {
                crate::types::store::TypeData::Record(row_id) => Some(ctx.store.record_row(*row_id).clone()),
                _ => None,
            });
            for entry in entries {
                let field_knowledge = source_row
                    .as_ref()
                    .and_then(|row| entry.label().and_then(|label| row.find_field(&crate::field_key::FieldKey::symbol(label))))
                    .map(|field_ty| fact.knowledge.derive_known_type(field_ty, EvidenceOrigin::PatternDecomposition))
                    .unwrap_or_else(|| match &fact.knowledge {
                        TypeKnowledge::Unknown(reason) => TypeKnowledge::Unknown(reason.clone()),
                        TypeKnowledge::Dynamic(reason) => TypeKnowledge::Dynamic(reason.clone()),
                        TypeKnowledge::Known(_) => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
                    });
                bind_declaration_pattern(
                    ctx,
                    &entry.pattern,
                    ValueSemanticFact::new(field_knowledge),
                    None,
                    causal_invalidity,
                    mutable,
                    phase,
                    initialization,
                    range,
                    explanation,
                    None,
                );
            }
            if let Some(rest_pattern) = rest {
                let rest_knowledge = if let Some(row) = source_row {
                    let remainder_fields = row
                        .fields
                        .into_iter()
                        .filter(|field| {
                            !entries
                                .iter()
                                .any(|entry| entry.label().is_some_and(|label| field.key == crate::field_key::FieldKey::symbol(label)))
                        })
                        .collect();
                    match ctx.store.record_row_checked(remainder_fields, row.tail) {
                        Ok(row_id) => {
                            let rest_ty = ctx
                                .store
                                .intern_with_kind(crate::types::store::TypeData::Record(row_id), crate::types::id::KindId::TYPE);
                            fact.knowledge.derive_known_type(rest_ty, EvidenceOrigin::PatternDecomposition)
                        }
                        Err(_) => TypeKnowledge::Unknown(UnknownReason::UncheckedExpression),
                    }
                } else {
                    TypeKnowledge::Unknown(UnknownReason::UncheckedExpression)
                };
                bind_declaration_pattern(
                    ctx,
                    rest_pattern,
                    ValueSemanticFact::new(rest_knowledge),
                    None,
                    causal_invalidity,
                    mutable,
                    phase,
                    initialization,
                    range,
                    explanation,
                    None,
                );
            }
        }
        _ => {}
    }
}

fn local_list_element(local_type: &LocalType) -> Option<LocalType> {
    match local_type {
        LocalType::Applied { arguments, .. } => arguments.first().cloned(),
        _ => None,
    }
}

fn validate_declaration_pattern(ctx: &mut CheckingContext<'_>, pattern: &Pattern) {
    match pattern {
        Pattern::Char { range, .. } => {
            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::DeclarationPatternRefutable,
                "character literal patterns are refutable and cannot be used in declaration destructuring",
                *range,
            ));
        }
        Pattern::Tuple { elements, .. } => {
            for element in elements {
                validate_declaration_pattern(ctx, element);
            }
        }
        Pattern::List { elements, rest, .. } => {
            for element in elements {
                validate_declaration_pattern(ctx, element);
            }
            if let Some(rest) = rest {
                validate_declaration_pattern(ctx, rest);
            }
        }
        Pattern::Record { entries, rest, .. } => {
            for entry in entries {
                validate_declaration_pattern(ctx, &entry.pattern);
            }
            if let Some(rest) = rest {
                validate_declaration_pattern(ctx, rest);
            }
        }
        Pattern::Or { range, .. } => {
            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::DeclarationPatternRefutable,
                "or-patterns are refutable and cannot be used in declaration destructuring",
                *range,
            ));
        }
        Pattern::Variant(variant) => {
            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::DeclarationPatternRefutable,
                "variant patterns are refutable and cannot be used in declaration destructuring",
                variant.range,
            ));
        }
        Pattern::Map { range, .. } => {
            ctx.emit_diagnostic(crate::diagnostic::SemanticDiagnostic::error_in(
                ctx.current_module.clone(),
                crate::diagnostic::DiagnosticCode::DeclarationPatternRefutable,
                "map patterns are refutable and cannot be used in declaration destructuring",
                *range,
            ));
        }
        Pattern::Name { .. } | Pattern::Wildcard { .. } => {}
    }
}
