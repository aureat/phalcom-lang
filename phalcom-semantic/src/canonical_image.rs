//! Completed, immutable Universe semantics and explicitly eligible query roots.
//!
//! Construction uses the ordinary analyzer through the foundation constructor,
//! then freezes only after its database transaction commits. This module never
//! calls the core compiler and never installs its snapshot in a local session.

use crate::canonical_baseline::{CanonicalUniverseSemanticBaseline, canonical_universe_semantic_baseline};
use crate::db::{DependencyEdge, InputFingerprint, ProductFingerprint, QueryKey, QueryKind, QueryState, SemanticDb, SemanticProduct};
use crate::declarations::TypeDeclarationShell;
use crate::identity::{DeclarationId, WorkspaceId};
use crate::session::{SemanticUpdateFailure, SemanticWorkspaceSession};
use crate::snapshot::SemanticSnapshot;
use crate::static_value::{StaticValueStore, StaticValueStoreId};
use crate::types::id::TypeStoreId;
use crate::types::store::TypeStore;
use crate::workspace::SemanticWorkspaceInput;
use phalcom_modules::canonical_products::{CanonicalExternalModuleWorld, CanonicalModuleProducts};
use phalcom_modules::{ModuleId, UniverseSourceProvider};
use phalcom_type_meta::allocation::{AllocationBudget, BudgetedSortedMap, SortedMapEntriesError};
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use thiserror::Error;

const LOADED_PUBLICATION_CENSUS_PHASES: usize = 10;
static LOADED_PUBLICATION_CENSUS_ACTIVE: AtomicBool = AtomicBool::new(false);
static LOADED_PUBLICATION_CENSUS_OBSERVED: [AtomicUsize; LOADED_PUBLICATION_CENSUS_PHASES] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static LOADED_PUBLICATION_CENSUS_LEDGER: [AtomicUsize; LOADED_PUBLICATION_CENSUS_PHASES] = [
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
    AtomicUsize::new(0),
];
static LOADED_FINGERPRINT_AGGREGATE_OBSERVED: AtomicUsize = AtomicUsize::new(0);
static LOADED_FINGERPRINT_QUERY_LOOKUP_OBSERVED: AtomicUsize = AtomicUsize::new(0);
static LOADED_FINGERPRINT_AGGREGATE_BY_KIND: [AtomicUsize; QueryKind::ALL.len()] = [const { AtomicUsize::new(0) }; QueryKind::ALL.len()];

#[cfg(test)]
std::thread_local! {
    static LOADED_FORMAL_IMAGE_FINAL_ARC_ATTEMPTED: Cell<bool> = const { Cell::new(false) };
}

#[doc(hidden)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct LoadedPublicationCensus {
    pub snapshot_clone: usize,
    pub snapshot_clone_ledger: usize,
    pub status_promotion: usize,
    pub status_promotion_ledger: usize,
    pub module_query_products: usize,
    pub module_query_products_ledger: usize,
    pub module_query_products_publication: usize,
    pub module_query_products_publication_ledger: usize,
    pub source_free_completeness: usize,
    pub source_free_completeness_ledger: usize,
    pub advisory_validation: usize,
    pub advisory_validation_ledger: usize,
    pub query_reader_validation: usize,
    pub query_reader_validation_ledger: usize,
    pub fingerprint_validation: usize,
    pub fingerprint_validation_ledger: usize,
    pub snapshot_arc: usize,
    pub snapshot_arc_ledger: usize,
    pub formal_image_arc: usize,
    pub formal_image_arc_ledger: usize,
    pub fingerprint_aggregate_observed: usize,
    pub fingerprint_query_lookup_observed: usize,
    pub fingerprint_aggregate_by_kind: Vec<(QueryKind, usize)>,
}

#[doc(hidden)]
pub fn reset_loaded_publication_census() {
    LOADED_PUBLICATION_CENSUS_ACTIVE.store(false, Ordering::Relaxed);
    for row in &LOADED_PUBLICATION_CENSUS_OBSERVED {
        row.store(0, Ordering::Relaxed);
    }
    for row in &LOADED_PUBLICATION_CENSUS_LEDGER {
        row.store(0, Ordering::Relaxed);
    }
    LOADED_FINGERPRINT_AGGREGATE_OBSERVED.store(0, Ordering::Relaxed);
    LOADED_FINGERPRINT_QUERY_LOOKUP_OBSERVED.store(0, Ordering::Relaxed);
    for row in &LOADED_FINGERPRINT_AGGREGATE_BY_KIND {
        row.store(0, Ordering::Relaxed);
    }
    LOADED_PUBLICATION_CENSUS_ACTIVE.store(true, Ordering::Relaxed);
}

#[doc(hidden)]
pub fn take_loaded_publication_census() -> LoadedPublicationCensus {
    LOADED_PUBLICATION_CENSUS_ACTIVE.store(false, Ordering::Relaxed);
    let row = |index: usize| LOADED_PUBLICATION_CENSUS_OBSERVED[index].load(Ordering::Relaxed);
    let ledger = |index: usize| LOADED_PUBLICATION_CENSUS_LEDGER[index].load(Ordering::Relaxed);
    LoadedPublicationCensus {
        snapshot_clone: row(0),
        snapshot_clone_ledger: ledger(0),
        status_promotion: row(1),
        status_promotion_ledger: ledger(1),
        module_query_products: row(2),
        module_query_products_ledger: ledger(2),
        module_query_products_publication: row(3),
        module_query_products_publication_ledger: ledger(3),
        source_free_completeness: row(4),
        source_free_completeness_ledger: ledger(4),
        advisory_validation: row(5),
        advisory_validation_ledger: ledger(5),
        query_reader_validation: row(6),
        query_reader_validation_ledger: ledger(6),
        fingerprint_validation: row(7),
        fingerprint_validation_ledger: ledger(7),
        snapshot_arc: row(8),
        snapshot_arc_ledger: ledger(8),
        formal_image_arc: row(9),
        formal_image_arc_ledger: ledger(9),
        fingerprint_aggregate_observed: LOADED_FINGERPRINT_AGGREGATE_OBSERVED.load(Ordering::Relaxed),
        fingerprint_query_lookup_observed: LOADED_FINGERPRINT_QUERY_LOOKUP_OBSERVED.load(Ordering::Relaxed),
        fingerprint_aggregate_by_kind: QueryKind::ALL
            .iter()
            .enumerate()
            .filter_map(|(index, kind)| {
                let observed = LOADED_FINGERPRINT_AGGREGATE_BY_KIND[index].load(Ordering::Relaxed);
                (observed != 0).then_some((*kind, observed))
            })
            .collect(),
    }
}

fn publication_census_checkpoint(allocations: &phalcom_type_meta::allocation::AllocationBudget) -> Option<(usize, usize)> {
    if !LOADED_PUBLICATION_CENSUS_ACTIVE.load(Ordering::Relaxed) {
        return None;
    }
    Some((phalcom_type_meta::allocation::current_observed_bytes()?, allocations.requested()))
}

fn record_publication_census(phase: usize, before: Option<(usize, usize)>, allocations: &phalcom_type_meta::allocation::AllocationBudget) {
    if let (Some((observed_before, ledger_before)), Some((observed_after, ledger_after))) = (before, publication_census_checkpoint(allocations)) {
        LOADED_PUBLICATION_CENSUS_OBSERVED[phase].fetch_add(observed_after - observed_before, Ordering::Relaxed);
        LOADED_PUBLICATION_CENSUS_LEDGER[phase].fetch_add(ledger_after - ledger_before, Ordering::Relaxed);
    }
}

/// Provenance backing an in-process semantic world.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CanonicalWorldProvenance {
    /// Source-backed world with complete retained source text.
    Source(Arc<BTreeMap<ModuleId, Arc<str>>>),
    /// Artifact-backed world identified by portable digests and compatibility identity.
    Artifact {
        source_digest: phalcom_compiler_artifact::ArtifactDigest,
        compatibility: phalcom_compiler_artifact::CanonicalCompatibilityIdentity,
    },
}

/// In-process semantic world provenance. This is not a portable artifact digest.
/// Source/interface meaning, module/semantic ABI, and native ABI are fixed by the
/// running binary. Overridden provider source content contributes its own digest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CanonicalSemanticWorldId {
    pub(crate) source: u64,
    // Exact identities/text roots prevent fingerprint collisions from making
    // distinct private providers a compatible semantic world.
    pub(crate) provenance: CanonicalWorldProvenance,
    pub(crate) compatibility: &'static str,
}

impl CanonicalSemanticWorldId {
    pub fn for_source_world(source: u64, provenance: Arc<BTreeMap<ModuleId, Arc<str>>>) -> Self {
        Self {
            source,
            provenance: CanonicalWorldProvenance::Source(provenance),
            compatibility: concat!(env!("CARGO_PKG_VERSION"), "/canonical-semantic-module-native-abi-v1"),
        }
    }

    pub fn for_artifact_world(
        source: u64,
        source_digest: phalcom_compiler_artifact::ArtifactDigest,
        compatibility: phalcom_compiler_artifact::CanonicalCompatibilityIdentity,
    ) -> Self {
        Self {
            source,
            provenance: CanonicalWorldProvenance::Artifact { source_digest, compatibility },
            compatibility: concat!(env!("CARGO_PKG_VERSION"), "/canonical-semantic-module-native-abi-v1"),
        }
    }

    pub fn source_fingerprint(&self) -> u64 {
        self.source
    }

    pub fn provenance(&self) -> &CanonicalWorldProvenance {
        &self.provenance
    }

    pub fn portable_compatibility(&self) -> Option<&phalcom_compiler_artifact::CanonicalCompatibilityIdentity> {
        match &self.provenance {
            CanonicalWorldProvenance::Artifact { compatibility, .. } => Some(compatibility),
            _ => None,
        }
    }

    pub(crate) fn clone_with_allocation_budget(&self, allocations: &mut AllocationBudget) -> Result<Self, phalcom_type_meta::allocation::AllocationError> {
        let provenance = match &self.provenance {
            CanonicalWorldProvenance::Source(sources) => CanonicalWorldProvenance::Source(Arc::clone(sources)),
            CanonicalWorldProvenance::Artifact { source_digest, compatibility } => {
                let compatibility = compatibility.clone_with_allocation_budget(allocations)?;
                CanonicalWorldProvenance::Artifact {
                    source_digest: *source_digest,
                    compatibility,
                }
            }
        };
        Ok(Self {
            source: self.source,
            provenance,
            compatibility: self.compatibility,
        })
    }
}

/// Identity and extent of the final canonical arena, not the bootstrap arena.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalTypePrefixId {
    store: TypeStoreId,
    types: usize,
}

impl CanonicalTypePrefixId {
    pub fn store(&self) -> TypeStoreId {
        self.store
    }

    pub fn type_count(&self) -> usize {
        self.types
    }
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
    /// A portable artifact failed bounded structural or allocation validation.
    #[error("canonical artifact validation: {0}")]
    Artifact(#[from] phalcom_compiler_artifact::ArtifactError),
    /// Loaded publication exceeded or failed its explicit allocation ledger.
    #[error("canonical artifact publication allocation: {0}")]
    Allocation(#[from] phalcom_type_meta::allocation::AllocationError),
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
    /// Query content no longer agrees with the finalized structural authority.
    FinalAggregateMismatch,
    /// A dependency is not retained or its observed fingerprint changed.
    MutableDependency,
    /// Cyclic dependencies lack an independently validated freeze contract.
    DependencyCycle,
}

/// Family-level persistence eligibility shared by in-memory freezing and the
/// future artifact projection. Eligibility alone does not make a row reusable:
/// freeze still verifies provenance, final content and dependency closure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalQueryPersistence {
    /// A structural positive result may be retained after row validation.
    EligibleStructural,
    /// A proof or body can observe the extensible local candidate set.
    OpenWorld,
    /// No independently approved relocation/immutability contract exists.
    Unsupported,
}

/// Single family classification for both canonical freeze and artifact planning.
pub const fn canonical_query_persistence(kind: QueryKind) -> CanonicalQueryPersistence {
    match kind {
        QueryKind::ParsedModule
        | QueryKind::UnlinkedInterface
        | QueryKind::ResolvedImports
        | QueryKind::LinkedInterface
        | QueryKind::DeclarationShell
        | QueryKind::HierarchyEdge
        | QueryKind::CallableDefinition
        | QueryKind::CallableSignature
        | QueryKind::FieldSignature
        | QueryKind::DeclarationSurface
        | QueryKind::TraitHeader
        | QueryKind::TraitSurface
        | QueryKind::DataDeclaration
        | QueryKind::EnumDeclaration
        | QueryKind::AssociatedSurface
        | QueryKind::TypeMemberDeclaration
        | QueryKind::ModuleStaticBindingCatalog
        | QueryKind::ModuleStaticBindings => CanonicalQueryPersistence::EligibleStructural,
        QueryKind::GenericConformance
        | QueryKind::ConformanceDispatch
        | QueryKind::TraitDispatchBucket
        | QueryKind::CallableBody
        | QueryKind::CheckedStaticBody => CanonicalQueryPersistence::OpenWorld,
        QueryKind::SemanticComponent
        | QueryKind::ConstructorInitialization
        | QueryKind::AttributeArgument
        | QueryKind::CallableEffects
        | QueryKind::CallableControl
        | QueryKind::CallableTermination
        | QueryKind::CallableContracts
        | QueryKind::VerificationConditions
        | QueryKind::SourceStructure
        | QueryKind::SourceFormalAttachment
        | QueryKind::AdvisoryCallable
        | QueryKind::AdvisoryModule
        | QueryKind::ModuleDiagnostics
        | QueryKind::ModuleMetadata
        | QueryKind::EnumRequirements
        | QueryKind::StaticDomainCaseCatalog
        | QueryKind::StaticCaseSignature
        | QueryKind::AssociatedStaticDomainDeclaration
        | QueryKind::ResolvedImport
        | QueryKind::LinkedName
        | QueryKind::PublicExport
        | QueryKind::StaticDefinition
        | QueryKind::StaticConstant => CanonicalQueryPersistence::Unsupported,
    }
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
            Self::FinalAggregateMismatch => PerfCounter::CanonicalQueryRejectedFinalAggregateMismatch,
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
    pub retained_by_kind: CanonicalQueryKindCounts,
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

/// Read-only retained counts, with a budgeted contiguous backing for loaded
/// artifact images and the ordinary tree backing for source-built images.
#[derive(Debug)]
pub enum CanonicalQueryKindCounts {
    Source(BTreeMap<QueryKind, usize>),
    Frozen(Arc<BudgetedSortedMap<QueryKind, usize>>),
}

impl Default for CanonicalQueryKindCounts {
    fn default() -> Self {
        Self::Source(BTreeMap::new())
    }
}

impl CanonicalQueryKindCounts {
    fn increment_source(&mut self, kind: QueryKind) {
        let Self::Source(counts) = self else {
            unreachable!("source query freezing uses a mutable source census")
        };
        *counts.entry(kind).or_default() += 1;
    }

    pub fn get(&self, kind: &QueryKind) -> Option<&usize> {
        match self {
            Self::Source(counts) => counts.get(kind),
            Self::Frozen(counts) => counts.get(kind),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&QueryKind, &usize)> {
        match self {
            Self::Source(counts) => CanonicalQueryKindCountsIter::Source(counts.iter()),
            Self::Frozen(counts) => CanonicalQueryKindCountsIter::Frozen(counts.as_entries().iter()),
        }
    }

    fn from_loaded_entries(entries: &CanonicalQueryEntries, allocations: &mut AllocationBudget) -> Result<Self, CanonicalImageError> {
        let mut distinct_kinds = 0usize;
        let mut previous = None;
        for (key, _) in entries.iter() {
            let kind = key.kind();
            if previous != Some(kind) {
                distinct_kinds += 1;
                previous = Some(kind);
            }
        }
        let mut counts = allocations.vector::<(QueryKind, usize)>(distinct_kinds)?;
        previous = None;
        for (key, _) in entries.iter() {
            let kind = key.kind();
            if previous == Some(kind) {
                counts.last_mut().expect("the first query kind is inserted before reuse").1 += 1;
            } else {
                counts.push((kind, 1));
                previous = Some(kind);
            }
        }
        let counts = BudgetedSortedMap::from_sorted_entries(counts).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey | SortedMapEntriesError::OutOfOrder => {
                CanonicalImageError::IncompletePublication("query kind order is not canonical".into())
            }
        })?;
        Ok(Self::Frozen(allocations.arc(counts)?))
    }
}

enum CanonicalQueryKindCountsIter<'a> {
    Source(std::collections::btree_map::Iter<'a, QueryKind, usize>),
    Frozen(std::slice::Iter<'a, (QueryKind, usize)>),
}

impl<'a> Iterator for CanonicalQueryKindCountsIter<'a> {
    type Item = (&'a QueryKind, &'a usize);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(kind, count)| (kind, count)),
        }
    }
}

/// Immutable ordered query-entry authority. Artifact imports retain a charged
/// sorted buffer instead of rebuilding native tree nodes.
#[derive(Debug)]
pub(crate) enum CanonicalQueryEntries {
    Source(BTreeMap<QueryKey, CanonicalQueryEntry>),
    Frozen(Arc<BudgetedSortedMap<QueryKey, CanonicalQueryEntry>>),
}

impl CanonicalQueryEntries {
    fn get(&self, key: &QueryKey) -> Option<&CanonicalQueryEntry> {
        match self {
            Self::Source(entries) => entries.get(key),
            Self::Frozen(entries) => entries.get(key),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Source(entries) => entries.len(),
            Self::Frozen(entries) => entries.len(),
        }
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn iter(&self) -> CanonicalQueryEntriesIter<'_> {
        match self {
            Self::Source(entries) => CanonicalQueryEntriesIter::Source(entries.iter()),
            Self::Frozen(entries) => CanonicalQueryEntriesIter::Frozen(entries.as_entries().iter()),
        }
    }

    fn entry_at(&self, index: usize) -> Option<(&QueryKey, &CanonicalQueryEntry)> {
        match self {
            Self::Source(_) => None,
            Self::Frozen(entries) => entries.as_entries().get(index).map(|(key, entry)| (key, entry)),
        }
    }

    fn index_of(&self, key: &QueryKey) -> Option<usize> {
        match self {
            Self::Source(_) => None,
            Self::Frozen(entries) => entries.as_entries().binary_search_by(|(candidate, _)| candidate.cmp(key)).ok(),
        }
    }

    pub(crate) fn from_loaded_entries(
        mut entries: Vec<(QueryKey, CanonicalQueryEntry)>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, CanonicalImageError> {
        entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let entries = BudgetedSortedMap::from_sorted_entries(entries).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey => CanonicalImageError::IncompletePublication("duplicate reconstructed query owner".into()),
            SortedMapEntriesError::OutOfOrder => unreachable!("query entries were sorted before frozen construction"),
        })?;
        Ok(Self::Frozen(allocations.arc(entries)?))
    }
}

enum CanonicalQueryEntriesIter<'a> {
    Source(std::collections::btree_map::Iter<'a, QueryKey, CanonicalQueryEntry>),
    Frozen(std::slice::Iter<'a, (QueryKey, CanonicalQueryEntry)>),
}

impl<'a> Iterator for CanonicalQueryEntriesIter<'a> {
    type Item = (&'a QueryKey, &'a CanonicalQueryEntry);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(key, entry)| (key, entry)),
        }
    }
}

/// All exact dependency fingerprints retained by a formal image.
#[derive(Debug)]
pub enum CanonicalDependencyFingerprints {
    Source(BTreeMap<QueryKey, ProductFingerprint>),
    Frozen(Arc<BudgetedSortedMap<QueryKey, ProductFingerprint>>),
}

impl CanonicalDependencyFingerprints {
    pub fn get(&self, key: &QueryKey) -> Option<&ProductFingerprint> {
        match self {
            Self::Source(values) => values.get(key),
            Self::Frozen(values) => values.get(key),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&QueryKey, &ProductFingerprint)> {
        match self {
            Self::Source(values) => CanonicalDependencyFingerprintsIter::Source(values.iter()),
            Self::Frozen(values) => CanonicalDependencyFingerprintsIter::Frozen(values.as_entries().iter()),
        }
    }

    pub fn keys(&self) -> impl Iterator<Item = &QueryKey> {
        self.iter().map(|(key, _)| key)
    }

    pub(crate) fn from_loaded_entries(
        mut values: Vec<(QueryKey, ProductFingerprint)>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, CanonicalImageError> {
        values.sort_unstable_by(|left, right| left.0.cmp(&right.0));
        let values = BudgetedSortedMap::from_sorted_entries(values).map_err(|error| match error {
            SortedMapEntriesError::DuplicateKey => CanonicalImageError::IncompletePublication("duplicate dependency fingerprint owner".into()),
            SortedMapEntriesError::OutOfOrder => unreachable!("fingerprints were sorted before frozen construction"),
        })?;
        Ok(Self::Frozen(allocations.arc(values)?))
    }
}

enum CanonicalDependencyFingerprintsIter<'a> {
    Source(std::collections::btree_map::Iter<'a, QueryKey, ProductFingerprint>),
    Frozen(std::slice::Iter<'a, (QueryKey, ProductFingerprint)>),
}

impl<'a> Iterator for CanonicalDependencyFingerprintsIter<'a> {
    type Item = (&'a QueryKey, &'a ProductFingerprint);

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Source(iter) => iter.next(),
            Self::Frozen(iter) => iter.next().map(|(key, value)| (key, value)),
        }
    }
}

/// Read-only query roots with their owning final arena and static-value root.
#[derive(Debug)]
pub struct CanonicalSemanticQueryImage {
    world: CanonicalSemanticWorldId,
    entries: CanonicalQueryEntries,
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

    /// Constructs an immutable query image from validated artifact entries without a mutable `SemanticDb` (`P7.T9`).
    pub(crate) fn from_loaded_artifact_entries(
        world: CanonicalSemanticWorldId,
        entries: CanonicalQueryEntries,
        store: Arc<TypeStore>,
        static_values: Arc<StaticValueStore>,
        allocations: &mut AllocationBudget,
    ) -> Result<Self, CanonicalImageError> {
        let mut census = CanonicalQueryCensus {
            total_rows_examined: entries.len(),
            ..CanonicalQueryCensus::default()
        };
        for (key, entry) in entries.iter() {
            if matches!(key, QueryKey::ParsedModule(_)) || canonical_query_persistence(key.kind()) != CanonicalQueryPersistence::EligibleStructural {
                return Err(CanonicalImageError::IncompletePublication(format!(
                    "unsupported or source-dependent query in artifact image: {key:?}"
                )));
            }
            for edge in entry.dependencies.iter() {
                if &edge.dependent != key {
                    return Err(CanonicalImageError::IncompletePublication(format!(
                        "malformed dependency edge dependent for {key:?}"
                    )));
                }
                let Some(dep_entry) = entries.get(&edge.dependency) else {
                    return Err(CanonicalImageError::IncompletePublication(format!(
                        "unclosed query dependency {:?} -> {:?}",
                        key, edge.dependency
                    )));
                };
                if dep_entry.product_fingerprint != edge.observed_fingerprint {
                    return Err(CanonicalImageError::IncompletePublication(format!(
                        "stale observed dependency fingerprint on {:?} -> {:?}",
                        key, edge.dependency
                    )));
                }
            }
        }
        let mut colors = allocations.vector::<u8>(entries.len())?;
        colors.resize(entries.len(), 0);
        let mut stack = allocations.vector::<(usize, usize)>(entries.len())?;
        for start in 0..entries.len() {
            if colors[start] != 0 {
                continue;
            }
            colors[start] = 1;
            stack.push((start, 0));
            while let Some(&(node, next_edge)) = stack.last() {
                let (_, entry) = entries
                    .entry_at(node)
                    .ok_or_else(|| CanonicalImageError::IncompletePublication("loaded query entries are not frozen in sorted order".into()))?;
                if next_edge < entry.dependencies.len() {
                    stack.last_mut().expect("traversal stack is nonempty").1 += 1;
                    let dependency = &entry.dependencies[next_edge].dependency;
                    let target = entries
                        .index_of(dependency)
                        .ok_or_else(|| CanonicalImageError::IncompletePublication(format!("unclosed query dependency from node {node}")))?;
                    match colors[target] {
                        0 => {
                            colors[target] = 1;
                            stack.push((target, 0));
                        }
                        1 => {
                            return Err(CanonicalImageError::IncompletePublication(
                                "cyclic query dependency graph in loaded artifact".into(),
                            ));
                        }
                        2 => {}
                        _ => unreachable!("query traversal colors are limited to 0, 1, and 2"),
                    }
                } else {
                    colors[node] = 2;
                    stack.pop();
                }
            }
        }
        census.retained_by_kind = CanonicalQueryKindCounts::from_loaded_entries(&entries, allocations)?;
        Ok(Self {
            world,
            entries,
            census,
            store,
            static_values,
        })
    }

    fn freeze(
        db: &SemanticDb,
        world: CanonicalSemanticWorldId,
        snapshot: &Arc<SemanticSnapshot>,
        modules: &CanonicalExternalModuleWorld,
    ) -> Result<Self, CanonicalImageError> {
        if db.has_open_transaction() {
            return Err(CanonicalImageError::OpenTransaction);
        }
        validate_final_declaration_shell_authority(db, snapshot)?;
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
                if !crate::db::query_key_module(key).is_some_and(|module| modules.linked().modules.contains_key(module)) {
                    return Some(QueryFreezeRejection::WorkspaceOrigin);
                }
                match canonical_query_persistence(key.kind()) {
                    CanonicalQueryPersistence::EligibleStructural => None,
                    CanonicalQueryPersistence::OpenWorld => Some(QueryFreezeRejection::NegativeLocalCandidateSet),
                    CanonicalQueryPersistence::Unsupported => Some(QueryFreezeRejection::UnsupportedQueryKind),
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
                    | (QueryKey::ModuleStaticBindings(_), SemanticProduct::ModuleStaticBindings(_))
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
            if matches!(
                key,
                QueryKey::CallableSignature(_) | QueryKey::FieldSignature(_) | QueryKey::DeclarationSurface(_) | QueryKey::TypeMemberDeclaration(_)
            ) && aggregate_structural_fingerprint(snapshot, key) != Some(*product_fingerprint)
            {
                census.rejections.insert(key.clone(), QueryFreezeRejection::FinalAggregateMismatch);
                continue;
            }
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
            census.retained_by_kind.increment_source(key.kind());
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
            entries: CanonicalQueryEntries::Source(entries),
            census,
            store: snapshot.store.clone(),
            static_values: snapshot.static_values.clone(),
        })
    }
}

fn validate_final_declaration_shell_authority(db: &SemanticDb, snapshot: &SemanticSnapshot) -> Result<(), CanonicalImageError> {
    for key in db.query_keys() {
        let QueryKey::DeclarationShell(owner) = key else {
            continue;
        };
        if db.query_state(key).and_then(QueryState::validated_revision) != Some(db.revision()) {
            continue;
        }
        let nominal = snapshot.declarations.get(owner).is_some();
        let alias = snapshot.type_aliases.get(owner).is_some();
        let shell = match db.product(key).map(AsRef::as_ref) {
            Some(SemanticProduct::DeclarationShell(shell)) => Some(shell.as_ref()),
            _ => None,
        };
        validate_final_declaration_shell_owner(owner, nominal, alias, shell)?;
    }
    Ok(())
}

fn validate_final_declaration_shell_owner(
    owner: &DeclarationId,
    nominal: bool,
    alias: bool,
    shell: Option<&TypeDeclarationShell>,
) -> Result<(), CanonicalImageError> {
    if nominal == alias {
        return Err(CanonicalImageError::IncompletePublication(format!(
            "ready declaration-shell query owner {owner:?} has {} final authorities",
            usize::from(nominal) + usize::from(alias)
        )));
    }
    let Some(shell) = shell else {
        return Err(CanonicalImageError::IncompletePublication(format!(
            "ready declaration-shell query owner {owner:?} has no typed shell product"
        )));
    };
    if shell.declaration() != owner {
        return Err(CanonicalImageError::IncompletePublication(format!(
            "ready declaration-shell query owner {owner:?} disagrees with its native product"
        )));
    }
    match shell {
        TypeDeclarationShell::Nominal(_) if nominal && !alias => Ok(()),
        TypeDeclarationShell::Alias(_) if alias && !nominal => Ok(()),
        _ => Err(CanonicalImageError::IncompletePublication(format!(
            "ready declaration-shell query owner {owner:?} variant disagrees with final semantic authority"
        ))),
    }
}

/// P5 persistence classification: a planning contract, not an assertion that
/// any corresponding process-local Rust object can be serialized unchanged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CanonicalPersistenceClass {
    RequiredSemantic,
    RequiredMetadata,
    EligibleQuery,
    OptionalPresentation,
    OptionalSourceMap,
    Rebuildable,
    ProcessLocal,
    NeverPersist,
}

/// Independently classified canonical product families. P7 must project
/// required products to relocatable DTOs, not serialize these objects in place.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum CanonicalProductFamily {
    ModulesAndInterfaces,
    ImportsAndTopology,
    TypeArena,
    StaticValueArena,
    DeclarationsAndAliases,
    HierarchyAndDispatch,
    CallableSignaturesAndDefinitions,
    CallableAnalyses,
    TraitsAndConformance,
    ConformanceWitnessesAndDispatch,
    DataAndEnums,
    AssociatedAndStaticSemantics,
    FrozenStructuralQueries,
    RuntimePublicMetadata,
    CompilerSourceAttachments,
    EditorPresentation,
    AdvisoryProducts,
    SourceMaps,
    Diagnostics,
    WorkspaceRevisionHistory,
    MutableSemanticDatabase,
    RuntimeHandles,
}

/// The inventory describes ownership and projected artifact policy.
/// RuntimePublic metadata is core-owned and composed with this image, not
/// embedded into the semantic snapshot. This inventory does not supply it.
pub const CANONICAL_PERSISTENCE_INVENTORY: &[(CanonicalProductFamily, CanonicalPersistenceClass)] = &[
    (CanonicalProductFamily::ModulesAndInterfaces, CanonicalPersistenceClass::RequiredSemantic),
    (CanonicalProductFamily::ImportsAndTopology, CanonicalPersistenceClass::RequiredSemantic),
    (CanonicalProductFamily::TypeArena, CanonicalPersistenceClass::RequiredSemantic),
    (CanonicalProductFamily::StaticValueArena, CanonicalPersistenceClass::RequiredSemantic),
    (CanonicalProductFamily::DeclarationsAndAliases, CanonicalPersistenceClass::RequiredSemantic),
    (CanonicalProductFamily::HierarchyAndDispatch, CanonicalPersistenceClass::RequiredSemantic),
    (
        CanonicalProductFamily::CallableSignaturesAndDefinitions,
        CanonicalPersistenceClass::RequiredSemantic,
    ),
    (CanonicalProductFamily::CallableAnalyses, CanonicalPersistenceClass::RequiredSemantic),
    (CanonicalProductFamily::TraitsAndConformance, CanonicalPersistenceClass::RequiredSemantic),
    (
        CanonicalProductFamily::ConformanceWitnessesAndDispatch,
        CanonicalPersistenceClass::RequiredSemantic,
    ),
    (CanonicalProductFamily::DataAndEnums, CanonicalPersistenceClass::RequiredSemantic),
    (
        CanonicalProductFamily::AssociatedAndStaticSemantics,
        CanonicalPersistenceClass::RequiredSemantic,
    ),
    (CanonicalProductFamily::FrozenStructuralQueries, CanonicalPersistenceClass::EligibleQuery),
    (CanonicalProductFamily::RuntimePublicMetadata, CanonicalPersistenceClass::RequiredMetadata),
    // Until P7 proves source-independent regeneration, compiler-facing source
    // attachments are required. They cannot be silently rebuilt from absent
    // canonical syntax in an artifact-only process.
    (CanonicalProductFamily::CompilerSourceAttachments, CanonicalPersistenceClass::RequiredSemantic),
    (CanonicalProductFamily::EditorPresentation, CanonicalPersistenceClass::OptionalPresentation),
    (CanonicalProductFamily::AdvisoryProducts, CanonicalPersistenceClass::Rebuildable),
    (CanonicalProductFamily::SourceMaps, CanonicalPersistenceClass::OptionalSourceMap),
    (CanonicalProductFamily::Diagnostics, CanonicalPersistenceClass::NeverPersist),
    (CanonicalProductFamily::WorkspaceRevisionHistory, CanonicalPersistenceClass::NeverPersist),
    (CanonicalProductFamily::MutableSemanticDatabase, CanonicalPersistenceClass::NeverPersist),
    (CanonicalProductFamily::RuntimeHandles, CanonicalPersistenceClass::ProcessLocal),
];

/// Typed, in-process P5 capability census. These values are deliberately not
/// stable artifact IDs or compatibility digests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CanonicalImageCapabilities {
    pub module_count: usize,
    pub frozen_query_count: usize,
    pub canonical_type_count: usize,
    pub runtime_metadata_composed: bool,
    pub portable_artifact_available: bool,
}

/// A successfully committed complete semantic publication and its immutable roots.
#[derive(Debug)]
pub struct CanonicalUniverseFormalImage {
    world: CanonicalSemanticWorldId,
    modules: Arc<CanonicalExternalModuleWorld>,
    build_sources: BTreeMap<ModuleId, Arc<phalcom_modules::ParsedModuleUnit>>,
    presentation_provider: Option<UniverseSourceProvider>,
    snapshot: Arc<SemanticSnapshot>,
    queries: Arc<CanonicalSemanticQueryImage>,
    dependency_fingerprints: CanonicalDependencyFingerprints,
    type_prefix: CanonicalTypePrefixId,
}

impl CanonicalUniverseFormalImage {
    /// Exact immutable dependency evidence, independent of query freeze eligibility.
    /// Aggregate-only entries do not imply a reusable query or typed query product.
    pub fn dependency_fingerprint(&self, key: &QueryKey) -> Option<ProductFingerprint> {
        self.queries
            .entry(key)
            .map(|entry| entry.product_fingerprint)
            .or_else(|| self.dependency_fingerprints.get(key).copied())
            .or_else(|| aggregate_structural_fingerprint(&self.snapshot, key))
    }

    /// Optional source-backed tooling capability; never used for canonical linking.
    pub(crate) fn presentation_provider(&self) -> Option<&UniverseSourceProvider> {
        self.presentation_provider.as_ref()
    }
    /// Provenance used to validate immutable-origin reads.
    pub fn world(&self) -> &CanonicalSemanticWorldId {
        &self.world
    }
    /// Complete authoritative final snapshot; all semantic families stay here.
    pub fn snapshot(&self) -> &Arc<SemanticSnapshot> {
        &self.snapshot
    }
    /// Module-layer P1 dependency adapter.
    pub fn modules(&self) -> &Arc<CanonicalExternalModuleWorld> {
        &self.modules
    }
    /// Complete parsed source set.
    pub fn sources(&self) -> &BTreeMap<ModuleId, Arc<phalcom_modules::ParsedModuleUnit>> {
        &self.build_sources
    }
    /// Provider-derived module interfaces.
    pub fn interfaces(&self) -> &phalcom_modules::authority::ModuleAuthorityMap<ModuleId, Arc<phalcom_modules::UnlinkedModuleInterface>> {
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
    /// Dependency fingerprints for all aggregated queries.
    pub fn dependency_fingerprints(&self) -> &CanonicalDependencyFingerprints {
        &self.dependency_fingerprints
    }
    /// Final arena identity and extent.
    pub fn type_prefix(&self) -> CanonicalTypePrefixId {
        self.type_prefix
    }
    /// Final static-value lineage.
    pub fn static_lineage(&self) -> StaticValueStoreId {
        self.snapshot.static_values.id()
    }
    /// Current in-memory capabilities; metadata composition belongs to core
    /// and portable artifact validation belongs to P7.
    pub fn capabilities(&self) -> CanonicalImageCapabilities {
        CanonicalImageCapabilities {
            module_count: self.modules.linked().modules.len(),
            frozen_query_count: self.queries.len(),
            canonical_type_count: self.type_prefix.types,
            runtime_metadata_composed: false,
            portable_artifact_available: matches!(self.world.provenance, CanonicalWorldProvenance::Artifact { .. }),
        }
    }
    /// Reader using the image's own validated arenas.
    pub fn reader(&self) -> Result<CanonicalQueryReader<'_>, CanonicalImageError> {
        self.queries.reader(&self.world, &self.snapshot.store, &self.snapshot.static_values)
    }

    /// Atomically constructs a source-free `CanonicalUniverseFormalImage` from validated loaded artifact parts (`P7.T9/T12`).
    pub fn from_loaded_artifact(
        world: CanonicalSemanticWorldId,
        modules: Arc<CanonicalExternalModuleWorld>,
        snapshot: Arc<SemanticSnapshot>,
        queries: Arc<CanonicalSemanticQueryImage>,
        dependency_fingerprints: CanonicalDependencyFingerprints,
        allocations: &mut phalcom_type_meta::allocation::AllocationBudget,
    ) -> Result<Arc<Self>, CanonicalImageError> {
        if !matches!(world.provenance, CanonicalWorldProvenance::Artifact { .. }) {
            return Err(CanonicalImageError::ForeignSemanticWorld);
        }
        let mut transaction = allocations.child(allocations.remaining());
        let snapshot_start = publication_census_checkpoint(&transaction);
        let mut snapshot_obj = (*snapshot).clone();
        record_publication_census(0, snapshot_start, &transaction);
        if snapshot_obj.module_products.unlinked.is_empty() {
            let module_products_start = publication_census_checkpoint(&transaction);
            let module_products = crate::snapshot::ModuleQueryProducts::from_external_modules_with_allocation_budget(modules.clone(), &mut transaction)
                .map_err(|error| match error {
                    crate::snapshot::ModuleQueryProductsBuildError::Allocation(error) => CanonicalImageError::Allocation(error),
                    crate::snapshot::ModuleQueryProductsBuildError::DuplicateKey => {
                        CanonicalImageError::IncompletePublication("duplicate key in loaded module query products".into())
                    }
                })?;
            record_publication_census(2, module_products_start, &transaction);
            let module_products_publish_start = publication_census_checkpoint(&transaction);
            let module_products = transaction.arc(module_products)?;
            snapshot_obj = snapshot_obj.with_module_products(module_products);
            record_publication_census(3, module_products_publish_start, &transaction);
        }
        let status_start = publication_census_checkpoint(&transaction);
        if snapshot_obj.status.is_partial() && snapshot_obj.has_source_free_artifact_contents() {
            snapshot_obj = snapshot_obj.with_status(crate::snapshot::SnapshotStatus::Complete);
        }
        record_publication_census(1, status_start, &transaction);
        let validation_start = publication_census_checkpoint(&transaction);
        if !snapshot_obj.is_source_free_artifact_complete() {
            return Err(CanonicalImageError::IncompletePublication(
                "loaded artifact snapshot failed source-free completeness check".into(),
            ));
        }
        record_publication_census(4, validation_start, &transaction);
        let advisory_start = publication_census_checkpoint(&transaction);
        if !snapshot_obj.advisory.modules.is_empty() || !snapshot_obj.advisory.callables.is_empty() {
            return Err(CanonicalImageError::IncompletePublication(
                "suppressed advisory products present in loaded compiler image".into(),
            ));
        }
        record_publication_census(5, advisory_start, &transaction);
        let reader_start = publication_census_checkpoint(&transaction);
        queries.reader(&world, &snapshot_obj.store, &snapshot_obj.static_values)?;
        record_publication_census(6, reader_start, &transaction);
        let fingerprint_start = publication_census_checkpoint(&transaction);
        for (key, fp) in dependency_fingerprints.iter() {
            let kind = key.kind();
            let aggregate_before = phalcom_type_meta::allocation::current_observed_bytes();
            if let Some(expected) = aggregate_structural_fingerprint(&snapshot_obj, key) {
                if expected != *fp {
                    return Err(CanonicalImageError::IncompletePublication(format!(
                        "loaded dependency fingerprint disagrees with relocated aggregate: {key:?}"
                    )));
                }
            }
            if let (Some(before), Some(after)) = (aggregate_before, phalcom_type_meta::allocation::current_observed_bytes()) {
                LOADED_FINGERPRINT_AGGREGATE_OBSERVED.fetch_add(after - before, Ordering::Relaxed);
                if let Some(index) = QueryKind::ALL.iter().position(|candidate| *candidate == kind) {
                    LOADED_FINGERPRINT_AGGREGATE_BY_KIND[index].fetch_add(after - before, Ordering::Relaxed);
                }
            }
            let lookup_before = phalcom_type_meta::allocation::current_observed_bytes();
            let query_fingerprint_mismatch = queries.entry(key).is_some_and(|entry| entry.product_fingerprint != *fp);
            if let (Some(before), Some(after)) = (lookup_before, phalcom_type_meta::allocation::current_observed_bytes()) {
                LOADED_FINGERPRINT_QUERY_LOOKUP_OBSERVED.fetch_add(after - before, Ordering::Relaxed);
            }
            if query_fingerprint_mismatch {
                return Err(CanonicalImageError::IncompletePublication(format!(
                    "loaded query entry disagrees with dependency fingerprint: {key:?}"
                )));
            }
        }
        record_publication_census(7, fingerprint_start, &transaction);
        let type_prefix = CanonicalTypePrefixId {
            store: snapshot_obj.store.id(),
            types: snapshot_obj.store.len(),
        };
        let snapshot_arc_start = publication_census_checkpoint(&transaction);
        let snapshot = transaction.arc(snapshot_obj)?;
        record_publication_census(8, snapshot_arc_start, &transaction);
        let formal_image_start = publication_census_checkpoint(&transaction);
        #[cfg(test)]
        LOADED_FORMAL_IMAGE_FINAL_ARC_ATTEMPTED.with(|attempted| attempted.set(true));
        let image = transaction.arc(Self {
            world,
            modules,
            build_sources: BTreeMap::new(),
            presentation_provider: None,
            snapshot,
            queries,
            dependency_fingerprints,
            type_prefix,
        })?;
        record_publication_census(9, formal_image_start, &transaction);
        allocations.commit_child(transaction)?;
        Ok(image)
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
    if std::env::var_os("PHALCOM_DENY_SOURCE_BOOTSTRAP").is_some() {
        panic!("Canonical source bootstrap attempted while denied by PHALCOM_DENY_SOURCE_BOOTSTRAP");
    }
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
    if std::env::var_os("PHALCOM_DENY_SOURCE_BOOTSTRAP").is_some() {
        panic!("Canonical source bootstrap attempted while denied by PHALCOM_DENY_SOURCE_BOOTSTRAP");
    }
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

fn canonical_semantic_linked_program(
    provider: &UniverseSourceProvider,
    modules: &CanonicalModuleProducts,
) -> Result<Arc<phalcom_modules::linker::SemanticLinkedProgram>, CanonicalImageError> {
    let filesystem = phalcom_modules::FilesystemSourceProvider::new();
    let mut resolver = phalcom_modules::ModuleResolver::with_universe_source_provider(&modules.linked().universe, &filesystem, provider.clone());
    let interfaces = modules
        .interfaces()
        .iter()
        .map(|(module, interface)| (module.clone(), (**interface).clone()))
        .collect::<BTreeMap<_, _>>();
    let mut resolved = BTreeMap::new();
    for (module, interface) in &interfaces {
        for import in &interface.imports {
            let path = match import {
                phalcom_modules::ImportSurface::Module(decl) => &decl.path,
                phalcom_modules::ImportSurface::Selective(decl) => &decl.path,
                phalcom_modules::ImportSurface::ReExport(decl) => &decl.path,
            };
            let target = resolver
                .resolve_import(module, path)
                .map_err(|error| CanonicalImageError::ModuleProducts(format!("resolve canonical semantic import {module}::{path}: {error}")))?;
            resolved.insert((module.clone(), path.to_string()), target.id);
        }
    }
    let linked = phalcom_modules::ModuleLinker::new(modules.linked().universe.clone(), interfaces)
        .link_semantic(ModuleId::universe_root(), &resolved)
        .map_err(|error| CanonicalImageError::ModuleProducts(format!("link canonical semantic imports: {error}")))?;
    Ok(Arc::new(linked))
}

fn build(provider: &UniverseSourceProvider) -> Result<Arc<CanonicalUniverseFormalImage>, CanonicalImageError> {
    if std::env::var_os("PHALCOM_DENY_SOURCE_BOOTSTRAP").is_some() {
        panic!("Canonical source bootstrap attempted while denied by PHALCOM_DENY_SOURCE_BOOTSTRAP");
    }
    let private;
    let foundation = if provider.is_canonical() {
        canonical_universe_semantic_baseline()
    } else {
        private = CanonicalUniverseSemanticBaseline::build_with_provider(provider.clone());
        &private
    };
    let modules = Arc::new(CanonicalModuleProducts::capture(provider, foundation.linked.clone()).map_err(CanonicalImageError::ModuleProducts)?);
    let semantic_linked = canonical_semantic_linked_program(provider, &modules)?;
    let world = CanonicalSemanticWorldId::for_source_world(
        modules.source_fingerprint(),
        Arc::new(modules.sources().iter().map(|(module, unit)| (module.clone(), unit.text.clone())).collect()),
    );
    // Explicit foundation path: never ordinary image-backed session creation.
    let mut session = SemanticWorkspaceSession::from_baseline(foundation, WorkspaceId::from_raw(1))
        .with_product_request(crate::request::SemanticProductRequest::canonical_compiler_image());
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
        semantic_linked: Some(semantic_linked),
        sources: modules.sources().clone(),
        interfaces: Arc::new(modules.interfaces().iter().map(|(m, i)| (m.clone(), i.clone())).collect()),
        import_products: Arc::new(BTreeMap::new()),
        module_graph_changed: false,
        import_sites_by_module: Arc::new(BTreeMap::new()),
        require_canonical_import_products: false,
        attribute_execution_mode: crate::attribute_analysis::AttributeExecutionMode::Debug,
        diagnostics: BTreeMap::new(),
        blocked_modules: BTreeSet::new(),
        generation: 0,
        topology: Some(topology),
        reverse_imports: None,
    };
    let update = image_span(ImagePhase::Bootstrap, || session.update(input))?;
    if provider.is_canonical() {
        COMPLETE_UPDATES.fetch_add(1, Ordering::Relaxed);
    }
    image_counter!(CanonicalImageCompleteUpdates, 1);
    let snapshot = update.snapshot;
    image_span(ImagePhase::Validate, || {
        validate_publication(
            &snapshot,
            modules.external(),
            &crate::request::SemanticProductRequest::canonical_compiler_image(),
        )
    })?;
    let queries = Arc::new(image_span(ImagePhase::QueryFreeze, || {
        CanonicalSemanticQueryImage::freeze(session.db(), world.clone(), &snapshot, modules.external())
    })?);
    let dependency_fingerprints = aggregate_dependency_fingerprints(session.db(), &snapshot, &queries)?;
    let type_prefix = CanonicalTypePrefixId {
        store: snapshot.store.id(),
        types: snapshot.store.len(),
    };
    let image = Arc::new(CanonicalUniverseFormalImage {
        world,
        build_sources: modules.sources().clone(),
        presentation_provider: Some(provider.clone()),
        modules: modules.external().clone(),
        snapshot,
        queries,
        dependency_fingerprints,
        type_prefix,
    });
    if provider.is_canonical() {
        BUILDS.fetch_add(1, Ordering::Relaxed);
    }
    image_counter!(CanonicalImageBuilds, 1);
    crate::prelude::PreludeTypeMap::initialize_from_snapshot(&image.snapshot);
    Ok(image)
}

pub(crate) fn aggregate_structural_fingerprint(snapshot: &SemanticSnapshot, key: &QueryKey) -> Option<ProductFingerprint> {
    use crate::db::fingerprint;
    match key {
        QueryKey::CallableSignature(id) => snapshot.callable_signatures.get(id).map(fingerprint::callable_signature_product_fingerprint),
        QueryKey::FieldSignature(id) => snapshot.field_signatures.get(id).map(fingerprint::field_signature_product_fingerprint),
        QueryKey::DeclarationSurface(id) => snapshot.dispatch.get_surface(id).map(fingerprint::declaration_surface_product_fingerprint),
        QueryKey::TypeMemberDeclaration(id) => snapshot
            .type_member_declarations
            .get(id)
            .map(|product| fingerprint::type_member_declaration_product_fingerprint(product)),
        QueryKey::HierarchyEdge(id) => {
            if snapshot.declarations.contains(id) {
                Some(fingerprint::hierarchy_edge_product_fingerprint(id, snapshot.hierarchy.superclass(id), None))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Derives dependency evidence from committed aggregate authority without relaxing
/// query freeze closure or manufacturing mutable database rows.
fn aggregate_dependency_fingerprints(
    db: &SemanticDb,
    snapshot: &SemanticSnapshot,
    queries: &CanonicalSemanticQueryImage,
) -> Result<CanonicalDependencyFingerprints, CanonicalImageError> {
    use crate::db::fingerprint;
    let mut fingerprints = BTreeMap::new();
    for (id, signature) in snapshot.callable_signatures.iter() {
        fingerprints.insert(
            QueryKey::CallableSignature(id.clone()),
            fingerprint::callable_signature_product_fingerprint(signature),
        );
    }
    for (id, signature) in snapshot.field_signatures.iter() {
        fingerprints.insert(
            QueryKey::FieldSignature(id.clone()),
            fingerprint::field_signature_product_fingerprint(signature),
        );
    }
    for (id, surface) in snapshot.dispatch.surfaces().iter() {
        fingerprints.insert(
            QueryKey::DeclarationSurface(id.clone()),
            fingerprint::declaration_surface_product_fingerprint(surface),
        );
    }
    for (id, product) in snapshot.type_member_declarations.iter() {
        fingerprints.insert(
            QueryKey::TypeMemberDeclaration(id.clone()),
            fingerprint::type_member_declaration_product_fingerprint(product),
        );
    }
    // Hierarchy fingerprints include source syntax (including generic arguments).
    // Preserve the completed owner's fingerprint, and require agreement with the
    // final aggregate rather than inventing an edge from an absent parent entry.
    for key in db.query_keys() {
        let QueryKey::HierarchyEdge(id) = key else {
            continue;
        };
        let Some(QueryState::Ready {
            validated_revision,
            product_fingerprint,
            ..
        }) = db.query_state(key)
        else {
            continue;
        };
        if *validated_revision != db.revision() || !snapshot.declarations.contains(id) {
            continue;
        }
        let Some(product) = db.product(key).and_then(|product| product.as_hierarchy_edge()) else {
            continue;
        };
        if product.class_decl != *id || product.super_decl.as_ref() != snapshot.hierarchy.superclass(id) {
            return Err(CanonicalImageError::IncompletePublication(format!(
                "hierarchy dependency disagrees with final aggregate: {key:?}"
            )));
        }
        fingerprints.insert(key.clone(), *product_fingerprint);
    }
    for (key, fingerprint) in &fingerprints {
        if queries.entry(key).is_some_and(|entry| entry.product_fingerprint != *fingerprint) {
            return Err(CanonicalImageError::IncompletePublication(format!(
                "frozen query disagrees with final aggregate: {key:?}"
            )));
        }
    }
    Ok(CanonicalDependencyFingerprints::Source(fingerprints))
}

fn validate_publication(
    snapshot: &SemanticSnapshot,
    modules: &CanonicalExternalModuleWorld,
    request: &crate::request::SemanticProductRequest,
) -> Result<(), CanonicalImageError> {
    let missing = if !snapshot.status.is_complete() {
        Some("unblocked final snapshot")
    } else if snapshot.sources.len() != modules.linked().modules.len() || modules.linked().modules.keys().any(|module| !snapshot.sources.contains_key(module)) {
        Some("complete canonical source set")
    } else if snapshot.callable_analyses.is_empty() || snapshot.callable_signatures.is_empty() || snapshot.trait_surfaces.iter().next().is_none() {
        Some("final callable/signature/trait publication")
    } else if snapshot.declarations.iter().next().is_none() {
        Some("canonical declaration products")
    } else if snapshot.hierarchy.superclasses.is_empty() {
        Some("canonical hierarchy products")
    } else if snapshot.dispatch.surfaces().is_empty() {
        Some("canonical dispatch products")
    } else if snapshot.data_semantics.data_decls.is_empty() {
        Some("canonical data semantics")
    } else if snapshot.enum_semantics.enums.is_empty() {
        Some("canonical enum semantics")
    } else if snapshot.type_aliases.iter().next().is_none() {
        Some("canonical type aliases")
    } else if snapshot.conformance_index.iter().next().is_none() {
        Some("canonical conformance index")
    } else if request.presentation == crate::request::PresentationLevel::None && !snapshot.presentation_sources.is_empty() {
        Some("suppressed presentation sources in compiler-image mode")
    } else if request.advisory == crate::request::AdvisoryLevel::None && (!snapshot.advisory.modules.is_empty() || !snapshot.advisory.callables.is_empty()) {
        Some("suppressed advisory products in compiler-image mode")
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

    fn loaded_publication_inputs() -> (
        CanonicalSemanticWorldId,
        Arc<CanonicalExternalModuleWorld>,
        Arc<SemanticSnapshot>,
        Arc<CanonicalSemanticQueryImage>,
        CanonicalDependencyFingerprints,
    ) {
        let source_image = canonical_universe_formal_image().unwrap();
        let world = CanonicalSemanticWorldId::for_artifact_world(
            1,
            phalcom_compiler_artifact::ArtifactDigest::ZERO,
            phalcom_compiler_artifact::CanonicalCompatibilityIdentity {
                package: phalcom_compiler_artifact::TeloscPackageIdentity::universe(),
                source_digest: phalcom_compiler_artifact::ArtifactDigest::ZERO,
                interface_digest: phalcom_compiler_artifact::ArtifactDigest::ZERO,
                semantic_digest: phalcom_compiler_artifact::ArtifactDigest::ZERO,
                semantic_model_version: 1,
                stable_identity_schema: phalcom_compiler_artifact::header::STABLE_IDENTITY_SCHEMA_V1,
                module_semantic_abi: phalcom_compiler_artifact::header::MODULE_SEMANTIC_ABI_V2,
                semantic_artifact_abi: phalcom_compiler_artifact::header::SEMANTIC_ARTIFACT_ABI_V3,
                query_fingerprint_abi: phalcom_compiler_artifact::header::QUERY_FINGERPRINT_ABI_V3,
                native_surface_schema_version: 1,
                native_catalog_fingerprint: [0; 16],
                type_metadata_schema_version: 1,
            },
        );
        let mut snapshot = (**source_image.snapshot()).clone();
        snapshot.sources = Arc::new(BTreeMap::new());
        snapshot.presentation_sources = Arc::new(BTreeMap::new());
        Arc::make_mut(&mut snapshot.module_products).unlinked = Arc::new(BTreeMap::new());
        let snapshot = Arc::new(snapshot);

        let mut setup_allocations = AllocationBudget::new(usize::MAX);
        let mut retained = source_image
            .query_image()
            .entries()
            .filter(|(key, _)| {
                !matches!(key, QueryKey::ParsedModule(_)) && canonical_query_persistence(key.kind()) == CanonicalQueryPersistence::EligibleStructural
            })
            .map(|(key, entry)| {
                (
                    key.clone(),
                    CanonicalQueryEntry {
                        input_fingerprint: entry.input_fingerprint,
                        product_fingerprint: entry.product_fingerprint,
                        dependencies: entry.dependencies.clone(),
                        product: entry.product.clone(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        loop {
            let keep = retained
                .iter()
                .filter_map(|(key, entry)| {
                    entry
                        .dependencies
                        .iter()
                        .all(|edge| {
                            &edge.dependent == key
                                && retained
                                    .get(&edge.dependency)
                                    .is_some_and(|dependency| dependency.product_fingerprint == edge.observed_fingerprint)
                        })
                        .then(|| key.clone())
                })
                .collect::<BTreeSet<_>>();
            if keep.len() == retained.len() {
                break;
            }
            retained.retain(|key, _| keep.contains(key));
        }
        assert!(!retained.is_empty(), "fixture must retain eligible closed structural queries");
        let retained_keys = retained.keys().cloned().collect::<BTreeSet<_>>();
        let entries = retained.into_iter().collect();
        let entries = CanonicalQueryEntries::from_loaded_entries(entries, &mut setup_allocations).unwrap();
        let queries = CanonicalSemanticQueryImage::from_loaded_artifact_entries(
            world.clone(),
            entries,
            snapshot.store.clone(),
            snapshot.static_values.clone(),
            &mut setup_allocations,
        )
        .unwrap();
        let queries = Arc::new(queries);
        let fingerprints = source_image
            .dependency_fingerprints()
            .iter()
            .filter(|(key, _)| retained_keys.contains(*key))
            .map(|(key, fingerprint)| (key.clone(), *fingerprint))
            .collect();
        let fingerprints = CanonicalDependencyFingerprints::from_loaded_entries(fingerprints, &mut setup_allocations).unwrap();

        (world, source_image.modules.clone(), snapshot, queries, fingerprints)
    }

    fn clone_dependency_fingerprints(fingerprints: &CanonicalDependencyFingerprints) -> CanonicalDependencyFingerprints {
        CanonicalDependencyFingerprints::Source(fingerprints.iter().map(|(key, fingerprint)| (key.clone(), *fingerprint)).collect())
    }

    fn publish_loaded_parts(
        world: &CanonicalSemanticWorldId,
        modules: &Arc<CanonicalExternalModuleWorld>,
        snapshot: &Arc<SemanticSnapshot>,
        queries: &Arc<CanonicalSemanticQueryImage>,
        fingerprints: &CanonicalDependencyFingerprints,
        allocations: &mut AllocationBudget,
    ) -> Result<Arc<CanonicalUniverseFormalImage>, CanonicalImageError> {
        CanonicalUniverseFormalImage::from_loaded_artifact(
            world.clone(),
            modules.clone(),
            snapshot.clone(),
            queries.clone(),
            clone_dependency_fingerprints(fingerprints),
            allocations,
        )
    }

    #[test]
    fn loaded_formal_image_final_arc_failure_does_not_commit_and_retry_succeeds() {
        let (world, modules, snapshot, queries, fingerprints) = loaded_publication_inputs();
        assert!(
            snapshot.module_products.external_modules.is_some(),
            "fixture must exercise shared external module authority publication"
        );
        let mut discovery = AllocationBudget::new(usize::MAX);
        let image =
            publish_loaded_parts(&world, &modules, &snapshot, &queries, &fingerprints, &mut discovery).expect("valid source-free loaded parts must publish");
        assert!(image.sources().is_empty());
        assert!(image.snapshot().is_source_free_artifact_complete());
        assert!(image.snapshot().advisory.modules.is_empty());
        assert!(image.snapshot().advisory.callables.is_empty());
        assert!(image.reader().is_ok());
        assert!(image.snapshot().module_products.external_modules.is_some());
        assert!(image.snapshot().module_queries().is_fully_indexed());
        let required = discovery.requested();
        assert!(required > 0);
        let control_query_identity = image
            .query_image()
            .entries()
            .map(|(key, entry)| (key.clone(), entry.product_fingerprint))
            .collect::<Vec<_>>();
        let control_dependency_identity = image
            .dependency_fingerprints()
            .iter()
            .map(|(key, fingerprint)| (key.clone(), *fingerprint))
            .collect::<Vec<_>>();

        let mut p3_failure_budget = AllocationBudget::new(0);
        let p3_failure = publish_loaded_parts(&world, &modules, &snapshot, &queries, &fingerprints, &mut p3_failure_budget);
        assert!(matches!(
            p3_failure,
            Err(CanonicalImageError::Allocation(phalcom_type_meta::allocation::AllocationError::Exhausted))
        ));
        assert_eq!(
            p3_failure_budget.requested(),
            0,
            "failed ModuleQueryProducts child must not commit to its parent"
        );

        let mut p3_retry_budget = AllocationBudget::new(required);
        let p3_retry = publish_loaded_parts(&world, &modules, &snapshot, &queries, &fingerprints, &mut p3_retry_budget)
            .expect("fresh exact-budget retry after P3 allocation failure must publish");
        assert_eq!(p3_retry_budget.requested(), required);
        assert_eq!(p3_retry.world(), image.world());
        assert_eq!(p3_retry.type_prefix(), image.type_prefix());
        assert_eq!(p3_retry.static_lineage(), image.static_lineage());
        assert_eq!(p3_retry.query_image().len(), image.query_image().len());
        assert_eq!(
            p3_retry
                .query_image()
                .entries()
                .map(|(key, entry)| (key.clone(), entry.product_fingerprint))
                .collect::<Vec<_>>(),
            control_query_identity
        );
        assert_eq!(
            p3_retry
                .dependency_fingerprints()
                .iter()
                .map(|(key, fingerprint)| (key.clone(), *fingerprint))
                .collect::<Vec<_>>(),
            control_dependency_identity
        );
        assert!(p3_retry.reader().is_ok());

        let mut mismatched = fingerprints
            .iter()
            .map(|(key, fingerprint)| (key.clone(), *fingerprint))
            .collect::<BTreeMap<_, _>>();
        let (_, fingerprint) = mismatched.iter_mut().next().expect("fixture has dependency fingerprints");
        *fingerprint = ProductFingerprint(fingerprint.0.wrapping_add(1));
        let mut validation_failure_budget = AllocationBudget::new(usize::MAX);
        let failed = CanonicalUniverseFormalImage::from_loaded_artifact(
            world.clone(),
            modules.clone(),
            snapshot.clone(),
            queries.clone(),
            CanonicalDependencyFingerprints::Source(mismatched),
            &mut validation_failure_budget,
        );
        assert!(matches!(failed, Err(CanonicalImageError::IncompletePublication(_))));
        assert_eq!(
            validation_failure_budget.requested(),
            0,
            "failed validation child must not commit to its parent"
        );

        let mut one_byte_short = AllocationBudget::new(required - 1);
        LOADED_FORMAL_IMAGE_FINAL_ARC_ATTEMPTED.with(|attempted| attempted.set(false));
        let failed = publish_loaded_parts(&world, &modules, &snapshot, &queries, &fingerprints, &mut one_byte_short);
        assert!(matches!(
            failed,
            Err(CanonicalImageError::Allocation(phalcom_type_meta::allocation::AllocationError::Exhausted))
        ));
        assert!(
            LOADED_FORMAL_IMAGE_FINAL_ARC_ATTEMPTED.with(Cell::get),
            "N-1 must reach the final formal-image Arc allocation before exhaustion"
        );
        assert_eq!(one_byte_short.requested(), 0, "failed final image child must not commit to its parent");

        let mut retry = AllocationBudget::new(required);
        let image = publish_loaded_parts(&world, &modules, &snapshot, &queries, &fingerprints, &mut retry).expect("a fresh exact-budget retry must publish");
        assert_eq!(retry.requested(), required);
        assert_eq!(image.world(), p3_retry.world());
        assert_eq!(image.type_prefix(), p3_retry.type_prefix());
        assert_eq!(image.static_lineage(), p3_retry.static_lineage());
        assert_eq!(image.query_image().len(), p3_retry.query_image().len());
        assert_eq!(
            image
                .query_image()
                .entries()
                .map(|(key, entry)| (key.clone(), entry.product_fingerprint))
                .collect::<Vec<_>>(),
            control_query_identity
        );
        assert_eq!(
            image
                .dependency_fingerprints()
                .iter()
                .map(|(key, fingerprint)| (key.clone(), *fingerprint))
                .collect::<Vec<_>>(),
            control_dependency_identity
        );
        assert!(image.sources().is_empty());
        assert!(image.snapshot().is_source_free_artifact_complete());
        assert!(image.reader().is_ok());
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
    fn required_query_profile_roots_are_canonical_immutable_leaves() {
        use crate::artifact::{CanonicalQueryRootRequirement, canonical_query_root_requirement};

        let image = canonical_universe_formal_image().unwrap().clone();
        let mut db = SemanticDb::with_canonical_image(WorkspaceId::from_raw(93), image.clone());
        let mut observed = std::collections::BTreeMap::<QueryKind, usize>::new();

        for (key, entry) in image.query_image().entries() {
            if canonical_query_root_requirement(key.kind()) != CanonicalQueryRootRequirement::Required {
                continue;
            }
            *observed.entry(key.kind()).or_default() += 1;
            assert!(db.is_current(key), "required canonical root must be current: {key:?}");
            assert!(db.validate_ready(key), "required canonical root must validate ready: {key:?}");
            assert_eq!(
                db.product_origin(key),
                Some(crate::db::SemanticProductOrigin::Canonical(image.world().clone())),
                "required canonical root must retain canonical origin: {key:?}"
            );
            assert!(db.local_query_state(key).is_none(), "required canonical root must not synthesize local state: {key:?}");
            assert_eq!(db.ready_product_fingerprint(key), Some(entry.product_fingerprint));
        }

        for kind in QueryKind::ALL {
            if canonical_query_root_requirement(kind) == CanonicalQueryRootRequirement::Required {
                assert!(observed.get(&kind).is_some_and(|count| *count > 0), "required canonical root family has no source root: {kind:?}");
            }
        }
    }

    #[test]
    fn freeze_rejects_structural_rows_changed_by_final_settlement_and_their_dependents() {
        let image = canonical_universe_formal_image().unwrap();
        let (id, surface) = image.snapshot().dispatch.surfaces().iter().next().unwrap();
        let key = QueryKey::DeclarationSurface(id.clone());
        let fingerprint = crate::db::fingerprint::declaration_surface_product_fingerprint(surface);
        let mut db = SemanticDb::with_workspace(WorkspaceId::from_raw(92));
        let publish_surface = |db: &mut SemanticDb, fingerprint| {
            db.publish_product_ready(
                key.clone(),
                db.revision(),
                InputFingerprint(1),
                fingerprint,
                SemanticProduct::DeclarationSurface(Arc::new(crate::db::product::DeclarationSurfaceProduct {
                    surface: Arc::new(surface.clone()),
                    diagnostics: Arc::from([]),
                })),
                [],
            )
            .unwrap();
        };
        publish_surface(&mut db, fingerprint);
        assert!(freeze(&db, image).entry(&key).is_some());
        let stale = ProductFingerprint(fingerprint.raw().wrapping_add(1));
        publish_surface(&mut db, stale);
        let module = image.sources().keys().next().unwrap().clone();
        let dependent = QueryKey::ParsedModule(module.clone());
        publish(
            &mut db,
            image,
            module,
            vec![DependencyEdge {
                dependent: dependent.clone(),
                dependency: key.clone(),
                observed_fingerprint: stale,
            }],
        );
        let queries = freeze(&db, image);
        assert_eq!(queries.census.rejections[&key], QueryFreezeRejection::FinalAggregateMismatch);
        assert_eq!(queries.census.rejections[&dependent], QueryFreezeRejection::MutableDependency);
        assert_eq!(image.dependency_fingerprint(&key), Some(fingerprint));
    }

    #[test]
    fn declaration_shell_freeze_requires_exact_final_nominal_or_alias_authority() {
        use crate::declarations::DeclarationTypeInfo;
        use crate::diagnostic::SemanticSourceSpan;
        use crate::identity::DeclarationId;
        use crate::type_alias::TypeAliasInfo;
        use crate::types::id::{KindId, TypeId};
        use phalcom_common::range::SourceRange;

        let module = ModuleId::universe_root();
        let nominal_owner = DeclarationId::new(module.clone(), "Nominal".into());
        let alias_owner = DeclarationId::new(module.clone(), "Alias".into());
        let nominal = TypeDeclarationShell::Nominal(DeclarationTypeInfo {
            declaration: nominal_owner.clone(),
            form: TypeId::DUMMY,
            class_object_type: TypeId::DUMMY,
            kind: KindId::TYPE,
            binding: Default::default(),
            generic_binder_header: None,
            generic_signature: None,
            supertype_template: None,
        });
        let alias = TypeDeclarationShell::Alias(TypeAliasInfo {
            declaration: alias_owner.clone(),
            kind: KindId::TYPE,
            kind_shape: "Type".into(),
            generic_signature: None,
            form: TypeId::DUMMY,
            structural_form: "Type".into(),
            dependencies: Box::new([]),
            source: SemanticSourceSpan::new(module, SourceRange::default()),
        });

        validate_final_declaration_shell_owner(&nominal_owner, true, false, Some(&nominal)).expect("nominal query matches final authority");
        validate_final_declaration_shell_owner(&alias_owner, false, true, Some(&alias)).expect("alias query matches final authority");

        let missing = validate_final_declaration_shell_owner(&nominal_owner, false, false, Some(&nominal)).unwrap_err();
        assert!(missing.to_string().contains("has 0 final authorities"));
        let ambiguous = validate_final_declaration_shell_owner(&nominal_owner, true, true, Some(&nominal)).unwrap_err();
        assert!(ambiguous.to_string().contains("has 2 final authorities"));
        let missing_product = validate_final_declaration_shell_owner(&nominal_owner, true, false, None).unwrap_err();
        assert!(missing_product.to_string().contains("has no typed shell product"));
        let wrong_owner = validate_final_declaration_shell_owner(&nominal_owner, true, false, Some(&alias)).unwrap_err();
        assert!(wrong_owner.to_string().contains("disagrees with its native product"));
        let wrong_variant = validate_final_declaration_shell_owner(&alias_owner, true, false, Some(&alias)).unwrap_err();
        assert!(wrong_variant.to_string().contains("variant disagrees with final semantic authority"));
    }

    #[test]
    fn canonical_aggregate_dependencies_are_exact_and_do_not_synthesize_queries() {
        let image = canonical_universe_formal_image().unwrap().clone();
        let key = image
            .dependency_fingerprints
            .keys()
            .find(|key| image.query_image().entry(key).is_none())
            .expect("aggregate-only canonical dependency")
            .clone();
        let fingerprint = image.dependency_fingerprint(&key).unwrap();
        let mut db = SemanticDb::with_canonical_image(WorkspaceId::from_raw(91), image.clone());
        let module = ModuleId::resolved(
            phalcom_modules::identity::ResolvedProjectId::from_raw(99),
            phalcom_modules::identity::ModulePath::root(),
        );
        let local = QueryKey::ParsedModule(module);
        let mut recorder = crate::db::DependencyRecorder::new(local.clone());
        db.record_dependency(&mut recorder, key.clone()).unwrap();
        let edges = recorder.finish();
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].observed_fingerprint, fingerprint);
        assert!(!db.is_current(&key));
        assert!(!db.is_reusable(&key, InputFingerprint(0)));
        assert!(db.product(&key).is_none());
        db.publish_product_ready(
            local.clone(),
            db.revision(),
            InputFingerprint(1),
            ProductFingerprint(2),
            SemanticProduct::ParsedModule(image.sources().values().next().unwrap().clone()),
            edges.clone(),
        )
        .unwrap();
        assert!(db.is_reusable(&local, InputFingerprint(1)));
        assert!(db.index().dependents_of(&key).is_none_or(|dependents| dependents.is_empty()));
        let mut mismatched = edges;
        mismatched[0].observed_fingerprint = ProductFingerprint(fingerprint.raw().wrapping_add(1));
        db.publish_product_ready(
            local.clone(),
            db.revision(),
            InputFingerprint(1),
            ProductFingerprint(2),
            SemanticProduct::ParsedModule(image.sources().values().next().unwrap().clone()),
            mismatched,
        )
        .unwrap();
        assert!(!db.is_reusable(&local, InputFingerprint(1)));
        let absent = crate::identity::DeclarationId::new(ModuleId::universe_root(), "MissingCanonicalDeclaration".into());
        let absent_callable = crate::identity::CallableId::new(
            absent.clone(),
            phalcom_common::selector::Selector::method("missing", []).unwrap(),
            crate::identity::DispatchSide::Instance,
        );
        for absent_key in [
            QueryKey::HierarchyEdge(absent.clone()),
            QueryKey::DeclarationSurface(absent),
            QueryKey::CallableSignature(absent_callable),
        ] {
            let mut recorder = crate::db::DependencyRecorder::new(local.clone());
            assert!(db.record_dependency(&mut recorder, absent_key).is_err());
            assert!(recorder.finish().is_empty());
        }
        db.set_state(key.clone(), QueryState::Vacant);
        assert!(db.dependency_fingerprint(&key).is_none());
        assert_eq!(db.query_keys().count(), 2);
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

    #[test]
    fn canonical_compiler_image_validate_publication_enforces_compiler_completeness() {
        let image = canonical_universe_formal_image().unwrap();
        let canonical_req = crate::request::SemanticProductRequest::canonical_compiler_image();
        assert!(validate_publication(image.snapshot(), image.modules(), &canonical_req).is_ok());

        // Presentation leakage in compiler-image mode fails validation
        let mut leaking_snapshot = (**image.snapshot()).clone();
        leaking_snapshot.presentation_sources = Arc::new(BTreeMap::from([(
            image.sources().keys().next().unwrap().clone(),
            Arc::from("leaked presentation"),
        )]));
        assert!(matches!(
            validate_publication(&leaking_snapshot, image.modules(), &canonical_req),
            Err(CanonicalImageError::IncompletePublication(msg)) if msg.contains("suppressed presentation sources")
        ));

        // Advisory leakage in compiler-image mode fails validation
        let mut leaking_advisory_snapshot = (**image.snapshot()).clone();
        let mut adv = (*leaking_advisory_snapshot.advisory).clone();
        Arc::make_mut(&mut adv.modules).insert(
            image.sources().keys().next().unwrap().clone(),
            Arc::new(crate::advisory::AdvisoryModuleProduct::new(
                image.sources().keys().next().unwrap().clone(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                crate::advisory::AdvisoryProductStatus::Complete,
            )),
        );
        leaking_advisory_snapshot.advisory = Arc::new(adv);
        assert!(matches!(
            validate_publication(&leaking_advisory_snapshot, image.modules(), &canonical_req),
            Err(CanonicalImageError::IncompletePublication(msg)) if msg.contains("suppressed advisory products")
        ));

        // Missing compiler-required products fail validation
        let mut missing_decl_snapshot = (**image.snapshot()).clone();
        missing_decl_snapshot.declarations = Arc::new(crate::declarations::DeclarationTypeTable::default());
        assert!(matches!(
            validate_publication(&missing_decl_snapshot, image.modules(), &canonical_req),
            Err(CanonicalImageError::IncompletePublication(msg)) if msg.contains("canonical declaration products")
        ));
    }

    #[test]
    fn canonical_compiler_image_retains_executable_numeric_trait_requirements() {
        let image = canonical_universe_formal_image().unwrap();
        let module = phalcom_modules::ModuleId::universe(phalcom_modules::ModulePath::from_components(vec![
            phalcom_modules::ModuleComponent::from_identifier("numbers").unwrap(),
            phalcom_modules::ModuleComponent::from_identifier("capabilities").unwrap(),
        ]));
        for trait_name in ["Zero", "One"] {
            let declaration = crate::identity::DeclarationId::new(module.clone(), trait_name.into());
            let surface = image.snapshot().trait_surfaces.get(&declaration).expect("canonical numeric trait surface");
            let selector = phalcom_common::selector::Selector::getter(trait_name.to_ascii_lowercase()).unwrap();
            let requirement = crate::traits::TraitRequirementId::new(declaration, selector, crate::identity::DispatchSide::Class);
            assert!(surface.get(&requirement).is_some(), "missing executable class requirement for {trait_name}");
        }
    }
}

#[cfg(test)]
mod negative_fact_tests {
    use super::*;

    #[test]
    fn query_persistence_classification_is_fail_closed_for_nonstructural_products() {
        assert_eq!(
            canonical_query_persistence(QueryKind::ParsedModule),
            CanonicalQueryPersistence::EligibleStructural
        );
        assert_eq!(
            canonical_query_persistence(QueryKind::ModuleStaticBindings),
            CanonicalQueryPersistence::EligibleStructural
        );
        assert_eq!(canonical_query_persistence(QueryKind::GenericConformance), CanonicalQueryPersistence::OpenWorld);
        assert_eq!(canonical_query_persistence(QueryKind::CallableBody), CanonicalQueryPersistence::OpenWorld);
        assert_eq!(canonical_query_persistence(QueryKind::StaticConstant), CanonicalQueryPersistence::Unsupported);
        assert_eq!(canonical_query_persistence(QueryKind::AdvisoryModule), CanonicalQueryPersistence::Unsupported);
    }

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
    fn persistence_inventory_is_exhaustive_unique_and_does_not_claim_portability() {
        use std::collections::BTreeSet;
        let all = [
            CanonicalProductFamily::ModulesAndInterfaces,
            CanonicalProductFamily::ImportsAndTopology,
            CanonicalProductFamily::TypeArena,
            CanonicalProductFamily::StaticValueArena,
            CanonicalProductFamily::DeclarationsAndAliases,
            CanonicalProductFamily::HierarchyAndDispatch,
            CanonicalProductFamily::CallableSignaturesAndDefinitions,
            CanonicalProductFamily::CallableAnalyses,
            CanonicalProductFamily::TraitsAndConformance,
            CanonicalProductFamily::ConformanceWitnessesAndDispatch,
            CanonicalProductFamily::DataAndEnums,
            CanonicalProductFamily::AssociatedAndStaticSemantics,
            CanonicalProductFamily::FrozenStructuralQueries,
            CanonicalProductFamily::RuntimePublicMetadata,
            CanonicalProductFamily::CompilerSourceAttachments,
            CanonicalProductFamily::EditorPresentation,
            CanonicalProductFamily::AdvisoryProducts,
            CanonicalProductFamily::SourceMaps,
            CanonicalProductFamily::Diagnostics,
            CanonicalProductFamily::WorkspaceRevisionHistory,
            CanonicalProductFamily::MutableSemanticDatabase,
            CanonicalProductFamily::RuntimeHandles,
        ];
        let listed: BTreeSet<_> = CANONICAL_PERSISTENCE_INVENTORY.iter().map(|(family, _)| *family).collect();
        assert_eq!(listed.len(), CANONICAL_PERSISTENCE_INVENTORY.len());
        assert_eq!(listed, BTreeSet::from(all));
        assert!(CANONICAL_PERSISTENCE_INVENTORY.contains(&(CanonicalProductFamily::RuntimePublicMetadata, CanonicalPersistenceClass::RequiredMetadata,)));
        assert!(CANONICAL_PERSISTENCE_INVENTORY.contains(&(CanonicalProductFamily::CompilerSourceAttachments, CanonicalPersistenceClass::RequiredSemantic,)));
        let image = canonical_universe_formal_image().unwrap();
        let capabilities = image.capabilities();
        assert_eq!(capabilities.module_count, image.sources().len());
        assert_eq!(capabilities.frozen_query_count, image.query_image().len());
        assert!(!capabilities.runtime_metadata_composed);
        assert!(!capabilities.portable_artifact_available);
    }

    #[test]
    fn canonical_image_exact_world_provenance_rejects_digest_collision() {
        let image = canonical_universe_formal_image().unwrap();
        let mut foreign = image.world.clone();
        let module = image.sources().keys().next().unwrap().clone();
        if let CanonicalWorldProvenance::Source(ref mut arc) = foreign.provenance {
            Arc::make_mut(arc).insert(module, Arc::from("different provider source"));
        }
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

    #[test]
    fn list_get_returns_inferred_anonymous_variant_union_contract() {
        let image = canonical_universe_formal_image().unwrap();
        let snapshot = image.snapshot();
        let list_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::List);
        let surface = snapshot.surfaces.get(&list_decl).expect("List surface");
        let get_sig = surface
            .instance
            .callable_signatures
            .iter()
            .find(|(sel, sig)| format!("{:?}", sel).contains("get") && sig.parameters.len() == 1)
            .map(|(_, sig)| sig)
            .expect("List.get(_) signature");
        assert_eq!(get_sig.parameters.len(), 1);
        assert_eq!(get_sig.parameters[0].local_name.as_str(), "index");
        let crate::types::evidence::TypeKnowledge::Known(ref ev) = get_sig.return_type else {
            panic!("expected known return type for List.get");
        };
        // Case B proof: return type is inferred flow-type, not declared annotation.
        assert_eq!(ev.origin(), crate::types::evidence::EvidenceOrigin::Flow);
        let crate::types::store::TypeData::Union(members) = snapshot.store.get(ev.ty()) else {
            panic!("expected Union type data for List.get return");
        };
        assert_eq!(members.len(), 2);
        let member_types: Vec<_> = members.iter().map(|m| snapshot.store.get(*m)).collect();

        let option_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Option);
        let expected_some_decl = phalcom_modules::DeclarationId::new(option_decl.module.clone(), "Some".into());
        let expected_none_decl = phalcom_modules::DeclarationId::new(option_decl.module.clone(), "None".into());

        let some_decl = member_types
            .iter()
            .find_map(|td| match td {
                crate::types::store::TypeData::Nominal { declaration } if declaration.name.as_ref() == "Some" => Some(declaration),
                _ => None,
            })
            .expect("Union must contain Some variant");
        assert_eq!(
            some_decl, &expected_some_decl,
            "Some variant identity must match canonical Option.Some declaration"
        );

        let none_decl = member_types
            .iter()
            .find_map(|td| match td {
                crate::types::store::TypeData::ClassObject { declaration } if declaration.name.as_ref() == "None" => Some(declaration),
                _ => None,
            })
            .expect("Union must contain None variant");
        assert_eq!(
            none_decl, &expected_none_decl,
            "None variant identity must match canonical Option.None declaration"
        );

        // Verify generic T contract on List and Option
        let list_info = snapshot.declarations.get(&list_decl).expect("List declaration");
        assert!(list_info.generic_signature.is_some(), "List must declare generic parameter T");
        assert_eq!(
            list_info.generic_signature.as_ref().unwrap().parameters.len(),
            1,
            "List has single type parameter T"
        );
        let option_info = snapshot.declarations.get(&option_decl).expect("Option declaration");
        assert!(option_info.generic_signature.is_some(), "Option must declare generic parameter T");
        assert_eq!(
            option_info.generic_signature.as_ref().unwrap().parameters.len(),
            1,
            "Option has single type parameter T"
        );
    }

    #[test]
    fn hierarchy_edge_fingerprint_fallback_and_query_authority_consumption() {
        let image = canonical_universe_formal_image().unwrap().clone();
        let list_decl = crate::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::List);
        let key = QueryKey::HierarchyEdge(list_decl);

        // 1. Existing declaration resolves to hierarchy-edge fingerprint
        let fp = image.dependency_fingerprint(&key);
        assert!(fp.is_some(), "existing declaration must yield hierarchy-edge fingerprint");

        // 2. Missing declaration resolves to None
        let missing_decl = phalcom_modules::DeclarationId::new(ModuleId::universe_root(), Box::from("MissingDeclNonexistent"));
        let missing_key = QueryKey::HierarchyEdge(missing_decl);
        assert_eq!(image.dependency_fingerprint(&missing_key), None);

        // 3. Structural fallback returns valid fingerprint for existing declaration and None for missing
        let structural_fp = aggregate_structural_fingerprint(image.snapshot(), &key);
        assert!(structural_fp.is_some());
        assert_eq!(aggregate_structural_fingerprint(image.snapshot(), &missing_key), None);
        // Authority is not mutated across repeated queries
        assert_eq!(image.dependency_fingerprint(&key), fp);

        // 4. Query reader consumption and database reuse
        let reader = image.reader().expect("valid reader");
        let (first_key, first_entry) = image.query_image().entries().next().expect("entry");
        let product = reader.product(first_key).expect("product");
        assert_eq!(reader.product_fingerprint(first_key), Some(first_entry.product_fingerprint));

        let mut db = SemanticDb::with_canonical_image(crate::identity::WorkspaceId::from_raw(99), image.clone());
        assert_eq!(db.dependency_fingerprint(&key), fp);
        assert!(db.validate_ready(first_key));
        assert_eq!(
            db.product_origin(first_key),
            Some(crate::db::SemanticProductOrigin::Canonical(image.world().clone()))
        );
        assert!(Arc::ptr_eq(db.product(first_key).unwrap(), product));
    }

    #[test]
    fn type_store_and_static_value_store_prefix_and_fork_invariants() {
        let image = canonical_universe_formal_image().unwrap();
        let p_types = image.snapshot().store.clone();
        let p_vals = image.snapshot().static_values.clone();

        // 1. TypeStore fork and isolation
        let p_type_count = p_types.type_count();
        let mut fork_a = p_types.fork_from_baseline();
        let fork_b = p_types.fork_from_baseline();

        // P and B unchanged before mutation
        assert_eq!(p_types.type_count(), p_type_count);
        assert_eq!(fork_b.type_count(), p_type_count);

        // Insert user-local material into A
        let local_decl = phalcom_modules::DeclarationId::new(ModuleId::universe_root(), Box::from("LocalTypePrefixTest"));
        let user_type = fork_a.intern_with_kind(crate::types::store::TypeData::Nominal { declaration: local_decl }, crate::KindId(0));
        assert!(user_type.0 as usize >= p_type_count);

        // Assert: P unchanged, B unchanged, A sees local suffix
        assert_eq!(p_types.type_count(), p_type_count);
        assert_eq!(fork_b.type_count(), p_type_count);
        assert_eq!(fork_a.type_count(), p_type_count + 1);

        // A and B still see canonical P
        for i in 0..p_type_count {
            let tid = crate::TypeId(i as u32);
            assert_eq!(fork_a.get(tid), p_types.get(tid));
            assert_eq!(fork_b.get(tid), p_types.get(tid));
        }

        // Reader-binding prefix extensions
        assert!(fork_a.extends_canonical_prefix(&p_types));
        assert!(fork_b.extends_canonical_prefix(&p_types));
        assert!(!p_types.extends_canonical_prefix(&fork_a));
        assert!(!fork_b.extends_canonical_prefix(&fork_a));

        // 2. StaticValueStore fork and isolation
        let p_val_count = p_vals.value_count();
        let mut val_fork_a = p_vals.as_ref().clone();
        let val_fork_b = p_vals.as_ref().clone();

        assert_eq!(p_vals.value_count(), p_val_count);
        assert_eq!(val_fork_b.value_count(), p_val_count);

        // Insert user-local material into A
        let _user_val = val_fork_a.intern_string("user_local_material_for_prefix_test");
        assert!(val_fork_a.value_count() > p_val_count);

        // Assert: P unchanged, B unchanged
        assert_eq!(p_vals.value_count(), p_val_count);
        assert_eq!(val_fork_b.value_count(), p_val_count);

        // A and B still see canonical P
        for i in 0..p_val_count {
            let vid = crate::static_value::StaticValueId(i as u32);
            assert_eq!(val_fork_a.get_value(vid), p_vals.get_value(vid));
            assert_eq!(val_fork_b.get_value(vid), p_vals.get_value(vid));
        }

        assert!(val_fork_a.extends_canonical_prefix(&p_vals));
        assert!(val_fork_b.extends_canonical_prefix(&p_vals));
        assert!(!p_vals.extends_canonical_prefix(&val_fork_a));
        assert!(!val_fork_b.extends_canonical_prefix(&val_fork_a));
    }

    #[test]
    fn prelude_source_artifact_equivalence() {
        let image = canonical_universe_formal_image().unwrap();
        let snapshot = image.snapshot();
        let source_prelude = crate::prelude::PreludeTypeMap::build_source_canonical_universe();
        let snapshot_prelude = crate::prelude::PreludeTypeMap::from_declarations(|decl| {
            snapshot.declarations.contains(decl)
                || snapshot.type_aliases.contains_key(decl)
                || snapshot.trait_headers.contains(decl)
                || crate::checker::proof_predicate::is_proof_predicate_declaration(decl).is_some()
        });
        assert_eq!(
            source_prelude.entries(),
            snapshot_prelude.entries(),
            "Source-derived and snapshot-derived preludes must have identical entries"
        );
    }

    #[test]
    fn prelude_repeated_compatible_artifact_loads_do_not_panic() {
        let image = canonical_universe_formal_image().unwrap();
        crate::prelude::PreludeTypeMap::initialize_from_snapshot(image.snapshot());
        crate::prelude::PreludeTypeMap::initialize_from_snapshot(image.snapshot());
    }

    #[test]
    #[should_panic(expected = "Mandatory prelude declaration \"Unit\"")]
    fn prelude_missing_mandatory_declaration_panics() {
        let image = canonical_universe_formal_image().unwrap();
        let snapshot = image.snapshot();
        let _ = crate::prelude::PreludeTypeMap::from_declarations(|decl| {
            if decl.name.as_ref() == "Unit" {
                false
            } else {
                snapshot.declarations.contains(decl)
                    || snapshot.type_aliases.contains_key(decl)
                    || snapshot.trait_headers.contains(decl)
                    || crate::checker::proof_predicate::is_proof_predicate_declaration(decl).is_some()
            }
        });
    }
}
