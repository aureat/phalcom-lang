//! Compiler-owned staged semantic database with dynamic dependencies,
//! deterministic scheduling, cancellation, and immutable snapshots.

pub mod budget;
pub mod dependency;
pub mod key;
pub mod metrics;
pub mod product;
pub mod query;
pub mod scheduler;
pub mod state;

pub use budget::{BudgetKind, BudgetReport, CancellationToken, QueryBudget};
pub use dependency::{DependencyEdge, DependencyIndex, DependencyRecorder};
pub use key::{ProductFingerprint, QueryKey};
pub use metrics::QueryMetrics;
pub use product::SemanticProduct;
pub use query::query_callable_body;
pub use scheduler::QueryScheduler;
pub use state::{PublishError, QueryOutcome, QueryState, QueryValue};

use crate::identity::{SemanticRevision, WorkspaceId};
use crate::types::store::TypeStore;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Compiler-owned staged semantic database for incremental analysis.
#[derive(Clone, Debug)]
pub struct SemanticDb {
    workspace: WorkspaceId,
    revision: SemanticRevision,
    store: Arc<TypeStore>,
    query_states: BTreeMap<QueryKey, QueryState>,
    index: DependencyIndex,
    scheduler: QueryScheduler,
    metrics: Arc<QueryMetrics>,
}

impl Default for SemanticDb {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticDb {
    pub fn new() -> Self {
        Self {
            workspace: WorkspaceId::from_raw(1),
            revision: SemanticRevision::from_raw(1),
            store: Arc::new(TypeStore::new()),
            query_states: BTreeMap::new(),
            index: DependencyIndex::new(),
            scheduler: QueryScheduler::new(),
            metrics: Arc::new(QueryMetrics::new()),
        }
    }

    pub fn with_workspace(workspace: WorkspaceId) -> Self {
        Self {
            workspace,
            revision: SemanticRevision::from_raw(1),
            store: Arc::new(TypeStore::new()),
            query_states: BTreeMap::new(),
            index: DependencyIndex::new(),
            scheduler: QueryScheduler::new(),
            metrics: Arc::new(QueryMetrics::new()),
        }
    }

    pub fn workspace(&self) -> WorkspaceId {
        self.workspace
    }

    pub fn revision(&self) -> SemanticRevision {
        self.revision
    }

    pub fn begin_revision(&mut self) -> SemanticRevision {
        self.revision = self.revision.next();
        self.revision
    }

    pub fn store(&self) -> &Arc<TypeStore> {
        &self.store
    }

    pub fn index(&self) -> &DependencyIndex {
        &self.index
    }

    pub fn scheduler_mut(&mut self) -> &mut QueryScheduler {
        &mut self.scheduler
    }

    pub fn metrics(&self) -> &QueryMetrics {
        &self.metrics
    }

    pub fn query_state(&self, key: &QueryKey) -> Option<&QueryState> {
        self.query_states.get(key)
    }

    pub fn set_state(&mut self, key: QueryKey, state: QueryState) {
        self.query_states.insert(key, state);
    }

    pub fn publish_ready(
        &mut self,
        key: QueryKey,
        revision: SemanticRevision,
        fingerprint: ProductFingerprint,
        value: QueryValue,
        dependencies: impl IntoIterator<Item = DependencyEdge>,
    ) -> Result<(), PublishError> {
        if revision != self.revision {
            return Err(PublishError::stale(self.revision, revision));
        }

        self.index.replace_dependencies(key.clone(), dependencies);
        self.query_states.insert(key, QueryState::Ready { revision, fingerprint, value });
        self.metrics.record_hit();
        Ok(())
    }

    pub fn invalidate(&mut self, seeds: impl IntoIterator<Item = QueryKey>) -> BTreeSet<QueryKey> {
        let closure = self.index.reverse_closure(seeds);
        for key in &closure {
            self.query_states.remove(key);
            self.index.remove_dependencies(key);
            self.metrics.record_invalidation();
        }
        closure
    }
}
