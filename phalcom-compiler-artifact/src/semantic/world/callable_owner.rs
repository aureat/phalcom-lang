//! Cross-links owned by the complete callable payload.
use super::*;

pub(super) fn validate_callable_owner(
    analysis: &CallableAnalysisRecordV1,
    world: &SemanticWorldV3,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<(), ArtifactError> {
    if [
        analysis.diagnostics.len(),
        analysis.dependencies.len(),
        analysis.semantic_dependencies.len(),
        analysis.explanations.len(),
        analysis.diagnostic_cause_count as usize,
    ]
    .into_iter()
    .any(|len| len > limits.max_semantic_entries_per_family)
    {
        return Err(invalid("callable owner exceeds its family limit"));
    }
    let explanation = |id: u32| {
        if id as usize >= analysis.explanations.len() {
            Err(invalid("callable explanation ordinal is absent"))
        } else {
            Ok(())
        }
    };
    for id in analysis
        .return_explanation
        .into_iter()
        .chain(analysis.expressions.iter().filter_map(|row| row.explanation))
        .chain(analysis.bindings.iter().filter_map(|row| row.explanation))
    {
        explanation(id)?;
    }
    let mut used = allocations.vector::<bool>(analysis.diagnostic_cause_count as usize)?;
    used.resize(analysis.diagnostic_cause_count as usize, false);
    let mut cause = |id: u32| -> Result<(), ArtifactError> {
        *used
            .get_mut(id as usize)
            .ok_or_else(|| invalid("callable diagnostic cause ordinal is absent"))? = true;
        Ok(())
    };
    let status_cause = |status: &CallableExpressionStatusV1| match status {
        CallableExpressionStatusV1::Invalid { cause }
        | CallableExpressionStatusV1::Suppressed {
            cause: CallableSuppressionCauseV1::One { cause },
        } => Some(*cause),
        _ => None,
    };
    for row in &analysis.expressions {
        if let Some(id) = status_cause(&row.status) {
            cause(id)?;
        }
        if let CallableCausalInvalidityV1::One { cause: id } = row.causal_invalidity {
            cause(id)?;
        }
    }
    for row in &analysis.bindings {
        if let CallableCausalInvalidityV1::One { cause: id } = row.causal_invalidity {
            cause(id)?;
        }
    }
    for exit in &analysis.normal_returns {
        if let Some(id) = status_cause(&exit.status) {
            cause(id)?;
        }
        if let CallableCausalInvalidityV1::One { cause: id } = exit.causal_invalidity {
            cause(id)?;
        }
        if let Some(dependency) = &exit.direct_result_dependency {
            if !analysis.dependencies.contains(dependency) {
                return Err(invalid("return dependency is absent from callable owner"));
            }
        }
    }
    for flow in std::iter::once(&analysis.entry_flow)
        .chain(analysis.normal_returns.iter().map(|exit| &exit.flow))
        .chain(analysis.throws.iter())
    {
        for field in &flow.fields {
            if let CallableCausalInvalidityV1::One { cause: id } = field.causal_invalidity {
                cause(id)?;
            }
        }
    }
    for diagnostic in &analysis.diagnostics {
        validate_witness_diagnostic(diagnostic, &world.callable_signatures, &world.type_graph, limits)?;
        if let Some(id) = diagnostic.root_cause {
            cause(id)?;
        }
        for reference in &diagnostic.explanations {
            if reference.callable != analysis.callable {
                return Err(invalid("diagnostic explanation belongs to another callable"));
            }
            explanation(reference.ordinal)?;
        }
    }
    for node in &analysis.explanations {
        for evidence in &node.evidence {
            if let CallableExplanationEvidenceV1::Suppressed { cause: id } = evidence {
                cause(*id)?;
            }
        }
    }
    if used.iter().any(|present| !present) {
        return Err(invalid("callable diagnostic cause ordinals must be dense"));
    }
    for dependency in &analysis.semantic_dependencies {
        validate_dependency(dependency, world, limits)?;
    }
    Ok(())
}

fn validate_dependency(dependency: &StaticSemanticDependencyV1, world: &SemanticWorldV3, limits: &ValidationLimits) -> Result<(), ArtifactError> {
    use StaticSemanticDependencyV1 as D;
    match dependency {
        D::GenericConformance { subject, trait_ref } => {
            validate_type_term(subject, &world.type_graph)?;
            for arg in &trait_ref.arguments {
                validate_type_id(*arg, &world.type_graph)?;
            }
        }
        D::TraitDispatchBucket { selector, .. } | D::StaticCaseSignature { selector, .. } => validate_static_case_selector(selector, limits)?,
        D::TypeMemberDeclaration { selector, .. } => validate_selector_base(selector, limits)?,
        D::FieldSignature { field } if field.name.is_empty() || field.name.len() > limits.max_string_bytes => {
            return Err(invalid("dependency field name is invalid"));
        }
        D::LinkedName { name, .. } | D::PublicExport { name, .. } if name.is_empty() || name.len() > limits.max_string_bytes => {
            return Err(invalid("dependency name is invalid"));
        }
        _ => {}
    }
    Ok(())
}
