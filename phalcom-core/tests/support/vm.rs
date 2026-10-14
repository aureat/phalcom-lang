//! Shared VM/compiler helpers for semantic responsibility-owned tests.

use phalcom_core::error::PhError;
use phalcom_core::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use phalcom_core::vm::VM;
use std::sync::Arc;
#[cfg(feature = "test-timings")]
use std::time::{Duration, Instant};

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

#[cfg(feature = "test-timings")]
fn print_inline_timing(
    source_bytes: usize,
    vm_new: Duration,
    bootstrap: phalcom_core::instrumentation::VmBootstrapTiming,
    inline_compile: Duration,
    inline_execution: Option<Duration>,
) {
    let enabled = std::env::var("PHALCOM_TEST_TIMINGS")
        .map(|value| value == "1" || value.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if !enabled {
        return;
    }

    let measured_bootstrap =
        bootstrap.canonical_universe_product + bootstrap.kernel_native + bootstrap.universe_module_compilation + bootstrap.universe_initializer_execution;
    let vm_unaccounted = vm_new.saturating_sub(measured_bootstrap);
    let current_thread = std::thread::current();
    let test_name = current_thread.name().unwrap_or("unnamed-test");
    eprintln!(
        "[INLINE-TIMING] test={test_name} source_bytes={source_bytes} vm_new={vm_new:?} canonical_product={:?} kernel_native={:?} universe_compile={:?} universe_initializers={:?} vm_unaccounted={vm_unaccounted:?} inline_compile={inline_compile:?} inline_execution={inline_execution:?}",
        bootstrap.canonical_universe_product, bootstrap.kernel_native, bootstrap.universe_module_compilation, bootstrap.universe_initializer_execution,
    );
}

pub(crate) fn run_inline(source: &str) -> Result<(VM, phalcom_core::heap::ObjRef), PhError> {
    #[cfg(feature = "test-timings")]
    let vm_started = Instant::now();
    let mut vm = universe_vm();
    #[cfg(feature = "test-timings")]
    let vm_new = vm_started.elapsed();
    #[cfg(feature = "test-timings")]
    let bootstrap = vm.bootstrap_timing();

    #[cfg(feature = "test-timings")]
    let compile_started = Instant::now();
    let program_result = ProgramCompiler::compile_entry_selection(EntrySelection::Inline(Arc::from(source)));
    #[cfg(feature = "test-timings")]
    let inline_compile = compile_started.elapsed();
    let program = match program_result {
        Ok(program) => program,
        Err(error) => {
            #[cfg(feature = "test-timings")]
            print_inline_timing(source.len(), vm_new, bootstrap, inline_compile, None);
            return Err(PhError::from(error));
        }
    };

    #[cfg(feature = "test-timings")]
    let execution_started = Instant::now();
    let execution_result = vm.run_compiled(&program);
    #[cfg(feature = "test-timings")]
    let inline_execution = execution_started.elapsed();
    #[cfg(feature = "test-timings")]
    print_inline_timing(source.len(), vm_new, bootstrap, inline_compile, Some(inline_execution));
    execution_result?;

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
