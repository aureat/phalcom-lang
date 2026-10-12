use crate::checker::proof_predicate::ProofPredicateGoal;
use crate::member_projection::{MemberProjection, MemberProjectionArgument, MemberProjectionSubject, StaticMemberProjectionOutcome};
use crate::static_domain::StaticValueDomain;
use crate::static_eval::application::{
    StaticSpecializationArgument, StaticSpecializationEnvironmentError, StaticSpecializationEnvironmentKey,
};
use crate::static_eval::error::StaticEvalError;
use crate::static_eval::machine::{StaticArgument, StaticEvaluator, Stop};
use crate::static_eval::residual::{
    StaticEvalTerm, StaticResidualApplication, StaticResidualData, StaticResidualParameter, StaticResidualRef,
};
use crate::traits::TraitRef;
use crate::types::id::TypeId;
use crate::types::parameter::TypeTerm;
use crate::types::store::TypeStore;
use crate::types::substitution::TypeSubstitution;

/// Specializes a static residual against a supplied specialization environment.
///
/// Implements specialization for:
/// - `StaticResidualData::Parameter`
/// - `StaticResidualData::Apply`
/// - `StaticResidualData::Proof`
/// - `StaticResidualData::MemberProjection`
///
/// Enforces:
/// - Substitution closure before conflict checking
/// - Exact evaluator reuse for closed applications
/// - Existing proof authority reuse for proofs
/// - Existing `StructuralReflectionProjector` reuse for member projections
/// - Exact residual node reuse on no new facts
/// Specializes a static residual using the evaluator's context, returning a terminal or symbolic outcome.
pub fn specialize_residual(
    evaluator: &mut StaticEvaluator<'_>,
    residual: StaticResidualRef,
    supplied: &StaticSpecializationEnvironmentKey,
) -> crate::static_eval::error::StaticEvalOutcome {
    evaluator.specialize_residual(residual, supplied)
}

pub(crate) fn specialize_residual_term(
    evaluator: &mut StaticEvaluator<'_>,
    residual: StaticResidualRef,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    if evaluator.is_cancelled() {
        return Err(Stop::Cancelled);
    }
    evaluator.charge_step()?;

    let residual_data = {
        let store = evaluator.residuals().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        store.get(residual).map_err(Stop::from)?.clone()
    };

    match residual_data {
        StaticResidualData::Parameter { parameter } => {
            specialize_parameter(evaluator, residual, &parameter, supplied)
        }
        StaticResidualData::Apply { application, result_domain } => {
            specialize_apply(evaluator, residual, &application, result_domain, supplied)
        }
        StaticResidualData::Proof { goal, result_type, environment } => {
            specialize_proof(evaluator, residual, &goal, result_type, &environment, supplied)
        }
        StaticResidualData::MemberProjection { projection, result_domain, environment } => {
            specialize_member_projection(evaluator, residual, &projection, result_domain, &environment, supplied)
        }
        _ => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
    }
}

fn specialize_parameter(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    parameter: &StaticResidualParameter,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    for (param, arg) in supplied.argument_bindings() {
        if param == parameter {
            match arg {
                StaticSpecializationArgument::Kinded(ty) => return Ok(StaticEvalTerm::Kinded(*ty)),
                StaticSpecializationArgument::Value(val) => return Ok(StaticEvalTerm::Value(*val)),
            }
        }
    }
    for (param, arg) in evaluator.inherited_specialization_environment().argument_bindings() {
        if param == parameter {
            match arg {
                StaticSpecializationArgument::Kinded(ty) => return Ok(StaticEvalTerm::Kinded(*ty)),
                StaticSpecializationArgument::Value(val) => return Ok(StaticEvalTerm::Value(*val)),
            }
        }
    }
    // No new binding supplied: reuse existing interned residual reference.
    Ok(StaticEvalTerm::Residual(original))
}

fn specialize_apply(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    application: &StaticResidualApplication,
    result_domain: StaticValueDomain,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    // 1. Compose captured environment with supplied specialization
    let composed_env = evaluator.compose_specialization_environment(&application.environment, supplied)?;

    // 2. Recursively specialize arguments
    let mut specialized_args = Vec::with_capacity(application.arguments.len());
    let mut any_residual = false;
    let mut args_changed = false;

    for arg in application.arguments.iter() {
        match arg {
            StaticEvalTerm::Residual(r) => {
                let spec_term = specialize_residual_term(evaluator, *r, supplied)?;
                if spec_term != *arg {
                    args_changed = true;
                }
                if matches!(spec_term, StaticEvalTerm::Residual(_)) {
                    any_residual = true;
                }
                specialized_args.push(spec_term);
            }
            concrete => {
                specialized_args.push(*concrete);
            }
        }
    }

    if any_residual {
        // If no changes to arguments or environment, reuse exact existing residual
        if !args_changed && composed_env == application.environment {
            return Ok(StaticEvalTerm::Residual(original));
        }

        let new_ref = evaluator.intern_applied_residual(
            StaticResidualApplication {
                definition: application.definition.clone(),
                arguments: specialized_args.into_boxed_slice(),
                environment: composed_env,
            },
            result_domain,
        )?;
        return Ok(StaticEvalTerm::Residual(new_ref));
    }

    // 3. All arguments are concrete -> evaluate ordinary static definition
    let static_args = specialized_args
        .into_iter()
        .map(|term| match term {
            StaticEvalTerm::Kinded(k) => StaticArgument::Kinded(k),
            StaticEvalTerm::Value(v) => StaticArgument::Value(v),
            StaticEvalTerm::Residual(_) => unreachable!(),
        })
        .collect::<Vec<_>>();

    evaluator.evaluate_specialized_application(application.definition.clone(), &static_args, composed_env)
}

fn specialize_proof(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    goal: &ProofPredicateGoal,
    result_type: TypeId,
    environment: &StaticSpecializationEnvironmentKey,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    // 1. Compose captured environment with supplied specialization
    let composed_env = evaluator.compose_specialization_environment(environment, supplied)?;

    // 2. Substitute goal propositions using substitution closure
    let (specialized_goal, changed) = evaluator.substitute_proof_goal(goal, &composed_env)?;

    // Check No-New-Facts identity
    if !changed && composed_env == *environment {
        if goal_has_open_terms(&specialized_goal, evaluator)? {
            return Ok(StaticEvalTerm::Residual(original));
        }
    }

    // 3. Check if specialized goal still has open terms
    if goal_has_open_terms(&specialized_goal, evaluator)? {
        let new_ref = evaluator.intern_proof_residual_node(specialized_goal, result_type, composed_env)?;
        return Ok(StaticEvalTerm::Residual(new_ref));
    }

    // 4. Closed goal: evaluate using existing proof authority
    let outcome = evaluator.evaluate_proof_predicate_with_environment(&specialized_goal, result_type, composed_env);
    match outcome {
        Ok(true) => Ok(StaticEvalTerm::Value(evaluator.intern_bool(true))),
        Ok(false) => Ok(StaticEvalTerm::Value(evaluator.intern_bool(false))),
        Err(Stop::Symbolic(r)) => Ok(StaticEvalTerm::Residual(r)),
        Err(stop) => Err(stop),
    }
}

fn specialize_member_projection(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    projection: &MemberProjection,
    result_domain: StaticValueDomain,
    environment: &StaticSpecializationEnvironmentKey,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    // 1. Compose captured environment with supplied specialization
    let composed_env = evaluator.compose_specialization_environment(environment, supplied)?;

    // 2. Substitute subject and arguments
    let (specialized_projection, changed) = evaluator.substitute_member_projection(projection, &composed_env)?;

    // Check No-New-Facts identity
    if !changed && composed_env == *environment {
        if member_projection_is_open(&specialized_projection, evaluator)? {
            return Ok(StaticEvalTerm::Residual(original));
        }
    }

    // 3. If still open: re-intern MemberProjection
    if member_projection_is_open(&specialized_projection, evaluator)? {
        let new_ref = evaluator.intern_member_projection_residual_node(specialized_projection, result_domain, composed_env)?;
        return Ok(StaticEvalTerm::Residual(new_ref));
    }

    // 4. Closed: evaluate using existing StructuralReflectionProjector
    let outcome = evaluator.project_member_with_environment(&specialized_projection, composed_env)?;
    match outcome {
        StaticMemberProjectionOutcome::ReducedValue(val) => Ok(StaticEvalTerm::Value(val)),
        StaticMemberProjectionOutcome::ReducedKinded(ty) => Ok(StaticEvalTerm::Kinded(ty)),
        StaticMemberProjectionOutcome::Stuck(stuck_proj) => {
            let new_ref = evaluator.intern_member_projection_residual_node(stuck_proj, result_domain, environment.clone())?;
            Ok(StaticEvalTerm::Residual(new_ref))
        }
        StaticMemberProjectionOutcome::Invalid(reason) => Err(Stop::invalid(StaticEvalError::MemberProjection(reason))),
        StaticMemberProjectionOutcome::Unknown(reason) => Err(Stop::Unknown(reason)),
        StaticMemberProjectionOutcome::Blocked(reason) => Err(Stop::Blocked(reason)),
        StaticMemberProjectionOutcome::Dynamic(obligation) => Err(Stop::Dynamic(obligation)),
        StaticMemberProjectionOutcome::Cancelled => Err(Stop::Cancelled),
        StaticMemberProjectionOutcome::BudgetExceeded(report) => Err(Stop::Budget(report)),
        StaticMemberProjectionOutcome::InternalFailure(msg) => Err(Stop::invalid(StaticEvalError::MemberProjection(msg))),
    }
}

pub(crate) fn substitute_proof_goal(
    goal: &ProofPredicateGoal,
    subst: &TypeSubstitution,
    types: &mut TypeStore,
) -> Result<(ProofPredicateGoal, bool), StaticSpecializationEnvironmentError> {
    match goal {
        ProofPredicateGoal::Equivalent { left, right } => {
            let sub_left = subst.try_apply(types, *left)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let sub_right = subst.try_apply(types, *right)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let changed = sub_left != *left || sub_right != *right;
            let (min, max) = if sub_left <= sub_right { (sub_left, sub_right) } else { (sub_right, sub_left) };
            Ok((ProofPredicateGoal::Equivalent { left: min, right: max }, changed))
        }
        ProofPredicateGoal::Subtype { lower, upper } => {
            let sub_lower = subst.try_apply(types, *lower)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let sub_upper = subst.try_apply(types, *upper)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let changed = sub_lower != *lower || sub_upper != *upper;
            Ok((ProofPredicateGoal::Subtype { lower: sub_lower, upper: sub_upper }, changed))
        }
        ProofPredicateGoal::Conforms { key } => {
            let mut changed = false;
            let sub_subject = match &key.subject {
                TypeTerm::Canonical(ty) => {
                    let sub_ty = subst.try_apply(types, *ty)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    if sub_ty != *ty {
                        changed = true;
                    }
                    TypeTerm::Canonical(sub_ty)
                }
                other => other.clone(),
            };
            let mut sub_args = Vec::with_capacity(key.trait_ref.arguments.len());
            for &arg_ty in key.trait_ref.arguments.iter() {
                let sub_arg = subst.try_apply(types, arg_ty)
                    .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                if sub_arg != arg_ty {
                    changed = true;
                }
                sub_args.push(sub_arg);
            }
            let sub_key = crate::generic_evidence::GenericConformanceKey::new(
                sub_subject,
                TraitRef::new(key.trait_ref.declaration.clone(), sub_args),
            );
            Ok((ProofPredicateGoal::Conforms { key: sub_key }, changed))
        }
    }
}

fn goal_has_open_terms(
    goal: &ProofPredicateGoal,
    evaluator: &StaticEvaluator<'_>,
) -> Result<bool, Stop> {
    match goal {
        ProofPredicateGoal::Equivalent { left, right } => {
            Ok(evaluator.is_open_type(*left)? || evaluator.is_open_type(*right)?)
        }
        ProofPredicateGoal::Subtype { lower, upper } => {
            Ok(evaluator.is_open_type(*lower)? || evaluator.is_open_type(*upper)?)
        }
        ProofPredicateGoal::Conforms { key } => {
            let open_subject = match &key.subject {
                TypeTerm::Canonical(ty) => evaluator.is_open_type(*ty)?,
                TypeTerm::SelfType(_) => true,
                TypeTerm::Infer(_) => false,
            };
            if open_subject {
                return Ok(true);
            }
            for &arg_ty in key.trait_ref.arguments.iter() {
                if evaluator.is_open_type(arg_ty)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
    }
}

pub(crate) fn substitute_member_projection(
    projection: &MemberProjection,
    subst: &TypeSubstitution,
    types: &mut TypeStore,
) -> Result<(MemberProjection, bool), StaticSpecializationEnvironmentError> {
    let mut changed = false;
    let sub_subject = match &projection.subject {
        MemberProjectionSubject::Type(ty) => {
            let sub_ty = subst.try_apply(types, *ty)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            if sub_ty != *ty {
                changed = true;
            }
            MemberProjectionSubject::Type(sub_ty)
        }
        other => other.clone(),
    };
    let mut sub_args = Vec::with_capacity(projection.arguments.len());
    for arg in projection.arguments.iter() {
        match arg {
            MemberProjectionArgument::Kinded(crate::types::type_member::TypeMemberArgument::Type(ty)) => {
                let sub_ty = subst.try_apply(types, *ty)
                    .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                if sub_ty != *ty {
                    changed = true;
                }
                sub_args.push(MemberProjectionArgument::Kinded(crate::types::type_member::TypeMemberArgument::Type(sub_ty)));
            }
            other => {
                sub_args.push(other.clone());
            }
        }
    }
    let sub_evidence = match &projection.evidence {
        Some(ev) => {
            let sub_subj = match &ev.subject {
                TypeTerm::Canonical(ty) => {
                    let sub_ty = subst.try_apply(types, *ty)
                        .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                    if sub_ty != *ty {
                        changed = true;
                    }
                    TypeTerm::Canonical(sub_ty)
                }
                other => other.clone(),
            };
            let mut sub_trait_args = Vec::with_capacity(ev.trait_ref.arguments.len());
            for &arg_ty in ev.trait_ref.arguments.iter() {
                let sub_arg = subst.try_apply(types, arg_ty)
                    .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                if sub_arg != arg_ty {
                    changed = true;
                }
                sub_trait_args.push(sub_arg);
            }
            Some(crate::generic_evidence::GenericConformanceKey::new(
                sub_subj,
                TraitRef::new(ev.trait_ref.declaration.clone(), sub_trait_args),
            ))
        }
        None => None,
    };
    let sub_proj = MemberProjection {
        subject: sub_subject,
        member: projection.member.clone(),
        arguments: sub_args.into_boxed_slice(),
        evidence: sub_evidence,
        result_domain: projection.result_domain,
    };
    Ok((sub_proj, changed))
}

fn member_projection_is_open(
    projection: &MemberProjection,
    evaluator: &StaticEvaluator<'_>,
) -> Result<bool, Stop> {
    if let MemberProjectionSubject::Type(ty) = &projection.subject {
        if evaluator.is_open_type(*ty)? {
            return Ok(true);
        }
    }
    for arg in projection.arguments.iter() {
        if let MemberProjectionArgument::Kinded(crate::types::type_member::TypeMemberArgument::Type(ty)) = arg {
            if evaluator.is_open_type(*ty)? {
                return Ok(true);
            }
        }
    }
    if let Some(ev) = &projection.evidence {
        if let TypeTerm::Canonical(ty) = ev.subject {
            if evaluator.is_open_type(ty)? {
                return Ok(true);
            }
        }
        for &arg_ty in ev.trait_ref.arguments.iter() {
            if evaluator.is_open_type(arg_ty)? {
                return Ok(true);
            }
        }
    }
    Ok(false)
}
