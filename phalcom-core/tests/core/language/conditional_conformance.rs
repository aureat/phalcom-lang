//! Language-level conditional conformance execution and runtime transport tests.

use super::vm_support;
use phalcom_core::value::Value;

fn named(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    let symbol = vm.interner.find(name)?;
    vm.heap.module(module).get(symbol)
}

#[test]
fn cv45_runtime_concrete_conditional_witness_executes() {
    let source = r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "boxed_user" }
}

class Box<T> {
  _value: T
  @constructor
  new(_ v: T) {
    _value = v
  }
}

impl<T> Printable for Box<T>
  where T |= Printable
{
  print -> String { "boxed_user" }
}

class Discharger {
  @class
  display<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

let b = Box.new(User.new())
let res = Discharger.display(b)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("conditional generic execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "boxed_user");
}

#[test]
fn cv46_runtime_conditional_default_executes() {
    let source = r#"
trait Printable {
  print -> String
  label -> String { "default:" + self.print }
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "user" }
}

class Box<T> {
  _value: T
  @constructor
  new(_ v: T) {
    _value = v
  }
}

impl<T> Printable for Box<T>
  where T |= Printable
{
  print -> String { "box" }
}

class Discharger {
  @class
  display_label<T>(_ item: T) -> String
    where T |= Printable
  {
    item.label
  }
}

let b = Box.new(User.new())
let res = Discharger.display_label(b)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("conditional default execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "default:box");
}

#[test]
fn cv47_nested_runtime_evidence_invokes_nested_behavior() {
    let source = r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "inner_user" }
}

class Box<T> {
  _value: T
  @constructor
  new(_ v: T) {
    _value = v
  }
  value -> T {
    _value
  }
}

class Forwarder {
  @class
  show<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

impl<T> Printable for Box<T>
  where T |= Printable
{
  print -> String {
    "box[" + Forwarder.show(self.value) + "]"
  }
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
let b = Box.new(u)
let res = Discharger.display(b)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("nested evidence execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "box[inner_user]");
}

#[test]
fn cv48_generic_assumed_nested_evidence() {
    let source = r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "assumed_user" }
}

class Box<T> {
  _value: T
  @constructor
  new(_ v: T) {
    _value = v
  }
  value -> T {
    _value
  }
}

class Helper {
  @class
  call_print<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

impl<T> Printable for Box<T>
  where T |= Printable
{
  print -> String {
    "derived(" + Helper.call_print(self.value) + ")"
  }
}

class Wrapper {
  @class
  wrap_and_display<T>(_ item: T) -> String
    where T |= Printable
  {
    let b = Box.new(item)
    Helper.call_print(b)
  }
}

let u = User.new()
let res = Wrapper.wrap_and_display(u)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("generic assumed evidence execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "derived(assumed_user)");
}

#[test]
fn cv49_runtime_type_env_receives_impl_substitution() {
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

class Box<T> {
  _value: T
  @constructor
  new(_ v: T) {
    _value = v
  }
  value -> T {
    _value
  }
}

impl<T> Printable for Box<T>
  where T |= Printable
{
  print -> String { "box_type_ok" }
}

class Consumer {
  @class
  run<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

let b = Box.new(User.new())
let res = Consumer.run(b)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("type env substitution execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "box_type_ok");
}

#[test]
fn cv50_slot_isolation_multiple_evidence_values() {
    let source = r#"
trait First {
  first -> String
}

trait Second {
  second -> String
}

class Alpha {
  @constructor
  new() {}
}
impl First for Alpha {
  first -> String { "alpha" }
}

class Beta {
  @constructor
  new() {}
}
impl Second for Beta {
  second -> String { "beta" }
}

class Pair<A, B> {
  _a: A
  _b: B
  @constructor
  new(_ a: A, _ b: B) {
    _a = a
    _b = b
  }
  a -> A { _a }
  b -> B { _b }
}

trait Combined {
  combine -> String
}

class Forwarder {
  @class
  f<T>(_ item: T) -> String where T |= First { item.first }
  @class
  s<T>(_ item: T) -> String where T |= Second { item.second }
}

impl<A, B> Combined for Pair<A, B>
  where A |= First, B |= Second
{
  combine -> String {
    Forwarder.f(self.a) + "+" + Forwarder.s(self.b)
  }
}

class Runner {
  @class
  run<T>(_ item: T) -> String
    where T |= Combined
  {
    item.combine
  }
}

let p = Pair.new(Alpha.new(), Beta.new())
let res = Runner.run(p)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("slot isolation execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "alpha+beta");
}

#[test]
fn cv51_environment_reuse_repeated_invocations() {
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

class Box<T> {
  _value: T
  @constructor
  new(_ v: T) {
    _value = v
  }
}

impl<T> Printable for Box<T>
  where T |= Printable
{
  print -> String { "box" }
}

class Caller {
  @class
  run<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

const b1 = Box.new(User.new())
const b2 = Box.new(User.new())
let r1 = Caller.run(b1)
let r2 = Caller.run(b2)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("environment reuse execution");
    assert_eq!(named(&vm, module, "r1").expect("r1").to_string(&vm), "box");
    assert_eq!(named(&vm, module, "r2").expect("r2").to_string(&vm), "box");
}

#[test]
fn cv52_runtime_anti_authority_no_solver_in_vm() {
    // Proves VM executes already-prepared evidence without runtime constraint solver
    let source = r#"
trait Printable {
  print -> String
}

class Item {
  @constructor
  new() {}
}

impl Printable for Item {
  print -> String { "item" }
}

class Discharger {
  @class
  run<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

let res = Discharger.run(Item.new())
"#;
    let (vm, module) = vm_support::run_inline(source).expect("anti-authority execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "item");
}

#[test]
fn cv73_runtime_rooting_nested_evidence_survives_gc() {
    let source = r#"
trait Printable {
  print -> String
}

class User {
  @constructor
  new() {}
}

impl Printable for User {
  print -> String { "user_gc" }
}

class Box<T> {
  _value: T
  @constructor
  new(_ v: T) {
    _value = v
  }
  value -> T {
    _value
  }
}

class Forwarder {
  @class
  show<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

impl<T> Printable for Box<T>
  where T |= Printable
{
  print -> String {
    Forwarder.show(self.value)
  }
}

class Consumer {
  @class
  run<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

let b = Box.new(User.new())
let res = Consumer.run(b)
"#;
    let (mut vm, module) = vm_support::run_inline(source).expect("gc execution");
    // Run full collection cycle
    vm.force_gc();
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "user_gc");
}

#[test]
fn cv59_cross_module_conditional_witness_executes() {
    let temp = tempfile::tempdir().expect("temporary project");
    let root = temp.path();
    std::fs::write(
        root.join("project.toml"),
        "[project]\nname = \"conditional_conformance_modules\"\nnamespace = \"conditional_conformance_modules\"\nversion = \"0.1.0\"\nentry = \"conditional_conformance_modules.main\"\n",
    )
    .expect("project manifest");
    let source = root.join("src");
    std::fs::create_dir_all(&source).expect("source directory");
    std::fs::write(source.join("package.ph"), "expose .provider\nexpose .main\n").expect("package manifest");
    std::fs::write(
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

class Box<T> {
  _value: T
  @constructor
  new(_ v: T) {
    _value = v
  }
}

impl<T> Printable for Box<T>
  where T |= Printable
{
  print -> String { "provider_box" }
}

class Discharger {
  @class
  display<T>(_ item: T) -> String
    where T |= Printable
  {
    item.print
  }
}

export Printable, User, Box, Discharger
"#,
    )
    .expect("provider source");
    std::fs::write(
        source.join("main.ph"),
        r#"
from .provider import User, Box, Discharger

let b = Box.new(User.new())
let result = Discharger.display(b)
"#,
    )
    .expect("consumer source");

    let program = phalcom_core::modules::compile::ProgramCompiler::compile_entry_selection(phalcom_core::modules::compile::EntrySelection::Project(
        std::path::PathBuf::from(root),
    ))
    .expect("cross-module conditional project compiles");
    let mut vm = phalcom_core::vm::VM::new();
    vm.run_compiled(&program).expect("cross-module conditional project executes");
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;
    assert_eq!(named(&vm, module, "result").expect("result").to_string(&vm), "provider_box");
}
