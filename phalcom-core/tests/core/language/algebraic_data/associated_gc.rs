//! GC edges for associated-family capabilities (Part 4).

use phalcom_core::heap::{InstanceObject, Object};
use phalcom_core::modules::semantic_lowering::ExecutableFamilyDescriptor;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

#[test]
fn bound_owner_survives_only_while_associated_family_is_rooted() {
    let mut vm = VM::new();
    vm.force_gc();
    let owner_class = vm.universe.classes.object_class;
    let owner = vm.heap.alloc(Object::Instance(InstanceObject::new(owner_class, 0)));
    let descriptor = Arc::new(ExecutableFamilyDescriptor { entries: Box::new([]) });
    let family = vm.heap.alloc_associated_family(descriptor, Some(Value::obj(owner)));

    vm.push_root_for_test(Value::obj(family));
    vm.force_gc();
    assert!(vm.heap.try_get(family).is_some());
    assert!(vm.heap.try_get(owner).is_some(), "bound owner must be traced through family capability");

    vm.pop_root_for_test();
    vm.force_gc();
    assert!(vm.heap.try_get(family).is_none());
    assert!(vm.heap.try_get(owner).is_none());
}
