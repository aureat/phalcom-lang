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
#[test]
fn data_generic_callable_reference_specialization() {
    let (vm, module) = run_inline(
        r#"
data Tagged<T>(_ id: Int)
let make_int = &Tagged<Int>::Tagged(_)
let make_str = &Tagged<String>::Tagged(_)
let t1 = make_int(10)
let t2 = make_str(10)
let eq = t1 == t2
let t1_id = t1.id
let t2_id = t2.id
"#,
    )
    .expect("generic data callable references should execute");
    let eq_sym = vm.interner.find("eq").unwrap();
    let t1_sym = vm.interner.find("t1_id").unwrap();
    let t2_sym = vm.interner.find("t2_id").unwrap();
    assert_eq!(vm.heap.module(module).get(eq_sym), Some(Value::bool(false)));
    assert_eq!(vm.heap.module(module).get(t1_sym), Some(Value::int(10)));
    assert_eq!(vm.heap.module(module).get(t2_sym), Some(Value::int(10)));
}

#[test]
fn data_associated_family_dynamic_pack_invocation() {
    let (vm, module) = run_inline(
        r#"
data Point(_ x: Int, _ y: Int)
let fam = &Point::Point...
let p = fam(10, 20)
let px = p.x
let py = p.y
"#,
    )
    .expect("whole-family capture and dynamic pack invocation on data should execute");
    let px_sym = vm.interner.find("px").unwrap();
    let py_sym = vm.interner.find("py").unwrap();
    assert_eq!(vm.heap.module(module).get(px_sym), Some(Value::int(10)));
    assert_eq!(vm.heap.module(module).get(py_sym), Some(Value::int(20)));
}

#[test]
fn data_component_custom_equality_protocol() {
    let (vm, module) = run_inline(
        r#"
class CustomKey {
  _id: Int

  @constructor
  new(_ id: Int) {
    _id = id
  }

  id() -> Int { _id }

  ==(_ other: Object) -> Bool {
    if other.class === self.class {
      _id === other.id()
    } else {
      false
    }
  }

  hash -> Int {
    _id.hash
  }
}

data Wrapper(_ key: CustomKey, _ tag: String)

let k1 = CustomKey.new(5)
let k2 = CustomKey.new(5)
let k3 = CustomKey.new(6)

let w1 = Wrapper(k1, "alpha")
let w2 = Wrapper(k2, "alpha")
let w3 = Wrapper(k3, "alpha")

let eq12 = w1 == w2
let neq12 = w1 != w2
let eq13 = w1 == w3
let same12 = w1 === w2
let hash_eq = w1.hash === w2.hash
"#,
    )
    .expect("custom component equality protocol should execute");
    let eq12_sym = vm.interner.find("eq12").unwrap();
    let neq12_sym = vm.interner.find("neq12").unwrap();
    let eq13_sym = vm.interner.find("eq13").unwrap();
    let same12_sym = vm.interner.find("same12").unwrap();
    let hash_eq_sym = vm.interner.find("hash_eq").unwrap();
    assert_eq!(vm.heap.module(module).get(eq12_sym), Some(Value::bool(true)));
    assert_eq!(vm.heap.module(module).get(neq12_sym), Some(Value::bool(false)));
    assert_eq!(vm.heap.module(module).get(eq13_sym), Some(Value::bool(false)));
    assert_eq!(vm.heap.module(module).get(same12_sym), Some(Value::bool(false)));
    assert_eq!(vm.heap.module(module).get(hash_eq_sym), Some(Value::bool(true)));
}

#[test]
fn open_generic_data_construction_fails_closed() {
    let src = r#"
data Container<T>(_ item: T)
fun <T> make_it(x: T) {
  Container(x)
}
"#;
    let res = run_inline(src);
    assert!(res.is_err(), "open generic data construction must fail closed during compilation");
}

#[test]
fn data_source_order_with_exception_proves_evaluation_semantics() {
    let src = r#"
data User {
  id: Int,
  name: String
}

let order = 0

let get_id = || {
  order = order + 1
  42
}

let fail_name = || {
  order = order + 10
  throw Error.new("fail")
}

let threw = false
try {
  let u = User { name: fail_name(), id: get_id() }
} catch e {
  threw = true
}
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let order_sym = vm.interner.find("order").unwrap();
    let threw_sym = vm.interner.find("threw").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(threw_sym), Some(Value::bool(true)));
    assert_eq!(vm.heap.module(main_mod).get(order_sym), Some(Value::int(10)));
}

#[test]
fn data_nullary_phantom_specialization_source_level() {
    let src = r#"
data Tag()

let t1 = Tag()
let t2 = Tag()
let same = t1 === t2
"#;
    let (vm, main_mod) = run_inline(src).expect("should run successfully");
    let same_sym = vm.interner.find("same").unwrap();
    assert_eq!(vm.heap.module(main_mod).get(same_sym), Some(Value::bool(true)));
}


#[test]
fn p4_shape_generic_tuple_specializations_share_behavior_and_keep_exact_descriptors() {
    let (vm, module) = run_inline(
        r#"
data MyTuple<***Es: CompleteTypePack>(***Es)

let one = MyTuple(1)
let two = MyTuple(1, "x")
let one_again = MyTuple(2)

let one_value = one
let two_value = two
let one_again_value = one_again
let same_class = one.class === two.class
"#,
    )
    .expect("shape-generic tuple data should execute");

    assert_eq!(
        vm.heap.module(module).get(vm.interner.find("same_class").unwrap()),
        Some(Value::bool(true))
    );

    let descriptor = |name: &str| {
        let value = vm.heap.module(module).get(vm.interner.find(name).unwrap()).unwrap();
        let object = value.as_obj().expect("shape-generic data is heap allocated");
        vm.heap.data(object).descriptor
    };
    let one = descriptor("one_value");
    let two = descriptor("two_value");
    let one_again = descriptor("one_again_value");

    assert_eq!(one, one_again, "identical exact specializations must reuse one descriptor");
    assert_ne!(one, two, "different exact tuple packs must have different descriptors");

    let one_desc = vm.data_registry.descriptor(one).expect("one descriptor");
    let two_desc = vm.data_registry.descriptor(two).expect("two descriptor");
    assert_eq!(one_desc.behavior_class, two_desc.behavior_class, "specializations must share one behavior class");
    assert_ne!(one_desc.exact_type, two_desc.exact_type, "exact semantic type identity must remain distinct");
    assert_ne!(one_desc.layout, two_desc.layout, "different tuple arities must use exact layouts");
}

#[test]
fn p4_shape_generic_record_specializations_expose_generated_fields_without_fake_declarations() {
    let (vm, module) = run_inline(
        r#"
data MyRecord<**Fs: RecordFields> {
  **Fs
}

let first = MyRecord { name: "Ada" }
let second = MyRecord { age: 42, active: true }
let first_again = MyRecord { name: "Grace" }

let name = first.name
let age = second.age
let active = second.active
let same_class = first.class === second.class
let first_value = first
let second_value = second
let first_again_value = first_again
"#,
    )
    .expect("shape-generic record data should execute");

    assert_eq!(
        vm.heap.module(module).get(vm.interner.find("name").unwrap()).unwrap().to_string(&vm),
        "Ada"
    );
    assert_eq!(
        vm.heap.module(module).get(vm.interner.find("age").unwrap()),
        Some(Value::int(42))
    );
    assert_eq!(
        vm.heap.module(module).get(vm.interner.find("active").unwrap()),
        Some(Value::bool(true))
    );
    assert_eq!(
        vm.heap.module(module).get(vm.interner.find("same_class").unwrap()),
        Some(Value::bool(true))
    );

    let descriptor = |name: &str| {
        let value = vm.heap.module(module).get(vm.interner.find(name).unwrap()).unwrap();
        vm.heap.data(value.as_obj().expect("record specialization is data")).descriptor
    };
    let first = descriptor("first_value");
    let second = descriptor("second_value");
    let first_again = descriptor("first_again_value");
    assert_eq!(first, first_again, "same exact record shape must reuse its descriptor");
    assert_ne!(first, second, "different exact record shapes must keep distinct descriptors");

    let first_desc = vm.data_registry.descriptor(first).expect("first descriptor");
    let second_desc = vm.data_registry.descriptor(second).expect("second descriptor");
    assert_eq!(first_desc.behavior_class, second_desc.behavior_class);
    assert_ne!(first_desc.exact_type, second_desc.exact_type);
    assert_ne!(first_desc.layout, second_desc.layout);
}

#[test]
fn p4_shape_generic_construction_through_generic_callable_keeps_behavior_class_identity() {
    let (vm, module) = run_inline(
        r#"
data MyRecord<**Fs: RecordFields> {
  **Fs
}

class Factory {
  @class make_record<T>(value: T) -> MyRecord<**{ payload: T }> {
    MyRecord { payload: value }
  }
}

const int_record: MyRecord<**{ payload: Int }> = Factory.make_record(value: 42)
const string_record: MyRecord<**{ payload: String }> = Factory.make_record(value: "text")
let same_class = int_record.class === string_record.class
"#,
    )
    .expect("generic callable should construct exact shape-generic specializations");

    assert_eq!(vm.heap.module(module).get(vm.interner.find("same_class").unwrap()), Some(Value::bool(true)));

    let descriptor = |name: &str| {
        let value = vm.heap.module(module).get(vm.interner.find(name).unwrap()).unwrap();
        vm.heap.data(value.as_obj().expect("shape-generic record is data")).descriptor
    };
    let int_descriptor = descriptor("int_record");
    let string_descriptor = descriptor("string_record");
    let int_desc = vm.data_registry.descriptor(int_descriptor).expect("Int specialization descriptor");
    let string_desc = vm.data_registry.descriptor(string_descriptor).expect("String specialization descriptor");
    assert_eq!(int_desc.behavior_class, string_desc.behavior_class);
    assert_ne!(int_desc.exact_type, string_desc.exact_type);
}

#[test]
fn p4_empty_layout_row_specializations_keep_exact_identity_through_generic_callable() {
    let (vm, module) = run_inline(
        r#"
data PhantomRow<**Fs: RecordFields>()

class Factory {
  @class make<T>(value: T) -> PhantomRow<**{ payload: T }> {
    PhantomRow()
  }
}

const int_row: PhantomRow<**{ payload: Int }> = Factory.make(value: 1)
const string_row: PhantomRow<**{ payload: String }> = Factory.make(value: "text")
const int_row_again: PhantomRow<**{ payload: Int }> = Factory.make(value: 2)
let same_class = int_row.class === string_row.class
"#,
    )
    .expect("generic callable should preserve phantom row specializations");

    assert_eq!(vm.heap.module(module).get(vm.interner.find("same_class").unwrap()), Some(Value::bool(true)));
    let descriptor = |name: &str| {
        let value = vm.heap.module(module).get(vm.interner.find(name).unwrap()).unwrap();
        vm.heap.data(value.as_obj().expect("phantom row is data")).descriptor
    };
    let int_id = descriptor("int_row");
    let string_id = descriptor("string_row");
    let int_again_id = descriptor("int_row_again");
    assert_ne!(int_id, string_id, "exact row type identity must survive an empty product layout");
    assert_eq!(int_id, int_again_id, "the same exact phantom specialization is interned");

    let int = vm.data_registry.descriptor(int_id).expect("Int phantom row descriptor");
    let string = vm.data_registry.descriptor(string_id).expect("String phantom row descriptor");
    assert_eq!(int.behavior_class, string.behavior_class);
    assert_eq!(int.layout, string.layout, "phantom row specializations have the same empty layout");
    assert_ne!(int.exact_type, string.exact_type);
}

#[test]
fn p4_fixed_and_expanded_record_components_use_exact_specialization_slots() {
    let (vm, module) = run_inline(
        r#"
data Envelope<**Fs: RecordFields> {
  id: Int,
  **Fs
}

let envelope = Envelope { title: "hello", id: 7 }
let id = envelope.id
let title = envelope.title
"#,
    )
    .expect("fixed plus expanded record data should execute");

    assert_eq!(vm.heap.module(module).get(vm.interner.find("id").unwrap()), Some(Value::int(7)));
    assert_eq!(
        vm.heap.module(module).get(vm.interner.find("title").unwrap()).unwrap().to_string(&vm),
        "hello"
    );
}
