//! Transfer functions for path-sensitive state update (Spec 04.5).

use super::predicate::{FlowPredicate, PredicateAuthority, TrustedFlowPredicate};
use super::state::FlowState;
use crate::identity::BindingId;
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus, TypeKnowledge, UnknownReason};
use crate::types::id::TypeId;
use crate::types::relation::{self, TypeHierarchy};
use crate::types::store::TypeStore;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppliedFlowRefinement {
    pub binding: BindingId,
    pub prior: TypeKnowledge,
    pub refined: TypeKnowledge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PredicateTransfer {
    Unchanged,
    Refined(AppliedFlowRefinement),
    Contradiction { binding: BindingId, prior: TypeKnowledge },
}

/// Applies a trusted predicate and returns the transfer outcome.
pub fn apply_predicate(state: &mut FlowState, predicate: &TrustedFlowPredicate, store: &mut TypeStore, hierarchy: &dyn TypeHierarchy) -> PredicateTransfer {
    let binding = predicate.predicate.binding();
    let prior = binding.and_then(|binding| state.get_current_type(binding)).cloned();
    let Some(binding) = binding else {
        return PredicateTransfer::Unchanged;
    };
    let Some(prior) = prior else {
        return PredicateTransfer::Unchanged;
    };

    let outcome = match &predicate.predicate {
        FlowPredicate::IsInstance { target, .. } => positive_type_refinement(state, binding, &prior, *target, predicate.authority, store, hierarchy),
        FlowPredicate::IsNotInstance { target, .. } => negative_type_refinement(state, binding, &prior, *target, store, hierarchy),
        FlowPredicate::IsNil { .. } => positive_nil_refinement(state, binding, &prior, store, hierarchy),
        FlowPredicate::NotNil { .. } => negative_nil_refinement(state, binding, &prior, store, hierarchy),
        FlowPredicate::Equal { target, .. } => positive_equal_refinement(state, binding, &prior, *target, store, hierarchy),
        FlowPredicate::NotEqual { target, .. } => negative_equal_refinement(state, binding, &prior, *target, store, hierarchy),
        FlowPredicate::EqualLiteral { .. }
        | FlowPredicate::NotEqualLiteral { .. }
        | FlowPredicate::OrderedPredicate { .. }
        | FlowPredicate::Truthy { .. }
        | FlowPredicate::Falsy { .. } => PredicateTransfer::Unchanged,
    };

    state.facts.insert_unexplained(predicate.predicate.clone());
    outcome
}

fn positive_type_refinement(
    state: &mut FlowState,
    binding: BindingId,
    prior: &TypeKnowledge,
    target: TypeId,
    authority: PredicateAuthority,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> PredicateTransfer {
    match prior {
        TypeKnowledge::Unknown(_) => {
            if authority == PredicateAuthority::AuthoritativeObservation {
                let refined = TypeKnowledge::established(target, EvidenceOrigin::Flow);
                state.assign(binding, refined.clone());
                PredicateTransfer::Refined(AppliedFlowRefinement {
                    binding,
                    prior: prior.clone(),
                    refined,
                })
            } else {
                PredicateTransfer::Unchanged
            }
        }
        TypeKnowledge::Dynamic(_) => PredicateTransfer::Unchanged,
        TypeKnowledge::Known(evidence) => {
            let prior_ty = evidence.ty();
            let prior_status = evidence.status();

            if prior_status == EvidenceStatus::Established {
                if relation::is_subtype(store, hierarchy, prior_ty, target) {
                    PredicateTransfer::Unchanged
                } else if relation::is_subtype(store, hierarchy, target, prior_ty) {
                    let refined = TypeKnowledge::established(target, EvidenceOrigin::Flow);
                    state.assign(binding, refined.clone());
                    PredicateTransfer::Refined(AppliedFlowRefinement {
                        binding,
                        prior: prior.clone(),
                        refined,
                    })
                } else if let crate::types::store::TypeData::Union(members) = store.get(prior_ty).clone() {
                    let matched: Vec<TypeId> = members
                        .iter()
                        .copied()
                        .filter(|&member| relation::is_subtype(store, hierarchy, member, target))
                        .collect();
                    if !matched.is_empty() {
                        let refined_ty = store.union(&matched);
                        let refined = TypeKnowledge::established(refined_ty, EvidenceOrigin::Flow);
                        if refined != *prior {
                            state.assign(binding, refined.clone());
                            PredicateTransfer::Refined(AppliedFlowRefinement {
                                binding,
                                prior: prior.clone(),
                                refined,
                            })
                        } else {
                            PredicateTransfer::Unchanged
                        }
                    } else {
                        PredicateTransfer::Contradiction { binding, prior: prior.clone() }
                    }
                } else {
                    PredicateTransfer::Contradiction { binding, prior: prior.clone() }
                }
            } else {
                // Assumed prior
                if authority == PredicateAuthority::AuthoritativeObservation {
                    if relation::is_subtype(store, hierarchy, target, prior_ty) {
                        let refined = TypeKnowledge::established(target, EvidenceOrigin::Flow);
                        state.assign(binding, refined.clone());
                        PredicateTransfer::Refined(AppliedFlowRefinement {
                            binding,
                            prior: prior.clone(),
                            refined,
                        })
                    } else if relation::is_subtype(store, hierarchy, prior_ty, target) {
                        // Narrower union/type retained because of assumption -> remains Assumed
                        PredicateTransfer::Unchanged
                    } else if let crate::types::store::TypeData::Union(members) = store.get(prior_ty).clone() {
                        let matched: Vec<TypeId> = members
                            .iter()
                            .copied()
                            .filter(|&member| relation::is_subtype(store, hierarchy, member, target))
                            .collect();
                        if matched.len() == 1 && matched[0] == target {
                            let refined = TypeKnowledge::established(target, EvidenceOrigin::Flow);
                            state.assign(binding, refined.clone());
                            PredicateTransfer::Refined(AppliedFlowRefinement {
                                binding,
                                prior: prior.clone(),
                                refined,
                            })
                        } else if !matched.is_empty() {
                            let refined_ty = store.union(&matched);
                            let refined = prior.derive_known_type(refined_ty, EvidenceOrigin::Flow);
                            if refined != *prior {
                                state.assign(binding, refined.clone());
                                PredicateTransfer::Refined(AppliedFlowRefinement {
                                    binding,
                                    prior: prior.clone(),
                                    refined,
                                })
                            } else {
                                PredicateTransfer::Unchanged
                            }
                        } else {
                            let refined = TypeKnowledge::established(target, EvidenceOrigin::Flow);
                            state.assign(binding, refined.clone());
                            PredicateTransfer::Refined(AppliedFlowRefinement {
                                binding,
                                prior: prior.clone(),
                                refined,
                            })
                        }
                    } else {
                        let refined = TypeKnowledge::established(target, EvidenceOrigin::Flow);
                        state.assign(binding, refined.clone());
                        PredicateTransfer::Refined(AppliedFlowRefinement {
                            binding,
                            prior: prior.clone(),
                            refined,
                        })
                    }
                } else {
                    if relation::is_subtype(store, hierarchy, target, prior_ty) {
                        let refined = prior.derive_known_type(target, EvidenceOrigin::Flow);
                        if refined != *prior {
                            state.assign(binding, refined.clone());
                            PredicateTransfer::Refined(AppliedFlowRefinement {
                                binding,
                                prior: prior.clone(),
                                refined,
                            })
                        } else {
                            PredicateTransfer::Unchanged
                        }
                    } else {
                        PredicateTransfer::Unchanged
                    }
                }
            }
        }
    }
}

fn negative_type_refinement(
    state: &mut FlowState,
    binding: BindingId,
    prior: &TypeKnowledge,
    target: TypeId,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> PredicateTransfer {
    let TypeKnowledge::Known(evidence) = prior else {
        return PredicateTransfer::Unchanged;
    };
    let prior_ty = evidence.ty();
    let prior_status = evidence.status();

    if let crate::types::store::TypeData::Union(members) = store.get(prior_ty).clone() {
        let remaining: Vec<TypeId> = members
            .iter()
            .copied()
            .filter(|&member| !relation::is_subtype(store, hierarchy, member, target))
            .collect();
        if remaining.is_empty() {
            if prior_status == EvidenceStatus::Established {
                PredicateTransfer::Contradiction { binding, prior: prior.clone() }
            } else {
                let refined = TypeKnowledge::Unknown(UnknownReason::InferenceConflict);
                state.assign(binding, refined.clone());
                PredicateTransfer::Refined(AppliedFlowRefinement {
                    binding,
                    prior: prior.clone(),
                    refined,
                })
            }
        } else if remaining.len() < members.len() {
            let refined_ty = store.union(&remaining);
            let refined = prior.derive_known_type(refined_ty, EvidenceOrigin::Flow);
            state.assign(binding, refined.clone());
            PredicateTransfer::Refined(AppliedFlowRefinement {
                binding,
                prior: prior.clone(),
                refined,
            })
        } else {
            PredicateTransfer::Unchanged
        }
    } else if relation::is_subtype(store, hierarchy, prior_ty, target) {
        if prior_status == EvidenceStatus::Established {
            PredicateTransfer::Contradiction { binding, prior: prior.clone() }
        } else {
            let refined = TypeKnowledge::Unknown(UnknownReason::InferenceConflict);
            state.assign(binding, refined.clone());
            PredicateTransfer::Refined(AppliedFlowRefinement {
                binding,
                prior: prior.clone(),
                refined,
            })
        }
    } else {
        PredicateTransfer::Unchanged
    }
}

fn positive_nil_refinement(
    state: &mut FlowState,
    binding: BindingId,
    prior: &TypeKnowledge,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> PredicateTransfer {
    let unit_ty = store.unit();
    positive_type_refinement(state, binding, prior, unit_ty, PredicateAuthority::AuthoritativeObservation, store, hierarchy)
}

fn negative_nil_refinement(
    state: &mut FlowState,
    binding: BindingId,
    prior: &TypeKnowledge,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> PredicateTransfer {
    let unit_ty = store.unit();
    negative_type_refinement(state, binding, prior, unit_ty, store, hierarchy)
}

fn positive_equal_refinement(
    state: &mut FlowState,
    binding: BindingId,
    prior: &TypeKnowledge,
    target: TypeId,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> PredicateTransfer {
    positive_type_refinement(state, binding, prior, target, PredicateAuthority::DerivedFilter, store, hierarchy)
}

fn negative_equal_refinement(
    state: &mut FlowState,
    binding: BindingId,
    prior: &TypeKnowledge,
    target: TypeId,
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
) -> PredicateTransfer {
    negative_type_refinement(state, binding, prior, target, store, hierarchy)
}
