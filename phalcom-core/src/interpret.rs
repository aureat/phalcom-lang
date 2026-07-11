use crate::closure::ClosureObject;
use crate::compiler::lib::{Compiler, CompilerError};
use crate::diagnostics::print_parse;
use crate::error::{IoError, PhError, PhResult};
use crate::frame::{CallContext, CallFrame};
use crate::module::ModuleObject;
use crate::vm::VM;
use phalcom_ast::parse_source;
use phalcom_common::{phref_new, PhRef};
use std::path::{Component, Path, PathBuf};
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

    let fs_path = append_ph_extension_if_missing(&importer_dir.join(&import_logical));
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
    let absolute_path = relative_path.canonicalize().map_err(|err| PhError::from(err))?;

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

impl Interpreter {
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
    pub fn compile_closure(&mut self, module: PhRef<ModuleObject>, source: &str) -> PhResult<PhRef<ClosureObject>> {
        let program = parse_source(&source, 0).map_err(|e| {
            let msg = e.kind.to_string();
            print_parse(&source, &msg, e.range.clone());
            PhError::Compile(CompilerError::Parse(e))
        })?;

        let compiler = Compiler::new(self, module.clone());
        let closure = compiler.compile(program)?;
        Ok(closure)
    }

    pub fn run_in_module(&mut self, module: PhRef<ModuleObject>, closure: PhRef<ClosureObject>) -> PhResult<()> {
        let module_sym = module.borrow().symbol();
        self.modules.insert(module_sym, module.clone());

        self.frames.clear();
        self.stack.clear();

        let frame = phref_new(CallFrame::new(closure, CallContext::Module { module }, 0, 0, None));
        self.frames.push(frame);
        self.run()?;
        Ok(())
    }

    pub fn interpret_source(&mut self, module: PhRef<ModuleObject>, source: &str) -> PhResult<()> {
        let closure = self.compile_closure(module.clone(), source).map_err(|err| {
            self.compiler_error(err.clone());
            err
        })?;

        let _ = self.run_in_module(module, closure).map_err(|err| {
            let _ = self.runtime_error(err.clone());
            err
        })?;

        Ok(())
    }
}
