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
pub(crate) mod static_fingerprint;

pub use budget::{BudgetKind, BudgetReport, CancellationToken, QueryBudget};
pub use dependency::{DependencyEdge, DependencyIndex, DependencyRecorder};
pub use key::{InputFingerprint, ProductFingerprint, QueryKey, QueryKind};
pub use metrics::QueryMetrics;
pub use product::SemanticProduct;
pub use query::{
    CallableBodyQuery, DeclarationSurfaceQuery, FormalQueryInputs, bootstrap_advisory_callable, query_advisory_callable, query_advisory_module,
    query_attribute_argument, query_bootstrap_trait_header, query_bootstrap_trait_surface, query_callable_body, query_callable_body_with_formal_inputs,
    query_callable_signature, query_declaration_shell, query_declaration_surface, query_hierarchy_edge, query_signatureless_callable_body,
    query_source_formal_attachment, query_source_structure, query_type_member_declaration,
};
pub use scheduler::QueryScheduler;
pub use state::{PublishError, QueryOutcome, QueryState, QueryValue};

use crate::identity::{ModuleId, SemanticRevision, WorkspaceId};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
#[cfg(feature = "perf-observability")]
use std::time::Instant;

#[derive(Clone, Copy)]
enum QueryLifecycleEvent {
    Request,
    DirectReuse,
    RevalidatedReuse,
    Recomputed,
    Invalidated,
    Blocked,
    Cancelled,
    BudgetExceeded,
    Failed,
}

#[cfg(feature = "perf-observability")]
fn record_semantic_db_counter(counter: telos_observability::PerfCounter, amount: usize) {
    telos_observability::PerfContext::current_thread().record_counter(counter, amount as u64);
}

/// Authority/lifetime domain of a typed query read.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SemanticProductOrigin {
    /// Immutable closed canonical world, accessible through its separate reader.
    Canonical(crate::canonical_image::CanonicalSemanticWorldId),
    /// Mutable workspace database; ordinary revision validation still applies.
    Workspace(WorkspaceId),
}

/// Compiler-owned staged semantic database for incremental analysis.
#[derive(Debug)]
pub struct SemanticDb {
    workspace: WorkspaceId,
    canonical: Option<Arc<crate::canonical_image::CanonicalUniverseFormalImage>>,
    revision: SemanticRevision,
    query_states: BTreeMap<QueryKey, QueryState>,
    query_keys_by_module: BTreeMap<ModuleId, BTreeSet<QueryKey>>,
    products: BTreeMap<QueryKey, Arc<SemanticProduct>>,
    last_known_good: BTreeMap<QueryKey, Arc<SemanticProduct>>,
    index: DependencyIndex,
    scheduler: QueryScheduler,
    metrics: Arc<QueryMetrics>,
    revision_recomputed: BTreeSet<QueryKey>,
    revision_revalidated: BTreeSet<QueryKey>,
    transaction: Option<SemanticDbUndoJournal>,
    #[cfg(feature = "perf-observability")]
    query_recompute_started: BTreeMap<QueryKey, Instant>,
}

#[derive(Debug)]
struct SemanticDbUndoJournal {
    revision: SemanticRevision,
    revision_recomputed: BTreeSet<QueryKey>,
    revision_revalidated: BTreeSet<QueryKey>,
    query_states: BTreeMap<QueryKey, Option<QueryState>>,
    query_keys_by_module: BTreeMap<ModuleId, Option<BTreeSet<QueryKey>>>,
    products: BTreeMap<QueryKey, Option<Arc<SemanticProduct>>>,
    last_known_good: BTreeMap<QueryKey, Option<Arc<SemanticProduct>>>,
    scheduler: QueryScheduler,
}

impl SemanticDbUndoJournal {
    fn new(db: &mut SemanticDb) -> Self {
        #[cfg(feature = "perf-observability")]
        let scheduler = {
            use telos_observability::{PerfContext, PerfCounter, PerfOutcome, PerfPhase};
            let context = PerfContext::current_thread();
            let span = context.span(PerfPhase::SemanticDbSchedulerSnapshot);
            let scheduler = db.scheduler.clone();
            span.finish(PerfOutcome::Success);
            context.record_counter(PerfCounter::SemanticDbSchedulerSnapshotEntries, scheduler.len() as u64);
            scheduler
        };
        #[cfg(not(feature = "perf-observability"))]
        let scheduler = db.scheduler.clone();
        Self {
            revision: db.revision,
            revision_recomputed: std::mem::take(&mut db.revision_recomputed),
            revision_revalidated: std::mem::take(&mut db.revision_revalidated),
            query_states: BTreeMap::new(),
            query_keys_by_module: BTreeMap::new(),
            products: BTreeMap::new(),
            last_known_good: BTreeMap::new(),
            scheduler,
        }
    }
}

impl Default for SemanticDb {
    fn default() -> Self {
        Self::new()
    }
}

impl SemanticDb {
    pub fn new() -> Self {
        Self {
            canonical: None,
            workspace: WorkspaceId::from_raw(1),
            revision: SemanticRevision::from_raw(1),
            query_states: BTreeMap::new(),
            query_keys_by_module: BTreeMap::new(),
            products: BTreeMap::new(),
            last_known_good: BTreeMap::new(),
            index: DependencyIndex::new(),
            scheduler: QueryScheduler::new(),
            metrics: Arc::new(QueryMetrics::new()),
            revision_recomputed: BTreeSet::new(),
            revision_revalidated: BTreeSet::new(),
            transaction: None,
            #[cfg(feature = "perf-observability")]
            query_recompute_started: BTreeMap::new(),
        }
    }

    pub fn with_workspace(workspace: WorkspaceId) -> Self {
        Self {
            workspace,
            canonical: None,
            revision: SemanticRevision::from_raw(1),
            query_states: BTreeMap::new(),
            query_keys_by_module: BTreeMap::new(),
            products: BTreeMap::new(),
            last_known_good: BTreeMap::new(),
            index: DependencyIndex::new(),
            scheduler: QueryScheduler::new(),
            metrics: Arc::new(QueryMetrics::new()),
            revision_recomputed: BTreeSet::new(),
            revision_revalidated: BTreeSet::new(),
            transaction: None,
            #[cfg(feature = "perf-observability")]
            query_recompute_started: BTreeMap::new(),
        }
    }

    /// Creates a local database over an immutable semantic world. Consumers must
    /// use the image's final arenas (or validated extensions), never foundation IDs.
    /// No canonical query is copied into local state, journals, or scheduler queues.
    pub fn with_canonical_image(workspace: WorkspaceId, image: Arc<crate::canonical_image::CanonicalUniverseFormalImage>) -> Self {
        let mut db = Self::with_workspace(workspace);
        db.canonical = Some(image);
        db
    }

    pub fn canonical_image(&self) -> Option<&Arc<crate::canonical_image::CanonicalUniverseFormalImage>> {
        self.canonical.as_ref()
    }

    fn canonical_entry(&self, key: &QueryKey) -> Option<&crate::canonical_image::CanonicalQueryEntry> {
        self.canonical.as_ref()?.query_image().entry(key)
    }

    /// Whether this exact query root is owned by the immutable canonical image.
    ///
    /// Dependency-only evidence is deliberately excluded: only an admitted
    /// canonical query entry owns a reusable typed query product.
    pub(crate) fn is_canonical_query(&self, key: &QueryKey) -> bool {
        self.canonical_entry(key).is_some()
    }

    /// Direct input fingerprint recorded for an immutable canonical query root.
    pub(crate) fn canonical_query_input_fingerprint(&self, key: &QueryKey) -> Option<InputFingerprint> {
        self.canonical_entry(key).map(|entry| entry.input_fingerprint)
    }

    /// Product fingerprint recorded for an immutable canonical query root.
    pub(crate) fn canonical_query_product_fingerprint(&self, key: &QueryKey) -> Option<ProductFingerprint> {
        self.canonical_entry(key).map(|entry| entry.product_fingerprint)
    }

    /// Canonical dependencies are immutable leaves; local dependencies need
    /// current-revision validation even when their fingerprint did not change.
    pub fn is_current(&self, key: &QueryKey) -> bool {
        if let Some(state) = self.query_states.get(key) {
            return state.is_ready() && state.validated_revision() == Some(self.revision);
        }
        self.canonical_entry(key).is_some()
    }

    /// Current dependency evidence from a local query or exact immutable image product.
    /// This does not make aggregate-only products reusable database queries.
    pub fn dependency_fingerprint(&self, key: &QueryKey) -> Option<ProductFingerprint> {
        if let Some(state) = self.query_states.get(key) {
            return (state.is_ready() && state.validated_revision() == Some(self.revision))
                .then(|| state.product_fingerprint())
                .flatten();
        }
        self.canonical.as_ref()?.dependency_fingerprint(key)
    }

    fn dependency_matches(&self, edge: &DependencyEdge) -> bool {
        self.dependency_fingerprint(&edge.dependency) == Some(edge.observed_fingerprint)
    }

    pub fn workspace(&self) -> WorkspaceId {
        self.workspace
    }

    pub fn revision(&self) -> SemanticRevision {
        self.revision
    }

    /// Starts a per-key undo journal for one semantic publication attempt.
    pub(crate) fn begin_transaction(&mut self) {
        assert!(self.transaction.is_none(), "semantic database transactions cannot nest");
        self.index.begin_transaction();
        self.transaction = Some(SemanticDbUndoJournal::new(self));
    }

    /// Whether a publication transaction is still open (freeze must reject it).
    pub(crate) fn has_open_transaction(&self) -> bool {
        self.transaction.is_some()
    }

    /// Keeps the current database roots and discards their undo journal.
    pub(crate) fn commit_transaction(&mut self) {
        self.index.commit_transaction();
        self.transaction.take();
    }

    /// Restores the database state to the roots captured at transaction start.
    pub(crate) fn rollback_transaction(&mut self) {
        let Some(undo) = self.transaction.take() else {
            return;
        };
        self.index.rollback_transaction();
        for (key, old) in undo.query_states {
            match old {
                Some(state) => {
                    self.query_states.insert(key, state);
                }
                None => {
                    self.query_states.remove(&key);
                }
            }
        }
        for (module, old) in undo.query_keys_by_module {
            match old {
                Some(keys) => {
                    self.query_keys_by_module.insert(module, keys);
                }
                None => {
                    self.query_keys_by_module.remove(&module);
                }
            }
        }
        restore_query_products(&mut self.products, undo.products);
        restore_query_products(&mut self.last_known_good, undo.last_known_good);
        self.revision = undo.revision;
        self.revision_recomputed = undo.revision_recomputed;
        self.revision_revalidated = undo.revision_revalidated;
        self.scheduler = undo.scheduler;
    }

    fn remember_query_state(&mut self, key: &QueryKey) {
        if self.transaction.as_ref().is_none_or(|transaction| transaction.query_states.contains_key(key)) {
            return;
        }
        let old = self.query_states.get(key).cloned();
        self.transaction.as_mut().expect("transaction checked").query_states.insert(key.clone(), old);
        #[cfg(feature = "perf-observability")]
        record_semantic_db_counter(telos_observability::PerfCounter::SemanticDbUndoQueryStateEntries, 1);
    }

    fn remember_query_products(&mut self, key: &QueryKey) {
        if self.transaction.as_ref().is_none_or(|transaction| transaction.products.contains_key(key)) {
            return;
        }
        let old = self.products.get(key).cloned();
        self.transaction.as_mut().expect("transaction checked").products.insert(key.clone(), old);
        #[cfg(feature = "perf-observability")]
        record_semantic_db_counter(telos_observability::PerfCounter::SemanticDbUndoProductEntries, 1);
    }

    fn remember_last_known_good(&mut self, key: &QueryKey) {
        if self
            .transaction
            .as_ref()
            .is_none_or(|transaction| transaction.last_known_good.contains_key(key))
        {
            return;
        }
        let old = self.last_known_good.get(key).cloned();
        self.transaction.as_mut().expect("transaction checked").last_known_good.insert(key.clone(), old);
        #[cfg(feature = "perf-observability")]
        record_semantic_db_counter(telos_observability::PerfCounter::SemanticDbUndoLastKnownGoodEntries, 1);
    }

    fn remember_module_query_keys(&mut self, module: &ModuleId) {
        if self
            .transaction
            .as_ref()
            .is_none_or(|transaction| transaction.query_keys_by_module.contains_key(module))
        {
            return;
        }
        let old = self.query_keys_by_module.get(module).cloned();
        self.transaction
            .as_mut()
            .expect("transaction checked")
            .query_keys_by_module
            .insert(module.clone(), old);
        #[cfg(feature = "perf-observability")]
        record_semantic_db_counter(telos_observability::PerfCounter::SemanticDbUndoModuleKeyEntries, 1);
    }

    pub fn begin_revision(&mut self) -> SemanticRevision {
        self.revision = self.revision.next();
        self.revision_recomputed.clear();
        self.revision_revalidated.clear();
        self.revision
    }

    /// Query products computed or validated during the current revision.
    ///
    /// These event-local sets let publication metrics describe actual query
    /// work without rescanning every cached product at the end of an update.
    pub fn revision_recomputed_keys(&self) -> impl Iterator<Item = &QueryKey> {
        self.revision_recomputed.iter()
    }

    /// Query products reused after dependency validation during the current revision.
    pub fn revision_revalidated_keys(&self) -> impl Iterator<Item = &QueryKey> {
        self.revision_revalidated.iter()
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

    fn record_query_event(&self, key: &QueryKey, event: QueryLifecycleEvent, duration_ns: Option<u64>) {
        #[cfg(feature = "perf-observability")]
        {
            use telos_observability::{PerfAttribution, PerfContext, PerfQueryEvent, PerfScope};
            let context = PerfContext::current_thread();
            let context = if let Some(module) = query_key_module(key) {
                context.with_attribution(PerfAttribution::module(PerfScope::Workspace, format!("{module:?}")))
            } else {
                context.with_attribution(PerfAttribution::entity(PerfScope::Workspace, None, format!("{key:?}")))
            };
            let event = match event {
                QueryLifecycleEvent::Request => PerfQueryEvent::Request,
                QueryLifecycleEvent::DirectReuse => PerfQueryEvent::DirectReuse,
                QueryLifecycleEvent::RevalidatedReuse => PerfQueryEvent::RevalidatedReuse,
                QueryLifecycleEvent::Recomputed => PerfQueryEvent::Recomputed,
                QueryLifecycleEvent::Invalidated => PerfQueryEvent::Invalidated,
                QueryLifecycleEvent::Blocked => PerfQueryEvent::Blocked,
                QueryLifecycleEvent::Cancelled => PerfQueryEvent::Cancelled,
                QueryLifecycleEvent::BudgetExceeded => PerfQueryEvent::BudgetExceeded,
                QueryLifecycleEvent::Failed => PerfQueryEvent::Failed,
            };
            context.record_query_event(key.kind().as_str(), event, duration_ns);
        }
        #[cfg(not(feature = "perf-observability"))]
        let _ = (key, event, duration_ns);
    }

    /// Marks the beginning of query computation after reuse validation failed.
    pub(crate) fn record_query_recompute_start(&mut self, key: &QueryKey) {
        #[cfg(feature = "perf-observability")]
        self.query_recompute_started.insert(key.clone(), Instant::now());
        #[cfg(not(feature = "perf-observability"))]
        let _ = key;
    }

    fn record_query_recompute_finish(&mut self, key: &QueryKey, event: QueryLifecycleEvent) {
        self.metrics.record_miss();
        #[cfg(feature = "perf-observability")]
        {
            let started = self.query_recompute_started.remove(key);
            let elapsed = started.map(|started| u64::try_from(started.elapsed().as_nanos()).unwrap_or(u64::MAX));
            self.record_query_event(key, event, elapsed);
        }
        #[cfg(not(feature = "perf-observability"))]
        let _ = (key, event);
    }

    /// Local mutable state only; canonical readers never synthesize a Ready row.
    pub fn local_query_state(&self, key: &QueryKey) -> Option<&QueryState> {
        self.query_states.get(key)
    }

    /// Origin of a typed product. Local origin does not assert revision currentness.
    pub fn product_origin(&self, key: &QueryKey) -> Option<SemanticProductOrigin> {
        if self.query_states.contains_key(key) {
            return self.product(key).map(|_| SemanticProductOrigin::Workspace(self.workspace));
        }
        self.canonical_entry(key)
            .map(|_| SemanticProductOrigin::Canonical(self.canonical.as_ref().unwrap().world().clone()))
    }

    pub fn query_state(&self, key: &QueryKey) -> Option<&QueryState> {
        self.query_states.get(key)
    }

    /// Returns all cached query identities in deterministic order.
    pub fn query_keys(&self) -> impl Iterator<Item = &QueryKey> {
        self.query_states.keys()
    }

    /// Returns cached query identities owned by one module.
    ///
    /// The index is maintained at query publication/state creation time so an
    /// incremental update can seed exact product worklists from changed module
    /// owners without scanning the entire database.
    pub fn query_keys_for_module(&self, module: &ModuleId) -> impl Iterator<Item = &QueryKey> {
        self.query_keys_by_module.get(module).into_iter().flat_map(|keys| keys.iter())
    }

    /// Returns the typed product published for a ready query.
    pub fn product(&self, key: &QueryKey) -> Option<&Arc<SemanticProduct>> {
        if let Some(state) = self.query_states.get(key) {
            return if state.is_ready() { self.products.get(key) } else { None };
        }
        let entry = self.canonical_entry(key)?;
        #[cfg(feature = "perf-observability")]
        record_semantic_db_counter(telos_observability::PerfCounter::CanonicalQueryImmutableHits, 1);
        Some(&entry.product)
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
        assert!(self.canonical_entry(&key).is_none(), "cannot mutate an immutable canonical query");
        let terminal_event = match &state {
            QueryState::Blocked { .. } => Some(QueryLifecycleEvent::Blocked),
            QueryState::Cancelled { .. } => Some(QueryLifecycleEvent::Cancelled),
            QueryState::BudgetExceeded { .. } => Some(QueryLifecycleEvent::BudgetExceeded),
            QueryState::Failed { .. } => Some(QueryLifecycleEvent::Failed),
            _ => None,
        };
        if let Some(event) = terminal_event {
            self.record_query_recompute_finish(&key, event);
        }
        self.remember_query_state(&key);
        self.remember_query_products(&key);
        self.index_query_key(&key);
        if !state.is_ready() {
            self.products.remove(&key);
        }
        self.query_states.insert(key, state);
    }

    /// Returns the recorded product fingerprint for a query if it is in the `Ready` state.
    pub fn ready_product_fingerprint(&self, key: &QueryKey) -> Option<ProductFingerprint> {
        if let Some(state) = self.query_states.get(key) {
            return state.product_fingerprint();
        }
        self.canonical_entry(key).map(|entry| entry.product_fingerprint)
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
            return self.canonical_entry(key).is_some_and(|entry| entry.input_fingerprint == input_fingerprint);
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
            if !self.dependency_matches(edge) {
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
        self.record_query_event(key, QueryLifecycleEvent::Request, None);
        let was_current = self.query_states.get(key).and_then(QueryState::validated_revision) == Some(self.revision);
        if !self.is_reusable(key, input_fingerprint) {
            return false;
        }

        if !self.query_states.contains_key(key) {
            #[cfg(feature = "perf-observability")]
            record_semantic_db_counter(telos_observability::PerfCounter::CanonicalQueryImmutableHits, 1);
            return true;
        }
        self.remember_query_state(key);
        let current_revision = self.revision;
        let Some(QueryState::Ready { validated_revision, .. }) = self.query_states.get_mut(key) else {
            return false;
        };
        *validated_revision = current_revision;
        self.revision_revalidated.insert(key.clone());
        self.record_query_event(
            key,
            if was_current {
                QueryLifecycleEvent::DirectReuse
            } else {
                QueryLifecycleEvent::RevalidatedReuse
            },
            None,
        );
        true
    }

    /// Marks a ready product current after its exact dependencies have already
    /// been revalidated by the caller and confirmed matching observed fingerprints.
    pub fn validate_ready(&mut self, key: &QueryKey) -> bool {
        self.record_query_event(key, QueryLifecycleEvent::Request, None);
        let was_current = self.query_states.get(key).and_then(QueryState::validated_revision) == Some(self.revision);
        if let Some(deps) = self.index.dependencies_of(key) {
            for edge in deps {
                if !self.dependency_matches(edge) {
                    return false;
                }
            }
        }
        if !self.query_states.contains_key(key) {
            return self.canonical_entry(key).is_some();
        }

        self.remember_query_state(key);
        let current_revision = self.revision;
        let Some(QueryState::Ready { validated_revision, .. }) = self.query_states.get_mut(key) else {
            return false;
        };
        *validated_revision = current_revision;
        self.revision_revalidated.insert(key.clone());
        self.record_query_event(
            key,
            if was_current {
                QueryLifecycleEvent::DirectReuse
            } else {
                QueryLifecycleEvent::RevalidatedReuse
            },
            None,
        );
        true
    }

    /// Records an edge to a dependency validated for the current revision.
    ///
    /// A stored `Ready` product from an older revision is insufficient until its
    /// own query has been revalidated. This prevents a dependent from observing
    /// stale transitive state merely because an old product fingerprint is still
    /// present in the cache.
    pub fn record_dependency(&self, recorder: &mut DependencyRecorder, dependency: QueryKey) -> Result<(), String> {
        if !self.query_states.contains_key(&dependency) {
            if let Some(fingerprint) = self.canonical.as_ref().and_then(|image| image.dependency_fingerprint(&dependency)) {
                recorder.record(dependency, fingerprint);
                return Ok(());
            }
        }
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
        if self.canonical_entry(&key).is_some() {
            return Err(PublishError::immutable(self.revision));
        }
        if revision != self.revision {
            return Err(PublishError::stale(self.revision, revision));
        }

        self.remember_query_state(&key);
        self.remember_query_products(&key);
        self.index_query_key(&key);
        let immutable = self.canonical.as_ref();
        let local_states = &self.query_states;
        self.index.replace_dependencies_with_leaves(key.clone(), dependencies, |dependency| {
            !local_states.contains_key(dependency) && immutable.is_some_and(|image| image.dependency_fingerprint(dependency).is_some())
        });
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
        self.record_query_recompute_finish(&key, QueryLifecycleEvent::Recomputed);
        self.revision_recomputed.insert(key);
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
        if self.canonical_entry(key).is_some() {
            return false;
        }
        self.remember_query_state(key);
        self.remember_query_products(key);
        let had_state = self.query_states.remove(key).is_some();
        let had_product = self.products.remove(key).is_some();
        self.index.remove_dependencies(key);
        if had_state || had_product {
            self.metrics.record_invalidation();
            self.record_query_event(key, QueryLifecycleEvent::Invalidated, None);
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
        self.remember_last_known_good(&key);
        self.products.insert(key.clone(), product);
        if let Some(product) = self.products.get(&key) {
            self.last_known_good.insert(key, product.clone());
        }
        Ok(())
    }

    pub fn update_callable_body_product(
        &mut self,
        callable: &crate::identity::CallableId,
        analysis: Arc<crate::checker::CallableAnalysis>,
        dependencies: Vec<crate::db::DependencyEdge>,
    ) {
        let key = QueryKey::CallableBody(callable.clone());
        self.remember_query_state(&key);
        self.remember_query_products(&key);
        self.remember_last_known_good(&key);
        let canonical_image = self.canonical.clone();
        let local_states = &self.query_states;
        self.index.replace_dependencies_with_leaves(key.clone(), dependencies, |dependency| {
            !local_states.contains_key(dependency) && canonical_image.as_ref().is_some_and(|image| image.dependency_fingerprint(dependency).is_some())
        });
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
        let seeds = seeds.into_iter().filter(|key| self.canonical_entry(key).is_none());
        let closure = self.index.reverse_closure(seeds);
        for key in &closure {
            self.remember_query_state(key);
            self.remember_query_products(key);
            self.query_states.remove(key);
            self.products.remove(key);
            self.index.remove_dependencies(key);
            self.metrics.record_invalidation();
            self.record_query_event(key, QueryLifecycleEvent::Invalidated, None);
        }
        closure
    }

    /// Invalidates only supplied query roots, preserving cached dependents until
    /// the refreshed root product proves whether their fingerprints changed.
    pub fn invalidate_roots(&mut self, roots: impl IntoIterator<Item = QueryKey>) -> BTreeSet<QueryKey> {
        let roots: BTreeSet<_> = roots.into_iter().filter(|key| self.canonical_entry(key).is_none()).collect();
        for key in &roots {
            self.remember_query_state(key);
            self.remember_query_products(key);
            self.query_states.remove(key);
            self.products.remove(key);
            self.index.remove_dependencies(key);
            self.metrics.record_invalidation();
            self.record_query_event(key, QueryLifecycleEvent::Invalidated, None);
        }
        roots
    }

    /// Permanently removes one obsolete query identity and all of its cached state.
    ///
    /// Unlike ordinary invalidation, retirement also drops the last-known-good
    /// product so a later re-addition cannot reuse a prior incarnation.
    pub fn retire_query(&mut self, key: &QueryKey) -> usize {
        if self.canonical_entry(key).is_some() {
            return 0;
        }
        let dependents = self.index.reverse_closure([key.clone()]);
        let mut retired = 0;
        for dependent in dependents.iter().filter(|dependent| *dependent != key) {
            self.remember_query_state(dependent);
            self.remember_query_products(dependent);
            retired += usize::from(self.query_states.remove(dependent).is_some());
            retired += usize::from(self.products.remove(dependent).is_some());
            self.index.remove_dependencies(dependent);
            self.metrics.record_invalidation();
            self.record_query_event(dependent, QueryLifecycleEvent::Invalidated, None);
        }
        self.remember_query_state(key);
        self.remember_query_products(key);
        self.remember_last_known_good(key);
        retired += usize::from(self.query_states.remove(key).is_some());
        retired += usize::from(self.products.remove(key).is_some());
        retired += usize::from(self.last_known_good.remove(key).is_some());
        if retired > 0 {
            self.record_query_event(key, QueryLifecycleEvent::Invalidated, None);
        }
        if let Some(module) = query_key_module(key) {
            self.remember_module_query_keys(module);
            if let Some(module_keys) = self.query_keys_by_module.get_mut(module) {
                module_keys.remove(key);
                if module_keys.is_empty() {
                    self.query_keys_by_module.remove(module);
                }
            }
        }
        self.index.remove_dependencies(key);
        self.index.purge_keys(&BTreeSet::from([key.clone()]));
        retired
    }

    /// Permanently removes every cached query product owned by one obsolete module.
    ///
    /// This is intentionally separate from invalidation: cancellation and ordinary
    /// recomputation retain last-known-good products, while deleted/reidentified
    /// module identities must not survive in a long-lived workspace session.
    pub fn purge_module(&mut self, module: &ModuleId) -> usize {
        let keys: BTreeSet<QueryKey> = self
            .query_states
            .keys()
            .chain(self.products.keys())
            .chain(self.last_known_good.keys())
            .filter(|key| query_key_module(key) == Some(module))
            .cloned()
            .collect();

        let mut purged = 0;
        for key in &keys {
            self.remember_query_state(key);
            self.remember_query_products(key);
            self.remember_last_known_good(key);
            purged += usize::from(self.query_states.remove(key).is_some());
            purged += usize::from(self.products.remove(key).is_some());
            purged += usize::from(self.last_known_good.remove(key).is_some());
            self.record_query_event(key, QueryLifecycleEvent::Invalidated, None);
        }
        self.remember_module_query_keys(module);
        self.query_keys_by_module.remove(module);
        self.index.purge_keys(&keys);
        purged
    }

    fn index_query_key(&mut self, key: &QueryKey) {
        if let Some(module) = query_key_module(key) {
            self.remember_module_query_keys(module);
            self.query_keys_by_module.entry(module.clone()).or_default().insert(key.clone());
        }
    }
}

fn restore_query_products(current: &mut BTreeMap<QueryKey, Arc<SemanticProduct>>, undo: BTreeMap<QueryKey, Option<Arc<SemanticProduct>>>) {
    for (key, old) in undo {
        match old {
            Some(product) => {
                current.insert(key, product);
            }
            None => {
                current.remove(&key);
            }
        }
    }
}

pub(crate) fn query_key_module(key: &QueryKey) -> Option<&ModuleId> {
    match key {
        QueryKey::ParsedModule(module)
        | QueryKey::UnlinkedInterface(module)
        | QueryKey::ResolvedImports(module)
        | QueryKey::LinkedInterface(module)
        | QueryKey::SemanticComponent(module)
        | QueryKey::SourceStructure(module)
        | QueryKey::AdvisoryModule(module)
        | QueryKey::ModuleDiagnostics(module)
        | QueryKey::ModuleMetadata(module)
        | QueryKey::ModuleStaticBindingCatalog(module)
        | QueryKey::ModuleStaticBindings(module) => Some(module),
        QueryKey::DeclarationShell(declaration)
        | QueryKey::HierarchyEdge(declaration)
        | QueryKey::DeclarationSurface(declaration)
        | QueryKey::TraitHeader(declaration)
        | QueryKey::TraitSurface(declaration)
        | QueryKey::DataDeclaration(declaration)
        | QueryKey::EnumDeclaration(declaration)
        | QueryKey::EnumRequirements(declaration)
        | QueryKey::ConstructorInitialization(declaration)
        | QueryKey::AssociatedSurface(declaration)
        | QueryKey::StaticDefinition(declaration) => Some(&declaration.module),
        QueryKey::StaticConstant(constant) => Some(&constant.0.module),
        QueryKey::CheckedStaticBody(body) => Some(&body.0.module),
        QueryKey::AttributeArgument(argument) => Some(argument.module()),
        QueryKey::AssociatedStaticDomainDeclaration(domain) => Some(&domain.owner.module),
        QueryKey::StaticDomainCaseCatalog(domain) => Some(match domain {
            crate::identity::StaticDomainDeclarationId::Nominal(declaration) => &declaration.module,
            crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.owner.module,
        }),
        QueryKey::StaticCaseSignature(case) => Some(match &case.domain {
            crate::identity::StaticDomainDeclarationId::Nominal(declaration) => &declaration.module,
            crate::identity::StaticDomainDeclarationId::Associated(declaration) => &declaration.owner.module,
        }),
        QueryKey::TypeMemberDeclaration(member) => Some(&member.owner.module),
        QueryKey::ResolvedImport(site) => Some(&site.importer),
        QueryKey::LinkedName(module, _) | QueryKey::PublicExport(module, _) => Some(module),
        QueryKey::FieldSignature(field) => Some(&field.owner.module),
        QueryKey::CallableDefinition(callable)
        | QueryKey::CallableSignature(callable)
        | QueryKey::CallableBody(callable)
        | QueryKey::CallableEffects(callable)
        | QueryKey::CallableControl(callable)
        | QueryKey::CallableTermination(callable)
        | QueryKey::CallableContracts(callable)
        | QueryKey::VerificationConditions(callable)
        | QueryKey::SourceFormalAttachment(callable)
        | QueryKey::AdvisoryCallable(callable) => Some(callable.module()),
        QueryKey::ConformanceDispatch(impl_id) => Some(&impl_id.module),
        QueryKey::GenericConformance(key) => Some(&key.trait_ref.declaration.module),
        QueryKey::TraitDispatchBucket(bucket) => match &bucket.target_family {
            crate::trait_dispatch::TraitDispatchTargetFamily::Declaration(decl) => Some(&decl.module),
            crate::trait_dispatch::TraitDispatchTargetFamily::ExactEnumCase(variant) => Some(&variant.owner.module),
        },
    }
}

#[cfg(test)]
mod transaction_tests {
    use super::{DependencyEdge, InputFingerprint, ProductFingerprint, QueryKey, SemanticDb, SemanticProduct};
    use crate::db::QueryState;
    use crate::identity::ModuleId;
    use std::collections::BTreeSet;
    use std::sync::Arc;

    fn empty_diagnostics() -> SemanticProduct {
        SemanticProduct::ModuleDiagnostics(Arc::from([]))
    }

    #[test]
    fn rollback_restores_query_products_dependencies_indexes_revision_and_scheduler() {
        let module = ModuleId::universe_root();
        let key = QueryKey::ModuleDiagnostics(module.clone());
        let dependent = QueryKey::PublicExport(module.clone(), "existing".to_owned());
        let added = QueryKey::PublicExport(module.clone(), "transient".to_owned());
        let mut db = SemanticDb::new();
        let published_revision = db.begin_revision();

        db.publish_product_ready(
            key.clone(),
            published_revision,
            InputFingerprint::new(1),
            ProductFingerprint::new(11),
            empty_diagnostics(),
            [],
        )
        .expect("initial product publishes");
        db.publish_product_ready(
            dependent.clone(),
            published_revision,
            InputFingerprint::new(2),
            ProductFingerprint::new(22),
            empty_diagnostics(),
            [DependencyEdge {
                dependent: dependent.clone(),
                dependency: key.clone(),
                observed_fingerprint: ProductFingerprint::new(11),
            }],
        )
        .expect("dependent product publishes");
        db.scheduler_mut().enqueue(key.clone());

        let original_revision = db.revision();
        let original_recomputed = db.revision_recomputed_keys().cloned().collect::<BTreeSet<_>>();
        let original_revalidated = db.revision_revalidated_keys().cloned().collect::<BTreeSet<_>>();
        let original_module_keys = db.query_keys_for_module(&module).cloned().collect::<BTreeSet<_>>();
        let original_key_state = db.query_state(&key).cloned().expect("root state exists");
        let original_dependent_state = db.query_state(&dependent).cloned().expect("dependent state exists");
        let original_product = db.product(&key).expect("root product exists").clone();
        let original_last_known_good = db.last_known_good_product(&key).expect("last good exists").clone();
        let original_dependent_edges = db.index().dependencies_of(&dependent).expect("dependency edges exist").to_vec();
        let original_invalidations = db.metrics().invalidations.load(std::sync::atomic::Ordering::Relaxed);

        db.begin_transaction();
        db.begin_revision();
        db.retire_query(&key);
        db.publish_product_ready(
            added.clone(),
            db.revision(),
            InputFingerprint::new(3),
            ProductFingerprint::new(33),
            empty_diagnostics(),
            [],
        )
        .expect("candidate product publishes");
        db.scheduler_mut().clear();
        db.scheduler_mut().enqueue(added.clone());
        db.purge_module(&module);

        db.rollback_transaction();

        assert_eq!(db.revision(), original_revision);
        assert_eq!(db.query_state(&key), Some(&original_key_state));
        assert_eq!(db.query_state(&dependent), Some(&original_dependent_state));
        assert!(Arc::ptr_eq(db.product(&key).expect("root product restored"), &original_product));
        assert!(Arc::ptr_eq(
            db.last_known_good_product(&key).expect("last good product restored"),
            &original_last_known_good
        ));
        assert_eq!(db.index().dependencies_of(&dependent), Some(original_dependent_edges.as_slice()));
        assert!(db.index().dependents_of(&key).is_some_and(|keys| keys.contains(&dependent)));
        assert_eq!(db.query_keys_for_module(&module).cloned().collect::<BTreeSet<_>>(), original_module_keys);
        assert_eq!(db.revision_recomputed_keys().cloned().collect::<BTreeSet<_>>(), original_recomputed);
        assert_eq!(db.revision_revalidated_keys().cloned().collect::<BTreeSet<_>>(), original_revalidated);
        assert!(db.query_state(&added).is_none());
        assert_eq!(db.scheduler_mut().len(), 1);
        assert_eq!(db.scheduler_mut().pop_next(), Some(key));
        assert!(db.metrics().invalidations.load(std::sync::atomic::Ordering::Relaxed) > original_invalidations);
    }

    #[test]
    fn commit_keeps_candidate_database_roots() {
        let module = ModuleId::universe_root();
        let key = QueryKey::ModuleDiagnostics(module);
        let mut db = SemanticDb::new();
        db.begin_transaction();
        let revision = db.begin_revision();
        db.publish_product_ready(
            key.clone(),
            revision,
            InputFingerprint::new(1),
            ProductFingerprint::new(10),
            empty_diagnostics(),
            [],
        )
        .expect("candidate product publishes");

        db.commit_transaction();

        assert_eq!(db.revision(), revision);
        assert!(db.query_state(&key).is_some_and(QueryState::is_ready));
        assert!(db.product(&key).is_some());
    }
}
