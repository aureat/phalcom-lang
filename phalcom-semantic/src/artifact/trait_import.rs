//! Certified PERF003 P7 trait header/surface reconstruction adapted to ARTF001.
//!
//! Current-main trait tables are BTreeMap-backed and have no budgeted frozen
//! constructor. P2 therefore preserves the complete trait contract as sorted
//! immutable staging rows, including diagnostics. T4 alone materializes the
//! live TraitHeaderTable/TraitSurfaceTable at the atomic publication boundary.

use super::diagnostic_import::{reconstruct_semantic_diagnostic, source_span};
use super::identity::{StableOwnerCatalog, canonical_selector};
use super::type_import::ReconstructedTypeGraph;
use crate::diagnostic::{SemanticDiagnostic, SemanticSourceSpan};
use crate::identity::{CallableOwnerId, DeclarationId, DispatchSide};
use crate::traits::{AssociatedTypeRequirementId, TraitAssociatedTypeRequirement, TraitRequirementId, TraitSurfaceMember};
use crate::types::parameter::GenericSignature;
use phalcom_common::modifier::Visibility;
use phalcom_compiler_artifact::semantic::aggregates::VisibilityV1;
use phalcom_compiler_artifact::semantic::world::{
    CallableAssociatedTypeRequirementIdentityV1, CallableRequirementIdentityV1, TraitAssociatedTypeRequirementV1, TraitSurfaceMemberV1, TraitSurfaceRecordV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;

/// P7 retained a source-member ordinal used by witness construction even
/// though current-main's public TraitSurfaceMember no longer stores it.
#[derive(Clone, Debug)]
pub struct ImportedTraitMember {
    pub member: TraitSurfaceMember,
    pub source_member_index: usize,
}

#[derive(Clone, Debug)]
pub struct ImportedTraitSurface {
    pub declaration: DeclarationId,
    pub generic_signature: Option<GenericSignature>,
    pub header_source: SemanticSourceSpan,
    pub members: Box<[ImportedTraitMember]>,
    pub associated_types: Box<[TraitAssociatedTypeRequirement]>,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

/// Exact trait contract authority staged for the imported generation.
#[derive(Clone, Debug, Default)]
pub struct ImportedTraitAuthority {
    surfaces: Box<[ImportedTraitSurface]>,
}

impl ImportedTraitAuthority {
    pub fn surface_len(&self) -> usize {
        self.surfaces.len()
    }

    pub fn member_len(&self) -> usize {
        self.surfaces.iter().map(|surface| surface.members.len()).sum()
    }

    pub fn associated_type_len(&self) -> usize {
        self.surfaces.iter().map(|surface| surface.associated_types.len()).sum()
    }

    pub fn diagnostic_len(&self) -> usize {
        self.surfaces.iter().map(|surface| surface.diagnostics.len()).sum()
    }

    pub(crate) fn surface(&self, declaration: &DeclarationId) -> Option<&ImportedTraitSurface> {
        self.surfaces
            .binary_search_by(|surface| surface.declaration.cmp(declaration))
            .ok()
            .map(|index| &self.surfaces[index])
    }
}

pub(crate) fn reconstruct_trait_authority(
    records: &[TraitSurfaceRecordV1],
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedTraitAuthority, ArtifactError> {
    let mut surfaces = allocations.vector::<ImportedTraitSurface>(records.len()).map_err(ArtifactError::Allocation)?;

    for record in records {
        surfaces.push(reconstruct_surface(record, types, owners, allocations)?);
    }

    surfaces.sort_by(|left, right| left.declaration.cmp(&right.declaration));
    if surfaces.windows(2).any(|pair| pair[0].declaration == pair[1].declaration) {
        return Err(invalid("trait surface is duplicated after stable-owner relocation"));
    }

    Ok(ImportedTraitAuthority {
        surfaces: allocations.boxed_slice(surfaces).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_surface(
    record: &TraitSurfaceRecordV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedTraitSurface, ArtifactError> {
    let declaration = owners.declaration(&record.declaration)?;
    let header_source = source_span(&record.header_source, owners)?;
    if header_source.module != declaration.module {
        return Err(invalid("trait header source belongs to another module"));
    }

    let generic_signature = record
        .generic_signature
        .as_ref()
        .map(|signature| super::semantic_import::reconstruct_signature(signature, types, owners, allocations))
        .transpose()?;

    let mut members = allocations
        .vector::<ImportedTraitMember>(record.members.len())
        .map_err(ArtifactError::Allocation)?;
    for row in &record.members {
        members.push(reconstruct_member(row, &declaration, types, owners, allocations)?);
    }
    members.sort_by(|left, right| left.member.requirement.cmp(&right.member.requirement));
    if members.windows(2).any(|pair| pair[0].member.requirement == pair[1].member.requirement) {
        return Err(invalid("trait requirement identity is duplicated"));
    }

    let mut associated_types = allocations
        .vector::<TraitAssociatedTypeRequirement>(record.associated_types.len())
        .map_err(ArtifactError::Allocation)?;
    for row in &record.associated_types {
        associated_types.push(reconstruct_associated_type(row, &declaration, types, owners, allocations)?);
    }
    associated_types.sort_by(|left, right| left.requirement.cmp(&right.requirement));
    if associated_types.windows(2).any(|pair| pair[0].requirement == pair[1].requirement) {
        return Err(invalid("trait associated-type requirement identity is duplicated"));
    }

    let mut diagnostics = allocations
        .vector::<SemanticDiagnostic>(record.diagnostics.len())
        .map_err(ArtifactError::Allocation)?;
    for diagnostic in &record.diagnostics {
        diagnostics.push(reconstruct_semantic_diagnostic(diagnostic, types, owners, allocations)?);
    }

    Ok(ImportedTraitSurface {
        declaration,
        generic_signature,
        header_source,
        members: allocations.boxed_slice(members).map_err(ArtifactError::Allocation)?,
        associated_types: allocations.boxed_slice(associated_types).map_err(ArtifactError::Allocation)?,
        diagnostics: allocations.boxed_slice(diagnostics).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_member(
    row: &TraitSurfaceMemberV1,
    declaration: &DeclarationId,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedTraitMember, ArtifactError> {
    let requirement = requirement_identity(&row.requirement, owners)?;
    if &requirement.owner != declaration {
        return Err(invalid("trait requirement belongs to another trait"));
    }

    let callable = owners.callable(&row.callable)?;
    if !matches!(&callable.owner, CallableOwnerId::Declaration(owner) if owner == declaration)
        || callable.side != requirement.side
        || callable.selector != requirement.selector
    {
        return Err(invalid("trait requirement callable disagrees with its requirement identity"));
    }

    let signature = super::callable_import::reconstruct_callable_signature(&row.signature, types, owners, allocations)?;
    if signature.callable != callable || signature.owner != *declaration || signature.side != requirement.side || signature.selector != requirement.selector {
        return Err(invalid("trait requirement signature disagrees with its exact callable identity"));
    }

    let source = source_span(&row.source, owners)?;
    if source.module != declaration.module {
        return Err(invalid("trait requirement source belongs to another module"));
    }
    let default_source = row.default_source.as_ref().map(|source| source_span(source, owners)).transpose()?;
    if default_source.as_ref().is_some_and(|source| source.module != declaration.module) {
        return Err(invalid("trait default source belongs to another module"));
    }
    if !row.default_present && default_source.is_some() {
        return Err(invalid("trait requirement has default source without a default implementation"));
    }

    let source_member_index = usize::try_from(row.source_member_index).map_err(|_| invalid("trait source member index exceeds address space"))?;
    Ok(ImportedTraitMember {
        member: TraitSurfaceMember {
            requirement,
            callable,
            signature,
            visibility: visibility(&row.visibility),
            source,
            source_member_index,
            default_present: row.default_present,
            default_source,
        },
        source_member_index,
    })
}

fn reconstruct_associated_type(
    row: &TraitAssociatedTypeRequirementV1,
    declaration: &DeclarationId,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<TraitAssociatedTypeRequirement, ArtifactError> {
    let requirement = associated_type_identity(&row.requirement, owners)?;
    if &requirement.owner != declaration {
        return Err(invalid("trait associated-type requirement belongs to another trait"));
    }
    let source = source_span(&row.source, owners)?;
    if source.module != declaration.module {
        return Err(invalid("trait associated-type source belongs to another module"));
    }
    Ok(TraitAssociatedTypeRequirement {
        requirement,
        name: allocations.boxed_str(&row.name).map_err(ArtifactError::Allocation)?,
        domain: super::associated_import::type_member_domain(&row.domain, types)?,
        source,
    })
}

fn requirement_identity(identity: &CallableRequirementIdentityV1, owners: &StableOwnerCatalog<'_>) -> Result<TraitRequirementId, ArtifactError> {
    let owner = owners.declaration(&identity.owner)?;
    let selector = canonical_selector(&identity.selector)?;
    let side = match identity.side {
        0 => DispatchSide::Instance,
        1 => DispatchSide::Class,
        _ => return Err(invalid("trait requirement has an invalid dispatch side")),
    };
    Ok(TraitRequirementId { owner, selector, side })
}

fn associated_type_identity(
    identity: &CallableAssociatedTypeRequirementIdentityV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<AssociatedTypeRequirementId, ArtifactError> {
    Ok(AssociatedTypeRequirementId::new(owners.declaration(&identity.owner)?, identity.index))
}

fn visibility(value: &VisibilityV1) -> Visibility {
    match value {
        VisibilityV1::Public => Visibility::Public,
        VisibilityV1::Private => Visibility::Private,
        VisibilityV1::Protected => Visibility::Protected,
        VisibilityV1::Module => Visibility::Module,
        VisibilityV1::Package => Visibility::Package,
        VisibilityV1::Internal => Visibility::Internal,
    }
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
