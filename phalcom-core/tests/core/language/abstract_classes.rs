//! Abstract class and declaration-only member execution coverage.

use super::vm_support;
use phalcom_core::value::Value;

fn named(vm: &phalcom_core::vm::VM, module: phalcom_core::heap::ObjRef, name: &str) -> Option<Value> {
    let symbol = vm.interner.find(name)?;
    vm.heap.module(module).get(symbol)
}

#[test]
fn declaration_only_member_makes_class_abstract_and_concrete_override_executes() {
    let source = r#"
class AbstractClock {
  now -> Int
}

class FixedClock is AbstractClock {
  now -> Int { 42 }
}

let result = FixedClock.new().now
"#;

    let (vm, module) = vm_support::run_inline(source).expect("abstract class source should execute");
    assert_eq!(named(&vm, module, "result").expect("result binding").as_int(), Some(42));

    let abstract_clock = named(&vm, module, "AbstractClock")
        .expect("abstract class binding")
        .as_obj()
        .expect("class object");
    assert!(vm.heap.class(abstract_clock).is_abstract);
}

#[test]
fn abstract_class_construction_is_rejected() {
    let source = r#"
class AbstractClock {
  now -> Int
}

let result = AbstractClock.new()
"#;

    let error = match vm_support::run_inline(source) {
        Ok(_) => panic!("abstract class construction must fail"),
        Err(error) => error,
    };
    let message = error.to_string();
    assert!(
        message.contains("abstractClass") || message.contains("cannot instantiate abstract class"),
        "{message}"
    );
}

#[test]
fn subclass_with_unresolved_abstract_member_remains_abstract() {
    let source = r#"
class AbstractClock {
  now -> Int
}

class IncompleteClock is AbstractClock {}
let result = IncompleteClock.new()
"#;

    let error = match vm_support::run_inline(source) {
        Ok(_) => panic!("incomplete concrete subclass construction must fail"),
        Err(error) => error,
    };
    let message = error.to_string();
    assert!(
        message.contains("abstractClass") || message.contains("cannot instantiate abstract class"),
        "{message}"
    );
}
