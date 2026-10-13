//! Dynamic query dependency recording and reverse invalidation indexing.

use crate::db::key::{ProductFingerprint, QueryKey};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// A recorded dependency edge between a dependent query and a query it consumed.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DependencyEdge {
    pub dependent: QueryKey,
    pub dependency: QueryKey,
    pub observed_fingerprint: ProductFingerprint,
}

/// Helper for recording consumed dependencies while executing a query.
#[derive(Clone, Debug)]
pub struct DependencyRecorder {
    dependent: QueryKey,
    edges: Vec<DependencyEdge>,
}

impl DependencyRecorder {
    pub fn new(dependent: QueryKey) -> Self {
        Self { dependent, edges: Vec::new() }
    }

    pub fn record(&mut self, dependency: QueryKey, observed_fingerprint: ProductFingerprint) {
        self.edges.push(DependencyEdge {
            dependent: self.dependent.clone(),
            dependency,
            observed_fingerprint,
        });
    }

    pub fn finish(self) -> Vec<DependencyEdge> {
        self.edges
    }
}

/// Two-way index tracking query dependencies and reverse invalidation edges.
#[derive(Clone, Debug, Default)]
pub struct DependencyIndex {
    forward: BTreeMap<QueryKey, Vec<DependencyEdge>>,
    reverse: BTreeMap<QueryKey, BTreeSet<QueryKey>>,
    transaction: Option<DependencyIndexUndo>,
}

#[derive(Clone, Debug, Default)]
struct DependencyIndexUndo {
    forward: BTreeMap<QueryKey, Option<Vec<DependencyEdge>>>,
    reverse: BTreeMap<QueryKey, Option<BTreeSet<QueryKey>>>,
}

impl DependencyIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn begin_transaction(&mut self) {
        assert!(self.transaction.is_none(), "dependency-index transactions cannot nest");
        self.transaction = Some(DependencyIndexUndo::default());
    }

    pub(crate) fn commit_transaction(&mut self) {
        self.transaction.take();
    }

    pub(crate) fn rollback_transaction(&mut self) {
        let Some(undo) = self.transaction.take() else {
            return;
        };
        for (key, old) in undo.forward {
            match old {
                Some(edges) => {
                    self.forward.insert(key, edges);
                }
                None => {
                    self.forward.remove(&key);
                }
            }
        }
        for (key, old) in undo.reverse {
            match old {
                Some(dependents) => {
                    self.reverse.insert(key, dependents);
                }
                None => {
                    self.reverse.remove(&key);
                }
            }
        }
    }

    fn remember_forward(&mut self, key: &QueryKey) {
        let Some(transaction) = &mut self.transaction else {
            return;
        };
        if transaction.forward.contains_key(key) {
            return;
        }
        transaction.forward.insert(key.clone(), self.forward.get(key).cloned());
        #[cfg(feature = "perf-observability")]
        {
            use telos_observability::{PerfContext, PerfCounter};
            let context = PerfContext::current_thread();
            context.record_counter(PerfCounter::SemanticDbUndoDependencyEntries, 1);
            context.record_counter(PerfCounter::SemanticDbUndoDependencyForwardEntries, 1);
        }
    }

    fn remember_reverse(&mut self, key: &QueryKey) {
        let Some(transaction) = &mut self.transaction else {
            return;
        };
        if transaction.reverse.contains_key(key) {
            return;
        }
        transaction.reverse.insert(key.clone(), self.reverse.get(key).cloned());
        #[cfg(feature = "perf-observability")]
        {
            use telos_observability::{PerfContext, PerfCounter};
            let context = PerfContext::current_thread();
            context.record_counter(PerfCounter::SemanticDbUndoDependencyEntries, 1);
            context.record_counter(PerfCounter::SemanticDbUndoDependencyReverseEntries, 1);
        }
    }

    pub fn replace_dependencies(&mut self, key: QueryKey, dependencies: impl IntoIterator<Item = DependencyEdge>) {
        self.replace_dependencies_with_leaves(key, dependencies, |_| false);
    }

    /// Retains forward observations of immutable roots without reverse edges.
    pub(crate) fn replace_dependencies_with_leaves(
        &mut self,
        key: QueryKey,
        dependencies: impl IntoIterator<Item = DependencyEdge>,
        is_immutable: impl Fn(&QueryKey) -> bool,
    ) {
        let edges: Vec<DependencyEdge> = dependencies.into_iter().collect();
        let mut reverse_keys = self
            .forward
            .get(&key)
            .into_iter()
            .flat_map(|old_edges| {
                old_edges
                    .iter()
                    .filter(|edge| !is_immutable(&edge.dependency))
                    .map(|edge| edge.dependency.clone())
            })
            .collect::<BTreeSet<_>>();
        reverse_keys.extend(edges.iter().filter(|edge| !is_immutable(&edge.dependency)).map(|edge| edge.dependency.clone()));
        self.remember_forward(&key);
        for dependency in reverse_keys {
            self.remember_reverse(&dependency);
        }
        if let Some(old_edges) = self.forward.remove(&key) {
            for edge in old_edges {
                if let Some(dependents) = self.reverse.get_mut(&edge.dependency) {
                    dependents.remove(&key);
                }
            }
        }

        for edge in &edges {
            if is_immutable(&edge.dependency) {
                continue;
            }
            self.reverse.entry(edge.dependency.clone()).or_default().insert(key.clone());
        }
        self.forward.insert(key, edges);
    }

    pub fn remove_dependencies(&mut self, key: &QueryKey) {
        let dependencies = self
            .forward
            .get(key)
            .map(|edges| edges.iter().map(|edge| edge.dependency.clone()).collect::<BTreeSet<_>>())
            .unwrap_or_default();
        self.remember_forward(key);
        for dependency in dependencies {
            // Immutable leaves have no reverse row to mutate or journal.
            if self.reverse.contains_key(&dependency) {
                self.remember_reverse(&dependency);
            }
        }
        if let Some(old_edges) = self.forward.remove(key) {
            for edge in old_edges {
                if let Some(dependents) = self.reverse.get_mut(&edge.dependency) {
                    dependents.remove(key);
                }
            }
        }
    }

    /// Removes obsolete query identities from both dependency directions.
    pub fn purge_keys(&mut self, keys: &BTreeSet<QueryKey>) {
        for key in keys {
            self.remove_dependencies(key);
        }
        let reverse_rows_to_change = self
            .reverse
            .iter()
            .filter(|(dependency, dependents)| keys.contains(*dependency) || dependents.iter().any(|dependent| keys.contains(dependent)))
            .map(|(dependency, _)| dependency.clone())
            .collect::<Vec<_>>();
        for dependency in reverse_rows_to_change {
            self.remember_reverse(&dependency);
        }
        self.reverse.retain(|dependency, dependents| {
            if keys.contains(dependency) {
                return false;
            }
            dependents.retain(|dependent| !keys.contains(dependent));
            !dependents.is_empty()
        });
    }

    pub fn dependencies_of(&self, key: &QueryKey) -> Option<&[DependencyEdge]> {
        self.forward.get(key).map(|v| v.as_slice())
    }

    pub fn dependents_of(&self, key: &QueryKey) -> Option<&BTreeSet<QueryKey>> {
        self.reverse.get(key)
    }

    /// Computes the deterministic transitive reverse closure of invalidation seeds.
    pub fn reverse_closure(&self, seeds: impl IntoIterator<Item = QueryKey>) -> BTreeSet<QueryKey> {
        let mut closure = BTreeSet::new();
        let mut worklist: VecDeque<QueryKey> = seeds.into_iter().collect();
        let seed_count = worklist.len() as u64;
        let mut nodes_visited = 0u64;
        let mut edges_considered = 0u64;

        while let Some(current) = worklist.pop_front() {
            if closure.insert(current.clone()) {
                nodes_visited = nodes_visited.saturating_add(1);
                if let Some(dependents) = self.reverse.get(&current) {
                    edges_considered = edges_considered.saturating_add(dependents.len() as u64);
                    for dep in dependents {
                        if !closure.contains(dep) {
                            worklist.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        #[cfg(feature = "perf-observability")]
        {
            use telos_observability::{PerfContext, PerfCounter};
            let context = PerfContext::current_thread();
            context.record_counter(PerfCounter::SemanticReverseClosureSeeds, seed_count);
            context.record_counter(PerfCounter::SemanticReverseClosureNodesVisited, nodes_visited);
            context.record_counter(PerfCounter::SemanticReverseClosureEdgesConsidered, edges_considered);
        }
        #[cfg(not(feature = "perf-observability"))]
        let _ = (seed_count, nodes_visited, edges_considered);

        closure
    }
}
