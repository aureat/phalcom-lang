//! C3 trait compiler/runtime boundary tests.

use super::vm_support;
use phalcom_core::modules::compile::{EntrySelection, ProgramAnalyzer, ProgramCompiler};
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

#[test]
fn trait_evidenced_default_and_witness_execute_without_target_installation() {
    let source = r#"
trait Identified {
  name -> String
  label -> String { self.name }
}

class User {}
impl Identified for User { name -> String { "user" } }
class Caller { run(_ user: User) -> String { user.label } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("trait evidence compiles");
    let lowering = &program.modules[&program.entry].lowering;
    assert!(!lowering.trait_invocations.is_empty(), "ordinary trait call has lowering evidence");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("trait evidence executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "user");
    let user = named(&vm, module, "User").expect("User binding");
    let user_class = user.as_obj().expect("User class");
    let label = vm.interner.find("label").expect("label symbol");
    assert!(vm.heap.class(user_class).get_method(label).is_none(), "trait default stays detached");
}

#[test]
fn trait_evidenced_bound_reference_executes_detached_default() {
    let source = r#"
trait Identified {
  name -> String
  label -> String { self.name }
}

class User {}
impl Identified for User { name -> String { "user" } }
class Caller { run(_ user: User) -> String { let f = &user.label; f() } }
let result = Caller.new().run(User.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("trait bound reference compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("trait bound reference executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "user");
}

#[test]
fn trait_default_executes_abstract_getter_satisfied_by_data_component() {
    let source = r#"
trait Named {
  name -> String
  label -> String { self.name }
}

data Person(name: String)
impl Named for Person {}
class Caller { run(_ person: Person) -> String { person.label } }
let result = Caller.new().run(Person(name: "data-user"))
"#;
    let analyzed = ProgramAnalyzer::analyze_entry_selection(EntrySelection::Inline(Arc::from(source)))
        .expect("data-component trait source analyzes");
    assert_eq!(analyzed.semantic.data_semantics.data_decls.len(), 1, "semantic snapshot must retain Person data semantics");
    let program = ProgramCompiler::compile_analyzed(&analyzed).expect("data-component trait witness compiles");
    assert_eq!(
        program.modules[&program.entry].lowering.data_decls.len(),
        1,
        "formal module lowering must retain Person data semantics"
    );
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("data-component trait witness executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result binding").to_string(&vm), "data-user");
}
