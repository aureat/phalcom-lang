use phalcom_compiler_artifact::container::TeloscContainerView;
use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_compiler_artifact::module_world::ModuleWorldV2;
use phalcom_compiler_artifact::query::QueryImageV4;
use phalcom_compiler_artifact::section::SectionKind;
use phalcom_compiler_artifact::semantic::world::SemanticWorldV4;
use phalcom_core::modules::canonical_artifact::{
    TeloscArtifactError, build_universe_telosc, load_universe_telosc_from_bytes, load_universe_telosc_from_bytes_with_allocation_summary,
    loader_allocation_diagnostic_stages, reset_loader_allocation_diagnostics,
};
use phalcom_core::modules::canonical_universe::canonical_universe_program;
use phalcom_semantic::SemanticWorkspaceSession;
use phalcom_semantic::artifact::import::{
    callable_import_allocation_diagnostic_stages,
    import_allocation_diagnostic_stages,
    reset_import_allocation_diagnostics,
    validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget,
};
use phalcom_semantic::artifact::query::reconstruct_query_image_v4_with_allocation_budget;
use phalcom_semantic::canonical_image::CanonicalSemanticWorldId;
use phalcom_semantic::identity::WorkspaceId;
use phalcom_semantic::types::store::TypeStore;
use phalcom_type_meta::allocation::AllocationBudget;
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::Arc;
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
        build_universe_telosc(&view, &limits).expect("build_universe_telosc must succeed").bytes
    };
    assert!(!artifact_bytes.is_empty());
    if let Some(path) = std::env::var_os("PHALCOM_SAVE_UNIVERSE_TELOSC_FOR_TEST") {
        std::fs::write(path, &artifact_bytes).expect("save canonical Universe artifact for focused import checks");
    }

    // Isolate the semantic import stage using a real Universe section. Decoding
    // is deliberately outside the allocator measurement; the importer receives
    // an already materialized SemanticWorldV4 and exact module authority,
    // matching the current core loader.
    let container = TeloscContainerView::parse(&artifact_bytes, &limits).expect("parse generated artifact");

    // Isolate MODULE_WORLD materialization
    let module_bytes = container.require_section(SectionKind::ModuleWorld).expect("artifact must contain MODULE_WORLD");
    let mut decode_module_budget = AllocationBudget::new(usize::MAX);
    let module_dto =
        ModuleWorldV2::decode_with_allocation_budget(module_bytes, &limits, &mut decode_module_budget).expect("decode canonical Universe module world");
    let mut module_transaction = AllocationBudget::new(usize::MAX);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let module_res = phalcom_modules::compiler_artifact::load_module_world_with_allocation_budget(&module_dto, &limits, &mut module_transaction);
    MEASURING.store(false, Ordering::Relaxed);
    let independently_observed_module = REQUESTED_BYTES.load(Ordering::Relaxed);
    let module_requested = module_transaction.requested();
    let module_delta = independently_observed_module as isize - module_requested as isize;
    eprintln!(
        "MODULE MATERIALIZATION:\n  ledger requested: {}\n  independently observed: {}\n  signed delta: {}",
        module_requested, independently_observed_module, module_delta
    );
    let modules = module_res.expect("module materialization must succeed");
    assert_eq!(
        independently_observed_module, module_requested,
        "independent module materialization allocator requests must match its ledger exactly"
    );

    let semantic_bytes = container
        .require_section(SectionKind::SemanticWorld)
        .expect("artifact must contain SEMANTIC_WORLD");
    let semantic_limits = ValidationLimits {
        max_decoded_section_bytes: usize::MAX as u64,
        ..limits
    };
    let decode_world = || {
        let mut decode_budget = AllocationBudget::new(usize::MAX);
        SemanticWorldV4::decode_with_allocation_budget(semantic_bytes, &semantic_limits, &mut decode_budget).expect("decode canonical Universe semantic world")
    };

    let type_world = decode_world();
    let mut validation_transaction = AllocationBudget::new(usize::MAX);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let validation = type_world.validate_with_allocation_budget(&semantic_limits, &mut validation_transaction);
    MEASURING.store(false, Ordering::Relaxed);
    let independently_observed_validation = REQUESTED_BYTES.load(Ordering::Relaxed);
    validation.expect("validate canonical Universe semantic world");
    let validation_requested = validation_transaction.requested();
    eprintln!(
        "SEMANTIC WORLD V4 VALIDATION: ledger={} observed={} delta={}",
        validation_requested,
        independently_observed_validation,
        independently_observed_validation.abs_diff(validation_requested)
    );
    assert_eq!(
        independently_observed_validation, validation_requested,
        "independent V4 validation allocator requests must match its ledger"
    );

    let exact_validation_limits = ValidationLimits {
        max_decoded_section_bytes: validation_requested as u64,
        ..limits
    };
    let mut exact_validation = AllocationBudget::new(validation_requested);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let exact_validation_result = type_world.validate_with_allocation_budget(&exact_validation_limits, &mut exact_validation);
    MEASURING.store(false, Ordering::Relaxed);
    let exact_validation_observed = REQUESTED_BYTES.load(Ordering::Relaxed);
    exact_validation_result.expect("V4 validation at its exact ledger budget must succeed");
    assert_eq!(exact_validation_observed, validation_requested);
    assert_eq!(exact_validation.requested(), validation_requested);

    let mut decode_validation_transaction = AllocationBudget::new(usize::MAX);
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    let decode_validation = SemanticWorldV4::decode_with_allocation_budget(semantic_bytes, &semantic_limits, &mut decode_validation_transaction);
    MEASURING.store(false, Ordering::Relaxed);
    let independently_observed_decode_validation = REQUESTED_BYTES.load(Ordering::Relaxed);
    decode_validation.expect("decode and validate canonical Universe semantic world");
    let decode_validation_requested = decode_validation_transaction.requested();
    eprintln!(
        "SEMANTIC WORLD V4 DECODE + VALIDATION: ledger={} observed={} delta={}",
        decode_validation_requested,
        independently_observed_decode_validation,
        independently_observed_decode_validation.abs_diff(decode_validation_requested)
    );
    assert_eq!(
        independently_observed_decode_validation, decode_validation_requested,
        "independent V4 decode-plus-validation allocator requests must match its ledger"
    );

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
    let first_import = validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget(
        decode_world(),
        &module_dto,
        modules.linked().universe.as_ref(),
        &semantic_limits,
        &mut discovery_transaction,
    )
    .expect("import canonical Universe semantic world");
    let semantic_requested = discovery_transaction.requested();
    assert!(semantic_requested > 0, "semantic import ledger must record allocation requests");

    let exact_semantic_limits = ValidationLimits {
        max_decoded_section_bytes: semantic_requested as u64,
        ..limits
    };
    let semantic_world_for_measurement = decode_world();
    let mut exact_transaction = AllocationBudget::new(semantic_requested);
    reset_import_allocation_diagnostics();
    REQUESTED_BYTES.store(0, Ordering::Relaxed);
    MEASURING.store(true, Ordering::Relaxed);
    phalcom_type_meta::allocation::register_allocation_probe(|| REQUESTED_BYTES.load(Ordering::Relaxed));
    let imported = validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget(
        semantic_world_for_measurement,
        &module_dto,
        modules.linked().universe.as_ref(),
        &exact_semantic_limits,
        &mut exact_transaction,
    );
    MEASURING.store(false, Ordering::Relaxed);
    let independently_observed_semantic = REQUESTED_BYTES.load(Ordering::Relaxed);
    let diagnostic_stages = import_allocation_diagnostic_stages();
    eprintln!(
        "SEMANTIC IMPORT EXACT CHECK:\n  ledger requested: {}\n  independently observed: {}\n  delta: {}",
        semantic_requested,
        independently_observed_semantic,
        independently_observed_semantic.abs_diff(semantic_requested)
    );
    for pair in diagnostic_stages.windows(2) {
        let ledger_delta = pair[1].ledger_bytes - pair[0].ledger_bytes;
        let observed_delta = pair[1].observed_bytes - pair[0].observed_bytes;
        if ledger_delta == observed_delta {
            continue;
        }
        eprintln!(
            "SEMANTIC IMPORT STAGE {} -> {}: ledger={} observed={} delta={}",
            pair[0].name,
            pair[1].name,
            ledger_delta,
            observed_delta,
            observed_delta.abs_diff(ledger_delta),
        );
    }
    for phase in callable_import_allocation_diagnostic_stages() {
        if phase.ledger_bytes == phase.observed_bytes {
            continue;
        }
        eprintln!(
            "CALLABLE IMPORT PHASE {}: ledger={} observed={} delta={}",
            phase.name,
            phase.ledger_bytes,
            phase.observed_bytes,
            phase.observed_bytes.abs_diff(phase.ledger_bytes),
        );
    }
    let imported = imported.expect("semantic import at exact ledger budget must succeed");
    assert_eq!(
        independently_observed_semantic, semantic_requested,
        "independent semantic-import allocator requests must match the child ledger exactly"
    );
    drop(imported);

    let mut one_byte_short = AllocationBudget::new(semantic_requested - 1);
    let failed_import = validate_and_import_semantic_world_with_module_world_and_project_universe_and_allocation_budget(
        decode_world(),
        &module_dto,
        modules.linked().universe.as_ref(),
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

    // Reconstruct the current query image against the imported semantic
    // snapshot and exact module authority, matching the production V3 loader.
    let mut query_decode_budget = AllocationBudget::new(usize::MAX);
    let query_bytes = container.require_section(SectionKind::QueryImage).expect("artifact must contain QUERY_IMAGE");
    let query_world = decode_world();
    let query_dto =
        QueryImageV4::decode_with_allocation_budget(query_bytes, &semantic_limits, &mut query_decode_budget).expect("decode canonical Universe query image");
    query_dto.validate(&semantic_limits).expect("validate canonical Universe query image");
    query_dto
        .validate_product_references(&module_dto, &query_world)
        .expect("validate query products against current module and semantic worlds");
    query_dto
        .validate_portable_structural_fingerprints_v4(&query_world)
        .expect("validate current query structural fingerprints");
    let header = container.header();
    let compatibility = phalcom_compiler_artifact::CanonicalCompatibilityIdentity {
        package: header.package_identity.clone(),
        source_digest: header.source_digest,
        interface_digest: header.interface_digest,
        semantic_digest: header.semantic_digest,
        semantic_model_version: header.semantic_model_version,
        stable_identity_schema: header.stable_identity_schema,
        module_semantic_abi: header.module_semantic_abi,
        semantic_artifact_abi: header.semantic_artifact_abi,
        query_fingerprint_abi: header.query_fingerprint_abi,
        native_surface_schema_version: header.native_surface_schema_version,
        native_catalog_fingerprint: header.native_catalog_fingerprint,
        type_metadata_schema_version: header.type_metadata_schema_version,
    };
    let world_id = CanonicalSemanticWorldId::for_artifact_world(1, header.source_digest, compatibility);
    let mut query_transaction = AllocationBudget::new(usize::MAX);
    let (query_image, _) =
        reconstruct_query_image_v4_with_allocation_budget(query_dto, &first_import.snapshot, &world_id, &modules, &semantic_limits, &mut query_transaction)
            .expect("reconstruct canonical Universe query image");
    assert!(
        query_image.entries().next().is_some(),
        "canonical Universe query image must retain its required roots"
    );
    drop((first_import, query_image, modules));

    // 2. Discover ledger requested bytes N with measurement disabled
    let (summary_res, ledger_requested) = load_universe_telosc_from_bytes_with_allocation_summary(&artifact_bytes, &limits);
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
    phalcom_semantic::artifact::query::reset_query_reconstruction_census();
    phalcom_semantic::canonical_image::reset_loaded_publication_census();
    reset_loader_allocation_diagnostics();
    let loaded = load_universe_telosc_from_bytes(&artifact_bytes, &limits_exact);
    MEASURING.store(false, Ordering::Relaxed);
    let observed = REQUESTED_BYTES.load(Ordering::Relaxed);
    let load_stages = loader_allocation_diagnostic_stages();
    for pair in load_stages.windows(2) {
        let ledger_delta = pair[1].ledger_bytes - pair[0].ledger_bytes;
        let observed_delta = pair[1].observed_bytes - pair[0].observed_bytes;
        assert_eq!(
            observed_delta,
            ledger_delta,
            "whole-load stage {} -> {} must account for every request",
            pair[0].name,
            pair[1].name,
        );
    }
    let query_census = phalcom_semantic::artifact::query::take_query_reconstruction_census();
    for (name, row) in [
        ("Q0 closure/root validation", query_census.closure_and_roots),
        ("Q1 owner decode + QueryKey reconstruction", query_census.owner_key_resolution),
        ("Q2 product + fingerprint reconstruction", query_census.product_reconstruction),
        ("Q3 dependency/entry/fingerprint assembly", query_census.entry_assembly),
        ("Q4 loaded query-image validation", query_census.loaded_validation),
        ("Q5 query image Arc publication", query_census.query_image_publication),
    ] {
        assert_eq!(
            row.observed_bytes,
            row.ledger_bytes,
            "query reconstruction stage {name} must account for every request"
        );
    }
    for row in &query_census.product_by_kind {
        assert_eq!(row.observed_bytes, row.ledger_bytes, "query product {:?} must account for every request", row.kind);
    }
    assert_eq!(
        query_census.product_buffer_observed,
        query_census.product_buffer_ledger,
        "query reconstruction product buffer must account for every request"
    );
    let whole_query_stage = load_stages
        .windows(2)
        .find(|pair| pair[1].name == "query reconstruction")
        .expect("whole-load query reconstruction boundary must be recorded");
    let query_census_observed = [
        query_census.closure_and_roots,
        query_census.owner_key_resolution,
        query_census.product_reconstruction,
        query_census.entry_assembly,
        query_census.loaded_validation,
        query_census.query_image_publication,
    ]
    .iter()
    .map(|row| row.observed_bytes)
    .sum::<usize>();
    let query_census_ledger = [
        query_census.closure_and_roots,
        query_census.owner_key_resolution,
        query_census.product_reconstruction,
        query_census.entry_assembly,
        query_census.loaded_validation,
        query_census.query_image_publication,
    ]
    .iter()
    .map(|row| row.ledger_bytes)
    .sum::<usize>();
    let qx_observed = whole_query_stage[1].observed_bytes - whole_query_stage[0].observed_bytes - query_census_observed;
    let qx_ledger = whole_query_stage[1].ledger_bytes - whole_query_stage[0].ledger_bytes - query_census_ledger;
    assert_eq!(qx_observed, 0, "QX must contain no unclassified allocator requests");
    assert_eq!(qx_ledger, 0, "QX must contain no unclassified ledger requests");
    let publication = phalcom_semantic::canonical_image::take_loaded_publication_census();
    assert_eq!(publication.fingerprint_aggregate_observed, 0, "fingerprint aggregation must not allocate");
    assert_eq!(publication.fingerprint_query_lookup_observed, 0, "fingerprint query lookup must not allocate");
    assert!(
        publication.fingerprint_aggregate_by_kind.is_empty(),
        "fingerprint aggregation by query kind must not allocate: {:?}",
        publication.fingerprint_aggregate_by_kind
    );
    for (name, ledger, stage_observed) in [
        ("snapshot clone", publication.snapshot_clone_ledger, publication.snapshot_clone),
        ("status promotion", publication.status_promotion_ledger, publication.status_promotion),
        ("ModuleQueryProducts reconstruction", publication.module_query_products_ledger, publication.module_query_products),
        (
            "ModuleQueryProducts Arc/publication",
            publication.module_query_products_publication_ledger,
            publication.module_query_products_publication,
        ),
        ("source-free completeness", publication.source_free_completeness_ledger, publication.source_free_completeness),
        ("advisory validation", publication.advisory_validation_ledger, publication.advisory_validation),
        ("query reader validation", publication.query_reader_validation_ledger, publication.query_reader_validation),
        ("fingerprint validation", publication.fingerprint_validation_ledger, publication.fingerprint_validation),
        ("final Arc<SemanticSnapshot>", publication.snapshot_arc_ledger, publication.snapshot_arc),
        ("Arc<CanonicalUniverseFormalImage>", publication.formal_image_arc_ledger, publication.formal_image_arc),
    ] {
        assert_eq!(
            stage_observed,
            ledger,
            "formal publication stage {name} must account for every request"
        );
    }

    let loaded = loaded.expect("load at exact budget N must succeed");

    // Contract B: the artifact-load transaction ends at the immutable,
    // complete image. Ordinary session construction is a separate infallible
    // in-memory handoff over that already-published image and is deliberately
    // outside N's loader allocation measurement.
    let image = loaded.formal_image.clone();
    assert!(image.snapshot().is_source_free_artifact_complete());
    let session = SemanticWorkspaceSession::from_image(WorkspaceId::from_raw(9001), image.clone());
    assert!(Arc::ptr_eq(
        session.last_snapshot().expect("image session publishes its initial snapshot"),
        image.snapshot(),
    ));
    assert!(Arc::ptr_eq(
        session
            .last_known_good_snapshot()
            .expect("image session retains its initial last-known-good snapshot"),
        image.snapshot(),
    ));

    // Drop artifact with measurement disabled
    drop((session, loaded));

    eprintln!(
        "FULL LOAD ALLOCATION DISCRIMINATOR:\n  ledger requested (N): {}\n  independent allocator requested: {}\n  delta: {}",
        n,
        observed,
        observed.abs_diff(n)
    );

    // 4. Verify N - 1 strictly fails with typed cumulative allocation exhaustion
    let limits_minus_one = ValidationLimits {
        max_total_allocation_bytes: (n - 1) as u64,
        ..limits
    };
    let (failed_cumulative, failed_cumulative_requested) = load_universe_telosc_from_bytes_with_allocation_summary(&artifact_bytes, &limits_minus_one);
    match failed_cumulative {
        Err(TeloscArtifactError::BudgetExceeded { limit_bytes }) => {
            assert_eq!(limit_bytes, n - 1, "cumulative exhaustion must record limit N - 1");
        }
        Err(other) => panic!("expected TeloscArtifactError::BudgetExceeded, got {other:?}"),
        Ok(_) => panic!("load with N - 1 budget must fail closed with budget exhaustion"),
    }
    assert!(failed_cumulative_requested <= n - 1, "failed load must not exceed limit");

    // Clean retry after cumulative allocation failure
    let retry_cumulative =
        load_universe_telosc_from_bytes(&artifact_bytes, &limits_exact).expect("clean retry after cumulative allocation exhaustion must succeed");
    drop(retry_cumulative);

    // 5. Verify local stage limit failure is distinct from cumulative exhaustion
    let local_section_limit = decode_module_budget.requested().saturating_add(module_transaction.requested());
    let limits_local_stage = ValidationLimits {
        max_total_allocation_bytes: n as u64,
        max_decoded_section_bytes: u64::try_from(local_section_limit).expect("local section limit fits u64"),
        ..limits
    };
    let (failed_local, failed_local_requested) = load_universe_telosc_from_bytes_with_allocation_summary(&artifact_bytes, &limits_local_stage);
    match failed_local {
        Err(TeloscArtifactError::Artifact(phalcom_compiler_artifact::ArtifactError::Allocation(phalcom_type_meta::allocation::AllocationError::Exhausted))) => {
        }
        Err(other) => panic!("expected typed local AllocationError::Exhausted, got {other:?}"),
        Ok(_) => panic!("load with insufficient section budget must fail closed"),
    }
    // Measure the exact preceding production stages to establish the expected
    // parent debit when the module materialization child transaction fails.
    let mut preceding_stage_budget = AllocationBudget::new(usize::MAX);
    let preceding_container =
        TeloscContainerView::parse_with_allocation_budget(&artifact_bytes, &limits, &mut preceding_stage_budget).expect("preceding container stage must parse");
    let preceding_module_bytes = preceding_container
        .require_section(SectionKind::ModuleWorld)
        .expect("preceding module section must exist");
    let preceding_module_dto =
        ModuleWorldV2::decode_with_allocation_budget(preceding_module_bytes, &limits, &mut preceding_stage_budget).expect("preceding module DTO stage must decode");
    phalcom_modules::compiler_artifact::load_module_world_with_allocation_budget(&preceding_module_dto, &limits, &mut preceding_stage_budget)
        .expect("preceding module materialization stage must succeed");
    let expected_preceding_debit = preceding_stage_budget.requested();
    assert_eq!(
        failed_local_requested, expected_preceding_debit,
        "failed child transaction must not commit its debit to parent"
    );

    // Clean retry after local stage allocation failure
    let retry_local =
        load_universe_telosc_from_bytes(&artifact_bytes, &limits_exact).expect("clean retry after local stage allocation exhaustion must succeed");
    drop(retry_local);

    assert_eq!(observed, n, "independent allocator requests must match production ledger exactly");
}
