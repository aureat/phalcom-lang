//! Compiler-owned staged semantic database with dynamic dependencies,
//! deterministic scheduling, cancellation, and immutable snapshots.

pub mod budget;
pub mod dependency;
pub mod fingerprint;
pub mod key;
pub mod metrics;
pub mod product;
pub mod query;
pub mod scheduler;
pub mod state;

pub use budget::{BudgetKind, BudgetReport, CancellationToken, QueryBudget};
pub use dependency::{DependencyEdge, DependencyIndex, DependencyRecorder};
pub use key::{InputFingerprint, ProductFingerprint, QueryKey};
pub use metrics::QueryMetrics;
pub use product::SemanticProduct;
pub use query::query_callable_body;
pub use scheduler::QueryScheduler;
pub use state::{PublishError, QueryOutcome, QueryState, QueryValue};

use crate::identity::{SemanticRevision, WorkspaceId};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Compiler-owned staged semantic database for incremental analysis.
#[derive(Debug)]
pub struct SemanticDb {
    workspace: WorkspaceId,
    revision: SemanticRevision,
    query_states: BTreeMap<QueryKey, QueryState>,
    products: BTreeMap<QueryKey, Arc<SemanticProduct>>,
    last_known_good: BTreeMap<QueryKey, Arc<SemanticProduct>>,
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
            query_states: BTreeMap::new(),
            products: BTreeMap::new(),
            last_known_good: BTreeMap::new(),
            index: DependencyIndex::new(),
            scheduler: QueryScheduler::new(),
            metrics: Arc::new(QueryMetrics::new()),
        }
    }

    pub fn with_workspace(workspace: WorkspaceId) -> Self {
        Self {
            workspace,
            revision: SemanticRevision::from_raw(1),
            query_states: BTreeMap::new(),
            products: BTreeMap::new(),
            last_known_good: BTreeMap::new(),
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

    /// Returns the typed product published for a ready query.
    pub fn product(&self, key: &QueryKey) -> Option<&Arc<SemanticProduct>> {
        let state = self.query_states.get(key)?;
        if state.is_ready() {
            self.products.get(key)
        } else {
            None
        }
    }

    /// Returns the last typed product that reached `Ready` for this key.
    ///
    /// This intentionally differs from [`Self::product`]: a failed or
    /// cancelled refresh must not replace a published good product, while
    /// current-generation query consumers still need an explicit way to ask
    /// for that last-known-good value.
    pub fn last_known_good_product(&self, key: &QueryKey) -> Option<&Arc<SemanticProduct>> {
        self.last_known_good.get(key)
    }

    pub fn set_state(&mut self, key: QueryKey, state: QueryState) {
        if !state.is_ready() {
            self.products.remove(&key);
        }
        self.query_states.insert(key, state);
    }

    /// Returns the recorded product fingerprint for a query if it is in the `Ready` state.
    pub fn ready_product_fingerprint(&self, key: &QueryKey) -> Option<ProductFingerprint> {
        self.query_states.get(key).and_then(|s| s.product_fingerprint())
    }

    /// Validates whether a query result is structurally reusable:
    /// 1. Query must be in `Ready` state.
    /// 2. Stored `input_fingerprint` must equal the requested `input_fingerprint`.
    /// 3. Every recorded dependency edge must currently be in `Ready` state with
    ///    its current `product_fingerprint` equal to the `observed_fingerprint` when the edge was recorded.
    pub fn is_reusable(&self, key: &QueryKey, input_fingerprint: InputFingerprint) -> bool {
        let Some(state) = self.query_states.get(key) else {
            return false;
        };
        let QueryState::Ready {
            input_fingerprint: stored_input_fp,
            ..
        } = state else {
            return false;
        };
        if *stored_input_fp != input_fingerprint {
            return false;
        }
        let Some(deps) = self.index.dependencies_of(key) else {
            return true;
        };
        for edge in deps {
            let Some(dep_state) = self.query_states.get(&edge.dependency) else {
                return false;
            };
            let QueryState::Ready {
                product_fingerprint: dep_prod_fp,
                ..
            } = dep_state else {
                return false;
            };
            if *dep_prod_fp != edge.observed_fingerprint {
                return false;
            }
        }
        true
    }

    /// Records a dependency edge using the current `Ready` product fingerprint of the dependency.
    pub fn record_dependency(
        &self,
        recorder: &mut DependencyRecorder,
        dependency: QueryKey,
    ) -> Result<(), String> {
        let Some(fp) = self.ready_product_fingerprint(&dependency) else {
            return Err(format!("query dependency {:?} is not Ready", dependency));
        };
        recorder.record(dependency, fp);
        Ok(())
    }

    pub fn publish_ready(
        &mut self,
        key: QueryKey,
        revision: SemanticRevision,
        input_fingerprint: InputFingerprint,
        product_fingerprint: ProductFingerprint,
        value: QueryValue,
        dependencies: impl IntoIterator<Item = DependencyEdge>,
    ) -> Result<(), PublishError> {
        if revision != self.revision {
            return Err(PublishError::stale(self.revision, revision));
        }

        self.index.replace_dependencies(key.clone(), dependencies);
        self.query_states.insert(
            key.clone(),
            QueryState::Ready {
                revision,
                input_fingerprint,
                product_fingerprint,
                value,
            },
        );
        self.products.remove(&key);
        self.metrics.record_hit();
        Ok(())
    }

    /// Publishes a lossless typed product alongside its query state.
    pub fn publish_product_ready(
        &mut self,
        key: QueryKey,
        revision: SemanticRevision,
        input_fingerprint: InputFingerprint,
        product_fingerprint: ProductFingerprint,
        product: SemanticProduct,
        dependencies: impl IntoIterator<Item = DependencyEdge>,
    ) -> Result<(), PublishError> {
        let query_value = product.to_query_value();
        let product = Arc::new(product);
        self.publish_ready(
            key.clone(),
            revision,
            input_fingerprint,
            product_fingerprint,
            query_value,
            dependencies,
        )?;
        self.products.insert(key.clone(), product);
        if let Some(product) = self.products.get(&key) {
            self.last_known_good.insert(key, product.clone());
        }
        Ok(())
    }

    pub fn invalidate(&mut self, seeds: impl IntoIterator<Item = QueryKey>) -> BTreeSet<QueryKey> {
        let closure = self.index.reverse_closure(seeds);
        for key in &closure {
            self.query_states.remove(key);
            self.products.remove(key);
            self.index.remove_dependencies(key);
            self.metrics.record_invalidation();
        }
        closure
    }
}
