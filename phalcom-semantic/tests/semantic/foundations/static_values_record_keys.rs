use num_bigint::BigInt;
use phalcom_semantic::data_semantics::DataSemanticTable;
use phalcom_semantic::enum_semantics::EnumSemanticTable;
use phalcom_semantic::field_key::FieldKey;
use phalcom_semantic::scalar::StaticInt;
use phalcom_semantic::static_value::{StaticFloat, StaticValueStore, StaticValueStoreError};
use phalcom_semantic::types::store::{TypeData, TypeStore};

#[test]
fn mixed_static_record_keys_are_canonical_and_content_fingerprinted_across_stores() {
    let large_int = StaticInt::new(BigInt::parse_bytes(b"1234567890123456789012345678901234567890", 10).unwrap());
    let mut first_store = StaticValueStore::new();
    let one = first_store.intern_int(1);
    let two = first_store.intern_int(2);
    let first = first_store
        .intern_record(vec![
            (FieldKey::string("same"), two),
            (FieldKey::bool(false), one),
            (FieldKey::int(large_int.clone()), two),
            (FieldKey::symbol("same"), one),
        ])
        .unwrap();
    let reordered = first_store
        .intern_record(vec![
            (FieldKey::symbol("same"), one),
            (FieldKey::int(large_int.clone()), two),
            (FieldKey::bool(false), one),
            (FieldKey::string("same"), two),
        ])
        .unwrap();

    assert_eq!(first, reordered, "source order must not affect the interned static Record");
    assert!(first_store.semantic_eq(first, reordered).unwrap());
    assert_eq!(first_store.semantic_hash(first).unwrap(), first_store.semantic_hash(reordered).unwrap());
    assert_eq!(
        first_store.content_fingerprint(first).unwrap(),
        first_store.content_fingerprint(reordered).unwrap()
    );

    let mut second_store = StaticValueStore::new();
    let one = second_store.intern_int(1);
    let two = second_store.intern_int(2);
    let independently_built = second_store
        .intern_record(vec![
            (FieldKey::bool(false), one),
            (FieldKey::symbol("same"), one),
            (FieldKey::string("same"), two),
            (FieldKey::int(large_int), two),
        ])
        .unwrap();
    assert_ne!(first, independently_built, "refs remain store-qualified");
    assert_eq!(
        first_store.semantic_hash(first).unwrap(),
        second_store.semantic_hash(independently_built).unwrap()
    );
    assert_eq!(
        first_store.content_fingerprint(first).unwrap(),
        second_store.content_fingerprint(independently_built).unwrap()
    );
}

#[test]
fn static_record_rejects_duplicates_by_exact_field_key_identity() {
    let mut values = StaticValueStore::new();
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    let keys = [
        FieldKey::symbol("same"),
        FieldKey::string("same"),
        FieldKey::int(StaticInt::from(5)),
        FieldKey::bool(true),
    ];

    for key in keys {
        let result = values.intern_record(vec![(key.clone(), one), (key.clone(), two)]);
        assert!(matches!(result, Err(StaticValueStoreError::DuplicateRecordKey(duplicate)) if duplicate == key));
    }

    let distinct_same_text = values.intern_record(vec![(FieldKey::symbol("same"), one), (FieldKey::string("same"), two)]);
    assert!(distinct_same_text.is_ok(), "Symbol and String with equal text remain distinct keys");
    let distinct_scalar_kinds = values.intern_record(vec![(FieldKey::int(StaticInt::from(1)), one), (FieldKey::bool(true), two)]);
    assert!(distinct_scalar_kinds.is_ok(), "Int and Bool keys remain distinct");
}

#[test]
fn field_key_conversion_owns_content_and_rejects_other_static_values() {
    let mut values = StaticValueStore::new();
    let symbol = values.intern_symbol("shared");
    let string = values.intern_string("shared");
    let integer = values.intern_int(BigInt::parse_bytes(b"999999999999999999999999999999999999", 10).unwrap());
    let boolean = values.intern_bool(true);

    let symbol_key = values.field_key_of(symbol).unwrap();
    let string_key = values.field_key_of(string).unwrap();
    let integer_key = values.field_key_of(integer).unwrap();
    let boolean_key = values.field_key_of(boolean).unwrap();
    assert_eq!(symbol_key, FieldKey::symbol("shared"));
    assert_eq!(string_key, FieldKey::string("shared"));
    assert!(matches!(integer_key, FieldKey::Int(value) if value.to_bigint() == BigInt::parse_bytes(b"999999999999999999999999999999999999", 10).unwrap()));
    assert_eq!(boolean_key, FieldKey::bool(true));
    drop(values);
    assert_eq!(symbol_key, FieldKey::symbol("shared"), "converted keys own their content");

    let mut values = StaticValueStore::new();
    let float = values.intern_float(StaticFloat::from_f64(1.5));
    let unit = values.intern_unit();
    assert_eq!(values.field_key_of(float), Err(StaticValueStoreError::NotAFieldKey));
    assert_eq!(values.field_key_of(unit), Err(StaticValueStoreError::NotAFieldKey));

    let foreign = StaticValueStore::new().intern_string("foreign");
    assert!(matches!(values.field_key_of(foreign), Err(StaticValueStoreError::StoreMismatch { .. })));
}

#[test]
fn static_record_type_preserves_mixed_canonical_row_keys() {
    let mut values = StaticValueStore::new();
    let one = values.intern_int(1);
    let two = values.intern_int(2);
    let record = values
        .intern_record(vec![
            (FieldKey::string("same"), two),
            (FieldKey::bool(true), one),
            (FieldKey::symbol("same"), one),
            (FieldKey::int(StaticInt::from(7)), two),
        ])
        .unwrap();
    let mut types = TypeStore::new();
    types.nominal_type(phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Int));
    types.nominal_type(phalcom_semantic::core_surface::universe_declaration(phalcom_native_meta::UniverseKey::Bool));
    let record_type = values
        .static_value_type(record, &mut types, &DataSemanticTable::new(), &EnumSemanticTable::new())
        .unwrap();
    let TypeData::Record(row_id) = types.get(record_type) else {
        panic!("expected a Record type for a static Record value");
    };
    assert_eq!(
        types.record_row(*row_id).fields.iter().map(|field| field.key.clone()).collect::<Vec<_>>(),
        vec![
            FieldKey::symbol("same"),
            FieldKey::string("same"),
            FieldKey::int(StaticInt::from(7)),
            FieldKey::bool(true),
        ]
    );
}
