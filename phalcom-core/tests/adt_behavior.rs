//! ADT root and hidden case behavior-class conformance (Part 4).

use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
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
fn enum_root_and_hidden_case_classes_have_descriptor_backed_hierarchy() {
    let source = r#"
enum Shape {
  @variant None
  @variant Pair(_ left: Int, _ right: Int)
}

let none = Shape::None
let pair = Shape::Pair(1, 2)
"#;
    let (vm, module) = run_inline(source).expect("ADT should execute");
    let root = vm
        .heap
        .module(module)
        .get(vm.interner.find("Shape").expect("Shape symbol"))
        .expect("enum root binding")
        .as_obj()
        .expect("enum root is class object");
    let owner = DeclarationId::new(vm.heap.module(module).id.clone(), "Shape".into());
    let enum_id = vm.adt_registry.enum_by_declaration(&owner).expect("registered enum");
    let enum_desc = vm.adt_registry.enum_descriptor(enum_id).expect("enum descriptor");
    assert_eq!(enum_desc.root_class, root);
    assert_eq!(enum_desc.variants.len(), 2);

    for variant_id in &enum_desc.variants {
        let variant = vm.adt_registry.variant_descriptor(*variant_id).expect("variant descriptor");
        assert_eq!(vm.heap.class(variant.behavior_class).superclass, Some(root));
        let hidden_name = format!("Shape::{}", variant.semantic_id.selector);
        if let Some(hidden_symbol) = vm.interner.find(&hidden_name) {
            assert!(
                vm.heap.module(module).slot_of(hidden_symbol).is_none(),
                "hidden case behavior class must not become module global"
            );
        }
    }

    let none = vm
        .heap
        .module(module)
        .get(vm.interner.find("none").expect("none symbol"))
        .expect("none binding");
    let pair = vm
        .heap
        .module(module)
        .get(vm.interner.find("pair").expect("pair symbol"))
        .expect("pair binding");
    assert_eq!(
        vm.case_behavior_class(none),
        Some(vm.adt_registry.variant_descriptor(enum_desc.variants[0]).unwrap().behavior_class)
    );
    assert_eq!(
        vm.case_behavior_class(pair),
        Some(vm.adt_registry.variant_descriptor(enum_desc.variants[1]).unwrap().behavior_class)
    );
}

#[test]
fn enum_root_direct_instantiation_remains_rejected() {
    let source = r#"
enum Status {
  @variant Pending
}

let invalid = Status.new()
"#;
    assert!(run_inline(source).is_err());
}
