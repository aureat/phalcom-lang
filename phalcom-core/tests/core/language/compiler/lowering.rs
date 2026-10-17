//! Formal semantic snapshot to backend lowering projection tests (Part 4).

use phalcom_common::selector::{SelectorKind, SelectorSlot};
use phalcom_core::bytecode::Bytecode;
use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use phalcom_core::modules::semantic_lowering::{AssociatedLoweringSpec, FamilyApplicationLoweringSpec, LoweringSiteKind, RuntimeTypeFormValueSpec};
use phalcom_core::typing::RuntimeTypeRecipe;
use phalcom_core::typing::handle::{MetadataPoolId, RuntimeTypeRef};
use phalcom_core::vm::VM;
use phalcom_semantic::enum_semantics::VariantShape;
use std::sync::Arc;

fn compile_inline(source: &str) -> Result<(VM, CompiledProgram, phalcom_core::heap::ObjRef), PhError> {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    let mut vm = VM::new();
    vm.materialize_program(&program)?;
    let closure = vm.compile_program_module_closure(&program.entry, source, &program)?;
    Ok((vm, program, closure))
}

#[test]
fn enum_lowering_projects_shapes_and_payload_slots_from_formal_semantics() {
    let source = r#"
data Weird {
  Marker
  Marker()
  Marker(_ left: Int, _ right: Int)
}

let singleton = Weird::Marker
let nullary = Weird::Marker()
let payload = Weird::Marker(1, 2)
"#;
    let (vm, program, closure) = compile_inline(source).expect("source should compile");
    let lowering = &program.modules[&program.entry].lowering;
    assert_eq!(lowering.enums.len(), 1);
    let variants = &lowering.enums[0].variants;
    assert_eq!(variants.len(), 3);
    assert_eq!(variants[0].shape, VariantShape::Singleton);
    assert_eq!(variants[1].shape, VariantShape::Constructor);
    assert_eq!(variants[2].shape, VariantShape::Constructor);
    assert!(variants[0].payload_fields.is_empty());
    assert!(variants[1].payload_fields.is_empty());
    assert_eq!(variants[2].payload_fields.len(), 2);
    assert_eq!(variants[2].payload_fields[0].slot, 0);
    assert_eq!(variants[2].payload_fields[1].slot, 1);
    assert_eq!(variants[2].payload_fields[0].local_name.as_ref(), "left");
    assert_eq!(variants[2].payload_fields[1].local_name.as_ref(), "right");

    let specs: Vec<_> = lowering.associated.values().collect();
    assert_eq!(specs.len(), 3);
    assert!(specs.iter().any(|spec| matches!(spec, AssociatedLoweringSpec::SingletonLoad { .. })));
    assert!(
        specs
            .iter()
            .any(|spec| matches!(spec, AssociatedLoweringSpec::ConstructVariant { arity: 0, .. }))
    );
    assert!(
        specs
            .iter()
            .any(|spec| matches!(spec, AssociatedLoweringSpec::ConstructVariant { arity: 2, .. }))
    );

    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(chunk.code.iter().any(|op| matches!(op, Bytecode::LoadVariantSingleton(_))));
    assert!(chunk.code.iter().any(|op| matches!(op, Bytecode::ConstructVariant { arity: 0, .. })));
    assert!(chunk.code.iter().any(|op| matches!(op, Bytecode::ConstructVariant { arity: 2, .. })));
}

#[test]
fn class_side_call_uses_ordinary_dispatch_without_associated_lowering() {
    let source = r#"
class Factory {
  @class make(value: Int) -> Int {
    value
  }
}

let result = Factory.make(value: 1)
"#;
    let (vm, program, closure) = compile_inline(source).expect("source should compile");
    let lowering = &program.modules[&program.entry].lowering;
    assert!(lowering.associated.is_empty(), "ordinary class-side calls must not use associated lowering");
    assert!(
        lowering.callable_references.is_empty(),
        "ordinary class-side calls must not use reference lowering"
    );

    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(chunk.code.iter().any(|op| matches!(op, Bytecode::Invoke(1, _))));
    assert!(!chunk.code.iter().any(|op| matches!(op, Bytecode::InvokeResolvedAssociated { .. })));
    assert!(!chunk.code.iter().any(|op| matches!(op, Bytecode::MakeFamily { .. })));
}

#[test]
fn applied_type_form_lowers_to_ordinary_class_object_reference() {
    let source = "let values = Set<Int>.new()";
    let (vm, program, closure) = compile_inline(source).expect("applied class-side construction should compile");
    let lowering = &program.modules[&program.entry].lowering;
    assert!(
        lowering
            .type_form_values
            .values()
            .any(|spec| { matches!(spec, RuntimeTypeFormValueSpec::ClassObject { declaration } if declaration.name.as_ref() == "Set") })
    );

    let chunk = &vm.heap.closure(closure).callable.chunk;
    assert!(matches!(chunk.code.first(), Some(Bytecode::GetLinked(_))));
    assert!(matches!(chunk.code.get(1), Some(Bytecode::InvokeSpecialized(_, _, _))));
}

#[test]
fn concrete_generic_getter_preserves_closed_call_environment() {
    let source = r#"
let value: Option<Int> = Option<Int>::Some(1)
let result = value.isSome
"#;
    let (_vm, program, closure) = compile_inline(source).expect("generic getter should compile");
    let lowering = &program.modules[&program.entry].lowering;
    let recipe = lowering
        .call_environment_recipes
        .values()
        .find(|recipe| recipe.callable.selector.as_ref() == "isSome")
        .expect("Option.isSome must retain its call-entry recipe");
    assert!(
        matches!(recipe.bindings.as_ref(), [(_, RuntimeTypeRecipe::Closed(_))]),
        "concrete Option<Int>.isSome must lower Option.T as Closed(Int), got {recipe:?}"
    );

    let runtime_chunk = &_vm.heap.closure(closure).callable.chunk;
    let recipe_index = runtime_chunk
        .code
        .iter()
        .find_map(|op| match op {
            Bytecode::InvokeSpecialized(0, _, recipe) => Some(*recipe),
            _ => None,
        })
        .expect("concrete generic getter must emit InvokeSpecialized");
    let runtime_recipe = runtime_chunk.executable_semantics.call_environment_recipe(recipe_index);
    assert!(matches!(
        runtime_recipe.bindings.as_ref(),
        [(_, RuntimeTypeRecipe::Closed(RuntimeTypeRef::Base { pool, .. }))] if *pool != MetadataPoolId(0)
    ));
    assert!(
        _vm.typing_registry.pool_count() > 1,
        "full Universe VM must keep canonical and user metadata pools distinct"
    );
}

#[test]
fn generic_getter_forwards_owner_parameter_as_template() {
    let source = r#"
class Probe<T> {
  check(_ value: Option<T>) -> Bool { value.isSome }
}
"#;
    let (_vm, program, _closure) = compile_inline(source).expect("generic getter body should compile");
    let recipes: Vec<_> = program.modules[&program.entry]
        .lowering
        .call_environment_recipes
        .values()
        .filter(|recipe| recipe.callable.selector.as_ref() == "isSome")
        .collect();
    assert_eq!(recipes.len(), 1, "generic Probe should have one Option.isSome recipe");
    assert!(
        matches!(recipes[0].bindings.as_ref(), [(_, RuntimeTypeRecipe::Template(_))]),
        "generic Probe<T>.check must forward Probe.T as a template, got {:?}",
        recipes[0]
    );
}

#[test]
fn family_application_lowering_projects_static_and_dynamic_records() {
    let source = r#"
data Weird {
  Marker
  Marker()
  Marker(_ value: Int)
}

let make = &Weird::Marker...;
let static_value = make(1)
let args = [1];
let dynamic_value = make(*args)
"#;
    let (_vm, program, _closure) = compile_inline(source).expect("source should compile");
    let lowering = &program.modules[&program.entry].lowering;
    assert_eq!(lowering.family_applications.len(), 2);
    assert!(
        lowering
            .family_applications
            .keys()
            .all(|site| { site.kind == LoweringSiteKind::FamilyApplication && site.range != phalcom_common::range::SourceRange::default() })
    );

    let static_spec = lowering
        .family_applications
        .values()
        .find(|spec| matches!(spec, FamilyApplicationLoweringSpec::Static { .. }))
        .expect("static family application lowering");
    let FamilyApplicationLoweringSpec::Static { operation, target, arity, .. } = static_spec else {
        unreachable!();
    };
    assert_eq!(operation.kind, SelectorKind::Method);
    assert_eq!(operation.slots.as_ref(), [SelectorSlot::Positional]);
    assert!(target.is_some());
    assert_eq!(*arity, 1);

    let dynamic_spec = lowering
        .family_applications
        .values()
        .find(|spec| matches!(spec, FamilyApplicationLoweringSpec::DynamicPack { .. }))
        .expect("dynamic family application lowering");
    let FamilyApplicationLoweringSpec::DynamicPack { candidates, .. } = dynamic_spec else {
        unreachable!();
    };
    assert_eq!(candidates.len(), 2);
    assert!(candidates.iter().all(|candidate| candidate.operation.kind == SelectorKind::Method));
    assert!(candidates.iter().all(|candidate| candidate.target.is_some()));
    assert!(candidates[0].operation.slots.is_empty());
    assert_eq!(candidates[1].operation.slots.as_ref(), [SelectorSlot::Positional]);
}
