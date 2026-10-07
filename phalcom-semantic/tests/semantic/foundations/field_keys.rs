use num_bigint::BigInt;
use phalcom_semantic::field_key::{FieldKey, FieldKeyKind};
use phalcom_semantic::scalar::StaticInt;
use std::cmp::Ordering;

#[test]
fn field_key_variants_and_kinds() {
    let sym = FieldKey::symbol("alpha");
    let str_k = FieldKey::string("beta");
    let int_k = FieldKey::int(StaticInt::from(42));
    let bool_k = FieldKey::bool(true);

    assert_eq!(sym.kind(), FieldKeyKind::Symbol);
    assert_eq!(str_k.kind(), FieldKeyKind::String);
    assert_eq!(int_k.kind(), FieldKeyKind::Int);
    assert_eq!(bool_k.kind(), FieldKeyKind::Bool);

    assert_eq!(sym.as_symbol(), Some("alpha"));
    assert_eq!(sym.as_string(), None);
    assert_eq!(str_k.as_string(), Some("beta"));
    assert_eq!(str_k.as_symbol(), None);
}

#[test]
fn field_key_canonical_ordering_across_kinds() {
    // Cross-kind order must be Symbol < String < Int < Bool (P4-INV-03)
    let sym = FieldKey::symbol("z");
    let str_k = FieldKey::string("a");
    let int_k = FieldKey::int(StaticInt::from(-100));
    let bool_k = FieldKey::bool(false);

    assert_eq!(sym.canonical_cmp(&str_k), Ordering::Less);
    assert_eq!(str_k.canonical_cmp(&int_k), Ordering::Less);
    assert_eq!(int_k.canonical_cmp(&bool_k), Ordering::Less);
    assert_eq!(sym.canonical_cmp(&bool_k), Ordering::Less);

    let mut keys = vec![bool_k.clone(), str_k.clone(), sym.clone(), int_k.clone()];
    keys.sort();
    assert_eq!(keys, vec![sym, str_k, int_k, bool_k]);
}

#[test]
fn field_key_canonical_ordering_within_kinds() {
    // Symbol lexical
    let s1 = FieldKey::symbol("apple");
    let s2 = FieldKey::symbol("banana");
    assert_eq!(s1.canonical_cmp(&s2), Ordering::Less);
    assert_eq!(s2.canonical_cmp(&s1), Ordering::Greater);
    assert_eq!(s1.canonical_cmp(&s1), Ordering::Equal);

    // String lexical
    let str1 = FieldKey::string("apple");
    let str2 = FieldKey::string("banana");
    assert_eq!(str1.canonical_cmp(&str2), Ordering::Less);
    assert_eq!(str2.canonical_cmp(&str1), Ordering::Greater);

    // Int mathematical (arbitrary precision)
    let i_neg_big = FieldKey::int(StaticInt::new(BigInt::parse_bytes(b"-1000000000000000000000000000000000", 10).unwrap()));
    let i_neg_small = FieldKey::int(StaticInt::from(-42));
    let i_zero = FieldKey::int(StaticInt::from(0));
    let i_pos_small = FieldKey::int(StaticInt::from(42));
    let i_pos_big = FieldKey::int(StaticInt::new(BigInt::parse_bytes(b"1000000000000000000000000000000000", 10).unwrap()));

    let mut int_keys = vec![i_pos_big.clone(), i_neg_small.clone(), i_zero.clone(), i_pos_small.clone(), i_neg_big.clone()];
    int_keys.sort();
    assert_eq!(int_keys, vec![i_neg_big, i_neg_small, i_zero, i_pos_small, i_pos_big]);

    // Bool: false < true
    let b_false = FieldKey::bool(false);
    let b_true = FieldKey::bool(true);
    assert_eq!(b_false.canonical_cmp(&b_true), Ordering::Less);
    assert_eq!(b_true.canonical_cmp(&b_false), Ordering::Greater);
    assert_eq!(b_false.canonical_cmp(&b_false), Ordering::Equal);
}

#[test]
fn field_key_same_text_symbol_and_string_are_distinct() {
    // P4-INV-04: Symbol and String with identical text are distinct
    let sym = FieldKey::symbol("content");
    let str_k = FieldKey::string("content");

    assert_ne!(sym, str_k);
    assert_eq!(sym.canonical_cmp(&str_k), Ordering::Less);
    assert_eq!(str_k.canonical_cmp(&sym), Ordering::Greater);
}

#[test]
fn field_key_int_and_bool_are_distinct() {
    // P4-INV-05: 1 and true are distinct; 0 and false are distinct
    let int_one = FieldKey::int(StaticInt::from(1));
    let bool_true = FieldKey::bool(true);
    assert_ne!(int_one, bool_true);
    assert_eq!(int_one.canonical_cmp(&bool_true), Ordering::Less);

    let int_zero = FieldKey::int(StaticInt::from(0));
    let bool_false = FieldKey::bool(false);
    assert_ne!(int_zero, bool_false);
    assert_eq!(int_zero.canonical_cmp(&bool_false), Ordering::Less);
}

#[test]
fn field_key_stable_fingerprints() {
    let sym = FieldKey::symbol("x");
    let str_k = FieldKey::string("x");
    let int_zero = FieldKey::int(StaticInt::from(0));
    let bool_false = FieldKey::bool(false);

    let fp_sym = sym.stable_fingerprint_bytes();
    let fp_str = str_k.stable_fingerprint_bytes();
    let fp_int = int_zero.stable_fingerprint_bytes();
    let fp_bool = bool_false.stable_fingerprint_bytes();

    // Reproducibility
    assert_eq!(fp_sym, sym.stable_fingerprint_bytes());
    assert_eq!(fp_str, str_k.stable_fingerprint_bytes());
    assert_eq!(fp_int, int_zero.stable_fingerprint_bytes());
    assert_eq!(fp_bool, bool_false.stable_fingerprint_bytes());

    // Distinct tags: 0 for Symbol, 1 for String, 2 for Int, 3 for Bool
    assert_eq!(fp_sym[0], 0);
    assert_eq!(fp_str[0], 1);
    assert_eq!(fp_int[0], 2);
    assert_eq!(fp_bool[0], 3);

    // Same text #x vs "x" must have different fingerprints
    assert_ne!(fp_sym, fp_str);
    // 0 vs false must have different fingerprints
    assert_ne!(fp_int, fp_bool);
}

#[test]
fn field_key_display_formatting() {
    assert_eq!(format!("{}", FieldKey::symbol("hello")), "#hello");
    assert_eq!(format!("{}", FieldKey::string("hello")), "\"hello\"");
    assert_eq!(format!("{}", FieldKey::int(StaticInt::from(42))), "42");
    assert_eq!(format!("{}", FieldKey::bool(true)), "true");
    assert_eq!(format!("{}", FieldKey::bool(false)), "false");
}
