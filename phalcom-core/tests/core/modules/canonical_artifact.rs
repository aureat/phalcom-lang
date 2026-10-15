use phalcom_compiler_artifact::limits::ValidationLimits;
use phalcom_core::modules::canonical_artifact::{build_universe_telosc, load_universe_telosc, load_universe_telosc_from_bytes, write_universe_telosc};
use phalcom_core::modules::canonical_universe::canonical_universe_program;
use phalcom_core::modules::compile::EntrySelection;
use tempfile::tempdir;

#[test]
fn build_and_load_universe_telosc_compiles_source_free_inline_user_entry() {
    let semantic_image = phalcom_semantic::canonical_image::canonical_universe_formal_image().expect("canonical semantic image");
    let semantic_snapshot = semantic_image.snapshot();
    for (callable, analysis) in semantic_snapshot.callable_analyses.iter().filter(|(callable, analysis)| {
        callable.module().to_string() == "universe:numbers"
            && analysis.body_source_owner == Some(phalcom_semantic::identity::SourceOwner::Module(callable.module().clone()))
    }) {
        let imported_i = analysis
            .bindings
            .values()
            .find(|binding| binding.name == "i")
            .expect("numbers i import binding");
        assert!(
            matches!(
                analysis.binding_origins.get(&imported_i.binding),
                Some(phalcom_semantic::checker::analysis::CallableBindingOrigin::ModuleValue {
                    symbol,
                    import: Some(_),
                }) if symbol.name.as_ref() == "i"
            ),
            "top-level imported i must retain its canonical module storage identity: {callable:?}, binding={imported_i:?}, origin={:?}",
            analysis.binding_origins.get(&imported_i.binding),
        );
        let imported_source = semantic_snapshot
            .source_index
            .module(callable.module())
            .expect("numbers source shard")
            .structure
            .bindings
            .values()
            .find(|binding| binding.name.as_ref() == "i" && binding.kind == phalcom_semantic::SourceBindingKind::Import)
            .expect("local import declaration site");
        assert_eq!(
            imported_source.declaration_site.owner,
            phalcom_semantic::identity::SourceOwner::Module(callable.module().clone())
        );
    }
    let internal_incidents = semantic_snapshot
        .callable_analyses
        .iter()
        .filter(|(_, analysis)| !analysis.internal_incidents.is_empty())
        .map(|(callable, analysis)| (callable.clone(), analysis.internal_incidents.clone(), analysis.status.clone()))
        .collect::<Vec<_>>();
    assert!(internal_incidents.is_empty(), "canonical callable internal incidents: {internal_incidents:#?}");
    let program = canonical_universe_program().expect("canonical universe program must compile");
    let view = program.compiler_image_view().expect("compiler image view must be present");
    let limits = ValidationLimits::default();

    // 1. Build deterministic universe.telosc artifact:
    let built = build_universe_telosc(&view, &limits).expect("build_universe_telosc must succeed");
    assert!(!built.bytes.is_empty());
    assert!(!built.summary.capabilities.portable_executable);
    assert!(built.summary.capabilities.module_world);
    assert!(built.summary.capabilities.semantic_world);
    assert!(built.summary.capabilities.query_image);
    assert!(built.summary.capabilities.runtime_public_metadata);
    assert!(built.summary.module_world_bytes > 0);
    assert!(built.summary.semantic_world_bytes > 0);
    assert!(built.summary.query_image_bytes > 0);
    assert!(built.summary.metadata_bytes > 0);

    // 2. Determinism check: building again from the same canonical program produces byte-identical output:
    let built2 = build_universe_telosc(&view, &limits).expect("second build must succeed");
    assert_eq!(built.bytes, built2.bytes, "build_universe_telosc must be bitwise deterministic");
    assert_eq!(built.summary.artifact_digest, built2.summary.artifact_digest);

    // 3. Atomic publication:
    let dir = tempdir().unwrap();
    let path = dir.path().join("universe.telosc");
    write_universe_telosc(&path, &built).expect("write_universe_telosc must succeed");
    assert!(path.is_file());

    // 4. Source-free loader from disk:
    let loaded_from_disk = load_universe_telosc(&path, &limits).expect("load_universe_telosc must succeed");
    assert!(loaded_from_disk.formal_image.sources().is_empty(), "loaded image must not retain source units");
    assert!(
        loaded_from_disk.formal_image.snapshot().is_source_free_artifact_complete(),
        "loaded snapshot must be source-free complete"
    );

    // 5. Source-free loader from in-memory bytes:
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits).expect("load_universe_telosc_from_bytes must succeed");
    assert!(loaded.formal_image.sources().is_empty());
    assert!(loaded.formal_image.snapshot().is_source_free_artifact_complete());

    let snapshot = loaded.formal_image.snapshot();
    assert!(!snapshot.field_signatures.is_empty(), "field signatures must be restored");
    assert!(snapshot.trait_headers.len() > 0, "trait headers must be restored");
    assert!(snapshot.trait_surfaces.iter().next().is_some(), "trait surfaces must be restored");
    let sample_surface = snapshot.trait_surfaces.iter().find(|(_, s)| !s.members.is_empty()).map(|(_, s)| s);
    assert!(sample_surface.is_some(), "trait surface members must be populated");
    assert!(snapshot.constructor_initializations.len() > 0, "constructor initializations must be restored");
    assert!(snapshot.conformance_index.iter().next().is_some(), "conformance index must be restored");
    let (_, sample_head) = snapshot.conformance_index.iter().next().unwrap();
    assert!(sample_head.authorized, "conformance head must preserve authorized flag");
    assert!(!snapshot.conformance_witness_plans.is_empty(), "witness plans must be restored");
    let sample_witness = snapshot
        .conformance_witness_plans
        .iter()
        .find(|(_, w)| !w.requirements.is_empty())
        .map(|(_, w)| w);
    assert!(sample_witness.is_some(), "witness plan requirements must be restored");
    assert!(matches!(
        sample_witness.unwrap().completeness,
        phalcom_semantic::impls::ConformanceCompleteness::Complete
    ));

    // 6. User compilation consuming the source-free loaded image:
    let compiled = loaded
        .compile_user_entry(EntrySelection::Inline("print(42);".into()))
        .expect("compiling user program against loaded universe image must succeed");
    assert!(!compiled.modules.is_empty());
    assert!(compiled.entry.path.is_root());
}

#[test]
fn load_universe_telosc_rejects_corrupted_artifact() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let mut built = build_universe_telosc(&view, &limits).expect("build");

    // Corrupt one byte in the container payload:
    let mid = built.bytes.len() / 2;
    built.bytes[mid] ^= 0xFF;

    let res = load_universe_telosc_from_bytes(&built.bytes, &limits);
    assert!(res.is_err(), "corrupted container bytes must be rejected");
}

#[test]
fn foreign_native_catalog_fingerprint_is_rejected_before_section_materialization() {
    use phalcom_compiler_artifact::{
        ArtifactDigest,
        container::TeloscContainerBuilder,
        header::{TELOSC_CONTAINER_SCHEMA_V2, TELOSC_MIN_READER_SCHEMA_V2, TeloscCapabilities, TeloscHeaderV2},
        identity::TeloscPackageIdentity,
        section::{SectionFlags, SectionKind},
    };

    let mut foreign_catalog = phalcom_native_surface::catalog_fingerprint().0;
    foreign_catalog[0] ^= 0xff;
    let header = TeloscHeaderV2 {
        container_schema: TELOSC_CONTAINER_SCHEMA_V2,
        minimum_reader_schema: TELOSC_MIN_READER_SCHEMA_V2,
        package_identity: TeloscPackageIdentity::universe(),
        semantic_model_version: phalcom_type_meta::SEMANTIC_MODEL_VERSION,
        stable_identity_schema: 1,
        module_semantic_abi: 1,
        semantic_artifact_abi: 1,
        query_fingerprint_abi: 1,
        native_surface_schema_version: phalcom_native_meta::NATIVE_SURFACE_SCHEMA_VERSION,
        native_catalog_fingerprint: foreign_catalog,
        type_metadata_schema_version: phalcom_type_meta::TYPE_METADATA_SCHEMA_VERSION,
        capabilities: TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1 | TeloscCapabilities::RUNTIME_PUBLIC_METADATA,
        source_digest: ArtifactDigest([1; 32]),
        interface_digest: ArtifactDigest([2; 32]),
        semantic_digest: ArtifactDigest([3; 32]),
        query_digest: ArtifactDigest([4; 32]),
        metadata_digest: ArtifactDigest([5; 32]),
        artifact_digest: ArtifactDigest::ZERO,
        section_count: 0,
    };
    let mut builder = TeloscContainerBuilder::new(header);
    for kind in [
        SectionKind::ModuleWorld,
        SectionKind::SemanticWorld,
        SectionKind::QueryImage,
        SectionKind::Metadata,
    ] {
        builder.add_section(kind, 1, SectionFlags::REQUIRED, vec![0xff]);
    }
    let bytes = builder.build().expect("container integrity should be valid");

    assert!(matches!(
        load_universe_telosc_from_bytes(&bytes, &ValidationLimits::default()),
        Err(phalcom_core::modules::canonical_artifact::TeloscArtifactError::Artifact(
            phalcom_compiler_artifact::ArtifactError::Validation(phalcom_compiler_artifact::ValidationError::NativeCatalogFingerprintMismatch { .. })
        ))
    ));
}

#[test]
fn write_universe_telosc_rejects_portable_executable_capability() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let mut built = build_universe_telosc(&view, &limits).expect("build");

    // P7 compiler-only writer must refuse PORTABLE_EXECUTABLE:
    built.summary.capabilities.portable_executable = true;
    let dir = tempdir().unwrap();
    let path = dir.path().join("universe.telosc");
    assert!(write_universe_telosc(&path, &built).is_err());
    assert!(!path.exists());
}

#[test]
fn load_universe_telosc_enforces_budget_limits_and_records_timings() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build");

    // Reject when decoded section byte limit is too small:
    let tight_limits = ValidationLimits {
        max_decoded_section_bytes: 100,
        ..ValidationLimits::default()
    };
    let res = load_universe_telosc_from_bytes(&built.bytes, &tight_limits);
    assert!(res.is_err());

    // Successful load records load timings:
    let loaded = load_universe_telosc_from_bytes(&built.bytes, &limits).expect("load");
    assert!(loaded.timings.section_validation.as_nanos() > 0);
    assert!(loaded.timings.relocation.as_nanos() > 0);
}

#[test]
fn load_universe_telosc_enforces_cumulative_transaction_budget() {
    let program = canonical_universe_program().expect("canonical universe program");
    let view = program.compiler_image_view().expect("compiler image view");
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).expect("build");

    let max_single_section = built
        .summary
        .module_world_bytes
        .max(built.summary.semantic_world_bytes)
        .max(built.summary.query_image_bytes)
        .max(built.summary.metadata_bytes);

    let cumulative_insufficient = (max_single_section + 1024) as u64;
    let budget_limits = ValidationLimits {
        max_total_allocation_bytes: cumulative_insufficient,
        ..ValidationLimits::default()
    };
    let res = load_universe_telosc_from_bytes(&built.bytes, &budget_limits);
    assert!(
        res.is_err(),
        "cumulative budget must fail when total section allocations exceed transaction budget"
    );
}

#[test]
fn bounded_file_acquisition_rejects_oversized_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("oversized.telosc");
    let file = std::fs::File::create(&path).unwrap();
    file.set_len(1024 * 1024 * 1024).unwrap();
    let limits = ValidationLimits {
        max_artifact_bytes: 16,
        ..Default::default()
    };
    assert!(matches!(
        load_universe_telosc(&path, &limits),
        Err(phalcom_core::modules::canonical_artifact::TeloscArtifactError::BudgetExceeded { .. })
    ));
}

#[test]
fn writer_preserves_previous_output_when_bytes_are_invalid() {
    use phalcom_core::modules::canonical_artifact::{BuiltUniverseTelosc, TeloscBuildSummary, TeloscCapabilities};
    let dir = tempdir().unwrap();
    let path = dir.path().join("universe.telosc");
    std::fs::write(&path, b"previous output").unwrap();
    let built = BuiltUniverseTelosc {
        bytes: vec![1, 2, 3],
        summary: TeloscBuildSummary {
            capabilities: TeloscCapabilities::compiler_only_v1(),
            module_world_bytes: 0,
            semantic_world_bytes: 0,
            query_image_bytes: 0,
            metadata_bytes: 0,
            artifact_digest: [0; 32],
        },
    };
    assert!(write_universe_telosc(&path, &built).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"previous output");
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn loader_rejects_rehashed_incompatible_header_fields() {
    use phalcom_compiler_artifact::{
        container::{TeloscContainerBuilder, TeloscContainerView},
        header::TeloscCapabilities,
        section::{SectionFlags, SectionKind},
    };
    let program = canonical_universe_program().unwrap();
    let view = program.compiler_image_view().unwrap();
    let limits = ValidationLimits::default();
    let built = build_universe_telosc(&view, &limits).unwrap();
    let parsed = TeloscContainerView::parse(&built.bytes, &limits).unwrap();
    for field in 0..11 {
        let mut header = parsed.header().clone();
        match field {
            0 => header.semantic_model_version += 1,
            1 => header.stable_identity_schema += 1,
            2 => header.module_semantic_abi += 1,
            3 => header.semantic_artifact_abi += 1,
            4 => header.query_fingerprint_abi += 1,
            5 => header.native_surface_schema_version += 1,
            6 => header.type_metadata_schema_version += 1,
            7 => {
                header.package_identity = phalcom_compiler_artifact::identity::TeloscPackageIdentity::Builtin {
                    namespace: "other".into(),
                    version: "0.1.0".into(),
                }
            }
            8 => header.capabilities.remove(TeloscCapabilities::CANONICAL_COMPILER_IMAGE_V1),
            9 => header.capabilities.insert(TeloscCapabilities::PRESENTATION),
            10 => header.native_catalog_fingerprint[0] ^= 0xff,
            _ => unreachable!(),
        }
        let mut builder = TeloscContainerBuilder::new(header);
        for kind in [
            SectionKind::ModuleWorld,
            SectionKind::SemanticWorld,
            SectionKind::QueryImage,
            SectionKind::Metadata,
        ] {
            builder.add_section(kind, 1, SectionFlags::REQUIRED, parsed.require_section(kind).unwrap().to_vec());
        }
        let bytes = builder.build().unwrap();
        // Integrity remains valid; the reader must reject compatibility itself.
        TeloscContainerView::parse(&bytes, &limits).unwrap();
        let result = load_universe_telosc_from_bytes(&bytes, &limits);
        if field == 10 {
            assert!(
                matches!(
                    result,
                    Err(phalcom_core::modules::canonical_artifact::TeloscArtifactError::Artifact(
                        phalcom_compiler_artifact::ArtifactError::Validation(
                            phalcom_compiler_artifact::ValidationError::NativeCatalogFingerprintMismatch { .. }
                        )
                    ))
                ),
                "foreign native catalog fingerprint must fail before section materialization"
            );
        } else {
            assert!(result.is_err(), "header field {field}");
        }
    }
}
