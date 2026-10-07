//! Focused F.2 GC proof.
//!
//! This is deliberately not a substitute for the repository's specified-but-
//! unbuilt PHALCOM_GC_STRESS lane. It proves the concrete PackBuilder trace edge
//! with the already-live `VM::force_gc()` test seam.

use phalcom_core::heap::{ArgumentPackBuilderObject, InstanceObject, Object};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;

fn settled_vm() -> VM {
    let mut vm = VM::new_kernel();
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
    builder.push_positional(Value::obj(positional));
    builder.reserve_label(label).expect("reserve test label");
    builder.fill_reserved(Value::obj(labeled)).expect("fill test label");

    let builder_ref = vm.heap.alloc(Object::PackBuilder(Box::new(builder)));
    vm.push_root_for_test(Value::obj(builder_ref));
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

#[test]
fn rooted_pack_root_keeps_children_alive() {
    let mut vm = settled_vm();
    let positional = alloc_instance(&mut vm);
    let labeled = alloc_instance(&mut vm);
    let label = vm.interner.intern("pack_gc_label");

    let pack = phalcom_core::product::finish_pack_value(&mut vm, vec![Value::obj(positional)], vec![(label, Value::obj(labeled))]).expect("finish_pack");

    let pack_ref = pack.as_obj().expect("pack obj");
    vm.push_root_for_test(pack);
    vm.force_gc();

    assert!(vm.heap.try_get(pack_ref).is_some(), "rooted pack must survive");
    assert!(vm.heap.try_get(positional).is_some(), "pack positional child must survive");
    assert!(vm.heap.try_get(labeled).is_some(), "pack labeled child must survive");

    vm.pop_root_for_test();
    vm.force_gc();

    assert!(vm.heap.try_get(pack_ref).is_none(), "unrooted pack must be collected");
    assert!(vm.heap.try_get(positional).is_none(), "positional child must be collected");
    assert!(vm.heap.try_get(labeled).is_none(), "labeled child must be collected");
}

#[test]
fn rooted_pack_slice_keeps_root_and_children_alive() {
    let mut vm = settled_vm();
    let pos0 = alloc_instance(&mut vm);
    let pos1 = alloc_instance(&mut vm);
    let lab0 = alloc_instance(&mut vm);
    let label = vm.interner.intern("slice_gc_label");

    let root_val =
        phalcom_core::product::finish_pack_value(&mut vm, vec![Value::obj(pos0), Value::obj(pos1)], vec![(label, Value::obj(lab0))]).expect("finish_pack root");

    let root_ref = root_val.as_obj().expect("root obj");

    // Create a slice that only visibly includes pos1 (offset 1, len 1)
    let slice_val = phalcom_core::product::finish_pack_slice_value(&mut vm, root_val, 1, 1, 0, 0).expect("finish_pack_slice");
    let slice_ref = slice_val.as_obj().expect("slice obj");

    // Root only the slice! Do NOT root root_val.
    vm.push_root_for_test(slice_val);
    vm.force_gc();

    assert!(vm.heap.try_get(slice_ref).is_some(), "rooted slice must survive");
    assert!(vm.heap.try_get(root_ref).is_some(), "root must survive through slice reference");
    assert!(vm.heap.try_get(pos0).is_some(), "pos0 in root backing must survive via root");
    assert!(vm.heap.try_get(pos1).is_some(), "pos1 in root backing must survive via root");
    assert!(vm.heap.try_get(lab0).is_some(), "lab0 in root backing must survive via root");

    // Drop the slice root. Now slice, root, and all children must be collected.
    vm.pop_root_for_test();
    vm.force_gc();

    assert!(vm.heap.try_get(slice_ref).is_none(), "unrooted slice must be collected");
    assert!(vm.heap.try_get(root_ref).is_none(), "root must be collected once slice is gone");
    assert!(vm.heap.try_get(pos0).is_none(), "pos0 must be collected");
    assert!(vm.heap.try_get(pos1).is_none(), "pos1 must be collected");
    assert!(vm.heap.try_get(lab0).is_none(), "lab0 must be collected");
}
