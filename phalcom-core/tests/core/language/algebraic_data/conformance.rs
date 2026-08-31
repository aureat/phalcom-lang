//! Small vertical conformance scenarios spanning semantic projection, lowering, and VM execution.

use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

fn run_inline(source: &str) -> Result<(VM, phalcom_core::heap::ObjRef), PhError> {
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    vm.run_compiled(&program)?;
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;
    Ok((vm, module))
}

#[test]
fn generic_constructor_keeps_runtime_identity_and_payload_without_type_proof() {
    let source = r#"
enum Option<T> {
  @variant Some(_ value: T)
  @variant None
}

let some = Option<Int>::Some(42)
"#;
    let (vm, module) = run_inline(source).expect("generic constructor should execute");
    let some = vm
        .heap
        .module(module)
        .get(vm.interner.find("some").expect("some symbol"))
        .expect("some binding");
    let variant = vm.runtime_variant_of(some).expect("runtime variant");
    let descriptor = vm.adt_registry.variant_descriptor(variant).expect("variant descriptor");
    assert_eq!(descriptor.payload_arity, 1);
    assert_eq!(vm.case_payload_at(some, 0).expect("payload"), Value::int(42));
    assert!(descriptor.singleton.is_none(), "payload case must not become singleton");
}

#[test]
fn gadt_result_type_is_erased_from_runtime_case_representation() {
    let source = r#"
enum Expr<T> {
  @variant IntLit(_ value: Int) -> Expr<Int>
  @variant BoolLit(_ value: Bool) -> Expr<Bool>
}

let expr = Expr<Int>::IntLit(42)
"#;
    let (vm, module) = run_inline(source).expect("GADT constructor should execute");
    let expr = vm
        .heap
        .module(module)
        .get(vm.interner.find("expr").expect("expr symbol"))
        .expect("expr binding");
    let variant = vm.runtime_variant_of(expr).expect("runtime variant");
    let descriptor = vm.adt_registry.variant_descriptor(variant).expect("variant descriptor");
    assert_eq!(descriptor.payload_arity, 1);
    assert_eq!(vm.case_payload_len(expr), Some(1));
    assert_eq!(vm.case_payload_at(expr, 0).expect("payload"), Value::int(42));
    // Runtime descriptor contains identity/layout/payload only; no GADT equality proof or substitution.
    assert!(descriptor.singleton.is_none());
}
