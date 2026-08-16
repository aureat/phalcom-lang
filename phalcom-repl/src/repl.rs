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
    /// Renders `self` for value echo by **sending `toString`**, degrading to the
    /// receiver's class name if that send raises (U-REPL §S4,
    /// [PDR-0008](../../../docs/decisions/0008-cell-boundary-diagnostics-and-state-hygiene.md) §4).
    ///
    /// This must send: `Value::to_string` is the *native* renderer and never
    /// dispatches for a plain instance, so a user `toString` override would be
    /// invisible. [`Value::to_display_string`] is the only path that performs the
    /// send.
    ///
    /// A raising `toString` degrades here and never fails the cell — the caller
    /// has already decided the outcome by the time echo runs.
    fn to_string_guarded(&self, vm: &mut VM) -> String;
}

impl ValueExt for Value {
    fn to_string_guarded(&self, vm: &mut VM) -> String {
        match self.to_display_string(vm) {
            Ok(rendered) => rendered,
            Err(_) => {
                // The failed send left frames and operands behind; drop them so the
                // next cell does not run on a dirty stack (PDR-0008 §4).
                vm.unwind_cell();
                degraded_render(*self, vm)
            }
        }
    }
}

/// Renders `value` without dispatching, for use when `toString` itself raised.
fn degraded_render(value: Value, vm: &VM) -> String {
    match value {
        Value::Obj(id) => match vm.heap.get(id) {
            Object::Instance(inst) => format!("<instance of {}>", vm.heap.class(inst.class).name_copy()),
            Object::Class(c) => c.name_copy(),
            _ => value.to_string(vm),
        },
        _ => value.to_string(vm),
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
        let src_norm = if src.ends_with('\n') { src.to_string() } else { format!("{src}\n") };

        self.history.push(src_norm.clone());
        self.next_cell += 1;

        // This pre-parse exists only to classify the cell as expression-or-statement;
        // `compile_closure_as` parses again and prints its own diagnostic. On the error
        // path we never reach it, so the diagnostic is printed here instead — without
        // this, a syntax error produces no output at all and `CellOutcome::Failed`'s
        // contract is a lie (PDR-0008 §1).
        let program = match parse_source(&src_norm, 0) {
            Ok(p) => p,
            Err(e) => {
                phalcom_core::diagnostics::print_parse(&src_norm, None, &e.kind.to_string(), e.range.clone());
                return CellOutcome::Failed;
            }
        };

        let is_expr_cell = matches!(program.statements.last(), Some(Statement::Expr { .. }));

        let closure = match self.vm.compile_closure_as(self.module, &src_norm, UnitKind::Repl) {
            Ok(c) => c,
            Err(err) => {
                let source_id = (self.vm.heap.module(self.module).sources.len().saturating_sub(1)) as u32;
                self.vm.compiler_error(err, self.module, source_id);
                return CellOutcome::Failed;
            }
        };

        match self.vm.run_cell(self.module, closure) {
            Ok(val) => {
                if is_expr_cell {
                    let underscore_sym = self.vm.get_or_intern("_");
                    let _ = self.vm.define_global(self.module, underscore_sym, val);
                    CellOutcome::Value(val)
                } else {
                    CellOutcome::Unit
                }
            }
            // `run_cell` reports the runtime error itself, while the frames that make
            // up the traceback still exist (PDR-0008 §2). Reporting again here would
            // print it twice.
            Err(_) => CellOutcome::Failed,
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
