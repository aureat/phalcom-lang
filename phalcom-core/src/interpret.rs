//! Source-to-execution driver: parse, compile and run a module on the [`VM`].
//!
//! This is the top-level entry the CLI and REPL call. It resolves module
//! paths, compiles source into a heap [`ObjRef`] closure via the compiler, and
//! runs it on the [`VM`]. Since [ADR-0009](../../docs/adr/accepted/0009-handle-arena-heap.md)
//! compiled closures, modules and frames are all [`ObjRef`] handles into the
//! VM's [`Heap`](crate::heap::Heap) rather than `Rc<RefCell<T>>` graphs.

use crate::compiler::lib::{Compiler, CompilerError};
use crate::diagnostics::print_parse;
use crate::error::{IoError, PhError, PhResult};
use crate::frame::{CallContext, CallFrame};
use crate::heap::{Object, ObjRef};
use crate::heap::ModuleObject;
use crate::vm::VM;
use phalcom_ast::parse_source;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use std::{fs, io};

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

fn append_ph_extension_if_missing(p: &Path) -> PathBuf {
    if p.extension().and_then(|s| s.to_str()) == Some("ph") {
        p.to_path_buf()
    } else {
        let mut p = p.to_path_buf();
        p.set_extension("ph");
        p
    }
}

pub fn resolve_import_path(importer_abs_path: &str, import_logical: &str) -> io::Result<String> {
    let importer_path_guess = PathBuf::from(importer_abs_path);
    let importer_dir = importer_path_guess.parent().unwrap_or(Path::new("."));

    let fs_path = append_ph_extension_if_missing(&importer_dir.join(import_logical));
    let canonical = fs_path.canonicalize()?;

    Ok(canonical.display().to_string())
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
    /// Parses and compiles `source` for `module`, returning the top-level
    /// closure [`ObjRef`] allocated on the [`Heap`](crate::heap::Heap).
    ///
    /// # Errors
    ///
    /// Returns [`PhError::Compile`] if `source` fails to parse or compile; the
    /// parse diagnostic is printed before the error is returned.
    pub fn compile_closure(&mut self, module: ObjRef, source: &str) -> PhResult<ObjRef> {
        let program = parse_source(source, 0).map_err(|e| {
            let msg = e.kind.to_string();
            print_parse(source, &msg, e.range.clone());
            PhError::Compile(CompilerError::Parse(e))
        })?;

        let compiler = Compiler::new(self, module);
        let closure = compiler.compile(program)?;
        Ok(closure)
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
        self.frames.push(frame);
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
        let source_arc = std::sync::Arc::new(source.to_string());
        self.heap.module_mut(module).source = Some(source_arc);

        let closure = self.compile_closure(module, source).inspect_err(|err| {
            self.compiler_error(err.clone());
        })?;

        self.run_in_module(module, closure).inspect_err(|err| {
            let _ = self.runtime_error(err.clone());
        })?;

        Ok(())
    }

    /// Resolves, loads, and memoizes the module named by `import_logical`
    /// relative to `importer_abs_path` (U15, DEC-U15 A+A: relative file-path
    /// resolution + whole-module binding).
    ///
    /// [`resolve_import_path`] canonicalizes the path (appending `.ph` if
    /// missing) against the importer's own directory; the canonical path is
    /// then probed against [`Universe::module_registry`](crate::universe::Universe::module_registry)
    /// **before** any compilation happens — a path already loaded (or
    /// mid-load, on a cyclic import) returns the identical [`ObjRef`] rather
    /// than recompiling, so a class defined in the imported unit keeps one
    /// identity across every importer (`isA` stays sound, U15 plan §1/§4).
    ///
    /// Unlike [`Self::run_in_module`] (the *outermost* program entry, which
    /// clears the frame/value stacks), this runs the imported unit's top
    /// level **re-entrantly**: a nested import happens mid-execution of the
    /// importer's own frame, so clearing would destroy live state. It
    /// follows the same re-entrant `run_until` pattern as
    /// [`Self::send_dynamic`](crate::vm::VM::send_dynamic) — push one fresh
    /// frame, drain exactly that activation, recover the frame stack to
    /// where it was.
    ///
    /// # Errors
    ///
    /// Returns [`PhError::Io`] if `import_logical` does not resolve to an
    /// existing, readable `.ph` file relative to `importer_abs_path`
    /// (canonicalization fails on a missing file — this is the "missing
    /// file raises a clean error with the attempted path" guarantee, U15
    /// plan §6); propagates any [`PhError::Compile`] or [`PhError::Runtime`]
    /// raised compiling or running the imported unit's top level (including
    /// the documented cyclic-import partial-init hazard, U15 plan §4).
    pub fn import_module(&mut self, importer_abs_path: &str, import_logical: &str) -> PhResult<ObjRef> {
        let canonical = resolve_import_path(importer_abs_path, import_logical)
            .map_err(|e| PhError::Io(IoError::Message(format!("Cannot import \"{import_logical}\": {e}"))))?;

        if let Some(&existing) = self.universe.module_registry.get(&canonical) {
            return Ok(existing);
        }

        let logical_name = Path::new(&canonical)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(import_logical)
            .to_string();

        // Allocate + register the Module *before* compiling/running it — the
        // memo the cyclic-import guard above relies on (see
        // `Universe::module_registry`'s doc for the partial-init hazard this
        // ordering accepts).
        let module_sym = self.interner.intern(&logical_name);
        let module_obj = ModuleObject::new(logical_name.clone(), module_sym, canonical.clone(), None);
        let module_id = self.heap.alloc(Object::Module(Box::new(module_obj)));
        self.universe.module_registry.insert(canonical.clone(), module_id);

        let source = fs::read_to_string(&canonical).map_err(|e| IoError::Message(format!("Failed to read file {canonical}: {e}")))?;
        self.heap.module_mut(module_id).source = Some(Arc::new(source.clone()));
        self.register_source(&logical_name, &source);

        let closure = self.compile_closure(module_id, &source)?;
        self.heap.module_mut(module_id).add_closure(closure);

        // Re-entrant run (mirrors `send_dynamic`): push a fresh frame for the
        // imported unit's top level and drain just that one activation —
        // never `run_in_module`, whose stack-clearing assumes it is the
        // *outermost* entry.
        let base_frames = self.frames.len();
        let stack_offset = self.stack.len();
        let frame = self.new_call_frame(closure, CallContext::Module { module: module_id }, 0, stack_offset, None);
        self.frames.push(frame);
        self.native_reentry_depth += 1;
        let result = self.run_until(base_frames);
        self.native_reentry_depth -= 1;
        result?;

        Ok(module_id)
    }
}
