use phalcom_compiler_artifact::ValidationLimits;
use phalcom_compiler_artifact::semantic::static_value::{FieldKeyV1, StaticValueGraphV1};
use phalcom_type_meta::field_key::FieldKeyRef;
use phalcom_type_meta::fingerprint::Fingerprint128;
use phalcom_type_meta::identity::{StableDeclarationRef, StableModuleRef, StableProjectRef, StableStaticConstantRef, StableVariantFieldRef, StableVariantRef};
use phalcom_type_meta::kind::KindNodeId;
use phalcom_type_meta::scalar::StaticInt;
use phalcom_type_meta::static_definition::{StaticDefinitionRecord, StaticParameterRecord};
use phalcom_type_meta::static_value::{
    StaticConstantRecord, StaticSequenceFamilyRef, StaticValueDomainRef, StaticValueNode, StaticValueNodeEntry, StaticValueNodeId,
    compute_static_value_fingerprint,
};
use phalcom_type_meta::type_node::TypeNodeId;

fn sample_module() -> StableModuleRef {
    StableModuleRef {
        project: StableProjectRef::Builtin {
            namespace: "universe".into(),
            version: "0.1.0".into(),
        },
        path: Box::new(["core".into(), "test".into()]),
    }
}

fn sample_decl(name: &str) -> StableDeclarationRef {
    StableDeclarationRef {
        module: sample_module(),
        path: Box::new([name.into()]),
    }
}

#[test]
fn field_key_ref_converts_isomorphically_with_wire_dto() {
    let keys = vec![
        FieldKeyRef::Symbol("alpha".into()),
        FieldKeyRef::String("beta".into()),
        FieldKeyRef::Int(StaticInt::from(42)),
        FieldKeyRef::Int("-99999999999999999999999999999999999999".parse::<StaticInt>().unwrap()),
        FieldKeyRef::Bool(true),
        FieldKeyRef::Bool(false),
    ];

    for key in &keys {
        let wire = FieldKeyV1::from(key);
        let back = FieldKeyRef::try_from(&wire).expect("convert back to FieldKeyRef");
        assert_eq!(key, &back);
    }
}

#[test]
fn static_value_nodes_round_trip_through_artifact_graph() {
    let large_int = "1234567890123456789012345678901234567890".parse::<StaticInt>().unwrap();
    let decl = sample_decl("Option");
    let variant = StableVariantRef {
        owner: decl.clone(),
        selector: "Some(_) = 1".into(),
    };

    let entries = vec![
        // 0: Unit
        StaticValueNodeEntry {
            node: StaticValueNode::Unit,
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 1: Bool
        StaticValueNodeEntry {
            node: StaticValueNode::Bool(true),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 2: Char
        StaticValueNodeEntry {
            node: StaticValueNode::Char('λ'),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 3: Large Int
        StaticValueNodeEntry {
            node: StaticValueNode::Int(large_int.clone()),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 4: FloatBits
        StaticValueNodeEntry {
            node: StaticValueNode::FloatBits(1.2345_f64.to_bits()),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 5: String
        StaticValueNodeEntry {
            node: StaticValueNode::String("hello world".into()),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 6: Symbol
        StaticValueNodeEntry {
            node: StaticValueNode::Symbol("sym".into()),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 7: Type
        StaticValueNodeEntry {
            node: StaticValueNode::Type(TypeNodeId(10)),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 8: Tuple (0, 1)
        StaticValueNodeEntry {
            node: StaticValueNode::Tuple(vec![StaticValueNodeId(0), StaticValueNodeId(1)].into_boxed_slice()),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 9: Sequence Array (1, 1)
        StaticValueNodeEntry {
            node: StaticValueNode::Sequence {
                family: StaticSequenceFamilyRef::Array,
                result_type: TypeNodeId(11),
                elements: vec![StaticValueNodeId(1), StaticValueNodeId(1)].into_boxed_slice(),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 10: Record with heterogeneous keys
        StaticValueNodeEntry {
            node: StaticValueNode::Record(
                vec![
                    (FieldKeyRef::Symbol("a".into()), StaticValueNodeId(0)),
                    (FieldKeyRef::Int(StaticInt::from(1)), StaticValueNodeId(1)),
                ]
                .into_boxed_slice(),
            ),
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 11: Variant (Option::Some(3))
        StaticValueNodeEntry {
            node: StaticValueNode::Variant {
                variant: variant.clone(),
                result_type: TypeNodeId(12),
                fields: vec![(
                    StableVariantFieldRef {
                        variant: variant.clone(),
                        index: 0,
                    },
                    StaticValueNodeId(3),
                )]
                .into_boxed_slice(),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
        // 12: Pack
        StaticValueNodeEntry {
            node: StaticValueNode::Pack {
                positionals: vec![StaticValueNodeId(0)].into_boxed_slice(),
                labeled: vec![("lbl".into(), StaticValueNodeId(1))].into_boxed_slice(),
            },
            structural_fingerprint: Fingerprint128::ZERO,
        },
    ];

    // Compute structural fingerprints on live entries
    let mut live_entries = entries;
    for i in 0..live_entries.len() {
        let fp = compute_static_value_fingerprint(
            &live_entries[i].node,
            |cid| live_entries[cid.0 as usize].structural_fingerprint,
            |tid| Fingerprint128::from_u128(tid.0 as u128),
        );
        live_entries[i].structural_fingerprint = fp;
    }

    // Convert live metadata entries -> wire graph
    let wire_graph = StaticValueGraphV1::from_metadata_nodes(&live_entries).expect("convert to StaticValueGraphV1");
    wire_graph.validate(&ValidationLimits::default()).expect("validate wire graph");

    // Convert wire graph -> restored metadata entries
    let restored_entries = wire_graph.to_metadata_nodes().expect("convert back to StaticValueNodeEntry");

    assert_eq!(live_entries.len(), restored_entries.len());
    for (i, (expected, actual)) in live_entries.iter().zip(restored_entries.iter()).enumerate() {
        assert_eq!(&expected.node, &actual.node, "mismatch at node index {i}");
        assert_eq!(
            expected.structural_fingerprint, actual.structural_fingerprint,
            "fingerprint mismatch at node index {i}"
        );
    }
}

#[test]
fn static_definition_record_durable_authority() {
    let decl = sample_decl("Calculate");
    let param = StaticParameterRecord {
        index: 0,
        name: "N".into(),
        domain: StaticValueDomainRef::Kind(KindNodeId(0)),
        mode: phalcom_type_meta::generic::GenericParameterModeRef::Single,
    };
    let def_record = StaticDefinitionRecord {
        id: decl.clone(),
        parameters: vec![param].into_boxed_slice(),
        result_domain: StaticValueDomainRef::ValueType(TypeNodeId(5)),
        semantic_fingerprint: Fingerprint128::from_u128(0xaabb_ccdd_eeff),
    };

    let serialized = serde_json::to_string(&def_record).expect("serialize static def record");
    let deserialized: StaticDefinitionRecord = serde_json::from_str(&serialized).expect("deserialize static def record");

    assert_eq!(def_record, deserialized);
    assert_eq!(deserialized.id, decl);
    assert_eq!(deserialized.parameters.len(), 1);
    assert_eq!(deserialized.parameters[0].name.as_ref(), "N");
}

#[test]
fn static_constant_record_durable_authority() {
    let module = sample_module();
    let const_ref = StableStaticConstantRef {
        module,
        name: "BUFFER_SIZE".into(),
    };
    let const_record = StaticConstantRecord {
        id: const_ref.clone(),
        domain: StaticValueDomainRef::ValueType(TypeNodeId(1)),
        value: StaticValueNodeId(42),
        value_fingerprint: Fingerprint128::from_u128(0x1111_2222),
        dependency_fingerprint: Fingerprint128::from_u128(0x3333_4444),
    };

    let serialized = serde_json::to_string(&const_record).expect("serialize static const record");
    let deserialized: StaticConstantRecord = serde_json::from_str(&serialized).expect("deserialize static const record");

    assert_eq!(const_record, deserialized);
    assert_eq!(deserialized.id, const_ref);
    assert_eq!(deserialized.value, StaticValueNodeId(42));
}
