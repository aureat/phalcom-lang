//! Canonical generic conformance evidence, lexical evidence environments,
//! deterministic evidence layout, and direct proof queries.

use crate::declarations::DeclarationTypeTable;
use crate::identity::ImplId;
use crate::impls::{ConformanceEvidence, ConformanceIndex, ConformanceResolution, ConformanceWitnessPlan};
use crate::traits::{TraitRef, TraitRequirementId, TraitSurfaceTable};
use crate::types::evidence::UnknownReason;
use crate::types::id::TypeId;
use crate::types::outcome::{
    BlockReason, BudgetReport, CancellationToken, DynamicBoundaryObligation, QueryBudget,
    RelationOutcome,
};
use crate::types::parameter::{GenericConstraint, TypeParameterOwner, TypeTerm};
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
use crate::types::substitution::TypeSubstitution;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Stable semantic identity for one direct generic conformance proposition.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GenericConformanceKey {
    pub subject: TypeTerm,
    pub trait_ref: TraitRef,
}

impl GenericConformanceKey {
    pub fn new(subject: TypeTerm, trait_ref: TraitRef) -> Self {
        Self { subject, trait_ref }
    }
}

/// Lexical assumption of conformance attached to a type parameter owner.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AssumedConformanceEvidence {
    pub key: GenericConformanceKey,
    pub owner: TypeParameterOwner,
}

/// Proven generic obligation evidence across constraint forms.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NestedGenericEvidence {
    Conformance(GenericConformanceEvidence),
    Equivalent,
    Subtype,
}

/// Origin of a derived conformance proof.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DerivedConformanceOrigin {
    ConditionalSource {
        exact_source: Arc<ConformanceEvidence>,
    },
    GuardedRule {
        rule: Arc<crate::derived_conformance::DerivedConformanceRule>,
        exact_subject: TypeId,
        exact_trait_ref: TraitRef,
    },
}

/// Derived conformance evidence for an instantiated conditional conformance or guarded rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DerivedConformanceEvidence {
    pub origin: DerivedConformanceOrigin,
    pub impl_bindings: TypeSubstitution,
    pub nested: Box<[NestedGenericEvidence]>,
}

impl DerivedConformanceEvidence {
    pub fn exact_source(&self) -> Option<&Arc<ConformanceEvidence>> {
        match &self.origin {
            DerivedConformanceOrigin::ConditionalSource { exact_source } => Some(exact_source),
            DerivedConformanceOrigin::GuardedRule { .. } => None,
        }
    }

    pub fn requirements(&self) -> &BTreeMap<crate::traits::TraitRequirementId, crate::impls::RequirementSelectionTemplate> {
        match &self.origin {
            DerivedConformanceOrigin::ConditionalSource { exact_source } => &exact_source.requirements,
            DerivedConformanceOrigin::GuardedRule { rule, .. } => &rule.requirements,
        }
    }

    pub fn requirement_views(&self) -> &BTreeMap<crate::traits::TraitRequirementId, crate::traits::InstantiatedTraitRequirement> {
        match &self.origin {
            DerivedConformanceOrigin::ConditionalSource { exact_source } => &exact_source.requirement_views,
            DerivedConformanceOrigin::GuardedRule { rule, .. } => &rule.requirement_views,
        }
    }
}

/// Proven generic conformance evidence: exact (unconditional source impl),
/// assumed (derived from a lexical generic bound), or derived (instantiated conditional conformance).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericConformanceEvidence {
    Exact(Arc<ConformanceEvidence>),
    Assumed(AssumedConformanceEvidence),
    Derived(Arc<DerivedConformanceEvidence>),
}

/// Status of an active or completed proof goal in the recursive memo table.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProofMemoState {
    Visiting,
    Completed(GenericConformanceProof),
}

/// Typed generic obligation proof result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericObligationProof {
    Conformance(GenericConformanceProof),
    Equivalent(GenericEquivalenceProof),
    Subtype(RelationOutcome<()>),
}

impl GenericObligationProof {
    pub fn is_proven(&self) -> bool {
        match self {
            Self::Conformance(p) => p.is_proven(),
            Self::Equivalent(p) => p.is_proven(),
            Self::Subtype(outcome) => outcome.is_proven(),
        }
    }
}

/// Typed direct conformance proof result preserving rich terminal outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericConformanceProof {
    Proven(GenericConformanceEvidence),
    Missing,
    Ambiguous(Box<[GenericConformanceEvidence]>),
    Incomplete(ImplId),
    CoherenceConflict(Box<[ImplId]>),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

impl GenericConformanceProof {
    pub fn is_proven(&self) -> bool {
        matches!(self, Self::Proven(_))
    }
}

/// Typed generic equivalence proof result preserving rich terminal outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericEquivalenceProof {
    Proven,
    NotProven,
    Ambiguous,
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Recursive,
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}

impl GenericEquivalenceProof {
    pub fn is_proven(&self) -> bool {
        matches!(self, Self::Proven)
    }
}

/// Complete semantic identity for one generic evidence slot.
/// Both the conformance subject/trait and the requirement identity determine the slot.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GenericEvidenceSlotKey {
    pub conformance: GenericConformanceKey,
    pub requirement: TraitRequirementId,
}

impl GenericEvidenceSlotKey {
    pub fn new(conformance: GenericConformanceKey, requirement: TraitRequirementId) -> Self {
        Self { conformance, requirement }
    }
}

/// Error returned when evidence layout slot capacity is exceeded.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericEvidenceLayoutError {
    SlotCapacityExceeded(usize),
}

/// Deterministic executable evidence slot layout.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GenericEvidenceLayout {
    pub slots: Box<[(GenericEvidenceSlotKey, u16)]>,
    pub slot_map: BTreeMap<GenericEvidenceSlotKey, u16>,
}

impl GenericEvidenceLayout {
    pub fn empty() -> Self {
        Self {
            slots: vec![].into_boxed_slice(),
            slot_map: BTreeMap::new(),
        }
    }

    pub fn new(slots: Box<[(GenericEvidenceSlotKey, u16)]>) -> Self {
        let slot_map = slots.iter().cloned().collect();
        Self { slots, slot_map }
    }

    /// Builds a deterministic evidence layout from direct conformance assumptions
    /// and canonical trait surfaces.
    pub fn build(
        conformances: &BTreeMap<GenericConformanceKey, AssumedConformanceEvidence>,
        trait_surfaces: &TraitSurfaceTable,
    ) -> Result<Self, GenericEvidenceLayoutError> {
        let mut slot_keys = BTreeSet::new();
        for (key, _) in conformances {
            if let Some(surface) = trait_surfaces.get(&key.trait_ref.declaration) {
                for req_id in surface.members.keys() {
                    slot_keys.insert(GenericEvidenceSlotKey::new(key.clone(), req_id.clone()));
                }
            }
        }

        let mut slots = Vec::with_capacity(slot_keys.len());
        let mut slot_map = BTreeMap::new();
        for (index, slot_key) in slot_keys.into_iter().enumerate() {
            if index > u16::MAX as usize {
                return Err(GenericEvidenceLayoutError::SlotCapacityExceeded(index));
            }
            let slot_idx = index as u16;
            slots.push((slot_key.clone(), slot_idx));
            slot_map.insert(slot_key, slot_idx);
        }

        Ok(Self {
            slots: slots.into_boxed_slice(),
            slot_map,
        })
    }

    pub fn slot_for(&self, key: &GenericEvidenceSlotKey) -> Option<u16> {
        self.slot_map.get(key).copied()
    }

    pub fn len(&self) -> usize {
        self.slots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.slots.is_empty()
    }
}

/// Lexical, compositional generic evidence environment for a checking scope.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct GenericEvidenceEnvironment {
    constraints: Box<[GenericConstraint]>,
    conformances: BTreeMap<GenericConformanceKey, AssumedConformanceEvidence>,
    equivalences: BTreeMap<TypeTerm, BTreeSet<TypeTerm>>,
    refinements: BTreeMap<TypeTerm, TypeTerm>,
    layout: GenericEvidenceLayout,
}

fn build_equivalences_and_refinements(
    constraints: &[GenericConstraint],
) -> (BTreeMap<TypeTerm, BTreeSet<TypeTerm>>, BTreeMap<TypeTerm, TypeTerm>) {
    let mut adjacency: BTreeMap<TypeTerm, BTreeSet<TypeTerm>> = BTreeMap::new();
    for c in constraints {
        if let GenericConstraint::Equivalent { left, right } = c {
            adjacency.entry(left.clone()).or_default().insert(right.clone());
            adjacency.entry(right.clone()).or_default().insert(left.clone());
        }
    }

    let mut visited = BTreeSet::new();
    let mut components: Vec<BTreeSet<TypeTerm>> = Vec::new();
    for node in adjacency.keys() {
        if !visited.contains(node) {
            let mut comp = BTreeSet::new();
            let mut queue = vec![node.clone()];
            visited.insert(node.clone());
            comp.insert(node.clone());
            while let Some(curr) = queue.pop() {
                if let Some(neighbors) = adjacency.get(&curr) {
                    for n in neighbors {
                        if visited.insert(n.clone()) {
                            comp.insert(n.clone());
                            queue.push(n.clone());
                        }
                    }
                }
            }
            components.push(comp);
        }
    }

    let mut equivalences: BTreeMap<TypeTerm, BTreeSet<TypeTerm>> = BTreeMap::new();
    let mut refinements: BTreeMap<TypeTerm, TypeTerm> = BTreeMap::new();

    for comp in components {
        for term in &comp {
            equivalences.insert(term.clone(), comp.clone());
            let others: Vec<&TypeTerm> = comp.iter().filter(|t| *t != term).collect();
            if others.len() == 1 {
                refinements.insert(term.clone(), (*others[0]).clone());
            }
        }
    }

    (equivalences, refinements)
}

impl GenericEvidenceEnvironment {
    pub fn empty() -> Self {
        Self {
            constraints: vec![].into_boxed_slice(),
            conformances: BTreeMap::new(),
            equivalences: BTreeMap::new(),
            refinements: BTreeMap::new(),
            layout: GenericEvidenceLayout::empty(),
        }
    }

    pub fn new(
        constraints: Box<[GenericConstraint]>,
        conformances: BTreeMap<GenericConformanceKey, AssumedConformanceEvidence>,
        layout: GenericEvidenceLayout,
    ) -> Self {
        let (equivalences, refinements) = build_equivalences_and_refinements(&constraints);
        Self {
            constraints,
            conformances,
            equivalences,
            refinements,
            layout,
        }
    }

    /// Builds a generic evidence environment from a slice of constraints and owner,
    /// returning an error if evidence layout capacity is exceeded.
    pub fn try_from_constraints(
        constraints: &[GenericConstraint],
        owner: TypeParameterOwner,
        trait_surfaces: &TraitSurfaceTable,
    ) -> Result<Self, GenericEvidenceLayoutError> {
        let mut conformances = BTreeMap::new();
        for constraint in constraints {
            if let GenericConstraint::Conforms { subject, trait_ref } = constraint {
                let key = GenericConformanceKey::new(subject.clone(), trait_ref.clone());
                conformances.entry(key.clone()).or_insert_with(|| AssumedConformanceEvidence {
                    key,
                    owner: owner.clone(),
                });
            }
        }
        let (equivalences, refinements) = build_equivalences_and_refinements(constraints);
        let layout = GenericEvidenceLayout::build(&conformances, trait_surfaces)?;
        Ok(Self {
            constraints: constraints.to_vec().into_boxed_slice(),
            conformances,
            equivalences,
            refinements,
            layout,
        })
    }

    /// Composes two evidence environments, returning an error if combined layout capacity is exceeded.
    pub fn try_compose(&self, other: &Self, trait_surfaces: &TraitSurfaceTable) -> Result<Self, GenericEvidenceLayoutError> {
        let mut combined_constraints = self.constraints.to_vec();
        for c in other.constraints.iter() {
            if !combined_constraints.contains(c) {
                combined_constraints.push(c.clone());
            }
        }
        combined_constraints.sort();
        combined_constraints.dedup();

        let mut conformances = self.conformances.clone();
        for (key, evidence) in &other.conformances {
            conformances.entry(key.clone()).or_insert_with(|| evidence.clone());
        }

        let (equivalences, refinements) = build_equivalences_and_refinements(&combined_constraints);
        let layout = GenericEvidenceLayout::build(&conformances, trait_surfaces)?;
        Ok(Self {
            constraints: combined_constraints.into_boxed_slice(),
            conformances,
            equivalences,
            refinements,
            layout,
        })
    }

    pub fn constraints(&self) -> &[GenericConstraint] {
        &self.constraints
    }

    pub fn conformances(&self) -> &BTreeMap<GenericConformanceKey, AssumedConformanceEvidence> {
        &self.conformances
    }

    pub fn layout(&self) -> &GenericEvidenceLayout {
        &self.layout
    }

    pub fn assumed_conformance(&self, key: &GenericConformanceKey) -> Option<&AssumedConformanceEvidence> {
        self.conformances.get(key)
    }

    pub fn slot_for(&self, key: &GenericEvidenceSlotKey) -> Option<u16> {
        self.layout.slot_for(key)
    }

    /// Are two terms provably equivalent under lexical facts?
    pub fn are_equivalent(&self, left: &TypeTerm, right: &TypeTerm) -> GenericEquivalenceProof {
        if left == right {
            return GenericEquivalenceProof::Proven;
        }
        if let Some(equivs) = self.equivalences.get(left) {
            if equivs.contains(right) {
                return GenericEquivalenceProof::Proven;
            }
        }
        GenericEquivalenceProof::NotProven
    }

    /// Returns all terms provably equivalent to `start` in the lexical environment.
    pub fn equivalent_terms(&self, start: &TypeTerm) -> BTreeSet<TypeTerm> {
        self.equivalences.get(start).cloned().unwrap_or_default()
    }

    /// Does projection P have a unique lexical refinement?
    pub fn unique_refinement(&self, projection: &TypeTerm) -> Option<&TypeTerm> {
        self.refinements.get(projection)
    }
}

/// Recursive proof session context owning memoization, cycle tracking,
/// query budget, cancellation, and semantic authorities.
pub struct ConformanceProofContext<'a> {
    pub lexical_evidence: Option<&'a GenericEvidenceEnvironment>,
    pub conformance_index: Option<&'a ConformanceIndex>,
    pub witness_plans: Option<&'a BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>>,
    pub trait_surfaces: Option<&'a TraitSurfaceTable>,
    pub derived_rules: Option<&'a crate::derived_conformance::DerivedConformanceRuleIndex>,
    pub declarations: Option<&'a DeclarationTypeTable>,
    pub hierarchy: Option<&'a dyn TypeHierarchy>,
    pub store: Option<&'a mut TypeStore>,
    pub memo: BTreeMap<GenericConformanceKey, ProofMemoState>,
    pub stack: Vec<GenericConformanceKey>,
    pub budget: QueryBudget,
    pub cancellation: CancellationToken,
}

impl<'a> ConformanceProofContext<'a> {
    pub fn new(
        lexical_evidence: Option<&'a GenericEvidenceEnvironment>,
        conformance_index: Option<&'a ConformanceIndex>,
        witness_plans: Option<&'a BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>>,
        trait_surfaces: Option<&'a TraitSurfaceTable>,
        declarations: Option<&'a DeclarationTypeTable>,
        hierarchy: Option<&'a dyn TypeHierarchy>,
        store: Option<&'a mut TypeStore>,
    ) -> Self {
        Self {
            lexical_evidence,
            conformance_index,
            witness_plans,
            trait_surfaces,
            derived_rules: None,
            declarations,
            hierarchy,
            store,
            memo: BTreeMap::new(),
            stack: Vec::new(),
            budget: QueryBudget::default(),
            cancellation: CancellationToken::new(),
        }
    }

    pub fn with_derived_rules(
        mut self,
        derived_rules: Option<&'a crate::derived_conformance::DerivedConformanceRuleIndex>,
    ) -> Self {
        self.derived_rules = derived_rules;
        self
    }

    pub fn with_budget_and_cancellation(
        mut self,
        budget: QueryBudget,
        cancellation: CancellationToken,
    ) -> Self {
        self.budget = budget;
        self.cancellation = cancellation;
        self
    }
}

pub fn substitute_type_term(term: &TypeTerm, subst: &TypeSubstitution, store: &mut TypeStore) -> TypeTerm {
    match term {
        TypeTerm::Canonical(ty) => TypeTerm::Canonical(subst.apply(store, *ty)),
        TypeTerm::SelfType(s) => TypeTerm::SelfType(s.clone()),
        TypeTerm::Infer(i) => TypeTerm::Infer(*i),
    }
}

pub fn substitute_trait_ref(trait_ref: &TraitRef, subst: &TypeSubstitution, store: &mut TypeStore) -> TraitRef {
    let args: Vec<TypeId> = trait_ref.arguments.iter().map(|&arg| subst.apply(store, arg)).collect();
    TraitRef::new(trait_ref.declaration.clone(), args)
}

pub fn substitute_generic_constraint(
    constraint: &GenericConstraint,
    subst: &TypeSubstitution,
    store: &mut TypeStore,
) -> GenericConstraint {
    match constraint {
        GenericConstraint::Conforms { subject, trait_ref } => GenericConstraint::Conforms {
            subject: substitute_type_term(subject, subst, store),
            trait_ref: substitute_trait_ref(trait_ref, subst, store),
        },
        GenericConstraint::Equivalent { left, right } => GenericConstraint::Equivalent {
            left: substitute_type_term(left, subst, store),
            right: substitute_type_term(right, subst, store),
        },
        GenericConstraint::Subtype { lower, upper } => GenericConstraint::Subtype {
            lower: substitute_type_term(lower, subst, store),
            upper: substitute_type_term(upper, subst, store),
        },
    }
}

fn normalize_term_in_context(
    term: &TypeTerm,
    context: &mut ConformanceProofContext,
) -> Result<TypeTerm, GenericEquivalenceProof> {
    match term {
        TypeTerm::Canonical(ty) => {
            let ty = *ty;
            let Some(surfaces) = context.trait_surfaces else {
                return Ok(TypeTerm::Canonical(ty));
            };
            let Some(decls) = context.declarations else {
                return Ok(TypeTerm::Canonical(ty));
            };
            let Some(hier) = context.hierarchy else {
                return Ok(TypeTerm::Canonical(ty));
            };
            let mode = if let Some(env) = context.lexical_evidence {
                crate::types::ProjectionNormalizationMode::Generic {
                    evidence: env,
                    conformance_index: context.conformance_index,
                    witness_plans: context.witness_plans,
                    trait_surfaces: surfaces,
                    derived_rules: context.derived_rules,
                    declarations: decls,
                    hierarchy: hier,
                }
            } else if let (Some(index), Some(plans)) = (context.conformance_index, context.witness_plans) {
                crate::types::ProjectionNormalizationMode::Exact {
                    conformance_index: index,
                    witness_plans: plans,
                    trait_surfaces: surfaces,
                    derived_rules: context.derived_rules,
                    declarations: decls,
                    hierarchy: hier,
                }
            } else {
                return Ok(TypeTerm::Canonical(ty));
            };

            let Some(store) = context.store.as_deref_mut() else {
                return Ok(TypeTerm::Canonical(ty));
            };

            let mut norm_ctx = crate::types::ProjectionNormalizationContext::new(
                mode,
                &mut context.budget,
                &context.cancellation,
            );

            match crate::types::normalize_type(store, ty, &mut norm_ctx) {
                crate::types::ProjectionNormalizationResult::Normalized(norm_ty)
                | crate::types::ProjectionNormalizationResult::Symbolic(norm_ty) => {
                    Ok(TypeTerm::Canonical(norm_ty))
                }
                crate::types::ProjectionNormalizationResult::Recursive => {
                    Err(GenericEquivalenceProof::Recursive)
                }
                crate::types::ProjectionNormalizationResult::Blocked(reason) => {
                    Err(GenericEquivalenceProof::Blocked(reason))
                }
                crate::types::ProjectionNormalizationResult::Dynamic(obligation) => {
                    Err(GenericEquivalenceProof::Dynamic(obligation))
                }
                crate::types::ProjectionNormalizationResult::Cancelled => {
                    Err(GenericEquivalenceProof::Cancelled)
                }
                crate::types::ProjectionNormalizationResult::BudgetExceeded(report) => {
                    Err(GenericEquivalenceProof::BudgetExceeded(report))
                }
                crate::types::ProjectionNormalizationResult::InternalFailure(msg) => {
                    Err(GenericEquivalenceProof::InternalFailure(msg))
                }
                crate::types::ProjectionNormalizationResult::Incomplete
                | crate::types::ProjectionNormalizationResult::Unknown(_)
                | crate::types::ProjectionNormalizationResult::Ambiguous(_) => {
                    Ok(TypeTerm::Canonical(ty))
                }
            }
        }
        other => Ok(other.clone()),
    }
}

/// Evaluates a generic obligation constraint under the current proof session.
pub fn prove_constraint(
    constraint: &GenericConstraint,
    substitution: &TypeSubstitution,
    context: &mut ConformanceProofContext,
) -> GenericObligationProof {
    let subst_constraint = if let Some(store) = context.store.as_deref_mut() {
        substitute_generic_constraint(constraint, substitution, store)
    } else {
        constraint.clone()
    };

    match subst_constraint {
        GenericConstraint::Conforms { subject, trait_ref } => {
            let normalized_subject = match normalize_term_in_context(&subject, context) {
                Ok(term) => term,
                Err(proof) => return GenericObligationProof::Conformance(map_equivalence_proof_to_conformance_proof(proof)),
            };
            let key = GenericConformanceKey::new(normalized_subject, trait_ref);
            let proof = prove_conformance_with_context(&key, context);
            GenericObligationProof::Conformance(proof)
        }
        GenericConstraint::Equivalent { left, right } => {
            let left = match normalize_term_in_context(&left, context) {
                Ok(term) => term,
                Err(proof) => return GenericObligationProof::Equivalent(proof),
            };
            let right = match normalize_term_in_context(&right, context) {
                Ok(term) => term,
                Err(proof) => return GenericObligationProof::Equivalent(proof),
            };

            if left == right {
                return GenericObligationProof::Equivalent(GenericEquivalenceProof::Proven);
            }
            if let Some(env) = context.lexical_evidence {
                let proof = env.are_equivalent(&left, &right);
                if proof.is_proven() {
                    return GenericObligationProof::Equivalent(proof);
                }
                if let (Some(surfaces), Some(store)) = (context.trait_surfaces, context.store.as_deref_mut()) {
                    for (assumed_key, _) in env.conformances() {
                        if let Some(surface) = surfaces.get(&assumed_key.trait_ref.declaration) {
                            if let Some(signature) = &surface.generic_signature {
                                let contract_constraints = crate::traits::instantiate_trait_generic_contract_for_term(
                                    store,
                                    signature,
                                    &assumed_key.subject,
                                    &assumed_key.trait_ref,
                                );
                                for constraint in contract_constraints {
                                    if let GenericConstraint::Equivalent { left: cl, right: cr } = constraint {
                                        if (cl == left && cr == right) || (cl == right && cr == left) {
                                            return GenericObligationProof::Equivalent(GenericEquivalenceProof::Proven);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            GenericObligationProof::Equivalent(GenericEquivalenceProof::NotProven)
        }
        GenericConstraint::Subtype { lower, upper } => {
            let lower = match normalize_term_in_context(&lower, context) {
                Ok(term) => term,
                Err(GenericEquivalenceProof::Recursive) => {
                    return GenericObligationProof::Subtype(RelationOutcome::Blocked(BlockReason::RecursiveFixpoint));
                }
                Err(GenericEquivalenceProof::Blocked(reason)) => {
                    return GenericObligationProof::Subtype(RelationOutcome::Blocked(reason));
                }
                Err(GenericEquivalenceProof::Dynamic(obligation)) => {
                    return GenericObligationProof::Subtype(RelationOutcome::DynamicBoundary(obligation));
                }
                Err(GenericEquivalenceProof::Cancelled) => {
                    return GenericObligationProof::Subtype(RelationOutcome::Cancelled);
                }
                Err(GenericEquivalenceProof::BudgetExceeded(report)) => {
                    return GenericObligationProof::Subtype(RelationOutcome::BudgetExceeded(report));
                }
                Err(GenericEquivalenceProof::InternalFailure(msg)) => {
                    return GenericObligationProof::Subtype(RelationOutcome::InternalFailure(msg.to_string()));
                }
                Err(_) => {
                    return GenericObligationProof::Subtype(RelationOutcome::Refuted(
                        crate::types::outcome::RelationFailure::Custom("equivalence failed during subtype normalization".into())
                    ));
                }
            };
            let upper = match normalize_term_in_context(&upper, context) {
                Ok(term) => term,
                Err(GenericEquivalenceProof::Recursive) => {
                    return GenericObligationProof::Subtype(RelationOutcome::Blocked(BlockReason::RecursiveFixpoint));
                }
                Err(GenericEquivalenceProof::Blocked(reason)) => {
                    return GenericObligationProof::Subtype(RelationOutcome::Blocked(reason));
                }
                Err(GenericEquivalenceProof::Dynamic(obligation)) => {
                    return GenericObligationProof::Subtype(RelationOutcome::DynamicBoundary(obligation));
                }
                Err(GenericEquivalenceProof::Cancelled) => {
                    return GenericObligationProof::Subtype(RelationOutcome::Cancelled);
                }
                Err(GenericEquivalenceProof::BudgetExceeded(report)) => {
                    return GenericObligationProof::Subtype(RelationOutcome::BudgetExceeded(report));
                }
                Err(GenericEquivalenceProof::InternalFailure(msg)) => {
                    return GenericObligationProof::Subtype(RelationOutcome::InternalFailure(msg.to_string()));
                }
                Err(_) => {
                    return GenericObligationProof::Subtype(RelationOutcome::Refuted(
                        crate::types::outcome::RelationFailure::Custom("equivalence failed during subtype normalization".into())
                    ));
                }
            };

            if lower == upper {
                return GenericObligationProof::Subtype(RelationOutcome::proven(()));
            }
            match (&lower, &upper) {
                (TypeTerm::Canonical(sub_ty), TypeTerm::Canonical(sup_ty)) => {
                    let sub_ty = *sub_ty;
                    let sup_ty = *sup_ty;
                    if let (Some(hierarchy), Some(store)) = (context.hierarchy, context.store.as_deref_mut()) {
                        let outcome = crate::types::relation::check_subtype_bounded(
                            store,
                            hierarchy,
                            sub_ty,
                            sup_ty,
                            &mut context.budget,
                            &context.cancellation,
                        );
                        GenericObligationProof::Subtype(outcome)
                    } else {
                        GenericObligationProof::Subtype(RelationOutcome::InternalFailure(
                            "hierarchy or store is unavailable".into(),
                        ))
                    }
                }
                _ => GenericObligationProof::Subtype(RelationOutcome::Refuted(
                    crate::types::outcome::RelationFailure::Custom(
                        "non-canonical terms cannot be checked for subtyping".into(),
                    ),
                )),
            }
        }
    }
}

fn map_conformance_resolution_to_proof(res: ConformanceResolution) -> GenericConformanceProof {
    match res {
        ConformanceResolution::Proven(ev) => GenericConformanceProof::Proven(GenericConformanceEvidence::Exact(ev)),
        ConformanceResolution::NotDeclared => GenericConformanceProof::Missing,
        ConformanceResolution::InvalidSource(impl_id) | ConformanceResolution::Incomplete(impl_id) => {
            GenericConformanceProof::Incomplete(impl_id)
        }
        ConformanceResolution::CoherenceConflict(impls) => GenericConformanceProof::CoherenceConflict(impls),
        ConformanceResolution::Unknown(reason) => GenericConformanceProof::Unknown(reason),
        ConformanceResolution::Blocked(reason) => GenericConformanceProof::Blocked(reason),
        ConformanceResolution::Dynamic(obligation) => GenericConformanceProof::Dynamic(obligation),
        ConformanceResolution::Cancelled => GenericConformanceProof::Cancelled,
        ConformanceResolution::BudgetExceeded(report) => GenericConformanceProof::BudgetExceeded(report),
        ConformanceResolution::InternalFailure(msg) => GenericConformanceProof::InternalFailure(msg),
    }
}

fn map_equivalence_proof_to_conformance_proof(proof: GenericEquivalenceProof) -> GenericConformanceProof {
    match proof {
        GenericEquivalenceProof::Proven => unreachable!(),
        GenericEquivalenceProof::NotProven | GenericEquivalenceProof::Ambiguous => GenericConformanceProof::Missing,
        GenericEquivalenceProof::Blocked(reason) => GenericConformanceProof::Blocked(reason),
        GenericEquivalenceProof::Dynamic(obligation) => GenericConformanceProof::Dynamic(obligation),
        GenericEquivalenceProof::Recursive => GenericConformanceProof::Blocked(BlockReason::RecursiveFixpoint),
        GenericEquivalenceProof::Cancelled => GenericConformanceProof::Cancelled,
        GenericEquivalenceProof::BudgetExceeded(report) => GenericConformanceProof::BudgetExceeded(report),
        GenericEquivalenceProof::InternalFailure(msg) => GenericConformanceProof::InternalFailure(msg),
    }
}

fn map_subtype_outcome_to_conformance_proof(outcome: RelationOutcome<()>) -> GenericConformanceProof {
    match outcome {
        RelationOutcome::Proven { .. } => unreachable!(),
        RelationOutcome::Refuted(_) => GenericConformanceProof::Missing,
        RelationOutcome::DynamicBoundary(obligation) => GenericConformanceProof::Dynamic(obligation),
        RelationOutcome::Blocked(reason) => GenericConformanceProof::Blocked(reason),
        RelationOutcome::Cancelled => GenericConformanceProof::Cancelled,
        RelationOutcome::BudgetExceeded(report) => GenericConformanceProof::BudgetExceeded(report),
        RelationOutcome::InternalFailure(msg) => GenericConformanceProof::InternalFailure(msg.into_boxed_str()),
    }
}

fn evaluate_conformance_goal(
    key: &GenericConformanceKey,
    context: &mut ConformanceProofContext,
) -> GenericConformanceProof {
    // 1. Check lexical assumptions in the environment
    if let Some(env) = context.lexical_evidence {
        if let Some(assumed) = env.assumed_conformance(key) {
            return GenericConformanceProof::Proven(GenericConformanceEvidence::Assumed(assumed.clone()));
        }
        for equiv_subject in env.equivalent_terms(&key.subject) {
            let equiv_key = GenericConformanceKey::new(equiv_subject, key.trait_ref.clone());
            if let Some(assumed) = env.assumed_conformance(&equiv_key) {
                return GenericConformanceProof::Proven(GenericConformanceEvidence::Assumed(assumed.clone()));
            }
        }
        if let (Some(surfaces), Some(store)) = (context.trait_surfaces, context.store.as_deref_mut()) {
            for (assumed_key, assumed_evidence) in env.conformances() {
                if let Some(surface) = surfaces.get(&assumed_key.trait_ref.declaration) {
                    if let Some(signature) = &surface.generic_signature {
                        let contract_constraints = crate::traits::instantiate_trait_generic_contract_for_term(
                            store,
                            signature,
                            &assumed_key.subject,
                            &assumed_key.trait_ref,
                        );
                        for constraint in contract_constraints {
                            if let GenericConstraint::Conforms { subject, trait_ref } = constraint {
                                if trait_ref == key.trait_ref {
                                    if subject == key.subject || env.are_equivalent(&subject, &key.subject).is_proven() {
                                        return GenericConformanceProof::Proven(GenericConformanceEvidence::Assumed(assumed_evidence.clone()));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Candidate collection across exact/conditional nominal sources and guarded derived rules
    let mut successes = Vec::new();
    let mut failure_proofs = Vec::new();
    let mut conflicting_impls = Vec::new();

    // 2A. Nominal conformance index lookup (exact / conditional)
    if let TypeTerm::Canonical(target_type) = key.subject {
        if let (Some(index), Some(plans), Some(surfaces), Some(decls), Some(hier)) = (
            context.conformance_index,
            context.witness_plans,
            context.trait_surfaces,
            context.declarations,
            context.hierarchy,
        ) {
            if surfaces.contains(&key.trait_ref.declaration) {
                if let Some(store) = context.store.as_deref_mut() {
                    let matches = index.query_exact(store, target_type, &key.trait_ref);
                    match matches.len() {
                        0 => {}
                        1 => {
                            let head = &matches[0];
                            if let Some(plan) = plans.get(&head.impl_id) {
                                if !matches!(&plan.completeness, crate::impls::ConformanceCompleteness::Incomplete { .. }) {
                                    let generic_signature = plan.generic_signature.as_ref();
                                    let constraints = generic_signature.map_or(&[] as &[GenericConstraint], |s| &s.constraints);

                                    if constraints.is_empty() {
                                        let store = context.store.as_deref_mut().unwrap();
                                        let exact_res = crate::impls::resolve_conformance_evidence_with_surfaces(
                                            index,
                                            plans,
                                            surfaces,
                                            decls,
                                            store,
                                            hier,
                                            target_type,
                                            &key.trait_ref,
                                        );
                                        match exact_res {
                                            crate::impls::ConformanceResolution::Proven(exact_ev) => {
                                                conflicting_impls.push(head.impl_id.clone());
                                                successes.push(GenericConformanceEvidence::Exact(exact_ev));
                                            }
                                            other => {
                                                failure_proofs.push(map_conformance_resolution_to_proof(other));
                                            }
                                        }
                                    } else {
                                        let impl_bindings = TypeSubstitution::from_bindings(head.impl_bindings.clone());
                                        let constraints_vec = constraints.to_vec();
                                        let mut nested_evidences = Vec::with_capacity(constraints_vec.len());
                                        let mut failure_proof = None;

                                        for constraint in &constraints_vec {
                                            let obligation_proof = prove_constraint(constraint, &impl_bindings, context);
                                            match obligation_proof {
                                                GenericObligationProof::Conformance(GenericConformanceProof::Proven(ev)) => {
                                                    nested_evidences.push(NestedGenericEvidence::Conformance(ev));
                                                }
                                                GenericObligationProof::Conformance(failing) => {
                                                    failure_proof = Some(failing);
                                                    break;
                                                }
                                                GenericObligationProof::Equivalent(GenericEquivalenceProof::Proven) => {
                                                    nested_evidences.push(NestedGenericEvidence::Equivalent);
                                                }
                                                GenericObligationProof::Equivalent(failing) => {
                                                    failure_proof = Some(map_equivalence_proof_to_conformance_proof(failing));
                                                    break;
                                                }
                                                GenericObligationProof::Subtype(outcome) => {
                                                    if outcome.is_proven() {
                                                        nested_evidences.push(NestedGenericEvidence::Subtype);
                                                    } else {
                                                        failure_proof = Some(map_subtype_outcome_to_conformance_proof(outcome));
                                                        break;
                                                    }
                                                }
                                            }
                                        }

                                        if let Some(failing) = failure_proof {
                                            failure_proofs.push(failing);
                                        } else {
                                            let (index, plans, surfaces, decls, hier) = (
                                                context.conformance_index.unwrap(),
                                                context.witness_plans.unwrap(),
                                                context.trait_surfaces.unwrap(),
                                                context.declarations.unwrap(),
                                                context.hierarchy.unwrap(),
                                            );
                                            let store = context.store.as_deref_mut().unwrap();
                                            let exact_res = crate::impls::resolve_conformance_evidence_with_surfaces(
                                                index,
                                                plans,
                                                surfaces,
                                                decls,
                                                store,
                                                hier,
                                                target_type,
                                                &key.trait_ref,
                                            );
                                            match exact_res {
                                                crate::impls::ConformanceResolution::Proven(exact_ev) => {
                                                    conflicting_impls.push(head.impl_id.clone());
                                                    let derived = Arc::new(DerivedConformanceEvidence {
                                                        origin: DerivedConformanceOrigin::ConditionalSource { exact_source: exact_ev },
                                                        impl_bindings,
                                                        nested: nested_evidences.into_boxed_slice(),
                                                    });
                                                    successes.push(GenericConformanceEvidence::Derived(derived));
                                                }
                                                other => failure_proofs.push(map_conformance_resolution_to_proof(other)),
                                            }
                                        }
                                    }
                                } else {
                                    failure_proofs.push(GenericConformanceProof::Incomplete(head.impl_id.clone()));
                                }
                            } else {
                                failure_proofs.push(GenericConformanceProof::Incomplete(head.impl_id.clone()));
                            }
                        }
                        _ => {
                            return GenericConformanceProof::CoherenceConflict(
                                matches.into_iter().map(|item| item.impl_id).collect::<Vec<_>>().into_boxed_slice(),
                            );
                        }
                    }
                }
            }
        }
    }

    // 2B. Guarded derived conformance rules lookup
    if let (Some(rules), Some(surfaces)) = (context.derived_rules, context.trait_surfaces) {
        if surfaces.contains(&key.trait_ref.declaration) {
            let candidate_rules = rules.rules_for_trait(&key.trait_ref.declaration);
            for rule in candidate_rules {
                if let TypeTerm::Canonical(target_type) = key.subject {
                    let mut impl_bindings = TypeSubstitution::new();
                    impl_bindings.bind(rule.target_parameter, target_type);

                    let instantiated_trait_ref = if let Some(store) = context.store.as_deref_mut() {
                        substitute_trait_ref(&rule.trait_ref, &impl_bindings, store)
                    } else {
                        rule.trait_ref.clone()
                    };

                    if instantiated_trait_ref == key.trait_ref {
                        let constraints_vec = rule.generic_signature.constraints.to_vec();
                        let mut nested_evidences = Vec::with_capacity(constraints_vec.len());
                        let mut failure_proof = None;

                        for constraint in &constraints_vec {
                            let obligation_proof = prove_constraint(constraint, &impl_bindings, context);
                            match obligation_proof {
                                GenericObligationProof::Conformance(GenericConformanceProof::Proven(ev)) => {
                                    nested_evidences.push(NestedGenericEvidence::Conformance(ev));
                                }
                                GenericObligationProof::Conformance(failing) => {
                                    failure_proof = Some(failing);
                                    break;
                                }
                                GenericObligationProof::Equivalent(GenericEquivalenceProof::Proven) => {
                                    nested_evidences.push(NestedGenericEvidence::Equivalent);
                                }
                                GenericObligationProof::Equivalent(failing) => {
                                    failure_proof = Some(map_equivalence_proof_to_conformance_proof(failing));
                                    break;
                                }
                                GenericObligationProof::Subtype(outcome) => {
                                    if outcome.is_proven() {
                                        nested_evidences.push(NestedGenericEvidence::Subtype);
                                    } else {
                                        failure_proof = Some(map_subtype_outcome_to_conformance_proof(outcome));
                                        break;
                                    }
                                }
                            }
                        }

                        // The rule's antecedent evidence is the executable
                        // evidence product. Validate the consequent trait
                        // contract in the same proof session, but do not add
                        // its implied constraints to `nested`: the Iterator
                        // proof is already present and must be reused rather
                        // than duplicated in derived evidence.
                        if failure_proof.is_none() {
                            let contract_constraints = surfaces
                                .get(&key.trait_ref.declaration)
                                .and_then(|trait_surface| trait_surface.generic_signature.as_ref())
                                .and_then(|signature| {
                                    context.store.as_deref_mut().map(|store| {
                                        crate::traits::instantiate_trait_generic_contract_for_term(
                                            store,
                                            signature,
                                            &key.subject,
                                            &key.trait_ref,
                                        )
                                    })
                                })
                                .unwrap_or_default();
                            for constraint in &contract_constraints {
                                let obligation_proof = prove_constraint(constraint, &impl_bindings, context);
                                match obligation_proof {
                                    GenericObligationProof::Conformance(GenericConformanceProof::Proven(_))
                                    | GenericObligationProof::Equivalent(GenericEquivalenceProof::Proven) => {}
                                    GenericObligationProof::Conformance(failing) => {
                                        failure_proof = Some(failing);
                                        break;
                                    }
                                    GenericObligationProof::Equivalent(failing) => {
                                        failure_proof = Some(map_equivalence_proof_to_conformance_proof(failing));
                                        break;
                                    }
                                    GenericObligationProof::Subtype(outcome) => {
                                        if !outcome.is_proven() {
                                            failure_proof = Some(map_subtype_outcome_to_conformance_proof(outcome));
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(failing) = failure_proof {
                            failure_proofs.push(failing);
                        } else {
                            conflicting_impls.push(rule.impl_id.clone());
                            let derived = Arc::new(DerivedConformanceEvidence {
                                origin: DerivedConformanceOrigin::GuardedRule {
                                    rule: rule.clone(),
                                    exact_subject: target_type,
                                    exact_trait_ref: key.trait_ref.clone(),
                                },
                                impl_bindings,
                                nested: nested_evidences.into_boxed_slice(),
                            });
                            successes.push(GenericConformanceEvidence::Derived(derived));
                        }
                    }
                }
            }
        }
    }

    // 3. Classify candidate successes / coherence / failures
    if successes.len() == 1 {
        return GenericConformanceProof::Proven(successes.remove(0));
    }
    if successes.len() > 1 {
        return GenericConformanceProof::CoherenceConflict(conflicting_impls.into_boxed_slice());
    }

    for failure in &failure_proofs {
        if matches!(failure, GenericConformanceProof::BudgetExceeded(_)) {
            return failure.clone();
        }
    }
    for failure in &failure_proofs {
        if matches!(failure, GenericConformanceProof::Cancelled) {
            return failure.clone();
        }
    }
    for failure in &failure_proofs {
        if matches!(failure, GenericConformanceProof::InternalFailure(_)) {
            return failure.clone();
        }
    }
    for failure in &failure_proofs {
        if matches!(failure, GenericConformanceProof::Blocked(_)) {
            return failure.clone();
        }
    }
    for failure in &failure_proofs {
        if matches!(failure, GenericConformanceProof::Dynamic(_)) {
            return failure.clone();
        }
    }
    for failure in &failure_proofs {
        if matches!(failure, GenericConformanceProof::Incomplete(_)) {
            return failure.clone();
        }
    }
    for failure in &failure_proofs {
        if matches!(failure, GenericConformanceProof::Unknown(_)) {
            return failure.clone();
        }
    }
    GenericConformanceProof::Missing
}

/// Recursive conformance proof query evaluated under an explicit session context.
pub fn prove_conformance_with_context(
    key: &GenericConformanceKey,
    context: &mut ConformanceProofContext,
) -> GenericConformanceProof {
    if context.cancellation.is_cancelled() {
        return GenericConformanceProof::Cancelled;
    }

    if let Some(state) = context.memo.get(key) {
        match state {
            ProofMemoState::Visiting => return GenericConformanceProof::Blocked(BlockReason::RecursiveFixpoint),
            ProofMemoState::Completed(proof) => return proof.clone(),
        }
    }

    if let Err(report) = context.budget.charge_step() {
        return GenericConformanceProof::BudgetExceeded(report);
    }

    context.stack.push(key.clone());
    context.memo.insert(key.clone(), ProofMemoState::Visiting);

    let proof = evaluate_conformance_goal(key, context);

    context.stack.pop();
    context.memo.insert(key.clone(), ProofMemoState::Completed(proof.clone()));
    proof
}

/// Direct generic conformance proof query.
/// Checks assumed evidence from the lexical environment first; if not present,
/// and the subject is a canonical concrete type, delegates to exact conformance resolution.
pub fn query_generic_conformance_proof(
    subject: &TypeTerm,
    trait_ref: &TraitRef,
    environment: Option<&GenericEvidenceEnvironment>,
    conformance_index: Option<&ConformanceIndex>,
    witness_plans: Option<&BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>>,
    trait_surfaces: Option<&TraitSurfaceTable>,
    derived_rules: Option<&crate::derived_conformance::DerivedConformanceRuleIndex>,
    declarations: Option<&DeclarationTypeTable>,
    hierarchy: Option<&dyn TypeHierarchy>,
    store: Option<&mut TypeStore>,
) -> GenericConformanceProof {
    let mut context = ConformanceProofContext::new(
        environment,
        conformance_index,
        witness_plans,
        trait_surfaces,
        declarations,
        hierarchy,
        store,
    ).with_derived_rules(derived_rules);
    let key = GenericConformanceKey::new(subject.clone(), trait_ref.clone());
    prove_conformance_with_context(&key, &mut context)
}
