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
pub use query::{
    CallableBodyQuery, DeclarationSurfaceQuery, FormalQueryInputs, bootstrap_advisory_callable, query_advisory_callable, query_advisory_module,
    query_callable_body, query_callable_body_with_formal_inputs, query_callable_signature, query_declaration_shell, query_declaration_surface,
    query_hierarchy_edge, query_signatureless_callable_body, query_source_formal_attachment, query_source_structure,
};
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
        if state.is_ready() { self.products.get(key) } else { None }
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

    /// Checks whether a stored query product is eligible for reuse.
    ///
    /// Reuse requires an unchanged direct input plus dependency products that
    /// have already been validated in the current semantic revision. The
    /// dependency's computation revision is deliberately irrelevant: an older
    /// product remains reusable after current-revision validation if its semantic
    /// product fingerprint is unchanged.
    pub fn is_reusable(&self, key: &QueryKey, input_fingerprint: InputFingerprint) -> bool {
        let Some(state) = self.query_states.get(key) else {
            return false;
        };
        let QueryState::Ready {
            input_fingerprint: stored_input_fp,
            ..
        } = state
        else {
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
                validated_revision,
                product_fingerprint: dep_prod_fp,
                ..
            } = dep_state
            else {
                return false;
            };
            if *validated_revision != self.revision || *dep_prod_fp != edge.observed_fingerprint {
                return false;
            }
        }
        true
    }

    /// Validates a cached query for the current revision and marks it reusable.
    ///
    /// This advances only the query's validation revision. The original
    /// computation revision and product fingerprint remain unchanged.
    pub fn validate_reuse(&mut self, key: &QueryKey, input_fingerprint: InputFingerprint) -> bool {
        if !self.is_reusable(key, input_fingerprint) {
            return false;
        }

        let current_revision = self.revision;
        let Some(QueryState::Ready { validated_revision, .. }) = self.query_states.get_mut(key) else {
            return false;
        };
        *validated_revision = current_revision;
        true
    }

    /// Records an edge to a dependency validated for the current revision.
    ///
    /// A stored `Ready` product from an older revision is insufficient until its
    /// own query has been revalidated. This prevents a dependent from observing
    /// stale transitive state merely because an old product fingerprint is still
    /// present in the cache.
    pub fn record_dependency(&self, recorder: &mut DependencyRecorder, dependency: QueryKey) -> Result<(), String> {
        let Some(state) = self.query_states.get(&dependency) else {
            return Err(format!("query dependency {:?} is not Ready", dependency));
        };
        let QueryState::Ready {
            validated_revision,
            product_fingerprint,
            ..
        } = state
        else {
            return Err(format!("query dependency {:?} is not Ready", dependency));
        };
        if *validated_revision != self.revision {
            return Err(format!(
                "query dependency {:?} is Ready but not validated for current revision {:?}",
                dependency, self.revision
            ));
        }

        recorder.record(dependency, *product_fingerprint);
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
                validated_revision: revision,
                input_fingerprint,
                product_fingerprint,
                value,
            },
        );
        self.products.remove(&key);
        self.metrics.record_hit();
        Ok(())
    }

    /// Discards one query so it can be recomputed without eagerly deleting dependents.
    ///
    /// Incoming reverse-dependency edges are intentionally preserved. Dependents retain
    /// their previous ready products until they are queried, at which point generic
    /// dependency-fingerprint validation decides whether they can be revalidated or
    /// must recompute. This is the product-stability propagation primitive: if this
    /// query republishes the same product fingerprint, downstream queries can reuse
    /// their existing products even though this query was recomputed in a newer revision.
    ///
    /// The last-known-good product is also preserved for cancellation/failure fallback.
    pub fn discard_for_recompute(&mut self, key: &QueryKey) -> bool {
        let had_state = self.query_states.remove(key).is_some();
        let had_product = self.products.remove(key).is_some();
        self.index.remove_dependencies(key);
        if had_state || had_product {
            self.metrics.record_invalidation();
            true
        } else {
            false
        }
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
        self.publish_ready(key.clone(), revision, input_fingerprint, product_fingerprint, query_value, dependencies)?;
        self.products.insert(key.clone(), product);
        if let Some(product) = self.products.get(&key) {
            self.last_known_good.insert(key, product.clone());
        }
        Ok(())
    }

    pub fn update_callable_body_product(&mut self, callable: &crate::identity::CallableId, analysis: Arc<crate::checker::CallableAnalysis>) {
        let key = QueryKey::CallableBody(callable.clone());
        let fp = analysis.dependency_fingerprint;
        let query_value = crate::db::product::SemanticProduct::CallableBody(analysis.clone()).to_query_value();
        if let Some(QueryState::Ready {
            product_fingerprint, value, ..
        }) = self.query_states.get_mut(&key)
        {
            *product_fingerprint = fp;
            *value = query_value;
        }
        let product = Arc::new(SemanticProduct::CallableBody(analysis));
        self.products.insert(key.clone(), product.clone());
        self.last_known_good.insert(key, product);
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

    /// Invalidates only supplied query roots, preserving cached dependents until
    /// the refreshed root product proves whether their fingerprints changed.
    pub fn invalidate_roots(&mut self, roots: impl IntoIterator<Item = QueryKey>) -> BTreeSet<QueryKey> {
        let roots: BTreeSet<_> = roots.into_iter().collect();
        for key in &roots {
            self.query_states.remove(key);
            self.products.remove(key);
            self.index.remove_dependencies(key);
            self.metrics.record_invalidation();
        }
        roots
    }
}
