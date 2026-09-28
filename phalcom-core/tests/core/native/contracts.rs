//! Runtime and generated-metadata contracts for fixed-return `System` natives.

use super::vm_support::native_vm;
use phalcom_core::primitive::string::string_raw_byte_at;
use phalcom_core::primitive::system::{system_class_print, system_gc};
use phalcom_core::primitive::time::{system_clock_monotonic_nanoseconds, system_wall_clock_unix_nanoseconds};
use phalcom_core::value::Value;
use phalcom_native_meta::{EffectSpec, NativeEffect, ParameterTupleSpec, TypeExprSpec, UniverseKey};
use phalcom_native_surface::NATIVE_SURFACES;

static EMPTY_TUPLE_PARAMS: ParameterTupleSpec = ParameterTupleSpec {
    positional: &[],
    labeled: &[],
    rest: None,
};
static EMPTY_TUPLE_TYPE: TypeExprSpec = TypeExprSpec::Tuple(&EMPTY_TUPLE_PARAMS);

fn native_record(selector: &str) -> &'static phalcom_native_surface::NativeSurfaceRecord {
    NATIVE_SURFACES
        .iter()
        .find(|record| record.owner() == UniverseKey::System && record.selector() == selector)
        .unwrap_or_else(|| panic!("missing System native {selector}"))
}

fn string_native_record(selector: &str) -> &'static phalcom_native_surface::NativeSurfaceRecord {
    NATIVE_SURFACES
        .iter()
        .find(|record| record.owner() == UniverseKey::String && record.selector() == selector)
        .unwrap_or_else(|| panic!("missing String native {selector}"))
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
    let unit_type = &EMPTY_TUPLE_TYPE;
    assert_eq!(print.returns(), unit_type);
    assert_eq!(print.callable().return_type, unit_type);

    let gc = native_record("gc");
    assert_eq!(gc.returns(), unit_type);
    assert_eq!(gc.callable().return_type, unit_type);
}

#[test]
fn unit_alias_is_not_a_runtime_class_catalog_entry() {
    let vm = native_vm();
    assert!(vm.resolve_builtin_class_name("Unit").is_none(), "Unit is a semantic alias, not a runtime class");
    assert_eq!(vm.resolve_builtin_class_name("Tuple"), Some(vm.universe.classes.tuple_class));
}

#[test]
fn string_raw_byte_at_publishes_bare_int_or_none_contract() {
    let raw_byte_at = string_native_record("rawByteAt(_)");
    assert_eq!(raw_byte_at.returns(), &TypeExprSpec::Universe(UniverseKey::Int));
    assert_eq!(raw_byte_at.callable().return_type, &TypeExprSpec::Universe(UniverseKey::Int));
}

#[test]
fn string_raw_byte_at_runtime_returns_int_or_none() {
    let mut vm = native_vm();
    let string = vm.alloc_string_value("A".to_owned());

    let in_range = string_raw_byte_at(&mut vm, &string, &[Value::int(0)]).expect("String.rawByteAt(0)");
    assert_eq!(in_range.as_int(), Some(65));

    let out_of_range = string_raw_byte_at(&mut vm, &string, &[Value::int(1)]).expect("String.rawByteAt(1)");
    assert!(out_of_range.is_none(), "out-of-range byte access must return None, got {out_of_range:?}");
}

#[test]
fn time_monotonic_native_metadata_is_owned_by_system_clock() {
    let system_clock = NATIVE_SURFACES
        .iter()
        .find(|record| record.owner() == UniverseKey::SystemClock && record.selector() == "monotonicNanoseconds")
        .expect("missing SystemClock monotonic native");
    assert_eq!(system_clock.returns(), &TypeExprSpec::Universe(UniverseKey::Int));
    assert_eq!(system_clock.effects(), EffectSpec::Known(&[NativeEffect::Nondeterminism]));
    assert!(
        !NATIVE_SURFACES
            .iter()
            .any(|record| record.owner() == UniverseKey::System && record.selector() == "monotonicNanoseconds")
    );
}

#[test]
fn time_monotonic_native_runtime_returns_int() {
    let mut vm = native_vm();
    let result = system_clock_monotonic_nanoseconds(&mut vm, &Value::none(), &[]).expect("SystemClock.monotonicNanoseconds");
    assert!(result.as_int().is_some(), "monotonic observation must return Int, got {result:?}");
}

#[test]
fn time_wall_native_metadata_is_owned_by_system_wall_clock() {
    let wall_clock = NATIVE_SURFACES
        .iter()
        .find(|record| record.owner() == UniverseKey::SystemWallClock && record.selector() == "unixNanoseconds")
        .expect("missing SystemWallClock wall-time native");
    assert_eq!(wall_clock.returns(), &TypeExprSpec::Universe(UniverseKey::Int));
    assert_eq!(wall_clock.effects(), EffectSpec::Known(&[NativeEffect::Nondeterminism]));
    assert!(
        !NATIVE_SURFACES
            .iter()
            .any(|record| record.owner() == UniverseKey::System && record.selector() == "unixNanoseconds")
    );
    assert!(
        !NATIVE_SURFACES
            .iter()
            .any(|record| record.selector() == "now" && record.owner() == UniverseKey::SystemWallClock)
    );
}

#[test]
fn time_wall_native_runtime_returns_int() {
    let mut vm = native_vm();
    let result = system_wall_clock_unix_nanoseconds(&mut vm, &Value::none(), &[]).expect("SystemWallClock.unixNanoseconds");
    assert!(result.as_int().is_some(), "wall-time observation must return Int, got {result:?}");
}
