use num_bigint::BigInt;
use phalcom_type_meta::{StaticInt, TypeNode};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Neg;
use std::sync::Arc;

#[test]
fn static_int_normalizes_small_and_large_values_without_changing_equality() {
    let small = StaticInt::from(7_i64);
    let large_value = BigInt::parse_bytes(b"9223372036854775808", 10).unwrap();
    let large = StaticInt::from(large_value.clone());

    assert!(matches!(small, StaticInt::Small(7)));
    assert!(matches!(large, StaticInt::Large(_)));
    assert_eq!(large, StaticInt::from(large_value));
    assert_ne!(small, large);
}

#[test]
fn type_nodes_round_trip_exact_positive_and_negative_integers_as_portable_text() {
    let positive = BigInt::parse_bytes(b"9223372036854775808", 10).unwrap();
    let negative = BigInt::parse_bytes(b"9223372036854775809", 10).unwrap().neg();
    let nodes = [
        TypeNode::ConstInt(StaticInt::from(positive.clone())),
        TypeNode::ConstInt(StaticInt::from(negative.clone())),
    ];

    let json = serde_json::to_string(&nodes).unwrap();
    assert!(json.contains("9223372036854775808"));
    assert!(json.contains("-9223372036854775809"));

    let decoded: Vec<TypeNode> = serde_json::from_str(&json).unwrap();
    assert_eq!(decoded, nodes);
}

#[test]
fn static_int_small_and_large_inputs_share_mathematical_equality_hash_and_order() {
    for value in [i64::MIN, -7, 0, 7, i64::MAX] {
        let small = StaticInt::Small(value);
        let large = StaticInt::Large(Arc::new(BigInt::from(value)));
        let hash = |integer: &StaticInt| {
            let mut hasher = DefaultHasher::new();
            integer.hash(&mut hasher);
            hasher.finish()
        };
        assert_eq!(small, large);
        assert_eq!(hash(&small), hash(&large));
        assert_eq!(small.cmp(&large), std::cmp::Ordering::Equal);
    }
}

#[test]
fn static_int_serde_normalizes_decimal_text_and_reads_legacy_numbers() {
    for json in ["\"0007\"", "\"+7\"", "7"] {
        let value: StaticInt = serde_json::from_str(json).unwrap();
        assert!(matches!(value, StaticInt::Small(7)));
        assert_eq!(serde_json::to_string(&value).unwrap(), "\"7\"");
    }
    let zero: StaticInt = serde_json::from_str("\"-000\"").unwrap();
    assert!(matches!(zero, StaticInt::Small(0)));
    for invalid in ["1.5", "null", "\"1e20\"", "\"not an integer\""] {
        assert!(serde_json::from_str::<StaticInt>(invalid).is_err());
    }
}

#[test]
fn type_member_arguments_round_trip_exact_integers() {
    use phalcom_type_meta::TypeMemberArgumentRef;
    for decimal in ["1234567890123456789012345678901234567890", "-1234567890123456789012345678901234567890"] {
        let argument = TypeMemberArgumentRef::ConstInt(StaticInt::from(BigInt::parse_bytes(decimal.as_bytes(), 10).unwrap()));
        let json = serde_json::to_string(&argument).unwrap();
        assert_eq!(json, format!("{{\"ConstInt\":\"{decimal}\"}}"));
        assert_eq!(serde_json::from_str::<TypeMemberArgumentRef>(&json).unwrap(), argument);
    }
}
