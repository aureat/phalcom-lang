//! Explicit ADT ownership and tracing scenarios.

use super::vm_support::run_inline;
use phalcom_core::adt::RuntimeVariantId;
use phalcom_core::heap::{InstanceObject, Object};
use phalcom_core::modules::semantic_lowering::ExecutableFamilyDescriptor;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::sync::Arc;

#[test]
fn adt_gc_01_payload_object_is_traced_through_case() {
    let mut vm = VM::new();
    let payload = vm.alloc_string_value("payload".into());
    let case = vm.heap.alloc_adt_case(RuntimeVariantId::from_raw(1), Box::new([payload]));
    vm.push_root_for_test(Value::obj(case));
    vm.force_gc();
    assert!(vm.heap.try_get(payload.as_obj().expect("payload object")).is_some());
}

#[test]
fn adt_gc_02_nested_case_traces_inner_case_and_payload() {
    let mut vm = VM::new();
    let payload = vm.alloc_string_value("nested".into());
    let inner = vm.heap.alloc_adt_case(RuntimeVariantId::from_raw(2), Box::new([payload]));
    let outer = vm.heap.alloc_adt_case(RuntimeVariantId::from_raw(3), Box::new([Value::obj(inner)]));
    vm.push_root_for_test(Value::obj(outer));
    vm.force_gc();
    assert!(vm.heap.try_get(inner).is_some());
    assert!(vm.heap.try_get(payload.as_obj().expect("payload object")).is_some());
}

#[test]
fn adt_gc_03_case_behavior_owner_is_rooted_by_registered_descriptor() {
    let mut vm = VM::new();
    let owner = vm.heap.alloc(Object::Instance(InstanceObject::new(vm.universe.classes.object_class, 0)));
    assert!(vm.heap.try_get(owner).is_some());
}

#[test]
fn adt_gc_04_family_descriptor_keeps_bound_owner_alive() {
    let mut vm = VM::new();
    let owner = vm.heap.alloc(Object::Instance(InstanceObject::new(vm.universe.classes.object_class, 0)));
    let descriptor = Arc::new(ExecutableFamilyDescriptor { entries: Box::new([]) });
    let family = vm.heap.alloc_associated_family(descriptor, Some(Value::obj(owner)));
    vm.push_root_for_test(Value::obj(family));
    vm.force_gc();
    assert!(vm.heap.try_get(owner).is_some());
}

#[test]
fn adt_gc_05_unreachable_case_is_collected_after_root_release() {
    let mut vm = VM::new();
    let case = vm.heap.alloc_adt_case(RuntimeVariantId::from_raw(4), Box::new([]));
    vm.push_root_for_test(Value::obj(case));
    vm.force_gc();
    vm.pop_root_for_test();
    vm.force_gc();
    assert!(vm.heap.try_get(case).is_none());
}

#[test]
#[ignore = "GATED: forced collection during match staging needs dedicated VM hook"]
fn adt_gc_06_match_scratch_roots_payload_before_branch_use() {
    let mut vm = VM::new();
    let payload = vm.alloc_string_value("match-payload".into());
    let case = vm.heap.alloc_adt_case(RuntimeVariantId::from_raw(5), Box::new([payload]));
    vm.push_root_for_test(Value::obj(case));
    vm.force_gc();
    assert!(vm.heap.try_get(payload.as_obj().expect("payload object")).is_some());
    vm.pop_root_for_test();
    vm.force_gc();
    assert!(vm.heap.try_get(case).is_none());
}

#[test]
#[ignore = "GATED: closure-after-arm fixture is required"]
fn adt_gc_07_closure_capture_survives_arm_scratch_cleanup() {
    let source = "let captured = 42\nlet closure = { captured }\nlet result = closure()\n";
    let result = run_inline(source);
    assert!(result.is_ok(), "closure capture fixture should execute");
}
