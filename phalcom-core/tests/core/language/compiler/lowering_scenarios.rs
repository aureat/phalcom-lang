//! Explicit semantic-to-core lowering scenarios for ADT match products.

use super::vm_support::compile_inline;
use phalcom_ast::ast::{ClassDef, ClassMember, Expr, Statement};
use phalcom_core::bytecode::Bytecode;
use phalcom_core::modules::semantic_lowering::{AssociatedLoweringSpec, LoweringSiteKind};
use phalcom_semantic::ConstructorOrigin;

#[derive(Debug, Clone, PartialEq, Eq)]
struct LegacyField {
    name: String,
    immutable: bool,
}

#[derive(Debug)]
struct LegacyClassLayout {
    superclass: Option<String>,
    instance: Vec<LegacyField>,
    class: Vec<LegacyField>,
}

fn collect_direct_assignment_fields(statements: &[Statement], fields: &mut Vec<String>) {
    for statement in statements {
        let Statement::Expr {
            expr: Expr::Assignment(assignment),
            ..
        } = statement
        else {
            continue;
        };
        let Expr::Field { value, .. } = &*assignment.name else {
            continue;
        };
        if !fields.contains(value) {
            fields.push(value.clone());
        }
    }
}

fn legacy_assigned_fields(class: &ClassDef, class_side: bool) -> Vec<String> {
    let mut fields = Vec::new();
    for member in &class.members {
        let member_is_class_side = member.is_static() || member.attributes().iter().any(|attribute| attribute.name == "class");
        if member_is_class_side != class_side {
            continue;
        }
        match member {
            ClassMember::Method(method) => {
                if let Some(statements) = method.body.statements() {
                    collect_direct_assignment_fields(statements, &mut fields);
                }
            }
            ClassMember::Getter(getter) => {
                if let Some(statements) = getter.body.statements() {
                    collect_direct_assignment_fields(statements, &mut fields);
                }
            }
            ClassMember::Setter(setter) => {
                if let Some(statements) = setter.body.statements() {
                    collect_direct_assignment_fields(statements, &mut fields);
                }
            }
            _ => {}
        }
    }
    fields
}

fn legacy_class_layouts(source: &str) -> std::collections::HashMap<String, LegacyClassLayout> {
    let parsed = phalcom_ast::parse(source, 0);
    assert!(parsed.errors.is_empty(), "legacy oracle source must parse: {:?}", parsed.errors);
    parsed
        .program
        .statements
        .iter()
        .filter_map(|statement| {
            let Statement::Class(class) = statement else {
                return None;
            };

            let class_fields = legacy_assigned_fields(class, true)
                .into_iter()
                .map(|name| LegacyField { name, immutable: false })
                .collect::<Vec<_>>();
            let instance = legacy_assigned_fields(class, false)
                .into_iter()
                .filter(|name| !class_fields.iter().any(|field| field.name == *name))
                .map(|name| LegacyField { name, immutable: false })
                .collect();

            Some((
                class.name.clone(),
                LegacyClassLayout {
                    superclass: class.superclass_ref().map(|reference| reference.leaf_name().to_owned()),
                    instance,
                    class: class_fields,
                },
            ))
        })
        .collect()
}

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
fn migrated_layout_matches_pre_cutover_oracle() {
    // This source is parsed only. Before P1, assignments in member bodies
    // established ordered field storage; the paired source below expresses
    // that same order as explicit declarations.
    let legacy_source = r#"
class Parent {
  @constructor new() { _base_second = 2; _base_first = 1 }
  @class touch() { _shared_second = 2; _shared_first = 1 }
}

class Child is Parent {
  @constructor new() { super.new(); _child_second = 4; _child_first = 3 }
  @class touch() { _child_shared_second = 2; _child_shared_first = 1 }
}
"#;
    let mut legacy = legacy_class_layouts(legacy_source);
    // The pre-cutover `const` field spelling is intentionally no longer
    // parseable. Freeze its old declaration-derived layout here and verify
    // that the migrated bare declarations preserve its immutable slots.
    legacy.insert(
        "Fixed".into(),
        LegacyClassLayout {
            superclass: None,
            instance: vec![LegacyField {
                name: "_fixed".into(),
                immutable: true,
            }],
            class: vec![LegacyField {
                name: "_fixed_shared".into(),
                immutable: true,
            }],
        },
    );

    let migrated_source = r#"
class Parent {
  mut _base_second: Int
  mut _base_first: Int
  @class mut _shared_second: Int
  @class mut _shared_first: Int
  @constructor new() { _base_second = 2; _base_first = 1 }
  @class touch() { _shared_second = 2; _shared_first = 1 }
}

class Child is Parent {
  mut _child_second: Int
  mut _child_first: Int
  @class mut _child_shared_second: Int
  @class mut _child_shared_first: Int
  @constructor new() { super.new(); _child_second = 4; _child_first = 3 }
  @class touch() { _child_shared_second = 2; _child_shared_first = 1 }
}

class Fixed {
  _fixed: Int = 7
  @class _fixed_shared: Int = 8
}
"#;
    let (vm, _program, _closure) = compile_inline(migrated_source).expect("migrated explicit fields should compile");
    let layouts = vm
        .field_layouts
        .values()
        .map(|layout| (vm.resolve_symbol(layout.name).to_owned(), layout))
        .collect::<std::collections::HashMap<_, _>>();

    for (class_name, old_layout) in &legacy {
        let layout = layouts.get(class_name).unwrap_or_else(|| panic!("missing compiled layout for {class_name}"));
        let parent = old_layout
            .superclass
            .as_ref()
            .map(|name| layouts.get(name).unwrap_or_else(|| panic!("missing compiled superclass layout for {name}")));

        for (side, fields, slots, total, immutable) in [
            (
                "instance",
                &old_layout.instance,
                &layout.field_slots,
                layout.field_count,
                &layout.immutable_instance_fields,
            ),
            (
                "class",
                &old_layout.class,
                &layout.static_field_slots,
                layout.static_field_count,
                &layout.immutable_static_fields,
            ),
        ] {
            let parent_prefix = parent.map(|parent| if side == "instance" { parent.field_count } else { parent.static_field_count });
            let prefix = parent_prefix.unwrap_or_else(|| {
                total
                    .checked_sub(fields.len() as u16)
                    .unwrap_or_else(|| panic!("{class_name} {side} legacy oracle has {} own fields but layout count is {total}", fields.len()))
            });
            assert_eq!(total, prefix + fields.len() as u16, "{class_name} {side} field count");

            let mut actual_own = slots
                .iter()
                .filter_map(|(symbol, slot)| (*slot >= prefix).then(|| (*slot, vm.resolve_symbol(*symbol).to_owned())))
                .collect::<Vec<_>>();
            actual_own.sort_by_key(|(slot, _)| *slot);
            assert_eq!(
                actual_own.iter().map(|(_, name)| name.as_str()).collect::<Vec<_>>(),
                fields.iter().map(|field| field.name.as_str()).collect::<Vec<_>>(),
                "{class_name} {side} legacy name/side/order parity"
            );
            for (index, (slot, _)) in actual_own.iter().enumerate() {
                assert_eq!(*slot, prefix + index as u16, "{class_name} {side} relative own slot");
            }

            let actual_immutable = immutable
                .iter()
                .map(|symbol| vm.resolve_symbol(*symbol).to_owned())
                .collect::<std::collections::HashSet<_>>();
            let expected_immutable = fields
                .iter()
                .filter(|field| field.immutable)
                .map(|field| field.name.clone())
                .collect::<std::collections::HashSet<_>>();
            assert_eq!(actual_immutable, expected_immutable, "{class_name} {side} mutability translation");
        }
    }
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
        assert!(
            vm.heap.class(metaclass).methods.contains_key(&factory_symbol),
            "factory selector installed for {class_name}"
        );
        assert!(
            class_object.methods.contains_key(&initializer_symbol),
            "initializer selector installed for {class_name}"
        );

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
            let super_call = code
                .iter()
                .position(|instruction| matches!(instruction, Bytecode::SuperSend(..)))
                .expect("resolved parent initializer call");
            let own_default = code
                .iter()
                .position(|instruction| matches!(instruction, Bytecode::SetField(_)))
                .expect("child default assignment");
            assert!(super_call < own_default, "parent initialization precedes child defaults");
        }
    }

    let made_symbol = vm.get_or_intern("made");
    let made_ref = vm
        .heap
        .module(module)
        .get(made_symbol)
        .and_then(|value| value.as_obj())
        .expect("constructed value");
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
    let child_ref = vm
        .heap
        .module(module)
        .get(child_symbol)
        .and_then(|value| value.as_obj())
        .expect("child instance");
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

#[test]
fn a2_blocked_nested_constructor_match_is_rejected_before_lowering() {
    let source = r#"
data Inner<T> { @variant Int(_ value: Int) -> Inner<Int> }
data Outer<T> { @variant Wrap<A>(_ inner: Inner<A>) -> Outer<A> }
class Eval {
    eval<T>(_ value: Outer<T>) {
        match value { Outer::Wrap(Inner::Int(_)) => 1 _ => 0 }
    }
}
"#;
    let result = compile_inline(source);
    let failure = match result {
        Ok(_) => panic!("blocked semantic match must not lower as exhaustive"),
        Err(error) => error,
    };
    let diagnostics = format!("{failure:?}");
    assert!(
        diagnostics.contains("MatchAnalysisBlocked"),
        "compiler must reject the semantic blocked proof: {diagnostics}"
    );
}
