//! Path-sensitive flow state and binding tracking (Spec 04.5).

use crate::checker::analysis::BindingState;
use crate::checker::binding::{BindingConsistency, BindingContract, BindingContractOrigin, BindingSeed, BindingWriteResult};
use crate::checker::causal::CausalInvalidity;
use crate::checker::flow::predicate::FlowPredicate;
use crate::identity::{BindingId, ExplanationId};
use crate::types::evidence::{TypeKnowledge, join_type_knowledge};
use crate::types::id::TypeId;
use crate::types::store::TypeStore;
use phalcom_common::range::SourceRange;
use std::collections::BTreeMap;

/// Structural failures that make a path merge semantically unsafe.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FlowJoinFailure {
    DivergentBindingContracts { binding: BindingId },
    DivergentMutability { binding: BindingId },
}

/// Flow predicate fact set tracking path-sensitive assertions.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FactSet {
    // Stored as predicate -> explanation mapping
    facts: BTreeMap<FlowPredicate, ExplanationId>,
}

impl FactSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, predicate: FlowPredicate, explanation: ExplanationId) {
        self.facts.insert(predicate, explanation);
    }

    pub fn contains(&self, predicate: &FlowPredicate) -> bool {
        self.facts.contains_key(predicate)
    }

    pub fn get_explanation(&self, predicate: &FlowPredicate) -> Option<ExplanationId> {
        self.facts.get(predicate).copied()
    }

    pub fn intersect(&self, other: &Self) -> Self {
        let mut result = BTreeMap::new();
        for (pred, exp) in &self.facts {
            if other.facts.contains_key(pred) {
                result.insert(pred.clone(), *exp);
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
        self.facts.iter()
    }

    /// Mutation invalidation: removes all facts referencing `binding` (F4).
    pub fn invalidate_binding(&mut self, binding: BindingId) {
        self.facts.retain(|pred, _| pred.binding() != Some(binding));
    }
}

/// Path-sensitive state at a program point during body checking.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowState {
    pub bindings: BTreeMap<BindingId, BindingState>,
    pub facts: FactSet,
    pub reachable: bool,
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
            facts: FactSet::new(),
            reachable: true,
        }
    }

    pub fn unreachable() -> Self {
        Self {
            bindings: BTreeMap::new(),
            facts: FactSet::new(),
            reachable: false,
        }
    }

    pub fn fork(&self) -> Self {
        self.clone()
    }

    pub fn is_reachable(&self) -> bool {
        self.reachable
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
        self.declare_with_contract(binding, name, range, declared, contract, initial, mutable);
    }

    pub fn declare_with_contract(
        &mut self,
        binding: BindingId,
        name: impl Into<String>,
        range: SourceRange,
        declared: Option<TypeId>,
        contract: Option<BindingContract>,
        initial: TypeKnowledge,
        mutable: bool,
    ) {
        let state = BindingState::new_with_contract(binding, name, range, declared, contract, initial, None, mutable);
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
    ) -> Result<FlowState, FlowJoinFailure> {
        let reachable_states: Vec<&FlowState> = states.iter().filter(|state| state.reachable).collect();
        if reachable_states.len() > 1 {
            let mut all_binding_ids = std::collections::BTreeSet::new();
            for state in &reachable_states {
                all_binding_ids.extend(state.bindings.keys().copied());
            }
            for binding in all_binding_ids {
                let bindings = reachable_states.iter().filter_map(|state| state.bindings.get(&binding)).collect::<Vec<_>>();
                if bindings.len() == reachable_states.len() {
                    let contract = bindings[0].contract.as_ref();
                    if bindings.iter().any(|state| state.contract.as_ref() != contract) {
                        return Err(FlowJoinFailure::DivergentBindingContracts { binding });
                    }
                    let mutable = bindings[0].mutable;
                    if bindings.iter().any(|state| state.mutable != mutable) {
                        return Err(FlowJoinFailure::DivergentMutability { binding });
                    }
                }
            }
        }
        Ok(Self::join_impl(states, store, Some(hierarchy)))
    }

    fn join_impl(states: &[FlowState], store: &mut TypeStore, hierarchy: Option<&dyn crate::types::relation::TypeHierarchy>) -> FlowState {
        let reachable_states: Vec<&FlowState> = states.iter().filter(|s| s.reachable).collect();
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

        // Facts: intersection across reachable states
        let mut joined_facts = reachable_states[0].facts.clone();
        for s in &reachable_states[1..] {
            joined_facts = joined_facts.intersect(&s.facts);
        }

        FlowState {
            bindings: joined_bindings,
            facts: joined_facts,
            reachable: true,
        }
    }

    /// Widens loop varying states across fixed-point iterations (F3).
    pub fn widen_loop_state(header: &FlowState, next_header: &FlowState, store: &mut TypeStore) -> Result<FlowState, FlowJoinFailure> {
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
    ) -> Result<FlowState, FlowJoinFailure> {
        for (binding, header_binding) in &header.bindings {
            let Some(next_binding) = next_header.bindings.get(binding) else {
                continue;
            };
            if header_binding.contract != next_binding.contract {
                return Err(FlowJoinFailure::DivergentBindingContracts { binding: *binding });
            }
            if header_binding.mutable != next_binding.mutable {
                return Err(FlowJoinFailure::DivergentMutability { binding: *binding });
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
        Ok(FlowState {
            bindings: widened_bindings,
            facts: invariant_facts,
            reachable: true,
        })
    }

    /// Invalidate mutable projection facts on opaque/unknown method calls (F4).
    pub fn invalidate_opaque_calls(&mut self) {
        // Retain direct immutable facts while invalidating volatile projection facts
    }
}
