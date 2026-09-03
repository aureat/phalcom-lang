//! Shared VM/compiler helpers for semantic responsibility-owned tests.

use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use phalcom_core::vm::VM;
use std::sync::Arc;

/// Creates kernel-only VM state for tests that do not need native primitives
/// or source-authored Universe behavior.
#[allow(dead_code)]
pub(crate) fn kernel_vm() -> VM {
    VM::new_kernel()
}

/// Creates native-floor VM state for tests that need registered primitives but
/// do not need source-authored Universe behavior.
#[allow(dead_code)]
pub(crate) fn native_vm() -> VM {
    VM::new_native()
}

/// Creates full shipping VM state for source-language runtime tests.
pub(crate) fn universe_vm() -> VM {
    VM::new()
}

pub(crate) fn run_inline(source: &str) -> Result<(VM, phalcom_core::heap::ObjRef), PhError> {
    let mut vm = universe_vm();
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    vm.run_compiled(&program)?;
    let entry_id = program.initialization_order.last().expect("entry module");
    let module = vm.module_registry.get(entry_id).expect("entry module registered").object;
    Ok((vm, module))
}

pub(crate) fn compile_inline(source: &str) -> Result<(VM, CompiledProgram, phalcom_core::heap::ObjRef), PhError> {
    let program = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source))).map_err(PhError::from)?;
    let mut vm = universe_vm();
    vm.materialize_program(&program)?;
    let closure = vm.compile_program_module_closure(&program.entry, source, &program)?;
    Ok((vm, program, closure))
}
