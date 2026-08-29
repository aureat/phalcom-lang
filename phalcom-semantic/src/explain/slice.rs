//! Backward causal slicing over explanation graphs (Spec 04.5).

use super::arena::ExplanationArena;
use super::node::ExplanationNode;
use crate::identity::ExplanationId;
use std::collections::{BTreeSet, VecDeque};

/// Extracts the backward causal slice of an explanation DAG starting from `root`.
///
/// This compatibility view keeps the historical root-first breadth-first
/// ordering. User-facing deterministic traces should use [`causal_trace`].
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

/// Returns a deterministic causal trace with dependencies before dependants.
///
/// Shared ancestors are emitted once, parent traversal follows the stable
/// parent order stored on each node, the root is last, and malformed cycles
/// are cut by the visited set instead of recursing indefinitely.
pub fn causal_trace(arena: &ExplanationArena, root: ExplanationId) -> Vec<&ExplanationNode> {
    fn visit<'a>(
        arena: &'a ExplanationArena,
        id: ExplanationId,
        visiting: &mut BTreeSet<ExplanationId>,
        visited: &mut BTreeSet<ExplanationId>,
        result: &mut Vec<&'a ExplanationNode>,
    ) {
        if visited.contains(&id) || !visiting.insert(id) {
            return;
        }

        if let Some(node) = arena.get(id) {
            for parent in &node.parents {
                visit(arena, *parent, visiting, visited, result);
            }
            if visited.insert(id) {
                result.push(node);
            }
        }

        visiting.remove(&id);
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    let mut result = Vec::new();
    visit(arena, root, &mut visiting, &mut visited, &mut result);
    result
}
