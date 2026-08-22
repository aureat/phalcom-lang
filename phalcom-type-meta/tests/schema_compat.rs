use phalcom_type_meta::bundle::SemanticMetadataBundle;
use phalcom_type_meta::encode::decode_metadata_json;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{StableTypeParameterOwnerRef, StableTypeParameterRef, TypeParameterRecord, VarianceRef};
use phalcom_type_meta::header::{
    ArtifactIdentityScheme, MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity,
    SEMANTIC_MODEL_VERSION, SemanticMetadataHeader, TYPE_METADATA_SCHEMA_VERSION,
};
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef};
use phalcom_type_meta::kind::{KindNode, KindNodeEntry, KindNodeId};
use phalcom_type_meta::scoped_type::{ScopedOpenRecordTypeRef, ScopedRecordTailRef, ScopedTypeNode, ScopedTypeNodeEntry, ScopedTypeNodeId};
use phalcom_type_meta::type_node::{OpenRecordTypeRef, RecordFieldRef, TypeNode, TypeNodeEntry, TypeNodeId};
use phalcom_type_meta::validate::{MetadataValidationError, ValidationLimits, validate_metadata_bundle};

fn sample_header(version: u32, record_rows: bool) -> SemanticMetadataHeader {
    SemanticMetadataHeader {
        schema_version: version,
        semantic_model_version: SEMANTIC_MODEL_VERSION,
        producer: ProducerIdentity("phalcom-test".into()),
        producer_version: "0.1.0".into(),
        native_surface_schema_version: NATIVE_SURFACE_SCHEMA_VERSION,
        profile: MetadataProfile::RuntimePublic,
        features: MetadataFeatures {
            type_lambdas: true,
            record_rows,
            runtime_type_constants: false,
            source_occurrences: false,
            advanced_sections: Box::new([]),
        },
        identity_scheme: ArtifactIdentityScheme::V1Standard,
        source_fingerprint: Fingerprint128::ZERO,
        interface_fingerprint: Fingerprint128::ZERO,
    }
}

fn sample_bundle(version: u32, record_rows: bool) -> SemanticMetadataBundle {
    SemanticMetadataBundle {
        header: sample_header(version, record_rows),
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
        aliases: Box::new([]),
        callables: Box::new([]),
        fields: Box::new([]),
        module_roots: Box::new([]),
        runtime_roots: Box::new([]),
        occurrences: Box::new([]),
        extensions: Box::new([]),
    }
}

#[test]
fn test_v1_fixture_decodes_and_validates_under_v2_decoder() {
    let fixture_str = include_str!("fixtures/schema-v1/basic.json");
    let limits = ValidationLimits::default();
    let bundle = decode_metadata_json(fixture_str, &limits).expect("failed to decode v1 fixture");
    assert_eq!(bundle.header.schema_version, 1);
    assert_eq!(bundle.types.len(), 2);
    assert!(matches!(bundle.types[1].form, TypeNode::Record(_)));
}

#[test]
fn test_unsupported_schema_versions_rejected() {
    let limits = ValidationLimits::default();

    // Version 0
    let b0 = sample_bundle(0, false);
    let err0 = validate_metadata_bundle(&b0, &limits).unwrap_err();
    assert_eq!(
        err0,
        MetadataValidationError::UnsupportedSchemaVersion {
            found: 0,
            minimum: MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION,
            maximum: TYPE_METADATA_SCHEMA_VERSION,
        }
    );

    // Version 3 (future)
    let b3 = sample_bundle(3, false);
    let err3 = validate_metadata_bundle(&b3, &limits).unwrap_err();
    assert_eq!(
        err3,
        MetadataValidationError::UnsupportedSchemaVersion {
            found: 3,
            minimum: MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION,
            maximum: TYPE_METADATA_SCHEMA_VERSION,
        }
    );
}

#[test]
fn test_schema_v1_cannot_contain_record_row_constructs() {
    let limits = ValidationLimits::default();

    // v1 with record_rows feature bit = true -> error
    let b_feat = sample_bundle(1, true);
    assert!(validate_metadata_bundle(&b_feat, &limits).is_err());

    // v1 with KindNode::RecordRow -> error
    let mut b_kind = sample_bundle(1, false);
    b_kind.kinds = Box::new([KindNodeEntry {
        node: KindNode::RecordRow,
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    assert!(validate_metadata_bundle(&b_kind, &limits).is_err());
}

#[test]
fn test_schema_v2_open_record_validation() {
    let limits = ValidationLimits::default();
    let mut b = sample_bundle(2, true);

    b.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    let decl_ref = StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "test".into(),
                version: "0.1.0".into(),
            },
            path: Box::new(["mod".into()]),
        },
        path: Box::new(["MyType".into()]),
    };

    let param_ref = StableTypeParameterRef {
        owner: StableTypeParameterOwnerRef::Declaration(decl_ref.clone()),
        index: 0,
    };

    b.parameters = Box::new([TypeParameterRecord {
        id: param_ref.clone(),
        name: "R".into(),
        kind: KindNodeId(1), // RecordRow
        variance: VarianceRef::Invariant,
        source: None,
    }]);

    // Valid open record
    b.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::OpenRecord(OpenRecordTypeRef {
                fields: Box::new([
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                    },
                    RecordFieldRef {
                        name: "b".into(),
                        ty: TypeNodeId(0),
                    },
                ]),
                tail: param_ref.clone(),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    validate_metadata_bundle(&b, &limits).expect("v2 open record should validate");

    // Negative test: unsorted fields
    let mut b_unsorted = b.clone();
    b_unsorted.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::OpenRecord(OpenRecordTypeRef {
                fields: Box::new([
                    RecordFieldRef {
                        name: "z".into(),
                        ty: TypeNodeId(0),
                    },
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                    },
                ]),
                tail: param_ref.clone(),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    assert!(matches!(
        validate_metadata_bundle(&b_unsorted, &limits).unwrap_err(),
        MetadataValidationError::RecordFieldOrderInvalid { .. }
    ));

    // Negative test: duplicate fields
    let mut b_dup = b.clone();
    b_dup.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::OpenRecord(OpenRecordTypeRef {
                fields: Box::new([
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                    },
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                    },
                ]),
                tail: param_ref.clone(),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    assert!(matches!(
        validate_metadata_bundle(&b_dup, &limits).unwrap_err(),
        MetadataValidationError::RecordDuplicateField { .. }
    ));

    // Negative test: tail parameter is not kind RecordRow (point it to Type kind 0)
    let mut b_kind_mismatch = b.clone();
    b_kind_mismatch.parameters = Box::new([TypeParameterRecord {
        id: param_ref.clone(),
        name: "T".into(),
        kind: KindNodeId(0), // Type instead of RecordRow
        variance: VarianceRef::Invariant,
        source: None,
    }]);
    assert!(matches!(
        validate_metadata_bundle(&b_kind_mismatch, &limits).unwrap_err(),
        MetadataValidationError::RecordTailKindMismatch { .. }
    ));

    // Negative test: missing record_rows feature
    let mut b_no_feat = b.clone();
    b_no_feat.header.features.record_rows = false;
    assert_eq!(
        validate_metadata_bundle(&b_no_feat, &limits).unwrap_err(),
        MetadataValidationError::RecordRowFeatureRequired
    );
}

#[test]
fn test_scoped_open_record_validation() {
    let limits = ValidationLimits::default();
    let mut b = sample_bundle(2, true);

    b.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    // Scoped lambda with row binder at depth 0, index 0
    b.scoped_types = Box::new([
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::OpenRecord(ScopedOpenRecordTypeRef {
                fields: Box::new([]),
                tail: ScopedRecordTailRef::Bound { depth: 0, index: 0 },
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::Lambda {
                parameter_kinds: Box::new([KindNodeId(1)]), // RecordRow binder
                body: ScopedTypeNodeId(0),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    validate_metadata_bundle(&b, &limits).expect("scoped open record with row binder should validate");
}
