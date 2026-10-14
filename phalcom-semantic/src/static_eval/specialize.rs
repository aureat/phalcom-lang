use crate::checker::proof_predicate::ProofPredicateGoal;
use crate::field_key::FieldKey;
use crate::identity::{DataComponentId, DataConstructorId, DeclarationId, VariantFieldId, VariantId};
use crate::member_projection::{MemberProjection, MemberProjectionArgument, MemberProjectionSubject, StaticMemberProjectionOutcome};
use crate::static_domain::StaticValueDomain;
use crate::static_eval::application::{StaticSpecializationArgument, StaticSpecializationEnvironmentError, StaticSpecializationEnvironmentKey};
use crate::static_eval::error::StaticEvalError;
use crate::static_eval::machine::{StaticArgument, StaticEvaluator, Stop};
use crate::static_eval::residual::{StaticEvalTerm, StaticResidualApplication, StaticResidualData, StaticResidualParameter, StaticResidualRef};
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
/// - `StaticResidualData::{Pack, Tuple, Record, Data, Variant}`
///
/// Enforces:
/// - Substitution closure before conflict checking
/// - Formal argument domain and static-value store validation
/// - Substitution through captured arguments and residual result domains
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

    let (residual_data, trusted_context) = {
        let store = evaluator.residuals().ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?;
        (
            store.get(residual).map_err(Stop::from)?.clone(),
            store.trusted_context(residual).map_err(Stop::from)?.clone(),
        )
    };

    let previous_context = evaluator.replace_trusted_branch_context(trusted_context.clone());
    let result = match residual_data {
        StaticResidualData::Parameter { parameter } => specialize_parameter(evaluator, residual, &parameter, supplied),
        StaticResidualData::Apply { application, result_domain } => specialize_apply(evaluator, residual, &application, result_domain, supplied),
        StaticResidualData::Proof {
            goal,
            result_type,
            environment,
        } => specialize_proof(evaluator, residual, &goal, result_type, &environment, supplied),
        StaticResidualData::MemberProjection {
            projection,
            result_domain,
            environment,
        } => specialize_member_projection(evaluator, residual, &projection, result_domain, &environment, supplied),
        StaticResidualData::Pack {
            positionals,
            labels,
            labeled_values,
            result_type,
        } => specialize_pack(evaluator, residual, &positionals, &labels, &labeled_values, result_type, supplied),
        StaticResidualData::Tuple { elements, result_type } => specialize_tuple(evaluator, residual, &elements, result_type, supplied),
        StaticResidualData::Record { fields, result_type } => specialize_record(evaluator, residual, &fields, result_type, supplied),
        StaticResidualData::Data {
            owner,
            constructor,
            result_type,
            components,
        } => specialize_data(evaluator, residual, &owner, &constructor, result_type, &components, supplied),
        StaticResidualData::Variant { variant, result_type, fields } => specialize_variant(evaluator, residual, &variant, result_type, &fields, supplied),
        _ => Err(Stop::invalid(StaticEvalError::UnsupportedExpression)),
    };
    let result = match result {
        Ok(StaticEvalTerm::Residual(reference)) if reference == residual && evaluator.trusted_branch_context() != &trusted_context => evaluator
            .reintern_residual_with_trusted_context(
                evaluator
                    .residuals()
                    .and_then(|store| store.get(residual).ok())
                    .cloned()
                    .ok_or_else(|| Stop::invalid(StaticEvalError::UnsupportedExpression))?,
                evaluator.trusted_branch_context().clone(),
            )
            .map(StaticEvalTerm::Residual),
        other => other,
    };
    evaluator.replace_trusted_branch_context(previous_context);
    result
}

fn specialize_parameter(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    parameter: &StaticResidualParameter,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    let inherited = evaluator.inherited_specialization_environment().clone();
    let composed = evaluator.compose_specialization_environment(&inherited, supplied)?;
    let (domain, domain_changed) = evaluator.substitute_specialization_domain(parameter.domain, &composed)?;

    let mut matching_argument = None;
    for (formal, argument) in composed.argument_bindings() {
        if formal.owner == parameter.owner && formal.index == parameter.index {
            if matching_argument.replace(*argument).is_some() {
                return Err(Stop::from(StaticSpecializationEnvironmentError::MalformedEnvironment));
            }
        }
    }
    if let Some(argument) = matching_argument {
        let argument = match argument {
            StaticSpecializationArgument::Kinded(ty) => {
                let ty = evaluator.substitute_specialization_type(ty, &composed)?;
                StaticSpecializationArgument::Kinded(ty)
            }
            StaticSpecializationArgument::Value(value) => StaticSpecializationArgument::Value(value),
        };
        return evaluator.validate_specialization_argument(domain, argument);
    }

    if !domain_changed {
        // No new binding supplied: reuse existing interned residual reference.
        return Ok(StaticEvalTerm::Residual(original));
    }

    let mut specialized_parameter = parameter.clone();
    specialized_parameter.domain = domain;
    let residual = evaluator.intern_parameter_residual_node(specialized_parameter)?;
    Ok(StaticEvalTerm::Residual(residual))
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
    let (result_domain, result_domain_changed) = evaluator.substitute_specialization_domain(result_domain, &composed_env)?;

    // 2. Recursively specialize arguments
    let mut specialized_args = Vec::with_capacity(application.arguments.len());
    let mut any_residual = false;
    let mut args_changed = false;

    for arg in application.arguments.iter() {
        match arg {
            StaticEvalTerm::Residual(r) => {
                let spec_term = specialize_residual_term(evaluator, *r, &composed_env)?;
                if spec_term != *arg {
                    args_changed = true;
                }
                if matches!(spec_term, StaticEvalTerm::Residual(_)) {
                    any_residual = true;
                }
                specialized_args.push(spec_term);
            }
            StaticEvalTerm::Kinded(ty) => {
                let specialized = evaluator.substitute_specialization_type(*ty, &composed_env)?;
                if specialized != *ty {
                    args_changed = true;
                }
                specialized_args.push(StaticEvalTerm::Kinded(specialized));
            }
            StaticEvalTerm::Value(value) => {
                evaluator.validate_static_value_reference(*value)?;
                specialized_args.push(StaticEvalTerm::Value(*value));
            }
        }
    }

    if any_residual || value_domain_is_open(result_domain, evaluator)? {
        // If no changes to arguments or environment, reuse exact existing residual
        if !args_changed && !result_domain_changed && composed_env == application.environment {
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

    // 3. Arguments and result domain are closed -> evaluate ordinary static definition
    let static_args = specialized_args
        .into_iter()
        .map(|term| match term {
            StaticEvalTerm::Kinded(k) => StaticArgument::Kinded(k),
            StaticEvalTerm::Value(v) => StaticArgument::Value(v),
            StaticEvalTerm::Residual(_) => unreachable!(),
        })
        .collect::<Vec<_>>();

    let result = evaluator.evaluate_specialized_application(application.definition.clone(), &static_args, composed_env)?;
    evaluator.validate_specialized_result(result, result_domain)
}

fn specialize_proof(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    goal: &ProofPredicateGoal,
    result_type: TypeId,
    environment: &StaticSpecializationEnvironmentKey,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    if supplied.is_empty() && evaluator.inherited_specialization_environment().is_empty() && goal_has_open_terms(goal, evaluator)? {
        return Ok(StaticEvalTerm::Residual(original));
    }
    // 1. Compose captured environment with supplied specialization
    let composed_env = evaluator.compose_specialization_environment(environment, supplied)?;

    // 2. Substitute goal propositions using substitution closure
    let (specialized_goal, changed) = evaluator.substitute_proof_goal(goal, &composed_env)?;
    let (specialized_result_domain, result_type_changed) =
        evaluator.substitute_specialization_domain(StaticValueDomain::ValueType(result_type), &composed_env)?;
    let StaticValueDomain::ValueType(specialized_result_type) = specialized_result_domain else {
        unreachable!();
    };
    let goal_is_open = goal_has_open_terms(&specialized_goal, evaluator)?;
    let result_type_is_open = evaluator.is_open_type(specialized_result_type)?;

    // Check No-New-Facts identity
    if !changed && !result_type_changed && composed_env == *environment && (goal_is_open || result_type_is_open) {
        return Ok(StaticEvalTerm::Residual(original));
    }

    // 3. Check if the goal or its declared result type still has open terms
    if goal_is_open || result_type_is_open {
        let new_ref = evaluator.intern_proof_residual_node(specialized_goal, specialized_result_type, composed_env)?;
        return Ok(StaticEvalTerm::Residual(new_ref));
    }

    // 4. Closed goal: evaluate using existing proof authority
    let outcome = evaluator.evaluate_proof_predicate_with_environment(&specialized_goal, specialized_result_type, composed_env);
    match outcome {
        Ok(true) => {
            let result = StaticEvalTerm::Value(evaluator.intern_bool(true));
            evaluator.validate_specialized_result(result, StaticValueDomain::ValueType(specialized_result_type))
        }
        Ok(false) => {
            let result = StaticEvalTerm::Value(evaluator.intern_bool(false));
            evaluator.validate_specialized_result(result, StaticValueDomain::ValueType(specialized_result_type))
        }
        Err(Stop::Symbolic(r)) => evaluator.validate_specialized_result(StaticEvalTerm::Residual(r), StaticValueDomain::ValueType(specialized_result_type)),
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
    if supplied.is_empty()
        && evaluator.inherited_specialization_environment().is_empty()
        && (member_projection_is_open(projection, evaluator)? || value_domain_is_open(result_domain, evaluator)?)
    {
        return Ok(StaticEvalTerm::Residual(original));
    }
    // 1. Compose captured environment with supplied specialization
    let composed_env = evaluator.compose_specialization_environment(environment, supplied)?;
    let (result_domain, result_domain_changed) = evaluator.substitute_specialization_domain(result_domain, &composed_env)?;

    // 2. Substitute subject and arguments
    let (specialized_projection, changed) = evaluator.substitute_member_projection(projection, &composed_env)?;
    if specialized_projection.result_domain != result_domain {
        return Err(Stop::invalid(StaticEvalError::InvalidResultDomain));
    }

    // Check No-New-Facts identity
    if !changed && !result_domain_changed && composed_env == *environment {
        if member_projection_is_open(&specialized_projection, evaluator)? {
            return Ok(StaticEvalTerm::Residual(original));
        }
    }

    // 3. If still open: re-intern MemberProjection
    if member_projection_is_open(&specialized_projection, evaluator)? || value_domain_is_open(result_domain, evaluator)? {
        let new_ref = evaluator.intern_member_projection_residual_node(specialized_projection, result_domain, composed_env)?;
        return Ok(StaticEvalTerm::Residual(new_ref));
    }

    // 4. Closed: evaluate using existing StructuralReflectionProjector
    let outcome = evaluator.project_member_with_environment(&specialized_projection, composed_env.clone())?;
    match outcome {
        StaticMemberProjectionOutcome::ReducedValue(val) => evaluator.validate_specialized_result(StaticEvalTerm::Value(val), result_domain),
        StaticMemberProjectionOutcome::ReducedKinded(ty) => evaluator.validate_specialized_result(StaticEvalTerm::Kinded(ty), result_domain),
        StaticMemberProjectionOutcome::Stuck(stuck_proj) => {
            if stuck_proj.member != specialized_projection.member
                || stuck_proj.arguments != specialized_projection.arguments
                || stuck_proj.result_domain != result_domain
            {
                return Err(Stop::invalid(StaticEvalError::InvalidResultDomain));
            }
            let new_ref = evaluator.intern_member_projection_residual_node(stuck_proj, result_domain, composed_env)?;
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

fn specialize_pack(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    positionals: &[StaticEvalTerm],
    labels: &[Box<str>],
    labeled_values: &[StaticEvalTerm],
    result_type: TypeId,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    if labels.len() != labeled_values.len() {
        return Err(Stop::invalid(StaticEvalError::InvalidResultDomain));
    }
    let inherited = evaluator.inherited_specialization_environment().clone();
    let composed = evaluator.compose_specialization_environment(&inherited, supplied)?;
    let (domain, type_changed) = evaluator.substitute_specialization_domain(StaticValueDomain::ValueType(result_type), &composed)?;
    let StaticValueDomain::ValueType(result_type) = domain else {
        return Err(Stop::invalid(StaticEvalError::InvalidResultDomain));
    };
    let mut changed = type_changed;
    let mut symbolic = false;
    let mut specialize_lane_term = |element: StaticEvalTerm| -> Result<StaticEvalTerm, Stop> {
        if evaluator.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        evaluator.charge_step()?;
        let term = match element {
            StaticEvalTerm::Value(value) => {
                evaluator.validate_static_value_reference(value)?;
                StaticEvalTerm::Value(value)
            }
            StaticEvalTerm::Kinded(ty) => {
                let ty = evaluator.substitute_specialization_type(ty, &composed)?;
                StaticEvalTerm::Value(evaluator.intern_kinded_static_value(ty)?)
            }
            StaticEvalTerm::Residual(reference) => specialize_residual_term(evaluator, reference, &composed)?,
        };
        changed |= term != element;
        symbolic |= matches!(term, StaticEvalTerm::Residual(_));
        Ok(term)
    };
    let positionals = positionals.iter().copied().map(&mut specialize_lane_term).collect::<Result<Vec<_>, _>>()?;
    let labeled_values = labeled_values.iter().copied().map(&mut specialize_lane_term).collect::<Result<Vec<_>, _>>()?;
    if symbolic {
        if !changed {
            return Ok(StaticEvalTerm::Residual(original));
        }
        let residual = evaluator.intern_structural_residual_node(StaticResidualData::Pack {
            positionals: positionals.into_boxed_slice(),
            labels: labels.into(),
            labeled_values: labeled_values.into_boxed_slice(),
            result_type,
        })?;
        return Ok(StaticEvalTerm::Residual(residual));
    }
    let positionals = positionals
        .into_iter()
        .map(|term| match term {
            StaticEvalTerm::Value(value) => Ok(value),
            _ => Err(Stop::invalid(StaticEvalError::InvalidResultDomain)),
        })
        .collect::<Result<Vec<_>, Stop>>()?;
    let labeled = labels
        .iter()
        .cloned()
        .zip(labeled_values)
        .map(|(label, term)| match term {
            StaticEvalTerm::Value(value) => Ok((label, value)),
            _ => Err(Stop::invalid(StaticEvalError::InvalidResultDomain)),
        })
        .collect::<Result<Vec<_>, Stop>>()?;
    let value = evaluator.intern_pack_static_values(positionals, labeled)?;
    evaluator.validate_specialized_result(StaticEvalTerm::Value(value), domain)
}

fn specialize_tuple(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    elements: &[StaticEvalTerm],
    result_type: TypeId,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    let inherited = evaluator.inherited_specialization_environment().clone();
    let composed = evaluator.compose_specialization_environment(&inherited, supplied)?;
    let (domain, type_changed) = evaluator.substitute_specialization_domain(StaticValueDomain::ValueType(result_type), &composed)?;
    let StaticValueDomain::ValueType(result_type) = domain else {
        return Err(Stop::invalid(StaticEvalError::InvalidResultDomain));
    };
    let mut changed = type_changed;
    let mut symbolic = false;
    let mut specialized = Vec::with_capacity(elements.len());
    for &element in elements {
        if evaluator.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        evaluator.charge_step()?;
        let term = match element {
            StaticEvalTerm::Value(value) => {
                evaluator.validate_static_value_reference(value)?;
                StaticEvalTerm::Value(value)
            }
            StaticEvalTerm::Kinded(ty) => {
                let ty = evaluator.substitute_specialization_type(ty, &composed)?;
                StaticEvalTerm::Value(evaluator.intern_kinded_static_value(ty)?)
            }
            StaticEvalTerm::Residual(reference) => specialize_residual_term(evaluator, reference, &composed)?,
        };
        changed |= term != element;
        symbolic |= matches!(term, StaticEvalTerm::Residual(_));
        specialized.push(term);
    }
    if symbolic {
        if !changed {
            return Ok(StaticEvalTerm::Residual(original));
        }
        let residual = evaluator.intern_tuple_residual_node(specialized.into_boxed_slice(), result_type)?;
        return Ok(StaticEvalTerm::Residual(residual));
    }
    let values = specialized
        .into_iter()
        .map(|term| match term {
            StaticEvalTerm::Value(value) => value,
            _ => unreachable!("all tuple elements were reduced"),
        })
        .collect();
    let value = evaluator.intern_tuple_static_values(values)?;
    evaluator.validate_specialized_result(StaticEvalTerm::Value(value), domain)
}

fn specialize_record(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    fields: &[(FieldKey, StaticEvalTerm)],
    result_type: TypeId,
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    let inherited = evaluator.inherited_specialization_environment().clone();
    let composed = evaluator.compose_specialization_environment(&inherited, supplied)?;
    let (domain, type_changed) = evaluator.substitute_specialization_domain(StaticValueDomain::ValueType(result_type), &composed)?;
    let StaticValueDomain::ValueType(result_type) = domain else {
        return Err(Stop::invalid(StaticEvalError::InvalidResultDomain));
    };
    let mut changed = type_changed;
    let mut symbolic = false;
    let mut specialized = Vec::with_capacity(fields.len());
    for (key, element) in fields {
        let element = *element;
        if evaluator.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        evaluator.charge_step()?;
        let term = match element {
            StaticEvalTerm::Value(value) => {
                evaluator.validate_static_value_reference(value)?;
                StaticEvalTerm::Value(value)
            }
            StaticEvalTerm::Kinded(ty) => {
                let ty = evaluator.substitute_specialization_type(ty, &composed)?;
                StaticEvalTerm::Value(evaluator.intern_kinded_static_value(ty)?)
            }
            StaticEvalTerm::Residual(reference) => specialize_residual_term(evaluator, reference, &composed)?,
        };
        changed |= term != element;
        symbolic |= matches!(term, StaticEvalTerm::Residual(_));
        specialized.push((key.clone(), term));
    }
    if symbolic {
        if !changed {
            return Ok(StaticEvalTerm::Residual(original));
        }
        let residual = evaluator.intern_record_residual_node(specialized, result_type)?;
        return Ok(StaticEvalTerm::Residual(residual));
    }
    let values = specialized
        .into_iter()
        .map(|(key, term)| match term {
            StaticEvalTerm::Value(value) => (key, value),
            _ => unreachable!("all Record fields were reduced"),
        })
        .collect();
    let value = evaluator.intern_record_static_values(values)?;
    evaluator.validate_specialized_result(StaticEvalTerm::Value(value), domain)
}

fn specialize_structural_fields<T: Clone>(
    evaluator: &mut StaticEvaluator<'_>,
    fields: &[(T, StaticEvalTerm)],
    environment: &StaticSpecializationEnvironmentKey,
) -> Result<(Vec<(T, StaticEvalTerm)>, bool, bool), Stop> {
    let mut changed = false;
    let mut symbolic = false;
    let mut result = Vec::with_capacity(fields.len());
    for (id, element) in fields {
        let element = *element;
        if evaluator.is_cancelled() {
            return Err(Stop::Cancelled);
        }
        evaluator.charge_step()?;
        let term = match element {
            StaticEvalTerm::Value(value) => {
                evaluator.validate_static_value_reference(value)?;
                StaticEvalTerm::Value(value)
            }
            StaticEvalTerm::Kinded(ty) => {
                let ty = evaluator.substitute_specialization_type(ty, environment)?;
                StaticEvalTerm::Value(evaluator.intern_kinded_static_value(ty)?)
            }
            StaticEvalTerm::Residual(reference) => specialize_residual_term(evaluator, reference, environment)?,
        };
        changed |= term != element;
        symbolic |= matches!(term, StaticEvalTerm::Residual(_));
        result.push((id.clone(), term));
    }
    Ok((result, changed, symbolic))
}

fn specialize_data(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    owner: &DeclarationId,
    constructor: &DataConstructorId,
    result_type: TypeId,
    components: &[(DataComponentId, StaticEvalTerm)],
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    let inherited = evaluator.inherited_specialization_environment().clone();
    let composed = evaluator.compose_specialization_environment(&inherited, supplied)?;
    let (domain, type_changed) = evaluator.substitute_specialization_domain(StaticValueDomain::ValueType(result_type), &composed)?;
    let StaticValueDomain::ValueType(result_type) = domain else {
        return Err(Stop::invalid(StaticEvalError::InvalidResultDomain));
    };
    let (components, changed, symbolic) = specialize_structural_fields(evaluator, components, &composed)?;
    if symbolic {
        if !changed && !type_changed {
            return Ok(StaticEvalTerm::Residual(original));
        }
        let residual = evaluator.intern_structural_residual_node(StaticResidualData::Data {
            owner: owner.clone(),
            constructor: constructor.clone(),
            result_type,
            components: components.into_boxed_slice(),
        })?;
        return Ok(StaticEvalTerm::Residual(residual));
    }
    let values = components
        .into_iter()
        .map(|(id, term)| match term {
            StaticEvalTerm::Value(value) => Ok((id, value)),
            _ => Err(Stop::invalid(StaticEvalError::InvalidResultDomain)),
        })
        .collect::<Result<Vec<_>, Stop>>()?;
    let value = evaluator.intern_data_static_value(owner, constructor, result_type, values)?;
    evaluator.validate_specialized_result(StaticEvalTerm::Value(value), domain)
}

fn specialize_variant(
    evaluator: &mut StaticEvaluator<'_>,
    original: StaticResidualRef,
    variant: &VariantId,
    result_type: TypeId,
    fields: &[(VariantFieldId, StaticEvalTerm)],
    supplied: &StaticSpecializationEnvironmentKey,
) -> Result<StaticEvalTerm, Stop> {
    let inherited = evaluator.inherited_specialization_environment().clone();
    let composed = evaluator.compose_specialization_environment(&inherited, supplied)?;
    let (domain, type_changed) = evaluator.substitute_specialization_domain(StaticValueDomain::ValueType(result_type), &composed)?;
    let StaticValueDomain::ValueType(result_type) = domain else {
        return Err(Stop::invalid(StaticEvalError::InvalidResultDomain));
    };
    let (fields, changed, symbolic) = specialize_structural_fields(evaluator, fields, &composed)?;
    if symbolic {
        if !changed && !type_changed {
            return Ok(StaticEvalTerm::Residual(original));
        }
        let residual = evaluator.intern_structural_residual_node(StaticResidualData::Variant {
            variant: variant.clone(),
            result_type,
            fields: fields.into_boxed_slice(),
        })?;
        return Ok(StaticEvalTerm::Residual(residual));
    }
    let values = fields
        .into_iter()
        .map(|(id, term)| match term {
            StaticEvalTerm::Value(value) => Ok((id, value)),
            _ => Err(Stop::invalid(StaticEvalError::InvalidResultDomain)),
        })
        .collect::<Result<Vec<_>, Stop>>()?;
    let value = evaluator.intern_variant_static_value(variant, result_type, values)?;
    evaluator.validate_specialized_result(StaticEvalTerm::Value(value), domain)
}

pub(crate) fn substitute_proof_goal(
    goal: &ProofPredicateGoal,
    subst: &TypeSubstitution,
    types: &mut TypeStore,
) -> Result<(ProofPredicateGoal, bool), StaticSpecializationEnvironmentError> {
    match goal {
        ProofPredicateGoal::Equivalent { left, right } => {
            let sub_left = subst
                .try_apply(types, *left)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let sub_right = subst
                .try_apply(types, *right)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let changed = sub_left != *left || sub_right != *right;
            let (min, max) = if sub_left <= sub_right {
                (sub_left, sub_right)
            } else {
                (sub_right, sub_left)
            };
            Ok((ProofPredicateGoal::Equivalent { left: min, right: max }, changed))
        }
        ProofPredicateGoal::Subtype { lower, upper } => {
            let sub_lower = subst
                .try_apply(types, *lower)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let sub_upper = subst
                .try_apply(types, *upper)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            let changed = sub_lower != *lower || sub_upper != *upper;
            Ok((
                ProofPredicateGoal::Subtype {
                    lower: sub_lower,
                    upper: sub_upper,
                },
                changed,
            ))
        }
        ProofPredicateGoal::Conforms { key } => {
            let mut changed = false;
            let sub_subject = match &key.subject {
                TypeTerm::Canonical(ty) => {
                    let sub_ty = subst
                        .try_apply(types, *ty)
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
                let sub_arg = subst
                    .try_apply(types, arg_ty)
                    .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
                if sub_arg != arg_ty {
                    changed = true;
                }
                sub_args.push(sub_arg);
            }
            let sub_key = crate::generic_evidence::GenericConformanceKey::new(sub_subject, TraitRef::new(key.trait_ref.declaration.clone(), sub_args));
            Ok((ProofPredicateGoal::Conforms { key: sub_key }, changed))
        }
    }
}

fn goal_has_open_terms(goal: &ProofPredicateGoal, evaluator: &StaticEvaluator<'_>) -> Result<bool, Stop> {
    match goal {
        ProofPredicateGoal::Equivalent { left, right } => Ok(evaluator.is_open_type(*left)? || evaluator.is_open_type(*right)?),
        ProofPredicateGoal::Subtype { lower, upper } => Ok(evaluator.is_open_type(*lower)? || evaluator.is_open_type(*upper)?),
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
            let sub_ty = subst
                .try_apply(types, *ty)
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
                let sub_ty = subst
                    .try_apply(types, *ty)
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
                    let sub_ty = subst
                        .try_apply(types, *ty)
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
                let sub_arg = subst
                    .try_apply(types, arg_ty)
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
    let sub_result_domain = substitute_value_domain(projection.result_domain, subst, types)?;
    if sub_result_domain != projection.result_domain {
        changed = true;
    }
    let sub_proj = MemberProjection {
        subject: sub_subject,
        member: projection.member.clone(),
        arguments: sub_args.into_boxed_slice(),
        evidence: sub_evidence,
        result_domain: sub_result_domain,
    };
    Ok((sub_proj, changed))
}

fn member_projection_is_open(projection: &MemberProjection, evaluator: &StaticEvaluator<'_>) -> Result<bool, Stop> {
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
    value_domain_is_open(projection.result_domain, evaluator)
}

fn value_domain_is_open(domain: StaticValueDomain, evaluator: &StaticEvaluator<'_>) -> Result<bool, Stop> {
    match domain {
        StaticValueDomain::Kind(_) => Ok(false),
        StaticValueDomain::ValueType(ty) => evaluator.is_open_type(ty),
    }
}

fn substitute_value_domain(
    domain: StaticValueDomain,
    subst: &TypeSubstitution,
    types: &mut TypeStore,
) -> Result<StaticValueDomain, StaticSpecializationEnvironmentError> {
    match domain {
        StaticValueDomain::Kind(kind) => Ok(StaticValueDomain::Kind(kind)),
        StaticValueDomain::ValueType(ty) => {
            let specialized = subst
                .try_apply(types, ty)
                .map_err(|_| StaticSpecializationEnvironmentError::UnsupportedCanonicalSubstitution)?;
            if specialized.index() >= types.len() || types.kind_of(specialized) != crate::types::id::KindId::TYPE {
                return Err(StaticSpecializationEnvironmentError::ArgumentDomainMismatch);
            }
            Ok(StaticValueDomain::ValueType(specialized))
        }
    }
}
