//! Canonical generic conformance evidence, lexical evidence environments,
//! deterministic evidence layout, and direct proof queries.

use crate::declarations::DeclarationTypeTable;
use crate::identity::ImplId;
use crate::impls::{ConformanceEvidence, ConformanceIndex, ConformanceResolution, ConformanceWitnessPlan};
use crate::traits::{TraitRef, TraitRequirementId, TraitSurfaceTable};
use crate::types::evidence::UnknownReason;
use crate::types::outcome::{BlockReason, BudgetReport, DynamicBoundaryObligation};
use crate::types::parameter::{GenericConstraint, TypeParameterOwner, TypeTerm};
use crate::types::relation::TypeHierarchy;
use crate::types::store::TypeStore;
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

/// Proven generic conformance evidence: either exact (backed by a source impl)
/// or assumed (derived from a lexical generic bound).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GenericConformanceEvidence {
    Exact(Arc<ConformanceEvidence>),
    Assumed(AssumedConformanceEvidence),
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
    layout: GenericEvidenceLayout,
}

impl GenericEvidenceEnvironment {
    pub fn empty() -> Self {
        Self {
            constraints: vec![].into_boxed_slice(),
            conformances: BTreeMap::new(),
            layout: GenericEvidenceLayout::empty(),
        }
    }

    pub fn new(
        constraints: Box<[GenericConstraint]>,
        conformances: BTreeMap<GenericConformanceKey, AssumedConformanceEvidence>,
        layout: GenericEvidenceLayout,
    ) -> Self {
        Self {
            constraints,
            conformances,
            layout,
        }
    }

    /// Builds a generic evidence environment from a slice of constraints and owner.
    pub fn from_constraints(
        constraints: &[GenericConstraint],
        owner: TypeParameterOwner,
        trait_surfaces: &TraitSurfaceTable,
    ) -> Self {
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
        let layout = GenericEvidenceLayout::build(&conformances, trait_surfaces).unwrap_or_default();
        Self {
            constraints: constraints.to_vec().into_boxed_slice(),
            conformances,
            layout,
        }
    }

    /// Composes two evidence environments (e.g. enclosing declaration + callable).
    pub fn compose(&self, other: &Self, trait_surfaces: &TraitSurfaceTable) -> Self {
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

        let layout = GenericEvidenceLayout::build(&conformances, trait_surfaces).unwrap_or_default();
        Self {
            constraints: combined_constraints.into_boxed_slice(),
            conformances,
            layout,
        }
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
    declarations: Option<&DeclarationTypeTable>,
    hierarchy: Option<&dyn TypeHierarchy>,
    store: Option<&mut TypeStore>,
) -> GenericConformanceProof {
    // 1. Check lexical assumptions in the environment
    let key = GenericConformanceKey::new(subject.clone(), trait_ref.clone());
    if let Some(env) = environment {
        if let Some(assumed) = env.assumed_conformance(&key) {
            return GenericConformanceProof::Proven(GenericConformanceEvidence::Assumed(assumed.clone()));
        }
    }

    // 2. Exact fallback for canonical types
    match subject {
        TypeTerm::Canonical(target_type) => {
            if let (Some(index), Some(plans), Some(surfaces), Some(decls), Some(hier), Some(st)) = (
                conformance_index,
                witness_plans,
                trait_surfaces,
                declarations,
                hierarchy,
                store,
            ) {
                if !surfaces.contains(&trait_ref.declaration) {
                    return GenericConformanceProof::InternalFailure("trait surface is unavailable".into());
                }
                let exact_res = crate::impls::resolve_conformance_evidence_with_surfaces(
                    index,
                    plans,
                    surfaces,
                    decls,
                    st,
                    hier,
                    *target_type,
                    trait_ref,
                );
                match exact_res {
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
            } else {
                GenericConformanceProof::Missing
            }
        }
        _ => GenericConformanceProof::Missing,
    }
}
