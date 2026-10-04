//! Explicit semantic-to-core lowering scenarios for ADT match products.

use super::vm_support::compile_inline;
use phalcom_core::bytecode::Bytecode;
use phalcom_core::modules::semantic_lowering::{AssociatedLoweringSpec, LoweringSiteKind};
use phalcom_semantic::ConstructorOrigin;

#[test]
fn adt_lower_01_one_associated_site_has_one_lowering_record() {
    let source = "data State { @variant Ready }\nlet value = State::Ready\n";
    let (vm, program, _closure) = compile_inline(source).expect("singleton source should compile");
    let lowering = &program.modules[&program.entry].lowering;
    assert_eq!(lowering.associated.len(), 1);
    assert!(lowering.associated.keys().all(|site| site.kind == LoweringSiteKind::AssociatedLookup));
    let _ = vm;
}

#[test]
fn adt_lower_02_multiple_sites_keep_distinct_source_keys() {
    let source = "data State { @variant Ready }\nlet first = State::Ready\nlet second = State::Ready\n";
    let (_vm, program, _closure) = compile_inline(source).expect("multiple singleton sites should compile");
    assert_eq!(program.modules[&program.entry].lowering.associated.len(), 2);
}

#[test]
fn adt_lower_03_exact_candidate_retains_variant_identity() {
    let source = "data State { @variant Ready }\nlet value = State::Ready\n";
    let (_vm, program, _closure) = compile_inline(source).expect("singleton source should compile");
    assert!(
        program.modules[&program.entry]
            .lowering
            .associated
            .values()
            .any(|spec| matches!(spec, AssociatedLoweringSpec::SingletonLoad { .. }))
    );
}

#[test]
fn adt_lower_04_mixed_field_layout_has_physical_slots() {
    let source = "data Pair { @variant Pair(_ left: Int, named right: String) }\nlet value = Pair::Pair(1, named: \"two\")\n";
    let (_vm, program, _closure) = compile_inline(source).expect("mixed field source should compile");
    let variants = &program.modules[&program.entry].lowering.enums[0].variants;
    let pair = variants.iter().find(|variant| variant.payload_fields.len() == 2).expect("Pair lowering");
    assert_eq!(pair.payload_fields[0].slot, 0);
    assert_eq!(pair.payload_fields[1].slot, 1);
}

#[test]
#[ignore = "RED: family candidate-specific lowering projection remains incomplete"]
fn adt_lower_05_candidate_specific_slots_follow_variant_field_ids() {
    let source = "data Animal { @variant Dog(_ name: Int) @variant Dog(_ name: Int, age: Int) }\nlet value = Animal::Dog(1, age: 2)\nlet result = match value { Dog(x, ...) => x _ => 0 }\n";
    let (_vm, program, _closure) = compile_inline(source).expect("family projection source should compile");
    assert!(!program.modules[&program.entry].lowering.family_applications.is_empty());
}

#[test]
fn adt_lower_06_wildcard_child_does_not_require_payload_extraction() {
    let source = "data Boxed { @variant Value(_ value: Int) }\nlet value = Boxed::Value(1)\nlet result = match value { Boxed::Value(_) => 1 }\n";
    let (vm, _program, closure) = compile_inline(source).expect("wildcard source should compile");
    assert!(
        !vm.heap
            .closure(closure)
            .callable
            .chunk
            .code
            .iter()
            .any(|op| matches!(op, Bytecode::GetVariantPayload(..)))
    );
}

#[test]
#[ignore = "RED: selector-gap lowering projection remains incomplete"]
fn adt_lower_07_selector_gap_fields_are_not_extracted() {
    let source = "data Animal { @variant Dog(_ name: Int) @variant Dog(_ name: Int, age: Int, breed: String) }\nlet value = Animal::Dog(1, age: 2, breed: \"terrier\")\nlet result = match value { Dog(x, ..., breed: y) => x _ => 0 }\n";
    let (_vm, program, _closure) = compile_inline(source).expect("selector-gap source should reach lowering");
    assert!(!program.modules[&program.entry].lowering.enums.is_empty());
}

#[test]
#[ignore = "GATED: cross-module field layout fixture is required"]
fn adt_lower_08_imported_field_layout_uses_declared_physical_slot() {
    let source = "data Imported { @variant Value(_ first: Int, named second: String) }\nlet value = Imported::Value(1, named: \"two\")\n";
    let (_vm, program, _closure) = compile_inline(source).expect("field-layout fixture should compile");
    let variant = &program.modules[&program.entry].lowering.enums[0].variants[0];
    assert_eq!(variant.payload_fields.iter().map(|field| field.slot).collect::<Vec<_>>(), vec![0, 1]);
}

#[test]
fn adt_lower_09_lowering_preserves_semantic_declaration_order() {
    let source = "data State { @variant Ready @variant Done }\nlet first = State::Ready\nlet second = State::Done\n";
    let (_vm, program, _closure) = compile_inline(source).expect("ordered source should compile");
    let variants = &program.modules[&program.entry].lowering.enums[0].variants;
    assert_eq!(variants.len(), 2);
    assert_eq!(variants[0].id.selector.encode(), "Ready");
    assert_eq!(variants[1].id.selector.encode(), "Done");
}

#[test]
fn explicit_field_layout_preserves_declaration_order_and_mutability() {
    let source = "class Slots {\n  _first: Int\n  mut _second: Int\n}\n";
    let (vm, _program, _closure) = compile_inline(source).expect("explicit fields should compile");
    let layout = vm
        .field_layouts
        .values()
        .find(|layout| vm.resolve_symbol(layout.name) == "Slots")
        .expect("Slots layout");
    let names = layout.field_slots.keys().map(|name| vm.resolve_symbol(*name)).collect::<Vec<_>>();
    assert_eq!(names, ["_first", "_second"]);
    assert_eq!(layout.immutable_instance_slots.len(), 1);
    let immutable_slot = *layout.immutable_instance_slots.iter().next().expect("immutable slot");
    assert_eq!(immutable_slot, 0);
}

#[test]
fn undeclared_assignment_cannot_add_a_field_layout_slot() {
    let source = "class Missing { @constructor new() { _inferred = 1 } }\n";
    let error = match compile_inline(source) {
        Ok(_) => panic!("assignment must not create a field layout entry"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("FieldNotDeclared"), "{error}");
}

#[test]
fn compiler_lowering_receives_canonical_constructor_products() {
    let source = "@construct\nclass Constructed {\n  _required: Int\n  _defaulted: String = \"default\"\n}\n";
    let (_vm, program, _closure) = compile_inline(source).expect("constructor class should compile");
    let products = &program.modules[&program.entry].lowering.constructor_initializations;
    let product = products
        .iter()
        .map(|(_, product)| product)
        .find(|product| product.origin == ConstructorOrigin::Construct)
        .expect("compiler lowering receives the semantic @construct product");
    assert_eq!(
        product.factory.selector.slots.as_ref(),
        &[phalcom_common::selector::SelectorSlot::Label("required".into())]
    );
    assert_eq!(
        product.initializer.selector.slots.as_ref(),
        &[phalcom_common::selector::SelectorSlot::Label("required".into())]
    );
    assert_eq!(product.default_fields.len(), 1);
    assert_eq!(product.default_fields[0].name.as_ref(), "_defaulted");
    assert_eq!(product.parameter_fields.len(), 1);
    assert_eq!(product.parameter_fields[0].field.name.as_ref(), "_required");
}

#[test]
fn compiler_installs_and_executes_canonical_constructor_projections() {
    let source = r#"
@construct
class Constructed {
  _required: Int
  _defaulted: Int = 4
}

@data
class DataValue {
  _item: Int
}

class ParentDefaults {
  _base: Int = 1
}

class ChildDefaults is ParentDefaults {
  _own: Int = 2
  @constructor new() { super.new() }
}

let made = Constructed.new(required: 7)
let data_value = DataValue.new(item: 3)
let parent = ParentDefaults.new()
let child = ChildDefaults.new()
"#;
    let (mut vm, program, closure) = compile_inline(source).expect("constructor projections should compile");
    let module = vm.entry_module().expect("entry module materialized");
    vm.run_in_module(module, closure).expect("canonical constructor paths should execute");

    let products = &program.modules[&program.entry].lowering.constructor_initializations;
    let installed = ["Constructed", "DataValue", "ParentDefaults", "ChildDefaults"];
    for class_name in installed {
        let product = products
            .iter()
            .map(|(_, product)| product)
            .find(|product| product.factory.try_declaration_owner().is_some_and(|owner| owner.name.as_ref() == class_name))
            .unwrap_or_else(|| panic!("semantic product for {class_name}"));
        let class_symbol = vm.get_or_intern(class_name);
        let class_ref = vm
            .heap
            .module(module)
            .get(class_symbol)
            .and_then(|value| value.as_obj())
            .unwrap_or_else(|| panic!("runtime class {class_name}"));
        let class_object = vm.heap.class(class_ref);
        let metaclass = class_object.class;
        let factory_symbol = vm.interner.find(&product.factory.selector.encode()).expect("factory selector interned");
        let initializer_symbol = vm.interner.find(&product.initializer.selector.encode()).expect("initializer selector interned");
        assert!(vm.heap.class(metaclass).methods.contains_key(&factory_symbol), "factory selector installed for {class_name}");
        assert!(class_object.methods.contains_key(&initializer_symbol), "initializer selector installed for {class_name}");

        let initializer_ref = class_object.methods[&initializer_symbol];
        let initializer_closure = match &vm.heap.method(initializer_ref).kind {
            phalcom_core::method::MethodKind::Closure(closure) => *closure,
            phalcom_core::method::MethodKind::Primitive(_) => panic!("initializer is bytecode for {class_name}"),
        };
        let code = &vm.heap.closure(initializer_closure).callable.chunk.code;
        let writes = code.iter().filter(|instruction| matches!(instruction, Bytecode::SetField(_))).count();
        let expected_writes = if class_name == "Constructed" { 2 } else { 1 };
        assert_eq!(writes, expected_writes, "product field effects reach emitted initializer for {class_name}");
        if class_name == "ChildDefaults" {
            let super_call = code.iter().position(|instruction| matches!(instruction, Bytecode::SuperSend(..))).expect("resolved parent initializer call");
            let own_default = code.iter().position(|instruction| matches!(instruction, Bytecode::SetField(_))).expect("child default assignment");
            assert!(super_call < own_default, "parent initialization precedes child defaults");
        }
    }

    let made_symbol = vm.get_or_intern("made");
    let made_ref = vm.heap.module(module).get(made_symbol).and_then(|value| value.as_obj()).expect("constructed value");
    let made = vm.heap.instance(made_ref);
    let made_class = vm.heap.class(made.class);
    let required = vm.interner.find("_required").expect("required field symbol");
    let defaulted = vm.interner.find("_defaulted").expect("defaulted field symbol");
    let required_slot = usize::from(*made_class.field_slots.get(&required).expect("required slot"));
    let defaulted_slot = usize::from(*made_class.field_slots.get(&defaulted).expect("default slot"));
    assert_eq!(made.slots[required_slot].as_int(), Some(7));
    assert_eq!(made.slots[defaulted_slot].as_int(), Some(4));

    let data_symbol = vm.get_or_intern("data_value");
    let data_ref = vm.heap.module(module).get(data_symbol).and_then(|value| value.as_obj()).expect("data instance");
    let data = vm.heap.instance(data_ref);
    let data_class = vm.heap.class(data.class);
    let item = vm.interner.find("_item").expect("data field symbol");
    let item_slot = usize::from(*data_class.field_slots.get(&item).expect("data field slot"));
    assert_eq!(data.slots[item_slot].as_int(), Some(3));

    let child_symbol = vm.get_or_intern("child");
    let child_ref = vm.heap.module(module).get(child_symbol).and_then(|value| value.as_obj()).expect("child instance");
    let child = vm.heap.instance(child_ref);
    let child_class = vm.heap.class(child.class);
    let base = vm.interner.find("_base").expect("base field symbol");
    let own = vm.interner.find("_own").expect("own field symbol");
    let base_slot = usize::from(*child_class.field_slots.get(&base).expect("base slot"));
    let own_slot = usize::from(*child_class.field_slots.get(&own).expect("own slot"));
    assert_eq!(child.slots[base_slot].as_int(), Some(1));
    assert_eq!(child.slots[own_slot].as_int(), Some(2));
}

#[test]
fn compiler_lowering_receives_resolved_parent_initializer_call_site() {
    let source = "class Parent { _base: Int = 1 }\nclass Child is Parent { @constructor new() { super.new() } }\n";
    let (_vm, program, _closure) = compile_inline(source).expect("explicit parent initializer should compile");
    let lowering = &program.modules[&program.entry].lowering;
    assert_eq!(lowering.constructor_super_initializers.len(), 1);
    let initializer = lowering.constructor_super_initializers.values().next().expect("resolved initializer");
    assert_eq!(initializer.selector.encode(), "init new()");
}

#[test]
fn adt_lower_10_executable_lowering_contains_no_gadt_proof_products() {
    let source = "data Expr<T> { @variant Int(_ value: Int) -> Expr<Int> @variant Bool(_ value: Bool) -> Expr<Bool> }\nlet value = Expr<Int>::Int(1)\n";
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
    let result = compile_inline("data State { @variant Ready @variant Done }\nlet result = match State::Ready { State::Ready => 1 }\n");
    assert!(result.is_err(), "non-exhaustive source match must not use bytecode fallback");
}

#[test]
fn lexical_if_and_match_branches_lower_inline_without_closure_construction() {
    let source = "data State { @variant Ready @variant Done }\nconst conditional = if true { 1 } else { 2 }\nconst matched = match State::Ready { State::Ready => { 3 } State::Done => { 4 } }\n";
    let (vm, _program, closure) = compile_inline(source).expect("lexical control regions should compile");
    let code = &vm.heap.closure(closure).callable.chunk.code;
    assert!(
        !code.iter().any(|instruction| matches!(instruction, Bytecode::Closure(_))),
        "lexical if/match bodies execute inline and must not construct Closure values: {code:?}"
    );

    let (vm, _program, closure) = compile_inline("const action = || { 5 }\n").expect("explicit Closure should compile");
    let code = &vm.heap.closure(closure).callable.chunk.code;
    assert!(
        code.iter().any(|instruction| matches!(instruction, Bytecode::Closure(_))),
        "explicit Closure remains a first-class value"
    );
}
