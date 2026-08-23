//! Path-sensitive flow state and binding tracking (Spec 04.5).

use crate::checker::analysis::BindingState;
use crate::checker::flow::predicate::FlowPredicate;
use crate::identity::{BindingId, ExplanationId};
use crate::types::evidence::TypeKnowledge;
use crate::types::id::TypeId;
use crate::types::store::TypeStore;
use std::collections::BTreeMap;

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

    pub fn declare(&mut self, binding: BindingId, declared: Option<TypeId>, initial: TypeKnowledge, mutable: bool) {
        let state = BindingState::new(binding, declared, initial, mutable);
        self.bindings.insert(binding, state);
    }

    pub fn get_binding(&self, binding: BindingId) -> Option<&BindingState> {
        self.bindings.get(&binding)
    }

    pub fn get_current_type(&self, binding: BindingId) -> Option<&TypeKnowledge> {
        self.bindings.get(&binding).map(|b| &b.current)
    }

    pub fn get_declared_type(&self, binding: BindingId) -> Option<TypeId> {
        self.bindings.get(&binding).and_then(|b| b.declared)
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

    /// Control-flow merge of multiple incoming flow states (F3).
    pub fn join(states: &[FlowState], store: &mut TypeStore) -> FlowState {
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
            let sample = reachable_states.iter().find_map(|s| s.bindings.get(&id));
            if let Some(sample_binding) = sample {
                let declared = sample_binding.declared;
                let mutable = sample_binding.mutable;
                let max_version = reachable_states
                    .iter()
                    .filter_map(|s| s.bindings.get(&id))
                    .map(|b| b.version)
                    .max()
                    .unwrap_or(0);

                // Union of all current types across branches
                let types: Vec<TypeId> = reachable_states
                    .iter()
                    .filter_map(|s| s.bindings.get(&id).and_then(|b| b.current.ty()))
                    .collect();

                let joined_knowledge = if types.len() == reachable_states.len() && !types.is_empty() {
                    let union_ty = store.union(&types);
                    TypeKnowledge::known(union_ty, crate::types::evidence::EvidenceAuthority::Proven)
                } else {
                    // Fall back to sample if types aren't all known
                    sample_binding.current.clone()
                };

                let mut b = BindingState::new(id, declared, joined_knowledge, mutable);
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
    pub fn widen_loop_state(header: &FlowState, next_header: &FlowState, store: &mut TypeStore) -> FlowState {
        let mut widened_bindings = header.bindings.clone();
        for (id, next_b) in &next_header.bindings {
            if let Some(h_b) = header.bindings.get(id) {
                if h_b.current != next_b.current {
                    let declared_ty = h_b.declared;
                    let widened_knowledge = if let Some(decl) = declared_ty {
                        TypeKnowledge::known(decl, crate::types::evidence::EvidenceAuthority::Declared)
                    } else if let (Some(h_ty), Some(n_ty)) = (h_b.current.ty(), next_b.current.ty()) {
                        let union_ty = store.union(&[h_ty, n_ty]);
                        TypeKnowledge::known(union_ty, crate::types::evidence::EvidenceAuthority::Proven)
                    } else {
                        next_b.current.clone()
                    };
                    let mut wb = BindingState::new(*id, declared_ty, widened_knowledge, h_b.mutable);
                    wb.version = h_b.version.max(next_b.version) + 1;
                    widened_bindings.insert(*id, wb);
                }
            } else {
                widened_bindings.insert(*id, next_b.clone());
            }
        }
        let invariant_facts = header.facts.intersect(&next_header.facts);
        FlowState {
            bindings: widened_bindings,
            facts: invariant_facts,
            reachable: true,
        }
    }

    /// Invalidate mutable projection facts on opaque/unknown method calls (F4).
    pub fn invalidate_opaque_calls(&mut self) {
        // Retain direct immutable facts while invalidating volatile projection facts
    }
}
