use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use phalcom_compiler_artifact::*;
use phalcom_type_meta::allocation::AllocationBudget;

struct CountingSystem;

static TRACK_REQUESTS: AtomicBool = AtomicBool::new(false);
static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);

// SAFETY: This allocator forwards every operation unchanged to `System` and
// only records requested layout sizes using atomics while the test enables it.
unsafe impl GlobalAlloc for CountingSystem {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if TRACK_REQUESTS.load(Ordering::Relaxed) {
            REQUESTED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        // SAFETY: `layout` is forwarded unchanged to the system allocator.
        unsafe { System.alloc(layout) }
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        if TRACK_REQUESTS.load(Ordering::Relaxed) {
            REQUESTED_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        // SAFETY: `layout` is forwarded unchanged to the system allocator.
        unsafe { System.alloc_zeroed(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: `ptr` and `layout` came from the forwarded system allocation.
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        if TRACK_REQUESTS.load(Ordering::Relaxed) {
            REQUESTED_BYTES.fetch_add(new_size, Ordering::Relaxed);
        }
        // SAFETY: `ptr`, `layout`, and `new_size` are forwarded unchanged.
        unsafe { System.realloc(ptr, layout, new_size) }
    }
}

#[global_allocator]
static ALLOCATOR: CountingSystem = CountingSystem;

fn header() -> TeloscHeaderV2 {
    TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity: TeloscPackageIdentity::universe(),
        semantic_model_version: 1,
        stable_identity_schema: 1,
        module_semantic_abi: 1,
        semantic_artifact_abi: 1,
        query_fingerprint_abi: 1,
        native_surface_schema_version: 1,
        native_catalog_fingerprint: [6; 16],
        type_metadata_schema_version: 1,
        capabilities: TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1 | TeloscCapabilities::RUNTIME_PUBLIC_METADATA,
        source_digest: ArtifactDigest([1; 32]),
        interface_digest: ArtifactDigest([2; 32]),
        semantic_digest: ArtifactDigest([3; 32]),
        query_digest: ArtifactDigest([4; 32]),
        metadata_digest: ArtifactDigest([5; 32]),
        artifact_digest: ArtifactDigest::ZERO,
        section_count: 0,
    }
}

#[test]
fn container_parse_ledger_matches_allocator_requests() {
    let mut builder = TeloscContainerBuilder::new(header());
    for (kind, data) in [
        (SectionKind::ModuleWorld, b"module".as_slice()),
        (SectionKind::SemanticWorld, b"semantic".as_slice()),
        (SectionKind::QueryImage, b"query".as_slice()),
        (SectionKind::Metadata, b"metadata".as_slice()),
    ] {
        let schema = match kind {
            SectionKind::SemanticWorld => phalcom_compiler_artifact::semantic::world::SEMANTIC_WORLD_SCHEMA_V3,
            SectionKind::QueryImage => phalcom_compiler_artifact::query::QUERY_IMAGE_SCHEMA_V2,
            _ => 1,
        };
        builder.add_section(kind, schema, SectionFlags::REQUIRED, data.to_vec());
    }
    let bytes = builder.build().unwrap();
    let limits = ValidationLimits::default();
    let mut budget = AllocationBudget::new(1024 * 1024);

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    TRACK_REQUESTS.store(true, Ordering::Relaxed);
    let view = TeloscContainerView::parse_with_allocation_budget(&bytes, &limits, &mut budget).unwrap();
    TRACK_REQUESTS.store(false, Ordering::Relaxed);

    assert_eq!(view.directory.len(), 4);
    assert_eq!(REQUESTED_BYTES.load(Ordering::Relaxed), budget.requested());
    assert_eq!(budget.requested(), 8 + 5 + 4 * std::mem::size_of::<SectionDirectoryEntryV1>());
}
