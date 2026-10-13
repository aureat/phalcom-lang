//! Completed, immutable Universe semantics and explicitly eligible query roots.
//!
//! Construction uses the ordinary analyzer through the foundation constructor,
//! then freezes only after its database transaction commits. This module never
//! calls the core compiler and never installs its snapshot in a local session.

use crate::canonical_baseline::{CanonicalUniverseSemanticBaseline, canonical_universe_semantic_baseline};
use crate::db::{DependencyEdge, InputFingerprint, ProductFingerprint, QueryKey, QueryKind, QueryState, SemanticDb, SemanticProduct};
use crate::identity::WorkspaceId;
use crate::session::{SemanticUpdateFailure, SemanticWorkspaceSession};
use crate::snapshot::SemanticSnapshot;
use crate::static_value::{StaticValueStore, StaticValueStoreId};
use crate::types::id::TypeStoreId;
use crate::types::store::TypeStore;
use crate::workspace::SemanticWorkspaceInput;
use phalcom_modules::canonical_products::CanonicalModuleProducts;
use phalcom_modules::{ModuleId, UniverseSourceProvider};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use thiserror::Error;

/// In-process semantic world provenance. This is not a portable artifact digest.
/// Source/interface meaning, module/semantic ABI, and native ABI are fixed by the
/// running binary. Overridden provider source content contributes its own digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSemanticWorldId {
    source: u64,
    // Exact identities/text roots prevent fingerprint collisions from making
    // distinct private providers a compatible semantic world.
    provenance: Arc<BTreeMap<ModuleId, Arc<str>>>,
    compatibility: &'static str,
}

/// Identity and extent of the final canonical arena, not the bootstrap arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalTypePrefixId {
    store: TypeStoreId,
    types: usize,
}

/// Failure before a completed image can be safely published or read.
#[derive(Debug, Error)]
pub enum CanonicalImageError {
    /// Module-layer source/interface capture failed.
    #[error("canonical module products: {0}")]
    ModuleProducts(String),
    /// The existing foundation boundary panicked; the cache remains retryable.
    #[error("canonical semantic bootstrap panicked; no image published")]
    BootstrapPanicked,
    /// Same-thread cache reentry would deadlock.
    #[error("canonical formal-image initialization reentered")]
    BootstrapRecursion,
    /// Ordinary analysis failed and rolled back.
    #[error("canonical semantic update: {0}")]
    SemanticUpdate(#[from] SemanticUpdateFailure),
    /// Query export was attempted before successful transaction commit.
    #[error("cannot freeze an open semantic transaction")]
    OpenTransaction,
    /// The committed snapshot lacks a required final publication family.
    #[error("canonical publication is incomplete: {0}")]
    IncompletePublication(String),
    /// A reader requested products from another semantic world.
    #[error("canonical query reader has a foreign semantic world")]
    ForeignSemanticWorld,
    /// A reader's arena does not preserve the final canonical prefix.
    #[error("canonical query reader has an incompatible type prefix")]
    IncompatibleTypePrefix,
    /// Static references have another lineage or different prefix content.
    #[error("canonical query reader has an incompatible static-value root")]
    IncompatibleStaticValueStore,
}

/// Exact rejection category for each examined query row.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum QueryFreezeRejection {
    /// No committed computation exists.
    Vacant,
    /// Still computing.
    Computing,
    /// Cancelled computation.
    Cancelled,
    /// Resource budget exhausted.
    BudgetExceeded,
    /// Blocked semantics remain in the reference snapshot and diagnostics.
    BlockedRequiresClassification,
    /// Failed semantics remain in the reference snapshot and diagnostics.
    FailedRequiresClassification,
    /// Query belongs to a workspace outside this closed Universe.
    WorkspaceOrigin,
    /// Ready row has not been validated in the committing revision.
    StaleValidation,
    /// Query family has not been independently proven relocatable.
    UnsupportedQueryKind,
    /// Proof/search depends on the extensible local candidate universe.
    NegativeLocalCandidateSet,
    /// Ready state has no matching typed product.
    MissingTypedProduct,
    /// Typed payload does not match its query family.
    ProductKindMismatch,
    /// A dependency is not retained or its observed fingerprint changed.
    MutableDependency,
    /// Cyclic dependencies lack an independently validated freeze contract.
    DependencyCycle,
}

#[cfg(feature = "perf-observability")]
impl QueryFreezeRejection {
    fn record(self, count: usize) {
        use ::telos_observability::PerfCounter;
        let counter = match self {
            Self::Vacant => PerfCounter::CanonicalQueryRejectedVacant,
            Self::Computing => PerfCounter::CanonicalQueryRejectedComputing,
            Self::Cancelled => PerfCounter::CanonicalQueryRejectedCancelled,
            Self::BudgetExceeded => PerfCounter::CanonicalQueryRejectedBudgetExceeded,
            Self::BlockedRequiresClassification => PerfCounter::CanonicalQueryRejectedBlockedRequiresClassification,
            Self::FailedRequiresClassification => PerfCounter::CanonicalQueryRejectedFailedRequiresClassification,
            Self::WorkspaceOrigin => PerfCounter::CanonicalQueryRejectedWorkspaceOrigin,
            Self::StaleValidation => PerfCounter::CanonicalQueryRejectedStaleValidation,
            Self::UnsupportedQueryKind => PerfCounter::CanonicalQueryRejectedUnsupportedQueryKind,
            Self::NegativeLocalCandidateSet => PerfCounter::CanonicalQueryRejectedNegativeLocalCandidateSet,
            Self::MissingTypedProduct => PerfCounter::CanonicalQueryRejectedMissingTypedProduct,
            Self::ProductKindMismatch => PerfCounter::CanonicalQueryRejectedProductKindMismatch,
            Self::MutableDependency => PerfCounter::CanonicalQueryRejectedMutableDependency,
            Self::DependencyCycle => PerfCounter::CanonicalQueryRejectedDependencyCycle,
        };
        ::telos_observability::PerfContext::current_thread().record_counter(counter, count as u64);
    }
}

/// Census accounting includes every row, including rejected terminal states.
#[derive(Debug, Default)]
pub struct CanonicalQueryCensus {
    /// Total committed database rows examined.
    pub total_rows_examined: usize,
    /// Retained row count by typed query family.
    pub retained_by_kind: BTreeMap<QueryKind, usize>,
    /// Exact rejected key and reason; diagnostic products stay in the snapshot.
    pub rejections: BTreeMap<QueryKey, QueryFreezeRejection>,
    /// Original failed/blocked diagnostic detail, without mutable execution state.
    pub terminal_diagnostics: BTreeMap<QueryKey, Arc<str>>,
}

/// Immutable metadata and typed product for one closed canonical query.
#[derive(Debug)]
pub struct CanonicalQueryEntry {
    /// Direct input observation at successful publication.
    pub input_fingerprint: InputFingerprint,
    /// Dependency-visible product content fingerprint.
    pub product_fingerprint: ProductFingerprint,
    /// Frozen forward observations, never a mutable reverse invalidation graph.
    pub dependencies: Arc<[DependencyEdge]>,
    pub(crate) product: Arc<SemanticProduct>,
}

/// Read-only query roots with their owning final arena and static-value root.
#[derive(Debug)]
pub struct CanonicalSemanticQueryImage {
    world: CanonicalSemanticWorldId,
    entries: BTreeMap<QueryKey, CanonicalQueryEntry>,
    census: CanonicalQueryCensus,
    store: Arc<TypeStore>,
    static_values: Arc<StaticValueStore>,
}

/// Validated immutable-origin reader. Borrowing the consumer stores prevents
/// mutation of their validated prefixes for the lifetime of this reader.
pub struct CanonicalQueryReader<'a> {
    image: &'a CanonicalSemanticQueryImage,
    _store: &'a TypeStore,
    _static_values: &'a StaticValueStore,
}

impl CanonicalQueryReader<'_> {
    /// Looks up a typed product with canonical origin, never a local Ready row.
    pub fn product(&self, key: &QueryKey) -> Option<&Arc<SemanticProduct>> {
        let product = &self.image.entries.get(key)?.product;
        image_counter!(CanonicalQueryImmutableHits, 1);
        Some(product)
    }
    /// Returns the immutable product observation for forward local dependencies.
    pub fn product_fingerprint(&self, key: &QueryKey) -> Option<ProductFingerprint> {
        self.image.entries.get(key).map(|entry| entry.product_fingerprint)
    }
    /// The origin of every product returned by this reader.
    pub fn origin(&self) -> crate::db::SemanticProductOrigin {
        crate::db::SemanticProductOrigin::Canonical(self.image.world.clone())
    }
    /// Compatible closed semantic world.
    pub fn world(&self) -> &CanonicalSemanticWorldId {
        &self.image.world
    }
}

impl CanonicalSemanticQueryImage {
    /// Number of eligible roots.
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    /// Whether no family was eligible.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    /// Deterministic entry metadata; products require a validated reader.
    pub fn entries(&self) -> impl Iterator<Item = (&QueryKey, &CanonicalQueryEntry)> {
        self.entries.iter()
    }
    pub(crate) fn entry(&self, key: &QueryKey) -> Option<&CanonicalQueryEntry> {
        self.entries.get(key)
    }

    /// Complete accounting of retained and rejected rows.
    pub fn census(&self) -> &CanonicalQueryCensus {
        &self.census
    }
    /// Binds a reader to one compatible semantic world and final arena prefix.
    pub fn reader<'a>(
        &'a self,
        world: &CanonicalSemanticWorldId,
        store: &'a TypeStore,
        values: &'a StaticValueStore,
    ) -> Result<CanonicalQueryReader<'a>, CanonicalImageError> {
        if world != &self.world {
            return Err(CanonicalImageError::ForeignSemanticWorld);
        }
        if !std::ptr::eq(store, self.store.as_ref()) && !store.extends_canonical_prefix(&self.store) {
            image_counter!(CanonicalTypePrefixValidationErrors, 1);
            return Err(CanonicalImageError::IncompatibleTypePrefix);
        }
        if !std::ptr::eq(values, self.static_values.as_ref()) && !values.extends_canonical_prefix(&self.static_values) {
            return Err(CanonicalImageError::IncompatibleStaticValueStore);
        }
        Ok(CanonicalQueryReader {
            image: self,
            _store: store,
            _static_values: values,
        })
    }

    fn freeze(
        db: &SemanticDb,
        world: CanonicalSemanticWorldId,
        snapshot: &Arc<SemanticSnapshot>,
        modules: &CanonicalModuleProducts,
    ) -> Result<Self, CanonicalImageError> {
        if db.has_open_transaction() {
            return Err(CanonicalImageError::OpenTransaction);
        }
        let mut census = CanonicalQueryCensus::default();
        let mut entries = BTreeMap::new();
        for key in db.query_keys() {
            census.total_rows_examined += 1;
            let state = db.query_state(key).expect("query key enumerates a state");
            match state {
                QueryState::Failed { failure, .. } => {
                    census.terminal_diagnostics.insert(key.clone(), Arc::from(failure.as_str()));
                }
                QueryState::Blocked { reason, .. } => {
                    census.terminal_diagnostics.insert(key.clone(), Arc::from(format!("{reason:?}")));
                }
                _ => {}
            }
            let rejection = match state {
                QueryState::Vacant => Some(QueryFreezeRejection::Vacant),
                QueryState::Computing { .. } => Some(QueryFreezeRejection::Computing),
                QueryState::Cancelled { .. } => Some(QueryFreezeRejection::Cancelled),
                QueryState::BudgetExceeded { .. } => Some(QueryFreezeRejection::BudgetExceeded),
                QueryState::Blocked { .. } => Some(QueryFreezeRejection::BlockedRequiresClassification),
                QueryState::Failed { .. } => Some(QueryFreezeRejection::FailedRequiresClassification),
                QueryState::Ready { validated_revision, .. } if *validated_revision != db.revision() => Some(QueryFreezeRejection::StaleValidation),
                QueryState::Ready { .. } => None,
            };
            let rejection = rejection.or_else(|| {
                if !crate::db::query_key_module(key).is_some_and(|module| modules.sources().contains_key(module)) {
                    return Some(QueryFreezeRejection::WorkspaceOrigin);
                }
                match key {
                    QueryKey::GenericConformance(_)
                    | QueryKey::ConformanceDispatch(_)
                    | QueryKey::TraitDispatchBucket(_)
                    | QueryKey::CallableBody(_)
                    | QueryKey::CheckedStaticBody(_) => Some(QueryFreezeRejection::NegativeLocalCandidateSet),
                    // Structural positive canonical products only. Proof bodies,
                    // presentation/advisory, static evaluation, and aggregate
                    // environment queries need their own relocation contracts.
                    QueryKey::ParsedModule(_)
                    | QueryKey::UnlinkedInterface(_)
                    | QueryKey::ResolvedImports(_)
                    | QueryKey::LinkedInterface(_)
                    | QueryKey::DeclarationShell(_)
                    | QueryKey::HierarchyEdge(_)
                    | QueryKey::CallableDefinition(_)
                    | QueryKey::CallableSignature(_)
                    | QueryKey::FieldSignature(_)
                    | QueryKey::DeclarationSurface(_)
                    | QueryKey::TraitHeader(_)
                    | QueryKey::TraitSurface(_)
                    | QueryKey::DataDeclaration(_)
                    | QueryKey::EnumDeclaration(_)
                    | QueryKey::AssociatedSurface(_)
                    | QueryKey::TypeMemberDeclaration(_)
                    | QueryKey::ModuleStaticBindingCatalog(_) => None,
                    _ => Some(QueryFreezeRejection::UnsupportedQueryKind),
                }
            });
            if let Some(reason) = rejection {
                census.rejections.insert(key.clone(), reason);
                continue;
            }
            let Some(product) = db.product(key) else {
                census.rejections.insert(key.clone(), QueryFreezeRejection::MissingTypedProduct);
                continue;
            };
            if !matches!(
                (key, product.as_ref()),
                (QueryKey::ParsedModule(_), SemanticProduct::ParsedModule(_))
                    | (QueryKey::UnlinkedInterface(_), SemanticProduct::UnlinkedInterface(_))
                    | (QueryKey::ResolvedImports(_), SemanticProduct::ResolvedImports(_))
                    | (QueryKey::LinkedInterface(_), SemanticProduct::LinkedInterface(_))
                    | (QueryKey::DeclarationShell(_), SemanticProduct::DeclarationShell(_))
                    | (QueryKey::HierarchyEdge(_), SemanticProduct::HierarchyEdge(_))
                    | (QueryKey::CallableDefinition(_), SemanticProduct::CallableDefinition(_))
                    | (QueryKey::CallableSignature(_), SemanticProduct::CallableSignature(_))
                    | (QueryKey::FieldSignature(_), SemanticProduct::FieldSignature(_))
                    | (QueryKey::DeclarationSurface(_), SemanticProduct::DeclarationSurface(_))
                    | (QueryKey::TraitHeader(_), SemanticProduct::TraitHeader(_))
                    | (QueryKey::TraitSurface(_), SemanticProduct::TraitSurface(_))
                    | (QueryKey::DataDeclaration(_), SemanticProduct::DataDeclaration(_))
                    | (QueryKey::EnumDeclaration(_), SemanticProduct::EnumDeclaration(_))
                    | (QueryKey::AssociatedSurface(_), SemanticProduct::AssociatedSurface(_))
                    | (QueryKey::TypeMemberDeclaration(_), SemanticProduct::TypeMemberDeclaration(_))
                    | (QueryKey::ModuleStaticBindingCatalog(_), SemanticProduct::ModuleStaticBindingCatalog(_))
            ) {
                census.rejections.insert(key.clone(), QueryFreezeRejection::ProductKindMismatch);
                continue;
            }
            let QueryState::Ready {
                input_fingerprint,
                product_fingerprint,
                ..
            } = state
            else {
                unreachable!()
            };
            entries.insert(
                key.clone(),
                CanonicalQueryEntry {
                    input_fingerprint: *input_fingerprint,
                    product_fingerprint: *product_fingerprint,
                    product: product.clone(),
                    dependencies: Arc::from(db.index().dependencies_of(key).unwrap_or_default()),
                },
            );
        }
        // Reject the transitive closure of non-retained or stale observations.
        loop {
            let rejected = entries
                .iter()
                .filter(|(key, entry)| {
                    entry.dependencies.iter().any(|edge| {
                        edge.dependent != **key
                            || !entries
                                .get(&edge.dependency)
                                .is_some_and(|dependency| dependency.product_fingerprint == edge.observed_fingerprint)
                    })
                })
                .map(|(key, _)| key.clone())
                .collect::<Vec<_>>();
            if rejected.is_empty() {
                break;
            }
            for key in rejected {
                entries.remove(&key);
                census.rejections.insert(key, QueryFreezeRejection::MutableDependency);
            }
        }
        // Leaf elimination proves acyclic closed observations without retaining
        // a scheduler or reverse index. All surviving cyclic dependents reject.
        let mut pending: BTreeSet<_> = entries.keys().cloned().collect();
        loop {
            let leaves = pending
                .iter()
                .filter(|key| entries[*key].dependencies.iter().all(|edge| !pending.contains(&edge.dependency)))
                .cloned()
                .collect::<Vec<_>>();
            if leaves.is_empty() {
                break;
            }
            for key in leaves {
                pending.remove(&key);
            }
        }
        for key in pending {
            entries.remove(&key);
            census.rejections.insert(key, QueryFreezeRejection::DependencyCycle);
        }
        for key in entries.keys() {
            *census.retained_by_kind.entry(key.kind()).or_default() += 1;
        }
        #[cfg(feature = "perf-observability")]
        {
            let mut by_reason = BTreeMap::new();
            for reason in census.rejections.values() {
                *by_reason.entry(*reason).or_insert(0_usize) += 1;
            }
            for (reason, count) in by_reason {
                reason.record(count);
            }
        }
        image_counter!(CanonicalQueryRowsExamined, census.total_rows_examined);
        image_counter!(CanonicalQueryReadyEligible, entries.len());
        image_counter!(CanonicalQueryRejected, census.rejections.len());
        Ok(Self {
            world,
            entries,
            census,
            store: snapshot.store.clone(),
            static_values: snapshot.static_values.clone(),
        })
    }
}

/// A successfully committed complete semantic publication and its immutable roots.
#[derive(Debug)]
pub struct CanonicalUniverseFormalImage {
    provider: UniverseSourceProvider,
    world: CanonicalSemanticWorldId,
    modules: Arc<CanonicalModuleProducts>,
    snapshot: Arc<SemanticSnapshot>,
    queries: Arc<CanonicalSemanticQueryImage>,
    type_prefix: CanonicalTypePrefixId,
}

impl CanonicalUniverseFormalImage {
    /// Exact provider used to build this canonical world.
    pub(crate) fn provider(&self) -> &UniverseSourceProvider { &self.provider }
    /// Provenance used to validate immutable-origin reads.
    pub fn world(&self) -> &CanonicalSemanticWorldId {
        &self.world
    }
    /// Complete authoritative final snapshot; all semantic families stay here.
    pub fn snapshot(&self) -> &Arc<SemanticSnapshot> {
        &self.snapshot
    }
    /// Module-layer P1 dependency adapter.
    pub fn modules(&self) -> &Arc<CanonicalModuleProducts> {
        &self.modules
    }
    /// Complete parsed source set.
    pub fn sources(&self) -> &BTreeMap<ModuleId, Arc<phalcom_modules::ParsedModuleUnit>> {
        self.modules.sources()
    }
    /// Provider-derived module interfaces.
    pub fn interfaces(&self) -> &BTreeMap<ModuleId, Arc<phalcom_modules::UnlinkedModuleInterface>> {
        self.modules.interfaces()
    }
    /// Foundation-linked canonical program.
    pub fn linked(&self) -> &Arc<phalcom_modules::LinkedProgram> {
        self.modules.linked()
    }
    /// Immutable query roots, separate from a session database.
    pub fn query_image(&self) -> &Arc<CanonicalSemanticQueryImage> {
        &self.queries
    }
    /// Final arena identity and extent.
    pub fn type_prefix(&self) -> CanonicalTypePrefixId {
        self.type_prefix
    }
    /// Final static-value lineage.
    pub fn static_lineage(&self) -> StaticValueStoreId {
        self.snapshot.static_values.id()
    }
    /// Reader using the image's own validated arenas.
    pub fn reader(&self) -> Result<CanonicalQueryReader<'_>, CanonicalImageError> {
        self.queries.reader(&self.world, &self.snapshot.store, &self.snapshot.static_values)
    }
}

macro_rules! image_counter {
    ($counter:ident, $amount:expr) => {
        #[cfg(feature = "perf-observability")]
        ::telos_observability::PerfContext::current_thread().record_counter(::telos_observability::PerfCounter::$counter, $amount as u64);
    };
}
use image_counter;

fn image_span<T, E>(#[allow(unused_variables)] phase: ImagePhase, operation: impl FnOnce() -> Result<T, E>) -> Result<T, E> {
    #[cfg(feature = "perf-observability")]
    {
        ::telos_observability::PerfContext::current_thread().with_span_result(
            phase.perf(),
            |_| operation(),
            |_| ::telos_observability::PerfOutcome::SemanticError,
        )
    }
    #[cfg(not(feature = "perf-observability"))]
    {
        operation()
    }
}
#[derive(Clone, Copy)]
enum ImagePhase {
    CacheLookup,
    Build,
    Bootstrap,
    Validate,
    QueryFreeze,
}
#[cfg(feature = "perf-observability")]
impl ImagePhase {
    fn perf(self) -> ::telos_observability::PerfPhase {
        use ::telos_observability::PerfPhase;
        match self {
            Self::CacheLookup => PerfPhase::CanonicalImageCacheLookup,
            Self::Build => PerfPhase::CanonicalImageBuild,
            Self::Bootstrap => PerfPhase::CanonicalImageBootstrap,
            Self::Validate => PerfPhase::CanonicalImageValidate,
            Self::QueryFreeze => PerfPhase::CanonicalImageQueryFreeze,
        }
    }
}

static IMAGE: OnceLock<Arc<CanonicalUniverseFormalImage>> = OnceLock::new();
static INITIALIZE: Mutex<()> = Mutex::new(());
static BUILDS: AtomicUsize = AtomicUsize::new(0);
static COMPLETE_UPDATES: AtomicUsize = AtomicUsize::new(0);
thread_local! { static BUILDING: Cell<bool> = const { Cell::new(false) }; }
struct BootstrapGuard;
impl BootstrapGuard {
    fn enter() -> Result<Self, CanonicalImageError> {
        if BUILDING.with(|flag| flag.replace(true)) {
            image_counter!(CanonicalImageRecursionAttempts, 1);
            return Err(CanonicalImageError::BootstrapRecursion);
        }
        Ok(Self)
    }
}
impl Drop for BootstrapGuard {
    fn drop(&mut self) {
        BUILDING.with(|flag| flag.set(false));
    }
}

/// Builtin provider access. Failed initialization is never cached; compatible
/// concurrent callers share one successful build. No callbacks can reenter it.
pub fn canonical_universe_formal_image() -> Result<&'static Arc<CanonicalUniverseFormalImage>, CanonicalImageError> {
    image_span(ImagePhase::CacheLookup, || {
        if let Some(image) = IMAGE.get() {
            image_counter!(CanonicalImageCacheHits, 1);
            return Ok(image);
        }
        let _guard = BootstrapGuard::enter()?;
        let _initialize = INITIALIZE.lock().unwrap_or_else(|poison| poison.into_inner());
        if let Some(image) = IMAGE.get() {
            image_counter!(CanonicalImageCacheHits, 1);
            return Ok(image);
        }
        let image = build_catching_panics(&UniverseSourceProvider::new())?;
        let _ = IMAGE.set(image);
        Ok(IMAGE.get().expect("successful initializer publishes image"))
    })
}

/// Explicit providers share the builtin image only for the sealed builtin
/// source authority; overrides always build a private world and arena lineage.
pub fn canonical_universe_formal_image_with_provider(provider: &UniverseSourceProvider) -> Result<Arc<CanonicalUniverseFormalImage>, CanonicalImageError> {
    if provider.is_canonical() {
        return canonical_universe_formal_image().cloned();
    }
    let _guard = BootstrapGuard::enter()?;
    build_catching_panics(provider)
}

/// Successful builtin image constructions, excluding private-world builds.
pub fn canonical_universe_formal_image_build_count() -> usize {
    BUILDS.load(Ordering::Relaxed)
}
/// Completed builtin semantic updates (no warm consumer adds a pass).
pub fn canonical_universe_complete_update_count() -> usize {
    COMPLETE_UPDATES.load(Ordering::Relaxed)
}

fn build_catching_panics(provider: &UniverseSourceProvider) -> Result<Arc<CanonicalUniverseFormalImage>, CanonicalImageError> {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| image_span(ImagePhase::Build, || build(provider))))
        .map_err(|_| CanonicalImageError::BootstrapPanicked)?
}

fn build(provider: &UniverseSourceProvider) -> Result<Arc<CanonicalUniverseFormalImage>, CanonicalImageError> {
    let private;
    let foundation = if provider.is_canonical() {
        canonical_universe_semantic_baseline()
    } else {
        private = CanonicalUniverseSemanticBaseline::build_with_provider(provider.clone());
        &private
    };
    let modules = Arc::new(CanonicalModuleProducts::capture(provider, foundation.linked.clone()).map_err(CanonicalImageError::ModuleProducts)?);
    let world = CanonicalSemanticWorldId {
        source: modules.source_fingerprint(),
        provenance: Arc::new(modules.sources().iter().map(|(module, unit)| (module.clone(), unit.text.clone())).collect()),
        compatibility: concat!(env!("CARGO_PKG_VERSION"), "/canonical-semantic-module-native-abi-v1"),
    };
    // Explicit foundation path: never ordinary image-backed session creation.
    let mut session = SemanticWorkspaceSession::from_foundation_for_image(foundation, WorkspaceId::from_raw(1));
    let interfaces = modules
        .interfaces()
        .iter()
        .map(|(module, interface)| (module.clone(), (**interface).clone()))
        .collect();
    let locations = modules
        .sources()
        .iter()
        .filter_map(|(module, unit)| unit.source.clone().map(|location| (module.clone(), location)))
        .collect();
    let topology = Arc::new(phalcom_modules::ModuleTopology::from_parts(
        phalcom_modules::ResolverGeneration(0),
        &modules.linked().universe,
        &interfaces,
        &locations,
    ));
    let input = SemanticWorkspaceInput {
        linked: modules.linked().clone(),
        sources: modules.sources().clone(),
        interfaces: Arc::new(modules.interfaces().clone()),
        import_products: Arc::new(BTreeMap::new()),
        module_graph_changed: false,
        import_sites_by_module: Arc::new(BTreeMap::new()),
        require_canonical_import_products: false,
        diagnostics: BTreeMap::new(),
        blocked_modules: BTreeSet::new(),
        generation: 0,
        topology: Some(topology),
        reverse_imports: None,
        product_request: crate::request::SemanticProductRequest::default(),
    };
    let update = image_span(ImagePhase::Bootstrap, || session.update(input))?;
    if provider.is_canonical() {
        COMPLETE_UPDATES.fetch_add(1, Ordering::Relaxed);
    }
    image_counter!(CanonicalImageCompleteUpdates, 1);
    let snapshot = update.snapshot;
    image_span(ImagePhase::Validate, || validate_publication(&snapshot, &modules))?;
    let queries = Arc::new(image_span(ImagePhase::QueryFreeze, || {
        CanonicalSemanticQueryImage::freeze(session.db(), world.clone(), &snapshot, &modules)
    })?);
    let type_prefix = CanonicalTypePrefixId {
        store: snapshot.store.id(),
        types: snapshot.store.len(),
    };
    let image = Arc::new(CanonicalUniverseFormalImage {
        provider: provider.clone(),
        world,
        modules,
        snapshot,
        queries,
        type_prefix,
    });
    if provider.is_canonical() {
        BUILDS.fetch_add(1, Ordering::Relaxed);
    }
    image_counter!(CanonicalImageBuilds, 1);
    Ok(image)
}

fn validate_publication(snapshot: &SemanticSnapshot, modules: &CanonicalModuleProducts) -> Result<(), CanonicalImageError> {
    let missing = if !snapshot.status.is_complete() {
        Some("unblocked final snapshot")
    } else if snapshot.sources.len() != modules.sources().len() || modules.sources().keys().any(|module| !snapshot.sources.contains_key(module)) {
        Some("complete canonical source set")
    } else if snapshot.callable_analyses.is_empty() || snapshot.callable_signatures.is_empty() || snapshot.trait_surfaces.iter().next().is_none() {
        Some("final callable/signature/trait publication")
    } else {
        None
    };
    if let Some(missing) = missing {
        image_counter!(CanonicalImageCompletenessFailures, 1);
        return Err(CanonicalImageError::IncompletePublication(missing.into()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{BudgetKind, BudgetReport};
    use crate::types::outcome::BlockReason;

    fn publish(db: &mut SemanticDb, image: &CanonicalUniverseFormalImage, module: ModuleId, dependencies: Vec<DependencyEdge>) {
        let product = SemanticProduct::ParsedModule(image.sources()[&module].clone());
        db.publish_product_ready(
            QueryKey::ParsedModule(module),
            db.revision(),
            InputFingerprint(7),
            ProductFingerprint(9),
            product,
            dependencies,
        )
        .unwrap();
    }
    fn freeze(db: &SemanticDb, image: &CanonicalUniverseFormalImage) -> CanonicalSemanticQueryImage {
        CanonicalSemanticQueryImage::freeze(db, image.world.clone(), image.snapshot(), image.modules()).unwrap()
    }

    #[test]
    fn canonical_image_database_reads_are_immutable_leaves_across_local_rollback() {
        let image = canonical_universe_formal_image().unwrap().clone();
        let (key, entry) = image.query_image().entries().next().unwrap();
        let key = key.clone();
        let input = entry.input_fingerprint;
        let fingerprint = entry.product_fingerprint;
        let retained = image.reader().unwrap().product(&key).unwrap().clone();
        let mut db = SemanticDb::with_canonical_image(crate::identity::WorkspaceId::from_raw(90), image.clone());
        assert!(db.local_query_state(&key).is_none());
        assert!(db.is_current(&key));
        assert!(db.validate_reuse(&key, input));
        assert!(db.validate_ready(&key));
        assert_eq!(
            db.product_origin(&key),
            Some(crate::db::SemanticProductOrigin::Canonical(image.world().clone()))
        );
        assert!(Arc::ptr_eq(db.product(&key).unwrap(), &retained));
        let local = QueryKey::ParsedModule(ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(99),
            phalcom_modules::identity::ModulePath::root(),
        ));
        let mut recorder = crate::db::DependencyRecorder::new(local.clone());
        db.record_dependency(&mut recorder, key.clone()).unwrap();
        let edges = recorder.finish();
        assert_eq!(edges[0].observed_fingerprint, fingerprint);
        db.begin_transaction();
        db.publish_product_ready(
            local.clone(),
            db.revision(),
            InputFingerprint(1),
            ProductFingerprint(2),
            SemanticProduct::ParsedModule(image.sources().values().next().unwrap().clone()),
            edges,
        )
        .unwrap();
        assert!(db.is_reusable(&local, InputFingerprint(1)));
        assert!(db.invalidate([key.clone()]).is_empty());
        assert!(db.product(&local).is_some());
        assert!(!db.discard_for_recompute(&key));
        assert_eq!(db.retire_query(&key), 0);
        let result = db.publish_product_ready(
            key.clone(),
            db.revision(),
            input,
            fingerprint,
            SemanticProduct::ParsedModule(image.sources().values().next().unwrap().clone()),
            [],
        );
        assert!(result.unwrap_err().is_immutable());
        db.rollback_transaction();
        assert!(db.local_query_state(&local).is_none());
        assert!(db.local_query_state(&key).is_none());
        assert!(Arc::ptr_eq(db.product(&key).unwrap(), &retained));
        assert_eq!(db.query_keys().count(), 0);
    }

    #[test]
    fn canonical_image_freeze_requires_commit_and_classifies_every_nonready_state() {
        let image = canonical_universe_formal_image().unwrap();
        let modules = image.sources().keys().take(6).cloned().collect::<Vec<_>>();
        assert_eq!(modules.len(), 6);
        let mut db = SemanticDb::new();
        db.begin_transaction();
        assert!(matches!(
            CanonicalSemanticQueryImage::freeze(&db, image.world.clone(), image.snapshot(), image.modules()),
            Err(CanonicalImageError::OpenTransaction)
        ));
        db.rollback_transaction();
        let revision = db.revision();
        let states = [
            QueryState::Vacant,
            QueryState::Computing { revision, stack_index: 0 },
            QueryState::Cancelled { revision },
            QueryState::BudgetExceeded {
                revision,
                report: BudgetReport::new(BudgetKind::Steps, 1, 2),
            },
            QueryState::Blocked {
                revision,
                reason: BlockReason::SuppressedDependency,
            },
            QueryState::Failed {
                revision,
                failure: "fixture failure".into(),
            },
        ];
        for (module, state) in modules.iter().zip(states) {
            db.set_state(QueryKey::ParsedModule(module.clone()), state);
        }
        let queries = freeze(&db, image);
        assert!(queries.is_empty());
        assert_eq!(queries.census.total_rows_examined, 6);
        assert_eq!(
            queries.census.terminal_diagnostics[&QueryKey::ParsedModule(modules[5].clone())].as_ref(),
            "fixture failure"
        );
        let reasons = [
            QueryFreezeRejection::Vacant,
            QueryFreezeRejection::Computing,
            QueryFreezeRejection::Cancelled,
            QueryFreezeRejection::BudgetExceeded,
            QueryFreezeRejection::BlockedRequiresClassification,
            QueryFreezeRejection::FailedRequiresClassification,
        ];
        for (module, reason) in modules.iter().zip(reasons) {
            assert_eq!(queries.census.rejections[&QueryKey::ParsedModule(module.clone())], reason);
        }
    }

    #[test]
    fn canonical_image_freeze_closes_dependencies_and_local_discards_do_not_mutate_roots() {
        let image = canonical_universe_formal_image().unwrap();
        let mut modules = image.sources().keys();
        let first = modules.next().unwrap().clone();
        let second = modules.next().unwrap().clone();
        let first_key = QueryKey::ParsedModule(first.clone());
        let second_key = QueryKey::ParsedModule(second.clone());
        let mut db = SemanticDb::new();
        publish(&mut db, image, first.clone(), vec![]);
        publish(
            &mut db,
            image,
            second.clone(),
            vec![DependencyEdge {
                dependent: second_key.clone(),
                dependency: first_key.clone(),
                observed_fingerprint: ProductFingerprint(9),
            }],
        );
        let queries = freeze(&db, image);
        assert_eq!(queries.len(), 2);
        let reader = queries.reader(&image.world, &image.snapshot.store, &image.snapshot.static_values).unwrap();
        let retained = reader.product(&second_key).unwrap().clone();
        db.discard_for_recompute(&first_key);
        db.discard_for_recompute(&second_key);
        assert!(db.local_query_state(&first_key).is_none());
        assert!(db.product_origin(&second_key).is_none());
        assert!(Arc::ptr_eq(reader.product(&second_key).unwrap(), &retained));
        assert_eq!(reader.origin(), crate::db::SemanticProductOrigin::Canonical(image.world.clone()));
        assert_eq!(freeze(&db, image).len(), 0);
        publish(
            &mut db,
            image,
            second,
            vec![DependencyEdge {
                dependent: second_key.clone(),
                dependency: first_key,
                observed_fingerprint: ProductFingerprint(9),
            }],
        );
        assert_eq!(freeze(&db, image).census.rejections[&second_key], QueryFreezeRejection::MutableDependency);
    }

    #[test]
    fn canonical_image_rejects_foreign_stale_and_cyclic_dependencies() {
        let image = canonical_universe_formal_image().unwrap();
        let module = image.sources().keys().next().unwrap().clone();
        let key = QueryKey::ParsedModule(module.clone());
        let mut db = SemanticDb::new();
        publish(
            &mut db,
            image,
            module.clone(),
            vec![DependencyEdge {
                dependent: key.clone(),
                dependency: key.clone(),
                observed_fingerprint: ProductFingerprint(9),
            }],
        );
        assert_eq!(freeze(&db, image).census.rejections[&key], QueryFreezeRejection::DependencyCycle);
        db.discard_for_recompute(&key);
        publish(&mut db, image, module.clone(), vec![]);
        db.begin_revision();
        assert_eq!(freeze(&db, image).census.rejections[&key], QueryFreezeRejection::StaleValidation);
        let foreign = ModuleId::resolved(
            phalcom_modules::ResolvedProjectId::from_raw(55),
            phalcom_modules::ModulePath::from_components(vec![]),
        );
        let foreign_key = QueryKey::ParsedModule(foreign);
        db.publish_product_ready(
            foreign_key.clone(),
            db.revision(),
            InputFingerprint(7),
            ProductFingerprint(9),
            SemanticProduct::ParsedModule(image.sources()[&module].clone()),
            vec![],
        )
        .unwrap();
        assert_eq!(freeze(&db, image).census.rejections[&foreign_key], QueryFreezeRejection::WorkspaceOrigin);
    }

    #[test]
    fn canonical_image_reader_validates_all_final_arena_and_static_roots() {
        let image = canonical_universe_formal_image().unwrap();
        let snapshot = image.snapshot();
        let fork = snapshot.store.fork_from_baseline();
        assert_ne!(fork.id(), snapshot.store.id());
        assert!(image.queries.reader(&image.world, &fork, &snapshot.static_values).is_ok());
        assert!(matches!(
            image.queries.reader(&image.world, &TypeStore::new(), &snapshot.static_values),
            Err(CanonicalImageError::IncompatibleTypePrefix)
        ));
        assert!(matches!(
            image.queries.reader(&image.world, &snapshot.store, &StaticValueStore::new()),
            Err(CanonicalImageError::IncompatibleStaticValueStore)
        ));
        let foreign_world = CanonicalSemanticWorldId {
            source: image.world.source ^ 1,
            ..image.world.clone()
        };
        assert!(matches!(
            image.queries.reader(&foreign_world, &snapshot.store, &snapshot.static_values),
            Err(CanonicalImageError::ForeignSemanticWorld)
        ));
    }

    #[test]
    fn canonical_image_bootstrap_reentry_fails_before_locking() {
        let guard = BootstrapGuard::enter().unwrap();
        assert!(matches!(BootstrapGuard::enter(), Err(CanonicalImageError::BootstrapRecursion)));
        drop(guard);
        assert!(BootstrapGuard::enter().is_ok());
    }
}

#[cfg(test)]
mod negative_fact_tests {
    use super::*;

    #[test]
    fn canonical_image_never_freezes_negative_local_conformance_search() {
        use crate::db::product::GenericConformanceQueryProduct;
        use crate::generic_evidence::{GenericConformanceKey, GenericConformanceProof};
        use crate::types::parameter::TypeTerm;
        let image = canonical_universe_formal_image().unwrap();
        let (declaration, _) = image.snapshot.trait_surfaces.iter().next().unwrap();
        let proposition = GenericConformanceKey::new(
            TypeTerm::Canonical(image.snapshot.store.unit()),
            crate::traits::TraitRef::new(declaration.clone(), Vec::new().into_boxed_slice()),
        );
        let key = QueryKey::GenericConformance(proposition.clone());
        let mut db = SemanticDb::new();
        db.publish_product_ready(
            key.clone(),
            db.revision(),
            InputFingerprint(1),
            ProductFingerprint(2),
            SemanticProduct::GenericConformance(Arc::new(GenericConformanceQueryProduct {
                key: proposition,
                proof: GenericConformanceProof::Missing,
            })),
            vec![],
        )
        .unwrap();
        let frozen = CanonicalSemanticQueryImage::freeze(&db, image.world.clone(), image.snapshot(), image.modules()).unwrap();
        assert!(frozen.is_empty());
        assert_eq!(frozen.census.rejections[&key], QueryFreezeRejection::NegativeLocalCandidateSet);
        // A local candidate-set change retires the local result. It has no
        // immutable canonical negative fact to fall back to.
        db.invalidate([key.clone()]);
        assert!(db.product(&key).is_none());
        assert!(
            frozen
                .reader(image.world(), &image.snapshot.store, &image.snapshot.static_values)
                .unwrap()
                .product(&key)
                .is_none()
        );
    }

    #[test]
    fn canonical_image_exact_world_provenance_rejects_digest_collision() {
        let image = canonical_universe_formal_image().unwrap();
        let mut foreign = image.world.clone();
        let module = image.sources().keys().next().unwrap().clone();
        Arc::make_mut(&mut foreign.provenance).insert(module, Arc::from("different provider source"));
        assert_eq!(foreign.source, image.world.source);
        assert!(matches!(
            image.queries.reader(&foreign, &image.snapshot.store, &image.snapshot.static_values),
            Err(CanonicalImageError::ForeignSemanticWorld)
        ));
    }

    #[test]
    fn canonical_image_static_lineage_id_does_not_hide_divergent_values() {
        let mut first = StaticValueStore::new();
        let mut second = first.clone();
        let first_ref = first.intern_int(1);
        let second_ref = second.intern_int(2);
        assert_eq!(first_ref.store_id(), second_ref.store_id());
        assert_eq!(first_ref.value_id(), second_ref.value_id());
        assert!(!first.extends_canonical_prefix(&second));
        assert!(!second.extends_canonical_prefix(&first));
    }
}
