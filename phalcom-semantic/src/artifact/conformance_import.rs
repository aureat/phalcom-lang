//! Certified PERF003 P7 conformance-head reconstruction adapted to ARTF001.
//!
//! Conformance heads are immutable semantic source authority. Witness selection,
//! proof evidence, and dispatch buckets remain separate later P7 transplants.
//! Rows are staged in stable identity order until T4 publishes the one snapshot.

use super::identity::StableOwnerCatalog;
use super::type_import::ReconstructedTypeGraph;
use crate::diagnostic::SemanticSourceSpan;
use crate::identity::ImplId;
use crate::impls::{ConformanceContribution, ConformanceTarget, conformance_is_authorized};
use crate::traits::TraitRef;
use phalcom_common::range::SourceRange;
use phalcom_compiler_artifact::semantic::world::{ConformanceHeadRecordV1, ConformanceTargetV1};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

/// Exact source conformance contributions staged for the imported snapshot.
#[derive(Clone, Debug, Default)]
pub struct ImportedConformanceAuthority {
    entries: Box<[(ImplId, ConformanceContribution)]>,
}

impl ImportedConformanceAuthority {
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub(crate) fn get(&self, implementation: &ImplId) -> Option<&ConformanceContribution> {
        self.entries
            .binary_search_by(|(candidate, _)| candidate.cmp(implementation))
            .ok()
            .map(|index| &self.entries[index].1)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&ImplId, &ConformanceContribution)> {
        self.entries.iter().map(|(implementation, contribution)| (implementation, contribution))
    }
}

/// Port of P7 conformance-head import, with current-main identity authority
/// rechecking the portable authorization claim after relocation.
pub(crate) fn reconstruct_conformance_authority(
    records: &[ConformanceHeadRecordV1],
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedConformanceAuthority, ArtifactError> {
    let mut entries = allocations
        .vector::<(ImplId, ConformanceContribution)>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        let impl_id = owners.implementation(&record.impl_id)?;
        let source_module = owners.module(&record.source_module)?;
        if impl_id.module != source_module {
            return Err(invalid("conformance implementation identity disagrees with its source module"));
        }

        let target = match &record.target {
            ConformanceTargetV1::Declaration { declaration } => ConformanceTarget::Declaration(owners.declaration(declaration)?),
            ConformanceTargetV1::ExactEnumCase { variant } => ConformanceTarget::ExactEnumCase(owners.variant(variant)?),
        };
        let target_head = *types
            .types
            .get(record.target_type.0 as usize)
            .ok_or_else(|| invalid("conformance head references an absent target type"))?;

        let trait_declaration = owners.declaration(&record.trait_ref.declaration)?;
        let mut arguments = allocations.vector(record.trait_ref.arguments.len()).map_err(ArtifactError::Allocation)?;
        for argument in &record.trait_ref.arguments {
            arguments.push(
                *types
                    .types
                    .get(argument.0 as usize)
                    .ok_or_else(|| invalid("conformance trait reference contains an absent type argument"))?,
            );
        }
        let trait_ref = TraitRef::new(trait_declaration, allocations.boxed_slice(arguments).map_err(ArtifactError::Allocation)?);

        let generic_signature = record
            .generic_signature
            .as_ref()
            .map(|signature| super::semantic_import::reconstruct_signature(signature, types, owners, allocations))
            .transpose()?;

        let current_authorized = conformance_is_authorized(&source_module, &trait_ref, &target);
        if current_authorized != record.authorized {
            return Err(invalid("portable conformance authorization disagrees with current project-qualified ownership"));
        }
        if record.eligible && !record.authorized {
            return Err(invalid("ineligible conformance authorization cannot be published as eligible"));
        }

        let contribution = ConformanceContribution {
            impl_id: impl_id.clone(),
            source_module: source_module.clone(),
            trait_ref,
            target,
            target_head,
            generic_signature,
            authorized: record.authorized,
            eligible: record.eligible,
            // P7's ConformanceHeadRecordV1 intentionally carries no source
            // range; source module identity is the durable authority here.
            source: SemanticSourceSpan::new(source_module, SourceRange::default()),
            diagnostics: Box::new([]),
        };
        entries.push((impl_id, contribution));
    }

    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(invalid("conformance head is duplicated after stable-owner relocation"));
    }

    Ok(ImportedConformanceAuthority {
        entries: allocations.boxed_slice(entries).map_err(ArtifactError::Allocation)?,
    })
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
