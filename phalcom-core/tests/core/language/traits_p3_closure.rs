use super::vm_support;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompileError, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_semantic::DiagnosticCode;
use std::sync::Arc;

fn named(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    let symbol = vm.interner.find(name)?;
    vm.heap.module(module).get(symbol)
}

#[test]
fn exact_enum_case_conformance_executes_without_leaking_to_sibling() {
    let source = r#"
trait Tagged { tag -> String }
enum Result<T> {
  Ok(_ value: T)
  Error(_ message: String)
}
impl Tagged for Result<Int>::Ok(_) { tag -> String { "ok" } }
let result = Result<Int>::Ok(42).tag
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("exact-case trait conformance compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("exact-case trait conformance executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result").to_string(&vm), "ok");
}

#[test]
fn generic_source_conformance_executes_for_multiple_exact_targets() {
    let source = r#"
trait Tagged { tag -> String }
class Value<T> { @constructor new() {} }
impl<T> Tagged for Value<T> { tag -> String { "generic" } }
class Caller {
  stringTag(_ value: Value<String>) -> String { value.tag }
  intTag(_ value: Value<Int>) -> String { value.tag }
}
let caller = Caller.new()
let stringValue: Value<String> = Value.new()
let intValue: Value<Int> = Value.new()
let stringResult = caller.stringTag(stringValue)
let intResult = caller.intTag(intValue)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("generic source conformance compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("generic source conformance executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "generic");
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "generic");
}

#[test]
fn generic_trait_bound_reference_survives_forced_gc() {
    let source = r#"
trait Tagged<T> { tag(_ value: T) -> T }
class Identity {}
impl Tagged<Int> for Identity { tag(_ value: Int) -> Int { value } }
class Caller {
  run(_ identity: Identity) -> Int {
    let f = &identity.tag(_)
    System.gc
    f(42)
  }
}
let result = Caller.new().run(Identity.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("generic trait bound reference compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("generic trait bound reference survives GC");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result").to_string(&vm), "42");
}

#[test]
fn competing_trait_defaults_fail_with_dedicated_ambiguity_diagnostic() {
    let source = r#"
trait Pretty { render -> String { "pretty" } }
trait Debuggable { render -> String { "debug" } }
class Item {}
impl Pretty for Item {}
impl Debuggable for Item {}
class Caller { run(_ item: Item) -> String { item.render } }
let result = Caller.new().run(Item.new())
"#;
    let error = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect_err("competing defaults must fail closed");
    let ProgramCompileError::Semantic(diagnostics) = error else {
        panic!("expected semantic ambiguity failure");
    };
    assert!(
        diagnostics
            .iter()
            .flat_map(|(_, diagnostics)| diagnostics.iter())
            .any(|diagnostic| diagnostic.code == DiagnosticCode::TraitDispatchAmbiguous),
        "expected dedicated trait ambiguity diagnostic: {diagnostics:#?}"
    );
}

#[test]
fn shared_concrete_trait_witnesses_converge_to_the_inherent_runtime_member() {
    let source = r#"
trait Pretty { render -> String }
trait Debuggable { render -> String }
class Item { render -> String { "item" } }
impl Pretty for Item {}
impl Debuggable for Item {}
class Caller { run(_ item: Item) -> String { item.render } }
let result = Caller.new().run(Item.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("convergent traits compile");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("convergent traits execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result").to_string(&vm), "item");
}
