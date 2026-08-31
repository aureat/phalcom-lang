//! Runtime ADT primitive conformance (Part 5 boundary owned by Part 4).

use phalcom_core::error::{PhError, RuntimeError};
use phalcom_core::modules::compile::{EntrySelection, ProgramCompiler};
use phalcom_core::value::Value;
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
fn adt_case_primitives_cover_singletons_heap_cases_and_non_adt_values() {
    let source = r#"
enum Shape {
  @variant None
  @variant Pair(_ left: Int, _ right: Int)
}

let singleton = Shape::None
let pair = Shape::Pair(1, 2)
"#;
    let (vm, module) = run_inline(source).expect("ADT source should execute");
    let singleton = vm
        .heap
        .module(module)
        .get(vm.interner.find("singleton").expect("singleton symbol"))
        .expect("singleton binding");
    let pair = vm
        .heap
        .module(module)
        .get(vm.interner.find("pair").expect("pair symbol"))
        .expect("pair binding");

    let singleton_variant = vm.runtime_variant_of(singleton).expect("singleton variant");
    let pair_variant = vm.runtime_variant_of(pair).expect("heap variant");
    assert_ne!(singleton_variant, pair_variant);
    assert!(vm.value_is_variant(singleton, singleton_variant));
    assert!(!vm.value_is_variant(singleton, pair_variant));
    assert!(vm.value_is_variant(pair, pair_variant));

    assert_eq!(vm.case_payload_len(singleton), Some(0));
    assert_eq!(vm.case_payload_len(pair), Some(2));
    assert!(matches!(
        vm.case_payload_at(singleton, 0),
        Err(RuntimeError::InvalidVariantPayloadSlot { slot: 0, len: 0 })
    ));
    assert_eq!(vm.case_payload_at(pair, 0).expect("first payload"), Value::int(1));
    assert_eq!(vm.case_payload_at(pair, 1).expect("second payload"), Value::int(2));
    assert!(matches!(
        vm.case_payload_at(pair, 2),
        Err(RuntimeError::InvalidVariantPayloadSlot { slot: 2, len: 2 })
    ));

    let singleton_class = vm.case_behavior_class(singleton).expect("singleton behavior class");
    let pair_class = vm.case_behavior_class(pair).expect("heap behavior class");
    assert_ne!(singleton_class, pair_class);
    assert!(vm.case_behavior_class(Value::int(7)).is_none());
    assert_eq!(vm.runtime_variant_of(Value::int(7)), None);
    assert_eq!(vm.case_payload_len(Value::int(7)), None);
    assert!(matches!(
        vm.case_payload_at(Value::int(7), 0),
        Err(RuntimeError::InvalidVariantPayloadSlot { slot: 0, len: 0 })
    ));
}
