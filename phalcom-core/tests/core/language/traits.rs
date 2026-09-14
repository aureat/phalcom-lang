//! C3 trait compiler/runtime boundary tests.

use super::vm_support;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use std::sync::Arc;

fn named(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    let symbol = vm.interner.find(name)?;
    vm.heap.module(module).get(symbol)
}

#[test]
fn trait_declaration_is_compile_time_only_and_surrounding_code_executes() {
    let source = r#"
trait Display {
  render() -> String { "trait" }
}

let result = "surrounding"
"#;
    let mut vm = vm_support::universe_vm();
    let class_count = vm.classes.len();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("trait source compiles");
    vm.run_compiled(&program).expect("trait-containing module executes");
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;

    assert_eq!(vm.classes.len(), class_count, "trait must not allocate a runtime class");
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "surrounding");
    assert!(named(&vm, module, "Display").is_none(), "trait must not bind an ordinary runtime value");
}

#[test]
fn trait_default_is_not_injected_into_an_unrelated_runtime_class() {
    let source = r#"
trait Display {
  render() -> String { "trait" }
}

class Other {
  render() -> String { "other" }
}

let result = Other.new().render()
"#;
    let (vm, module) = vm_support::run_inline(source).expect("trait and unrelated class execute");
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "other");
}
