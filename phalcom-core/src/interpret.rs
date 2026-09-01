//! Source-to-execution driver: compile and run programs on the [`VM`].
//!
//! This is the top-level entry the CLI and REPL call.

use crate::compiler::lib::{Compiler, CompilerError, UnitKind};
use crate::error::{PhError, PhResult};
use crate::frame::{CallContext, CallFrame};
use crate::heap::ObjRef;
use crate::modules::compile::{CompiledProgram, EntrySelection, ProgramCompiler};
use crate::modules::{CompileBindings, RuntimeLinkedRead};
use crate::vm::VM;
use phalcom_ast::ast::Program;
use phalcom_ast::parse_source;
use phalcom_modules::ImportBindingId;
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

pub fn exit_success() -> ! {
    exit(ExitCode::Success)
}

pub fn exit(code: ExitCode) -> ! {
    std::process::exit(code as i32)
}

pub fn io_error(msg: String) {
    eprintln!("{msg}");
    exit(ExitCode::IOError);
}

pub struct Interpreter {
    pub vm: VM,
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}

impl Interpreter {
    /// Creates an interpreter over a freshly bootstrapped [`VM`].
    pub fn new() -> Self {
        Self { vm: VM::new() }
    }

    /// Compiles and runs an entry selection.
    pub fn run_entry(&mut self, entry: EntrySelection) -> PhResult<()> {
        let program = ProgramCompiler::compile_entry_selection(entry)?;
        self.vm.run_compiled(&program)
    }
}

impl VM {
    /// Parses and compiles `source` for `module` with `kind`, returning the top-level
    /// closure [`ObjRef`] allocated on the [`Heap`](crate::heap::Heap).
    pub fn compile_closure_as(&mut self, module: ObjRef, source: &str, kind: UnitKind) -> PhResult<ObjRef> {
        self.compile_closure_as_with_bindings(module, source, kind, None)
    }

    /// Compiles a pre-parsed AST for `module` with `kind`.
    pub fn compile_ast_as_with_bindings(
        &mut self,
        module: ObjRef,
        source_id: u32,
        program: Program,
        kind: UnitKind,
        bindings: Option<CompileBindings>,
    ) -> PhResult<ObjRef> {
        self.unit_kind = kind;
        let bindings = self.attach_prelude_bindings(module, bindings.unwrap_or_default());
        let compiler = Compiler::new_with_bindings(self, module, source_id, kind, Some(bindings));
        let closure = compiler.compile(program)?;
        Ok(closure)
    }

    fn attach_prelude_bindings(&mut self, module: ObjRef, mut bindings: CompileBindings) -> CompileBindings {
        let mut prelude = self.prelude_bindings.iter().collect::<Vec<_>>();
        prelude.sort_by(|(left, _), (right, _)| self.resolve_symbol(**left).cmp(self.resolve_symbol(**right)));
        let start = self.heap.module(module).linked_reads.len();
        for (offset, (name, binding)) in prelude.into_iter().enumerate() {
            let Ok(index) = u32::try_from(start + offset) else { break };
            self.heap
                .module_mut(module)
                .linked_reads
                .push(RuntimeLinkedRead::Binding(*binding));
            bindings.add_prelude(self.resolve_symbol(*name).to_owned().into_boxed_str(), ImportBindingId(index));
        }
        bindings
    }

    /// Compiles a pre-parsed AST for `module` with `kind`.
    pub fn compile_ast_as(&mut self, module: ObjRef, source_id: u32, program: Program, kind: UnitKind) -> PhResult<ObjRef> {
        self.compile_ast_as_with_bindings(module, source_id, program, kind, None)
    }

    /// Parses and compiles one already-linked module with its closed namespace.
    pub fn compile_closure_as_with_bindings(&mut self, module: ObjRef, source: &str, kind: UnitKind, bindings: Option<CompileBindings>) -> PhResult<ObjRef> {
        let source_id = self.heap.module_mut(module).push_source(Arc::new(source.to_string()));
        let program = parse_source(source, 0).map_err(|e| PhError::Compile(CompilerError::Parse(e)))?;
        self.compile_ast_as_with_bindings(module, source_id, program, kind, bindings)
    }

    /// Installs runtime entries for symbolic `GetLinked` reads.
    pub fn install_linked_reads(&mut self, module: ObjRef, reads: Vec<RuntimeLinkedRead>) {
        self.heap.module_mut(module).linked_reads = reads;
    }

    /// Parses and compiles `source` for `module`, returning the top-level
    /// closure [`ObjRef`] allocated on the [`Heap`](crate::heap::Heap).
    pub fn compile_closure(&mut self, module: ObjRef, source: &str) -> PhResult<ObjRef> {
        self.compile_closure_as(module, source, UnitKind::File)
    }

    /// Runs a materialized compiled program: compiles only missing initializers and initializes the DAG.
    pub fn run_compiled(&mut self, program: &CompiledProgram) -> PhResult<()> {
        self.materialize_program(program)?;

        for (id, compiled_mod) in &program.modules {
            let record = self
                .module_registry
                .get(id)
                .ok_or_else(|| crate::error::RuntimeError::Internal(format!("materialized module {id} missing from registry")))?;
            if matches!(
                record.state,
                crate::modules::registry::ModuleState::Initialized | crate::modules::registry::ModuleState::Failed
            ) {
                continue;
            }
            if self.heap.module(record.object).closure.is_none() {
                if let Some(source_text) = &compiled_mod.source_text {
                    let _ = self.compile_program_module_closure(id, source_text, program)?;
                }
            }
        }

        self.initialize_program(program)
    }

    /// Runs a top-level `closure` within `module` on a fresh frame.
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

    /// Compiles and runs `source` for `module`, reporting diagnostics on
    /// failure.
    pub fn interpret_source(&mut self, module: ObjRef, source: &str) -> PhResult<()> {
        let closure = match self.compile_closure(module, source) {
            Ok(closure) => closure,
            Err(err) => {
                let source_id = self.heap.module(module).sources.len().saturating_sub(1) as u32;

                self.compiler_error(&err, module, source_id);
                return Err(err);
            }
        };

        if let Err(err) = self.run_in_module(module, closure) {
            self.report_runtime_error(&err);
            return Err(err);
        }

        Ok(())
    }
}
