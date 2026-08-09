//! Focused F.2 GC proof.
//!
//! This is deliberately not a substitute for the repository's specified-but-
//! unbuilt PHALCOM_GC_STRESS lane. It proves the concrete PackBuilder trace edge
//! with the already-live `VM::force_gc()` test seam.

use phalcom_core::heap::{ArgumentPackBuilderObject, InstanceObject, Object};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn settled_vm() -> VM {
    let mut vm = VM::new();
    vm.force_gc();
    vm
}

fn alloc_instance(vm: &mut VM) -> phalcom_core::heap::ObjRef {
    let class = vm.universe.classes.object_class;
    vm.heap.alloc(Object::Instance(InstanceObject {
        class,
        slots: Vec::new().into_boxed_slice(),
    }))
}

#[test]
fn rooted_pack_builder_keeps_positional_and_labeled_values_alive() {
    let mut vm = settled_vm();
    let positional = alloc_instance(&mut vm);
    let labeled = alloc_instance(&mut vm);
    let label = vm.interner.intern("f2_gc_label");

    let mut builder = ArgumentPackBuilderObject::new();
    builder.push_positional(Value::Obj(positional));
    builder.reserve_label(label).expect("reserve test label");
    builder.fill_reserved(Value::Obj(labeled)).expect("fill test label");

    let builder_ref = vm.heap.alloc(Object::PackBuilder(Box::new(builder)));
    vm.push_root_for_test(Value::Obj(builder_ref));
    vm.force_gc();

    assert!(vm.heap.try_get(builder_ref).is_some(), "rooted builder must survive");
    assert!(vm.heap.try_get(positional).is_some(), "builder positional lane must be traced");
    assert!(vm.heap.try_get(labeled).is_some(), "builder labeled-value lane must be traced");

    vm.pop_root_for_test();
    vm.force_gc();

    assert!(vm.heap.try_get(builder_ref).is_none(), "unrooted transient builder must be collectible");
    assert!(vm.heap.try_get(positional).is_none(), "positional value must die with its only holder");
    assert!(vm.heap.try_get(labeled).is_none(), "labeled value must die with its only holder");
}
