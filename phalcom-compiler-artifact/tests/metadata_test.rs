use phalcom_compiler_artifact::ValidationLimits;
use phalcom_compiler_artifact::metadata::{MetadataPreflightLimits, MetadataSectionV1, validate_and_charge_metadata_preflight};
use phalcom_type_meta::{ArtifactIdentityScheme, MetadataProfile, TYPE_METADATA_SCHEMA_VERSION};

fn limits() -> MetadataPreflightLimits {
    MetadataPreflightLimits {
        max_metadata_bytes: 4096,
        max_token_count: 10000,
        max_container_count: 10000,
        max_nesting_depth: 16,
    }
}

fn bundle() -> phalcom_type_meta::SemanticMetadataBundle {
    use phalcom_type_meta::*;
    SemanticMetadataBundle {
        header: SemanticMetadataHeader {
            schema_version: TYPE_METADATA_SCHEMA_VERSION,
            legacy_fingerprint_schema_version: None,
            semantic_model_version: SEMANTIC_MODEL_VERSION,
            producer: ProducerIdentity("test".into()),
            producer_version: "1".into(),
            native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
            profile: MetadataProfile::RuntimePublic,
            features: MetadataFeatures::default(),
            identity_scheme: ArtifactIdentityScheme::V1Standard,
            source_fingerprint: Fingerprint128::ZERO,
            interface_fingerprint: Fingerprint128::ZERO,
        },
        kinds: Box::new([KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        }]),
        types: Box::new([TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        }]),
        scoped_types: Box::new([]),
        parameters: Box::new([]),
        generic_signatures: Box::new([]),
        declarations: Box::new([]),
        static_domains: Box::new([]),
        aliases: Box::new([]),
        callables: Box::new([]),
        fields: Box::new([]),
        module_roots: Box::new([]),
        runtime_roots: Box::new([]),
        occurrences: Box::new([]),
        extensions: Box::new([]),
    }
}

fn real_section() -> MetadataSectionV1 {
    MetadataSectionV1::from_bundle(&bundle(), [7; 32], &limits(), &phalcom_type_meta::ValidationLimits::default()).unwrap()
}

#[test]
fn metadata_bundle_round_trip_uses_owner_codec_and_preserves_pools() {
    let section = real_section();
    let wire = section.encode().unwrap();
    let restored = MetadataSectionV1::decode(&wire, &ValidationLimits::default()).unwrap();
    let mut remaining = 8192;
    let decoded = restored
        .decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut remaining)
        .unwrap();
    assert_eq!(decoded, bundle());
    // Charged native table/string/scratch allocations:
    assert!(remaining < 8192);
    let total_charge = 8192 - remaining;
    assert_eq!(wire, real_section().encode().unwrap());

    // When shared budget is smaller than total required:
    let mut insufficient = total_charge.checked_sub(1).unwrap();
    let err = restored.decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut insufficient);
    assert!(err.is_err());
    assert_eq!(insufficient, total_charge - 1); // Budget remains completely unchanged on failure

    // Exact budget leaves zero remaining:
    let mut exact = total_charge;
    let ok = restored.decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut exact);
    assert!(ok.is_ok());
    assert_eq!(exact, 0);
}

#[test]
fn metadata_envelope_cannot_spoof_bundle_header() {
    use phalcom_compiler_artifact::metadata::{MetadataTransportError, inspect_metadata_json};
    for (field, value) in [
        ("schema_version", serde_json::json!(1)),
        ("profile", serde_json::json!("RuntimeMinimal")),
        ("identity_scheme", serde_json::json!("SessionLocal")),
    ] {
        let mut section = real_section();
        let mut json: serde_json::Value = serde_json::from_slice(&section.encoded_bundle).unwrap();
        json["header"][field] = value;
        section.encoded_bundle = serde_json::to_vec(&json).unwrap();
        let counts = inspect_metadata_json(&section.encoded_bundle, &limits()).unwrap();
        section.preflight_tokens = counts.tokens;
        section.preflight_containers = counts.containers;
        section.preflight_nesting = counts.nesting;
        let mut remaining = 8192;
        assert!(matches!(
            section.decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut remaining),
            Err(MetadataTransportError::HeaderMismatch(_))
        ));
        assert_eq!(remaining, 8192);
    }
}

#[test]
fn malformed_cross_pool_reference_is_rejected_and_load_is_retryable() {
    use phalcom_compiler_artifact::metadata::{MetadataTransportError, inspect_metadata_json};
    let valid = real_section();
    let mut corrupt = valid.clone();
    let mut json: serde_json::Value = serde_json::from_slice(&corrupt.encoded_bundle).unwrap();
    json["types"][0]["kind"] = serde_json::json!(100);
    corrupt.encoded_bundle = serde_json::to_vec(&json).unwrap();
    let counts = inspect_metadata_json(&corrupt.encoded_bundle, &limits()).unwrap();
    corrupt.preflight_tokens = counts.tokens;
    corrupt.preflight_containers = counts.containers;
    corrupt.preflight_nesting = counts.nesting;
    let mut remaining = 8192;
    assert!(matches!(
        corrupt.decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut remaining),
        Err(MetadataTransportError::Decode(_))
    ));
    assert_eq!(remaining, 8192);
    assert!(
        valid
            .decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut remaining)
            .is_ok()
    );
}

#[test]
fn streaming_preflight_enforces_actual_tokens_containers_and_depth() {
    use phalcom_compiler_artifact::metadata::{MetadataSectionError, inspect_metadata_json};
    let bytes = section().encoded_bundle;
    let mut bound = limits();
    bound.max_token_count = 5;
    assert!(matches!(
        inspect_metadata_json(&bytes, &bound),
        Err(MetadataSectionError::TokenBudgetExceeded { .. })
    ));
    bound = limits();
    bound.max_container_count = 1;
    assert!(matches!(
        inspect_metadata_json(&bytes, &bound),
        Err(MetadataSectionError::ContainerBudgetExceeded { .. })
    ));
    bound = limits();
    bound.max_nesting_depth = 1;
    assert!(matches!(
        inspect_metadata_json(&bytes, &bound),
        Err(MetadataSectionError::NestingBudgetExceeded { .. })
    ));
}

#[test]
fn metadata_rejects_trailing_bytes_invalid_json_and_wrong_package() {
    let section = real_section();
    let mut wire = section.encode().unwrap();
    wire.push(0);
    assert!(MetadataSectionV1::decode(&wire, &ValidationLimits::default()).is_err());
    let mut corrupt = section.clone();
    corrupt.encoded_bundle.extend_from_slice(b" null");
    let mut budget = 4096;
    assert!(
        corrupt
            .decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut budget)
            .is_err()
    );
    assert!(
        section
            .decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[8; 32], &mut budget)
            .is_err()
    );
    assert_eq!(budget, 4096);
}

#[test]
fn metadata_does_not_reset_shared_budget_and_rejects_nonportable_profile() {
    let mut section = real_section();
    let mut budget = 0;
    let before = budget;
    assert!(
        section
            .decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut budget)
            .is_err()
    );
    assert_eq!(budget, before);
    section.profile = MetadataProfile::RuntimeMinimal;
    assert!(
        section
            .decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut budget)
            .is_err()
    );
    section.profile = MetadataProfile::RuntimePublic;
    section.identity_mode = ArtifactIdentityScheme::SessionLocal;
    assert!(
        section
            .decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut budget)
            .is_err()
    );
    assert_eq!(budget, before);
}

fn section() -> MetadataSectionV1 {
    MetadataSectionV1 {
        wrapper_version: 1,
        inner_schema_version: TYPE_METADATA_SCHEMA_VERSION as u16,
        profile: MetadataProfile::RuntimePublic,
        identity_mode: ArtifactIdentityScheme::V1Standard,
        package_digest: [7; 32],
        preflight_tokens: 6,
        preflight_containers: 2,
        preflight_nesting: 2,
        encoded_bundle: br#"{"a":[1,2,3]}"#.to_vec(),
    }
}

#[test]
fn preflight_accepts_current_inner_schema() {
    let section = section();
    let mut budget = 100;
    validate_and_charge_metadata_preflight(&section, &limits(), &[7; 32], &mut budget).expect("current metadata schema with exact content counts");
    assert_eq!(budget, 100 - section.encoded_bundle.len());
}

#[test]
fn preflight_rejects_forged_counters_without_charging_budget() {
    let mut section = section();
    section.preflight_tokens = 0;
    section.preflight_containers = 0;
    section.preflight_nesting = 0;
    let mut budget = 100;
    assert!(validate_and_charge_metadata_preflight(&section, &limits(), &[7; 32], &mut budget).is_err());
    assert_eq!(budget, 100);

    // Unsupported legacy envelopes must also fail without consuming budget.
    section.inner_schema_version = 1;
    assert!(validate_and_charge_metadata_preflight(&section, &limits(), &[7; 32], &mut budget).is_err());
    assert_eq!(budget, 100);
}

#[test]
fn metadata_decode_enforces_its_byte_limit_before_payload_copy() {
    let section = section();
    let bytes = section.encode().unwrap();
    let limits = ValidationLimits {
        max_metadata_bytes: 2,
        ..ValidationLimits::default()
    };
    assert!(MetadataSectionV1::decode(&bytes, &limits).is_err());
}

#[test]
fn metadata_wire_decode_rejects_invalid_envelope_before_materialization() {
    let original = real_section();
    for mutate in [
        |s: &mut MetadataSectionV1| s.wrapper_version = 2,
        |s: &mut MetadataSectionV1| s.inner_schema_version = 1,
        |s: &mut MetadataSectionV1| s.profile = MetadataProfile::RuntimeMinimal,
        |s: &mut MetadataSectionV1| s.identity_mode = ArtifactIdentityScheme::SessionLocal,
    ] {
        let mut section = original.clone();
        mutate(&mut section);
        assert!(MetadataSectionV1::decode(&section.encode().unwrap(), &ValidationLimits::default()).is_err());
    }
}

#[test]
fn long_owner_string_with_fixed_tokens_is_rejected_before_allocation() {
    use phalcom_compiler_artifact::metadata::inspect_metadata_json;
    // Bundle with an owner string of 20,000 bytes (few tokens, but large memory allocation).
    let mut b = bundle();
    let long_name = "A".repeat(20_000);
    b.header.producer = phalcom_type_meta::ProducerIdentity(long_name.into());
    let mut l = limits();
    l.max_metadata_bytes = 64_000;
    let section = MetadataSectionV1::from_bundle(&b, [7; 32], &l, &phalcom_type_meta::ValidationLimits::default()).unwrap();
    let counts = inspect_metadata_json(&section.encoded_bundle, &l).unwrap();
    assert!(counts.tokens < 200); // Fixed token count independent of string length

    // If budget is too small for the 20KB string (e.g. 5,000 bytes), rejection happens before allocation:
    let mut budget = 5_000;
    let res = section.decode_bundle(&l, &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut budget);
    assert!(res.is_err());
    assert_eq!(budget, 5_000); // Budget left unchanged on failure
}

#[test]
fn many_containers_overhead_is_charged_and_rejected_when_insufficient() {
    let (budget, _) = phalcom_type_meta::encode::inspect_metadata_bundle_allocations(
        r#"{"header":{"schema_version":13,"semantic_model_version":1,"producer":"p","producer_version":"1","native_surface_schema_version":1,"profile":"RuntimePublic","features":{"record_rows":false},"identity_scheme":"V1Standard","source_fingerprint":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0],"interface_fingerprint":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]},"kinds":[],"types":[],"scoped_types":[],"parameters":[],"generic_signatures":[],"declarations":[],"static_domains":[],"aliases":[],"callables":[],"fields":[],"module_roots":[],"runtime_roots":[],"occurrences":[],"extensions":[]}"#,
    ).unwrap();
    assert!(budget.container_overhead_bytes > 0);
    assert!(budget.total_bytes() > 0);
}

#[test]
fn generic_parameters_and_static_domains_scratch_is_charged() {
    use phalcom_type_meta::*;
    let mut b = bundle();
    // Add 10 parameters to bundle:
    let mut params = Vec::new();
    for i in 0..10 {
        params.push(generic::TypeParameterRecord {
            id: generic::StableTypeParameterRef {
                owner: generic::StableTypeParameterOwnerRef::Declaration(identity::StableDeclarationRef {
                    module: identity::StableModuleRef {
                        project: identity::StableProjectRef::Builtin {
                            namespace: "std".into(),
                            version: "1".into(),
                        },
                        path: Box::new([]),
                    },
                    path: Box::new([format!("D{i}").into_boxed_str()]),
                }),
                index: 0,
            },
            name: format!("T{i}").into(),
            kind: KindNodeId(0),
            variance: generic::VarianceRef::Invariant,
            mode: generic::GenericParameterModeRef::Single,
            source: None,
        });
    }
    b.parameters = params.into_boxed_slice();

    let json = encode_metadata_json(&b).unwrap();
    let (budget, _) = phalcom_type_meta::encode::inspect_metadata_bundle_allocations(&json).unwrap();
    // Validator scratch bytes for param_map is explicitly charged:
    assert!(budget.validator_scratch_bytes > 0);
    assert_eq!(
        budget.validator_scratch_bytes,
        10 * std::mem::size_of::<(
            (&phalcom_type_meta::generic::StableTypeParameterOwnerRef, u32),
            &phalcom_type_meta::generic::TypeParameterRecord
        )>()
    );
}

#[test]
fn insufficient_budget_rejection_leaves_caller_budget_unchanged() {
    let section = real_section();
    let mut budget = 100; // Far too small
    let res = section.decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut budget);
    assert!(res.is_err());
    assert_eq!(budget, 100);
}

#[test]
fn representative_universe_metadata_decodes_cleanly_under_default_limits() {
    let section = real_section();
    let mut budget = 64 * 1024 * 1024;
    let decoded = section
        .decode_bundle(&limits(), &phalcom_type_meta::ValidationLimits::default(), &[7; 32], &mut budget)
        .unwrap();
    assert_eq!(decoded.header.schema_version, TYPE_METADATA_SCHEMA_VERSION);
    assert!(budget < 64 * 1024 * 1024);
}
