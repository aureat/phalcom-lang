//! Source-to-execution driver: parse, compile and run a module on the [`VM`].
//!
//! This is the top-level entry the CLI and REPL call. It resolves module
//! paths, compiles source into a heap [`ObjRef`] closure via the compiler, and
//! runs it on the [`VM`]. Since [ADR-0009](../../docs/adr/accepted/0009-handle-arena-heap.md)
//! compiled closures, modules and frames are all [`ObjRef`] handles into the
//! VM's [`Heap`](crate::heap::Heap) rather than `Rc<RefCell<T>>` graphs.

use crate::compiler::lib::{Compiler, CompilerError, UnitKind};
use crate::error::{IoError, PhError, PhResult};
use crate::frame::{CallContext, CallFrame};
use crate::heap::ObjRef;
use crate::modules::{CompileBindings, RuntimeLinkedRead};
use crate::vm::VM;
use phalcom_ast::parse_source;
use std::fs;
use std::path::{Component, Path, PathBuf};
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

pub fn normalize_path(path: &str) -> String {
    Path::new(path)
        .components()
        .map(|component| match component {
            Component::Prefix(s) => s.as_os_str().to_str().unwrap_or("<invalid>"),
            Component::RootDir => "",
            Component::CurDir => ".",
            Component::ParentDir => "..",
            Component::Normal(s) => s.to_str().unwrap_or("<invalid>"),
        })
        .collect::<Vec<&str>>()
        .join("/")
}

pub struct ModuleInfo {
    pub path: String,
    pub name: String,
    pub file_name: String,
}

pub fn resolve_module_path(path: &str) -> PhResult<String> {
    let relative_path = PathBuf::from(path);
    let absolute_path = relative_path.canonicalize().map_err(PhError::from)?;

    if absolute_path.is_dir() {
        return Err(IoError::Message(format!("Should be a file. \"{}\" is a directory.", absolute_path.display())).into());
    }

    Ok(absolute_path.display().to_string())
}

pub enum InterpretResult {
    Success,
    CompileError(PhError),
    RuntimeError(PhError),
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

    pub fn run_file(&mut self, file_path: &str) -> PhResult<()> {
        let file_path = resolve_module_path(file_path)?;
        let abs: PathBuf = PathBuf::from(file_path);

        let src = fs::read_to_string(&abs).map_err(|e| IoError::Message(format!("Failed to read file {}: {}", abs.display(), e)))?;

        let main = self.vm.create_module("main", &abs.display().to_string());

        match self.vm.interpret_source(main, &src) {
            Ok(_) => exit(ExitCode::Success),
            Err(PhError::Runtime(_)) => exit(ExitCode::RuntimeError),
            Err(PhError::Compile(_)) => exit(ExitCode::CompileError),
            Err(_) => exit(ExitCode::GenericError),
        }
    }
}

impl VM {
    /// Parses and compiles `source` for `module` with `kind`, returning the top-level
    /// closure [`ObjRef`] allocated on the [`Heap`](crate::heap::Heap).
    pub fn compile_closure_as(&mut self, module: ObjRef, source: &str, kind: UnitKind) -> PhResult<ObjRef> {
        self.compile_closure_as_with_bindings(module, source, kind, None)
    }

    /// Parses and compiles one already-linked module with its closed namespace.
    ///
    /// The linker owns import discovery and target identity. This entry point
    /// only lowers source against the supplied immutable binding table; it
    /// never resolves or loads another module.
    pub fn compile_closure_as_with_bindings(&mut self, module: ObjRef, source: &str, kind: UnitKind, bindings: Option<CompileBindings>) -> PhResult<ObjRef> {
        self.unit_kind = kind;
        let source_id = self.heap.module_mut(module).push_source(Arc::new(source.to_string()));
        let program = parse_source(source, 0).map_err(|e| PhError::Compile(CompilerError::Parse(e)))?;

        let compiler = match bindings {
            Some(bindings) => Compiler::new_with_bindings(self, module, source_id, kind, Some(bindings)),
            None => Compiler::new(self, module, source_id, kind),
        };
        let closure = compiler.compile(program)?;
        Ok(closure)
    }

    /// Installs runtime entries for symbolic `GetLinked` reads.
    ///
    /// Part II exposes this narrow materialization seam; constructing the
    /// entries from a `LinkedProgram` belongs to the Part III runtime layer.
    pub fn install_linked_reads(&mut self, module: ObjRef, reads: Vec<RuntimeLinkedRead>) {
        self.heap.module_mut(module).linked_reads = reads;
    }

    /// Parses and compiles `source` for `module`, returning the top-level
    /// closure [`ObjRef`] allocated on the [`Heap`](crate::heap::Heap).
    ///
    /// `source` is appended to
    /// [`ModuleObject::sources`](crate::heap::ModuleObject::sources) and the
    /// resulting index stamped into every [`Chunk`](crate::chunk::Chunk)
    /// produced here, so each compiled unit — each REPL cell — keeps its own
    /// text for diagnostics (U-REPL §D2).
    ///
    /// # Errors
    ///
    /// Returns [`PhError::Compile`] if `source` fails to parse or compile; the
    /// parse diagnostic is printed before the error is returned.
    pub fn compile_closure(&mut self, module: ObjRef, source: &str) -> PhResult<ObjRef> {
        self.compile_closure_as(module, source, UnitKind::File)
    }

    /// Installs `module` and runs its top-level `closure` on a fresh frame.
    ///
    /// Both `module` and `closure` are [`ObjRef`] handles into the
    /// [`Heap`](crate::heap::Heap). The frame and value stacks are cleared
    /// before the run.
    ///
    /// # Errors
    ///
    /// Returns [`PhError::Runtime`] if execution raises an uncaught error.
    pub fn run_in_module(&mut self, module: ObjRef, closure: ObjRef) -> PhResult<()> {
        let module_sym = self.heap.module(module).symbol();
        self.modules.insert(module_sym, module);

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
    ///
    /// # Errors
    ///
    /// Returns [`PhError::Compile`] on a compile failure (after printing a
    /// compiler diagnostic) or [`PhError::Runtime`] on an uncaught runtime
    /// error (after printing a runtime diagnostic).
    pub fn interpret_source(&mut self, module: ObjRef, source: &str) -> PhResult<()> {
        // No source registration here: `compile_closure` records the text and
        // stamps its index into the chunks it builds, so source is registered
        // exactly where it is compiled (U-REPL §D2).
        let closure = self.compile_closure(module, source).inspect_err(|err| {
            let source_id = self.heap.module(module).sources.len().saturating_sub(1) as u32;
            self.compiler_error(err.clone(), module, source_id);
        })?;

        self.run_in_module(module, closure).inspect_err(|err| {
            let _ = self.runtime_error(err.clone());
        })?;

        Ok(())
    }
}
