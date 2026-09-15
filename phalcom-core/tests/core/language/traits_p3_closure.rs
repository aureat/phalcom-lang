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
fn direct_field_delegation_executes_as_ordinary_accessors() {
    let source = r#"
trait CounterTrait {
  mut count: Int
  doubled -> Int { self.count + self.count }
}

class Counter {
  mut _count: Int
  @constructor new(_ initial: Int) { _count = initial }
}
impl Counter { mut count via _count }
impl CounterTrait for Counter {}

let counter = Counter.new(3)
let before = counter.count
counter.count = 8
let after = counter.count
let doubled = counter.doubled
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("delegated accessors compile");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("delegated accessors execute");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "before").expect("before").to_string(&vm), "3");
    assert_eq!(named(&vm, module, "after").expect("after").to_string(&vm), "8");
    assert_eq!(named(&vm, module, "doubled").expect("doubled").to_string(&vm), "16");
}

#[test]
fn delegated_getter_coexists_with_custom_inherent_setter() {
    let source = r#"
class Counter {
  mut _count: Int
  @constructor new(_ initial: Int) { _count = initial }
  count via _count
}
impl Counter {
  count=(_ value: Int) { _count = value + 1 }
}

let counter = Counter.new(2)
counter.count = 6
let result = counter.count
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("custom setter composition compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("custom setter composition executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result").to_string(&vm), "7");
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

#[test]
fn high_density_c4_stress_program_preserves_exact_evidence_and_runtime_results() {
    let source = r#"
trait Named {
  name -> String
  label -> String { "[" + self.name + "]" }
}
trait DeepLabel {
  name -> String
  label -> String { "[" + self.name + "]" }
  decorated -> String { "<" + self.label + ">" }
}
trait Echo<T> {
  value -> T
}
trait DoubleEcho<T> {
  echo(_ value: T) -> T
  twice(_ value: T) -> T { self.echo(self.echo(value)) }
}
trait Tagged { tag -> String }
trait BoxLabel {
  tag -> String
  label -> String { self.tag }
}
trait Steppable { step() -> Int }

class Base { name -> String { "base" } }
class Derived is Base {}
impl Named for Derived {}
impl DeepLabel for Derived {}

data Person(name: String)
impl Named for Person {}
impl Tagged for Person { tag -> String { "person" } }

data Payload<T>(_ value: T)
impl<T> Echo<T> for Payload<T> {}
class Value<T> { @constructor new() {} }
class Doubler {}
impl DoubleEcho<Int> for Doubler { echo(_ value: Int) -> Int { value * 2 } }
impl Tagged for Value<String> { tag -> String { "text" } }
impl Tagged for Value<Int> { tag -> String { "number" } }

class NumericBase {}
class Numeric is NumericBase {}
class Box<T> { @constructor new() {} }
impl<T> Box<T> where T <: NumericBase { tag -> String { "numeric" } }
impl<T> BoxLabel for Box<T> {}

enum Result<T> { Ok(_ value: T) Error(_ message: String) }
impl Tagged for Result<Int>::Ok(_) { tag -> String { "ok" } }

class Counter {
  _state
  @constructor new(_ initial) {
    _state = initial
  }
  step() -> Int {
    _state = _state + 1
    _state
  }
}
impl Steppable for Counter {}

class Caller {
  run(_ derived: Derived, _ person: Person, _ text: Value<String>, _ number: Value<Int>, _ text_payload: Payload<String>, _ number_payload: Payload<Int>, _ box: Box<Numeric>, _ doubler: Doubler, _ counter: Counter) -> String {
    let step = &counter.step()
    let first = step()
    let second = step()
    "\(derived.decorated)|\(person.label)|\(person.tag)|\(text_payload.value)|\(number_payload.value)|\(doubler.twice(3))|\(text.tag)|\(number.tag)|\(box.label)|\(Result<Int>::Ok(7).tag)|\(first)|\(second)"
  }
}

let result = Caller.new().run(Derived.new(), Person(name: "person"), Value.new(), Value.new(), Payload("payload"), Payload(3), Box.new(), Doubler.new(), Counter.new(0))
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("dense C4 stress program compiles");
    let lowering = &program.modules[&program.entry].lowering;
    assert!(
        lowering.trait_invocations.len() >= 6,
        "stress program must retain multiple trait invocation sites: {}",
        lowering.trait_invocations.len()
    );

    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("dense C4 stress program executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(
        named(&vm, module, "result").expect("result").to_string(&vm),
        "<[base]>|[person]|person|payload|3|12|text|number|numeric|ok|1|2"
    );

    for (class_name, selector_name) in [("Derived", "decorated"), ("Value", "tag"), ("Box", "label")] {
        let class = named(&vm, module, class_name).expect("class binding").as_obj().expect("class object");
        let selector = vm.interner.find(selector_name).expect("selector");
        assert!(
            vm.heap.class(class).get_method(selector).is_none(),
            "trait-only selector {selector_name} must remain detached from {class_name}"
        );
    }
}
