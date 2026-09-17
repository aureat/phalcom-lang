//! Language-level guarded derived conformance execution and runtime transport tests (LANG005.C6.P4 T6 / CV-40..46).

use super::vm_support;
use phalcom_core::value::Value;

fn named(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    let symbol = vm.interner.find(name)?;
    vm.heap.module(module).get(symbol)
}

#[test]
fn cv40_iterator_like_derived_iterable_iterator_preserves_identity() {
    let source = r#"
trait Stepper {
  type Item
  step -> Int
}

trait Stream {
  type Item
  type Iterator
  iterator -> Self::Iterator
}

class CounterIter {
  _val: Int
  @constructor
  new(_ v: Int) {
    _val = v
  }
}

impl Stepper for CounterIter {
  type Item = Int
  step -> Int { 42 }
}

impl<I> Stream for I
  where I |= Stepper
{
  type Item = <I as Stepper>::Item
  type Iterator = I
  iterator -> I {
    self
  }
}

class Probe {
  @class
  same<I>(_ iter: I) -> Bool
    where I |= Stream
  {
    iter.iterator === iter
  }
}

let c = CounterIter.new(10)
let res = Probe.same(c)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("derived conformance identity execution");
    assert_eq!(named(&vm, module, "res").expect("res").as_bool(), Some(true));
}

#[test]
fn cv41_generic_callee_requiring_iterable_accepts_iterator_derived_evidence() {
    let source = r#"
trait Stepper {
  type Item
  step -> Int
}

trait Stream {
  type Item
  type Iterator
  iterator -> Self::Iterator
}

class CounterIter {
  @constructor
  new() {}
}

impl Stepper for CounterIter {
  type Item = Int
  step -> Int { 99 }
}

impl<I> Stream for I
  where I |= Stepper
{
  type Item = <I as Stepper>::Item
  type Iterator = I
  iterator -> I {
    self
  }
}

class Consumer {
  @class
  drain<S>(_ stream: S) -> String
    where S |= Stream
  {
    "drained"
  }

  @class
  forward<T>(_ iter: T) -> String
    where T |= Stepper
  {
    Consumer.drain(iter)
  }
}

let c = CounterIter.new()
let res = Consumer.forward(c)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("generic callee accepts derived evidence");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "drained");
}

#[test]
fn cv42_guarded_derived_witness_executes_with_type_environment() {
    let source = r#"
trait Stepper {
  type Item
  step -> String
}

trait Stream {
  type Item
  type Iterator
  iterator -> Self::Iterator
}

class TokenIter {
  _val: String
  @constructor
  new(_ v: String) {
    _val = v
  }
  val -> String { _val }
}

impl Stepper for TokenIter {
  type Item = String
  step -> String { self.val }
}

impl<I> Stream for I
  where I |= Stepper
{
  type Item = <I as Stepper>::Item
  type Iterator = I
  iterator -> I {
    self
  }
}

class Invoker {
  @class
  run<S>(_ stream: S) -> String
    where S |= Stream
  {
    let it = stream.iterator
    "ok"
  }
}

let t = TokenIter.new("abc")
let res = Invoker.run(t)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("derived witness type env execution");
    assert_eq!(named(&vm, module, "res").expect("res").to_string(&vm), "ok");
}

#[test]
fn cv43_guarded_derived_witness_executes_with_nested_conformance_environment() {
    let source = r#"
trait Stepper {
  type Item
  step -> String
}

trait Stream {
  type Item
  type Iterator
  iterator -> Self::Iterator
}

class MyIter {
  @constructor
  new() {}
}

impl Stepper for MyIter {
  type Item = String
  step -> String { "stepped" }
}

class Helper {
  @class
  read<T>(_ s: T) -> String
    where T |= Stepper
  {
    s.step
  }
}

impl<I> Stream for I
  where I |= Stepper
{
  type Item = <I as Stepper>::Item
  type Iterator = I
  iterator -> I {
    let msg = Helper.read(self)
    self
  }
}

class Discharger {
  @class
  run<S>(_ stream: S) -> Bool
    where S |= Stream
  {
    let it = stream.iterator
    it === stream
  }
}

let m = MyIter.new()
let res = Discharger.run(m)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("nested conformance env execution");
    assert_eq!(named(&vm, module, "res").expect("res").as_bool(), Some(true));
}

#[test]
fn cv44_repeated_guarded_derived_calls_reuse_runtime_environments() {
    let source = r#"
trait Stepper {
  type Item
  step -> Int
}

trait Stream {
  type Item
  type Iterator
  iterator -> Self::Iterator
}

class CounterIter {
  @constructor
  new() {}
}

impl Stepper for CounterIter {
  type Item = Int
  step -> Int { 1 }
}

impl<I> Stream for I
  where I |= Stepper
{
  type Item = <I as Stepper>::Item
  type Iterator = I
  iterator -> I {
    self
  }
}

class Discharger {
  @class
  run<S>(_ stream: S) -> Bool
    where S |= Stream
  {
    let it = stream.iterator
    it === stream
  }
}

let c1 = CounterIter.new()
let c2 = CounterIter.new()
let r1 = Discharger.run(c1)
let r2 = Discharger.run(c2)
let r3 = Discharger.run(c1)
"#;
    let (vm, module) = vm_support::run_inline(source).expect("environment reuse execution");
    assert_eq!(named(&vm, module, "r1").expect("r1").as_bool(), Some(true));
    assert_eq!(named(&vm, module, "r2").expect("r2").as_bool(), Some(true));
    assert_eq!(named(&vm, module, "r3").expect("r3").as_bool(), Some(true));
}

#[test]
fn cv45_guarded_derived_evidence_survives_gc_before_deferred_invocation() {
    let source = r#"
trait Stepper {
  type Item
  step -> String
}

trait Stream {
  type Item
  type Iterator
  iterator -> Self::Iterator
}

class CounterIter {
  _v: String
  @constructor
  new(_ v: String) {
    _v = v
  }
  v -> String { _v }
}

impl Stepper for CounterIter {
  type Item = String
  step -> String { self.v }
}

impl<I> Stream for I
  where I |= Stepper
{
  type Item = <I as Stepper>::Item
  type Iterator = I
  iterator -> I {
    self
  }
}

class Discharger {
  @class
  run<S>(_ stream: S) -> Bool
    where S |= Stream
  {
    let it = stream.iterator
    it === stream
  }
}

let c = CounterIter.new("gc_preserved")
let res = Discharger.run(c)
"#;
    let (mut vm, module) = vm_support::run_inline(source).expect("gc execution");
    vm.force_gc();
    assert_eq!(named(&vm, module, "res").expect("res").as_bool(), Some(true));
}

#[test]
fn cv46_vm_executes_guarded_derived_evidence_without_rule_solver() {
    let source = r#"
trait Stepper {
  type Item
  step -> Int
}

trait Stream {
  type Item
  type Iterator
  iterator -> Self::Iterator
}

class Iter {
  @constructor
  new() {}
}

impl Stepper for Iter {
  type Item = Int
  step -> Int { 100 }
}

impl<I> Stream for I
  where I |= Stepper
{
  type Item = <I as Stepper>::Item
  type Iterator = I
  iterator -> I {
    self
  }
}

class Probe {
  @class
  check<S>(_ stream: S) -> Bool
    where S |= Stream
  {
    let it = stream.iterator
    it === stream
  }
}

let res = Probe.check(Iter.new())
"#;
    let (vm, module) = vm_support::run_inline(source).expect("vm execution without solver");
    assert_eq!(named(&vm, module, "res").expect("res").as_bool(), Some(true));
}
