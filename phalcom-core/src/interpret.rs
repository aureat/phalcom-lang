//! Source-to-execution driver: compile and run programs on the [`VM`].

use crate::compiler::lib::{Compiler, CompilerError, UnitKind};
use crate::error::{PhError, PhResult};
use crate::frame::{CallContext, CallFrame};
use crate::heap::ObjRef;
use crate::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use crate::modules::{CompileBindings, ModuleState, RuntimeLinkedRead};
use crate::vm::VM;
use phalcom_ast::parse_source;
use std::sync::Arc;

pub enum ExitCode {
    Success = 0,
    GenericError = 1,
    Usage = 64,
    CompileError = 65,
    RuntimeError = 70,
    NoInput = 66,
    IOError = 74,
}

pub fn exit_success() -> ! { exit(ExitCode::Success) }
pub fn exit(code: ExitCode) -> ! { std::process::exit(code as i32) }
pub fn io_error(msg: String) { eprintln!("{msg}"); exit(ExitCode::IOError); }

pub struct Interpreter { pub vm: VM }

impl Default for Interpreter {
    fn default() -> Self { Self::new() }
}

impl Interpreter {
    pub fn new() -> Self { Self { vm: VM::new() } }

    pub fn run_entry(&mut self, entry: EntrySelection) -> PhResult<()> {
        let program = ProgramCompiler::compile_entry_selection(entry)?;
        self.vm.run_compiled(&program)
    }
}

impl VM {
    pub fn compile_closure_as(&mut self, module: ObjRef, source: &str, kind: UnitKind) -> PhResult<ObjRef> {
        self.compile_closure_as_with_bindings(module, source, kind, None)
    }

    pub fn compile_closure_as_with_bindings(
        &mut self,
        module: ObjRef,
        source: &str,
        kind: UnitKind,
        bindings: Option<CompileBindings>,
    ) -> PhResult<ObjRef> {
        self.unit_kind = kind;
        let source_id = self.heap.module_mut(module).push_source(Arc::new(source.to_string()));
        let program = parse_source(source, 0).map_err(|e| PhError::Compile(CompilerError::Parse(e)))?;
        let compiler = match bindings {
            Some(bindings) => Compiler::new_with_bindings(self, module, source_id, kind, Some(bindings)),
            None => Compiler::new(self, module, source_id, kind),
        };
        Ok(compiler.compile(program)?)
    }

    pub fn install_linked_reads(&mut self, module: ObjRef, reads: Vec<RuntimeLinkedRead>) {
        self.heap.module_mut(module).linked_reads = reads;
    }

    pub fn compile_closure(&mut self, module: ObjRef, source: &str) -> PhResult<ObjRef> {
        self.compile_closure_as(module, source, UnitKind::File)
    }

    /// Run a linked program without doing parser/compiler work for modules in a
    /// terminal lifecycle state. Failed records are deliberately left untouched
    /// so initialize_program can reproduce their sticky typed failure.
    pub fn run_compiled(&mut self, program: &CompiledProgram) -> PhResult<()> {
        self.materialize_program(program)?;
        for (id, compiled_mod) in &program.modules {
            let state = self
                .module_registry
                .get(id)
                .map(|record| record.state)
                .ok_or_else(|| crate::error::RuntimeError::Internal(format!("materialized module {id} missing from registry")))?;
            if state != ModuleState::Prepared {
                continue;
            }
            let obj = self.module_registry.get(id).expect("record observed above").object;
            if self.heap.module(obj).closure.is_some() {
                continue;
            }
            if let Some(source_text) = &compiled_mod.source_text {
                let _ = self.compile_program_module_closure(id, source_text, program)?;
            }
        }
        self.initialize_program(program)
    }

    pub fn run_in_module(&mut self, module: ObjRef, closure: ObjRef) -> PhResult<()> {
        self.frames.clear();
        self.stack.clear();
        let mut frame = CallFrame::new(closure, CallContext::Module { module }, 0, 0, None);
        frame.generation = self.next_frame_generation;
        self.next_frame_generation = self.next_frame_generation.wrapping_add(1);
        self.push_frame(frame)?;
        self.run()?;
        Ok(())
    }

    pub fn interpret_source(&mut self, module: ObjRef, source: &str) -> PhResult<()> {
        let closure = self.compile_closure(module, source).inspect_err(|err| {
            let source_id = self.heap.module(module).sources.len().saturating_sub(1) as u32;
            self.compiler_error(err.clone(), module, source_id);
        })?;
        self.run_in_module(module, closure).inspect_err(|err| { let _ = self.runtime_error(err.clone()); })?;
        Ok(())
    }
}
