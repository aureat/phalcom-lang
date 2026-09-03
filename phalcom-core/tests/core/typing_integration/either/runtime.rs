use super::super::support::{either_runtime_source as runtime_source, run_inline, slot};
use phalcom_core::value::Value;

/// GEN-51/52/53: static generic operations must execute to the correct variants and primitive payload observations.
#[test]
fn either_runtime_surface_produces_expected_values() {
    let source = runtime_source();
    let (vm, module) = run_inline(&source).expect("Either runtime conformance program must compile and execute");

    assert_eq!(slot(&vm, module, "runtimeLeftIsLeft"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeLeftIsRight"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "runtimeRightIsLeft"), Some(Value::bool(false)));
    assert_eq!(slot(&vm, module, "runtimeRightIsRight"), Some(Value::bool(true)));

    assert_eq!(slot(&vm, module, "mappedRightValue"), Some(Value::int(42)));
    assert_eq!(slot(&vm, module, "mappedLeftPreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "mappedLeftSideValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "bimapRightValue"), Some(Value::int(42)));
    assert_eq!(slot(&vm, module, "flatMappedValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "flatMappedLeftPreserved"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "swappedLeftValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "swappedRightValue"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "fallbackValue"), Some(Value::int(99)));
    assert_eq!(slot(&vm, module, "preservedValue"), Some(Value::int(41)));
    assert_eq!(slot(&vm, module, "recoveredValue"), Some(Value::int(77)));
    assert_eq!(slot(&vm, module, "unrecoveredValue"), Some(Value::int(41)));
    assert_eq!(slot(&vm, module, "orElseLeftValue"), Some(Value::int(100)));
    assert_eq!(slot(&vm, module, "orElseRightValue"), Some(Value::int(41)));
    assert_eq!(slot(&vm, module, "zipValue"), Some(Value::int(41)));
    assert_eq!(slot(&vm, module, "runtimeAll"), Some(Value::bool(true)));
}
