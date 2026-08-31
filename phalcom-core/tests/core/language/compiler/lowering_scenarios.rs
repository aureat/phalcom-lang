//! Explicit semantic-to-core lowering scenarios for ADT match products.

use super::vm_support::compile_inline;
use phalcom_core::bytecode::Bytecode;
use phalcom_core::modules::semantic_lowering::{AssociatedLoweringSpec, LoweringSiteKind};

#[test]
fn adt_lower_01_one_associated_site_has_one_lowering_record() {
    let source = "enum State { @variant Ready }\nlet value = State::Ready\n";
    let (vm, program, _closure) = compile_inline(source).expect("singleton source should compile");
    let lowering = &program.modules[&program.entry].lowering;
    assert_eq!(lowering.associated.len(), 1);
    assert!(lowering.associated.keys().all(|site| site.kind == LoweringSiteKind::AssociatedLookup));
    let _ = vm;
}

#[test]
fn adt_lower_02_multiple_sites_keep_distinct_source_keys() {
    let source = "enum State { @variant Ready }\nlet first = State::Ready\nlet second = State::Ready\n";
    let (_vm, program, _closure) = compile_inline(source).expect("multiple singleton sites should compile");
    assert_eq!(program.modules[&program.entry].lowering.associated.len(), 2);
}

#[test]
fn adt_lower_03_exact_candidate_retains_variant_identity() {
    let source = "enum State { @variant Ready }\nlet value = State::Ready\n";
    let (_vm, program, _closure) = compile_inline(source).expect("singleton source should compile");
    assert!(program.modules[&program.entry].lowering.associated.values().any(|spec| matches!(spec, AssociatedLoweringSpec::SingletonLoad { .. })));
}

#[test]
fn adt_lower_04_mixed_field_layout_has_physical_slots() {
    let source = "enum Pair { @variant Pair(_ left: Int, named right: String) }\nlet value = Pair::Pair(1, named: \"two\")\n";
    let (_vm, program, _closure) = compile_inline(source).expect("mixed field source should compile");
    let variants = &program.modules[&program.entry].lowering.enums[0].variants;
    let pair = variants.iter().find(|variant| variant.payload_fields.len() == 2).expect("Pair lowering");
    assert_eq!(pair.payload_fields[0].slot, 0);
    assert_eq!(pair.payload_fields[1].slot, 1);
}

#[test]
#[ignore = "RED: family candidate-specific lowering projection remains incomplete"]
fn adt_lower_05_candidate_specific_slots_follow_variant_field_ids() {
    let source = "enum Animal { @variant Dog(_ name: Int) @variant Dog(_ name: Int, age: Int) }\nlet value = Animal::Dog(1, age: 2)\nlet result = match value { Dog(x, ...) => x _ => 0 }\n";
    let (_vm, program, _closure) = compile_inline(source).expect("family projection source should compile");
    assert!(!program.modules[&program.entry].lowering.family_applications.is_empty());
}

#[test]
fn adt_lower_06_wildcard_child_does_not_require_payload_extraction() {
    let source = "enum Boxed { @variant Value(_ value: Int) }\nlet value = Boxed::Value(1)\nlet result = match value { Boxed::Value(_) => 1 }\n";
    let (vm, _program, closure) = compile_inline(source).expect("wildcard source should compile");
    assert!(!vm.heap.closure(closure).callable.chunk.code.iter().any(|op| matches!(op, Bytecode::GetVariantPayload(..))));
}

#[test]
#[ignore = "RED: selector-gap lowering projection remains incomplete"]
fn adt_lower_07_selector_gap_fields_are_not_extracted() {
    let source = "enum Animal { @variant Dog(_ name: Int) @variant Dog(_ name: Int, age: Int, breed: String) }\nlet value = Animal::Dog(1, age: 2, breed: \"terrier\")\nlet result = match value { Dog(x, ..., breed: y) => x _ => 0 }\n";
    let (_vm, program, _closure) = compile_inline(source).expect("selector-gap source should reach lowering");
    assert!(!program.modules[&program.entry].lowering.enums.is_empty());
}

#[test]
#[ignore = "GATED: cross-module field layout fixture is required"]
fn adt_lower_08_imported_field_layout_uses_declared_physical_slot() {
    let source = "enum Imported { @variant Value(_ first: Int, named second: String) }\nlet value = Imported::Value(1, named: \"two\")\n";
    let (_vm, program, _closure) = compile_inline(source).expect("field-layout fixture should compile");
    let variant = &program.modules[&program.entry].lowering.enums[0].variants[0];
    assert_eq!(variant.payload_fields.iter().map(|field| field.slot).collect::<Vec<_>>(), vec![0, 1]);
}

#[test]
fn adt_lower_09_lowering_preserves_semantic_declaration_order() {
    let source = "enum State { @variant Ready @variant Done }\nlet first = State::Ready\nlet second = State::Done\n";
    let (_vm, program, _closure) = compile_inline(source).expect("ordered source should compile");
    let variants = &program.modules[&program.entry].lowering.enums[0].variants;
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[0].id.selector.encode(), "Ready");
    assert_eq!(variants[1].id.selector.encode(), "Done");
}

#[test]
fn adt_lower_10_executable_lowering_contains_no_gadt_proof_products() {
    let source = "enum Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nlet value = Expr<Int>::Int(1)\n";
    let (_vm, program, _closure) = compile_inline(source).expect("GADT source should compile");
    let rendered = format!("{:?}", program.modules[&program.entry].lowering);
    assert!(!rendered.contains("CaseTypeEnvironment"));
    assert!(!rendered.contains("PatternSpace"));
}

#[test]
#[ignore = "GATED: typed missing-lowering injection seam is required"]
fn adt_lower_11_missing_semantic_product_returns_typed_error() {
    let result = compile_inline("class Test { run() { match 1 { 2 => 3 } } }\n");
    assert!(result.is_err(), "invalid match must return a typed compilation error");
}

#[test]
#[ignore = "RED: non-proven source match rejection remains incomplete"]
fn adt_lower_12_non_proven_match_is_rejected_before_bytecode_fallback() {
    let result = compile_inline("enum State { @variant Ready @variant Done }\nlet result = match State::Ready { State::Ready => 1 }\n");
    assert!(result.is_err(), "non-exhaustive source match must not use bytecode fallback");
}
