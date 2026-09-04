use super::super::support::{row_runtime_source, run_inline, slot, Fixture};
use phalcom_core::value::Value;

/// ROW-RUNTIME-01 & ROW-RUNTIME-02: semantic preflight and runtime observation of row-polymorphic pipelines.
#[test]
fn row_runtime_pipeline_executes_with_expected_observations() {
    let source = row_runtime_source();
    let fixture = Fixture::new(&source);
    fixture.assert_no_errors();

    let (vm, module) = run_inline(&source).expect("row runtime program must compile and execute");
    assert_eq!(slot(&vm, module, "runtimePreservedValue"), Some(Value::int(42)));
    assert_eq!(slot(&vm, module, "runtimeCachedField"), Some(Value::bool(true)));
    assert_eq!(slot(&vm, module, "runtimeTagIsEntity"), Some(Value::bool(true)));
}
