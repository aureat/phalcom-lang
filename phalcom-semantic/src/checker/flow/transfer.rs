//! Transfer functions for path-sensitive state update (Spec 04.5).

use super::predicate::FlowPredicate;
use super::state::FlowState;
use crate::identity::BindingId;
use crate::types::evidence::{EvidenceOrigin, TypeKnowledge};
use crate::types::id::TypeId;
use crate::types::relation::{self, TypeHierarchy};
use crate::types::store::TypeStore;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedFlowRefinement {
    pub binding: BindingId,
    pub prior: TypeKnowledge,
    pub refined: TypeKnowledge,
}

/// Applies a predicate and returns the concrete binding refinement, if any.
/// Explanation identity is deliberately allocated by the checking context.
pub fn apply_predicate(
    state: &mut FlowState,
    predicate: &FlowPredicate,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> Option<AppliedFlowRefinement> {
    let binding = predicate.binding();
    let prior = binding.and_then(|binding| state.get_current_type(binding)).cloned();
    match predicate {
        FlowPredicate::IsInstance { binding, target } => {
            refine_binding_type(state, *binding, *target, store, hierarchy);
        }
        FlowPredicate::IsNotInstance { binding, target } => {
            if let Some(current_ty) = state.get_current_type(*binding).and_then(|k| k.ty()) {
                if let crate::types::store::TypeData::Union(members) = store.get(current_ty).clone() {
                    let remaining: Vec<TypeId> = members
                        .iter()
                        .copied()
                        .filter(|&member| !relation::is_subtype(store, hierarchy, member, *target))
                        .collect();
                    if !remaining.is_empty() && remaining.len() < members.len() {
                        let refined = store.union(&remaining);
                        state.assign(*binding, TypeKnowledge::established(refined, EvidenceOrigin::Flow));
                    }
                }
            }
        }
        FlowPredicate::IsNil { binding } => {
            state.assign(*binding, TypeKnowledge::established(store.unit(), EvidenceOrigin::Flow));
        }
        FlowPredicate::NotNil { binding } => {
            // Filter out nil from union if present
            if let Some(current_ty) = state.get_current_type(*binding).and_then(|k| k.ty()) {
                if let crate::types::store::TypeData::Union(members) = store.get(current_ty).clone() {
                    let non_nil: Vec<TypeId> = members.iter().copied().filter(|&m| m != store.unit()).collect();
                    if !non_nil.is_empty() && non_nil.len() < members.len() {
                        let refined = store.union(&non_nil);
                        state.assign(*binding, TypeKnowledge::established(refined, EvidenceOrigin::Flow));
                    }
                }
            }
        }
        FlowPredicate::Equal { binding, target } => {
            state.assign(*binding, TypeKnowledge::established(*target, EvidenceOrigin::Flow));
        }
        FlowPredicate::NotEqual { binding, target } => {
            if let Some(current_ty) = state.get_current_type(*binding).and_then(|k| k.ty()) {
                if let crate::types::store::TypeData::Union(members) = store.get(current_ty).clone() {
                    let remaining: Vec<TypeId> = members.iter().copied().filter(|&m| m != *target).collect();
                    if !remaining.is_empty() && remaining.len() < members.len() {
                        let refined = store.union(&remaining);
                        state.assign(*binding, TypeKnowledge::established(refined, EvidenceOrigin::Flow));
                    }
                }
            }
        }
        FlowPredicate::EqualLiteral { .. }
        | FlowPredicate::NotEqualLiteral { .. }
        | FlowPredicate::OrderedPredicate { .. }
        | FlowPredicate::Truthy { .. }
        | FlowPredicate::Falsy { .. } => {}
    }

    state.facts.insert_unexplained(predicate.clone());
    let (Some(binding), Some(prior)) = (binding, prior) else {
        return None;
    };
    let refined = state.get_current_type(binding)?.clone();
    (refined != prior).then_some(AppliedFlowRefinement { binding, prior, refined })
}

fn refine_binding_type(state: &mut FlowState, binding: BindingId, target: TypeId, store: &mut TypeStore, hierarchy: &dyn TypeHierarchy) {
    if let Some(current_ty) = state.get_current_type(binding).and_then(|k| k.ty()) {
        if let crate::types::store::TypeData::Union(members) = store.get(current_ty).clone() {
            let matched: Vec<TypeId> = members
                .iter()
                .copied()
                .filter(|&member| relation::is_subtype(store, hierarchy, member, target))
                .collect();
            if !matched.is_empty() {
                let refined = store.union(&matched);
                state.assign(binding, TypeKnowledge::established(refined, EvidenceOrigin::Flow));
                return;
            }
        }

        if current_ty == target || relation::is_subtype(store, hierarchy, target, current_ty) {
            state.assign(binding, TypeKnowledge::established(target, EvidenceOrigin::Flow));
        }
    }
}
