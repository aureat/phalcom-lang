//! Language-level generic conformance execution and runtime transport tests.

use super::vm_support;
use phalcom_core::value::Value;
use std::fs;
use std::path::PathBuf;

fn named(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    let symbol = vm.interner.find(name)?;
    vm.heap.module(module).get(symbol)
}

#[test]
fn bounded_generic_function_dispatches_exact_witness_and_default() {
    let source = r#"
trait Printable {
  print -> String
  label -> String { "printable:" + self.print }
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "user" }
}

class Discharger {
  @class
  display<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }

  @class
  display_labeled<T>(_ item: T) -> String
    where T |= Printable
  {
    item.label
  }
}

let u = User.new()
let res1 = Discharger.display(u)
let res2 = Discharger.display_labeled(u)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("bounded generic execution");
    assert_eq!(named(&vm, module, "res1").expect("res1").to_string(&vm), "user");
    assert_eq!(named(&vm, module, "res2").expect("res2").to_string(&vm), "printable:user");
}

#[test]
fn bounded_generic_caller_forwards_evidence_to_bounded_callee_at_runtime() {
    let source = r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "user_forwarded" }
}

class Forwarder {
  @class
  inner<T>(_ value: T) -> String
    where T |= Printable
  {
    value.print
  }

  @class
  outer<T>(_ value: T) -> String
    where T |= Printable
  {
    Forwarder.inner(value)
  }
}

let u = User.new()
let res = Forwarder.outer(u)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("bounded generic forwarding execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "user_forwarded");
}

#[test]
fn multiple_type_parameters_with_distinct_traits_execute() {
    let source = r#"
trait First {
  first -> String
}

trait Second {
  second -> String
}

class ItemA {
  @constructor
  new() {}
}
impl First for ItemA {
  first -> String { "A" }
}

class ItemB {
  @constructor
  new() {}
}
impl Second for ItemB {
  second -> String { "B" }
}

class Combiner {
  @class
  combine<T, U>(_ a: T, _ b: U) -> String
    where T |= First, U |= Second
  {
    a.first + ":" + b.second
  }
}

let a = ItemA.new()
let b = ItemB.new()
let res = Combiner.combine(a, b)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("multiple bounds execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "A:B");
}

#[test]
fn cross_module_bounded_generic_call_executes_selected_evidence() {
    let temp = tempfile::tempdir().expect("temporary project");
    let root = temp.path();
    fs::write(
        root.join("project.toml"),
        "[project]\nname = \"generic_conformance_modules\"\nnamespace = \"generic_conformance_modules\"\nversion = \"0.1.0\"\nentry = \"generic_conformance_modules.main\"\n",
    )
    .expect("project manifest");
    let source = root.join("src");
    fs::create_dir_all(&source).expect("source directory");
    fs::write(source.join("package.ph"), "expose .provider\nexpose .main\n").expect("package manifest");
    fs::write(
        source.join("provider.ph"),
        r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "provider_user" }
}

class Discharger {
  @class
  display<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

export Printable, User, Discharger
"#,
    )
    .expect("provider source");
    fs::write(
        source.join("main.ph"),
        r#"
from .provider import User, Discharger

let user = User.new()
let result = Discharger.display(user)
"#,
    )
    .expect("consumer source");

    let program = phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(
        phalcom_core::modules::compile::EntrySelection::Project(PathBuf::from(root)),
    )
    .expect("cross-module generic project compiles");
    let mut vm = phalcom_core::vm::VM::new();
    vm.run_compiled(&program).expect("cross-module generic project executes");
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;
    assert_eq!(named(&vm, module, "result").expect("result").to_string(&vm), "provider_user");
}

#[test]
fn same_trait_on_distinct_type_parameters_selects_distinct_witnesses() {
    let source = r#"
trait Formatter {
  format -> String
}

class LeftItem {
  @constructor
  new() {}
}
impl Formatter for LeftItem {
  format -> String { "LEFT" }
}

class RightItem {
  @constructor
  new() {}
}
impl Formatter for RightItem {
  format -> String { "RIGHT" }
}

class PairFormatter {
  @class
  format_both<T, U>(_ left: T, _ right: U) -> String
    where T |= Formatter, U |= Formatter
  {
    left.format + "<>" + right.format
  }
}

let l = LeftItem.new()
let r = RightItem.new()
let res = PairFormatter.format_both(l, r)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("same trait on distinct parameters execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "LEFT<>RIGHT");
}

#[test]
#[ignore = "scheduled for T5 owner/receiver evidence composition"]
fn owner_generic_evidence_executes_after_receiver_specialization() {
    let source = r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}
impl Printable for User {
  print -> String { "owner_user" }
}

class Container<T> where T |= Printable {
  _val: T

  @constructor
  new(_ val: T) {
    _val = val
  }

  show() -> String {
    _val.print
  }
}

let u = User.new()
let c = Container.new(u)
let res = c.show()
"#;
    let program = phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(
        phalcom_core::modules::compile::EntrySelection::Inline(std::sync::Arc::from(source)),
    ).expect("owner evidence lowering");
    assert!(!program.modules[&program.entry].lowering.trait_requirement_invocations.is_empty(), "owner body requires canonical trait lowering");
    let mut vm = vm_support::universe_vm();
    vm.run_compiled(&program).expect("owner generic evidence execution");
    let module = vm.module_registry.get(&program.entry).unwrap().object;
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "owner_user");
}

#[test]
#[ignore = "scheduled for T5 owner/receiver evidence composition"]
fn owner_and_method_local_evidence_execute_together() {
    let source = r#"
trait First {
  first -> String
}

trait Second {
  second -> String
}

class ItemA {
  @constructor
  new() {}
}
impl First for ItemA {
  first -> String { "ItemA" }
}

class ItemB {
  @constructor
  new() {}
}
impl Second for ItemB {
  second -> String { "ItemB" }
}

class Wrapper<T> where T |= First {
  _val: T

  @constructor
  new(_ val: T) {
    _val = val
  }

  combine<U>(_ other: U) -> String
    where U |= Second
  {
    _val.first + "::" + other.second
  }
}

let a = ItemA.new()
let b = ItemB.new()
let w = Wrapper.new(a)
let res = w.combine(b)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("owner + method generic evidence execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "ItemA::ItemB");
}

#[test]
#[ignore = "scheduled for exact-enum-case generic bounds in T3-T5"]
fn exact_enum_case_conformance_executes() {
    let source = r#"
trait StatusReport {
  report -> String
}

enum Status {
  @variant Active
  @variant Inactive
}

impl StatusReport for Status::Active {
  report -> String { "active_status" }
}

class Checker {
  @class
  check<T>(_ status: T) -> String
    where T |= StatusReport
  {
    status.report
  }
}

let s = Status::Active
let res = Checker.check(s)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("exact enum case generic execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "active_status");
}

#[test]
fn runtime_conformance_environment_reuses_identical_environments() {
    let source = r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}
impl Printable for User {
  print -> String { "user" }
}

class Discharger {
  @class
  display<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

let u = User.new()
let res1 = Discharger.display(u)
let res2 = Discharger.display(u)
let res3 = Discharger.display(u)
"#;
    let (mut vm, module) = vm_support::run_inline(source).expect("environment reuse test");
    assert_eq!(named(&vm, module, "res1").expect("res1").to_string(&vm), "user");
    assert_eq!(named(&vm, module, "res2").expect("res2").to_string(&vm), "user");
    assert_eq!(named(&vm, module, "res3").expect("res3").to_string(&vm), "user");
    use phalcom_core::typing::RuntimeConformanceEnvironmentId;
    let cardinality = vm.runtime_conformance_environments.len();
    assert!(cardinality > 1, "execution must materialize a nonempty environment");
    for index in 1..cardinality {
        let id = RuntimeConformanceEnvironmentId(index as u32);
        let environment = vm.runtime_conformance_environments.get(id).unwrap().clone();
        assert_eq!(vm.runtime_conformance_environments.intern(environment.clone()), id);
        assert_eq!(vm.runtime_conformance_environments.intern(environment), id);
        assert_eq!(vm.runtime_conformance_environments.len(), cardinality);
    }
}

#[test]
fn owner_generic_evidence_is_forwarded_from_an_enclosing_generic_call() {
    let source = r#"
trait Printable { print -> String }
class User { @constructor new() {} }
impl Printable for User { print -> String { "forwarded_owner" } }
class Container<T> where T |= Printable {
  _val: T
  @constructor new(_ val: T) { _val = val }
  show() -> String { _val.print }
}
class Runner {
  @class run<U>(_ container: Container<U>) -> String where U |= Printable {
    container.show()
  }
}
let c = Container.new(User.new())
let res = Runner.run(c)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("forwarded owner evidence execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "forwarded_owner");
}
