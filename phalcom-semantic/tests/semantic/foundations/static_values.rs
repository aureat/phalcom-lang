use num_bigint::BigInt;
use phalcom_semantic::core_surface::universe_declaration;
use phalcom_semantic::static_value::{StaticFloat, StaticInt, StaticValueData, StaticValueStore, StaticValueStoreError, static_float_eq, static_float_hash};
use phalcom_semantic::types::{KindId, TypeStore};

#[test]
fn static_value_store_preserves_exact_primitives_and_snapshot_lineage() {
    let mut types = TypeStore::new();
    let bool_ty = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Bool));
    let int_ty = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Int));
    let float_ty = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Float));
    let string_ty = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::String));
    let symbol_ty = types.nominal_type(universe_declaration(phalcom_native_meta::UniverseKey::Symbol));

    let mut values = StaticValueStore::new();
    let unit = values.intern_unit();
    let yes = values.intern_bool(true);
    let no = values.intern_bool(false);
    assert_ne!(yes, no);
    assert!(matches!(values.get(yes), Ok(StaticValueData::Bool(true))));
    assert!(matches!(values.get(unit), Ok(StaticValueData::Unit)));
    assert_eq!(
        values.domain_of(unit, &types).unwrap(),
        phalcom_semantic::static_domain::StaticValueDomain::ValueType(types.unit())
    );
    assert_eq!(
        values.domain_of(yes, &types).unwrap(),
        phalcom_semantic::static_domain::StaticValueDomain::ValueType(bool_ty)
    );

    let small = values.intern_int(BigInt::from(42));
    assert!(matches!(values.get(small), Ok(StaticValueData::Int(StaticInt::Small(42)))));
    let large_positive = BigInt::from(i64::MAX) + BigInt::from(1u8);
    let large_negative = BigInt::from(i64::MIN) - BigInt::from(1u8);
    let large = values.intern_int(large_positive.clone());
    let negative_large = values.intern_int(large_negative.clone());
    assert!(matches!(values.get(large), Ok(StaticValueData::Int(StaticInt::Large(_)))));
    assert!(matches!(values.get(negative_large), Ok(StaticValueData::Int(StaticInt::Large(_)))));
    let large_again = values.intern_int(large_positive);
    let negative_large_again = values.intern_int(large_negative);
    assert!(values.semantic_eq(large, large_again).unwrap());
    assert!(values.semantic_eq(negative_large, negative_large_again).unwrap());
    assert_eq!(
        values.domain_of(large, &types).unwrap(),
        phalcom_semantic::static_domain::StaticValueDomain::ValueType(int_ty)
    );

    let positive_zero = StaticFloat::from_f64(0.0);
    let negative_zero = StaticFloat::from_f64(-0.0);
    assert_ne!(positive_zero.bits(), negative_zero.bits());
    assert!(static_float_eq(&positive_zero, &negative_zero));
    assert_eq!(static_float_hash(&positive_zero), static_float_hash(&negative_zero));
    let positive_zero_ref = values.intern_float(positive_zero);
    let negative_zero_ref = values.intern_float(negative_zero);
    assert_ne!(positive_zero_ref, negative_zero_ref);
    assert!(values.semantic_eq(positive_zero_ref, negative_zero_ref).unwrap());
    assert_eq!(
        values.semantic_hash(positive_zero_ref).unwrap(),
        values.semantic_hash(negative_zero_ref).unwrap()
    );
    assert_ne!(
        values.content_fingerprint(positive_zero_ref).unwrap(),
        values.content_fingerprint(negative_zero_ref).unwrap()
    );
    let nan_a = StaticFloat::from_bits(0x7ff8_0000_0000_0001);
    let nan_b = StaticFloat::from_bits(0x7ff8_0000_0000_0002);
    let nan_a_ref = values.intern_float(nan_a);
    let nan_b_ref = values.intern_float(nan_b);
    assert!(!values.semantic_eq(nan_a_ref, nan_a_ref).unwrap());
    assert!(!values.semantic_eq(nan_a_ref, nan_b_ref).unwrap());
    assert_eq!(static_float_hash(&nan_a), static_float_hash(&nan_b));
    assert_eq!(values.semantic_hash(nan_a_ref).unwrap(), values.semantic_hash(nan_b_ref).unwrap());
    assert_ne!(values.content_fingerprint(nan_a_ref).unwrap(), values.content_fingerprint(nan_b_ref).unwrap());
    assert_eq!(
        values.domain_of(positive_zero_ref, &types).unwrap(),
        phalcom_semantic::static_domain::StaticValueDomain::ValueType(float_ty)
    );
    let infinity = values.intern_float(StaticFloat::from_f64(f64::INFINITY));
    assert!(matches!(values.get(infinity), Ok(StaticValueData::Float(value)) if value.to_f64().is_infinite()));
    let finite = values.intern_float(StaticFloat::from_f64(3.25));
    assert!(matches!(values.get(finite), Ok(StaticValueData::Float(value)) if value.to_f64() == 3.25));

    let string_a = values.intern_string("héllo");
    let string_b = values.intern_string(String::from("héllo"));
    assert_eq!(string_a, string_b);
    assert!(values.semantic_eq(string_a, string_b).unwrap());
    assert_eq!(
        values.domain_of(string_a, &types).unwrap(),
        phalcom_semantic::static_domain::StaticValueDomain::ValueType(string_ty)
    );
    let symbol_a = values.intern_symbol("name");
    let symbol_b = values.intern_symbol(String::from("name"));
    assert_eq!(symbol_a, symbol_b);
    assert!(values.semantic_eq(symbol_a, symbol_b).unwrap());
    assert_eq!(
        values.domain_of(symbol_a, &types).unwrap(),
        phalcom_semantic::static_domain::StaticValueDomain::ValueType(symbol_ty)
    );

    let proper = values.intern_type(int_ty, &types).unwrap();
    assert!(matches!(values.get(proper), Ok(StaticValueData::Type(_))));
    let static_kind = types.static_domain_kind(types.const_int_domain());
    let static_term_type = types.nominal_form(universe_declaration(phalcom_native_meta::UniverseKey::Int), static_kind);
    let static_term = values.intern_static_term(static_term_type, &types).unwrap();
    assert!(matches!(values.get(static_term), Ok(StaticValueData::StaticTerm(_))));
    assert!(matches!(
        values.intern_type(static_term_type, &types),
        Err(StaticValueStoreError::NonProperType { .. })
    ));
    assert!(matches!(
        values.intern_static_term(int_ty, &types),
        Err(StaticValueStoreError::ProperTypeAsStaticTerm(_))
    ));
    assert_eq!(
        values.domain_of(proper, &types).unwrap(),
        phalcom_semantic::static_domain::StaticValueDomain::Kind(KindId::TYPE)
    );
    assert_eq!(
        values.domain_of(static_term, &types).unwrap(),
        phalcom_semantic::static_domain::StaticValueDomain::Kind(static_kind)
    );

    let clone = values.clone();
    assert_eq!(clone.id(), values.id());
    assert_eq!(clone.get(small).unwrap(), values.get(small).unwrap());
    let appended = values.intern_int(99);
    assert_eq!(clone.get(small).unwrap(), values.get(small).unwrap());
    assert!(values.get(appended).is_ok());
    let unrelated = StaticValueStore::new();
    assert!(matches!(unrelated.get(small), Err(StaticValueStoreError::StoreMismatch { .. })));
    assert!(matches!(
        values.intern_type(phalcom_semantic::types::TypeId(u32::MAX), &types),
        Err(StaticValueStoreError::UnknownType(_))
    ));
}
