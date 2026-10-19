//! Certified PERF003 P7 conformance-evidence graph reconstruction adapted to ARTF001.
//!
//! P7 reconstructed recursive native Arc/BTreeMap proof objects during import.
//! Current-main's proof objects contain map-backed environments without a
//! budgeted frozen constructor, so P2 crosses the trust boundary into a
//! relocation-complete immutable ordinal graph instead. T4 resolves these
//! already-validated ordinals into the one live semantic generation.

use super::callable_import::ImportedCallableAuthority;
use super::conformance_import::ImportedConformanceAuthority;
use super::diagnostic_import::source_span;
use super::identity::StableOwnerCatalog;
use super::trait_import::ImportedTraitAuthority;
use super::type_import::ReconstructedTypeGraph;
use super::witness_import::{
    ImportedAssociatedTypePlan, ImportedRequirementSelection, ImportedTypeEnvironment,
    associated_type_requirement_identity, check_evidence_index, mapped_parameter, mapped_type,
    reconstruct_associated_type_plan, reconstruct_generic_conformance_key,
    reconstruct_requirement_selection, reconstruct_requirement_view, reconstruct_type_environment,
};
use crate::generic_evidence::{CompilerDerivedConformanceKind, GenericConformanceKey};
use crate::identity::{DeclarationId, ImplId, ModuleId};
use crate::impls::{AssociatedTypeBindingValue, ExactAssociatedTypeBinding};
use crate::structural::{
    OrderedProductForm, StructuralClassification, UnorderedProductForm,
};
use crate::traits::{
    AssociatedTypeRequirementId, InstantiatedTraitRequirement, TraitRef, TraitRequirementId,
};
use crate::types::id::{TypeId, TypeParameterId};
use crate::types::parameter::{GenericConstraint, GenericSignature, TypeParameterOwner};
use phalcom_compiler_artifact::semantic::type_store::ParameterOwnerV1;
use phalcom_compiler_artifact::semantic::world::{
    CallableCompilerDerivedConformanceEvidenceV1, CallableCompilerDerivedConformanceKindV1,
    CallableConformanceEvidenceNodeV1, CallableDerivedConformanceEvidenceV1,
    CallableDerivedConformanceOriginV1, CallableDerivedConformanceRuleV1,
    CallableExactConformanceEvidenceV1,
    CallableNestedGenericEvidenceV1, CallableOrderedProductFormV1,
    CallableStructuralClassificationV1, CallableUnorderedProductFormV1, TraitRefV1,
};
use phalcom_compiler_artifact::{ArtifactError, ValidationError, ValidationLimits};
use phalcom_type_meta::allocation::AllocationBudget;

/// Relocated exact proof payload with ordinal references retained for the
/// final native graph materialization.
#[derive(Clone, Debug)]
pub struct ImportedExactConformanceEvidence {
    pub source_impl: ImplId,
    pub exact_target: TypeId,
    pub exact_trait_ref: TraitRef,
    pub impl_environment: ImportedTypeEnvironment,
    pub requirement_views: Box<[(TraitRequirementId, InstantiatedTraitRequirement)]>,
    pub requirements: Box<[(TraitRequirementId, ImportedRequirementSelection)]>,
    pub associated_types: Box<[(AssociatedTypeRequirementId, ExactAssociatedTypeBinding)]>,
    pub trait_header_contract: Box<[GenericConstraint]>,
    pub trait_header_conformance_proofs: Option<Box<[(GenericConformanceKey, u32)]>>,
}

#[derive(Clone, Debug)]
pub struct ImportedDerivedConformanceRule {
    pub impl_id: ImplId,
    pub source_module: ModuleId,
    pub target_parameter: TypeParameterId,
    pub target_template: TypeId,
    pub trait_ref: TraitRef,
    pub generic_signature: GenericSignature,
    pub associated_type_plan: ImportedAssociatedTypePlan,
    pub requirement_views: Box<[(TraitRequirementId, InstantiatedTraitRequirement)]>,
    pub requirements: Box<[(TraitRequirementId, ImportedRequirementSelection)]>,
    pub source: crate::diagnostic::SemanticSourceSpan,
}

#[derive(Clone, Debug)]
pub enum ImportedDerivedConformanceOrigin {
    ConditionalSource { exact_source: u32 },
    GuardedRule {
        rule: Box<ImportedDerivedConformanceRule>,
        exact_subject: TypeId,
        exact_trait_ref: TraitRef,
    },
}

#[derive(Clone, Debug)]
pub enum ImportedNestedGenericEvidence {
    Conformance { evidence: u32 },
    Equivalent,
    Subtype,
}

#[derive(Clone, Debug)]
pub struct ImportedDerivedConformanceEvidence {
    pub origin: ImportedDerivedConformanceOrigin,
    pub impl_bindings: Box<[(TypeParameterId, TypeId)]>,
    pub nested: Box<[ImportedNestedGenericEvidence]>,
}

#[derive(Clone, Debug)]
pub struct ImportedCompilerDerivedConformanceEvidence {
    pub key: GenericConformanceKey,
    pub kind: CompilerDerivedConformanceKind,
    pub classification: StructuralClassification,
}

#[derive(Clone, Debug)]
pub enum ImportedConformanceEvidenceNode {
    Exact(ImportedExactConformanceEvidence),
    Assumed {
        key: GenericConformanceKey,
        owner: TypeParameterOwner,
    },
    Derived(ImportedDerivedConformanceEvidence),
    CompilerDerived(ImportedCompilerDerivedConformanceEvidence),
}

/// Reusable relocated proof graph shared by witness plans and callable analyses.
#[derive(Clone, Debug, Default)]
pub struct ImportedConformanceEvidenceGraph {
    nodes: Box<[ImportedConformanceEvidenceNode]>,
}

impl ImportedConformanceEvidenceGraph {
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub(crate) fn node(&self, ordinal: u32) -> Option<&ImportedConformanceEvidenceNode> {
        self.nodes.get(ordinal as usize)
    }

    pub(crate) fn nodes(&self) -> &[ImportedConformanceEvidenceNode] {
        &self.nodes
    }
}

pub(crate) fn reconstruct_conformance_evidence_graph(
    records: &[CallableConformanceEvidenceNodeV1],
    conformances: &ImportedConformanceAuthority,
    prior_witnesses: &[super::witness_import::ImportedWitnessPlanPreflight],
    traits: &ImportedTraitAuthority,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedConformanceEvidenceGraph, ArtifactError> {
    if records.len() > limits.max_semantic_entries_per_family {
        return Err(invalid("conformance evidence graph exceeds semantic entry limit"));
    }
    let mut nodes = allocations
        .vector::<ImportedConformanceEvidenceNode>(records.len())
        .map_err(ArtifactError::Allocation)?;

    for record in records {
        nodes.push(match record {
            CallableConformanceEvidenceNodeV1::Exact { evidence } => {
                ImportedConformanceEvidenceNode::Exact(reconstruct_exact(
                    evidence,
                    records,
                    conformances,
                    prior_witnesses,
                    traits,
                    callables,
                    types,
                    owners,
                    limits,
                    allocations,
                )?)
            }
            CallableConformanceEvidenceNodeV1::Assumed { key, owner } => {
                ImportedConformanceEvidenceNode::Assumed {
                    key: reconstruct_generic_conformance_key(key, types, owners, allocations)?,
                    owner: parameter_owner(owner, owners)?,
                }
            }
            CallableConformanceEvidenceNodeV1::Derived { evidence } => {
                ImportedConformanceEvidenceNode::Derived(reconstruct_derived(
                    evidence,
                    records,
                    traits,
                    callables,
                    types,
                    owners,
                    limits,
                    allocations,
                )?)
            }
            CallableConformanceEvidenceNodeV1::CompilerDerived { evidence } => {
                ImportedConformanceEvidenceNode::CompilerDerived(reconstruct_compiler_derived(
                    evidence,
                    types,
                    owners,
                    allocations,
                )?)
            }
        });
    }

    Ok(ImportedConformanceEvidenceGraph {
        nodes: allocations.boxed_slice(nodes).map_err(ArtifactError::Allocation)?,
    })
}

fn reconstruct_exact(
    record: &CallableExactConformanceEvidenceV1,
    all_nodes: &[CallableConformanceEvidenceNodeV1],
    conformances: &ImportedConformanceAuthority,
    prior_witnesses: &[super::witness_import::ImportedWitnessPlanPreflight],
    traits: &ImportedTraitAuthority,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedExactConformanceEvidence, ArtifactError> {
    let source_impl = owners.implementation(&record.source_impl)?;
    // Certified P7 resolves Exact evidence against source witness plans that
    // were already reconstructed earlier in canonical witness order. The
    // native source-plan fingerprint is intentionally unavailable until T4,
    // but the ordering/identity dependency must already be proven here.
    if !prior_witnesses.iter().any(|plan| plan.impl_id == source_impl) {
        return Err(invalid(
            "exact conformance evidence has no previously staged source witness plan",
        ));
    }
    if conformances.get(&source_impl).is_none() {
        return Err(invalid(
            "exact conformance evidence source witness has no imported conformance head",
        ));
    }
    let exact_target = mapped_type(record.exact_target, types, "exact conformance target")?;
    let exact_trait_ref = trait_ref(&record.exact_trait_ref, types, owners, allocations)?;
    let trait_surface = traits
        .surface(&exact_trait_ref.declaration)
        .ok_or_else(|| invalid("exact conformance evidence references an absent imported trait surface"))?;
    let impl_environment = reconstruct_type_environment(&record.impl_environment, types, allocations)?;

    let mut requirement_views = allocations
        .vector::<(TraitRequirementId, InstantiatedTraitRequirement)>(record.requirement_views.len())
        .map_err(ArtifactError::Allocation)?;
    for view in &record.requirement_views {
        requirement_views.push(reconstruct_requirement_view(
            view,
            &exact_trait_ref.declaration,
            trait_surface,
            types,
            owners,
            allocations,
        )?);
    }
    requirement_views.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(
        &requirement_views,
        "exact conformance evidence contains duplicate requirement views",
    )?;

    let mut requirements = allocations
        .vector::<(TraitRequirementId, ImportedRequirementSelection)>(record.requirements.len())
        .map_err(ArtifactError::Allocation)?;
    for entry in &record.requirements {
        let requirement = super::witness_import::requirement_identity(&entry.requirement, owners)?;
        if requirement.owner != exact_trait_ref.declaration {
            return Err(invalid("exact conformance requirement selection belongs to another trait"));
        }
        let selection = reconstruct_requirement_selection(
            &entry.selection,
            &exact_trait_ref.declaration,
            callables,
            types,
            owners,
            all_nodes.len(),
            limits.max_recursion_depth,
            0,
            allocations,
        )?;
        requirements.push((requirement, selection));
    }
    requirements.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(
        &requirements,
        "exact conformance evidence contains duplicate requirement selections",
    )?;

    let mut associated_types = allocations
        .vector::<(AssociatedTypeRequirementId, ExactAssociatedTypeBinding)>(record.associated_types.len())
        .map_err(ArtifactError::Allocation)?;
    for binding in &record.associated_types {
        let requirement = associated_type_requirement_identity(&binding.requirement, owners)?;
        if requirement.owner != exact_trait_ref.declaration {
            return Err(invalid("exact associated-type evidence belongs to another trait"));
        }
        let value = match &binding.value {
            phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeBindingValueV1::Type { ty } => {
                AssociatedTypeBindingValue::Type(mapped_type(*ty, types, "exact associated-type binding")?)
            }
            phalcom_compiler_artifact::semantic::world::CallableAssociatedTypeBindingValueV1::Dynamic => {
                AssociatedTypeBindingValue::Dynamic
            }
        };
        associated_types.push((
            requirement.clone(),
            ExactAssociatedTypeBinding {
                requirement,
                value,
                source_impl: owners.implementation(&binding.source_impl)?,
                source: source_span(&binding.source, owners)?,
            },
        ));
    }
    associated_types.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(
        &associated_types,
        "exact conformance evidence contains duplicate associated-type bindings",
    )?;

    let mut trait_header_contract = allocations
        .vector::<GenericConstraint>(record.trait_header_contract.len())
        .map_err(ArtifactError::Allocation)?;
    for constraint in &record.trait_header_contract {
        trait_header_contract.push(super::semantic_import::reconstruct_constraint(
            constraint,
            types,
            owners,
            allocations,
        )?);
    }

    let trait_header_conformance_proofs = record
        .trait_header_conformance_proofs
        .as_ref()
        .map(|proofs| {
            let mut imported = allocations
                .vector::<(GenericConformanceKey, u32)>(proofs.len())
                .map_err(ArtifactError::Allocation)?;
            for proof in proofs {
                check_evidence_index(proof.evidence, all_nodes.len())?;
                imported.push((
                    reconstruct_generic_conformance_key(&proof.key, types, owners, allocations)?,
                    proof.evidence,
                ));
            }
            imported.sort_by(|left, right| left.0.cmp(&right.0));
            if imported.windows(2).any(|pair| pair[0].0 == pair[1].0) {
                return Err(invalid("exact trait-header proof map contains duplicate generic conformance keys"));
            }
            allocations.boxed_slice(imported).map_err(ArtifactError::Allocation)
        })
        .transpose()?;

    Ok(ImportedExactConformanceEvidence {
        source_impl,
        exact_target,
        exact_trait_ref,
        impl_environment,
        requirement_views: allocations.boxed_slice(requirement_views).map_err(ArtifactError::Allocation)?,
        requirements: allocations.boxed_slice(requirements).map_err(ArtifactError::Allocation)?,
        associated_types: allocations.boxed_slice(associated_types).map_err(ArtifactError::Allocation)?,
        trait_header_contract: allocations.boxed_slice(trait_header_contract).map_err(ArtifactError::Allocation)?,
        trait_header_conformance_proofs,
    })
}

fn reconstruct_derived(
    record: &CallableDerivedConformanceEvidenceV1,
    all_nodes: &[CallableConformanceEvidenceNodeV1],
    traits: &ImportedTraitAuthority,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedDerivedConformanceEvidence, ArtifactError> {
    let origin = match &record.origin {
        CallableDerivedConformanceOriginV1::ConditionalSource { exact_source } => {
            check_evidence_index(*exact_source, all_nodes.len())?;
            if !matches!(
                all_nodes.get(*exact_source as usize),
                Some(CallableConformanceEvidenceNodeV1::Exact { .. })
            ) {
                return Err(invalid("derived conformance conditional source does not reference exact evidence"));
            }
            ImportedDerivedConformanceOrigin::ConditionalSource {
                exact_source: *exact_source,
            }
        }
        CallableDerivedConformanceOriginV1::GuardedRule {
            rule,
            exact_subject,
            exact_trait_ref,
        } => {
            let imported_rule = reconstruct_derived_rule(
                rule,
                all_nodes.len(),
                traits,
                callables,
                types,
                owners,
                limits,
                allocations,
            )?;
            ImportedDerivedConformanceOrigin::GuardedRule {
                rule: allocations.boxed(imported_rule).map_err(ArtifactError::Allocation)?,
                exact_subject: mapped_type(*exact_subject, types, "guarded derived exact subject")?,
                exact_trait_ref: trait_ref(exact_trait_ref, types, owners, allocations)?,
            }
        }
    };

    let mut impl_bindings = allocations
        .vector::<(TypeParameterId, TypeId)>(record.impl_bindings.len())
        .map_err(ArtifactError::Allocation)?;
    for binding in &record.impl_bindings {
        impl_bindings.push((
            mapped_parameter(binding.parameter, types, "derived conformance parameter")?,
            mapped_type(binding.ty, types, "derived conformance binding")?,
        ));
    }
    impl_bindings.sort_by_key(|(parameter, _)| *parameter);
    if impl_bindings.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err(invalid("derived conformance binds one impl parameter more than once"));
    }

    let mut nested = allocations
        .vector::<ImportedNestedGenericEvidence>(record.nested.len())
        .map_err(ArtifactError::Allocation)?;
    for child in &record.nested {
        nested.push(match child {
            CallableNestedGenericEvidenceV1::Conformance { evidence } => {
                check_evidence_index(*evidence, all_nodes.len())?;
                ImportedNestedGenericEvidence::Conformance { evidence: *evidence }
            }
            CallableNestedGenericEvidenceV1::Equivalent => ImportedNestedGenericEvidence::Equivalent,
            CallableNestedGenericEvidenceV1::Subtype => ImportedNestedGenericEvidence::Subtype,
        });
    }

    Ok(ImportedDerivedConformanceEvidence {
        origin,
        impl_bindings: allocations.boxed_slice(impl_bindings).map_err(ArtifactError::Allocation)?,
        nested: allocations.boxed_slice(nested).map_err(ArtifactError::Allocation)?,
    })
}

pub(crate) fn reconstruct_derived_rule(
    rule: &CallableDerivedConformanceRuleV1,
    evidence_count: usize,
    traits: &ImportedTraitAuthority,
    callables: &ImportedCallableAuthority,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    limits: &ValidationLimits,
    allocations: &mut AllocationBudget,
) -> Result<ImportedDerivedConformanceRule, ArtifactError> {
    let impl_id = owners.implementation(&rule.impl_id)?;
    let source_module = owners.module(&rule.source_module)?;
    if impl_id.module != source_module {
        return Err(invalid("derived conformance rule implementation disagrees with its source module"));
    }
    let target_parameter = mapped_parameter(rule.target_parameter, types, "derived conformance target parameter")?;
    let target_template = mapped_type(rule.target_template, types, "derived conformance target template")?;
    let trait_ref = trait_ref(&rule.trait_ref, types, owners, allocations)?;
    let trait_surface = traits
        .surface(&trait_ref.declaration)
        .ok_or_else(|| invalid("derived conformance rule references an absent imported trait surface"))?;
    let generic_signature = super::semantic_import::reconstruct_signature(
        &rule.generic_signature,
        types,
        owners,
        allocations,
    )?;
    let associated_type_plan = reconstruct_associated_type_plan(
        &rule.associated_type_plan,
        &impl_id,
        &trait_ref.declaration,
        types,
        owners,
        allocations,
    )?;

    let mut requirement_views = allocations
        .vector::<(TraitRequirementId, InstantiatedTraitRequirement)>(rule.requirement_views.len())
        .map_err(ArtifactError::Allocation)?;
    for view in &rule.requirement_views {
        requirement_views.push(reconstruct_requirement_view(
            view,
            &trait_ref.declaration,
            trait_surface,
            types,
            owners,
            allocations,
        )?);
    }
    requirement_views.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(
        &requirement_views,
        "derived conformance rule contains duplicate requirement views",
    )?;

    let mut requirements = allocations
        .vector::<(TraitRequirementId, ImportedRequirementSelection)>(rule.requirements.len())
        .map_err(ArtifactError::Allocation)?;
    for entry in &rule.requirements {
        let requirement = super::witness_import::requirement_identity(&entry.requirement, owners)?;
        if requirement.owner != trait_ref.declaration {
            return Err(invalid("derived conformance requirement selection belongs to another trait"));
        }
        requirements.push((
            requirement,
            reconstruct_requirement_selection(
                &entry.selection,
                &trait_ref.declaration,
                callables,
                types,
                owners,
                evidence_count,
                limits.max_recursion_depth,
                0,
                allocations,
            )?,
        ));
    }
    requirements.sort_by(|left, right| left.0.cmp(&right.0));
    reject_duplicate_keys(
        &requirements,
        "derived conformance rule contains duplicate requirement selections",
    )?;

    let source = source_span(&rule.source, owners)?;
    if source.module != source_module {
        return Err(invalid("derived conformance rule source span belongs to another module"));
    }

    Ok(ImportedDerivedConformanceRule {
        impl_id,
        source_module,
        target_parameter,
        target_template,
        trait_ref,
        generic_signature,
        associated_type_plan,
        requirement_views: allocations.boxed_slice(requirement_views).map_err(ArtifactError::Allocation)?,
        requirements: allocations.boxed_slice(requirements).map_err(ArtifactError::Allocation)?,
        source,
    })
}

fn reconstruct_compiler_derived(
    evidence: &CallableCompilerDerivedConformanceEvidenceV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<ImportedCompilerDerivedConformanceEvidence, ArtifactError> {
    Ok(ImportedCompilerDerivedConformanceEvidence {
        key: reconstruct_generic_conformance_key(&evidence.key, types, owners, allocations)?,
        kind: match &evidence.kind {
            CallableCompilerDerivedConformanceKindV1::Product => CompilerDerivedConformanceKind::Product,
            CallableCompilerDerivedConformanceKindV1::OrderedProduct => CompilerDerivedConformanceKind::OrderedProduct,
            CallableCompilerDerivedConformanceKindV1::UnorderedProduct => CompilerDerivedConformanceKind::UnorderedProduct,
            CallableCompilerDerivedConformanceKindV1::Sum => CompilerDerivedConformanceKind::Sum,
        },
        classification: classification(&evidence.classification, owners)?,
    })
}

fn classification(
    classification: &CallableStructuralClassificationV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<StructuralClassification, ArtifactError> {
    Ok(match classification {
        CallableStructuralClassificationV1::Unsupported => StructuralClassification::Unsupported,
        CallableStructuralClassificationV1::OrderedProduct { form } => {
            StructuralClassification::OrderedProduct(match form {
                CallableOrderedProductFormV1::Unit => OrderedProductForm::Unit,
                CallableOrderedProductFormV1::Tuple => OrderedProductForm::Tuple,
                CallableOrderedProductFormV1::TupleFromPacks => OrderedProductForm::TupleFromPacks,
                CallableOrderedProductFormV1::Pack => OrderedProductForm::Pack,
                CallableOrderedProductFormV1::PackFromPacks => OrderedProductForm::PackFromPacks,
                CallableOrderedProductFormV1::Data { owner } => {
                    OrderedProductForm::Data { owner: owners.declaration(owner)? }
                }
            })
        }
        CallableStructuralClassificationV1::UnorderedProduct { form } => {
            StructuralClassification::UnorderedProduct(match form {
                CallableUnorderedProductFormV1::Record => UnorderedProductForm::Record,
                CallableUnorderedProductFormV1::Data { owner } => {
                    UnorderedProductForm::Data { owner: owners.declaration(owner)? }
                }
            })
        }
        CallableStructuralClassificationV1::Sum { owner } => {
            StructuralClassification::Sum { owner: owners.declaration(owner)? }
        }
    })
}

fn parameter_owner(
    owner: &ParameterOwnerV1,
    owners: &StableOwnerCatalog<'_>,
) -> Result<TypeParameterOwner, ArtifactError> {
    Ok(match owner {
        ParameterOwnerV1::Declaration { owner } => TypeParameterOwner::Declaration(owners.declaration(owner)?),
        ParameterOwnerV1::Callable { owner } => TypeParameterOwner::Callable(owners.callable(owner)?),
        ParameterOwnerV1::Impl { owner } => TypeParameterOwner::Impl(owners.implementation(owner)?),
    })
}

fn trait_ref(
    row: &TraitRefV1,
    types: &ReconstructedTypeGraph,
    owners: &StableOwnerCatalog<'_>,
    allocations: &mut AllocationBudget,
) -> Result<TraitRef, ArtifactError> {
    let mut arguments = allocations
        .vector::<TypeId>(row.arguments.len())
        .map_err(ArtifactError::Allocation)?;
    for argument in &row.arguments {
        arguments.push(mapped_type(*argument, types, "trait reference argument")?);
    }
    Ok(TraitRef::new(
        owners.declaration(&row.declaration)?,
        allocations.boxed_slice(arguments).map_err(ArtifactError::Allocation)?,
    ))
}

fn reject_duplicate_keys<K: PartialEq, T>(
    rows: &[(K, T)],
    message: &'static str,
) -> Result<(), ArtifactError> {
    if rows.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        Err(invalid(message))
    } else {
        Ok(())
    }
}

fn invalid(message: impl Into<String>) -> ArtifactError {
    ValidationError::ReferentialIntegrity(message.into()).into()
}
