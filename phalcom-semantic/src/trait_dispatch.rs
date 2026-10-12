//! Bounded source index for trait-evidenced ordinary member dispatch.
//!
//! This index is deliberately only a discovery product.  It answers which
//! conformance requirements could be relevant for a receiver family and
//! selector; exact target matching and P2 conformance evidence remain the
//! authority for deciding whether a candidate is usable.

use crate::db::ProductFingerprint;
use crate::declarations::DeclarationTypeTable;
use crate::generic_evidence::{GenericConformanceEvidence, GenericConformanceKey, GenericConformanceProof, GenericEvidenceEnvironment};
use crate::identity::{CallableId, DataComponentId, DeclarationId, DispatchSide, ImplId, TypeId, VariantId};
use crate::impls::{ConformanceIndex, ConformanceTarget, ConformanceWitnessPlan, RequirementSelectionTemplate};
use crate::signature::CallableSemanticSignature;
use crate::traits::{TraitRef, TraitRequirementId, TraitSurfaceTable};
use crate::types::evidence::UnknownReason;
use crate::types::outcome::{BlockReason, BudgetReport, DynamicBoundaryObligation};
use crate::types::relation::TypeHierarchy;
use crate::types::store::{TypeData, TypeStore};
use phalcom_common::selector::Selector;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Coarse target family used to bound ordinary trait-dispatch discovery.
///
/// Generic applications are bucketed by their nominal declaration while an
/// exact enum case remains exact.  The latter is important: a conformance for
/// one case must not become a conformance for the enum root or a sibling case.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TraitDispatchTargetFamily {
    Declaration(DeclarationId),
    ExactEnumCase(VariantId),
}

impl From<&ConformanceTarget> for TraitDispatchTargetFamily {
    fn from(target: &ConformanceTarget) -> Self {
        match target {
            ConformanceTarget::Declaration(declaration) => Self::Declaration(declaration.clone()),
            ConformanceTarget::ExactEnumCase(variant) => Self::ExactEnumCase(variant.clone()),
        }
    }
}

/// Stable ordinary lookup bucket key.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TraitDispatchBucketKey {
    pub target_family: TraitDispatchTargetFamily,
    pub selector: Selector,
    pub side: DispatchSide,
}

/// One body-independent source contribution to a trait-dispatch bucket.
///
/// This is not conformance evidence.  In particular, incomplete conformances
/// may still be retained here so the exact query can preserve their proof
/// state instead of confusing them with an absent source contribution.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TraitDispatchContribution {
    pub impl_id: ImplId,
    pub target_family: TraitDispatchTargetFamily,
    pub trait_declaration: DeclarationId,
    pub requirement: TraitRequirementId,
    pub selector: Selector,
    pub side: DispatchSide,
}

/// Immutable, deterministic discovery index for ordinary trait dispatch.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TraitDispatchIndex {
    buckets: BTreeMap<TraitDispatchBucketKey, Box<[TraitDispatchContribution]>>,
    fingerprint: ProductFingerprint,
}

/// Borrowed immutable semantic inputs used by checker body analysis. The view
/// contains no mutable cache and does not replace any P1/P2 product.
pub struct ConformanceSemanticView<'a> {
    pub trait_dispatch: &'a TraitDispatchIndex,
    pub conformance_index: &'a ConformanceIndex,
    pub witness_plans: &'a BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>,
    pub trait_surfaces: &'a TraitSurfaceTable,
    pub derived_conformance_rules: &'a crate::derived_conformance::DerivedConformanceRuleIndex,
    /// Optional view of current declaration-owned callable results. It is
    /// supplied only after the session's callable-result refresh so dispatch
    /// can project inferred trait-default returns without storing a second
    /// copy in trait/conformance products.
    pub callable_signatures: Option<&'a crate::signature::CallableSignatureTable>,
}

impl ConformanceSemanticView<'_> {
    /// Returns the dependency fingerprint for one exact conformance view.
    /// Source plans own their semantic contract fingerprint; guarded derived
    /// rules retain their existing independent fingerprint authority.
    pub fn fingerprint_for(&self, impl_id: &ImplId) -> Option<ProductFingerprint> {
        self.witness_plans
            .get(impl_id)
            .map(|plan| plan.fingerprint)
            .or_else(|| self.derived_conformance_rules.get_rule(impl_id).map(|rule| rule.fingerprint))
    }

    pub fn bucket_fingerprint(&self, key: &TraitDispatchBucketKey) -> ProductFingerprint {
        self.trait_dispatch.bucket_fingerprint(key)
    }

    pub fn fingerprint(&self) -> ProductFingerprint {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.trait_dispatch.fingerprint().raw().hash(&mut hasher);
        for (impl_id, plan) in self.witness_plans {
            impl_id.hash(&mut hasher);
            plan.fingerprint.raw().hash(&mut hasher);
        }
        for (declaration, surface) in self.trait_surfaces.iter() {
            declaration.hash(&mut hasher);
            crate::db::fingerprint::trait_surface_product_fingerprint(surface).raw().hash(&mut hasher);
        }
        for rule in self.derived_conformance_rules.iter() {
            rule.fingerprint.raw().hash(&mut hasher);
        }
        ProductFingerprint::new(hasher.finish())
    }
}

impl TraitDispatchIndex {
    pub fn build(conformance_index: &ConformanceIndex, trait_surfaces: &TraitSurfaceTable) -> Self {
        let mut buckets: BTreeMap<TraitDispatchBucketKey, Vec<TraitDispatchContribution>> = BTreeMap::new();
        for (impl_id, contribution) in conformance_index.iter().filter(|(_, contribution)| contribution.is_lookup_eligible()) {
            let target_family = TraitDispatchTargetFamily::from(&contribution.target);
            let Some(surface) = trait_surfaces.get(&contribution.trait_ref.declaration) else {
                continue;
            };
            for (requirement, member) in surface.iter() {
                let entry = TraitDispatchContribution {
                    impl_id: impl_id.clone(),
                    target_family: target_family.clone(),
                    trait_declaration: contribution.trait_ref.declaration.clone(),
                    requirement: requirement.clone(),
                    selector: member.requirement.selector.clone(),
                    side: member.requirement.side,
                };
                buckets
                    .entry(TraitDispatchBucketKey {
                        target_family: target_family.clone(),
                        selector: entry.selector.clone(),
                        side: entry.side,
                    })
                    .or_default()
                    .push(entry);
            }
        }

        let buckets = buckets
            .into_iter()
            .map(|(key, mut entries)| {
                entries.sort();
                entries.dedup();
                (key, entries.into_boxed_slice())
            })
            .collect::<BTreeMap<_, _>>();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        buckets.hash(&mut hasher);
        Self {
            buckets,
            fingerprint: ProductFingerprint::new(hasher.finish()),
        }
    }

    pub fn candidates_for(&self, target_family: &TraitDispatchTargetFamily, selector: &Selector, side: DispatchSide) -> &[TraitDispatchContribution] {
        self.buckets
            .get(&TraitDispatchBucketKey {
                target_family: target_family.clone(),
                selector: selector.clone(),
                side,
            })
            .map(Box::as_ref)
            .unwrap_or(&[])
    }

    pub fn buckets(&self) -> &BTreeMap<TraitDispatchBucketKey, Box<[TraitDispatchContribution]>> {
        &self.buckets
    }

    pub fn bucket_fingerprint(&self, key: &TraitDispatchBucketKey) -> ProductFingerprint {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        key.hash(&mut hasher);
        if let Some(entries) = self.buckets.get(key) {
            entries.hash(&mut hasher);
        }
        ProductFingerprint::new(hasher.finish())
    }

    pub fn fingerprint(&self) -> ProductFingerprint {
        self.fingerprint
    }

    pub fn len(&self) -> usize {
        self.buckets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buckets.is_empty()
    }
}

/// One exact, proven trait requirement exposed by ordinary lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitEvidencedMemberCandidate {
    pub exact_target: TypeId,
    pub exact_trait_ref: TraitRef,
    pub requirement: TraitRequirementId,
    pub source_impl: ImplId,
    pub evidence: GenericConformanceEvidence,
    pub evidence_fingerprint: ProductFingerprint,
    pub selection: RequirementSelectionTemplate,
    pub signature: CallableSemanticSignature,
    pub requirement_targets: BTreeMap<TraitRequirementId, RequirementSelectionTemplate>,
    pub convergence: TraitDispatchConvergenceKey,
}

/// Semantic identity used when several proven trait requirements expose one
/// ordinary selector. It distinguishes detached conformance callables and
/// defaults from shared inherent/data capabilities.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum TraitDispatchConvergenceKey {
    Inherent(CallableId),
    ConditionalInherent {
        callable: CallableId,
        impl_id: ImplId,
        applicability_fingerprint: ProductFingerprint,
    },
    DataComponent(DataComponentId),
    ConformanceCallable(CallableId),
    TraitDefault {
        callable: CallableId,
        evidence_fingerprint: ProductFingerprint,
    },
}

/// Per-expression semantic selection produced after exact P1/P2 proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TraitDispatchSelection {
    pub source_impl: ImplId,
    pub exact_target: TypeId,
    pub exact_trait_ref: TraitRef,
    pub requirement: TraitRequirementId,
    pub callable: Option<CallableId>,
    pub signature: CallableSemanticSignature,
    pub evidence_fingerprint: ProductFingerprint,
    /// Canonical proof product authorizing this selection, including any
    /// guarded rule substitution and nested conformance evidence.
    pub evidence: GenericConformanceEvidence,
    pub selection: RequirementSelectionTemplate,
    pub requirement_targets: BTreeMap<TraitRequirementId, RequirementSelectionTemplate>,
}

/// Assumed trait dispatch selection produced from a generic direct conformance bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssumedTraitDispatchSelection {
    pub conformance: GenericConformanceKey,
    pub requirement: TraitRequirementId,
    pub signature: CallableSemanticSignature,
}

/// The semantic origins of a trait-dispatch expression. Abstract sites
/// inside a checked default carry only their canonical requirement; ordinary
/// sites carry exact conformance evidence and the selected target; assumed sites
/// carry the generic conformance assumption and resolved requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraitDispatchSite {
    AbstractRequirement { requirement: TraitRequirementId },
    Evidenced(TraitDispatchSelection),
    Assumed(AssumedTraitDispatchSelection),
}

/// Proof-aware result of bounded trait-evidenced discovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraitDispatchResolution {
    Found(TraitDispatchSelection),
    Ambiguous(Box<[TraitEvidencedMemberCandidate]>),
    CoherenceConflict(Box<[ImplId]>),
    Missing,
    Incomplete(ImplId),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

/// Terminal proof state retained when ordinary member discovery found a trait
/// candidate but could not produce executable conformance evidence. This is
/// deliberately distinct from runtime-dynamic dispatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraitDispatchTerminal {
    Incomplete(ImplId),
    CoherenceConflict(Box<[ImplId]>),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

/// Computes the bounded target-family bucket for an ordinary receiver.
pub fn target_family_for_receiver(store: &TypeStore, receiver: TypeId) -> Option<TraitDispatchTargetFamily> {
    match store.get(receiver) {
        TypeData::ExactCase { variant, .. } => Some(TraitDispatchTargetFamily::ExactEnumCase(store.variant_identity(*variant).clone())),
        _ => store.nominal_origin_declaration(receiver).cloned().map(TraitDispatchTargetFamily::Declaration),
    }
}

/// Resolves ordinary trait candidates using only the immutable discovery index
/// and exact P1/P2 products. The caller supplies a mutable cloned store when
/// resolving from a snapshot, matching the existing pure exact-evidence query.
pub fn resolve_trait_evidenced_candidates(
    index: &TraitDispatchIndex,
    conformance_index: &ConformanceIndex,
    plans: &std::collections::BTreeMap<ImplId, std::sync::Arc<ConformanceWitnessPlan>>,
    trait_surfaces: &TraitSurfaceTable,
    derived_rules: &crate::derived_conformance::DerivedConformanceRuleIndex,
    lexical_evidence: Option<&GenericEvidenceEnvironment>,
    declarations: &DeclarationTypeTable,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    receiver: TypeId,
    selector: &Selector,
    side: DispatchSide,
) -> TraitDispatchResolution {
    let target_family = target_family_for_receiver(store, receiver);
    let mut candidate_goals = std::collections::BTreeSet::new();
    // Source surfaces own selector discovery. Explicit conformance products
    // and guarded-rule consequents contribute only possible canonical trait
    // goals; neither is treated as evidence until the generic proof authority
    // resolves that goal below.
    for (trait_declaration, surface) in trait_surfaces.iter() {
        let requirements = surface
            .iter()
            .filter(|(_, member)| member.requirement.selector == *selector && member.requirement.side == side)
            .map(|(requirement, _)| requirement.clone())
            .collect::<Vec<_>>();
        if requirements.is_empty() {
            continue;
        }

        if let Some(target_family) = target_family.as_ref() {
            for contribution in index.candidates_for(target_family, selector, side) {
                if contribution.trait_declaration != *trait_declaration {
                    continue;
                }
                let Some(source) = conformance_index.get(&contribution.impl_id) else {
                    continue;
                };
                for requirement in &requirements {
                    candidate_goals.insert((source.trait_ref.clone(), requirement.clone()));
                }
            }
        }

        for rule in derived_rules.rules_for_trait(trait_declaration) {
            let mut substitution = crate::types::substitution::TypeSubstitution::new();
            substitution.bind(rule.target_parameter, receiver);
            let trait_ref = crate::generic_evidence::substitute_trait_ref(&rule.trait_ref, &substitution, store);
            for requirement in &requirements {
                candidate_goals.insert((trait_ref.clone(), requirement.clone()));
            }
        }
    }

    let mut proven = Vec::new();
    let mut terminal = None;
    for (trait_ref, requirement) in candidate_goals {
        let proof = crate::generic_evidence::query_generic_conformance_proof(
            &crate::types::parameter::TypeTerm::Canonical(receiver),
            &trait_ref,
            lexical_evidence,
            Some(conformance_index),
            Some(plans),
            Some(trait_surfaces),
            Some(derived_rules),
            Some(declarations),
            Some(hierarchy),
            Some(&mut *store),
        );
        match proof {
            GenericConformanceProof::Proven(evidence) => {
                if let Some(candidate) = candidate_from_proof(evidence, receiver, &requirement, store) {
                    proven.push(candidate);
                }
            }
            GenericConformanceProof::Ambiguous(evidences) => {
                for evidence in evidences.into_vec() {
                    if let Some(candidate) = candidate_from_proof(evidence, receiver, &requirement, store) {
                        proven.push(candidate);
                    }
                }
            }
            GenericConformanceProof::CoherenceConflict(impls) => {
                retain_terminal(&mut terminal, TraitDispatchResolution::CoherenceConflict(impls));
            }
            GenericConformanceProof::Incomplete(impl_id) => retain_terminal(&mut terminal, TraitDispatchResolution::Incomplete(impl_id)),
            GenericConformanceProof::Unknown(reason) => retain_terminal(&mut terminal, TraitDispatchResolution::Unknown(reason)),
            GenericConformanceProof::Blocked(reason) => retain_terminal(&mut terminal, TraitDispatchResolution::Blocked(reason)),
            GenericConformanceProof::Dynamic(obligation) => retain_terminal(&mut terminal, TraitDispatchResolution::Dynamic(obligation)),
            GenericConformanceProof::Cancelled => retain_terminal(&mut terminal, TraitDispatchResolution::Cancelled),
            GenericConformanceProof::BudgetExceeded(report) => retain_terminal(&mut terminal, TraitDispatchResolution::BudgetExceeded(report)),
            GenericConformanceProof::InternalFailure(message) => retain_terminal(&mut terminal, TraitDispatchResolution::InternalFailure(message)),
            GenericConformanceProof::Missing => {}
        }
    }
    if let Some(conflict @ TraitDispatchResolution::CoherenceConflict(_)) = terminal.as_ref() {
        return conflict.clone();
    }
    if proven.is_empty() {
        return terminal.unwrap_or(TraitDispatchResolution::Missing);
    }
    proven.sort_by_key(|candidate| (candidate.convergence.clone(), candidate.source_impl.clone(), candidate.requirement.clone()));
    proven.dedup_by(|left, right| left.convergence == right.convergence);
    if proven.len() == 1 {
        let candidate = proven.pop().expect("one candidate");
        return TraitDispatchResolution::Found(TraitDispatchSelection {
            source_impl: candidate.source_impl,
            exact_target: receiver,
            exact_trait_ref: candidate.exact_trait_ref,
            requirement: candidate.requirement,
            callable: selected_callable(&candidate.selection),
            signature: candidate.signature,
            evidence_fingerprint: candidate.evidence_fingerprint,
            evidence: candidate.evidence,
            selection: candidate.selection,
            requirement_targets: candidate.requirement_targets,
        });
    }
    TraitDispatchResolution::Ambiguous(proven.into_boxed_slice())
}

fn candidate_from_proof(
    evidence: GenericConformanceEvidence,
    exact_target: TypeId,
    requirement_id: &TraitRequirementId,
    store: &mut TypeStore,
) -> Option<TraitEvidencedMemberCandidate> {
    let (source_impl, exact_trait_ref, requirement_views, selections) = match &evidence {
        GenericConformanceEvidence::Exact(exact) => (
            exact.source_impl.clone(),
            exact.exact_trait_ref.clone(),
            &exact.requirement_views,
            &exact.requirements,
        ),
        GenericConformanceEvidence::Derived(derived) => {
            let source_impl = match &derived.origin {
                crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => exact_source.source_impl.clone(),
                crate::generic_evidence::DerivedConformanceOrigin::GuardedRule { rule, .. } => rule.impl_id.clone(),
            };
            let exact_trait_ref = match &derived.origin {
                crate::generic_evidence::DerivedConformanceOrigin::ConditionalSource { exact_source } => exact_source.exact_trait_ref.clone(),
                crate::generic_evidence::DerivedConformanceOrigin::GuardedRule { exact_trait_ref, .. } => exact_trait_ref.clone(),
            };
            (source_impl, exact_trait_ref, derived.requirement_views(), derived.requirements())
        }
        // Direct generic assumptions retain their established
        // `TraitDispatchSite::Assumed` path in the checker.
        GenericConformanceEvidence::Assumed(_) => return None,
        // Compiler-derived structural capabilities have no dispatchable methods.
        GenericConformanceEvidence::CompilerDerived(_) => return None,
    };
    let requirement = requirement_views.get(requirement_id)?;
    let selection = selections.get(requirement_id)?.clone();
    let signature = match &evidence {
        GenericConformanceEvidence::Derived(derived) => match &derived.origin {
            crate::generic_evidence::DerivedConformanceOrigin::GuardedRule { exact_subject, .. } => {
                let mut environment = crate::types::environment::TypeEnvironment::new();
                for (&param, &ty) in derived.impl_bindings.bindings() {
                    environment.bind_param(param, ty);
                }
                environment.bind_self(*exact_subject);
                let mut sig = requirement.signature.clone();
                sig.parameters = sig
                    .parameters
                    .into_vec()
                    .into_iter()
                    .map(|mut parameter| {
                        parameter.declared_type = crate::traits::instantiate_declared_type_fact(store, &environment, &parameter.declared_type);
                        parameter.contract = match parameter.contract {
                            crate::signature::ParameterContract::Fixed(_) => crate::signature::ParameterContract::Fixed(parameter.declared_type.clone()),
                            crate::signature::ParameterContract::Rest { mode, contract } => crate::signature::ParameterContract::Rest {
                                mode,
                                contract: match contract {
                                    crate::signature::RestValueContract::Unconstrained => crate::signature::RestValueContract::Unconstrained,
                                    crate::signature::RestValueContract::Element(fact) => {
                                        crate::signature::RestValueContract::Element(crate::traits::instantiate_declared_type_fact(store, &environment, &fact))
                                    }
                                    crate::signature::RestValueContract::Lane(lane) => crate::signature::RestValueContract::Lane(
                                        crate::types::environment::TypeView::new(lane, environment.clone()).materialize(store),
                                    ),
                                    crate::signature::RestValueContract::Invalid(reason) => crate::signature::RestValueContract::Invalid(reason),
                                },
                            },
                        };
                        parameter.rest = parameter.contract.rest_mode();
                        parameter
                    })
                    .collect::<Vec<_>>()
                    .into_boxed_slice();
                sig.declared_return = crate::traits::instantiate_declared_type_fact(store, &environment, &sig.declared_return);
                if let Some(inferred_return) = &mut sig.inferred_return {
                    *inferred_return = inferred_return.map_type(|ty| crate::types::environment::TypeView::new(ty, environment.clone()).materialize(store));
                }
                sig
            }
            _ => requirement.signature.clone(),
        },
        _ => requirement.signature.clone(),
    };
    let requirement_targets = selections.clone();
    let evidence_fingerprint = evidence_fingerprint(&evidence);
    let convergence = selection_convergence_key(&selection, evidence_fingerprint);
    Some(TraitEvidencedMemberCandidate {
        exact_target,
        exact_trait_ref,
        requirement: requirement_id.clone(),
        source_impl,
        evidence,
        evidence_fingerprint,
        selection,
        signature,
        requirement_targets,
        convergence,
    })
}

/// Materializes one ordinary selected trait requirement from proof evidence
/// already obtained for an exact target and trait reference. This keeps
/// callers that resolve a known capability on the canonical selection path
/// without asking the broad selector-discovery index to choose among
/// unrelated trait arguments.
pub(crate) fn selection_from_proven_requirement(
    evidence: GenericConformanceEvidence,
    exact_target: TypeId,
    requirement: &TraitRequirementId,
    store: &mut TypeStore,
) -> Option<TraitDispatchSelection> {
    let candidate = candidate_from_proof(evidence, exact_target, requirement, store)?;
    Some(TraitDispatchSelection {
        source_impl: candidate.source_impl,
        exact_target: candidate.exact_target,
        exact_trait_ref: candidate.exact_trait_ref,
        requirement: candidate.requirement,
        callable: selected_callable(&candidate.selection),
        signature: candidate.signature,
        evidence_fingerprint: candidate.evidence_fingerprint,
        evidence: candidate.evidence,
        selection: candidate.selection,
        requirement_targets: candidate.requirement_targets,
    })
}

fn evidence_fingerprint(evidence: &GenericConformanceEvidence) -> ProductFingerprint {
    crate::db::fingerprint::generic_conformance_evidence_fingerprint(evidence)
}

pub fn selected_callable(selection: &RequirementSelectionTemplate) -> Option<CallableId> {
    match selection {
        RequirementSelectionTemplate::InherentCallable { callable, .. }
        | RequirementSelectionTemplate::ConformanceCallable { callable }
        | RequirementSelectionTemplate::TraitDefault { callable } => Some(callable.clone()),
        RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => Some(candidate.callable.clone()),
        RequirementSelectionTemplate::DataComponent { .. } => None,
    }
}

fn selection_convergence_key(selection: &RequirementSelectionTemplate, evidence_fingerprint: ProductFingerprint) -> TraitDispatchConvergenceKey {
    match selection {
        RequirementSelectionTemplate::InherentCallable {
            callable,
            conditional_impl: Some(impl_id),
            applicability: Some(applicability),
        } => TraitDispatchConvergenceKey::ConditionalInherent {
            callable: callable.clone(),
            impl_id: impl_id.clone(),
            applicability_fingerprint: inherent_specialization_fingerprint(applicability),
        },
        RequirementSelectionTemplate::InherentCallable { callable, .. } => TraitDispatchConvergenceKey::Inherent(callable.clone()),
        RequirementSelectionTemplate::ConformanceCallable { callable } => TraitDispatchConvergenceKey::ConformanceCallable(callable.clone()),
        RequirementSelectionTemplate::DataComponent { component, .. } => TraitDispatchConvergenceKey::DataComponent(component.clone()),
        RequirementSelectionTemplate::TraitDefault { callable } => TraitDispatchConvergenceKey::TraitDefault {
            callable: callable.clone(),
            evidence_fingerprint,
        },
        RequirementSelectionTemplate::ConditionalInherent { candidate, .. } => TraitDispatchConvergenceKey::ConditionalInherent {
            callable: candidate.callable.clone(),
            impl_id: candidate.conditional_impl.clone().expect("conditional selection has impl provenance"),
            applicability_fingerprint: candidate.applicability.as_ref().map(inherent_specialization_fingerprint).unwrap_or_default(),
        },
    }
}

pub(crate) fn inherent_specialization_fingerprint(applicability: &crate::impls::InherentImplSpecialization) -> ProductFingerprint {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    applicability.impl_id.hash(&mut hasher);
    applicability.receiver.hash(&mut hasher);
    applicability.owner_view.hash(&mut hasher);
    let mut bindings = applicability.bindings.iter().map(|(parameter, ty)| (*parameter, *ty)).collect::<Vec<_>>();
    bindings.sort();
    bindings.hash(&mut hasher);
    applicability.environment.self_binding.hash(&mut hasher);
    let mut environment_bindings = applicability
        .environment
        .bindings
        .iter()
        .map(|(parameter, ty)| (*parameter, *ty))
        .collect::<Vec<_>>();
    environment_bindings.sort();
    environment_bindings.hash(&mut hasher);
    ProductFingerprint::new(hasher.finish())
}

fn retain_terminal(slot: &mut Option<TraitDispatchResolution>, candidate: TraitDispatchResolution) {
    let replace = slot
        .as_ref()
        .is_none_or(|current| trait_dispatch_terminal_rank(&candidate) > trait_dispatch_terminal_rank(current));
    if replace {
        *slot = Some(candidate);
    }
}

/// Stable proof-state precedence for a bucket that produced no executable
/// trait candidate. A proven invalid/incomplete conformance must never be
/// hidden by an unrelated unresolved candidate, and control/solver terminal
/// states must remain stronger than ordinary lack of evidence.
fn trait_dispatch_terminal_rank(resolution: &TraitDispatchResolution) -> u8 {
    match resolution {
        TraitDispatchResolution::CoherenceConflict(_) => 110,
        TraitDispatchResolution::Incomplete(_) => 100,
        TraitDispatchResolution::InternalFailure(_) => 90,
        TraitDispatchResolution::Cancelled => 80,
        TraitDispatchResolution::BudgetExceeded(_) => 70,
        TraitDispatchResolution::Blocked(_) => 60,
        TraitDispatchResolution::Dynamic(_) => 50,
        TraitDispatchResolution::Unknown(_) => 40,
        TraitDispatchResolution::Found(_) | TraitDispatchResolution::Ambiguous(_) | TraitDispatchResolution::Missing => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::{TraitDispatchResolution, retain_terminal};
    use crate::identity::{ImplId, ImplLocalId};
    use crate::types::evidence::UnknownReason;
    use phalcom_modules::identity::{ModuleId, ModulePath, ResolvedProjectId};

    fn impl_id(local: u32) -> ImplId {
        ImplId::new(ModuleId::resolved(ResolvedProjectId::from_raw(77), ModulePath::root()), ImplLocalId(local))
    }

    #[test]
    fn proven_incomplete_terminal_outranks_unknown_regardless_of_candidate_order() {
        let mut unknown_first = None;
        retain_terminal(&mut unknown_first, TraitDispatchResolution::Unknown(UnknownReason::NoTypeEvidence));
        retain_terminal(&mut unknown_first, TraitDispatchResolution::Incomplete(impl_id(1)));
        assert!(matches!(unknown_first, Some(TraitDispatchResolution::Incomplete(_))));

        let mut incomplete_first = None;
        retain_terminal(&mut incomplete_first, TraitDispatchResolution::Incomplete(impl_id(2)));
        retain_terminal(&mut incomplete_first, TraitDispatchResolution::Unknown(UnknownReason::NoTypeEvidence));
        assert!(matches!(incomplete_first, Some(TraitDispatchResolution::Incomplete(_))));
    }

    #[test]
    fn internal_failure_terminal_outranks_plain_unknown() {
        let mut terminal = None;
        retain_terminal(&mut terminal, TraitDispatchResolution::Unknown(UnknownReason::NoTypeEvidence));
        retain_terminal(&mut terminal, TraitDispatchResolution::InternalFailure("broken evidence product".into()));
        assert!(matches!(terminal, Some(TraitDispatchResolution::InternalFailure(_))));
    }
}
