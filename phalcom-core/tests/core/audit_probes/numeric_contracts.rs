//! Numeric domain and primitive contract audit probes.

use phalcom_core::{value::Value, vm::VM};

#[test]
#[ignore = "AUD-RUNTIME-006: numeric precision loss in mixed integer/float quotient"]
fn numeric_contracts_remainder_and_quotient() {
    let mut vm = VM::new_native();
    let cases: [(&str, Value, &str, Value, Value); 4] = [
        (
            "negative float remainder",
            Value::float(-5.0),
            "%(_)",
            Value::float(3.0),
            Value::float(-2.0),
        ),
        (
            "negative divisor float remainder",
            Value::float(5.0),
            "%(_)",
            Value::float(-3.0),
            Value::float(2.0),
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
        let result = vm
            .send_dynamic(receiver, sym, &[arg])
            .unwrap_or_else(|e| panic!("failed {label}: {e:?}"));
        assert_eq!(result, expected, "{label}");
    }
}
