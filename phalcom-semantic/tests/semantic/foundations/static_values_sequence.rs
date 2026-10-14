use num_bigint::BigInt;
use phalcom_semantic::core_surface::universe_declaration;
use phalcom_semantic::data_semantics::DataSemanticTable;
use phalcom_semantic::enum_semantics::EnumSemanticTable;
use phalcom_semantic::materialization::{DeferredStructuralKind, MaterializationCapability, MaterializationDescriptor, SequenceMaterializationKind};
use phalcom_semantic::static_value::{StaticFloat, StaticInt, StaticSequenceFamily, StaticValueData, StaticValueStore, StaticValueStoreError};
use phalcom_semantic::types::store::{StaticArgument, TupleTypeElement, TypeData};
use phalcom_semantic::types::{KindId, TypeId, TypeStore};
use std::sync::Arc;

fn array_type(types: &mut TypeStore, element: TypeId) -> TypeId {
    let origin_kind = types.arrow_kind(vec![KindId::TYPE].into_boxed_slice(), KindId::TYPE);
    let origin = types.intern_with_kind(
        TypeData::Nominal {
            declaration: universe_declaration(phalcom_native_meta::UniverseKey::Array),
        },
        origin_kind,
    );
    types
        .apply_static_form(origin, &[StaticArgument::Term(element)])
        .expect("Array type application")
}

fn vector_type(types: &mut TypeStore, element: TypeId, extent: BigInt) -> TypeId {
    let static_int_kind = types.static_domain_kind(types.const_int_domain());
    let origin_kind = types.arrow_kind(vec![KindId::TYPE, static_int_kind].into_boxed_slice(), KindId::TYPE);
    let origin = types.intern_with_kind(
        TypeData::Nominal {
            declaration: universe_declaration(phalcom_native_meta::UniverseKey::Vector),
        },
        origin_kind,
    );
    let extent = types.const_int(extent);
    types
        .apply_static_form(origin, &[StaticArgument::Term(element), StaticArgument::Term(extent)])
        .expect("Vector type application")
}

#[test]
fn static_sequence_storage_validates_family_extent_and_ownership() {
    let mut types = TypeStore::new();
    let int_type = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Int));
    let array = array_type(&mut types, int_type);
    let vector_two = vector_type(&mut types, int_type, BigInt::from(2));
    let vector_zero = vector_type(&mut types, int_type, BigInt::from(0));
    let vector_negative = vector_type(&mut types, int_type, BigInt::from(-1));
    let huge_extent = (BigInt::from(1) << 100usize) + BigInt::from(7);
    let vector_huge = vector_type(&mut types, int_type, huge_extent.clone());

    let mut values = StaticValueStore::new();
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    let array_value = values.intern_sequence(array, vec![one, two], &types).expect("Array value");
    let same_array = values.intern_sequence(array, vec![one, two], &types).expect("same Array value");
    assert_eq!(array_value, same_array, "identical sequence nodes are interned canonically");
    let StaticValueData::Sequence(array_id) = values.get(array_value).unwrap() else {
        panic!("expected Sequence");
    };
    let array_id = *array_id;
    let array_data = values.get_sequence(array_id).unwrap();
    assert_eq!(array_data.family(), StaticSequenceFamily::Array);
    assert_eq!(array_data.result_type(), array);
    assert_eq!(array_data.elements(), &[one, two]);

    let vector_value = values.intern_sequence(vector_two, vec![one, two], &types).expect("Vector value");
    assert_ne!(array_value, vector_value, "Array and Vector retain distinct result identity");
    assert!(matches!(values.get(vector_value), Ok(StaticValueData::Sequence(_))));
    assert!(
        values.intern_sequence(vector_two, vec![one], &types).is_err(),
        "exact Vector extent must match contents"
    );
    let empty_vector = values.intern_sequence(vector_zero, Vec::new(), &types).expect("zero extent Vector");
    assert!(matches!(values.get(empty_vector), Ok(StaticValueData::Sequence(_))));
    assert!(matches!(
        values.intern_sequence(vector_negative, Vec::new(), &types),
        Err(StaticValueStoreError::NegativeSequenceExtent(_))
    ));
    assert!(matches!(
        values.intern_sequence(vector_huge, vec![one, two], &types),
        Err(StaticValueStoreError::SequenceExtentMismatch { extent, actual: 2 }) if extent.to_bigint() == huge_extent
    ));

    let mut foreign_store = StaticValueStore::new();
    let foreign_child = foreign_store.intern_int(1);
    assert!(matches!(
        values.intern_sequence(array, vec![foreign_child], &types),
        Err(StaticValueStoreError::StoreMismatch { .. })
    ));

    let clone = values.clone();
    assert_eq!(clone.id(), values.id());
    assert_eq!(clone.get(array_value).unwrap(), values.get(array_value).unwrap());
    assert_eq!(
        clone.get_sequence(array_id).unwrap().elements(),
        values.get_sequence(array_id).unwrap().elements()
    );
    let appended = values.intern_int(99);
    assert!(values.get(appended).is_ok());
    assert_eq!(clone.get(array_value).unwrap(), values.get(array_value).unwrap());
}

#[test]
fn static_sequence_type_equality_hash_and_fingerprint_are_complete() {
    let mut types = TypeStore::new();
    let int_type = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Int));
    let float_type = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Float));
    let array_int = array_type(&mut types, int_type);
    let vector_two_int = vector_type(&mut types, int_type, BigInt::from(2));
    let array_float = array_type(&mut types, float_type);

    let mut values = StaticValueStore::new();
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    let array_12 = values.intern_sequence(array_int, vec![one, two], &types).unwrap();
    let array_21 = values.intern_sequence(array_int, vec![two, one], &types).unwrap();
    let vector_12 = values.intern_sequence(vector_two_int, vec![one, two], &types).unwrap();
    assert!(values.semantic_eq(array_12, array_12).unwrap());
    assert!(!values.semantic_eq(array_12, array_21).unwrap(), "logical order is semantic");
    assert!(!values.semantic_eq(array_12, vector_12).unwrap(), "family/type identity is semantic");
    assert_eq!(values.semantic_hash(array_12).unwrap(), values.semantic_hash(array_12).unwrap());
    assert_ne!(values.content_fingerprint(array_12).unwrap(), values.content_fingerprint(array_21).unwrap());
    assert_ne!(values.content_fingerprint(array_12).unwrap(), values.content_fingerprint(vector_12).unwrap());

    let plus_zero = values.intern_float(StaticFloat::from_f64(0.0));
    let minus_zero = values.intern_float(StaticFloat::from_f64(-0.0));
    let zero_a = values.intern_sequence(array_float, vec![plus_zero], &types).unwrap();
    let zero_b = values.intern_sequence(array_float, vec![minus_zero], &types).unwrap();
    assert!(values.semantic_eq(zero_a, zero_b).unwrap(), "sequence equality follows Telos Float equality");
    assert_eq!(values.semantic_hash(zero_a).unwrap(), values.semantic_hash(zero_b).unwrap());
    assert_ne!(values.content_fingerprint(zero_a).unwrap(), values.content_fingerprint(zero_b).unwrap());

    let mut other_store = StaticValueStore::new();
    other_store.intern_string("unrelated allocation before the sequence");
    other_store.intern_bool(false);
    let other_one = other_store.intern_int(1);
    let other_two = other_store.intern_int(2);
    let other_vector = other_store.intern_sequence(vector_two_int, vec![other_one, other_two], &types).unwrap();
    let other_array = other_store.intern_sequence(array_int, vec![other_one, other_two], &types).unwrap();
    assert_eq!(
        values.content_fingerprint(array_12).unwrap(),
        other_store.content_fingerprint(other_array).unwrap(),
        "fingerprints do not depend on static-value arena allocation order"
    );
    assert_eq!(
        values.semantic_hash(array_12).unwrap(),
        other_store.semantic_hash(other_array).unwrap(),
        "semantic hashes do not depend on static-value arena allocation order"
    );
    assert_ne!(
        values.content_fingerprint(array_12).unwrap(),
        other_store.content_fingerprint(other_vector).unwrap()
    );

    assert_eq!(
        values
            .static_value_type(array_12, &mut types, &DataSemanticTable::new(), &EnumSemanticTable::new())
            .unwrap(),
        array_int,
        "the retained canonical result type is projected exactly"
    );
    assert_eq!(
        values.domain_of(array_12, &types),
        Err(StaticValueStoreError::StructuralValueNeedsSemanticProducts)
    );
    assert_eq!(
        values.domain_of(vector_12, &types),
        Err(StaticValueStoreError::StructuralValueNeedsSemanticProducts)
    );
}

#[test]
fn static_sequence_materialization_is_recursive_and_truthfully_deferred() {
    let mut types = TypeStore::new();
    let int_type = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Int));
    let array_int = array_type(&mut types, int_type);
    let vector_two = vector_type(&mut types, int_type, BigInt::from(2));
    let array_of_arrays = array_type(&mut types, array_int);
    let tuple_type = types.tuple(vec![TupleTypeElement::unlabeled(int_type)].into_boxed_slice());
    let array_of_tuples = array_type(&mut types, tuple_type);

    let mut values = StaticValueStore::new();
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    let array_value = values.intern_sequence(array_int, vec![one, two], &types).expect("Array descriptor source");
    let vector_value = values.intern_sequence(vector_two, vec![one, two], &types).expect("Vector descriptor source");
    let nested_child = values.intern_sequence(array_int, vec![one], &types).expect("nested Array child");
    let nested_value = values
        .intern_sequence(array_of_arrays, vec![nested_child], &types)
        .expect("nested sequence descriptor source");
    let tuple = values.intern_tuple(vec![one]).expect("deferred Tuple child");
    let deferred_outer = values.intern_sequence(array_of_tuples, vec![tuple], &types).expect("deferred outer sequence");

    let array_descriptor = MaterializationDescriptor::from_static_value(&values, array_value).expect("Array descriptor");
    assert_eq!(
        array_descriptor,
        MaterializationDescriptor::Sequence {
            kind: SequenceMaterializationKind::FrozenArray,
            elements: Arc::from([
                MaterializationDescriptor::Int(StaticInt::Small(1)),
                MaterializationDescriptor::Int(StaticInt::Small(2))
            ]),
        }
    );
    assert_eq!(array_descriptor.capability(), MaterializationCapability::Sequence);
    assert!(!array_descriptor.is_primitive(), "a materializable sequence is not a primitive value");

    let vector_descriptor = MaterializationDescriptor::from_static_value(&values, vector_value).expect("Vector descriptor");
    assert_eq!(
        vector_descriptor,
        MaterializationDescriptor::Sequence {
            kind: SequenceMaterializationKind::Vector,
            elements: Arc::from([
                MaterializationDescriptor::Int(StaticInt::Small(1)),
                MaterializationDescriptor::Int(StaticInt::Small(2))
            ]),
        }
    );

    let nested_descriptor = MaterializationDescriptor::from_static_value(&values, nested_value).expect("nested descriptor");
    assert_eq!(
        nested_descriptor,
        MaterializationDescriptor::Sequence {
            kind: SequenceMaterializationKind::FrozenArray,
            elements: Arc::from([MaterializationDescriptor::Sequence {
                kind: SequenceMaterializationKind::FrozenArray,
                elements: Arc::from([MaterializationDescriptor::Int(StaticInt::Small(1))]),
            }]),
        }
    );

    let deferred_descriptor = MaterializationDescriptor::from_static_value(&values, deferred_outer).expect("deferred descriptor");
    assert_eq!(
        deferred_descriptor,
        MaterializationDescriptor::DeferredStructural(DeferredStructuralKind::Sequence)
    );
    assert_eq!(deferred_descriptor.capability(), MaterializationCapability::DeferredStructural);

    let clone = nested_descriptor.clone();
    drop(values);
    assert_eq!(
        clone, nested_descriptor,
        "descriptor clones own their recursive payload without arena references"
    );
}

#[test]
fn descriptor_materialization_handles_reasonably_deep_nested_sequences() {
    let mut types = TypeStore::new();
    let mut value = StaticValueStore::new();
    let mut element_type = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Int));
    let mut nested = value.intern_int(1);
    const DEPTH: usize = 128;

    for _ in 0..DEPTH {
        element_type = array_type(&mut types, element_type);
        nested = value.intern_sequence(element_type, vec![nested], &types).expect("nested static Array");
    }

    let descriptor = MaterializationDescriptor::from_static_value(&value, nested).expect("nested descriptor");
    let mut depth = 0;
    let mut current = &descriptor;
    loop {
        match current {
            MaterializationDescriptor::Sequence {
                kind: SequenceMaterializationKind::FrozenArray,
                elements,
            } => {
                depth += 1;
                current = elements.first().expect("nested element");
            }
            MaterializationDescriptor::Int(StaticInt::Small(1)) => break,
            other => panic!("unexpected nested descriptor at depth {depth}: {other:?}"),
        }
    }
    assert_eq!(depth, DEPTH);
}
