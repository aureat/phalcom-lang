//! Lossless P7 semantic-diagnostic reconstruction for artifact-backed authority.
//!
//! Diagnostics remain part of semantic products. This importer preserves stable
//! codes, source ownership, labels, guidance, explanation references, fixes,
//! and root-cause identities under the same allocation transaction as the
//! surrounding semantic image.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::diagnostic::{
    DiagnosticCode, DiagnosticFix, DiagnosticGuidance, DiagnosticLabel, DiagnosticSeverity,
    ExplanationRef, SemanticDiagnostic, SemanticSourceSpan,
};
use crate::identity::{DiagnosticCauseId, ExplanationId};
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::module_world::SpanV1;
use phalcom_compiler_artifact::semantic::world::{
    CallableDiagnosticSeverityV1, CallableEvidenceDiagnosticGuidanceV1, CallableEvidenceDiagnosticV1,
    EnumRequirementSourceV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

pub(crate) fn reconstruct_semantic_diagnostic(
    record: &CallableEvidenceDiagnosticV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<SemanticDiagnostic, ArtifactError> {
    let code = DiagnosticCode::from_stable_code(&record.code)
        .ok_or_else(|| invalid("semantic diagnostic carries an unknown stable code"))?;
    let primary = source_span(&record.primary, owners)?;
    let primary_range = range(&record.primary_range, "semantic diagnostic primary")?;
    if primary.range != primary_range {
        return Err(invalid("semantic diagnostic primary range disagrees with its source span"));
    }

    let mut labels = allocations
        .vector::<DiagnosticLabel>(record.labels.len())
        .map_err(ArtifactError::Allocation)?;
    for label in &record.labels {
        let span = source_span(&label.source, owners)?;
        let label_range = range(&label.range, "semantic diagnostic label")?;
        if span.range != label_range {
            return Err(invalid("semantic diagnostic label range disagrees with its source span"));
        }
        labels.push(DiagnosticLabel {
            span,
            range: label_range,
            message: allocations.string(&label.message).map_err(ArtifactError::Allocation)?,
        });
    }

    let mut notes = allocations
        .vector::<String>(record.notes.len())
        .map_err(ArtifactError::Allocation)?;
    for note in &record.notes {
        notes.push(allocations.string(note).map_err(ArtifactError::Allocation)?);
    }

    let mut helps = allocations
        .vector::<String>(record.helps.len())
        .map_err(ArtifactError::Allocation)?;
    for help in &record.helps {
        helps.push(allocations.string(help).map_err(ArtifactError::Allocation)?);
    }

    let mut guidance = allocations
        .vector::<DiagnosticGuidance>(record.guidance.len())
        .map_err(ArtifactError::Allocation)?;
    for item in &record.guidance {
        guidance.push(match item {
            CallableEvidenceDiagnosticGuidanceV1::ChangeAnnotation { range: source_range, ty } => {
                DiagnosticGuidance::ChangeAnnotation {
                    range: range(source_range, "diagnostic annotation guidance")?,
                    ty: type_id(*ty, types)?,
                }
            }
            CallableEvidenceDiagnosticGuidanceV1::SupplyAssignableValue { expected } => {
                DiagnosticGuidance::SupplyAssignableValue {
                    expected: type_id(*expected, types)?,
                }
            }
            CallableEvidenceDiagnosticGuidanceV1::UseCallableShape { callable } => {
                DiagnosticGuidance::UseCallableShape {
                    callable: owners.callable(callable)?,
                }
            }
            CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { range: source_range, expected } => {
                DiagnosticGuidance::EstablishTypeEvidence {
                    range: range(source_range, "diagnostic evidence guidance")?,
                    expected: expected.map(|ty| type_id(ty, types)).transpose()?,
                }
            }
            CallableEvidenceDiagnosticGuidanceV1::ResolveGenericParameter { parameter } => {
                DiagnosticGuidance::ResolveGenericParameter {
                    parameter: *types
                        .parameters
                        .get(parameter.0 as usize)
                        .ok_or_else(|| invalid("diagnostic guidance references an absent type parameter"))?,
                }
            }
        });
    }

    let mut explanations = allocations
        .vector::<ExplanationRef>(record.explanations.len())
        .map_err(ArtifactError::Allocation)?;
    for reference in &record.explanations {
        explanations.push(ExplanationRef::new(
            owners.callable(&reference.callable)?,
            ExplanationId(reference.ordinal),
        ));
    }

    let mut fixes = allocations
        .vector::<DiagnosticFix>(record.fixes.len())
        .map_err(ArtifactError::Allocation)?;
    for fix in &record.fixes {
        let replacement = fix
            .replacement
            .as_ref()
            .map(|replacement| -> Result<(SourceRange, String), ArtifactError> {
                Ok((
                    range(&replacement.range, "diagnostic fix replacement")?,
                    allocations
                        .string(&replacement.text)
                        .map_err(ArtifactError::Allocation)?,
                ))
            })
            .transpose()?;
        fixes.push(DiagnosticFix {
            message: allocations.string(&fix.message).map_err(ArtifactError::Allocation)?,
            replacement,
        });
    }

    Ok(SemanticDiagnostic {
        code,
        severity: match &record.severity {
            CallableDiagnosticSeverityV1::Error => DiagnosticSeverity::Error,
            CallableDiagnosticSeverityV1::Warning => DiagnosticSeverity::Warning,
            CallableDiagnosticSeverityV1::Information => DiagnosticSeverity::Information,
            CallableDiagnosticSeverityV1::Hint => DiagnosticSeverity::Hint,
        },
        message: allocations.string(&record.message).map_err(ArtifactError::Allocation)?,
        primary,
        primary_range,
        labels,
        notes,
        helps,
        guidance,
        explanations,
        fixes,
        root_cause: record.root_cause.map(DiagnosticCauseId),
    })
}

pub(crate) fn source_span(
    source: &EnumRequirementSourceV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<SemanticSourceSpan, ArtifactError> {
    Ok(SemanticSourceSpan::new(
        owners.module(&source.module)?,
        range(&source.range, "semantic source span")?,
    ))
}

pub(crate) fn range(
    span: &SpanV1,
    family: &str,
) -> Result<SourceRange, ArtifactError> {
    if span.start > span.end {
        return Err(invalid(format!("{family} is inverted")));
    }
    Ok(SourceRange {
        start: usize::try_from(span.start)
            .map_err(|_| invalid(format!("{family} start exceeds address space")))?,
        end: usize::try_from(span.end)
            .map_err(|_| invalid(format!("{family} end exceeds address space")))?,
    })
}

fn type_id(
    id: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId,
    types: &ReconstructedTypeGraph,
) -> Result<crate::types::TypeId, ArtifactError> {
    types
        .types
        .get(id.0 as usize)
        .copied()
        .ok_or_else(|| invalid("semantic diagnostic references an absent type"))
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
