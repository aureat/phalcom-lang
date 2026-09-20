//! Numeric domain and primitive contract audit probes.

use num_bigint::BigInt;
use phalcom_core::{value::Value, vm::VM};

#[test]
fn numeric_contracts_remainder_and_quotient() {
    let mut vm = VM::new_native();
    let cases: [(&str, Value, &str, Value, Value); 4] = [
        ("negative float remainder", Value::float(-5.0), "%(_)", Value::float(3.0), Value::float(1.0)),
        (
            "negative divisor float remainder",
            Value::float(5.0),
            "%(_)",
            Value::float(-3.0),
            Value::float(-1.0),
        ),
        (
            "mixed exact quotient",
            Value::int(9_007_199_254_740_993),
            "~/(_)",
            Value::float(1.0),
            Value::int(9_007_199_254_740_993),
        ),
        (
            "integer quotient control",
            Value::int(9_007_199_254_740_993),
            "~/(_)",
            Value::int(1),
            Value::int(9_007_199_254_740_993),
        ),
    ];

    for (label, receiver, selector, arg, expected) in cases {
        let sym = vm.get_or_intern(selector);
        let result = vm.send_dynamic(receiver, sym, &[arg]).unwrap_or_else(|e| panic!("failed {label}: {e:?}"));
        assert_eq!(result, expected, "{label}");
    }
}

#[test]
fn numeric_contracts_fixed_width_minimum_divided_by_negative_one_is_exact() {
    let mut vm = VM::new_native();
    let minimum = Value::int(i64::MIN);
    let negative_one = Value::int(-1);
    let floor_div = vm.get_or_intern("~/(_)");
    let modulo = vm.get_or_intern("%(_)");

    let quotient = vm.send_dynamic(minimum, floor_div, &[negative_one]).unwrap();
    let quotient_id = quotient.as_obj().expect("minimum quotient must promote to LargeInt");
    let expected_quotient = -BigInt::from(i64::MIN);
    assert_eq!(vm.heap.as_large_int(quotient_id), Some(&expected_quotient));

    let remainder = vm.send_dynamic(minimum, modulo, &[negative_one]).unwrap();
    assert_eq!(remainder, Value::int(0));
}

#[test]
fn numeric_contracts_float_remainder_special_values_and_signed_zero() {
    let mut vm = VM::new_native();
    let modulo = vm.get_or_intern("%(_)");

    let negative_zero = vm.send_dynamic(Value::float(-0.0), modulo, &[Value::float(3.0)]).unwrap();
    assert_eq!(negative_zero.as_float().unwrap().to_bits(), 0.0f64.to_bits());

    let positive_zero = vm.send_dynamic(Value::float(0.0), modulo, &[Value::float(-3.0)]).unwrap();
    assert_eq!(positive_zero.as_float().unwrap().to_bits(), (-0.0f64).to_bits());

    for (receiver, argument) in [
        (Value::float(f64::NAN), Value::float(3.0)),
        (Value::float(3.0), Value::float(f64::NAN)),
        (Value::float(f64::INFINITY), Value::float(3.0)),
        (Value::float(3.0), Value::float(f64::INFINITY)),
        (Value::float(3.0), Value::float(0.0)),
    ] {
        let result = vm.send_dynamic(receiver, modulo, &[argument]).unwrap();
        assert!(result.as_float().unwrap().is_nan());
    }
}

#[test]
fn numeric_contracts_mixed_floor_division_preserves_exact_float_value() {
    let mut vm = VM::new_native();
    let floor_div = vm.get_or_intern("~/(_)");
    for (receiver, argument, expected) in [
        (Value::int(9_007_199_254_740_991), Value::float(1.0), 9_007_199_254_740_991),
        (Value::int(9_007_199_254_740_992), Value::float(1.0), 9_007_199_254_740_992),
        (Value::int(9_007_199_254_740_993), Value::float(1.0), 9_007_199_254_740_993),
        (Value::int(-7), Value::float(2.5), -3),
        (Value::int(7), Value::float(-2.5), -3),
    ] {
        let result = vm.send_dynamic(receiver, floor_div, &[argument]).unwrap();
        assert_eq!(result.as_int(), Some(expected));
    }
}

#[test]
fn numeric_contracts_mixed_relations_preserve_exact_integer_order() {
    let mut vm = VM::new_native();
    let cases = [("<", false), ("<=", false), (">", true), (">=", true)];
    for (operator, expected) in cases {
        let selector_name = format!("{operator}(_)");
        let selector = vm.get_or_intern(&selector_name);
        let result = vm
            .send_dynamic(Value::int(9_007_199_254_740_993), selector, &[Value::float(9_007_199_254_740_992.0)])
            .expect("mixed relation should execute");
        assert_eq!(result.as_bool(), Some(expected), "operator {operator}");
    }
}
