use super::support::{run_inline, runtime_source, slot};
use phalcom_core::value::Value;

/// MON-RUNTIME-01..06: the concrete Either monad and the generic HKT algorithms
/// must execute to the expected observable primitive values.
#[test]
fn monad_higher_kinded_runtime_surface_produces_expected_values() {
    let source = runtime_source();
    let (vm, module) = run_inline(&source).expect("monads runtime conformance program must compile and execute");

    assert_eq!(slot(&vm, module, "monadMappedRightValue"), Some(Value::int(42)));
    assert_eq!(slot(&vm, module, "monadMappedLeftPreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadPureValue"), Some(Value::int(7)));
    assert_eq!(slot(&vm, module, "monadMap2Value"), Some(Value::int(3)));
    assert_eq!(slot(&vm, module, "monadMap2FailurePreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadFlatMapValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "monadFlatMapFailurePreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeKleisliValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeTraverseSuccessValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeTraverseFailurePreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeAll"), Some(Value::bool(true)));
}
