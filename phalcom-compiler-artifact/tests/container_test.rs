use phalcom_compiler_artifact::*;

fn create_sample_header() -> TeloscHeaderV2 {
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
fn minimal_container_round_trips() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);

    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"module-world-bytes".to_vec());
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"semantic-world-bytes".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"query-image-bytes".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"metadata-bytes".to_vec());

    let bytes = builder.build().expect("build container");
    let limits = ValidationLimits::default();
    let view = TeloscContainerView::parse(&bytes, &limits).expect("parse container");

    assert_eq!(view.header().package_identity, TeloscPackageIdentity::universe());
    assert_eq!(view.header().native_catalog_fingerprint, [6; 16]);
    assert_eq!(view.get_section(SectionKind::ModuleWorld), Some(&b"module-world-bytes"[..]));
    assert_eq!(view.get_section(SectionKind::SemanticWorld), Some(&b"semantic-world-bytes"[..]));
    assert_eq!(view.get_section(SectionKind::QueryImage), Some(&b"query-image-bytes"[..]));
    assert_eq!(view.get_section(SectionKind::Metadata), Some(&b"metadata-bytes"[..]));
}

#[test]
fn container_directory_uses_the_shared_allocation_transaction() {
    use phalcom_type_meta::allocation::AllocationBudget;
    use std::alloc::Layout;

    let mut builder = TeloscContainerBuilder::new(create_sample_header());
    for (kind, bytes) in [
        (SectionKind::ModuleWorld, b"module".as_slice()),
        (SectionKind::SemanticWorld, b"semantic".as_slice()),
        (SectionKind::QueryImage, b"query".as_slice()),
        (SectionKind::Metadata, b"metadata".as_slice()),
    ] {
        builder.add_section(kind, 1, SectionFlags::REQUIRED, bytes.to_vec());
    }
    let bytes = builder.build().unwrap();
    let limits = ValidationLimits::default();
    let expected = Layout::array::<SectionDirectoryEntryV1>(4).unwrap().size() + "universe".len() + "0.1.0".len();

    let mut exact = AllocationBudget::new(expected);
    let view = TeloscContainerView::parse_with_allocation_budget(&bytes, &limits, &mut exact).unwrap();
    assert_eq!(view.directory.len(), 4);
    assert_eq!(exact.requested(), expected);
    assert_eq!(exact.remaining(), 0);

    let mut short = AllocationBudget::new(expected - 1);
    assert!(TeloscContainerView::parse_with_allocation_budget(&bytes, &limits, &mut short).is_err());
    assert_eq!((short.remaining(), short.requested()), (expected - 1, 0));
}

#[test]
fn header_is_fixed_endian() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m".to_vec());
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());

    let bytes = builder.build().expect("build");
    assert_eq!(&bytes[0..8], TELOSC_MAGIC);
    // container_schema u32 little endian: 2, 0, 0, 0
    assert_eq!(&bytes[8..12], &[2, 0, 0, 0]);
}

#[test]
fn container_schema_v1_is_rejected_without_reinterpreting_its_header() {
    let mut builder = TeloscContainerBuilder::new(create_sample_header());
    for kind in [
        SectionKind::ModuleWorld,
        SectionKind::SemanticWorld,
        SectionKind::QueryImage,
        SectionKind::Metadata,
    ] {
        builder.add_section(kind, 1, SectionFlags::REQUIRED, vec![kind.tag() as u8]);
    }
    let mut bytes = builder.build().expect("build v2 container");
    bytes[8..12].copy_from_slice(&1_u32.to_le_bytes());

    let error = TeloscContainerView::parse(&bytes, &ValidationLimits::default()).expect_err("v1 header must be rejected");
    assert!(error.to_string().contains("unsupported container schema"));
}

#[test]
fn unknown_optional_section_is_skipped() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m".to_vec());
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());
    builder.add_section(SectionKind::Unknown(999), 1, SectionFlags::empty(), b"future-extension".to_vec());

    let bytes = builder.build().expect("build");
    let view = TeloscContainerView::parse(&bytes, &ValidationLimits::default()).expect("parse");
    assert_eq!(view.get_section(SectionKind::Unknown(999)), Some(&b"future-extension"[..]));
}

#[test]
fn unknown_required_section_is_rejected() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m".to_vec());
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());
    builder.add_section(SectionKind::Unknown(999), 1, SectionFlags::REQUIRED, b"mandatory-extension".to_vec());

    let bytes = builder.build().expect("build");
    let err = TeloscContainerView::parse(&bytes, &ValidationLimits::default()).unwrap_err();
    match err {
        ArtifactError::Container(ContainerError::UnknownRequiredSection { tag: 999 }) => {}
        other => panic!("expected UnknownRequiredSection, got {:?}", other),
    }
}

#[test]
fn duplicate_required_section_is_rejected() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m1".to_vec());
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m2".to_vec());
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());

    let bytes = builder.build().expect("build");
    let err = TeloscContainerView::parse(&bytes, &ValidationLimits::default()).unwrap_err();
    match err {
        ArtifactError::Container(ContainerError::DuplicateSection(SectionKind::ModuleWorld)) => {}
        other => panic!("expected DuplicateSection, got {:?}", other),
    }
}

#[test]
fn overlapping_sections_are_rejected() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m".to_vec());
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());

    let mut bytes = builder.build().expect("build");
    // Tamper with directory entry 1 to overlap entry 0
    // Find directory offset
    let mut reader = BinaryReader::new(&bytes);
    let (_hdr, _) = TeloscHeaderV2::decode(&mut reader, &ValidationLimits::default()).unwrap();
    let dir_pos = reader.position();

    // In directory entry 1: offset is at dir_pos + SECTION_DIRECTORY_ENTRY_SIZE + 12
    let entry1_offset_pos = dir_pos + SECTION_DIRECTORY_ENTRY_SIZE + 12;
    // Overwrite offset to 0
    bytes[entry1_offset_pos..entry1_offset_pos + 8].copy_from_slice(&0u64.to_le_bytes());

    let err = TeloscContainerView::parse(&bytes, &ValidationLimits::default()).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Container(ContainerError::OffsetOverflow { .. }) | ArtifactError::Container(ContainerError::OverlappingSections { .. })
    ));
}

#[test]
fn offset_overflow_is_rejected() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m".to_vec());
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());

    let mut bytes = builder.build().expect("build");
    let mut reader = BinaryReader::new(&bytes);
    let (_hdr, _) = TeloscHeaderV2::decode(&mut reader, &ValidationLimits::default()).unwrap();
    let dir_pos = reader.position();

    // Tamper with entry 0 offset to u64::MAX - 5
    let entry0_offset_pos = dir_pos + 12;
    bytes[entry0_offset_pos..entry0_offset_pos + 8].copy_from_slice(&(u64::MAX - 5).to_le_bytes());

    let err = TeloscContainerView::parse(&bytes, &ValidationLimits::default()).unwrap_err();
    assert!(matches!(
        err,
        ArtifactError::Container(ContainerError::OffsetOverflow { .. }) | ArtifactError::Container(ContainerError::OutOfFileSection { .. })
    ));
}

#[test]
fn section_size_limit_is_enforced() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, vec![0u8; 100]);
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());

    let bytes = builder.build().expect("build");
    let mut tight_limits = ValidationLimits::default();
    tight_limits.max_encoded_section_bytes = 50; // Smaller than 100

    let err = TeloscContainerView::parse(&bytes, &tight_limits).unwrap_err();
    match err {
        ArtifactError::Container(ContainerError::SectionBudgetViolation { section, .. }) => {
            assert_eq!(section, SectionKind::ModuleWorld);
        }
        other => panic!("expected SectionBudgetViolation, got {:?}", other),
    }
}

#[test]
fn section_digest_mismatch_is_rejected() {
    let header = create_sample_header();
    let mut builder = TeloscContainerBuilder::new(header);
    builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m".to_vec());
    builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
    builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
    builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());

    let mut bytes = builder.build().expect("build");
    // Corrupt the very last byte (part of metadata section)
    let last = bytes.len() - 1;
    bytes[last] ^= 0xff;

    let err = TeloscContainerView::parse(&bytes, &ValidationLimits::default()).unwrap_err();
    match err {
        ArtifactError::Container(ContainerError::SectionDigestMismatch { section, .. }) => {
            assert_eq!(section, SectionKind::Metadata);
        }
        other => panic!("expected SectionDigestMismatch, got {:?}", other),
    }
}

#[test]
fn whole_artifact_digest_is_deterministic() {
    let make_bytes = || {
        let header = create_sample_header();
        let mut builder = TeloscContainerBuilder::new(header);
        builder.add_section(SectionKind::ModuleWorld, 1, SectionFlags::REQUIRED, b"m".to_vec());
        builder.add_section(SectionKind::SemanticWorld, 1, SectionFlags::REQUIRED, b"s".to_vec());
        builder.add_section(SectionKind::QueryImage, 1, SectionFlags::REQUIRED, b"q".to_vec());
        builder.add_section(SectionKind::Metadata, 1, SectionFlags::REQUIRED, b"meta".to_vec());
        builder.build().expect("build")
    };

    let bytes1 = make_bytes();
    let bytes2 = make_bytes();
    assert_eq!(bytes1, bytes2);

    let view1 = TeloscContainerView::parse(&bytes1, &ValidationLimits::default()).unwrap();
    let view2 = TeloscContainerView::parse(&bytes2, &ValidationLimits::default()).unwrap();
    assert_eq!(view1.header().artifact_digest, view2.header().artifact_digest);
    assert_ne!(view1.header().artifact_digest, ArtifactDigest::ZERO);
}
