//! Live-identity preflight for certified PERF003 P7 witness plans.
//!
//! The compiler-artifact layer already validates the portable witness graph.
//! This module performs the next trust transition: every witness owner, trait
//! contract, requirement view, selected callable, and diagnostic is relocated
//! into current-main authority and cross-checked against the semantic products
//! already staged by ARTF001. Recursive requirement selections and conformance
//! evidence are intentionally left for the next transplant slice; no partial
//! ConformanceWitnessPlan is published here.

use super::callable_import::ImportedCallableAuthority;
use super::conformance_import::ImportedConformanceAuthority;
use super::diagnostic_import::reconstruct_semantic_diagnostic;
use super::identity::{StableOwnerCatalog, canonical_selector};
use super::trait_import::ImportedTraitAuthority;
use super::type_import::ReconstructedTypeGraph;
use crate::diagnostic::SemanticDiagnostic;
use crate::identity::{CallableId, CallableOwnerId, DispatchSide, ImplId};
use crate::surface::MemberVisibility;
use crate::traits::{InstantiatedTraitRequirement, TraitRef, TraitRequirementId};
use crate::types::id::TypeId;
use crate::types::parameter::GenericSignature;
use phalcom_compiler_artifact::semantic::aggregates::VisibilityV1;
use phalcom_compiler_artifact::semantic::world::{
    CallableRequirementIdentityV1, CallableRequirementViewV1, WitnessPlanV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::StableCallableRef;

/// The non-recursive, fully relocated portion of one P7 witness plan.
///
/// `wire_fingerprint` is retained only as transport evidence. It is not a
/// current-main ProductFingerprint; the completed witness importer must rebuild
/// a native ConformanceWitnessPlan and compare its freshly computed fingerprint
/// before publication.
#[derive(Clone, Debug)]
pub struct ImportedWitnessPlanPreflight {
    pub impl_id: ImplId,
    pub source_module: crate::identity::ModuleId,
    pub target_template: TypeId,
    pub trait_ref_template: TraitRef,
    pub generic_signature: Option<GenericSignature>,
    pub requirement_views: Box<[(TraitRequirementId, InstantiatedTraitRequirement)]>,
    pub selected_callables: Box<[CallableId]>,
    pub invalid_explicit_members: bool,
    pub diagnostics: Box<[SemanticDiagnostic]>,
    pub requirement_selection_count: usize,
    pub conformance_evidence_count: usize,
    wire_fingerprint: u64,
}

/// Witness plans that have crossed the portable→live identity boundary but do
/// not yet claim recursive proof reconstruction.
#[derive(Clone, Debug, Default)]
pub struct ImportedWitnessPreflightAuthority {
    plans: Box<[ImportedWitnessPlanPreflight]>,
}

impl ImportedWitnessPreflightAuthority {
    pub fn len(&self) -> usize {
        self.plans.len()
    }

    pub fn is_empty(&self) -> bool {
        self.plans.is_empty()
    }

    pub fn requirement_view_len(&self) -> usize {
        self.plans.iter().map(|plan| plan.requirement_views.len()).sum()
    }

    pub fn diagnostic_len(&self) -> usize {
        self.plans.iter().map(|plan| plan.diagnostics.len()).sum()
    }

    pub fn evidence_node_len(&self) -> usize {
        self.plans.iter().map(|plan| plan.conformance_evidence_count).sum()
    }

    pub(crate) fn get(&self, implementation: &ImplId) -> Option<&ImportedWitnessPlanPreflight> {
        self.plans
            .binary_search_by(|plan| plan.impl_id.cmp(implementation))
            .ok()
            .map(|index| &self.plans[index])
    }
}

pub(crate) fn reconstruct_witness_preflight(
    records: &[WitnessPlanV1],
    conformances: &ImportedConformanceAuthority,
    traits: &ImportedTraitAuthority,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedWitnessPreflightAuthority, ArtifactError> {
    let mut plans = allocations
        .vector::<ImportedWitnessPlanPreflight>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        plans.push(reconstruct_plan(
            record,
            conformances,
            traits,
            callables,
            types,
            owners,
            allocations,
        )?);
    }

    plans.sort_by(|left, right| left.impl_id.cmp(&right.impl_id));
    if plans.windows(2).any(|pair| pair[0].impl_id == pair[1].impl_id) {
        return Err(invalid("witness plan is duplicated after stable-owner relocation"));
    }

    Ok(ImportedWitnessPreflightAuthority {
        plans: allocations.boxed_slice(plans).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_plan(
    record: &WitnessPlanV1,
    conformances: &ImportedConformanceAuthority,
    traits: &ImportedTraitAuthority,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedWitnessPlanPreflight, ArtifactError> {
    if compute_witness_layout_fingerprint(&record.ordered_selected_callables)
        != record.slot_layout_fingerprint
    {
        return Err(invalid("witness plan slot layout fingerprint mismatch"));
    }

    let impl_id = owners.implementation(&record.impl_id)?;
    let source_module = owners.module(&record.source_module)?;
    let contribution = conformances
        .get(&impl_id)
        .ok_or_else(|| invalid("witness plan has no imported conformance head"))?;
    if contribution.source_module != source_module {
        return Err(invalid("witness plan source module disagrees with its conformance head"));
    }

    let trait_declaration = owners.declaration(&record.trait_ref_template.declaration)?;
    let trait_surface = traits
        .surface(&trait_declaration)
        .ok_or_else(|| invalid("witness plan references an absent imported trait surface"))?;

    let mut trait_arguments = allocations
        .vector::<TypeId>(record.trait_ref_template.arguments.len())
        .map_err(ArtifactError::Allocation)?;
    for argument in &record.trait_ref_template.arguments {
        trait_arguments.push(
            *types
                .types
                .get(argument.0 as usize)
                .ok_or_else(|| invalid("witness trait reference contains an absent type argument"))?,
        );
    }
    let trait_ref_template = TraitRef::new(
        trait_declaration.clone(),
        allocations.boxed_slice(trait_arguments).map_err(ArtifactError::Allocation)?,
    );
    let target_template = *types
        .types
        .get(record.target_template.0 as usize)
        .ok_or_else(|| invalid("witness target template references an absent type"))?;
    let generic_signature = record
        .generic_signature
        .as_ref()
        .map(|signature| super::semantic_import::reconstruct_signature(signature, types, owners, allocations))
        .transpose()?;

    // This is a stronger current-main check than portable row validation:
    // the witness must describe the exact live conformance head it will later
    // complete, after project-qualified identity relocation.
    if contribution.trait_ref != trait_ref_template
        || contribution.target_head != target_template
        || contribution.generic_signature != generic_signature
    {
        return Err(invalid("witness plan header disagrees with its imported conformance head"));
    }

    let mut views = allocations
        .vector::<(TraitRequirementId, InstantiatedTraitRequirement)>(record.requirement_views.len())
        .map_err(ArtifactError::Allocation)?;
    for view in &record.requirement_views {
        views.push(reconstruct_requirement_view(
            view,
            &trait_declaration,
            trait_surface,
            types,
            owners,
            allocations,
        )?);
    }
    views.sort_by(|left, right| left.0.cmp(&right.0));
    if views.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(invalid("witness requirement view is duplicated after relocation"));
    }

    let mut selected_callables = allocations
        .vector::<CallableId>(record.ordered_selected_callables.len())
        .map_err(ArtifactError::Allocation)?;
    for callable in &record.ordered_selected_callables {
        let callable = owners.callable(callable)?;
        if callables.signature(&callable).is_none() {
            return Err(invalid("witness selected callable has no imported canonical signature"));
        }
        selected_callables.push(callable);
    }

    let mut diagnostics = allocations
        .vector::<SemanticDiagnostic>(record.diagnostics.len())
        .map_err(ArtifactError::Allocation)?;
    for diagnostic in &record.diagnostics {
        diagnostics.push(reconstruct_semantic_diagnostic(
            diagnostic,
            types,
            owners,
            allocations,
        )?);
    }

    Ok(ImportedWitnessPlanPreflight {
        impl_id,
        source_module,
        target_template,
        trait_ref_template,
        generic_signature,
        requirement_views: allocations.boxed_slice(views).map_err(ArtifactError::Allocation)?,
        selected_callables: allocations
            .boxed_slice(selected_callables)
            .map_err(ArtifactError::Allocation)?,
        invalid_explicit_members: record.invalid_explicit_members,
        diagnostics: allocations.boxed_slice(diagnostics).map_err(ArtifactError::Allocation)?,
        requirement_selection_count: record.requirements.len(),
        conformance_evidence_count: record.conformance_evidence.len(),
        wire_fingerprint: record.fingerprint,
    })
}

fn reconstruct_requirement_view(
    view: &CallableRequirementViewV1,
    trait_declaration: &crate::identity::DeclarationId,
    trait_surface: &super::trait_import::ImportedTraitSurface,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<(TraitRequirementId, InstantiatedTraitRequirement), ArtifactError> {
    let requirement = requirement_identity(&view.requirement, owners)?;
    if &requirement.owner != trait_declaration {
        return Err(invalid("witness requirement view belongs to another trait"));
    }

    let source_member = trait_surface
        .members
        .binary_search_by(|member| member.member.requirement.cmp(&requirement))
        .ok()
        .and_then(|index| trait_surface.members.get(index))
        .ok_or_else(|| invalid("witness requirement view is absent from its imported trait surface"))?;

    let callable = owners.callable(&view.callable)?;
    if !matches!(&callable.owner, CallableOwnerId::Declaration(owner) if owner == trait_declaration)
        || callable.side != requirement.side
        || callable.selector != requirement.selector
        || source_member.member.callable != callable
    {
        return Err(invalid("witness requirement view callable disagrees with its trait contract"));
    }

    let signature = super::callable_import::reconstruct_callable_signature(
        &view.signature,
        types,
        owners,
        allocations,
    )?;
    if signature.callable != callable
        || signature.owner != *trait_declaration
        || signature.side != requirement.side
        || signature.selector != requirement.selector
    {
        return Err(invalid("witness requirement view signature disagrees with its identity"));
    }

    let default_callable = view
        .default_callable
        .as_ref()
        .map(|callable| owners.callable(callable))
        .transpose()?;
    if source_member.member.default_present != default_callable.is_some()
        || default_callable.as_ref().is_some_and(|default| default != &callable)
    {
        return Err(invalid("witness requirement default callable disagrees with its trait surface"));
    }

    let imported = InstantiatedTraitRequirement {
        requirement: requirement.clone(),
        callable,
        signature,
        visibility: visibility(&view.visibility),
        default_callable,
    };
    Ok((requirement, imported))
}

fn requirement_identity(
    identity: &CallableRequirementIdentityV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<TraitRequirementId, ArtifactError> {
    Ok(TraitRequirementId {
        owner: owners.declaration(&identity.owner)?,
        selector: canonical_selector(&identity.selector)?,
        side: match identity.side {
            0 => DispatchSide::Instance,
            1 => DispatchSide::Class,
            _ => return Err(invalid("witness requirement identity has an invalid dispatch side")),
        },
    })
}

fn visibility(value: &VisibilityV1) -> MemberVisibility {
    match value {
        VisibilityV1::Public => MemberVisibility::Public,
        VisibilityV1::Private => MemberVisibility::Private,
        VisibilityV1::Protected => MemberVisibility::Protected,
        VisibilityV1::Module => MemberVisibility::Module,
        VisibilityV1::Package => MemberVisibility::Package,
        VisibilityV1::Internal => MemberVisibility::Internal,
    }
}

/// Exact certified P7 slot-layout algorithm. This is intentionally not replaced
/// with a new hash: producers and certified artifacts already depend on it.
fn compute_witness_layout_fingerprint(callables: &[StableCallableRef]) -> [u8; 32] {
    let mut digest = [0u8; 32];
    for (index, callable) in callables.iter().enumerate() {
        for (byte_index, byte) in callable.selector.as_bytes().iter().enumerate() {
            digest[byte_index % 32] ^= byte.wrapping_add(index as u8);
        }
    }
    digest
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
