//! Runtime behavior of frozen associated-family capabilities (Part 4).

use phalcom_core::error::PhError;
use phalcom_core::heap::Object;
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::vm::VM;
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
fn associated_family_freezes_variant_capabilities() {
    let source = r#"
enum Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

let family = Weird::Marker::*;
"#;
    let (vm, module) = run_inline(source).expect("associated family should execute");
    let family = vm
        .heap
        .module(module)
        .get(vm.interner.find("family").expect("family symbol"))
        .expect("family binding");
    let family_ref = family.as_obj().expect("family must be heap capability");
    let Object::AssociatedFamily(family_obj) = vm.heap.get(family_ref) else {
        panic!("expected AssociatedFamily object")
    };
    assert_eq!(family_obj.descriptor.entries.len(), 3);
    assert_eq!(
        family_obj
            .descriptor
            .entries
            .iter()
            .filter(|entry| matches!(entry.member_kind, phalcom_semantic::types::family::FamilyMemberTypeKind::Value))
            .count(),
        1
    );
    assert_eq!(
        family_obj
            .descriptor
            .entries
            .iter()
            .filter(|entry| matches!(entry.member_kind, phalcom_semantic::types::family::FamilyMemberTypeKind::Callable))
            .count(),
        2
    );
    assert!(family_obj.bound_owner.is_none(), "variant family needs no runtime hierarchy owner");
}
