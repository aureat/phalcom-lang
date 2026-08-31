//! ADT / GADT runtime, associated resolution, and family lowering tests (Part 4).

use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

fn run_inline(source: &str) -> Result<(VM, phalcom_core::heap::ObjRef), PhError> {
    let mut vm = VM::new();
    let src: Arc<str> = source.into();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(src)).map_err(PhError::from)?;
    vm.run_compiled(&program)?;
    let entry_id = program.initialization_order.last().expect("entry module");
    let mod_obj = vm.module_registry.get(entry_id).unwrap().object;
    Ok((vm, mod_obj))
}

#[test]
fn enum_singletons_are_canonical_and_constructors_allocate_cases() {
    let src = r#"
enum Status {
  @variant Pending
  @variant Active(code: Int)
}

let p1 = Status::Pending
let p2 = Status::Pending
let a1 = Status::Active(code: 42)
let a2 = Status::Active(code: 42)

let same_singleton = p1 === p2
let diff_constructors = not (a1 === a2)
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let same_singleton_sym = vm.interner.find("same_singleton").unwrap();
    let diff_constructors_sym = vm.interner.find("diff_constructors").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(same_singleton_sym), Some(Value::bool(true)));
    assert_eq!(vm.heap.module(main_mod).get(diff_constructors_sym), Some(Value::bool(true)));
}

#[test]
fn singleton_access_and_nullary_construction_have_distinct_identity_laws() {
    let src = r#"
enum CustomOption {
  @variant None
  @variant Nullary()
}

let s1 = CustomOption::None
let s2 = CustomOption::None
let n1 = CustomOption::Nullary()
let n2 = CustomOption::Nullary()

let singletons_canonical = s1 === s2
let nullaries_distinct = not (n1 === n2)
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let singletons_canonical_sym = vm.interner.find("singletons_canonical").unwrap();
    let nullaries_distinct_sym = vm.interner.find("nullaries_distinct").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(singletons_canonical_sym), Some(Value::bool(true)));
    assert_eq!(vm.heap.module(main_mod).get(nullaries_distinct_sym), Some(Value::bool(true)));
}

#[test]
fn enum_case_payloads_are_readable_by_declared_field() {
    let src = r#"
enum Shape {
  @variant Circle(r: Int)
  @variant Rectangle(w: Int, h: Int)
}

let c = Shape::Circle(r: 10)
let r = Shape::Rectangle(w: 20, h: 30)
let r_w = r.w
let r_h = r.h
let c_r = c.r
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let r_w_sym = vm.interner.find("r_w").unwrap();
    let r_h_sym = vm.interner.find("r_h").unwrap();
    let c_r_sym = vm.interner.find("c_r").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(r_w_sym), Some(Value::int(20)));
    assert_eq!(vm.heap.module(main_mod).get(r_h_sym), Some(Value::int(30)));
    assert_eq!(vm.heap.module(main_mod).get(c_r_sym), Some(Value::int(10)));
}

#[test]
fn enum_root_instantiation_is_rejected() {
    let src = r#"
enum Status {
  @variant Pending
  @variant Active(code: Int)
}

let s = Status.new()
"#;
    let res = run_inline(src);
    assert!(res.is_err());
}

#[test]
fn enum_case_payload_mutation_is_rejected() {
    let src = r#"
enum Status {
  @variant Active(code: Int)
}

let a = Status::Active(code: 42)
a.code = 99
"#;
    let res = run_inline(src);
    assert!(res.is_err());
}

#[test]
fn enum_case_methods_dispatch_through_nested_payloads() {
    let src = r#"
enum Expr {
  @variant Lit(val: Int) {
    eval() -> Int {
      self.val
    }
  }
  @variant Add(left: Expr, right: Expr) {
    eval() -> Int {
      self.left.eval() + self.right.eval()
    }
  }
}

let e1 = Expr::Lit(val: 5)
let e2 = Expr::Lit(val: 10)
let sum = Expr::Add(left: e1, right: e2)
let result = sum.eval()
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let result_sym = vm.interner.find("result").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(result_sym), Some(Value::int(15)));
}

#[test]
fn associated_behavior_invocations_dispatch_normally() {
    let src = r#"
class MathUtils {
  @class add(a: Int, b: Int) -> Int {
    a + b
  }
}

let sum = MathUtils::add(a: 10, b: 20)
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let sum_sym = vm.interner.find("sum").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(sum_sym), Some(Value::int(30)));
}
