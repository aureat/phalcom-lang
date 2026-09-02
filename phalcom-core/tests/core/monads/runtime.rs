use super::support::{Fixture, run_inline, runtime_source, slot};
use phalcom_core::value::Value;

/// MON-RUNTIME-00: the exact source executed by the VM must independently pass
/// semantic analysis with no errors or analyzer incidents before runtime is
/// considered meaningful evidence.
#[test]
fn runtime_fixture_is_semantically_valid_before_execution() {
    let source = runtime_source();
    let f = Fixture::new(&source);
    f.assert_no_errors();
}

/// MON-RUNTIME-01..07: the concrete Either monad and generic HKT algorithms
/// execute to the expected primitive observations, including failure-path
/// short-circuit behavior.
#[test]
fn monad_higher_kinded_runtime_surface_produces_expected_values() {
    let source = runtime_source();
    let (vm, module) = run_inline(&source).expect("monads runtime conformance program must compile and execute");

    assert_eq!(slot(&vm, module, "monadMappedRightValue"), Some(Value::int(42)));
    assert_eq!(slot(&vm, module, "monadMappedLeftPreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadMapLeftShortCircuited"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadPureValue"), Some(Value::int(7)));
    assert_eq!(slot(&vm, module, "monadMap2Value"), Some(Value::int(3)));
    assert_eq!(slot(&vm, module, "monadMap2FailurePreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadMap2LeftShortCircuited"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadMap2RightFailurePreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadMap2RightShortCircuited"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadFlatMapValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadFlatMapFailurePreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadFlatMapShortCircuited"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeKleisliValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeTraverseSuccessValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeTraverseFailurePreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeTraverseShortCircuited"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeAll"), Some(Value::bool(true)));
}
