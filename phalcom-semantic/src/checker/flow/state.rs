//! Path-sensitive flow state and binding tracking (Spec 04.5).

use crate::checker::analysis::BindingState;
use crate::checker::binding::{BindingConsistency, BindingContract, BindingContractOrigin, BindingSeed, BindingWriteResult};
use crate::checker::causal::CausalInvalidity;
use crate::checker::flow::predicate::FlowPredicate;
use crate::identity::FieldId;
use crate::identity::{AnalysisIncidentId, BindingId, ExplanationId};
use crate::types::denotation::SemanticDenotation;
use crate::types::evidence::{DynamicReason, EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason, join_type_knowledge};
use crate::types::id::TypeId;
use crate::types::store::TypeStore;
use phalcom_common::range::SourceRange;
use std::collections::{BTreeMap, BTreeSet};

/// Structural failures that make a path merge semantically unsafe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlowInvariantFailure {
    DivergentBindingContract {
        binding: BindingId,
        left: Option<BindingContract>,
        right: Option<BindingContract>,
    },
    DivergentMutability {
        binding: BindingId,
        left: bool,
        right: bool,
    },
    DivergentFieldContract {
        field: FieldId,
        left: Box<TypeKnowledge>,
        right: Box<TypeKnowledge>,
    },
}

#[cfg(test)]
mod field_tests {
    use super::*;
    use crate::identity::{DeclarationId, DispatchSide, ModuleId};
    use crate::types::evidence::EvidenceOrigin;

    fn field() -> FieldId {
        FieldId::new(DeclarationId::new(ModuleId::core(), "Counter".into()), "_value", DispatchSide::Instance)
    }

    #[test]
    fn field_state_keeps_contract_current_and_initialization_separate() {
        let mut store = TypeStore::new();
        let int_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "Int".into()));
        let string_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "String".into()));
        let id = field();
        let contract = TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation);
        let mut flow = FlowState::new();
        flow.seed_field(FieldState {
            field: id.clone(),
            contract: contract.clone(),
            current: TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::MissingInitializer),
            initialization: FieldInitialization::Uninitialized,
            validity: FieldContractValidity::Unchecked,
            causal_invalidity: CausalInvalidity::Clean,
            version: 0,
        });
        flow.write_field(
            &id,
            TypeKnowledge::established(string_ty, EvidenceOrigin::Flow),
            FieldInitialization::DefinitelyInitialized,
            FieldContractValidity::Validated,
            CausalInvalidity::Clean,
        );
        let state = flow.get_field(&id).expect("field state");
        assert_eq!(state.contract, contract);
        assert_eq!(state.current.ty(), Some(string_ty));
        assert_eq!(state.version, 1);
    }

    #[test]
    fn field_join_tracks_current_and_definite_initialization_over_reachable_paths() {
        let mut store = TypeStore::new();
        let int_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "Int".into()));
        let float_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "Float".into()));
        let id = field();
        let contract = TypeKnowledge::assumed(int_ty, EvidenceOrigin::DeveloperAnnotation);
        let seed = |current, initialization| FieldState {
            field: id.clone(),
            contract: contract.clone(),
            current,
            initialization,
            validity: FieldContractValidity::Validated,
            causal_invalidity: CausalInvalidity::Clean,
            version: 0,
        };
        let mut left = FlowState::new();
        left.seed_field(seed(
            TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
            FieldInitialization::DefinitelyInitialized,
        ));
        let mut right = FlowState::new();
        right.seed_field(seed(
            TypeKnowledge::established(float_ty, EvidenceOrigin::Flow),
            FieldInitialization::DefinitelyInitialized,
        ));
        let joined = FlowState::join(&[left.clone(), right], &mut store);
        let joined_field = joined.get_field(&id).expect("joined field");
        assert_eq!(joined_field.initialization, FieldInitialization::DefinitelyInitialized);
        assert_ne!(joined_field.current.ty(), Some(int_ty));

        let mut uninitialized = FlowState::new();
        uninitialized.seed_field(seed(
            TypeKnowledge::Unknown(crate::types::evidence::UnknownReason::MissingInitializer),
            FieldInitialization::Uninitialized,
        ));
        let joined = FlowState::join(&[left.clone(), uninitialized], &mut store);
        assert_eq!(
            joined.get_field(&id).expect("joined field").initialization,
            FieldInitialization::MaybeInitialized
        );

        let unreachable = FlowState::unreachable();
        let joined = FlowState::join(&[left, unreachable], &mut store);
        assert_eq!(
            joined.get_field(&id).expect("reachable field").initialization,
            FieldInitialization::DefinitelyInitialized
        );
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FieldInitialization {
    Uninitialized,
    MaybeInitialized,
    DefinitelyInitialized,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldContractValidity {
    Unchecked,
    Validated,
    Assumed,
    Refuted,
    Blocked(crate::types::outcome::BlockReason),
    DynamicBoundary(crate::types::outcome::DynamicBoundaryObligation),
}

pub(crate) fn join_two_field_validities(a: FieldContractValidity, b: FieldContractValidity) -> FieldContractValidity {
    use FieldContractValidity::*;
    match (a, b) {
        (Refuted, _) | (_, Refuted) => Refuted,
        (Blocked(r), _) | (_, Blocked(r)) => Blocked(r),
        (DynamicBoundary(o), _) | (_, DynamicBoundary(o)) => DynamicBoundary(o),
        (Unchecked, _) | (_, Unchecked) => Unchecked,
        (Assumed, _) | (_, Assumed) => Assumed,
        (Validated, Validated) => Validated,
    }
}

pub fn join_field_validity(inputs: impl IntoIterator<Item = FieldContractValidity>) -> FieldContractValidity {
    let mut result = FieldContractValidity::Validated;
    let mut saw_any = false;

    for input in inputs {
        saw_any = true;
        result = join_two_field_validities(result, input);
    }

    if saw_any { result } else { FieldContractValidity::Unchecked }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldState {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub validity: FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
    pub version: u32,
}

/// Compatibility alias retained for callers compiled against the parent plan.
pub type FlowJoinFailure = FlowInvariantFailure;

/// Flow predicate fact set tracking path-sensitive assertions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FactSet {
    // Stored as predicate -> explanation mapping
    facts: BTreeMap<FlowPredicate, Option<ExplanationId>>,
}

impl FactSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, predicate: FlowPredicate, explanation: ExplanationId) {
        self.facts.insert(predicate, Some(explanation));
    }

    pub fn insert_unexplained(&mut self, predicate: FlowPredicate) {
        self.facts.entry(predicate).or_insert(None);
    }

    pub fn contains(&self, predicate: &FlowPredicate) -> bool {
        self.facts.contains_key(predicate)
    }

    pub fn get_explanation(&self, predicate: &FlowPredicate) -> Option<ExplanationId> {
        self.facts.get(predicate).copied().flatten()
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let mut result = BTreeMap::new();
        for (pred, exp) in &self.facts {
            if let Some(other_exp) = other.facts.get(pred) {
                result.insert(pred.clone(), exp.or(*other_exp));
            }
        }
        Self { facts: result }
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = (&FlowPredicate, &ExplanationId)> {
        self.facts
            .iter()
            .filter_map(|(predicate, explanation)| explanation.as_ref().map(|explanation| (predicate, explanation)))
    }

    pub fn predicate_keys(&self) -> impl Iterator<Item = &FlowPredicate> {
        self.facts.keys()
    }

    /// Mutation invalidation: removes all facts referencing `binding` (F4).
    pub fn invalidate_binding(&mut self, binding: BindingId) {
        self.facts.retain(|pred, _| pred.binding() != Some(binding));
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum KnowledgeFixpointKey {
    Known {
        ty: TypeId,
        status: EvidenceStatus,
        origin: EvidenceOrigin,
    },
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}

impl From<&TypeKnowledge> for KnowledgeFixpointKey {
    fn from(k: &TypeKnowledge) -> Self {
        match k {
            TypeKnowledge::Known(known) => Self::Known {
                ty: known.ty(),
                status: known.status(),
                origin: known.origin(),
            },
            TypeKnowledge::Unknown(u) => Self::Unknown(u.clone()),
            TypeKnowledge::Dynamic(d) => Self::Dynamic(d.clone()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BindingFixpointKey {
    pub contract: Option<BindingContract>,
    pub current: KnowledgeFixpointKey,
    pub denotation: Option<SemanticDenotation>,
    pub consistency: BindingConsistency,
    pub causal_invalidity: CausalInvalidity,
    pub mutable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FieldFixpointKey {
    pub contract: KnowledgeFixpointKey,
    pub current: KnowledgeFixpointKey,
    pub initialization: FieldInitialization,
    pub validity: FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FlowFixpointKey {
    pub reachable: bool,
    pub bindings: BTreeMap<BindingId, BindingFixpointKey>,
    pub fields: BTreeMap<FieldId, FieldFixpointKey>,
    pub predicates: BTreeSet<FlowPredicate>,
}

/// Path-sensitive state at a program point during body checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowState {
    pub bindings: BTreeMap<BindingId, BindingState>,
    pub fields: BTreeMap<FieldId, FieldState>,
    pub facts: FactSet,
    pub reachable: bool,
    poisoned: Option<AnalysisIncidentId>,
}

impl Default for FlowState {
    fn default() -> Self {
        Self::new()
    }
}

impl FlowState {
    pub fn new() -> Self {
        Self {
            bindings: BTreeMap::new(),
            fields: BTreeMap::new(),
            facts: FactSet::new(),
            reachable: true,
            poisoned: None,
        }
    }

    pub fn unreachable() -> Self {
        Self {
            bindings: BTreeMap::new(),
            fields: BTreeMap::new(),
            facts: FactSet::new(),
            reachable: false,
            poisoned: None,
        }
    }

    pub fn poisoned(incident: AnalysisIncidentId) -> Self {
        Self {
            bindings: BTreeMap::new(),
            fields: BTreeMap::new(),
            facts: FactSet::new(),
            reachable: false,
            poisoned: Some(incident),
        }
    }

    pub fn fork(&self) -> Self {
        self.clone()
    }

    pub fn is_reachable(&self) -> bool {
        self.reachable && self.poisoned.is_none()
    }

    pub fn is_poisoned(&self) -> bool {
        self.poisoned.is_some()
    }

    pub fn poisoned_by(&self) -> Option<AnalysisIncidentId> {
        self.poisoned
    }

    pub fn mark_unreachable(&mut self) {
        self.reachable = false;
    }

    pub fn declare(
        &mut self,
        binding: BindingId,
        name: impl Into<String>,
        range: SourceRange,
        declared: Option<TypeId>,
        initial: TypeKnowledge,
        mutable: bool,
    ) {
        let contract = declared.map(|ty| BindingContract {
            ty,
            origin: BindingContractOrigin::SourceAnnotation,
            source: Some(range),
        });
        self.declare_with_contract(binding, name, range, contract, initial, mutable);
    }

    pub fn declare_with_contract(
        &mut self,
        binding: BindingId,
        name: impl Into<String>,
        range: SourceRange,
        contract: Option<BindingContract>,
        initial: TypeKnowledge,
        mutable: bool,
    ) {
        let state = BindingState::new_with_contract(binding, name, range, contract, initial, None, mutable);
        self.bindings.insert(binding, state);
    }

    pub fn declare_seed(&mut self, binding: BindingId, seed: BindingSeed, current: TypeKnowledge, consistency: BindingConsistency) {
        let state = BindingState::from_seed(binding, seed, current, consistency);
        self.bindings.insert(binding, state);
    }

    pub fn get_binding(&self, binding: BindingId) -> Option<&BindingState> {
        self.bindings.get(&binding)
    }

    pub fn set_binding_explanation(&mut self, binding: BindingId, explanation: ExplanationId) {
        if let Some(state) = self.bindings.get_mut(&binding) {
            state.explanation = Some(explanation);
        }
    }

    pub fn get_current_type(&self, binding: BindingId) -> Option<&TypeKnowledge> {
        self.bindings.get(&binding).map(|b| &b.current)
    }

    pub fn get_declared_type(&self, binding: BindingId) -> Option<TypeId> {
        self.bindings.get(&binding).and_then(|b| b.contract.as_ref().map(|contract| contract.ty))
    }

    pub fn seed_field(&mut self, state: FieldState) {
        self.fields.insert(state.field.clone(), state);
    }

    pub fn get_field(&self, field: &FieldId) -> Option<&FieldState> {
        self.fields.get(field)
    }

    pub fn get_field_current(&self, field: &FieldId) -> Option<&TypeKnowledge> {
        self.fields.get(field).map(|state| &state.current)
    }

    pub fn write_field(
        &mut self,
        field: &FieldId,
        current: TypeKnowledge,
        initialization: FieldInitialization,
        validity: FieldContractValidity,
        causal_invalidity: CausalInvalidity,
    ) {
        if let Some(state) = self.fields.get_mut(field) {
            state.current = current;
            state.initialization = initialization;
            state.validity = validity;
            state.causal_invalidity = causal_invalidity;
            state.version += 1;
        }
    }

    /// Sequential assignment: replaces `current` fact, increments version,
    /// and invalidates dependent path facts (F4).
    pub fn assign(&mut self, binding: BindingId, new_knowledge: TypeKnowledge) {
        if let Some(b) = self.bindings.get_mut(&binding) {
            b.current = new_knowledge;
            b.version += 1;
        }
        self.facts.invalidate_binding(binding);
    }

    /// Applies a write only after declaration mutability and contract
    /// reconciliation have been checked by the semantic transfer layer.
    pub fn write(
        &mut self,
        binding: BindingId,
        new_knowledge: TypeKnowledge,
        denotation: Option<crate::types::denotation::SemanticDenotation>,
        consistency: BindingConsistency,
        causal_invalidity: CausalInvalidity,
    ) -> BindingWriteResult {
        let Some(state) = self.bindings.get_mut(&binding) else {
            return BindingWriteResult::Missing;
        };
        if !state.mutable {
            return BindingWriteResult::Immutable;
        }
        state.current = new_knowledge;
        state.denotation = denotation;
        state.consistency = consistency;
        state.causal_invalidity = causal_invalidity;
        state.version += 1;
        self.facts.invalidate_binding(binding);
        BindingWriteResult::Applied
    }

    /// Control-flow merge of multiple incoming flow states (F3).
    pub fn join(states: &[FlowState], store: &mut TypeStore) -> FlowState {
        Self::join_impl(states, store, None)
    }

    /// Control-flow merge with contract reconciliation against the canonical
    /// hierarchy. The compatibility `join` entry point remains available for
    /// low-level tests that do not have a hierarchy product.
    pub fn join_with_hierarchy(
        states: &[FlowState],
        store: &mut TypeStore,
        hierarchy: &dyn crate::types::relation::TypeHierarchy,
    ) -> Result<FlowState, FlowInvariantFailure> {
        let reachable_states: Vec<&FlowState> = states.iter().filter(|state| state.is_reachable()).collect();
        if reachable_states.len() > 1 {
            let mut all_binding_ids = std::collections::BTreeSet::new();
            for state in &reachable_states {
                all_binding_ids.extend(state.bindings.keys().copied());
            }
            for binding in all_binding_ids {
                let bindings = reachable_states.iter().filter_map(|state| state.bindings.get(&binding)).collect::<Vec<_>>();
                if bindings.len() == reachable_states.len() {
                    let contract = bindings[0].contract.as_ref();
                    if let Some(right) = bindings.iter().find(|state| state.contract.as_ref() != contract) {
                        return Err(FlowInvariantFailure::DivergentBindingContract {
                            binding,
                            left: contract.cloned(),
                            right: right.contract.clone(),
                        });
                    }
                    let mutable = bindings[0].mutable;
                    if let Some(right) = bindings.iter().find(|state| state.mutable != mutable) {
                        return Err(FlowInvariantFailure::DivergentMutability {
                            binding,
                            left: mutable,
                            right: right.mutable,
                        });
                    }
                }
            }
            let mut all_field_ids = std::collections::BTreeSet::new();
            for state in &reachable_states {
                all_field_ids.extend(state.fields.keys().cloned());
            }
            for field in all_field_ids {
                let fields = reachable_states.iter().filter_map(|state| state.fields.get(&field)).collect::<Vec<_>>();
                if fields.len() == reachable_states.len() {
                    let contract = &fields[0].contract;
                    if let Some(right) = fields.iter().find(|state| &state.contract != contract) {
                        return Err(FlowInvariantFailure::DivergentFieldContract {
                            field,
                            left: Box::new(contract.clone()),
                            right: Box::new(right.contract.clone()),
                        });
                    }
                }
            }
        }
        Ok(Self::join_impl(states, store, Some(hierarchy)))
    }

    fn join_impl(states: &[FlowState], store: &mut TypeStore, hierarchy: Option<&dyn crate::types::relation::TypeHierarchy>) -> FlowState {
        let reachable_states: Vec<&FlowState> = states.iter().filter(|s| s.is_reachable()).collect();
        if reachable_states.is_empty() {
            return FlowState::unreachable();
        }
        if reachable_states.len() == 1 {
            return reachable_states[0].clone();
        }

        let mut joined_bindings = BTreeMap::new();
        // Collect all binding IDs across reachable states
        let mut all_binding_ids = std::collections::BTreeSet::new();
        for s in &reachable_states {
            for id in s.bindings.keys() {
                all_binding_ids.insert(*id);
            }
        }

        for id in all_binding_ids {
            let Some(sample_binding) = reachable_states.iter().find_map(|s| s.bindings.get(&id)) else {
                continue;
            };
            if reachable_states.iter().all(|state| state.bindings.contains_key(&id)) {
                let contracts_match = reachable_states
                    .iter()
                    .all(|state| state.bindings.get(&id).and_then(|binding| binding.contract.as_ref()) == sample_binding.contract.as_ref());
                let contract = if contracts_match { sample_binding.contract.clone() } else { None };
                let mutable = reachable_states
                    .iter()
                    .all(|state| state.bindings.get(&id).is_some_and(|binding| binding.mutable));
                let max_version = reachable_states
                    .iter()
                    .filter_map(|s| s.bindings.get(&id))
                    .map(|b| b.version)
                    .max()
                    .unwrap_or(0);

                let joined_knowledge = join_type_knowledge(
                    store,
                    reachable_states
                        .iter()
                        .filter_map(|state| state.bindings.get(&id).map(|binding| binding.current.clone())),
                );

                let mut b = sample_binding.clone();
                // Divergent branches do not have one canonical contract. Do
                // not retain whichever branch happened to be visited first.
                b.contract = contract.clone();
                b.current = joined_knowledge;
                b.mutable = mutable;
                b.denotation = {
                    let first_denotation = sample_binding.denotation;
                    if reachable_states
                        .iter()
                        .all(|state| state.bindings.get(&id).and_then(|binding| binding.denotation) == first_denotation)
                    {
                        first_denotation
                    } else {
                        None
                    }
                };
                b.causal_invalidity = reachable_states
                    .iter()
                    .filter_map(|state| state.bindings.get(&id).map(|binding| binding.causal_invalidity))
                    .fold(CausalInvalidity::Clean, CausalInvalidity::join);
                if let Some(hierarchy) = hierarchy {
                    if let Some(contract) = b.contract.as_ref() {
                        let reconciliation = crate::checker::binding::reconcile_binding_contract(store, hierarchy, Some(contract), &b.current);
                        b.current = reconciliation.current;
                        b.consistency = reconciliation.consistency;
                    } else if contracts_match {
                        b.consistency = BindingConsistency::Unconstrained;
                    } else {
                        b.consistency = BindingConsistency::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint);
                    }
                } else if !contracts_match
                    || reachable_states
                        .iter()
                        .any(|state| state.bindings.get(&id).is_some_and(|binding| binding.consistency != sample_binding.consistency))
                {
                    b.consistency = BindingConsistency::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint);
                }
                b.version = max_version + 1;
                joined_bindings.insert(id, b);
            }
        }

        let mut joined_fields = BTreeMap::new();
        let mut all_field_ids = std::collections::BTreeSet::new();
        for state in &reachable_states {
            all_field_ids.extend(state.fields.keys().cloned());
        }
        for field in all_field_ids {
            let incoming = reachable_states.iter().filter_map(|state| state.fields.get(&field)).collect::<Vec<_>>();
            if incoming.len() != reachable_states.len() {
                continue;
            }
            let sample = incoming[0];
            let current = join_type_knowledge(store, incoming.iter().map(|state| state.current.clone()));
            let initialization = if incoming.iter().all(|state| state.initialization == FieldInitialization::DefinitelyInitialized) {
                FieldInitialization::DefinitelyInitialized
            } else if incoming.iter().all(|state| state.initialization == FieldInitialization::Uninitialized) {
                FieldInitialization::Uninitialized
            } else {
                FieldInitialization::MaybeInitialized
            };
            let validity = join_field_validity(incoming.iter().map(|state| state.validity.clone()));
            let causal_invalidity = incoming
                .iter()
                .map(|state| state.causal_invalidity)
                .fold(CausalInvalidity::Clean, CausalInvalidity::join);
            joined_fields.insert(
                field.clone(),
                FieldState {
                    field,
                    contract: sample.contract.clone(),
                    current,
                    initialization,
                    validity,
                    causal_invalidity,
                    version: incoming.iter().map(|state| state.version).max().unwrap_or(0) + 1,
                },
            );
        }

        // Facts: intersection across reachable states
        let mut joined_facts = reachable_states[0].facts.clone();
        for s in &reachable_states[1..] {
            joined_facts = joined_facts.intersect(&s.facts);
        }

        FlowState {
            bindings: joined_bindings,
            fields: joined_fields,
            facts: joined_facts,
            reachable: true,
            poisoned: None,
        }
    }

    /// Widens loop varying states across fixed-point iterations (F3).
    pub fn widen_loop_state(header: &FlowState, next_header: &FlowState, store: &mut TypeStore) -> Result<FlowState, FlowInvariantFailure> {
        let hierarchy = crate::types::relation::MapTypeHierarchy::default();
        Self::widen_loop_state_with_hierarchy(header, next_header, store, &hierarchy)
    }

    /// Widens loop state and immediately rechecks each widened current fact
    /// against its persistent contract using the canonical hierarchy.
    pub fn widen_loop_state_with_hierarchy(
        header: &FlowState,
        next_header: &FlowState,
        store: &mut TypeStore,
        hierarchy: &dyn crate::types::relation::TypeHierarchy,
    ) -> Result<FlowState, FlowInvariantFailure> {
        for (binding, header_binding) in &header.bindings {
            let Some(next_binding) = next_header.bindings.get(binding) else {
                continue;
            };
            if header_binding.contract != next_binding.contract {
                return Err(FlowInvariantFailure::DivergentBindingContract {
                    binding: *binding,
                    left: header_binding.contract.clone(),
                    right: next_binding.contract.clone(),
                });
            }
            if header_binding.mutable != next_binding.mutable {
                return Err(FlowInvariantFailure::DivergentMutability {
                    binding: *binding,
                    left: header_binding.mutable,
                    right: next_binding.mutable,
                });
            }
        }
        for (field, header_field) in &header.fields {
            let Some(next_field) = next_header.fields.get(field) else {
                continue;
            };
            if header_field.contract != next_field.contract {
                return Err(FlowInvariantFailure::DivergentFieldContract {
                    field: field.clone(),
                    left: Box::new(header_field.contract.clone()),
                    right: Box::new(next_field.contract.clone()),
                });
            }
        }
        let mut widened_bindings = header.bindings.clone();
        for (id, next_b) in &next_header.bindings {
            if let Some(h_b) = header.bindings.get(id) {
                if h_b.current != next_b.current {
                    let widened_knowledge = join_type_knowledge(store, [h_b.current.clone(), next_b.current.clone()]);
                    let mut wb = h_b.clone();
                    wb.current = widened_knowledge.clone();
                    wb.denotation = if h_b.denotation == next_b.denotation { h_b.denotation } else { None };
                    wb.causal_invalidity = h_b.causal_invalidity.join(next_b.causal_invalidity);
                    if let Some(contract) = wb.contract.as_ref() {
                        let reconciliation = crate::checker::binding::reconcile_binding_contract(store, hierarchy, Some(contract), &widened_knowledge);
                        wb.current = reconciliation.current;
                        wb.consistency = reconciliation.consistency;
                    } else if h_b.consistency != next_b.consistency {
                        wb.consistency = BindingConsistency::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint);
                    }
                    wb.version = h_b.version.max(next_b.version) + 1;
                    widened_bindings.insert(*id, wb);
                }
            }
        }
        let invariant_facts = header.facts.intersect(&next_header.facts);
        let widened_fields = Self::join_impl(&[header.clone(), next_header.clone()], store, Some(hierarchy)).fields;
        Ok(FlowState {
            bindings: widened_bindings,
            fields: widened_fields,
            facts: invariant_facts,
            reachable: true,
            poisoned: None,
        })
    }

    pub fn seed_binding(&mut self, state: BindingState) {
        self.bindings.insert(state.binding, state);
    }

    /// Invalidate mutable projection facts on opaque/unknown method calls (F4).
    pub fn invalidate_opaque_calls(&mut self) {
        // Retain direct immutable facts while invalidating volatile projection facts
    }

    /// Projects the flow state into a semantic key, stripping versions,
    /// explanation IDs, and AST allocation metadata.
    pub(crate) fn fixpoint_key(&self) -> FlowFixpointKey {
        let bindings = self
            .bindings
            .iter()
            .map(|(id, b)| {
                (
                    *id,
                    BindingFixpointKey {
                        contract: b.contract.clone(),
                        current: (&b.current).into(),
                        denotation: b.denotation,
                        consistency: b.consistency.clone(),
                        causal_invalidity: b.causal_invalidity,
                        mutable: b.mutable,
                    },
                )
            })
            .collect();

        let fields = self
            .fields
            .iter()
            .map(|(id, f)| {
                (
                    id.clone(),
                    FieldFixpointKey {
                        contract: (&f.contract).into(),
                        current: (&f.current).into(),
                        initialization: f.initialization,
                        validity: f.validity.clone(),
                        causal_invalidity: f.causal_invalidity,
                    },
                )
            })
            .collect();

        let predicates = self.facts.predicate_keys().cloned().collect();

        FlowFixpointKey {
            reachable: self.reachable,
            bindings,
            fields,
            predicates,
        }
    }

    /// Weakens changing/unstable dimensions to `Unknown(RecursiveFixpoint)`
    /// upon solver exhaustion while preserving stable contracts and facts.
    pub(crate) fn weaken_unstable_fixpoint_facts(previous: &FlowState, next: &FlowState) -> FlowState {
        let mut weakened_bindings = next.bindings.clone();
        for (id, next_b) in &next.bindings {
            if let Some(prev_b) = previous.bindings.get(id) {
                let prev_key = BindingFixpointKey {
                    contract: prev_b.contract.clone(),
                    current: (&prev_b.current).into(),
                    denotation: prev_b.denotation,
                    consistency: prev_b.consistency.clone(),
                    causal_invalidity: prev_b.causal_invalidity,
                    mutable: prev_b.mutable,
                };
                let next_key = BindingFixpointKey {
                    contract: next_b.contract.clone(),
                    current: (&next_b.current).into(),
                    denotation: next_b.denotation,
                    consistency: next_b.consistency.clone(),
                    causal_invalidity: next_b.causal_invalidity,
                    mutable: next_b.mutable,
                };
                if prev_key != next_key {
                    let mut b = next_b.clone();
                    b.current = TypeKnowledge::Unknown(UnknownReason::RecursiveFixpoint);
                    b.denotation = None;
                    b.consistency = if b.contract.is_some() {
                        BindingConsistency::Blocked(crate::types::outcome::BlockReason::RecursiveFixpoint)
                    } else {
                        BindingConsistency::Unconstrained
                    };
                    b.causal_invalidity = prev_b.causal_invalidity.join(next_b.causal_invalidity);
                    b.version = prev_b.version.max(next_b.version) + 1;
                    weakened_bindings.insert(*id, b);
                }
            }
        }

        let mut weakened_fields = next.fields.clone();
        for (id, next_f) in &next.fields {
            if let Some(prev_f) = previous.fields.get(id) {
                let prev_key = FieldFixpointKey {
                    contract: (&prev_f.contract).into(),
                    current: (&prev_f.current).into(),
                    initialization: prev_f.initialization,
                    validity: prev_f.validity.clone(),
                    causal_invalidity: prev_f.causal_invalidity,
                };
                let next_key = FieldFixpointKey {
                    contract: (&next_f.contract).into(),
                    current: (&next_f.current).into(),
                    initialization: next_f.initialization,
                    validity: next_f.validity.clone(),
                    causal_invalidity: next_f.causal_invalidity,
                };
                if prev_key != next_key {
                    let mut f = next_f.clone();
                    f.current = TypeKnowledge::Unknown(UnknownReason::RecursiveFixpoint);
                    f.initialization = if prev_f.initialization == next_f.initialization {
                        prev_f.initialization
                    } else {
                        FieldInitialization::MaybeInitialized
                    };
                    f.validity = join_two_field_validities(prev_f.validity.clone(), next_f.validity.clone());
                    f.causal_invalidity = prev_f.causal_invalidity.join(next_f.causal_invalidity);
                    f.version = prev_f.version.max(next_f.version) + 1;
                    weakened_fields.insert(id.clone(), f);
                }
            }
        }

        let invariant_facts = previous.facts.intersect(&next.facts);
        FlowState {
            bindings: weakened_bindings,
            fields: weakened_fields,
            facts: invariant_facts,
            reachable: previous.reachable && next.reachable,
            poisoned: previous.poisoned.or(next.poisoned),
        }
    }
}

#[cfg(test)]
mod fixpoint_projection_tests {
    use super::*;
    use crate::identity::{DeclarationId, ModuleId};

    #[test]
    fn fixpoint_key_ignores_versions_and_explanation_ids() {
        let mut store = TypeStore::new();
        let int_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "Int".into()));
        let binding_id = BindingId(1);

        let mut left = FlowState::new();
        left.seed_binding(BindingState {
            binding: binding_id,
            name: "x".into(),
            parameter: None,
            range: SourceRange::default(),
            contract: None,
            current: TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
            denotation: None,
            consistency: BindingConsistency::Unconstrained,
            causal_invalidity: CausalInvalidity::Clean,
            mutable: true,
            version: 0,
            explanation: Some(ExplanationId(10)),
        });

        let mut right = FlowState::new();
        right.seed_binding(BindingState {
            binding: binding_id,
            name: "x".into(),
            parameter: None,
            range: SourceRange::default(),
            contract: None,
            current: TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
            denotation: None,
            consistency: BindingConsistency::Unconstrained,
            causal_invalidity: CausalInvalidity::Clean,
            mutable: true,
            version: 5,
            explanation: Some(ExplanationId(999)),
        });

        assert_ne!(left, right);
        assert_eq!(left.fixpoint_key(), right.fixpoint_key());
    }

    #[test]
    fn weaken_unstable_fixpoint_facts_weakens_only_changing_dimensions() {
        let mut store = TypeStore::new();
        let int_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "Int".into()));
        let string_ty = store.nominal_type(DeclarationId::new(ModuleId::core(), "String".into()));
        let stable_id = BindingId(1);
        let unstable_id = BindingId(2);

        let mut prev = FlowState::new();
        prev.seed_binding(BindingState {
            binding: stable_id,
            name: "a".into(),
            parameter: None,
            range: SourceRange::default(),
            contract: None,
            current: TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
            denotation: None,
            consistency: BindingConsistency::Unconstrained,
            causal_invalidity: CausalInvalidity::Clean,
            mutable: false,
            version: 0,
            explanation: None,
        });
        prev.seed_binding(BindingState {
            binding: unstable_id,
            name: "b".into(),
            parameter: None,
            range: SourceRange::default(),
            contract: None,
            current: TypeKnowledge::established(int_ty, EvidenceOrigin::Flow),
            denotation: None,
            consistency: BindingConsistency::Unconstrained,
            causal_invalidity: CausalInvalidity::Clean,
            mutable: true,
            version: 0,
            explanation: None,
        });

        let mut next = prev.clone();
        next.write(
            unstable_id,
            TypeKnowledge::established(string_ty, EvidenceOrigin::Flow),
            None,
            BindingConsistency::Unconstrained,
            CausalInvalidity::Clean,
        );

        let weakened = FlowState::weaken_unstable_fixpoint_facts(&prev, &next);
        assert_eq!(weakened.get_binding(stable_id).unwrap().current.ty(), Some(int_ty));
        assert!(matches!(
            weakened.get_binding(unstable_id).unwrap().current,
            TypeKnowledge::Unknown(UnknownReason::RecursiveFixpoint)
        ));
    }
}
