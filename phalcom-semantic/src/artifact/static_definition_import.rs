//! Certified PERF003 P7 static-definition authority adapted to ARTF001.
//!
//! Current-main's live StaticDefinition is AST-backed and no longer has P7's
//! Operational body variant. The source-free canonical program is therefore
//! staged separately and exactly; T4 may project it into the single imported
//! generation without fabricating source syntax.

use super::identity::StableOwnerCatalog;
use super::static_import::ImportedStaticValueGraph;
use super::static_program_import::{
    ImportedCanonicalStaticProgram, reconstruct_canonical_static_program,
    reconstruct_static_definition_signature,
};
use super::type_import::ReconstructedTypeGraph;
use crate::diagnostic::SemanticDiagnostic;
use crate::identity::DeclarationId;
use crate::static_definition::StaticDefinitionSignature;
use crate::types::parameter::TypeParameterOwner;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::aggregates::StructuralPrimaryContractsV2;
use phalcom_compiler_artifact::semantic::world::StaticDefinitionRecordV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use std::sync::Arc;

/// Exact P7 static-definition semantic authority, independent of source AST.
#[derive(Clone, Debug)]
pub struct ImportedStaticDefinition {
    pub id: DeclarationId,
    pub signature: StaticDefinitionSignature,
    pub source_range: SourceRange,
    /// Present only for the Type-valued alias projection. The structural wire
    /// row is cross-validated during import.
    pub alias_owner: Option<DeclarationId>,
    pub canonical_program: Arc<ImportedCanonicalStaticProgram>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
}

/// Sorted immutable static-definition authority staged for T4.
#[derive(Clone, Debug, Default)]
pub struct ImportedStaticDefinitionAuthority {
    definitions: Box<[ImportedStaticDefinition]>,
}

impl ImportedStaticDefinitionAuthority {
    pub fn len(&self) -> usize {
        self.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.definitions.is_empty()
    }

    pub fn get(&self, id: &DeclarationId) -> Option<&ImportedStaticDefinition> {
        self.definitions
            .binary_search_by(|candidate| candidate.id.cmp(id))
            .ok()
            .map(|index| &self.definitions[index])
    }
}

/// Port of certified P7 `import_static_definition_owners` with the obsolete
/// live Operational body wrapper replaced by explicit staging authority.
pub(crate) fn reconstruct_static_definitions(
    records: &[StaticDefinitionRecordV1],
    contracts: &StructuralPrimaryContractsV2,
    static_values: &ImportedStaticValueGraph,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedStaticDefinitionAuthority, ArtifactError> {
    let mut definitions = allocations
        .vector::<ImportedStaticDefinition>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        let id = owners.declaration(&record.id)?;
        let signature =
            reconstruct_static_definition_signature(&record.signature, types, owners, allocations)?;
        if signature
            .generic_signature
            .as_ref()
            .is_some_and(|signature| signature.owner != TypeParameterOwner::Declaration(id.clone()))
        {
            return Err(invalid(
                "static definition generic signature belongs to a different declaration",
            ));
        }

        let canonical_program = reconstruct_canonical_static_program(
            &record.canonical_program,
            static_values,
            types,
            owners,
            limits,
            allocations,
        )?;
        if canonical_program.owner.0 != id {
            return Err(invalid(
                "canonical static program owner disagrees with its definition record",
            ));
        }

        let alias_owner = record
            .alias_info
            .as_ref()
            .map(|alias| -> Result<DeclarationId, ArtifactError> {
                if alias != &record.id {
                    return Err(invalid(
                        "static definition alias link belongs to another declaration",
                    ));
                }
                let structural_alias = contracts
                    .type_aliases
                    .iter()
                    .find(|row| row.declaration == *alias)
                    .ok_or_else(|| {
                        invalid(
                            "static definition alias link has no structural type-alias row",
                        )
                    })?;
                if !contracts
                    .declaration_shells
                    .iter()
                    .any(|shell| shell.declaration == *alias)
                {
                    return Err(invalid(
                        "static definition alias link has no structural declaration shell",
                    ));
                }
                owners.declaration(alias)
            })
            .transpose()?;

        let mut diagnostics = allocations
            .vector::<SemanticDiagnostic>(record.diagnostics.len())
            .map_err(ArtifactError::Allocation)?;
        for diagnostic in &record.diagnostics {
            diagnostics.push(super::diagnostic_import::reconstruct_semantic_diagnostic(
                diagnostic,
                types,
                owners,
                allocations,
            )?);
        }

        definitions.push(ImportedStaticDefinition {
            id,
            signature,
            source_range: super::diagnostic_import::range(
                &record.source,
                "static definition source",
            )?,
            alias_owner,
            canonical_program: allocations
                .arc(canonical_program)
                .map_err(ArtifactError::Allocation)?,
            diagnostics: allocations
                .arc_slice(diagnostics)
                .map_err(ArtifactError::Allocation)?,
        });
    }

    definitions.sort_by(|left, right| left.id.cmp(&right.id));
    if definitions.windows(2).any(|pair| pair[0].id == pair[1].id) {
        return Err(invalid(
            "static definition owner is duplicated after stable-owner relocation",
        ));
    }

    Ok(ImportedStaticDefinitionAuthority {
        definitions: allocations
            .boxed_slice(definitions)
            .map_err(ArtifactError::Allocation)?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
