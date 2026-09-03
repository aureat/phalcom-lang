use super::super::support::{expression_runtime_source as runtime_source, run_inline, slot, Fixture};
use phalcom_core::value::Value;

/// INT-RUNTIME-00: runtime source must pass semantic preflight before VM
/// observations are accepted as evidence.
#[test]
fn runtime_fixture_is_semantically_valid_before_execution() {
    let source = runtime_source();
    let f = Fixture::new(&source);
    f.assert_no_errors();
}

/// INT-RUNTIME-01..05: Expression evaluation over Either preserves success,
/// higher-order construction, Lift behavior, and failure short-circuiting.
#[test]
fn expression_runtime_surface_produces_expected_values() {
    let source = runtime_source();
    let (vm, module) = run_inline(&source).expect("Expression runtime conformance program must compile and execute");

    assert_eq!(slot(&vm, module, "expressionPureValue"), Some(Value::int(41)));
    assert_eq!(slot(&vm, module, "expressionAddValue"), Some(Value::int(42)));
    assert_eq!(slot(&vm, module, "expressionMapValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "expressionApplyValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "expressionLiftValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "expressionFailurePreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "expressionFailureShortCircuited"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "expressionTraverseValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "expressionRuntimeAll"), Some(Value::bool(true)));
}
