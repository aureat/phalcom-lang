//! Allocation-accounted reconstruction of callable diagnostics.

use super::*;
use crate::diagnostic::{DiagnosticFix, DiagnosticGuidance, DiagnosticLabel, DiagnosticSeverity, ExplanationRef, SemanticDiagnostic};
use phalcom_compiler_artifact::semantic::world::{
    CallableDiagnosticSeverityV1, CallableEvidenceDiagnosticGuidanceV1, CallableEvidenceDiagnosticV1, EnumRequirementSourceV1,
};

fn source_span(source: &EnumRequirementSourceV1, allocations: &mut AllocationBudget) -> Result<SemanticSourceSpan, ArtifactError> {
    let module = import_stable_module_with_allocation_budget(&source.module, allocations)?;
    if source.range.start > source.range.end {
        return Err(referential("callable source span is inverted"));
    }
    Ok(SemanticSourceSpan::new(
        module,
        SourceRange {
            start: usize::try_from(source.range.start).map_err(|_| referential("callable source start exceeds address space"))?,
            end: usize::try_from(source.range.end).map_err(|_| referential("callable source end exceeds address space"))?,
        },
    ))
}

pub(super) fn import_diagnostic_with_allocation_budget(
    record: &CallableEvidenceDiagnosticV1,
    relocation: &TypeRelocation,
    allocations: &mut AllocationBudget,
) -> Result<SemanticDiagnostic, ArtifactError> {
    let range = |range: &phalcom_compiler_artifact::module_world::SpanV1| -> Result<SourceRange, ArtifactError> {
        if range.start > range.end {
            return Err(referential("trait surface diagnostic range is inverted"));
        }
        Ok(SourceRange {
            start: usize::try_from(range.start).map_err(|_| referential("trait surface diagnostic range start exceeds address space"))?,
            end: usize::try_from(range.end).map_err(|_| referential("trait surface diagnostic range end exceeds address space"))?,
        })
    };
    let code = DiagnosticCode::from_stable_code(&record.code).ok_or_else(|| referential("trait surface diagnostic has an unknown stable code"))?;
    let primary = source_span(&record.primary, allocations)?;
    let primary_range = range(&record.primary_range)?;
    if primary.range != primary_range {
        return Err(referential("trait surface diagnostic primary range disagrees with its source span"));
    }
    let mut labels = allocations.vector(record.labels.len()).map_err(allocation_error)?;
    for label in &record.labels {
        labels.push(DiagnosticLabel {
            span: source_span(&label.source, allocations)?,
            range: range(&label.range)?,
            message: allocations.string(&label.message).map_err(allocation_error)?,
        });
    }
    let mut guidance = allocations.vector(record.guidance.len()).map_err(allocation_error)?;
    for item in &record.guidance {
        guidance.push(match item {
            CallableEvidenceDiagnosticGuidanceV1::ChangeAnnotation { range: source_range, ty } => DiagnosticGuidance::ChangeAnnotation {
                range: range(source_range)?,
                ty: relocation.ty(*ty)?,
            },
            CallableEvidenceDiagnosticGuidanceV1::SupplyAssignableValue { expected } => DiagnosticGuidance::SupplyAssignableValue {
                expected: relocation.ty(*expected)?,
            },
            CallableEvidenceDiagnosticGuidanceV1::UseCallableShape { callable } => DiagnosticGuidance::UseCallableShape {
                callable: import_stable_callable_with_allocation_budget(callable, allocations)?,
            },
            CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { range: source_range, expected } => DiagnosticGuidance::EstablishTypeEvidence {
                range: range(source_range)?,
                expected: expected.map(|ty| relocation.ty(ty)).transpose()?,
            },
            CallableEvidenceDiagnosticGuidanceV1::ResolveGenericParameter { parameter } => DiagnosticGuidance::ResolveGenericParameter {
                parameter: relocation.parameter(*parameter)?,
            },
        });
    }
    let mut explanations = allocations.vector(record.explanations.len()).map_err(allocation_error)?;
    for reference in &record.explanations {
        explanations.push(ExplanationRef::new(
            import_stable_callable_with_allocation_budget(&reference.callable, allocations)?,
            crate::identity::ExplanationId(reference.ordinal),
        ));
    }
    let mut fixes = allocations.vector(record.fixes.len()).map_err(allocation_error)?;
    for fix in &record.fixes {
        fixes.push(DiagnosticFix {
            message: allocations.string(&fix.message).map_err(allocation_error)?,
            replacement: fix
                .replacement
                .as_ref()
                .map(|replacement| Ok::<_, ArtifactError>((range(&replacement.range)?, allocations.string(&replacement.text).map_err(allocation_error)?)))
                .transpose()?,
        });
    }
    let mut notes = allocations.vector(record.notes.len()).map_err(allocation_error)?;
    for note in &record.notes {
        notes.push(allocations.string(note).map_err(allocation_error)?);
    }
    let mut helps = allocations.vector(record.helps.len()).map_err(allocation_error)?;
    for help in &record.helps {
        helps.push(allocations.string(help).map_err(allocation_error)?);
    }
    Ok(SemanticDiagnostic {
        code,
        severity: match record.severity {
            CallableDiagnosticSeverityV1::Error => DiagnosticSeverity::Error,
            CallableDiagnosticSeverityV1::Warning => DiagnosticSeverity::Warning,
            CallableDiagnosticSeverityV1::Information => DiagnosticSeverity::Information,
            CallableDiagnosticSeverityV1::Hint => DiagnosticSeverity::Hint,
        },
        message: allocations.string(&record.message).map_err(allocation_error)?,
        primary,
        primary_range,
        labels,
        notes,
        helps,
        guidance,
        explanations,
        fixes,
        root_cause: record.root_cause.map(crate::identity::DiagnosticCauseId),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use phalcom_compiler_artifact::semantic::type_store::{ArtifactTypeId, ArtifactTypeParameterId};
    use phalcom_compiler_artifact::semantic::world as wire;

    fn fixture() -> (CallableEvidenceDiagnosticV1, TypeRelocation) {
        let module = crate::identity::ModuleId::universe(crate::identity::ModulePath::from_components(vec![
            crate::identity::ModuleComponent::from_identifier("diagnostic_budget").unwrap(),
        ]));
        let owner = crate::identity::DeclarationId::new(module.clone(), "Packet".into());
        let callable = crate::identity::CallableId::method(owner, phalcom_common::selector::Selector::getter("inspect").unwrap(), DispatchSide::Instance);
        let callable = crate::metadata::stable_identity::to_stable_callable(&callable);
        let span = || phalcom_compiler_artifact::module_world::SpanV1 { start: 4, end: 12 };
        let source = || EnumRequirementSourceV1 {
            module: crate::metadata::stable_identity::to_stable_module(&module),
            range: span(),
        };
        let record = CallableEvidenceDiagnosticV1 {
            code: DiagnosticCode::TypeMismatch.as_str().into(),
            severity: CallableDiagnosticSeverityV1::Warning,
            message: "expected another type".into(),
            primary: source(),
            primary_range: span(),
            labels: vec![wire::CallableEvidenceDiagnosticLabelV1 {
                source: source(),
                range: span(),
                message: "value supplied here".into(),
            }],
            notes: vec!["a diagnostic note".into()],
            helps: vec!["a diagnostic help".into()],
            guidance: vec![
                CallableEvidenceDiagnosticGuidanceV1::ChangeAnnotation {
                    range: span(),
                    ty: ArtifactTypeId(0),
                },
                CallableEvidenceDiagnosticGuidanceV1::SupplyAssignableValue { expected: ArtifactTypeId(0) },
                CallableEvidenceDiagnosticGuidanceV1::UseCallableShape { callable: callable.clone() },
                CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence {
                    range: span(),
                    expected: Some(ArtifactTypeId(0)),
                },
                CallableEvidenceDiagnosticGuidanceV1::EstablishTypeEvidence { range: span(), expected: None },
                CallableEvidenceDiagnosticGuidanceV1::ResolveGenericParameter {
                    parameter: ArtifactTypeParameterId(0),
                },
            ],
            explanations: vec![wire::CallableEvidenceExplanationRefV1 { callable, ordinal: 19 }],
            fixes: vec![
                wire::CallableEvidenceDiagnosticFixV1 {
                    message: "replace value".into(),
                    replacement: Some(wire::CallableEvidenceDiagnosticReplacementV1 {
                        range: span(),
                        text: "replacement".into(),
                    }),
                },
                wire::CallableEvidenceDiagnosticFixV1 {
                    message: "inspect annotation".into(),
                    replacement: None,
                },
            ],
            root_cause: Some(23),
        };
        let relocation = TypeRelocation {
            store_id: TypeStore::new().id(),
            types: vec![crate::types::id::TypeId(81)],
            parameters: vec![crate::types::id::TypeParameterId(37)],
            kinds: Vec::new(),
            rows: Vec::new(),
            families: Vec::new(),
            variants: Vec::new(),
            domains: Vec::new(),
            constructors: Vec::new(),
            scoped: Vec::new(),
            lambdas: Vec::new(),
        };
        (record, relocation)
    }

    #[test]
    fn budgeted_diagnostic_preserves_fields_and_relocates_guidance() {
        let (mut record, relocation) = fixture();
        for severity in [
            CallableDiagnosticSeverityV1::Error,
            CallableDiagnosticSeverityV1::Warning,
            CallableDiagnosticSeverityV1::Information,
            CallableDiagnosticSeverityV1::Hint,
        ] {
            record.severity = severity;
            let mut expected_allocations = AllocationBudget::new(1 << 20);
            let expected = import_trait_surface_diagnostic(&record, &relocation, &mut expected_allocations).unwrap();
            let mut allocations = AllocationBudget::new(1 << 20);
            let actual = import_diagnostic_with_allocation_budget(&record, &relocation, &mut allocations).unwrap();
            assert_eq!(actual, expected);
            assert!(allocations.requested() > 0);
            assert!(matches!(
                actual.guidance[0],
                DiagnosticGuidance::ChangeAnnotation {
                    ty: crate::types::id::TypeId(81),
                    ..
                }
            ));
            assert!(matches!(
                actual.guidance[5],
                DiagnosticGuidance::ResolveGenericParameter {
                    parameter: crate::types::id::TypeParameterId(37)
                }
            ));
        }
    }

    #[test]
    fn budgeted_diagnostic_rejects_zero_budget_with_typed_allocation_error() {
        let (record, relocation) = fixture();
        let mut allocations = AllocationBudget::new(0);
        let error = import_diagnostic_with_allocation_budget(&record, &relocation, &mut allocations).unwrap_err();
        assert!(matches!(error, ArtifactError::Allocation(AllocationError::Exhausted)));
        assert_eq!(allocations.requested(), 0);
    }
}
