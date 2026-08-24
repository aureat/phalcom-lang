//! Message send and callable argument verification (Spec 04.5 / E5).

use super::context::CheckingContext;
use super::expected::ExpectedType;
use super::expression::analyze_expression;
use super::inference::{ConstraintOrigin, InferenceRelation, InferenceSession, InferenceSupport, InferenceTerm};
use crate::checker::policy::enforce_assignability;
use crate::diagnostic::{DiagnosticCode, SemanticDiagnostic};
use crate::dispatch::CallableSignature;
use crate::identity::{BodyId, ExpressionId, LocalExpressionId};
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::substitution::TypeSubstitution;
use phalcom_ast::ast::{Expr, PackItem, PackLabel};
use phalcom_common::range::SourceRange;

/// Checks arguments passed to a callable against expected parameter type knowledge.
pub fn check_arguments(ctx: &mut CheckingContext<'_>, args: &[Expr], param_types: &[TypeKnowledge], call_range: SourceRange) {
    for (i, arg) in args.iter().enumerate() {
        let expected = param_types.get(i).map(ExpectedType::from_knowledge).unwrap_or_default();
        let arg_typed = analyze_expression(ctx, arg, &expected);
        if let Some(param_k) = param_types.get(i) {
            enforce_assignability(
                ctx.store,
                &ctx.hierarchy,
                &arg_typed.knowledge,
                param_k,
                &ctx.current_module,
                DiagnosticCode::ArgumentMismatch,
                format!("argument at position {} does not match expected parameter type", i + 1),
                call_range,
                &mut ctx.diagnostics,
            );
        }
    }
}

/// Matches call argument pack items against a callable signature, validating labels and types.
pub fn match_callable_arguments(ctx: &mut CheckingContext<'_>, args: &[PackItem], signature: &CallableSignature, call_range: SourceRange) -> TypeKnowledge {
    resolve_call(ctx, signature, args, &ExpectedType::None, call_range)
}

/// Canonical call resolution with generic method inference and bidirectional parameter propagation.
pub fn resolve_call(
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

            let dummy_call_id = ExpressionId::new(BodyId(0), LocalExpressionId(0));
            let mut positional_idx = 0;

            for (arg_idx, arg) in args.iter().enumerate() {
                let dummy_arg_id = ExpressionId::new(BodyId(0), LocalExpressionId(arg_idx as u32));
                match arg {
                    PackItem::Positional { expr, .. } => {
                        while positional_idx < signature.parameters.len() {
                            let param = &signature.parameters[positional_idx];
                            positional_idx += 1;
                            if param.external_label.is_none() {
                                let param_term = if let Some(pty) = param.ty.ty() {
                                    session.type_id_to_inference(pty, &var_map, ctx.store)
                                } else {
                                    InferenceTerm::Canonical(ctx.store.unit())
                                };
                                let arg_expected = ExpectedType::Inference(param_term.clone());
                                let arg_typed = analyze_expression(ctx, expr, &arg_expected);
                                if let Some(arg_ty) = arg_typed.knowledge.ty() {
                                    if let Some(support) = inference_support(&arg_typed.knowledge) {
                                        session.add_constraint_with_support(
                                            InferenceRelation::Subtype(InferenceTerm::Canonical(arg_ty), param_term),
                                            ConstraintOrigin::Argument {
                                                call: dummy_call_id,
                                                argument: dummy_arg_id,
                                                parameter_index: (positional_idx - 1) as u16,
                                            },
                                            None,
                                            support,
                                        );
                                    }
                                }
                                break;
                            }
                        }
                    }
                    PackItem::Labeled { label, value, .. } => {
                        if let PackLabel::Static { text, .. } = label {
                            for (p_idx, param) in signature.parameters.iter().enumerate() {
                                if let Some(ref ext_label) = param.external_label {
                                    if ext_label == text {
                                        let param_term = if let Some(pty) = param.ty.ty() {
                                            session.type_id_to_inference(pty, &var_map, ctx.store)
                                        } else {
                                            InferenceTerm::Canonical(ctx.store.unit())
                                        };
                                        let arg_expected = ExpectedType::Inference(param_term.clone());
                                        let arg_typed = analyze_expression(ctx, value, &arg_expected);
                                        if let Some(arg_ty) = arg_typed.knowledge.ty() {
                                            if let Some(support) = inference_support(&arg_typed.knowledge) {
                                                session.add_constraint_with_support(
                                                    InferenceRelation::Subtype(InferenceTerm::Canonical(arg_ty), param_term),
                                                    ConstraintOrigin::Argument {
                                                        call: dummy_call_id,
                                                        argument: dummy_arg_id,
                                                        parameter_index: p_idx as u16,
                                                    },
                                                    None,
                                                    support,
                                                );
                                            }
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    PackItem::Expand { expr, .. } => {
                        analyze_expression(ctx, expr, &ExpectedType::None);
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
                        ConstraintOrigin::ExpectedResult { expression: dummy_call_id },
                        None,
                    );
                } else if let ExpectedType::Inference(exp_term) = expected {
                    session.add_constraint(
                        InferenceRelation::Subtype(ret_term.clone(), exp_term.clone()),
                        ConstraintOrigin::ExpectedResult { expression: dummy_call_id },
                        None,
                    );
                }
            }

            let outcome = session.solve(ctx.store, &ctx.hierarchy);
            let fixed_return = return_term.as_ref().and_then(|term| {
                if session.term_has_variables(term) {
                    None
                } else {
                    signature
                        .return_type
                        .ty()
                        .map(|ty| TypeKnowledge::established(ty, EvidenceOrigin::CallableSignature).with_range(call_range))
                }
            });
            return match outcome {
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
                        signature.return_type.clone().with_range(call_range)
                    }
                }
                crate::checker::inference::InferenceOutcome::Underconstrained(_) => {
                    fixed_return.unwrap_or(TypeKnowledge::Unknown(UnknownReason::UnderconstrainedTypeVariable))
                }
                crate::checker::inference::InferenceOutcome::Conflicting(_) => {
                    ctx.diagnostics.push(SemanticDiagnostic::error_in(
                        ctx.current_module.clone(),
                        DiagnosticCode::ArgumentMismatch,
                        "generic argument does not satisfy type constraints",
                        call_range,
                    ));
                    fixed_return.unwrap_or(TypeKnowledge::Unknown(UnknownReason::InferenceConflict))
                }
                crate::checker::inference::InferenceOutcome::Blocked(_) => fixed_return.unwrap_or(TypeKnowledge::Unknown(UnknownReason::InferenceBlocked)),
                crate::checker::inference::InferenceOutcome::Cancelled => fixed_return.unwrap_or(TypeKnowledge::Unknown(UnknownReason::InferenceCancelled)),
                crate::checker::inference::InferenceOutcome::BudgetExceeded(_) => {
                    fixed_return.unwrap_or(TypeKnowledge::Unknown(UnknownReason::InferenceBudgetExceeded))
                }
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
                let expected_arg = matched_param.map(|p| ExpectedType::from_knowledge(&p.ty)).unwrap_or_default();
                let arg_typed = analyze_expression(ctx, expr, &expected_arg);
                if let Some(param) = matched_param {
                    enforce_assignability(
                        ctx.store,
                        &ctx.hierarchy,
                        &arg_typed.knowledge,
                        &param.ty,
                        &ctx.current_module,
                        DiagnosticCode::ArgumentMismatch,
                        format!("positional argument `{}` does not match expected parameter type", param.local_name),
                        *range,
                        &mut ctx.diagnostics,
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
                    let expected_arg = matched_param.map(|p| ExpectedType::from_knowledge(&p.ty)).unwrap_or_default();
                    let arg_typed = analyze_expression(ctx, value, &expected_arg);
                    if let Some(param) = matched_param {
                        enforce_assignability(
                            ctx.store,
                            &ctx.hierarchy,
                            &arg_typed.knowledge,
                            &param.ty,
                            &ctx.current_module,
                            DiagnosticCode::ArgumentMismatch,
                            format!("argument for label `{}:` does not match expected parameter type", text),
                            *range,
                            &mut ctx.diagnostics,
                        );
                    }
                } else {
                    analyze_expression(ctx, value, &ExpectedType::None);
                }
            }
            PackItem::Expand { expr, .. } => {
                analyze_expression(ctx, expr, &ExpectedType::None);
            }
        }
    }

    // Successful non-generic dispatch establishes that this call reached a
    // concrete callable contract. Keep the contract's type, but upgrade the
    // call-site evidence to `Proven`; the declaration remains `Declared` in
    // the published surface and can still be checked independently against
    // the body.
    match &signature.return_type {
        TypeKnowledge::Known(evidence) => TypeKnowledge::established(evidence.ty, EvidenceOrigin::CallableSignature).with_range(call_range),
        other => other.clone().with_range(call_range),
    }
}

fn inference_support(knowledge: &TypeKnowledge) -> Option<InferenceSupport> {
    match knowledge.status() {
        Some(EvidenceStatus::Established) => Some(InferenceSupport::Established),
        Some(EvidenceStatus::Assumed) => Some(InferenceSupport::Assumed),
        None => None,
    }
}
