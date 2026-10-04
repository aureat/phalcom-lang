//! Immutable field initialization runtime checks.

use super::vm_support;

#[test]
fn early_method_read_of_immutable_field_traps_during_construction() {
    let source = r#"
class Cell {
  _value: Int

  @constructor new() {
    observe()
    _value = 1
  }

  observe() -> Int { _value }
}

let cell = Cell.new()
"#;

    let error = match vm_support::run_inline(source) {
        Ok(_) => panic!("early immutable field read must trap"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("immutable field `_value` is read before it is initialized"), "{error}");
}

#[test]
fn declared_default_is_visible_to_immutable_field_reads() {
    let source = r#"
class Cell {
  _value: Int = 7
  read() -> Int { _value }
}

let cell = Cell.new()
let result = cell.read()
"#;

    let (vm, module) = vm_support::run_inline(source).expect("initialized immutable field should be readable");
    let result = vm
        .heap
        .module(module)
        .get(vm.interner.find("result").expect("result binding"))
        .expect("result value");
    assert_eq!(result.as_int(), Some(7));
}
