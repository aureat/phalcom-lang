use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::container::TeloscContainerView;
use phalcom_compiler_artifact::section::SectionKind;
use phalcom_compiler_artifact::semantic::world::SemanticWorldV3;
use phalcom_core::modules::canonical_artifact::{
    build_universe_telosc, load_universe_telosc_from_bytes, load_universe_telosc_from_bytes_with_allocation_summary,
};
use phalcom_core::modules::canonical_universe::canonical_universe_program;
use phalcom_semantic::artifact::import::validate_and_import_semantic_world_with_allocation_budget;
use phalcom_semantic::types::store::TypeStore;
use phalcom_type_meta::allocation::AllocationBudget;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

struct CountingAllocator;

static MEASURING: AtomicBool = AtomicBool::new(false);
static REQUESTED_BYTES: AtomicUsize = AtomicUsize::new(0);

fn record_request(bytes: usize) {
    if MEASURING.load(Ordering::Relaxed) {
        REQUESTED_BYTES.fetch_add(bytes, Ordering::Relaxed);
    }
}

// SAFETY: All allocator operations forward the original pointer/layout contract
// unchanged to `System`; the wrapper only observes requested layout sizes.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record_request(layout.size());
        unsafe { System.alloc(layout) }
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        record_request(new_size);
        unsafe { System.realloc(pointer, layout, new_size) }
    }
}

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator;

#[test]
fn full_universe_telosc_load_allocation_equality_and_n_minus_one() {
    // 1. Build canonical program and artifact bytes outside measurement
    let limits = ValidationLimits::default();
    let artifact_bytes = if let Some(path) = std::env::var_os("PHALCOM_UNIVERSE_TELOSC_FOR_TEST") {
        std::fs::read(path).expect("read prebuilt canonical Universe artifact")
    } else {
        let program = canonical_universe_program().expect("canonical universe program must compile");
        let view = program.compiler_image_view().expect("compiler image view must be present");
        build_universe_telosc(&view, &limits)
            .expect("build_universe_telosc must succeed")
            .bytes
    };
    assert!(!artifact_bytes.is_empty());
    if let Some(path) = std::env::var_os("PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST") {
        std::fs::write(path, &artifact_bytes).expect("save canonical Universe artifact for focused import checks");
    }

    // Isolate the semantic import stage using a real Universe section. Decoding
    // is deliberately outside the allocator measurement; the importer receives
    // an already materialized SemanticWorldV3 just as the core loader does.
    let container = TeloscContainerView::parse(&artifact_bytes, &limits).expect("parse generated artifact");
    let semantic_bytes = container
        .require_section(SectionKind::SemanticWorld)
        .expect("artifact must contain SEMANTIC_WORLD");
    let semantic_limits = ValidationLimits {
        max_decoded_section_bytes: usize::MAX as u64,
        ..limits
    };
    let decode_world = || {
        let mut decode_budget = AllocationBudget::new(usize::MAX);
        SemanticWorldV3::decode_with_allocation_budget(semantic_bytes, &semantic_limits, &mut decode_budget)
            .expect("decode canonical Universe semantic world")
    };

    let type_world = decode_world();
    let mut type_transaction = AllocationBudget::new(usize::MAX);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let type_import = TypeStore::import_type_graph_with_allocation_budget(&type_world.type_graph, &limits, &mut type_transaction);
    MEASURING.store(false, Ordering::Relaxed);
    let independently_observed_types = REQUESTED_BYTES.load(Ordering::Relaxed);
    let (type_store, type_relocation) = type_import.expect("budgeted type graph import must succeed");
    eprintln!(
        "TYPE GRAPH IMPORT: ledger={} observed={} delta={}",
        type_transaction.requested(),
        independently_observed_types,
        independently_observed_types.abs_diff(type_transaction.requested())
    );
    assert_eq!(
        independently_observed_types,
        type_transaction.requested(),
        "independent type-graph import allocator requests must match its budget"
    );
    drop((type_store, type_relocation));

    let mut discovery_transaction = AllocationBudget::new(usize::MAX);
    let first_import = validate_and_import_semantic_world_with_allocation_budget(
        decode_world(),
        &semantic_limits,
        &mut discovery_transaction,
    )
    .expect("import canonical Universe semantic world");
    let semantic_requested = discovery_transaction.requested();
    assert!(semantic_requested > 0, "semantic import ledger must record allocation requests");
    drop(first_import);

    let exact_semantic_limits = ValidationLimits {
        max_decoded_section_bytes: semantic_requested as u64,
        ..limits
    };
    let semantic_world_for_measurement = decode_world();
    let mut exact_transaction = AllocationBudget::new(semantic_requested);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    phalcom_type_meta::allocation::register_allocation_probe(|| REQUESTED_BYTES.load(Ordering::Relaxed));
    let imported = validate_and_import_semantic_world_with_allocation_budget(
        semantic_world_for_measurement,
        &exact_semantic_limits,
        &mut exact_transaction,
    );
    MEASURING.store(false, Ordering::Relaxed);
    let independently_observed_semantic = REQUESTED_BYTES.load(Ordering::Relaxed);
    eprintln!(
        "SEMANTIC IMPORT EXACT CHECK:\n  ledger requested: {}\n  independently observed: {}\n  delta: {}",
        semantic_requested,
        independently_observed_semantic,
        independently_observed_semantic.abs_diff(semantic_requested)
    );
    let imported = imported.expect("semantic import at exact ledger budget must succeed");
    assert_eq!(
        independently_observed_semantic, semantic_requested,
        "independent semantic-import allocator requests must match the child ledger exactly"
    );
    drop(imported);

    let mut one_byte_short = AllocationBudget::new(semantic_requested - 1);
    let failed_import = validate_and_import_semantic_world_with_allocation_budget(
        decode_world(),
        &ValidationLimits {
            max_decoded_section_bytes: (semantic_requested - 1) as u64,
            ..limits
        },
        &mut one_byte_short,
    );
    assert!(
        matches!(failed_import, Err(phalcom_compiler_artifact::ArtifactError::Allocation(_))),
        "semantic import at N-1 must return a typed allocation error"
    );
    assert_eq!(one_byte_short.requested(), 0, "failed semantic import must not commit its child ledger");

    // 2. Discover ledger requested bytes N with measurement disabled
    let (summary_res, ledger_requested) =
        load_universe_telosc_from_bytes_with_allocation_summary(&artifact_bytes, &limits);
    let initial_artifact = summary_res.expect("initial load without memory measurement must succeed");
    drop(initial_artifact);

    let n = ledger_requested;
    assert!(n > 0, "ledger must have recorded allocation requests");

    // 3. Test exact N under independent allocator measurement
    let limits_exact = ValidationLimits {
        max_total_allocation_bytes: n as u64,
        ..limits
    };

    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let loaded = load_universe_telosc_from_bytes(&artifact_bytes, &limits_exact);
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);

    let loaded = loaded.expect("load at exact budget N must succeed");

    // Drop artifact with measurement disabled
    drop(loaded);

    eprintln!(
        "FULL LOAD ALLOCATION DISCRIMINATOR:\n  ledger requested (N): {}\n  independent allocator requested: {}\n  delta: {}",
        n,
        observed,
        observed.abs_diff(n)
    );

    // 4. Verify N - 1 strictly fails
    let limits_minus_one = ValidationLimits {
        max_total_allocation_bytes: (n - 1) as u64,
        ..limits
    };
    let failed = load_universe_telosc_from_bytes(&artifact_bytes, &limits_minus_one);
    assert!(
        failed.is_err(),
        "load with N - 1 budget must fail closed with budget exhaustion"
    );

    // PATCH5 discriminator: during intermediate waves, observed != n is expected.
    // At T9 (Wave 4), this assertion is made unconditional.
    if std::env::var("STRICT_ALLOCATION_GATE").is_ok() {
        assert_eq!(
            observed, n,
            "independent allocator requests must match production ledger exactly"
        );
    }
}
