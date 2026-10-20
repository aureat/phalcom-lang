use phalcom_compiler_artifact::digest::ArtifactDigest;
use phalcom_compiler_artifact::header::{MODULE_SEMANTIC_ABI_V2, QUERY_FINGERPRINT_ABI_V3, SEMANTIC_ARTIFACT_ABI_V3, STABLE_IDENTITY_SCHEMA_V1};
use phalcom_compiler_artifact::identity::{CanonicalCompatibilityIdentity, TeloscPackageIdentity};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{GenericParameterModeRef, StableTypeParameterOwnerRef, StableTypeParameterRef, TypeParameterRecord, VarianceRef};
use phalcom_type_meta::identity::{
    SourceSpanRef, StableCallableOwnerRef, StableCallableRef, StableDeclarationRef, StableDispatchSide, StableFieldRef, StableModuleRef, StableProjectRef,
    StableTypeMemberRef, StableVariantFieldRef, StableVariantRef,
};
use phalcom_type_meta::kind::KindNodeId;

#[test]
fn builtin_universe_module_identity_round_trips_deterministically() {
    let builtin_project = StableProjectRef::Builtin {
        namespace: "universe".into(),
        version: "0.1.0".into(),
    };
    let pkg_id = TeloscPackageIdentity::try_from(&builtin_project).expect("convert builtin project");
    assert_eq!(pkg_id, TeloscPackageIdentity::universe());
    assert!(pkg_id.is_universe());
    assert!(pkg_id.is_exact_universe());

    let universe_module = StableModuleRef {
        project: builtin_project.clone(),
        path: Box::new(["core".into(), "kernel".into()]),
    };

    let serialized = serde_json::to_string(&universe_module).expect("serialize universe module");
    let deserialized: StableModuleRef = serde_json::from_str(&serialized).expect("deserialize universe module");

    assert_eq!(universe_module, deserialized);
    assert_eq!(deserialized.project, builtin_project);
    let expected_path: Box<[Box<str>]> = vec!["core".into(), "kernel".into()].into_boxed_slice();
    assert_eq!(deserialized.path, expected_path);
}

#[test]
fn ordinary_project_module_identity_survives_process_separation() {
    let project_fp = Fingerprint128::from_u128(0x1234_5678_9abc_def0_cafe_babe_feed_face);
    let package_project = StableProjectRef::Package {
        package: "slotmap".into(),
        version: "1.0.0".into(),
        artifact_fingerprint: project_fp,
    };
    let pkg_id = TeloscPackageIdentity::try_from(&package_project).expect("convert package project");
    assert_eq!(
        pkg_id,
        TeloscPackageIdentity::Package {
            package: "slotmap".into(),
            version: "1.0.0".into(),
        }
    );
    assert!(!pkg_id.is_universe());

    let package_module = StableModuleRef {
        project: package_project.clone(),
        path: Box::new(["container".into(), "sparse".into()]),
    };

    let serialized = serde_json::to_string(&package_module).expect("serialize package module");
    let deserialized: StableModuleRef = serde_json::from_str(&serialized).expect("deserialize package module");

    assert_eq!(package_module, deserialized);
}

#[test]
fn declaration_identity_and_synthetic_main_classification() {
    let module = StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: Box::new(["core".into(), "string".into()]),
    };

    let nominal_decl = StableDeclarationRef {
        module: module.clone(),
        path: Box::new(["String".into()]),
    };
    assert!(!nominal_decl.is_synthetic_main());

    let synthetic_decl = StableDeclarationRef {
        module: module.clone(),
        path: Box::new(["<main>".into()]),
    };
    assert!(synthetic_decl.is_synthetic_main());

    let serialized = serde_json::to_string(&nominal_decl).expect("serialize nominal decl");
    let deserialized: StableDeclarationRef = serde_json::from_str(&serialized).expect("deserialize nominal decl");
    assert_eq!(nominal_decl, deserialized);
}

#[test]
fn member_and_variant_identities_round_trip_without_ephemeral_ids() {
    let decl = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: Box::new(["core".into(), "option".into()]),
        },
        path: Box::new(["Option".into()]),
    };

    // Variant identity
    let variant = StableVariantRef {
        owner: decl.clone(),
        selector: "Some(_) = 1".into(),
    };
    let variant_field = StableVariantFieldRef {
        variant: variant.clone(),
        index: 0,
    };

    let serialized_vf = serde_json::to_string(&variant_field).expect("serialize variant field");
    let deserialized_vf: StableVariantFieldRef = serde_json::from_str(&serialized_vf).expect("deserialize variant field");
    assert_eq!(variant_field, deserialized_vf);

    // Callable identity (Declaration owner)
    let method_callable = StableCallableRef {
        owner: StableCallableOwnerRef::Declaration(decl.clone()),
        side: StableDispatchSide::Instance,
        selector: "map(_) = 1".into(),
    };
    let serialized_c = serde_json::to_string(&method_callable).expect("serialize callable");
    let deserialized_c: StableCallableRef = serde_json::from_str(&serialized_c).expect("deserialize callable");
    assert_eq!(method_callable, deserialized_c);
    assert_eq!(deserialized_c.owner.try_declaration(), Some(&decl));
    assert_eq!(deserialized_c.owner.enclosing_declaration(), Some(&decl));

    // Field identity
    let field = StableFieldRef {
        owner: decl.clone(),
        side: StableDispatchSide::Instance,
        name: "value".into(),
    };
    let serialized_f = serde_json::to_string(&field).expect("serialize field");
    let deserialized_f: StableFieldRef = serde_json::from_str(&serialized_f).expect("deserialize field");
    assert_eq!(field, deserialized_f);

    // Type member identity
    let type_member = StableTypeMemberRef {
        owner: decl.clone(),
        selector: "Item".into(),
    };
    let serialized_tm = serde_json::to_string(&type_member).expect("serialize type member");
    let deserialized_tm: StableTypeMemberRef = serde_json::from_str(&serialized_tm).expect("deserialize type member");
    assert_eq!(type_member, deserialized_tm);
}

#[test]
fn static_and_generic_argument_identities_round_trip() {
    let decl = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "universe".into(),
                version: "0.1.0".into(),
            },
            path: Box::new(["collections".into(), "list".into()]),
        },
        path: Box::new(["List".into()]),
    };

    let param_owner = StableTypeParameterOwnerRef::Declaration(decl);
    let param_id = StableTypeParameterRef { owner: param_owner, index: 0 };

    let param_record = TypeParameterRecord {
        id: param_id.clone(),
        name: "Element".into(),
        kind: KindNodeId(0),
        variance: VarianceRef::Covariant,
        mode: GenericParameterModeRef::Single,
        source: Some(SourceSpanRef { start: 10, end: 17 }),
    };

    let serialized = serde_json::to_string(&param_record).expect("serialize param record");
    let deserialized: TypeParameterRecord = serde_json::from_str(&serialized).expect("deserialize param record");
    assert_eq!(param_record, deserialized);
    assert_eq!(deserialized.id, param_id);
    assert_eq!(deserialized.variance, VarianceRef::Covariant);
}

#[test]
fn compatibility_identity_structural_equivalence() {
    let compat1 = CanonicalCompatibilityIdentity {
        package: TeloscPackageIdentity::universe(),
        source_digest: ArtifactDigest([1; 32]),
        interface_digest: ArtifactDigest([2; 32]),
        semantic_digest: ArtifactDigest([3; 32]),
        semantic_model_version: 1,
        stable_identity_schema: STABLE_IDENTITY_SCHEMA_V1,
        module_semantic_abi: MODULE_SEMANTIC_ABI_V2,
        semantic_artifact_abi: SEMANTIC_ARTIFACT_ABI_V3,
        query_fingerprint_abi: QUERY_FINGERPRINT_ABI_V3,
        native_surface_schema_version: 1,
        native_catalog_fingerprint: [9; 16],
        type_metadata_schema_version: 13,
    };
    let compat2 = compat1.clone();
    assert!(compat1.check_compatible(&compat2).is_ok());

    let serialized = serde_json::to_string(&compat1).expect("serialize compat identity");
    let deserialized: CanonicalCompatibilityIdentity = serde_json::from_str(&serialized).expect("deserialize compat identity");
    assert_eq!(compat1, deserialized);
    assert!(compat1.check_compatible(&deserialized).is_ok());
}

#[test]
fn session_and_source_artifact_projects_fail_conversion_to_package_identity() {
    let session = StableProjectRef::Session {
        session_fingerprint: Fingerprint128::from_u128(42),
    };
    assert!(TeloscPackageIdentity::try_from(&session).is_err());

    let source_artifact = StableProjectRef::SourceArtifact {
        logical_uri: "pkg:test".into(),
        source_fingerprint: Fingerprint128::from_u128(43),
    };
    assert!(TeloscPackageIdentity::try_from(&source_artifact).is_err());
}
