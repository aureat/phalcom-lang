//! Backward causal slicing over explanation graphs (Spec 04.5).

use super::arena::ExplanationArena;
use super::node::ExplanationNode;
use crate::identity::ExplanationId;
use std::collections::{BTreeSet, VecDeque};

/// Extracts the backward causal slice of an explanation DAG starting from `root`.
pub fn causal_slice(arena: &ExplanationArena, root: ExplanationId) -> Vec<&ExplanationNode> {
    let mut visited = BTreeSet::new();
    let mut queue = VecDeque::new();
    let mut result = Vec::new();

    queue.push_back(root);
    visited.insert(root);

    while let Some(current_id) = queue.pop_front() {
        if let Some(node) = arena.get(current_id) {
            result.push(node);
            for parent in &node.parents {
                if visited.insert(*parent) {
                    queue.push_back(*parent);
                }
            }
        }
    }

    result
}
