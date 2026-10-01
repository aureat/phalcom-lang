use phalcom_type_meta::bundle::SemanticMetadataBundle;
use phalcom_type_meta::declaration::{StaticDomainCaseRecord, StaticDomainDeclarationRecord, StaticDomainParameterRecord};
use phalcom_type_meta::encode::{decode_metadata_json, encode_metadata_json};
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::generic::{GenericParameterModeRef, StableTypeParameterOwnerRef, StableTypeParameterRef, TypeParameterRecord, VarianceRef};
use phalcom_type_meta::header::{
    ArtifactIdentityScheme, MIN_SUPPORTED_TYPE_METADATA_SCHEMA_VERSION, MetadataFeatures, MetadataProfile, NATIVE_SURFACE_SCHEMA_VERSION, ProducerIdentity,
    SEMANTIC_MODEL_VERSION, SemanticMetadataHeader, TYPE_METADATA_SCHEMA_VERSION,
};
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef, StableStaticDomainDeclarationRef};
use phalcom_type_meta::kind::{KindNode, KindNodeEntry, KindNodeId, StaticDomainRef};
use phalcom_type_meta::scoped_type::{
    ScopedOpenRecordTypeRef, ScopedRecordFieldRef, ScopedRecordRowTailRef, ScopedRecordRowTypeRef, ScopedRecordTailRef, ScopedTypeNode, ScopedTypeNodeEntry,
    ScopedTypeNodeId,
};
use phalcom_type_meta::type_node::{
    OpenRecordTypeRef, RecordFieldRef, StaticArgumentRef, StaticConstructorRef, StaticDataRef,
    StructuralMemberPresenceRef, TypeMemberProjectionRef, TypeMemberSelectorRef, TypeNode,
    TypeNodeEntry, TypeNodeId,
};
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
            static_domains: false,
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

fn scoped_record_row_bundle(version: u32) -> SemanticMetadataBundle {
    let declaration = schema_v8_static_declaration("RowOwner");
    let row_parameter = StableTypeParameterRef {
        owner: StableTypeParameterOwnerRef::Declaration(declaration),
        index: 0,
    };
    let mut bundle = sample_bundle(version, true);
    bundle.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::Arrow {
                parameters: Box::new([KindNodeId(0), KindNodeId(1)]),
                result: KindNodeId(1),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::Arrow {
                parameters: Box::new([KindNodeId(1)]),
                result: KindNodeId(1),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle.parameters = Box::new([TypeParameterRecord {
        id: row_parameter.clone(),
        name: "Fs".into(),
        kind: KindNodeId(1),
        variance: VarianceRef::Invariant,
        mode: GenericParameterModeRef::LabeledPack,
        source: None,
    }]);
    bundle.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(1),
            form: TypeNode::TypeMemberProjection(TypeMemberProjectionRef {
                subject: TypeNodeId(0),
                selector: TypeMemberSelectorRef::Named("Fields".into()),
                arguments: Box::new([]),
                source_declaration: None,
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle.scoped_types = Box::new([
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::Free(TypeNodeId(0)),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(1),
            form: ScopedTypeNode::RecordRow(ScopedRecordRowTypeRef {
                fields: Box::new([ScopedRecordFieldRef {
                    name: "payload".into(),
                    ty: ScopedTypeNodeId(0),
                    presence: StructuralMemberPresenceRef::Required,
                }]),
                tail: ScopedRecordRowTailRef::Closed,
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(2),
            form: ScopedTypeNode::Lambda {
                parameter_kinds: Box::new([KindNodeId(0), KindNodeId(1)]),
                parameter_modes: None,
                body: ScopedTypeNodeId(1),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(1),
            form: ScopedTypeNode::RecordRow(ScopedRecordRowTypeRef {
                fields: Box::new([]),
                tail: ScopedRecordRowTailRef::Bound { depth: 0, index: 1 },
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(1),
            form: ScopedTypeNode::RecordRow(ScopedRecordRowTypeRef {
                fields: Box::new([]),
                tail: ScopedRecordRowTailRef::FreeParameter(row_parameter),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(2),
            form: ScopedTypeNode::Lambda {
                parameter_kinds: Box::new([KindNodeId(0), KindNodeId(1)]),
                parameter_modes: None,
                body: ScopedTypeNodeId(3),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(1),
            form: ScopedTypeNode::RecordRow(ScopedRecordRowTypeRef {
                fields: Box::new([]),
                tail: ScopedRecordRowTailRef::Projection(TypeNodeId(1)),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle
}

#[test]
fn schema_v9_scoped_record_rows_round_trip_all_tail_forms_and_v8_remains_compatible() {
    let limits = ValidationLimits::default();
    let mut v8 = sample_bundle(8, true);
    v8.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::Arrow {
                parameters: Box::new([KindNodeId(1)]),
                result: KindNodeId(0),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    v8.scoped_types = Box::new([
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::OpenRecord(ScopedOpenRecordTypeRef {
                fields: Box::new([]),
                tail: ScopedRecordTailRef::Bound { depth: 0, index: 0 },
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(2),
            form: ScopedTypeNode::Lambda {
                parameter_kinds: Box::new([KindNodeId(1)]),
                parameter_modes: None,
                body: ScopedTypeNodeId(0),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    validate_metadata_bundle(&v8, &limits).expect("legacy schema-v8 scoped open-row bundle validates");
    let encoded_v8 = encode_metadata_json(&v8).unwrap();
    assert_eq!(decode_metadata_json(&encoded_v8, &limits).unwrap(), v8);

    let bundle = scoped_record_row_bundle(9);
    validate_metadata_bundle(&bundle, &limits).expect("v9 scoped row args should validate");
    let encoded = encode_metadata_json(&bundle).unwrap();
    assert_eq!(decode_metadata_json(&encoded, &limits).unwrap(), bundle);

    let mut old_schema = bundle.clone();
    old_schema.header.schema_version = 8;
    assert!(matches!(
        validate_metadata_bundle(&old_schema, &limits),
        Err(MetadataValidationError::Malformed(message)) if message.contains("schema before v9")
    ));

    let mut wrong_kind = bundle.clone();
    wrong_kind.scoped_types[3].kind = KindNodeId(0);
    assert!(
        matches!(validate_metadata_bundle(&wrong_kind, &limits), Err(MetadataValidationError::Malformed(message)) if message.contains("must have kind RecordRow"))
    );

    let mut wrong_field_kind = bundle.clone();
    wrong_field_kind.scoped_types[0].kind = KindNodeId(1);
    assert!(matches!(
        validate_metadata_bundle(&wrong_field_kind, &limits),
        Err(MetadataValidationError::Malformed(message)) if message.contains("field 'payload'") && message.contains("must have kind Type")
    ));

    let mut duplicate = bundle.clone();
    let ScopedTypeNode::RecordRow(row) = &mut duplicate.scoped_types[1].form else {
        unreachable!()
    };
    row.fields = Box::new([
        ScopedRecordFieldRef {
            name: "payload".into(),
            ty: ScopedTypeNodeId(0),
            presence: StructuralMemberPresenceRef::Required,
        },
        ScopedRecordFieldRef {
            name: "payload".into(),
            ty: ScopedTypeNodeId(0),
            presence: StructuralMemberPresenceRef::Required,
        },
    ]);
    assert!(matches!(
        validate_metadata_bundle(&duplicate, &limits),
        Err(MetadataValidationError::RecordDuplicateField { .. })
    ));

    let mut bad_projection = bundle;
    let ScopedTypeNode::RecordRow(row) = &mut bad_projection.scoped_types[4].form else {
        unreachable!()
    };
    row.tail = ScopedRecordRowTailRef::Projection(TypeNodeId(0));
    assert!(
        matches!(validate_metadata_bundle(&bad_projection, &limits), Err(MetadataValidationError::Malformed(message)) if message.contains("projection tail") && message.contains("RecordRow"))
    );
}

fn schema_v8_static_declaration(name: &str) -> StableDeclarationRef {
    StableDeclarationRef {
        module: StableModuleRef {
            project: StableProjectRef::Builtin {
                namespace: "schema-v8".into(),
                version: "1".into(),
            },
            path: Box::new(["types".into()]),
        },
        path: Box::new([name.into()]),
    }
}

fn schema_v8_nominal_domain_bundle(version: u32) -> (SemanticMetadataBundle, StableDeclarationRef) {
    let declaration = schema_v8_static_declaration("Shape");
    let mut bundle = sample_bundle(version, false);
    bundle.header.features.static_domains = true;
    bundle.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::StaticDomain(StaticDomainRef::Nominal {
                declaration: declaration.clone(),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    (bundle, declaration)
}

fn schema_v8_domain_record(
    declaration: &StableDeclarationRef,
    cases: Box<[StaticDomainCaseRecord]>,
) -> StaticDomainDeclarationRecord {
    StaticDomainDeclarationRecord {
        declaration: StableStaticDomainDeclarationRef::Nominal(declaration.clone()),
        owner_parameters: Box::new([]),
        cases,
    }
}

fn schema_v8_value_data_bundle() -> (SemanticMetadataBundle, StableDeclarationRef) {
    let (mut bundle, declaration) = schema_v8_nominal_domain_bundle(8);
    bundle.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(1),
            form: TypeNode::StaticData(StaticDataRef {
                domain: StaticDomainRef::Nominal {
                    declaration: declaration.clone(),
                },
                constructor: StaticConstructorRef {
                    declaration: declaration.clone(),
                    case_name: "Value".into(),
                },
                arguments: Box::new([StaticArgumentRef::Term(TypeNodeId(0))]),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle.static_domains = Box::new([schema_v8_domain_record(
        &declaration,
        Box::new([StaticDomainCaseRecord {
            name: "Value".into(),
            parameter_kinds: Box::new([KindNodeId(0)]),
        }]),
    )]);
    (bundle, declaration)
}

#[test]
fn schema_v8_requires_a_catalog_for_nominal_and_associated_static_domain_kinds() {
    let limits = ValidationLimits::default();
    let (nominal, declaration) = schema_v8_nominal_domain_bundle(8);
    assert_eq!(
        validate_metadata_bundle(&nominal, &limits).unwrap_err(),
        MetadataValidationError::MissingStaticDomainCatalogEntry(
            StableStaticDomainDeclarationRef::Nominal(declaration)
        )
    );

    let mut associated = sample_bundle(8, false);
    associated.header.features.static_domains = true;
    let owner = schema_v8_static_declaration("Container");
    let associated_declaration = phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef {
        owner,
        name: "State".into(),
    };
    associated.kinds = Box::new([KindNodeEntry {
        node: KindNode::StaticDomain(StaticDomainRef::Associated {
            declaration: associated_declaration.clone(),
            owner_environment: Box::new([]),
        }),
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    assert_eq!(
        validate_metadata_bundle(&associated, &limits).unwrap_err(),
        MetadataValidationError::MissingStaticDomainCatalogEntry(
            StableStaticDomainDeclarationRef::Associated(associated_declaration)
        )
    );
}

#[test]
fn older_projection_metadata_without_source_identity_remains_compatible() {
    use phalcom_type_meta::encode::decode_metadata_json;

    let mut bundle = sample_bundle(8, false);
    bundle.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::TypeMemberProjection(TypeMemberProjectionRef {
                subject: TypeNodeId(0),
                selector: TypeMemberSelectorRef::Named("Fields".into()),
                arguments: Box::new([]),
                source_declaration: None,
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    let mut wire = serde_json::to_value(&bundle).expect("schema-v8 bundle serializes");
    fn remove_projection_source_identity(value: &mut serde_json::Value) -> bool {
        match value {
            serde_json::Value::Object(object) => {
                if let Some(serde_json::Value::Object(projection)) = object.get_mut("TypeMemberProjection") {
                    return projection.remove("source_declaration").is_some();
                }
                object.values_mut().any(remove_projection_source_identity)
            }
            serde_json::Value::Array(items) => items.iter_mut().any(remove_projection_source_identity),
            _ => false,
        }
    }
    assert!(
        remove_projection_source_identity(&mut wire),
        "projection field should be present in current serialization"
    );

    let decoded = decode_metadata_json(&wire.to_string(), &ValidationLimits::default()).expect("legacy source-less projection remains valid metadata");
    let TypeNode::TypeMemberProjection(projection) = &decoded.types[1].form else {
        panic!("expected decoded type-member projection");
    };
    assert!(projection.source_declaration.is_none());
}

#[test]
fn schema_v8_rejects_duplicate_declarations_and_duplicate_or_unsorted_cases() {
    let limits = ValidationLimits::default();
    let (mut bundle, declaration) = schema_v8_nominal_domain_bundle(8);
    let record = schema_v8_domain_record(&declaration, Box::new([]));
    bundle.static_domains = Box::new([record.clone(), record]);
    assert_eq!(
        validate_metadata_bundle(&bundle, &limits).unwrap_err(),
        MetadataValidationError::DuplicateStaticDomainCatalogEntry(
            StableStaticDomainDeclarationRef::Nominal(declaration.clone())
        )
    );

    let (mut duplicate_cases, declaration) = schema_v8_nominal_domain_bundle(8);
    duplicate_cases.static_domains = Box::new([schema_v8_domain_record(
        &declaration,
        Box::new([
            StaticDomainCaseRecord {
                name: "A".into(),
                parameter_kinds: Box::new([]),
            },
            StaticDomainCaseRecord {
                name: "A".into(),
                parameter_kinds: Box::new([]),
            },
        ]),
    )]);
    assert!(matches!(
        validate_metadata_bundle(&duplicate_cases, &limits),
        Err(MetadataValidationError::DuplicateStaticDomainCase { case, .. }) if case.as_ref() == "A"
    ));

    let (mut unsorted_cases, declaration) = schema_v8_nominal_domain_bundle(8);
    unsorted_cases.static_domains = Box::new([schema_v8_domain_record(
        &declaration,
        Box::new([
            StaticDomainCaseRecord {
                name: "Z".into(),
                parameter_kinds: Box::new([]),
            },
            StaticDomainCaseRecord {
                name: "A".into(),
                parameter_kinds: Box::new([]),
            },
        ]),
    )]);
    assert!(matches!(
        validate_metadata_bundle(&unsorted_cases, &limits),
        Err(MetadataValidationError::Malformed(message)) if message.contains("static-domain cases must be sorted")
    ));
}

#[test]
fn schema_v8_validates_owner_parameter_modes_and_owner_environment_kinds() {
    let limits = ValidationLimits::default();
    let (mut invalid_mode, declaration) = schema_v8_nominal_domain_bundle(8);
    invalid_mode.static_domains = Box::new([StaticDomainDeclarationRecord {
        declaration: StableStaticDomainDeclarationRef::Nominal(declaration),
        owner_parameters: Box::new([StaticDomainParameterRecord {
            kind: KindNodeId(0),
            mode: GenericParameterModeRef::PositionalPack,
        }]),
        cases: Box::new([]),
    }]);
    assert!(matches!(
        validate_metadata_bundle(&invalid_mode, &limits),
        Err(MetadataValidationError::Malformed(message)) if message.contains("owner parameter mode")
    ));

    let owner = schema_v8_static_declaration("Container");
    let associated_declaration = phalcom_type_meta::identity::StableAssociatedStaticDomainDeclarationRef {
        owner,
        name: "State".into(),
    };
    let mut wrong_environment = sample_bundle(8, false);
    wrong_environment.header.features.static_domains = true;
    wrong_environment.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::StaticDomain(StaticDomainRef::Associated {
                declaration: associated_declaration.clone(),
                owner_environment: Box::new([StaticArgumentRef::Term(TypeNodeId(0))]),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    wrong_environment.types = Box::new([TypeNodeEntry {
        kind: KindNodeId(0),
        form: TypeNode::Unit,
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    wrong_environment.static_domains = Box::new([StaticDomainDeclarationRecord {
        declaration: StableStaticDomainDeclarationRef::Associated(associated_declaration),
        owner_parameters: Box::new([StaticDomainParameterRecord {
            kind: KindNodeId(1),
            mode: GenericParameterModeRef::Single,
        }]),
        cases: Box::new([]),
    }]);
    assert!(matches!(
        validate_metadata_bundle(&wrong_environment, &limits),
        Err(MetadataValidationError::StaticDomainOwnerKindMismatch { .. })
    ));
}

#[test]
fn schema_v8_validates_static_data_case_membership_arity_and_payload_kind() {
    let limits = ValidationLimits::default();
    let (mut valid_shape, declaration) = schema_v8_value_data_bundle();
    assert!(validate_metadata_bundle(&valid_shape, &limits).is_ok());

    let mut missing_case = valid_shape.clone();
    missing_case.static_domains = Box::new([schema_v8_domain_record(
        &declaration,
        Box::new([]),
    )]);
    assert!(matches!(
        validate_metadata_bundle(&missing_case, &limits),
        Err(MetadataValidationError::MissingStaticDomainCase { case, .. }) if case.as_ref() == "Value"
    ));

    valid_shape.types[1].form = TypeNode::StaticData(StaticDataRef {
        domain: StaticDomainRef::Nominal {
            declaration: declaration.clone(),
        },
        constructor: StaticConstructorRef {
            declaration: declaration.clone(),
            case_name: "Value".into(),
        },
        arguments: Box::new([]),
    });
    assert!(matches!(
        validate_metadata_bundle(&valid_shape, &limits),
        Err(MetadataValidationError::StaticDomainCaseArityMismatch { .. })
    ));

    let (mut wrong_kind, declaration) = schema_v8_value_data_bundle();
    wrong_kind.kinds = Box::new([
        wrong_kind.kinds[0].clone(),
        wrong_kind.kinds[1].clone(),
        KindNodeEntry {
            node: KindNode::StaticDomain(StaticDomainRef::ConstInt),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    let mut static_data = wrong_kind.types[1].clone();
    static_data.form = TypeNode::StaticData(StaticDataRef {
        domain: StaticDomainRef::Nominal {
            declaration: declaration.clone(),
        },
        constructor: StaticConstructorRef {
            declaration,
            case_name: "Value".into(),
        },
        arguments: Box::new([StaticArgumentRef::Term(TypeNodeId(1))]),
    });
    wrong_kind.types = Box::new([
        wrong_kind.types[0].clone(),
        TypeNodeEntry {
            kind: KindNodeId(2),
            form: TypeNode::ConstInt(7),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        static_data,
    ]);
    assert!(matches!(
        validate_metadata_bundle(&wrong_kind, &limits),
        Err(MetadataValidationError::StaticDomainCaseKindMismatch { .. })
    ));
}

#[test]
fn schema_v7_keeps_pre_catalog_static_domain_compatibility_floor() {
    let limits = ValidationLimits::default();
    let (v7, declaration) = schema_v8_nominal_domain_bundle(7);
    assert!(validate_metadata_bundle(&v7, &limits).is_ok());
    assert!(v7.static_domains.is_empty());
    assert_eq!(
        v7.kinds[1].node,
        KindNode::StaticDomain(StaticDomainRef::Nominal { declaration })
    );
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

    // Version 11 (future)
    let b11 = sample_bundle(11, false);
    let err11 = validate_metadata_bundle(&b11, &limits).unwrap_err();
    assert_eq!(
        err11,
        MetadataValidationError::UnsupportedSchemaVersion {
            found: 11,
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
fn schema_v5_static_domain_nodes_require_the_feature_and_matching_kinds() {
    let limits = ValidationLimits::default();

    let mut missing_feature = sample_bundle(5, false);
    missing_feature.kinds = Box::new([KindNodeEntry {
        node: KindNode::StaticDomain(StaticDomainRef::ConstInt),
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    missing_feature.types = Box::new([TypeNodeEntry {
        kind: KindNodeId(0),
        form: TypeNode::ConstInt(3),
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    assert!(validate_metadata_bundle(&missing_feature, &limits).is_err());

    let mut wrong_kind = missing_feature.clone();
    wrong_kind.header.features.static_domains = true;
    wrong_kind.kinds = Box::new([KindNodeEntry {
        node: KindNode::Type,
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    assert!(validate_metadata_bundle(&wrong_kind, &limits).is_err());
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
        mode: Default::default(),
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
                        presence: StructuralMemberPresenceRef::Required,
                    },
                    RecordFieldRef {
                        name: "b".into(),
                        ty: TypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Required,
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
                        presence: StructuralMemberPresenceRef::Required,
                    },
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Required,
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
                        presence: StructuralMemberPresenceRef::Required,
                    },
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Required,
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
        mode: Default::default(),
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
                parameter_modes: None,
                body: ScopedTypeNodeId(0),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    validate_metadata_bundle(&b, &limits).expect("scoped open record with row binder should validate");
}

#[test]
fn schema_v3_validates_complete_and_concat_pack_nodes_and_rejects_them_in_v2() {
    let limits = ValidationLimits::default();
    let mut bundle = sample_bundle(3, false);
    bundle.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::TypePack,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::LabeledTypePack,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::CompleteTypePack,
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(1),
            form: TypeNode::TypePack(Box::new([])),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(2),
            form: TypeNode::LabeledTypePack(Box::new([])),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(3),
            form: TypeNode::CompleteTypePack {
                positionals: TypeNodeId(1),
                labeled: TypeNodeId(2),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(1),
            form: TypeNode::TypePackConcat(Box::new([TypeNodeId(1)])),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(2),
            form: TypeNode::LabeledTypePackConcat(Box::new([TypeNodeId(2)])),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    validate_metadata_bundle(&bundle, &limits).expect("schema v3 C2 pack nodes should validate");

    let mut old_schema = bundle.clone();
    old_schema.header.schema_version = 2;
    assert!(validate_metadata_bundle(&old_schema, &limits).is_err());

    let mut invalid_mode = sample_bundle(3, false);
    invalid_mode.parameters = Box::new([TypeParameterRecord {
        id: StableTypeParameterRef {
            owner: StableTypeParameterOwnerRef::Declaration(StableDeclarationRef {
                module: StableModuleRef {
                    project: StableProjectRef::Builtin {
                        namespace: "test".into(),
                        version: "0.1.0".into(),
                    },
                    path: Box::new(["mod".into()]),
                },
                path: Box::new(["C".into()]),
            }),
            index: 0,
        },
        name: "P".into(),
        kind: KindNodeId(0),
        variance: VarianceRef::Invariant,
        mode: GenericParameterModeRef::PositionalPack,
        source: None,
    }]);
    assert!(validate_metadata_bundle(&invalid_mode, &limits).is_err());
}

#[test]
fn p3_schema_v4_scoped_pack_nodes_validate_and_reject_in_v3() {
    use phalcom_type_meta::scoped_type::{ScopedPackProjectionLaneRef, TypeLambdaRef};

    let limits = ValidationLimits::default();
    let mut bundle = sample_bundle(4, false);
    bundle.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::TypePack,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::LabeledTypePack,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::CompleteTypePack,
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::TypeLambda(TypeLambdaRef {
                parameter_kinds: Box::new([KindNodeId(1)]),
                parameter_modes: Some(Box::new([GenericParameterModeRef::PositionalPack])),
                body: ScopedTypeNodeId(0),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle.scoped_types = Box::new([
        ScopedTypeNodeEntry {
            kind: KindNodeId(1),
            form: ScopedTypeNode::TypePack(Box::new([])),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(2),
            form: ScopedTypeNode::LabeledTypePack(Box::new([])),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(3),
            form: ScopedTypeNode::CompleteTypePack {
                positionals: ScopedTypeNodeId(0),
                labeled: ScopedTypeNodeId(1),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(1),
            form: ScopedTypeNode::TypePackConcat(Box::new([ScopedTypeNodeId(0)])),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(2),
            form: ScopedTypeNode::LabeledTypePackConcat(Box::new([ScopedTypeNodeId(1)])),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::TupleFromPacks {
                positionals: ScopedTypeNodeId(0),
                labeled: ScopedTypeNodeId(1),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::CallableFromPacks {
                positionals: ScopedTypeNodeId(0),
                labeled: ScopedTypeNodeId(1),
                return_type: ScopedTypeNodeId(5), // Unit
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(1),
            form: ScopedTypeNode::PackProjection {
                subject: ScopedTypeNodeId(2),
                lane: ScopedPackProjectionLaneRef::Positionals,
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::Lambda {
                parameter_kinds: Box::new([KindNodeId(1)]),
                parameter_modes: Some(Box::new([GenericParameterModeRef::PositionalPack])),
                body: ScopedTypeNodeId(0),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    validate_metadata_bundle(&bundle, &limits).expect("schema v4 scoped pack bundle should validate");

    // V3 rejects scoped pack forms or explicit lambda modes
    let mut v3_bundle = bundle.clone();
    v3_bundle.header.schema_version = 3;
    assert!(validate_metadata_bundle(&v3_bundle, &limits).is_err());
}

#[test]
fn p3_schema_v4_rejects_malformed_lambda_modes() {
    use phalcom_type_meta::scoped_type::TypeLambdaRef;

    let limits = ValidationLimits::default();
    let mut bundle = sample_bundle(4, false);
    bundle.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::TypePack,
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    // Missing parameter_modes in v4
    bundle.types = Box::new([TypeNodeEntry {
        kind: KindNodeId(0),
        form: TypeNode::TypeLambda(TypeLambdaRef {
            parameter_kinds: Box::new([KindNodeId(1)]),
            parameter_modes: None,
            body: ScopedTypeNodeId(0),
        }),
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    bundle.scoped_types = Box::new([ScopedTypeNodeEntry {
        kind: KindNodeId(0),
        form: ScopedTypeNode::Bound { depth: 0, index: 0 },
        structural_fingerprint: Fingerprint128::ZERO,
    }]);
    assert!(validate_metadata_bundle(&bundle, &limits).is_err());

    // Mismatched length
    bundle.types[0].form = TypeNode::TypeLambda(TypeLambdaRef {
        parameter_kinds: Box::new([KindNodeId(1)]),
        parameter_modes: Some(Box::new([])),
        body: ScopedTypeNodeId(0),
    });
    assert!(validate_metadata_bundle(&bundle, &limits).is_err());

    // Mismatched mode to kind (PositionalPack mode with Type kind)
    bundle.types[0].form = TypeNode::TypeLambda(TypeLambdaRef {
        parameter_kinds: Box::new([KindNodeId(0)]),
        parameter_modes: Some(Box::new([GenericParameterModeRef::PositionalPack])),
        body: ScopedTypeNodeId(0),
    });
    assert!(validate_metadata_bundle(&bundle, &limits).is_err());
}

#[test]
fn schema_v10_structural_presence_and_remainders_roundtrip_and_validate() {
    use phalcom_type_meta::type_node::{
        AnonymousRecordRemainderRef, AnonymousRecordTypeRef, AnonymousTupleRemainderRef,
        OpenTupleTypeRef, TupleElementRef,
    };
    use phalcom_type_meta::scoped_type::{
        ScopedAnonymousRecordRemainderRef, ScopedAnonymousRecordTypeRef,
        ScopedAnonymousTupleRemainderRef, ScopedOpenTupleTypeRef, ScopedTupleElementRef,
    };

    let limits = ValidationLimits::default();
    let mut bundle = sample_bundle(10, true);
    bundle.kinds = Box::new([
        KindNodeEntry {
            node: KindNode::Type,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        KindNodeEntry {
            node: KindNode::RecordRow,
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle.types = Box::new([
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::OpenTuple(OpenTupleTypeRef {
                elements: Box::new([
                    TupleElementRef {
                        label: None,
                        ty: TypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Required,
                    },
                    TupleElementRef {
                        label: Some("opt".into()),
                        ty: TypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Optional,
                    },
                ]),
                positional_remainder: AnonymousTupleRemainderRef::Unconstrained,
                labeled_remainder: AnonymousTupleRemainderRef::Typed(TypeNodeId(0)),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        TypeNodeEntry {
            kind: KindNodeId(0),
            form: TypeNode::AnonymousRecord(AnonymousRecordTypeRef {
                fields: Box::new([
                    RecordFieldRef {
                        name: "a".into(),
                        ty: TypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Required,
                    },
                    RecordFieldRef {
                        name: "b".into(),
                        ty: TypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Optional,
                    },
                ]),
                remainder: AnonymousRecordRemainderRef::Typed(TypeNodeId(0)),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);
    bundle.scoped_types = Box::new([
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::Free(TypeNodeId(0)),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::OpenTuple(ScopedOpenTupleTypeRef {
                elements: Box::new([
                    ScopedTupleElementRef {
                        label: None,
                        ty: ScopedTypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Required,
                    },
                    ScopedTupleElementRef {
                        label: Some("opt".into()),
                        ty: ScopedTypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Optional,
                    },
                ]),
                positional_remainder: ScopedAnonymousTupleRemainderRef::Unconstrained,
                labeled_remainder: ScopedAnonymousTupleRemainderRef::Typed(ScopedTypeNodeId(0)),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        ScopedTypeNodeEntry {
            kind: KindNodeId(0),
            form: ScopedTypeNode::AnonymousRecord(ScopedAnonymousRecordTypeRef {
                fields: Box::new([
                    ScopedRecordFieldRef {
                        name: "a".into(),
                        ty: ScopedTypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Required,
                    },
                    ScopedRecordFieldRef {
                        name: "b".into(),
                        ty: ScopedTypeNodeId(0),
                        presence: StructuralMemberPresenceRef::Optional,
                    },
                ]),
                remainder: ScopedAnonymousRecordRemainderRef::Typed(ScopedTypeNodeId(0)),
            }),
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ]);

    validate_metadata_bundle(&bundle, &limits).expect("schema v10 bundle should validate");

    let encoded = encode_metadata_json(&bundle).expect("encode should succeed");
    let decoded = decode_metadata_json(&encoded, &limits).expect("decode should succeed");
    validate_metadata_bundle(&decoded, &limits).expect("decoded bundle should validate");
    assert_eq!(bundle, decoded);

    // Schema v9 rejects v10 structural nodes
    let mut v9_bundle = bundle.clone();
    v9_bundle.header.schema_version = 9;
    assert!(validate_metadata_bundle(&v9_bundle, &limits).is_err());
}

