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
data Result<T> {
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
fn associated_type_default_executes_for_multiple_exact_generic_targets() {
    let source = r#"
trait Cell {
  type Item
  echo(_ value: Self::Item) -> Self::Item { value }
}
class CellBox<T> { @constructor new() {} }
impl<T> Cell for CellBox<T> { type Item = T }
class Caller {
  stringEcho(_ value: CellBox<String>) -> String { value.echo("text") }
  intEcho(_ value: CellBox<Int>) -> Int { value.echo(42) }
}
let caller = Caller.new()
let stringBox: CellBox<String> = CellBox.new()
let intBox: CellBox<Int> = CellBox.new()
let stringResult = caller.stringEcho(stringBox)
let intResult = caller.intEcho(intBox)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("associated default compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("associated default executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "text");
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "42");

    let selector_name = "echo";
    let class = named(&vm, module, "CellBox").expect("CellBox binding").as_obj().expect("CellBox class");
    if let Some(selector) = vm.interner.find(selector_name) {
        assert!(
            vm.heap.class(class).get_method(selector).is_none(),
            "trait-only selector {selector_name} must remain detached from CellBox"
        );
    }
}

#[test]
fn associated_type_explicit_witness_executes_with_normalized_parameter_and_return() {
    let source = r#"
trait Transformer {
  type Item
  transform(_ value: Self::Item) -> Self::Item
}
class TransformBox<T> { @constructor new() {} }
impl<T> Transformer for TransformBox<T> {
  type Item = T
  transform(_ value: Self::Item) -> Self::Item { value }
}
class Caller {
  stringTransform(_ value: TransformBox<String>) -> String { value.transform("text") }
  intTransform(_ value: TransformBox<Int>) -> Int { value.transform(42) }
}
let caller = Caller.new()
let stringBox: TransformBox<String> = TransformBox.new()
let intBox: TransformBox<Int> = TransformBox.new()
let stringResult = caller.stringTransform(stringBox)
let intResult = caller.intTransform(intBox)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("associated witness compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("associated witness executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "text");
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "42");

    let class = named(&vm, module, "TransformBox")
        .expect("TransformBox binding")
        .as_obj()
        .expect("TransformBox class");
    if let Some(selector) = vm.interner.find("transform") {
        assert!(
            vm.heap.class(class).get_method(selector).is_none(),
            "trait-only selector transform must remain detached from TransformBox"
        );
    }
}

#[test]
fn exact_enum_case_associated_type_executes_without_sibling_leakage() {
    let source = r#"
trait PayloadView {
  type Item
  reveal(_ value: Self::Item) -> Self::Item { value }
}
data Payload<T> {
  Ok(_ value: T)
  Error(_ message: String)
}
impl PayloadView for Payload<Int>::Ok(_) { type Item = Int }
let result = Payload<Int>::Ok(0).reveal(42)
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("exact-case associated conformance compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("exact-case associated conformance executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result").to_string(&vm), "42");
}

#[test]
fn associated_type_bound_callable_survives_forced_gc() {
    let source = r#"
trait Tagged {
  type Item
  tag(_ value: Self::Item) -> Self::Item
}
class Identity {}
impl Tagged for Identity {
  type Item = Int
  tag(_ value: Self::Item) -> Self::Item { value }
}
class Caller {
  run(_ identity: Identity) -> Int {
    let f = &identity.tag(_)
    System.gc
    f(42)
  }
}
let result = Caller.new().run(Identity.new())
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("associated callable reference compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("associated callable reference survives GC");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "result").expect("result").to_string(&vm), "42");
}

#[test]
fn associated_labeled_tuple_binding_preserves_labels_and_values() {
    let source = r#"
trait EntrySource {
  type Item
  first -> Self::Item
}
class PairSource<K, V> {
  _key: K
  _value: V
  @constructor new(_ key: K, _ value: V) {
    _key = key
    _value = value
  }
  key { _key }
  value { _value }
}
impl<K, V> EntrySource for PairSource<K, V> {
  type Item = (key: K, value: V)
  first -> Self::Item { (key: self.key, value: self.value) }
}
let source: PairSource<Int, String> = PairSource.new(7, "text")
let entry = source.first
let key = entry[#key]
let value = entry[#value]
let labels = (entry.labelAt(0) == #key).ifTrue(|| { 1 }, ifFalse: || { 0 })
let labels2 = (entry.labelAt(1) == #value).ifTrue(|| { 1 }, ifFalse: || { 0 })
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("labeled tuple associated binding compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("labeled tuple associated binding executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "key").expect("key").to_string(&vm), "7");
    assert_eq!(named(&vm, module, "value").expect("value").to_string(&vm), "text");
    assert_eq!(named(&vm, module, "labels").expect("first label").to_string(&vm), "1");
    assert_eq!(named(&vm, module, "labels2").expect("second label").to_string(&vm), "1");
}

#[test]
fn concrete_associated_iterator_preserves_state_freshness_identity_and_exhaustion() {
    let source = r#"
trait C5Iterator {
  type Item
  next -> Option<Self::Item>
  iterator { self }
}
trait C5Iterable {
  type Item
  type Iterator
  iterator -> Self::Iterator
}
class C5List<T> {
  _values: List<T>
  @constructor new(_ values: List<T>) { _values = values }
  values { _values }
}
class C5ListIterator<T> {
  _values: List<T>
  _index: Int
  @constructor new(_ values: List<T>) {
    _values = values
    _index = 0
  }
  next -> Option<T> {
    if (_index < _values.size) {
      let value = _values[_index]
      _index = _index + 1
      Some(value)
    } else {
      None
    }
  }
  iterator { self }
}
impl<T> C5Iterator for C5ListIterator<T> { type Item = T }
impl<T> C5Iterable for C5ListIterator<T> {
  type Item = T
  type Iterator = C5ListIterator<T>
}
impl<T> C5Iterable for C5List<T> {
  type Item = T
  type Iterator = C5ListIterator<T>
  iterator -> Self::Iterator { C5ListIterator.new(self.values) }
}
let source: C5List<Int> = C5List.new([1, 2, 3])
let first = source.iterator
let firstOne = first.next
let firstTwo = first.next
let fresh = source.iterator
let freshOne = fresh.next
let exhausted = source.iterator
let exhaustedOne = exhausted.next
let exhaustedTwo = exhausted.next
let exhaustedThree = exhausted.next
let exhaustedNone = exhausted.next
let exhaustedNoneAgain = exhausted.next
let identity = source.iterator
let alias = identity.iterator
let sameIdentity = (identity === alias).ifTrue(|| { 1 }, ifFalse: || { 0 })
let identityOne = identity.next
let aliasTwo = alias.next
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("concrete associated iterator compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("concrete associated iterator executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "firstOne").expect("first one").to_string(&vm), "Some(1)");
    assert_eq!(named(&vm, module, "firstTwo").expect("first two").to_string(&vm), "Some(2)");
    assert_eq!(named(&vm, module, "freshOne").expect("fresh one").to_string(&vm), "Some(1)");
    assert_eq!(named(&vm, module, "exhaustedOne").expect("exhausted one").to_string(&vm), "Some(1)");
    assert_eq!(named(&vm, module, "exhaustedTwo").expect("exhausted two").to_string(&vm), "Some(2)");
    assert_eq!(named(&vm, module, "exhaustedThree").expect("exhausted three").to_string(&vm), "Some(3)");
    assert_eq!(named(&vm, module, "exhaustedNone").expect("exhausted none").to_string(&vm), "None");
    assert_eq!(named(&vm, module, "exhaustedNoneAgain").expect("exhausted none again").to_string(&vm), "None");
    assert_eq!(named(&vm, module, "sameIdentity").expect("same identity").to_string(&vm), "1");
    assert_eq!(named(&vm, module, "identityOne").expect("identity one").to_string(&vm), "Some(1)");
    assert_eq!(named(&vm, module, "aliasTwo").expect("alias two").to_string(&vm), "Some(2)");

    let failing_source = r#"
trait C5Iterator {
  type Item
  next -> Option<Self::Item>
}
class C5FailingIterator {
  _calls: Int
  @constructor new() { _calls = 0 }
  next -> Option<Int> {
    if (_calls == 0) {
      _calls = 1
      Some(1)
    } else {
      throw Error.new("boom")
    }
  }
}
impl C5Iterator for C5FailingIterator { type Item = Int }
let iterator = C5FailingIterator.new()
let first = iterator.next
let second = iterator.next
"#;
    let failing_program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(failing_source))).expect("failing iterator compiles");
    let mut failing_vm = vm_support::universe_vm();
    let error = failing_vm.run_compiled(&failing_program).expect_err("iterator failure must propagate");
    assert!(
        error.to_string().contains("boom"),
        "iterator failure should preserve the original error: {error}"
    );
}

#[test]
fn associated_type_vertical_executes_across_project_modules() {
    let directory = tempfile::tempdir().expect("temporary project directory");
    let source_directory = directory.path().join("src");
    std::fs::create_dir_all(&source_directory).expect("project source directory");
    std::fs::write(
        directory.path().join("project.toml"),
        "[project]\nname = \"c5_vertical\"\nnamespace = \"c5_vertical\"\nversion = \"0.1.0\"\nentry = \"c5_vertical.main\"\n",
    )
    .expect("project manifest");
    std::fs::write(source_directory.join("package.ph"), "expose .cell\nexpose .boxes\nexpose .main\n").expect("package exports");
    std::fs::write(
        source_directory.join("cell.ph"),
        r#"
trait Cell {
  type Item
  echo(_ value: Self::Item) -> Self::Item
}
export Cell
"#,
    )
    .expect("trait module");
    std::fs::write(
        source_directory.join("boxes.ph"),
        r#"
from .cell import Cell
class Box<T> { @constructor new() {} }
impl<T> Cell for Box<T> {
  type Item = T
  echo(_ value: Self::Item) -> Self::Item { value }
}
export Box
"#,
    )
    .expect("conformance module");
    std::fs::write(
        source_directory.join("main.ph"),
        r#"
from .boxes import Box
class Caller {
  stringEcho(_ value: Box<String>) -> String { value.echo("text") }
  intEcho(_ value: Box<Int>) -> Int { value.echo(42) }
}
let caller = Caller.new()
let stringBox: Box<String> = Box.new()
let intBox: Box<Int> = Box.new()
let stringResult = caller.stringEcho(stringBox)
let intResult = caller.intEcho(intBox)
export stringResult, intResult
"#,
    )
    .expect("consumer module");

    let program =
        ProgramCompiler::compile_entry_selection(EntrySelection::Project(directory.path().to_path_buf())).expect("cross-module associated project compiles");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("cross-module associated project executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "text");
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "42");
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
fn high_density_associated_type_stress_composes_projection_products_and_exact_cases() {
    let source = r#"
trait DenseCell {
  type Item
  echo(_ value: Self::Item) -> Self::Item { value }
  transform(_ value: Self::Item) -> Self::Item
  pack(_ value: Self::Item) -> List<Self::Item> { [self.transform(value)] }
}
trait EntrySource {
  type Item
  first -> Self::Item
}
class DenseBox<T> { @constructor new() {} }
impl<T> DenseCell for DenseBox<T> {
  type Item = T
  transform(_ value: Self::Item) -> Self::Item { value }
}
data Payload<T> {
  Ok(_ value: T)
  Error(_ message: String)
}
impl DenseCell for Payload<Int>::Ok(_) {
  type Item = Int
  transform(_ value: Self::Item) -> Self::Item { value * 2 }
}
class PairSource<K, V> {
  _key: K
  _value: V
  @constructor new(_ key: K, _ value: V) {
    _key = key
    _value = value
  }
  key { _key }
  value { _value }
}
impl<K, V> EntrySource for PairSource<K, V> {
  type Item = (key: K, value: V)
  first -> Self::Item { (key: self.key, value: self.value) }
}
class Caller {
  stringPack(_ value: DenseBox<String>) -> String { value.pack("packed")[0] }
  intTransform(_ value: DenseBox<Int>) -> Int { value.transform(21) }
}
let caller = Caller.new()
let stringBox: DenseBox<String> = DenseBox.new()
let intBox: DenseBox<Int> = DenseBox.new()
let stringResult = caller.stringPack(stringBox)
let intResult = caller.intTransform(intBox)
let bound = &stringBox.transform(_)
System.gc
let boundResult = bound("bound")
let payloadResult = Payload<Int>::Ok(3).transform(4)
let entry = PairSource.new(7, "text").first
let entryResult = entry[#key].toString + ":" + entry[#value]
"#;
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).expect("dense C5 program compiles");
    assert!(
        program.modules[&program.entry].lowering.trait_invocations.len() >= 5,
        "dense C5 program must retain multiple trait invocation sites: {}",
        program.modules[&program.entry].lowering.trait_invocations.len()
    );
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("dense C5 program executes");
    let module = vm.module_registry.get(&program.entry).expect("entry module").object;
    assert_eq!(named(&vm, module, "stringResult").expect("string result").to_string(&vm), "packed");
    assert_eq!(named(&vm, module, "intResult").expect("int result").to_string(&vm), "21");
    assert_eq!(named(&vm, module, "boundResult").expect("bound result").to_string(&vm), "bound");
    assert_eq!(named(&vm, module, "payloadResult").expect("payload result").to_string(&vm), "8");
    assert_eq!(named(&vm, module, "entryResult").expect("entry result").to_string(&vm), "7:text");
}

#[test]
fn high_density_trait_conformance_stress_program_preserves_exact_evidence_and_runtime_results() {
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

data Result<T> { Ok(_ value: T) Error(_ message: String) }
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
