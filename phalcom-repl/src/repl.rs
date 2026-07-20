use phalcom_ast::ast::Statement;
use phalcom_ast::parse_source;
use phalcom_core::compiler::lib::UnitKind;
use phalcom_core::heap::{ObjRef, Object};
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

/// Helper trait for error-guarded string rendering of REPL cell evaluation results.
pub trait ValueExt {
    /// Renders `self` as a string, falling back to class display name if `toString` panics or fails.
    fn to_string_guarded(&self, vm: &VM) -> String;
}

impl ValueExt for Value {
    fn to_string_guarded(&self, vm: &VM) -> String {
        match self {
            Value::Nil | Value::Bool(_) | Value::Number(_) | Value::Symbol(_) => self.to_string(vm),
            Value::Obj(id) => {
                let obj = vm.heap.get(*id);
                match obj {
                    Object::Str(_) => self.to_string(vm),
                    Object::Class(c) => c.name_copy(),
                    Object::Instance(inst) => {
                        let class_name = vm.heap.class(inst.class).name_copy();
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                            self.to_string(vm)
                        }))
                        .unwrap_or_else(|_| format!("<instance of {class_name}>"))
                    }
                    _ => self.to_string(vm),
                }
            }
        }
    }
}

/// Holds per-session state for one Phalcom REPL invocation.
pub struct ReplSession {
    pub vm: VM,
    pub module: ObjRef,
    pub cwd: PathBuf,
    pub next_cell: usize,
    /// Every cell's source, in submission order — `:reload`'s input (§S9).
    pub history: Vec<String>,
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
        let src_norm = if src.ends_with('\n') {
            src.to_string()
        } else {
            format!("{src}\n")
        };

        self.history.push(src_norm.clone());
        self.next_cell += 1;

        let program = match parse_source(&src_norm, 0) {
            Ok(p) => p,
            Err(_) => return CellOutcome::Failed,
        };

        let is_expr_cell = matches!(program.statements.last(), Some(Statement::Expr { .. }));

        let closure = match self.vm.compile_closure_as(self.module, &src_norm, UnitKind::Repl) {
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

    /// Discards session state and re-runs accumulated cell history in order.
    ///
    /// Builds a fresh `VM` and `Compiler` to prevent same-scope redeclaration
    /// and `class.already_defined` traps (§07 §4). Stops at the first failing cell.
    pub fn reload(&mut self) -> bool {
        let old_history = std::mem::take(&mut self.history);
        let abs_path = self.cwd.display().to_string();
        let mut new_vm = VM::new();
        let new_module = new_vm.create_module("main", &abs_path);

        self.vm = new_vm;
        self.module = new_module;
        self.next_cell = 1;
        self.history = Vec::new();

        for (idx, cell_src) in old_history.iter().enumerate() {
            if let CellOutcome::Failed = self.eval(cell_src) {
                eprintln!("Reload halted at cell {}: evaluation failed.", idx + 1);
                return false;
            }
        }
        true
    }
}
