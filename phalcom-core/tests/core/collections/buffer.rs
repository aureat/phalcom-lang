use super::vm_support;
use phalcom_core::error::RuntimeError;
use phalcom_core::heap::{BufferObject, ObjRef};
use phalcom_core::primitive::buffer::{buffer_class_new, buffer_raw_at, buffer_raw_capacity, buffer_raw_length, buffer_raw_push, buffer_raw_set};
use phalcom_core::primitive::map::map_raw_put;
use phalcom_core::primitive::set::set_raw_add;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn global(vm: &VM, module: ObjRef, name: &str) -> Value {
    vm.heap
        .module(module)
        .get(vm.interner.find(name).expect("interned global"))
        .expect("global value")
}

#[test]
fn buffer_is_bounded_typed_mutable_and_gc_traced() {
    let source = r#"
let buffer = Buffer<Int>.new(2)
let initialSize = buffer.size
let initialCapacity = buffer.capacity
let firstPush = buffer.push(11)
let secondPush = buffer.push(22)
let fullPush = buffer.push(33)
let firstPushSucceeded = firstPush.isOk
let secondPushSucceeded = secondPush.isOk
let fullPushFailed = fullPush.isErr
let sizeAfterPush = buffer.size
let capacityAfterPush = buffer.capacity
let first = buffer.at(0)
let setResult = buffer.set(-1, 44)
let last = buffer.at(-1)
let sizeAfterSet = buffer.size
let empty = Buffer<Int>.new(0)
let emptyPush = empty.push(1)
let emptyPushFailed = emptyPush.isErr
"#;
    let (mut vm, module) = vm_support::run_inline(source).expect("Buffer source API compiles and runs");
    let buffer = global(&vm, module, "buffer");
    assert_eq!(buffer.class(&vm), vm.universe.classes.buffer_class);
    assert_eq!(global(&vm, module, "initialSize"), Value::int(0));
    assert_eq!(global(&vm, module, "initialCapacity"), Value::int(2));
    assert_eq!(global(&vm, module, "firstPushSucceeded"), Value::bool(true));
    assert_eq!(global(&vm, module, "secondPushSucceeded"), Value::bool(true));
    assert_eq!(global(&vm, module, "fullPushFailed"), Value::bool(true));
    assert_eq!(global(&vm, module, "sizeAfterPush"), Value::int(2));
    assert_eq!(global(&vm, module, "capacityAfterPush"), Value::int(2));
    assert_eq!(global(&vm, module, "first"), Value::int(11));
    assert_eq!(global(&vm, module, "last"), Value::int(44));
    assert_eq!(global(&vm, module, "sizeAfterSet"), Value::int(2));
    assert_eq!(global(&vm, module, "emptyPushFailed"), Value::bool(true));

    let receiver = Value::obj(vm.universe.classes.buffer_class);
    for invalid in [Value::int(-1), Value::float(2.5), Value::float(2.0)] {
        assert!(buffer_class_new(&mut vm, &receiver, &[invalid]).is_err(), "invalid capacity {invalid:?}");
    }
    assert!(matches!(
        buffer_class_new(&mut vm, &receiver, &[Value::int(i64::MAX)]),
        Err(phalcom_core::error::PhError::Runtime(RuntimeError::ResourceLimit))
    ));

    assert_eq!(buffer_raw_length(&mut vm, &buffer, &[]).expect("raw size"), Value::int(2));
    assert_eq!(buffer_raw_capacity(&mut vm, &buffer, &[]).expect("logical capacity"), Value::int(2));
    assert_eq!(
        buffer_raw_push(&mut vm, &buffer, &[Value::int(55)]).expect("full push result"),
        Value::bool(false)
    );
    assert!(
        buffer_raw_at(&mut vm, &buffer, &[Value::int(2)]).is_err(),
        "index equal to size is not initialized"
    );
    assert!(
        buffer_raw_set(&mut vm, &buffer, &[Value::int(2), Value::int(66)]).is_err(),
        "set cannot create a sparse slot"
    );

    let map = Value::obj(vm.heap.alloc_map());
    assert!(map_raw_put(&mut vm, &map, &[buffer, Value::int(1)]).is_err(), "Buffer cannot be a Map key");
    let set = Value::obj(vm.heap.alloc_set());
    assert!(set_raw_add(&mut vm, &set, &[buffer]).is_err(), "Buffer cannot be a Set key");

    let child = vm.alloc_string_value("reachable only through Buffer".to_string());
    let child_id = child.as_obj().expect("child object");
    let mut storage = BufferObject::try_new(1).expect("one-value Buffer");
    storage.push(child).expect("store child in initialized prefix");
    let rooted_buffer = Value::obj(vm.heap.alloc_buffer(storage));
    let rooted_buffer_id = rooted_buffer.as_obj().expect("rooted Buffer");
    vm.push_root_for_test(rooted_buffer);
    vm.force_gc();
    assert!(vm.heap.try_get(child_id).is_some(), "Buffer traces initialized children");

    vm.heap.buffer_mut(rooted_buffer_id).set(0, Value::int(0)).expect("replace initialized child");
    vm.force_gc();
    assert!(vm.heap.try_get(child_id).is_none(), "replaced child is no longer retained by Buffer");
    vm.pop_root_for_test();
    vm.force_gc();
    assert!(vm.heap.try_get(rooted_buffer_id).is_none(), "unrooted Buffer is collectible");
}
