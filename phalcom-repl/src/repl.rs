use phalcom_ast::ast::Statement;
use phalcom_ast::parse_source;
use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::heap::ObjRef;
use phalcom_core::value::Value;
use phalcom_core::vm::VM;
use std::path::PathBuf;

/// Outcome of evaluating a single REPL cell.
#[derive(Debug)]
pub enum CellOutcome {
    /// An expression cell; render `// => {0}` (§S4 value echo).
    Value(Value),
    /// A statement cell; render nothing.
    Unit,
    /// Compile or runtime failure; diagnostic has already been printed.
    Failed,
}

/// Holds per-session state for one Phalcom REPL invocation.
pub struct ReplSession {
    pub(crate) vm: VM,
    pub(crate) module: ObjRef,
    pub(crate) cwd: PathBuf,
    pub(crate) next_cell: usize,
    /// Every cell's source, in submission order — `:reload`'s input (§S9).
    pub(crate) history: Vec<String>,
}

impl ReplSession {
    /// Starts a new REPL session rooted at the given working directory.
    pub fn start(cwd: PathBuf) -> ReplSession {
        let mut vm = VM::new();
        let abs_path = cwd.display().to_string();
        let module = vm.create_module("main", &abs_path);
        ReplSession {
            vm,
            module,
            cwd,
            next_cell: 1,
            history: Vec::new(),
        }
    }

    /// Evaluates one input cell.
    pub fn eval(&mut self, src: &str) -> CellOutcome {
        self.history.push(src.to_string());
        self.next_cell += 1;

        let program = match parse_source(src, 0) {
            Ok(p) => p,
            Err(_) => return CellOutcome::Failed,
        };

        let is_expr_cell = matches!(program.statements.last(), Some(Statement::Expr { .. }));

        let closure = match self.vm.compile_closure_as(self.module, src, UnitKind::Repl) {
            Ok(c) => c,
            Err(err) => {
                self.vm.compiler_error(err);
                return CellOutcome::Failed;
            }
        };

        match self.vm.run_cell(self.module, closure) {
            Ok(val) => {
                if is_expr_cell {
                    let module_sym = self.vm.heap.module(self.module).symbol();
                    let underscore_sym = self.vm.get_or_intern("_");
                    let _ = self.vm.define_global(module_sym, underscore_sym, val);
                    CellOutcome::Value(val)
                } else {
                    CellOutcome::Unit
                }
            }
            Err(err) => {
                let _ = self.vm.runtime_error(err);
                CellOutcome::Failed
            }
        }
    }
}

