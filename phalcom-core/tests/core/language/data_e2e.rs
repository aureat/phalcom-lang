//! End-to-end integration tests for first-class data declarations (LANG005.C1.P1).

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
fn data_positional_tuple_construction_and_component_access() {
    let src = r#"
data Point(_ x: Int, _ y: Int)

let p = Point(10, 20)
let px = p.x
let py = p.y
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let px_sym = vm.interner.find("px").unwrap();
    let py_sym = vm.interner.find("py").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(px_sym), Some(Value::int(10)));
    assert_eq!(vm.heap.module(main_mod).get(py_sym), Some(Value::int(20)));
}

#[test]
fn data_labeled_tuple_construction_and_access() {
    let src = r#"
data Point(x: Int, y: Int)

let p = Point(x: 10, y: 20)
let px = p.x
let py = p.y
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let px_sym = vm.interner.find("px").unwrap();
    let py_sym = vm.interner.find("py").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(px_sym), Some(Value::int(10)));
    assert_eq!(vm.heap.module(main_mod).get(py_sym), Some(Value::int(20)));
}

#[test]
fn data_record_construction_and_access() {
    let src = r#"
data User {
  id: Int,
  name: String
}

let u1 = User { id: 42, name: "Alice" }
let u2 = User { name: "Bob", id: 99 }

let u1_id = u1.id
let u1_name = u1.name
let u2_id = u2.id
let u2_name = u2.name
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let u1_id_sym = vm.interner.find("u1_id").unwrap();
    let u2_id_sym = vm.interner.find("u2_id").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(u1_id_sym), Some(Value::int(42)));
    assert_eq!(vm.heap.module(main_mod).get(u2_id_sym), Some(Value::int(99)));
}

#[test]
fn data_exact_sameness_value_semantics() {
    let src = r#"
data Point(_ x: Int, _ y: Int)

let p1 = Point(1, 2)
let p2 = Point(1, 2)
let p3 = Point(1, 3)

let eq12 = p1 === p2
let eq13 = p1 === p3
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let eq12_sym = vm.interner.find("eq12").unwrap();
    let eq13_sym = vm.interner.find("eq13").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(eq12_sym), Some(Value::bool(true)));
    assert_eq!(vm.heap.module(main_mod).get(eq13_sym), Some(Value::bool(false)));
}

#[test]
fn data_nullary_singleton_identity() {
    let src = r#"
data Empty()

let e1 = Empty()
let e2 = Empty()

let eq = e1 === e2
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let eq_sym = vm.interner.find("eq").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(eq_sym), Some(Value::bool(true)));
}

#[test]
fn data_nested_product_composition() {
    let src = r#"
data Point(_ x: Int, _ y: Int)
data Segment(_ start: Point, _ end: Point)

let s = Segment(Point(0, 1), Point(10, 20))
let sx = s.start.x
let sy = s.start.y
let ex = s.end.x
let ey = s.end.y

let s_copy = Segment(Point(0, 1), Point(10, 20))
let same_segment = s === s_copy
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let sx_sym = vm.interner.find("sx").unwrap();
    let sy_sym = vm.interner.find("sy").unwrap();
    let ex_sym = vm.interner.find("ex").unwrap();
    let ey_sym = vm.interner.find("ey").unwrap();
    let same_sym = vm.interner.find("same_segment").unwrap();

    assert_eq!(vm.heap.module(main_mod).get(sx_sym), Some(Value::int(0)));
    assert_eq!(vm.heap.module(main_mod).get(sy_sym), Some(Value::int(1)));
    assert_eq!(vm.heap.module(main_mod).get(ex_sym), Some(Value::int(10)));
    assert_eq!(vm.heap.module(main_mod).get(ey_sym), Some(Value::int(20)));
    assert_eq!(vm.heap.module(main_mod).get(same_sym), Some(Value::bool(true)));
}

#[test]
fn data_generic_applications_keep_exact_identity() {
    let (vm, module) = run_inline(
        r#"
data Box<T>(_ value: T)
let a = Box(1)
let b = Box(1.0)
let c = Box(1)
let different = a == b
let same = a === c
let same_class = a.class === b.class
"#,
    )
    .expect("generic data should execute");
    for (name, expected) in [("different", false), ("same", true), ("same_class", true)] {
        assert_eq!(
            vm.heap.module(module).get(vm.interner.find(name).unwrap()),
            Some(Value::bool(expected)),
            "{name}"
        );
    }
}

#[test]
fn equal_nested_data_have_equal_hashes() {
    let (vm, module) = run_inline(
        r#"
data Point(_ x: Int)
data Wrapper(_ point: Point, _ f: Float)
let a = Wrapper(Point(1), 0.0)
let b = Wrapper(Point(1), -0.0)
let equal = a == b
let hashes = a.hash == b.hash
"#,
    )
    .expect("nested data hashing should execute");
    for name in ["equal", "hashes"] {
        assert_eq!(vm.heap.module(module).get(vm.interner.find(name).unwrap()), Some(Value::bool(true)), "{name}");
    }
}

#[test]
fn data_record_evaluates_arguments_in_source_order() {
    let (vm, module) = run_inline(
        r#"
data Pair { first: Int, second: Int }
let n = 0
let next = || { n = n + 1; n }
let p = Pair { second: next(), first: next() }
let first = p.first
let second = p.second
"#,
    )
    .expect("record argument evaluation should execute");
    for (name, value) in [("first", 2), ("second", 1)] {
        assert_eq!(vm.heap.module(module).get(vm.interner.find(name).unwrap()), Some(Value::int(value)), "{name}");
    }
}

#[test]
fn data_int_components_preserve_arbitrary_precision() {
    let (mut vm, module) = run_inline(
        r#"
data Big(_ value: Int)
let b = Big(9223372036854775808)
let result = b.value == 9223372036854775808
"#,
    )
    .expect("large Int data component should execute");
    vm.force_gc();
    assert_eq!(vm.heap.module(module).get(vm.interner.find("result").unwrap()), Some(Value::bool(true)));
}

#[test]
fn data_constructor_first_class_callable_reference() {
    let (vm, module) = run_inline(
        r#"
data Point(_ x: Int, _ y: Int)
let ctor = &Point::Point(_, _)
let p = ctor(10, 20)
let direct = Point(10, 20)
let same = p === direct
let px = p.x
let py = p.y
"#,
    )
    .expect("callable reference to data constructor should execute");
    let same_sym = vm.interner.find("same").unwrap();
    let px_sym = vm.interner.find("px").unwrap();
    let py_sym = vm.interner.find("py").unwrap();
    assert_eq!(vm.heap.module(module).get(same_sym), Some(Value::bool(true)));
    assert_eq!(vm.heap.module(module).get(px_sym), Some(Value::int(10)));
    assert_eq!(vm.heap.module(module).get(py_sym), Some(Value::int(20)));
}

#[test]
fn data_phantom_type_specializations() {
    let (vm, module) = run_inline(
        r#"
data Tagged<T>(_ id: Int)
let t1 = Tagged<Int>::Tagged(42)
let t2 = Tagged<String>::Tagged(42)
let same_value = t1.id === t2.id
let equal = t1 == t2
let same_class = t1.class === t2.class
"#,
    )
    .expect("phantom generic data should execute");
    let same_val_sym = vm.interner.find("same_value").unwrap();
    let equal_sym = vm.interner.find("equal").unwrap();
    let same_class_sym = vm.interner.find("same_class").unwrap();
    assert_eq!(vm.heap.module(module).get(same_val_sym), Some(Value::bool(true)));
    assert_eq!(vm.heap.module(module).get(equal_sym), Some(Value::bool(false)));
    assert_eq!(vm.heap.module(module).get(same_class_sym), Some(Value::bool(true)));
}

