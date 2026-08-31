//! Shared VM/compiler helpers for semantic responsibility-owned tests.

use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use phalcom_core::vm::VM;
use std::sync::Arc;

pub(crate) fn run_inline(source: &str) -> Result<(VM, phalcom_core::heap::ObjRef), PhError> {
    let mut vm = VM::new();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    vm.run_compiled(&program)?;
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;
    Ok((vm, module))
}

pub(crate) fn compile_inline(source: &str) -> Result<(VM, CompiledProgram, phalcom_core::heap::ObjRef), PhError> {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    let mut vm = VM::new();
    vm.materialize_program(&program)?;
    let closure = vm.compile_program_module_closure(&program.entry, source, &program)?;
    Ok((vm, program, closure))
}
