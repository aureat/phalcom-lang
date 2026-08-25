//! Message send and callable argument verification (Spec 04.5 / E5).

use super::context::CheckingContext;
use super::expected::{ExpectationOrigin, ExpectedType};
use super::expression::analyze_expression;
use super::inference::{ConstraintOrigin, InferenceRelation, InferenceSession, InferenceSupport, InferenceTerm};
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::CallableSignature;
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::parameter::{GenericConstraint, TypeParameterOwner};
use crate::types::substitution::TypeSubstitution;
use phalcom_ast::ast::{Expr, PackItem, PackLabel};
use phalcom_common::range::SourceRange;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CallCheckResult {
    pub knowledge: TypeKnowledge,
    pub causal_invalidity: crate::checker::causal::CausalInvalidity,
    pub explanation_parents: Vec<crate::identity::ExplanationId>,
    pub callable: Option<crate::identity::CallableId>,
}

/// Promotes a complete callable return contract to call-site knowledge.
/// Unknown and dynamic contracts remain unknown/dynamic; only a concrete
/// exact-dispatch return receives established call-site status.
pub(crate) fn promote_exact_return(return_type: &TypeKnowledge, range: SourceRange) -> TypeKnowledge {
    match return_type {
        TypeKnowledge::Known(evidence) => TypeKnowledge::established(evidence.ty, EvidenceOrigin::CallableSignature).with_range(range),
        other => other.clone().with_range(range),
    }
}

/// Checks arguments passed to a callable against expected parameter type knowledge.
pub fn check_arguments(ctx: &mut CheckingContext<'_>, args: &[Expr], param_types: &[TypeKnowledge], call_range: SourceRange) {
    for (i, arg) in args.iter().enumerate() {
        let expected = param_types
            .get(i)
            .and_then(TypeKnowledge::ty)
            .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CallableSignature))
            .unwrap_or_default();
        let arg_typed = analyze_expression(ctx, arg, &expected);
        if let Some(param_k) = param_types.get(i) {
            ctx.enforce_assignability(
                &arg_typed.knowledge,
                param_k,
                DiagnosticCode::ArgumentMismatch,
                format!("argument at position {} does not match expected parameter type", i + 1),
                call_range,
            );
        }
    }
}

/// Matches call argument pack items against a callable signature, validating labels and types.
pub fn match_callable_arguments(ctx: &mut CheckingContext<'_>, args: &[PackItem], signature: &CallableSignature, call_range: SourceRange) -> TypeKnowledge {
    resolve_call(ctx, signature, args, &ExpectedType::None, call_range).knowledge
}

/// Canonical call resolution with generic method inference and bidirectional parameter propagation.
pub fn resolve_call(
    ctx: &mut CheckingContext<'_>,
    signature: &CallableSignature,
    args: &[PackItem],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> CallCheckResult {
    ctx.begin_call_causal_capture();
    let knowledge = resolve_call_inner(ctx, signature, args, expected, call_range);
    let (causal_invalidity, explanation_parents) = ctx.end_call_causal_capture();
    CallCheckResult {
        knowledge,
        causal_invalidity,
        explanation_parents,
        callable: ctx.resolved_callable_for_current_expression(),
    }
}

fn resolve_call_inner(
    ctx: &mut CheckingContext<'_>,
    signature: &CallableSignature,
    args: &[PackItem],
    expected: &ExpectedType,
    call_range: SourceRange,
) -> TypeKnowledge {
    // 1. Generic Callable Resolution via InferenceSession
    if let Some(ref generic_sig) = signature.generics {
        if !generic_sig.parameters.is_empty() {
            let mut session = InferenceSession::new();
            let var_map = session.instantiate_generic_signature(generic_sig, ctx.store);

            let Some(call_id) = ctx.current_expression_id() else {
                return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
            };
            let generic_callable = match &generic_sig.owner {
                TypeParameterOwner::Callable(callable) => callable.clone(),
                TypeParameterOwner::Declaration(declaration) => {
                    crate::identity::CallableId::new(declaration.clone(), signature.selector.clone(), ctx.current_side)
                }
            };
            for (constraint_index, constraint) in generic_sig.constraints.iter().enumerate() {
                let relation = match constraint {
                    GenericConstraint::Subtype { lower, upper } => {
                        let lower = session.type_term_to_inference(lower, &var_map, ctx.store);
                        let upper = session.type_term_to_inference(upper, &var_map, ctx.store);
                        match (lower, upper) {
                            (Ok(lower), Ok(upper)) => InferenceRelation::Subtype(lower, upper),
                            _ => return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
                        }
                    }
                    GenericConstraint::Equivalent { left, right } => {
                        let left = session.type_term_to_inference(left, &var_map, ctx.store);
                        let right = session.type_term_to_inference(right, &var_map, ctx.store);
                        match (left, right) {
                            (Ok(left), Ok(right)) => InferenceRelation::Equivalent(left, right),
                            _ => return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
                        }
                    }
                };
                session.add_constraint(
                    relation,
                    ConstraintOrigin::GenericWhere {
                        callable: generic_callable.clone(),
                        constraint_index: constraint_index as u16,
                    },
                    None,
                );
            }
            let mut positional_idx = 0;

            for arg in args.iter() {
                match arg {
                    PackItem::Positional { expr, .. } => {
                        let mut matched = false;
                        while positional_idx < signature.parameters.len() {
                            let param = &signature.parameters[positional_idx];
                            positional_idx += 1;
                            if param.external_label.is_none() {
                                let Some(pty) = param.ty.ty() else {
                                    return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
                                };
                                let param_term = session.type_id_to_inference(pty, &var_map, ctx.store);
                                let arg_expected = ExpectedType::inference_from(param_term.clone(), ExpectationOrigin::GenericArgument);
                                let arg_typed = analyze_expression(ctx, expr, &arg_expected);
                                if let Some(arg_ty) = arg_typed.knowledge.ty() {
                                    if let Some(support) = inference_support(&arg_typed.knowledge) {
                                        let explanation = arg_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id));
                                        session.add_constraint_with_support(
                                            InferenceRelation::Subtype(InferenceTerm::Canonical(arg_ty), param_term),
                                            ConstraintOrigin::Argument {
                                                call: call_id,
                                                argument: arg_typed.expression_id.expect("analyzed argument has expression identity"),
                                                parameter_index: (positional_idx - 1) as u16,
                                            },
                                            explanation,
                                            support,
                                        );
                                    }
                                }
                                matched = true;
                                break;
                            }
                        }
                        if !matched {
                            analyze_expression(ctx, expr, &ExpectedType::None);
                        }
                    }
                    PackItem::Labeled { label, value, .. } => {
                        if let PackLabel::Static { text, .. } = label {
                            let mut matched = false;
                            for (p_idx, param) in signature.parameters.iter().enumerate() {
                                if let Some(ref ext_label) = param.external_label {
                                    if ext_label == text {
                                        let Some(pty) = param.ty.ty() else {
                                            return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
                                        };
                                        let param_term = session.type_id_to_inference(pty, &var_map, ctx.store);
                                        let arg_expected = ExpectedType::inference_from(param_term.clone(), ExpectationOrigin::GenericArgument);
                                        let arg_typed = analyze_expression(ctx, value, &arg_expected);
                                        if let Some(arg_ty) = arg_typed.knowledge.ty() {
                                            if let Some(support) = inference_support(&arg_typed.knowledge) {
                                                let explanation = arg_typed.expression_id.and_then(|id| ctx.explanation_for_expression(id));
                                                session.add_constraint_with_support(
                                                    InferenceRelation::Subtype(InferenceTerm::Canonical(arg_ty), param_term),
                                                    ConstraintOrigin::Argument {
                                                        call: call_id,
                                                        argument: arg_typed.expression_id.expect("analyzed argument has expression identity"),
                                                        parameter_index: p_idx as u16,
                                                    },
                                                    explanation,
                                                    support,
                                                );
                                            }
                                        }
                                        matched = true;
                                        break;
                                    }
                                }
                            }
                            if !matched {
                                analyze_expression(ctx, value, &ExpectedType::None);
                            }
                        } else {
                            analyze_expression(ctx, value, &ExpectedType::None);
                            return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
                        }
                    }
                    PackItem::Expand { expr, .. } => {
                        analyze_expression(ctx, expr, &ExpectedType::None);
                        return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
                    }
                }
            }

            // Collect expected result constraint. Contextual expectation can
            // select a valid instantiation, but does not count as value support.
            let return_term = signature
                .return_type
                .ty()
                .map(|ret_ty| session.type_id_to_inference(ret_ty, &var_map, ctx.store));
            if let Some(ret_term) = return_term.as_ref() {
                if let Some(exp_ty) = expected.ty() {
                    session.add_constraint(
                        InferenceRelation::Subtype(ret_term.clone(), InferenceTerm::Canonical(exp_ty)),
                        ConstraintOrigin::ExpectedResult { expression: call_id },
                        None,
                    );
                } else if let ExpectedType::Inference { term: exp_term, .. } = expected {
                    session.add_constraint(
                        InferenceRelation::Subtype(ret_term.clone(), exp_term.clone()),
                        ConstraintOrigin::ExpectedResult { expression: call_id },
                        None,
                    );
                }
            }

            let outcome = session.solve(ctx.store, &ctx.hierarchy);
            let fixed_return = return_term.as_ref().and_then(|term| {
                if session.term_has_variables(term) {
                    None
                } else {
                    Some(promote_exact_return(&signature.return_type, call_range))
                }
            });
            return match &outcome {
                crate::checker::inference::InferenceOutcome::Solved(solution) => {
                    if let Some(ret_ty) = signature.return_type.ty() {
                        let mut subst = TypeSubstitution::new();
                        for (&param_id, var_term) in &var_map {
                            if let InferenceTerm::Var(v) = var_term {
                                if let Some(&solved_ty) = solution.substitutions.get(v) {
                                    subst.bind(param_id, solved_ty);
                                }
                            }
                        }
                        let specialized_ret = subst.apply(ctx.store, ret_ty);
                        let support = return_term.as_ref().and_then(|term| session.term_support(term));
                        match (session.term_has_variables(return_term.as_ref().expect("generic return term")), support) {
                            (true, Some(InferenceSupport::Established)) => {
                                TypeKnowledge::established(specialized_ret, EvidenceOrigin::GenericInference).with_range(call_range)
                            }
                            (true, Some(InferenceSupport::Assumed)) => {
                                TypeKnowledge::assumed(specialized_ret, EvidenceOrigin::GenericInference).with_range(call_range)
                            }
                            (true, None) => TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable),
                            (false, _) => fixed_return.expect("fixed generic return must be available"),
                        }
                    } else {
                        promote_exact_return(&signature.return_type, call_range)
                    }
                }
                crate::checker::inference::InferenceOutcome::Underconstrained(_) => terminal_generic_return(&outcome, fixed_return),
                crate::checker::inference::InferenceOutcome::Conflicting(_) => {
                    ctx.emit_diagnostic(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ArgumentMismatch,
                        "generic argument does not satisfy type constraints",
                        call_range,
                    ));
                    terminal_generic_return(&outcome, fixed_return)
                }
                crate::checker::inference::InferenceOutcome::Blocked(_)
                | crate::checker::inference::InferenceOutcome::Cancelled
                | crate::checker::inference::InferenceOutcome::BudgetExceeded(_) => terminal_generic_return(&outcome, fixed_return),
            };
        }
    }

    // 2. Non-generic Callable Resolution
    let mut positional_idx = 0;
    for arg in args {
        match arg {
            PackItem::Positional { expr, range } => {
                let mut matched_param = None;
                while positional_idx < signature.parameters.len() {
                    let param = &signature.parameters[positional_idx];
                    positional_idx += 1;
                    if param.external_label.is_none() {
                        matched_param = Some(param);
                        break;
                    }
                }
                let expected_arg = matched_param
                    .and_then(|p| p.ty.ty())
                    .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CallableSignature))
                    .unwrap_or_default();
                let arg_typed = analyze_expression(ctx, expr, &expected_arg);
                if let Some(param) = matched_param {
                    ctx.enforce_assignability(
                        &arg_typed.knowledge,
                        &param.ty,
                        DiagnosticCode::ArgumentMismatch,
                        format!("positional argument `{}` does not match expected parameter type", param.local_name),
                        *range,
                    );
                }
            }
            PackItem::Labeled { label, value, range } => {
                if let PackLabel::Static { text, .. } = label {
                    let mut matched_param = None;
                    for param in &signature.parameters {
                        if let Some(ref ext_label) = param.external_label {
                            if ext_label == text {
                                matched_param = Some(param);
                                break;
                            }
                        }
                    }
                    let expected_arg = matched_param
                        .and_then(|p| p.ty.ty())
                        .map(|ty| ExpectedType::proper_from(ty, ExpectationOrigin::CallableSignature))
                        .unwrap_or_default();
                    let arg_typed = analyze_expression(ctx, value, &expected_arg);
                    if let Some(param) = matched_param {
                        ctx.enforce_assignability(
                            &arg_typed.knowledge,
                            &param.ty,
                            DiagnosticCode::ArgumentMismatch,
                            format!("argument for label `{}:` does not match expected parameter type", text),
                            *range,
                        );
                    }
                } else {
                    analyze_expression(ctx, value, &ExpectedType::None);
                    return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
                }
            }
            PackItem::Expand { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
                return TypeKnowledge::Unknown(UnknownReason::InferenceBlocked);
            }
        }
    }

    // Successful non-generic dispatch establishes that this call reached a
    // concrete callable contract. Keep the contract's type, but upgrade the
    // call-site evidence to `Proven`; the declaration remains `Declared` in
    // the published surface and can still be checked independently against
    // the body.
    promote_exact_return(&signature.return_type, call_range)
}

fn terminal_generic_return(outcome: &crate::checker::inference::InferenceOutcome, fixed_return: Option<TypeKnowledge>) -> TypeKnowledge {
    if let Some(fixed_return) = fixed_return {
        return fixed_return;
    }
    match outcome {
        crate::checker::inference::InferenceOutcome::Underconstrained(_) => TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable),
        crate::checker::inference::InferenceOutcome::Conflicting(_) => TypeKnowledge::Unknown(UnknownReason::InferenceConflict),
        crate::checker::inference::InferenceOutcome::Blocked(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
        crate::checker::inference::InferenceOutcome::Cancelled => TypeKnowledge::Unknown(UnknownReason::InferenceCancelled),
        crate::checker::inference::InferenceOutcome::BudgetExceeded(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded),
        crate::checker::inference::InferenceOutcome::Solved(_) => TypeKnowledge::Unknown(UnknownReason::InferenceBlocked),
    }
}

fn inference_support(knowledge: &TypeKnowledge) -> Option<InferenceSupport> {
    match knowledge.status() {
        Some(EvidenceStatus::Established) => Some(InferenceSupport::Established),
        Some(EvidenceStatus::Assumed) => Some(InferenceSupport::Assumed),
        None => None,
    }
}

#[cfg(test)]
mod tests {
    use super::terminal_generic_return;
    use crate::checker::inference::{InferenceConflict, InferenceFailureReason, InferenceOutcome, InferenceTerm, UnderconstrainedInference};
    use crate::types::evidence::{EvidenceOrigin, TypeKnowledge, UnknownReason};
    use crate::types::id::TypeId;
    use crate::types::outcome::{BlockReason, BudgetKind, BudgetReport};

    fn terminal_outcomes() -> [(InferenceOutcome, UnknownReason); 5] {
        [
            (
                InferenceOutcome::Underconstrained(UnderconstrainedInference { unsolved_vars: Vec::new() }),
                UnknownReason::UnderconstrainedTypeVariable,
            ),
            (
                InferenceOutcome::Conflicting(InferenceConflict {
                    constraint_index: Some(2),
                    origin: None,
                    failure: InferenceFailureReason::StructuralMismatch {
                        left: Box::new(InferenceTerm::Canonical(TypeId(1))),
                        right: Box::new(InferenceTerm::Canonical(TypeId(2))),
                    },
                }),
                UnknownReason::InferenceConflict,
            ),
            (InferenceOutcome::Blocked(BlockReason::RecursiveFixpoint), UnknownReason::InferenceBlocked),
            (InferenceOutcome::Cancelled, UnknownReason::InferenceCancelled),
            (
                InferenceOutcome::BudgetExceeded(BudgetReport::new(BudgetKind::Steps, 0, 1)),
                UnknownReason::InferenceBudgetExceeded,
            ),
        ]
    }

    #[test]
    fn every_generic_terminal_outcome_keeps_its_reason_without_fixed_return() {
        for (outcome, reason) in terminal_outcomes() {
            assert_eq!(terminal_generic_return(&outcome, None), TypeKnowledge::Unknown(reason));
        }
    }

    #[test]
    fn every_generic_terminal_outcome_preserves_only_independent_fixed_return() {
        let fixed = TypeKnowledge::established(TypeId(99), EvidenceOrigin::CallableSignature);
        for (outcome, _) in terminal_outcomes() {
            assert_eq!(terminal_generic_return(&outcome, Some(fixed.clone())), fixed);
        }
    }
}
