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
}

impl DependencyIndex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_dependencies(&mut self, key: QueryKey, dependencies: impl IntoIterator<Item = DependencyEdge>) {
        if let Some(old_edges) = self.forward.remove(&key) {
            for edge in old_edges {
                if let Some(dependents) = self.reverse.get_mut(&edge.dependency) {
                    dependents.remove(&key);
                }
            }
        }

        let edges: Vec<DependencyEdge> = dependencies.into_iter().collect();
        for edge in &edges {
            self.reverse.entry(edge.dependency.clone()).or_default().insert(key.clone());
        }
        self.forward.insert(key, edges);
    }

    pub fn remove_dependencies(&mut self, key: &QueryKey) {
        if let Some(old_edges) = self.forward.remove(key) {
            for edge in old_edges {
                if let Some(dependents) = self.reverse.get_mut(&edge.dependency) {
                    dependents.remove(key);
                }
            }
        }
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

        while let Some(current) = worklist.pop_front() {
            if closure.insert(current.clone()) {
                if let Some(dependents) = self.reverse.get(&current) {
                    for dep in dependents {
                        if !closure.contains(dep) {
                            worklist.push_back(dep.clone());
                        }
                    }
                }
            }
        }

        closure
    }
}
