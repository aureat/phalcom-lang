//! Arena for interning and managing explanation nodes (Spec 04.5).

use super::node::{ExplanationNode, ExplanationStep};
use crate::identity::ExplanationId;
use crate::types::evidence::{EvidenceOrigin, EvidenceStatus};
use phalcom_type_meta::allocation::{AllocationBudget, AllocationError, BudgetedSortedMap, SortedMapEntriesError};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Immutable-map construction failure for an imported explanation arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExplanationArenaBuildError {
    /// The entry buffer was not strictly ordered by explanation identity.
    Entries(SortedMapEntriesError),
    /// Publishing the shared immutable backing exceeded the load budget.
    Allocation(AllocationError),
}

/// A failed attempt to add a node to an imported immutable explanation arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrozenExplanationArenaMutation;

#[derive(Clone, Debug)]
enum ExplanationArenaStorage {
    Source {
        nodes: BTreeMap<ExplanationId, ExplanationNode>,
        next_id: u32,
    },
    Frozen(Arc<BudgetedSortedMap<ExplanationId, ExplanationNode>>),
}

/// Arena holding explanation DAG nodes for a callable body.
#[derive(Clone, Debug)]
pub struct ExplanationArena {
    storage: ExplanationArenaStorage,
}

impl Default for ExplanationArena {
    fn default() -> Self {
        Self::new()
    }
}

impl ExplanationArena {
    /// Creates an empty mutable source-side explanation arena.
    pub fn new() -> Self {
        Self {
            storage: ExplanationArenaStorage::Source {
                nodes: BTreeMap::new(),
                next_id: 0,
            },
        }
    }

    /// Takes ownership of a pre-budgeted ordered node buffer and charges the
    /// shared frozen-map owner to the same load transaction.
    pub fn from_budgeted_entries(
        entries: Vec<(ExplanationId, ExplanationNode)>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, ExplanationArenaBuildError> {
        let nodes = BudgetedSortedMap::from_sorted_entries(entries).map_err(ExplanationArenaBuildError::Entries)?;
        let nodes = allocations.arc(nodes).map_err(ExplanationArenaBuildError::Allocation)?;
        Ok(Self {
            storage: ExplanationArenaStorage::Frozen(nodes),
        })
    }

    /// Returns whether this arena uses immutable imported backing.
    pub fn is_frozen(&self) -> bool {
        matches!(self.storage, ExplanationArenaStorage::Frozen(_))
    }

    /// Allocates a source-side explanation node with its derivation rule
    /// inferred from `step`.
    pub fn alloc(
        &mut self,
        step: ExplanationStep,
        status: EvidenceStatus,
        origin: EvidenceOrigin,
        parents: Vec<ExplanationId>,
    ) -> Result<ExplanationId, FrozenExplanationArenaMutation> {
        let rule = step.derivation_rule();
        self.alloc_full(step, rule, status, origin, Vec::new(), parents)
    }

    /// Allocates a source-side explanation node with complete derivation and
    /// evidence payload.
    pub fn alloc_full(
        &mut self,
        step: ExplanationStep,
        rule: super::node::DerivationRule,
        status: EvidenceStatus,
        origin: EvidenceOrigin,
        evidence: Vec<super::node::EvidenceRef>,
        parents: Vec<ExplanationId>,
    ) -> Result<ExplanationId, FrozenExplanationArenaMutation> {
        let ExplanationArenaStorage::Source { nodes, next_id } = &mut self.storage else {
            return Err(FrozenExplanationArenaMutation);
        };
        let id = ExplanationId(*next_id);
        *next_id += 1;
        nodes.insert(
            id,
            ExplanationNode {
                id,
                step,
                rule,
                status,
                origin,
                evidence,
                parents,
            },
        );
        Ok(id)
    }

    /// Looks up one explanation node by its callable-local identity.
    pub fn get(&self, id: ExplanationId) -> Option<&ExplanationNode> {
        match &self.storage {
            ExplanationArenaStorage::Source { nodes, .. } => nodes.get(&id),
            ExplanationArenaStorage::Frozen(nodes) => nodes.get(&id),
        }
    }

    /// Iterates nodes in stable local-identity order.
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (&ExplanationId, &ExplanationNode)> {
        match &self.storage {
            ExplanationArenaStorage::Source { nodes, .. } => ExplanationArenaIter::Source(nodes.iter()),
            ExplanationArenaStorage::Frozen(nodes) => ExplanationArenaIter::Frozen(nodes.as_entries().iter()),
        }
    }

    /// Returns the number of explanation nodes.
    pub fn len(&self) -> usize {
        match &self.storage {
            ExplanationArenaStorage::Source { nodes, .. } => nodes.len(),
            ExplanationArenaStorage::Frozen(nodes) => nodes.len(),
        }
    }

    /// Returns whether the arena contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

enum ExplanationArenaIter<'a> {
    Source(std::collections::btree_map::Iter<'a, ExplanationId, ExplanationNode>),
    Frozen(std::slice::Iter<'a, (ExplanationId, ExplanationNode)>),
}

impl<'a> Iterator for ExplanationArenaIter<'a> {
    type Item = (&'a ExplanationId, &'a ExplanationNode);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(id, node)| (id, node)),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for ExplanationArenaIter<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Source(iter) => iter.len(),
            Self::Frozen(iter) => iter.len(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ExplanationArena, ExplanationArenaBuildError};
    use crate::explain::node::{DerivationRule, ExplanationNode, ExplanationStep};
    use crate::identity::ExplanationId;
    use crate::types::evidence::{EvidenceOrigin, EvidenceStatus};
    use crate::types::id::TypeId;
    use phalcom_type_meta::allocation::{AllocationBudget, AllocationError};

    fn node(id: ExplanationId) -> ExplanationNode {
        ExplanationNode {
            id,
            step: ExplanationStep::Subtyping {
                actual: TypeId(1),
                expected: TypeId(2),
                proven: true,
            },
            rule: DerivationRule::TypeRelation,
            status: EvidenceStatus::Established,
            origin: EvidenceOrigin::Flow,
            evidence: Vec::new(),
            parents: Vec::new(),
        }
    }

    #[test]
    fn source_and_frozen_explanation_arenas_share_ordered_read_behavior() {
        let mut source = ExplanationArena::new();
        let first = source
            .alloc_full(
                node(ExplanationId(0)).step,
                DerivationRule::TypeRelation,
                EvidenceStatus::Established,
                EvidenceOrigin::Flow,
                Vec::new(),
                Vec::new(),
            )
            .unwrap();
        let second = source
            .alloc_full(
                node(ExplanationId(1)).step,
                DerivationRule::TypeRelation,
                EvidenceStatus::Assumed,
                EvidenceOrigin::Syntax,
                Vec::new(),
                vec![first],
            )
            .unwrap();
        let expected = source.iter().map(|(id, node)| (*id, node.clone())).collect::<Vec<_>>();

        let mut allocations = AllocationBudget::new(4096);
        let mut entries = allocations.vector(expected.len()).unwrap();
        entries.extend(expected.clone());
        let frozen = ExplanationArena::from_budgeted_entries(entries, &mut allocations).unwrap();

        assert_eq!(source.iter().map(|(id, node)| (*id, node.clone())).collect::<Vec<_>>(), expected);
        assert_eq!(frozen.iter().map(|(id, node)| (*id, node.clone())).collect::<Vec<_>>(), expected);
        assert_eq!(frozen.get(first), source.get(first));
        assert_eq!(frozen.get(second), source.get(second));
        assert_eq!(frozen.len(), 2);
        assert!(frozen.is_frozen());
        let mut frozen = frozen;
        assert!(
            frozen
                .alloc(
                    ExplanationStep::Subtyping {
                        actual: TypeId(3),
                        expected: TypeId(4),
                        proven: false
                    },
                    EvidenceStatus::Assumed,
                    EvidenceOrigin::Flow,
                    Vec::new()
                )
                .is_err()
        );
        assert_eq!(frozen.len(), 2);
    }

    #[test]
    fn frozen_explanation_owner_charge_requires_exact_budget() {
        fn build(budget: &mut AllocationBudget) -> Result<ExplanationArena, ExplanationArenaBuildError> {
            let mut entries = budget.vector(1).map_err(ExplanationArenaBuildError::Allocation)?;
            entries.push((ExplanationId(0), node(ExplanationId(0))));
            ExplanationArena::from_budgeted_entries(entries, budget)
        }

        let mut probe = AllocationBudget::new(4096);
        let _arena = build(&mut probe).unwrap();
        let exact = probe.requested();
        let mut fits = AllocationBudget::new(exact);
        assert_eq!(build(&mut fits).unwrap().len(), 1);
        assert_eq!(fits.requested(), exact);

        let mut short = AllocationBudget::new(exact - 1);
        assert!(matches!(
            build(&mut short),
            Err(ExplanationArenaBuildError::Allocation(AllocationError::Exhausted))
        ));
    }
}
