//! Runtime and generated-metadata contracts for fixed-return `System` natives.

use super::vm_support::native_vm;
use phalcom_core::primitive::system::{system_class_print, system_gc};
use phalcom_core::value::Value;
use phalcom_native_meta::{TypeExprSpec, UniverseKey};
use phalcom_native_surface::NATIVE_SURFACES;

fn native_record(selector: &str) -> &'static phalcom_native_surface::NativeSurfaceRecord {
    NATIVE_SURFACES
        .iter()
        .find(|record| record.owner() == UniverseKey::System && record.selector() == selector)
        .unwrap_or_else(|| panic!("missing System native {selector}"))
}

#[test]
fn system_print_runtime_returns_unit() {
    let mut vm = native_vm();
    let result = system_class_print(&mut vm, &Value::int(0), &[Value::int(1)]).expect("System.print");
    assert!(result.is_unit(), "System.print must return Unit, got {result:?}");
}

#[test]
fn system_gc_runtime_returns_unit() {
    let mut vm = native_vm();
    let system = Value::obj(vm.universe.classes.system_class);
    let result = system_gc(&mut vm, &system, &[]).expect("System.gc");
    assert_eq!(result, Value::unit());
}

#[test]
fn system_native_metadata_matches_language_contracts() {
    let print = native_record("print(_)");
    assert_eq!(print.returns(), &TypeExprSpec::Universe(UniverseKey::Unit));
    assert_eq!(print.callable().return_type, &TypeExprSpec::Universe(UniverseKey::Unit));

    let gc = native_record("gc");
    assert_eq!(gc.returns(), &TypeExprSpec::Universe(UniverseKey::Unit));
    assert_eq!(gc.callable().return_type, &TypeExprSpec::Universe(UniverseKey::Unit));
}
