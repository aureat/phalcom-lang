//! Compiler lowering conformance for associated ADT expressions (Part 4).

use phalcom_common::range::SourceRange;
use phalcom_core::bytecode::Bytecode;
use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use phalcom_core::vm::VM;
use std::sync::Arc;

fn compile_inline(source: &str) -> Result<(VM, CompiledProgram, phalcom_core::heap::ObjRef), PhError> {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    let mut vm = VM::new();
    vm.materialize_program(&program)?;
    let closure = vm.compile_program_module_closure(&program.entry, source, &program)?;
    Ok((vm, program, closure))
}

#[test]
fn three_variant_forms_use_distinct_lowering_paths() {
    let source = r#"
enum Weird {
  @variant Marker
  @variant Marker()
  @variant Marker(_ value: Int)
}

let singleton = Weird::Marker
let nullary = Weird::Marker()
let payload = Weird::Marker(1)
"#;
    let (vm, program, closure) = compile_inline(source).expect("source should compile");
    let chunk = &vm.heap.closure(closure).callable.chunk;

    let singleton_loads = chunk.code.iter().filter(|op| matches!(op, Bytecode::LoadVariantSingleton(_))).count();
    let nullary_constructs = chunk.code.iter().filter(|op| matches!(op, Bytecode::ConstructVariant { arity: 0, .. })).count();
    let payload_constructs = chunk.code.iter().filter(|op| matches!(op, Bytecode::ConstructVariant { arity: 1, .. })).count();

    assert_eq!(singleton_loads, 1, "bare singleton must load canonical value");
    assert_eq!(nullary_constructs, 1, "zero-arg constructor must allocate case");
    assert_eq!(payload_constructs, 1, "payload constructor must allocate case");
    assert!(!chunk.code.iter().any(|op| matches!(op, Bytecode::MakeFamily { .. })));

    let lowering = &program.modules[&program.entry].lowering;
    let associated_kinds: Vec<_> = lowering.associated.keys().map(|site| site.kind).collect();
    assert_eq!(associated_kinds.len(), 3);
    assert!(associated_kinds.iter().all(|kind| {
        matches!(
            kind,
            phalcom_core::modules::semantic_lowering::LoweringSiteKind::AssociatedLookup
                | phalcom_core::modules::semantic_lowering::LoweringSiteKind::AssociatedInvoke
        )
    }));
    assert!(lowering.associated.keys().all(|site| site.range != SourceRange::default()));
}
