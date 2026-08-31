//! ADT payload and hidden-class tracing conformance (Part 4).

use phalcom_core::adt::RuntimeVariantId;
use phalcom_core::error::PhError;
use phalcom_core::heap::Object;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use phalcom_modules::DeclarationId;
use std::sync::Arc;

fn run_inline(source: &str) -> Result<(VM, phalcom_core::heap::ObjRef), PhError> {
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    vm.run_compiled(&program)?;
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;
    Ok((vm, module))
}

#[test]
fn nested_adt_payload_edges_survive_then_collect_with_outer_case() {
    let mut vm = VM::new();
    vm.force_gc();
    let leaf = vm.alloc_string_value("leaf".to_owned());
    let inner = vm.heap.alloc_adt_case(RuntimeVariantId::from_raw(10), Box::new([leaf]));
    let outer = vm.heap.alloc_adt_case(RuntimeVariantId::from_raw(11), Box::new([Value::obj(inner)]));

    vm.push_root_for_test(Value::obj(outer));
    vm.force_gc();
    let leaf_ref = leaf.as_obj().expect("leaf handle");
    assert!(vm.heap.try_get(outer).is_some());
    assert!(vm.heap.try_get(inner).is_some());
    assert!(vm.heap.try_get(leaf_ref).is_some());

    vm.pop_root_for_test();
    vm.force_gc();
    assert!(vm.heap.try_get(outer).is_none());
    assert!(vm.heap.try_get(inner).is_none());
    assert!(vm.heap.try_get(leaf_ref).is_none());
}

#[test]
fn registered_hidden_case_classes_remain_gc_roots() {
    let source = r#"
enum Shape {
  @variant None
  @variant Pair(_ value: Int)
}

let pair = Shape::Pair(1)
"#;
    let (mut vm, module) = run_inline(source).expect("ADT should execute");
    let owner = DeclarationId::new(vm.heap.module(module).id.clone(), "Shape".into());
    let enum_id = vm.adt_registry.enum_by_declaration(&owner).expect("enum registered");
    let descriptor = vm.adt_registry.enum_descriptor(enum_id).expect("enum descriptor").clone();
    vm.force_gc();
    assert!(vm.heap.try_get(descriptor.root_class).is_some());
    for variant_id in descriptor.variants {
        let behavior = vm.adt_registry.variant_descriptor(variant_id).expect("variant descriptor").behavior_class;
        assert!(matches!(vm.heap.get(behavior), Object::Class(_)));
    }
}
