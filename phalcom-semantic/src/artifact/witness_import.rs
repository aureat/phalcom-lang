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
use super::diagnostic_import::{reconstruct_semantic_diagnostic, source_span};
use super::identity::{StableOwnerCatalog, canonical_selector};
use super::trait_import::ImportedTraitAuthority;
use super::type_import::ReconstructedTypeGraph;
use crate::diagnostic::SemanticDiagnostic;
use crate::generic_evidence::GenericConformanceKey;
use crate::identity::{CallableId, CallableOwnerId, DataComponentId, DispatchSide, ImplId};
use crate::impls::{
    AssociatedTypeBindingFailure, AssociatedTypeBindingFailureKind, AssociatedTypeBindingTemplate,
    AssociatedTypeBindingValue, ConformanceCompleteness, ConformanceFailure, InherentImplDomain,
    InherentImplTarget, RequirementFailure,
};
use crate::surface::MemberVisibility;
use crate::traits::{AssociatedTypeRequirementId, InstantiatedTraitRequirement, TraitRef, TraitRequirementId};
use crate::signature::CallableSemanticSignature;
use crate::types::id::{RecordRowId, TypeId, TypeParameterId};
use crate::types::outcome::DynamicBoundaryObligation;
use crate::types::parameter::GenericSignature;
use phalcom_compiler_artifact::semantic::aggregates::VisibilityV1;
use phalcom_compiler_artifact::semantic::world::{
    CallableAssociatedTypeBindingFailureKindV1, CallableAssociatedTypeBindingFailureV1,
    CallableAssociatedTypeBindingTemplateV1, CallableAssociatedTypeBindingValueV1,
    CallableAssociatedTypeRequirementIdentityV1, CallableConformanceAssociatedTypePlanV1,
    CallableConformanceCompletenessV1, CallableConformanceFailureV1,
    CallableEffectiveInherentWitnessV1, CallableGenericCallConformanceBindingV1,
    CallableGenericCallConformanceSourceV1, CallableInherentImplDomainV1,
    CallableInherentImplSpecializationV1, CallableInherentImplTargetV1,
    CallableRequirementIdentityV1, CallableRequirementSelectionEntryV1,
    CallableRequirementSelectionV1, CallableRequirementViewV1, CallableTypeEnvironmentV1,
    GenericConformanceKeyV1, WitnessPlanV1,
};
use phalcom_compiler_artifact::semantic::type_store::StaticArgumentV1;
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;
use phalcom_type_meta::identity::StableCallableRef;

/// Budgeted immutable staging form of P7 associated-type witness authority.
///
/// The live ConformanceAssociatedTypePlan is BTreeMap-backed. P2 keeps rows
/// sorted here so no unmetered map allocation occurs before T4 publication.
#[derive(Clone, Debug)]
pub struct ImportedAssociatedTypePlan {
    pub impl_id: ImplId,
    pub bindings: Box<[(AssociatedTypeRequirementId, AssociatedTypeBindingTemplate)]>,
    pub failures: Box<[AssociatedTypeBindingFailure]>,
    pub diagnostics: Box<[SemanticDiagnostic]>,
}

/// Budgeted immutable replacement for current-main's HashMap-backed
/// TypeEnvironment during artifact staging.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedTypeEnvironment {
    pub type_bindings: Box<[(TypeParameterId, TypeId)]>,
    pub dynamic_bindings: Box<[TypeParameterId]>,
    pub row_bindings: Box<[(TypeParameterId, RecordRowId)]>,
    pub self_binding: Option<TypeId>,
}

/// Evidence reference retained until the recursive conformance-evidence graph
/// is reconstructed. Ordinals have already passed wire validation and are
/// checked again against the owning witness evidence table here.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ImportedConformanceBindingSource {
    Exact { evidence: u32 },
    Forwarded { caller_conformance: GenericConformanceKey },
    Derived { evidence: u32 },
    CompilerDerived { evidence: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedGenericCallConformanceBinding {
    pub callee_conformance: GenericConformanceKey,
    pub source: ImportedConformanceBindingSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImportedInherentSpecialization {
    pub impl_id: ImplId,
    pub receiver: TypeId,
    pub owner_view: TypeId,
    pub bindings: Box<[(TypeParameterId, TypeId)]>,
    pub environment: ImportedTypeEnvironment,
    pub conformance_provenance: Option<Box<[ImportedGenericCallConformanceBinding]>>,
}

#[derive(Clone, Debug)]
pub struct ImportedEffectiveInherentWitness {
    pub callable: CallableId,
    pub signature: CallableSemanticSignature,
    pub visibility: MemberVisibility,
    pub owner: crate::identity::DeclarationId,
    pub conditional_impl: Option<ImplId>,
    pub applicability: Option<ImportedInherentSpecialization>,
    pub conditional_domain: Option<std::sync::Arc<InherentImplDomain>>,
}

#[derive(Clone, Debug)]
pub enum ImportedRequirementSelection {
    ConformanceCallable {
        callable: CallableId,
    },
    InherentCallable {
        callable: CallableId,
        conditional_impl: Option<ImplId>,
        applicability: Option<ImportedInherentSpecialization>,
    },
    ConditionalInherent {
        candidate: ImportedEffectiveInherentWitness,
        pending: ConformanceCompleteness,
        fallback: Option<Box<ImportedRequirementSelection>>,
    },
    DataComponent {
        component: DataComponentId,
        specialized_type: TypeId,
    },
    TraitDefault {
        callable: CallableId,
    },
}

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
    pub associated_type_plan: ImportedAssociatedTypePlan,
    pub completeness: ConformanceCompleteness,
    pub requirement_selections: Box<[(TraitRequirementId, ImportedRequirementSelection)]>,
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

    pub fn associated_binding_len(&self) -> usize {
        self.plans
            .iter()
            .map(|plan| plan.associated_type_plan.bindings.len())
            .sum()
    }

    pub fn associated_failure_len(&self) -> usize {
        self.plans
            .iter()
            .map(|plan| plan.associated_type_plan.failures.len())
            .sum()
    }

    pub fn evidence_node_len(&self) -> usize {
        self.plans.iter().map(|plan| plan.conformance_evidence_count).sum()
    }

    pub fn requirement_selection_len(&self) -> usize {
        self.plans.iter().map(|plan| plan.requirement_selections.len()).sum()
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
    limits: &ValidationLimits,
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
            limits.max_recursion_depth,
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
    max_depth: usize,
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

    let associated_type_plan = reconstruct_associated_type_plan(
        &record.associated_type_plan,
        &impl_id,
        &trait_declaration,
        types,
        owners,
        allocations,
    )?;
    let completeness = reconstruct_completeness(
        &record.completeness,
        &trait_declaration,
        types,
        owners,
        allocations,
    )?;

    let mut requirement_selections = allocations
        .vector::<(TraitRequirementId, ImportedRequirementSelection)>(record.requirements.len())
        .map_err(ArtifactError::Allocation)?;
    for entry in &record.requirements {
        let requirement = requirement_identity(&entry.requirement, owners)?;
        if requirement.owner != trait_declaration {
            return Err(invalid("witness requirement selection belongs to another trait"));
        }
        let selection = reconstruct_requirement_selection(
            &entry.selection,
            &trait_declaration,
            callables,
            types,
            owners,
            record.conformance_evidence.len(),
            max_depth,
            0,
            allocations,
        )?;
        requirement_selections.push((requirement, selection));
    }
    requirement_selections.sort_by(|left, right| left.0.cmp(&right.0));
    if requirement_selections.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(invalid("witness requirement selection is duplicated after relocation"));
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
        associated_type_plan,
        completeness,
        requirement_selections: allocations
            .boxed_slice(requirement_selections)
            .map_err(ArtifactError::Allocation)?,
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

fn reconstruct_requirement_selection(
    selection: &CallableRequirementSelectionV1,
    expected_trait: &crate::identity::DeclarationId,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    evidence_count: usize,
    max_depth: usize,
    depth: usize,
    allocations: &mut AllocationBudget,
) -> Result<ImportedRequirementSelection, ArtifactError> {
    if depth > max_depth {
        return Err(invalid("witness requirement-selection nesting limit exceeded"));
    }
    Ok(match selection {
        CallableRequirementSelectionV1::ConformanceCallable { callable } => {
            let callable = owners.callable(callable)?;
            require_canonical_callable(&callable, callables)?;
            ImportedRequirementSelection::ConformanceCallable { callable }
        }
        CallableRequirementSelectionV1::InherentCallable {
            callable,
            conditional_impl,
            applicability,
        } => {
            let callable = owners.callable(callable)?;
            require_canonical_callable(&callable, callables)?;
            let conditional_impl = conditional_impl
                .as_ref()
                .map(|implementation| owners.implementation(implementation))
                .transpose()?;
            let applicability = applicability
                .as_ref()
                .map(|specialization| {
                    reconstruct_inherent_specialization(
                        specialization,
                        types,
                        owners,
                        evidence_count,
                        allocations,
                    )
                })
                .transpose()?;
            if let (Some(expected), Some(actual)) = (&conditional_impl, &applicability) {
                if expected != &actual.impl_id {
                    return Err(invalid("inherent callable applicability belongs to another conditional impl"));
                }
            }
            ImportedRequirementSelection::InherentCallable {
                callable,
                conditional_impl,
                applicability,
            }
        }
        CallableRequirementSelectionV1::ConditionalInherent {
            candidate,
            pending,
            fallback,
        } => {
            let candidate = reconstruct_effective_inherent_witness(
                candidate,
                callables,
                types,
                owners,
                evidence_count,
                allocations,
            )?;
            let pending = reconstruct_completeness(
                pending,
                expected_trait,
                types,
                owners,
                allocations,
            )?;
            let fallback = fallback
                .as_deref()
                .map(|selection| {
                    let selection = reconstruct_requirement_selection(
                        selection,
                        expected_trait,
                        callables,
                        types,
                        owners,
                        evidence_count,
                        max_depth,
                        depth + 1,
                        allocations,
                    )?;
                    allocations.boxed(selection).map_err(ArtifactError::Allocation)
                })
                .transpose()?;
            ImportedRequirementSelection::ConditionalInherent {
                candidate,
                pending,
                fallback,
            }
        }
        CallableRequirementSelectionV1::DataComponent {
            owner,
            index,
            specialized_type,
        } => ImportedRequirementSelection::DataComponent {
            component: DataComponentId::new(owners.declaration(owner)?, *index),
            specialized_type: mapped_type(*specialized_type, types, "data-component specialized type")?,
        },
        CallableRequirementSelectionV1::TraitDefault { callable } => {
            let callable = owners.callable(callable)?;
            require_canonical_callable(&callable, callables)?;
            ImportedRequirementSelection::TraitDefault { callable }
        }
    })
}

fn reconstruct_effective_inherent_witness(
    witness: &CallableEffectiveInherentWitnessV1,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    evidence_count: usize,
    allocations: &mut AllocationBudget,
) -> Result<ImportedEffectiveInherentWitness, ArtifactError> {
    let callable = owners.callable(&witness.callable)?;
    let signature = super::callable_import::reconstruct_callable_signature(
        &witness.signature,
        types,
        owners,
        allocations,
    )?;
    if signature.callable != callable {
        return Err(invalid("effective inherent witness signature disagrees with its callable"));
    }
    if let Some(canonical) = callables.signature(&callable)
        && canonical != &signature
    {
        return Err(invalid("effective inherent witness signature differs from canonical callable authority"));
    }

    let owner = owners.declaration(&witness.owner)?;
    if callable.owner.try_declaration().is_some_and(|callable_owner| callable_owner != &owner) {
        return Err(invalid("effective inherent witness callable belongs to another declaration"));
    }
    let conditional_impl = witness
        .conditional_impl
        .as_ref()
        .map(|implementation| owners.implementation(implementation))
        .transpose()?;
    let applicability = witness
        .applicability
        .as_ref()
        .map(|specialization| {
            reconstruct_inherent_specialization(
                specialization,
                types,
                owners,
                evidence_count,
                allocations,
            )
        })
        .transpose()?;
    if let (Some(expected), Some(actual)) = (&conditional_impl, &applicability) {
        if expected != &actual.impl_id {
            return Err(invalid("effective inherent witness applicability belongs to another conditional impl"));
        }
    }

    let conditional_domain = witness
        .conditional_domain
        .as_ref()
        .map(|domain| reconstruct_inherent_domain(domain, types, owners, allocations))
        .transpose()?;
    if let (Some(expected), Some(domain)) = (&conditional_impl, &conditional_domain) {
        if expected != &domain.impl_id {
            return Err(invalid("effective inherent witness domain belongs to another conditional impl"));
        }
    }

    Ok(ImportedEffectiveInherentWitness {
        callable,
        signature,
        visibility: visibility(&witness.visibility),
        owner,
        conditional_impl,
        applicability,
        conditional_domain,
    })
}

fn reconstruct_inherent_domain(
    domain: &CallableInherentImplDomainV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<std::sync::Arc<InherentImplDomain>, ArtifactError> {
    let impl_id = owners.implementation(&domain.impl_id)?;
    let target = match &domain.target {
        CallableInherentImplTargetV1::Declaration { declaration } => {
            InherentImplTarget::Declaration(owners.declaration(declaration)?)
        }
        CallableInherentImplTargetV1::ExactEnumCase { variant } => {
            InherentImplTarget::ExactEnumCase(owners.variant(variant)?)
        }
    };
    let generic_signature = domain
        .generic_signature
        .as_ref()
        .map(|signature| super::semantic_import::reconstruct_signature(signature, types, owners, allocations))
        .transpose()?;
    let mut constraints = allocations
        .vector(domain.constraints.len())
        .map_err(ArtifactError::Allocation)?;
    for constraint in &domain.constraints {
        constraints.push(super::semantic_import::reconstruct_constraint(
            constraint,
            types,
            owners,
            allocations,
        )?);
    }
    allocations
        .arc(InherentImplDomain {
            impl_id,
            target,
            head_type: mapped_type(domain.head_type, types, "conditional inherent head type")?,
            generic_signature,
            constraints: allocations.boxed_slice(constraints).map_err(ArtifactError::Allocation)?,
        })
        .map_err(ArtifactError::Allocation)
}

fn reconstruct_inherent_specialization(
    specialization: &CallableInherentImplSpecializationV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    evidence_count: usize,
    allocations: &mut AllocationBudget,
) -> Result<ImportedInherentSpecialization, ArtifactError> {
    let impl_id = owners.implementation(&specialization.impl_id)?;
    let mut bindings = allocations
        .vector::<(TypeParameterId, TypeId)>(specialization.bindings.len())
        .map_err(ArtifactError::Allocation)?;
    for binding in &specialization.bindings {
        bindings.push((
            mapped_parameter(binding.parameter, types, "inherent specialization parameter")?,
            mapped_type(binding.ty, types, "inherent specialization type")?,
        ));
    }
    bindings.sort_by_key(|(parameter, _)| *parameter);
    reject_duplicate_parameter_pairs(&bindings, "inherent specialization binds one parameter more than once")?;

    let environment = reconstruct_type_environment(
        &specialization.environment,
        types,
        allocations,
    )?;
    let conformance_provenance = specialization
        .conformance_provenance
        .as_ref()
        .map(|bindings| {
            let mut imported = allocations
                .vector::<ImportedGenericCallConformanceBinding>(bindings.len())
                .map_err(ArtifactError::Allocation)?;
            for binding in bindings {
                imported.push(reconstruct_conformance_binding(
                    binding,
                    types,
                    owners,
                    evidence_count,
                    allocations,
                )?);
            }
            allocations.boxed_slice(imported).map_err(ArtifactError::Allocation)
        })
        .transpose()?;

    Ok(ImportedInherentSpecialization {
        impl_id,
        receiver: mapped_type(specialization.receiver, types, "inherent specialization receiver")?,
        owner_view: mapped_type(specialization.owner_view, types, "inherent specialization owner view")?,
        bindings: allocations.boxed_slice(bindings).map_err(ArtifactError::Allocation)?,
        environment,
        conformance_provenance,
    })
}

fn reconstruct_type_environment(
    environment: &CallableTypeEnvironmentV1,
    types: &ReconstructedTypeGraph,
    allocations: &mut AllocationBudget,
) -> Result<ImportedTypeEnvironment, ArtifactError> {
    let mut seen = allocations
        .vector::<TypeParameterId>(environment.bindings.len())
        .map_err(ArtifactError::Allocation)?;
    let mut type_bindings = allocations
        .vector::<(TypeParameterId, TypeId)>(environment.bindings.len())
        .map_err(ArtifactError::Allocation)?;
    let mut dynamic_bindings = allocations
        .vector::<TypeParameterId>(environment.bindings.len())
        .map_err(ArtifactError::Allocation)?;
    let mut row_bindings = allocations
        .vector::<(TypeParameterId, RecordRowId)>(environment.bindings.len())
        .map_err(ArtifactError::Allocation)?;

    for binding in &environment.bindings {
        let parameter = mapped_parameter(binding.parameter, types, "type environment parameter")?;
        seen.push(parameter);
        match &binding.argument {
            StaticArgumentV1::Term { term } => {
                type_bindings.push((parameter, mapped_type(*term, types, "type environment type")?));
            }
            StaticArgumentV1::DynamicType => dynamic_bindings.push(parameter),
            StaticArgumentV1::RecordRow { row } => {
                row_bindings.push((parameter, mapped_row(*row, types, "type environment row")?));
            }
        }
    }
    seen.sort_unstable();
    if seen.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(invalid("type environment binds one parameter more than once"));
    }
    type_bindings.sort_by_key(|(parameter, _)| *parameter);
    dynamic_bindings.sort_unstable();
    row_bindings.sort_by_key(|(parameter, _)| *parameter);

    Ok(ImportedTypeEnvironment {
        type_bindings: allocations.boxed_slice(type_bindings).map_err(ArtifactError::Allocation)?,
        dynamic_bindings: allocations.boxed_slice(dynamic_bindings).map_err(ArtifactError::Allocation)?,
        row_bindings: allocations.boxed_slice(row_bindings).map_err(ArtifactError::Allocation)?,
        self_binding: environment
            .self_binding
            .map(|ty| mapped_type(ty, types, "type environment Self binding"))
            .transpose()?,
    })
}

fn reconstruct_conformance_binding(
    binding: &CallableGenericCallConformanceBindingV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    evidence_count: usize,
    allocations: &mut AllocationBudget,
) -> Result<ImportedGenericCallConformanceBinding, ArtifactError> {
    let callee_conformance = reconstruct_generic_conformance_key(
        &binding.callee_conformance,
        types,
        owners,
        allocations,
    )?;
    let source = match &binding.source {
        CallableGenericCallConformanceSourceV1::Exact { evidence } => {
            check_evidence_index(*evidence, evidence_count)?;
            ImportedConformanceBindingSource::Exact { evidence: *evidence }
        }
        CallableGenericCallConformanceSourceV1::Forwarded { caller_conformance } => {
            ImportedConformanceBindingSource::Forwarded {
                caller_conformance: reconstruct_generic_conformance_key(
                    caller_conformance,
                    types,
                    owners,
                    allocations,
                )?,
            }
        }
        CallableGenericCallConformanceSourceV1::Derived { evidence } => {
            check_evidence_index(*evidence, evidence_count)?;
            ImportedConformanceBindingSource::Derived { evidence: *evidence }
        }
        CallableGenericCallConformanceSourceV1::CompilerDerived { evidence } => {
            check_evidence_index(*evidence, evidence_count)?;
            ImportedConformanceBindingSource::CompilerDerived { evidence: *evidence }
        }
    };
    Ok(ImportedGenericCallConformanceBinding {
        callee_conformance,
        source,
    })
}

fn reconstruct_generic_conformance_key(
    key: &GenericConformanceKeyV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<GenericConformanceKey, ArtifactError> {
    let mut arguments = allocations
        .vector::<TypeId>(key.trait_ref.arguments.len())
        .map_err(ArtifactError::Allocation)?;
    for argument in &key.trait_ref.arguments {
        arguments.push(mapped_type(*argument, types, "generic conformance trait argument")?);
    }
    Ok(GenericConformanceKey::new(
        super::semantic_import::reconstruct_term(&key.subject, types, owners)?,
        TraitRef::new(
            owners.declaration(&key.trait_ref.declaration)?,
            allocations.boxed_slice(arguments).map_err(ArtifactError::Allocation)?,
        ),
    ))
}

fn require_canonical_callable(
    callable: &CallableId,
    callables: &ImportedCallableAuthority,
) -> Result<(), ArtifactError> {
    if callables.signature(callable).is_none() {
        return Err(invalid("witness selection callable has no imported canonical signature"));
    }
    Ok(())
}

fn check_evidence_index(index: u32, evidence_count: usize) -> Result<(), ArtifactError> {
    let index = usize::try_from(index).map_err(|_| invalid("witness evidence ordinal exceeds address space"))?;
    if index >= evidence_count {
        return Err(invalid("witness selection references an absent conformance-evidence node"));
    }
    Ok(())
}

fn mapped_type(
    id: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeId,
    types: &ReconstructedTypeGraph,
    family: &str,
) -> Result<TypeId, ArtifactError> {
    types
        .types
        .get(id.0 as usize)
        .copied()
        .ok_or_else(|| invalid(format!("{family} references an absent type row")))
}

fn mapped_parameter(
    id: phalcom_compiler_artifact::semantic::type_store::ArtifactTypeParameterId,
    types: &ReconstructedTypeGraph,
    family: &str,
) -> Result<TypeParameterId, ArtifactError> {
    types
        .parameters
        .get(id.0 as usize)
        .copied()
        .ok_or_else(|| invalid(format!("{family} references an absent type-parameter row")))
}

fn mapped_row(
    id: phalcom_compiler_artifact::semantic::type_store::ArtifactRecordRowId,
    types: &ReconstructedTypeGraph,
    family: &str,
) -> Result<RecordRowId, ArtifactError> {
    types
        .rows
        .get(id.0 as usize)
        .copied()
        .ok_or_else(|| invalid(format!("{family} references an absent record-row")))
}

fn reject_duplicate_parameter_pairs(
    pairs: &[(TypeParameterId, TypeId)],
    message: &'static str,
) -> Result<(), ArtifactError> {
    if pairs.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        Err(invalid(message))
    } else {
        Ok(())
    }
}

fn reconstruct_completeness(
    completeness: &CallableConformanceCompletenessV1,
    expected_trait: &crate::identity::DeclarationId,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ConformanceCompleteness, ArtifactError> {
    Ok(match completeness {
        CallableConformanceCompletenessV1::Complete => ConformanceCompleteness::Complete,
        CallableConformanceCompletenessV1::Incomplete { failures } => {
            let mut imported = allocations
                .vector::<ConformanceFailure>(failures.len())
                .map_err(ArtifactError::Allocation)?;
            for failure in failures {
                imported.push(match failure {
                    CallableConformanceFailureV1::Behavioral { requirement, reason } => {
                        let requirement = requirement_identity(requirement, owners)?;
                        if &requirement.owner != expected_trait {
                            return Err(invalid("witness completeness behavioral failure belongs to another trait"));
                        }
                        ConformanceFailure::Behavioral(RequirementFailure {
                            requirement,
                            reason: allocations.boxed_str(reason).map_err(ArtifactError::Allocation)?,
                        })
                    }
                    CallableConformanceFailureV1::AssociatedType { failure } => {
                        let failure = reconstruct_associated_type_failure(failure, owners, allocations)?;
                        if failure
                            .requirement
                            .as_ref()
                            .is_some_and(|requirement| &requirement.owner != expected_trait)
                        {
                            return Err(invalid("witness completeness associated-type failure belongs to another trait"));
                        }
                        ConformanceFailure::AssociatedType(failure)
                    }
                });
            }
            ConformanceCompleteness::Incomplete {
                failures: allocations.boxed_slice(imported).map_err(ArtifactError::Allocation)?,
            }
        }
        CallableConformanceCompletenessV1::Unknown { reason } => {
            ConformanceCompleteness::Unknown(super::callable_import::import_unknown_reason(reason, allocations)?)
        }
        CallableConformanceCompletenessV1::Blocked { reason } => {
            ConformanceCompleteness::Blocked(super::callable_binding_import::block_reason(
                reason,
                types,
                owners,
                allocations,
            )?)
        }
        CallableConformanceCompletenessV1::Dynamic { reason } => {
            ConformanceCompleteness::Dynamic(DynamicBoundaryObligation {
                reason: allocations.string(reason).map_err(ArtifactError::Allocation)?,
            })
        }
        CallableConformanceCompletenessV1::Cancelled => ConformanceCompleteness::Cancelled,
        CallableConformanceCompletenessV1::BudgetExceeded { report } => {
            ConformanceCompleteness::BudgetExceeded(super::callable_binding_import::budget_report(report))
        }
        CallableConformanceCompletenessV1::InternalFailure { message } => {
            ConformanceCompleteness::InternalFailure(
                allocations.boxed_str(message).map_err(ArtifactError::Allocation)?,
            )
        }
    })
}

fn reconstruct_associated_type_plan(
    plan: &CallableConformanceAssociatedTypePlanV1,
    expected_impl: &ImplId,
    expected_trait: &crate::identity::DeclarationId,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedAssociatedTypePlan, ArtifactError> {
    let impl_id = owners.implementation(&plan.impl_id)?;
    if &impl_id != expected_impl {
        return Err(invalid("associated-type plan implementation disagrees with its witness owner"));
    }

    let mut bindings = allocations
        .vector::<(AssociatedTypeRequirementId, AssociatedTypeBindingTemplate)>(plan.bindings.len())
        .map_err(ArtifactError::Allocation)?;
    for binding in &plan.bindings {
        let requirement = associated_type_requirement_identity(&binding.requirement, owners)?;
        if &requirement.owner != expected_trait {
            return Err(invalid("associated-type witness binding belongs to another trait"));
        }
        let value_template = match &binding.value {
            CallableAssociatedTypeBindingValueV1::Type { ty } => AssociatedTypeBindingValue::Type(
                *types
                    .types
                    .get(ty.0 as usize)
                    .ok_or_else(|| invalid("associated-type witness binding references an absent type"))?,
            ),
            CallableAssociatedTypeBindingValueV1::Dynamic => AssociatedTypeBindingValue::Dynamic,
        };
        let template = AssociatedTypeBindingTemplate {
            requirement: requirement.clone(),
            value_template,
            source_impl: owners.implementation(&binding.source_impl)?,
            source: source_span(&binding.source, owners)?,
        };
        bindings.push((requirement, template));
    }
    bindings.sort_by(|left, right| left.0.cmp(&right.0));
    if bindings.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(invalid("associated-type witness binding is duplicated after relocation"));
    }

    let mut failures = allocations
        .vector::<AssociatedTypeBindingFailure>(plan.failures.len())
        .map_err(ArtifactError::Allocation)?;
    for failure in &plan.failures {
        let imported = reconstruct_associated_type_failure(failure, owners, allocations)?;
        if imported
            .requirement
            .as_ref()
            .is_some_and(|requirement| &requirement.owner != expected_trait)
        {
            return Err(invalid("associated-type witness failure belongs to another trait"));
        }
        failures.push(imported);
    }

    let mut diagnostics = allocations
        .vector::<SemanticDiagnostic>(plan.diagnostics.len())
        .map_err(ArtifactError::Allocation)?;
    for diagnostic in &plan.diagnostics {
        diagnostics.push(reconstruct_semantic_diagnostic(
            diagnostic,
            types,
            owners,
            allocations,
        )?);
    }

    Ok(ImportedAssociatedTypePlan {
        impl_id,
        bindings: allocations.boxed_slice(bindings).map_err(ArtifactError::Allocation)?,
        failures: allocations.boxed_slice(failures).map_err(ArtifactError::Allocation)?,
        diagnostics: allocations.boxed_slice(diagnostics).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_associated_type_failure(
    failure: &CallableAssociatedTypeBindingFailureV1,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<AssociatedTypeBindingFailure, ArtifactError> {
    Ok(AssociatedTypeBindingFailure {
        kind: match &failure.kind {
            CallableAssociatedTypeBindingFailureKindV1::Missing => AssociatedTypeBindingFailureKind::Missing,
            CallableAssociatedTypeBindingFailureKindV1::Duplicate => AssociatedTypeBindingFailureKind::Duplicate,
            CallableAssociatedTypeBindingFailureKindV1::Unknown => AssociatedTypeBindingFailureKind::Unknown,
            CallableAssociatedTypeBindingFailureKindV1::Invalid => AssociatedTypeBindingFailureKind::Invalid,
        },
        name: allocations.boxed_str(&failure.name).map_err(ArtifactError::Allocation)?,
        reason: allocations.boxed_str(&failure.reason).map_err(ArtifactError::Allocation)?,
        source: source_span(&failure.source, owners)?,
        requirement: failure
            .requirement
            .as_ref()
            .map(|requirement| associated_type_requirement_identity(requirement, owners))
            .transpose()?,
    })
}

fn associated_type_requirement_identity(
    identity: &CallableAssociatedTypeRequirementIdentityV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<AssociatedTypeRequirementId, ArtifactError> {
    Ok(AssociatedTypeRequirementId::new(
        owners.declaration(&identity.owner)?,
        identity.index,
    ))
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
