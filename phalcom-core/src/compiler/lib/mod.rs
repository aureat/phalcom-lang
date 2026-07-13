//! The AST-to-bytecode compiler.
//!
//! Lowers a parsed [`Program`] into a [`ClosureObject`] whose [`Chunk`] the VM
//! executes. String, method and superclass constants are materialized onto the
//! [`Heap`](crate::heap::Heap) as the compiler emits them — the compiler already
//! holds `&mut VM` and therefore the heap — and are referenced from the constant
//! pool by `Copy` [`Value::Obj`] handles
//! ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md),
//! [ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)).

mod class_decl;
mod error;
mod expr;
mod jumps;
mod loops;
mod patterns;
mod scope;
mod state;

pub use error::CompilerError;

use crate::bytecode::Bytecode;
use crate::callable::Callable;
use crate::closure::ClosureObject;
use crate::error::PhResult;
use crate::heap::{ObjRef, Object};
use crate::interner::Symbol;
use crate::value::Value;
use crate::vm::VM;
use phalcom_ast::ast::{BindingKind, Expr, MethodCallExpr, Pattern, Program, Statement};
use phalcom_common::range::EmptySourceRange;
use state::FunctionState;
use state::LoopContext;
use std::collections::HashSet;

pub(crate) struct Compiler<'vm> {
    pub(crate) vm: &'vm mut VM,
    module: ObjRef,
    /// The stack of function-compilation states, innermost body last. A block
    /// literal pushes a new state, compiles into it, and pops it; upvalue
    /// resolution indexes into this stack rather than following raw pointers.
    pub(crate) functions: Vec<FunctionState>,
    /// Names bound by an immutable module-level `let`.
    ///
    /// Module-level bindings become globals rather than stack locals
    /// ([`Bytecode::DefineGlobal`]), so [`Self::functions`] never sees them and
    /// cannot record their mutability. This set lets the assignment path reject
    /// a store to a `let` global at compile time
    /// ([ADR-0014](../../../docs/adr/0014-let-var-bindings.md)).
    immutable_globals: HashSet<Symbol>,
    /// The class name Symbol currently being compiled, if any (ADR-0011).
    current_class: Option<Symbol>,
    /// Whether the current method/scope context is static (metaclass-side) (ADR-0017).
    is_static_context: bool,
    /// The stack of enclosing `for`-loop contexts, innermost last (ADR-0035 §3,
    /// U-ITER specification §4). A `for` body pushes a [`LoopContext`] while it
    /// compiles; `break`/`continue` resolve to the top frame, and a
    /// `break`/`continue` with this stack empty is a compile error (C-ITER-7).
    loop_contexts: Vec<LoopContext>,
}

impl<'vm> Compiler<'vm> {
    pub(crate) fn new(vm: &'vm mut VM, module: ObjRef) -> Self {
        Compiler {
            vm,
            module,
            functions: vec![FunctionState::new(false, false)],
            immutable_globals: HashSet::new(),
            current_class: None,
            is_static_context: false,
            loop_contexts: Vec::new(),
        }
    }

    /// Compiles a block or method body into a heap-allocated closure.
    ///
    /// Slot 0 holds the receiver (`self` for a method, the block object
    /// otherwise). A body whose last statement leaves a value on the operand
    /// stack ([`Statement::Expr`], [`Statement::Let`], or [`Statement::Return`])
    /// returns that value. A **value-less** body — an empty body, or one ending
    /// in a [`Statement::Class`] or any other statement that leaves nothing
    /// behind — yields `None`: a [`Bytecode::Nil`] placeholder is pushed before
    /// the fallback [`Bytecode::Return`] so that falling off the end surfaces
    /// absence (U6-plan.md §4) rather than the receiver sitting in slot 0. This
    /// mirrors [`compile_inline_block_body`] so the inlined fast path and the
    /// non-inlined fallback agree.
    ///
    /// [`compile_inline_block_body`]: crate::compiler::inliner
    ///
    /// # Errors
    ///
    /// Propagates any error compiling the body's statements.
    fn compile_block(
        &mut self,
        statements: Vec<Statement>,
        name_sym: Symbol,
        params: Vec<String>,
        is_method: bool,
        is_constructor: bool,
    ) -> Result<ObjRef, CompilerError> {
        // Intern parameter and receiver names before pushing the function state.
        let mut param_symbols = Vec::with_capacity(params.len());
        for param_name in &params {
            param_symbols.push(self.vm.interner.intern(param_name));
        }
        let self_sym = self.vm.interner.intern("self");
        let dummy_sym = self.vm.interner.intern("<block-receiver>");

        // Push a fresh function-compilation state for this body.
        self.functions.push(FunctionState::new(is_constructor, !is_method));
        self.begin_scope();

        if is_method {
            // Slot 0 holds the receiver `self`.
            self.add_local(self_sym, true);
        } else {
            // Slot 0 holds the block object itself (blocks reach `self` via an
            // upvalue, functions.md §2), so we reserve it with a dummy local.
            self.add_local(dummy_sym, true);
        }

        if is_constructor {
            self.emit(Bytecode::GetSelf, EmptySourceRange);
            self.emit(Bytecode::NewInstance, EmptySourceRange);
            self.emit(Bytecode::SetLocal(0), EmptySourceRange);
            self.emit(Bytecode::Pop, EmptySourceRange);
        }

        for param_sym in param_symbols {
            self.add_local(param_sym, true);
        }

        let len = statements.len();
        let mut last_is_return = false;
        // Track whether the last statement leaves a value on the operand stack.
        // `Expr`, `Let`, and `Return` do; an empty body or a trailing `Class`
        // (or any other value-less statement) does not. This mirrors
        // `compile_inline_block_body` so the fast (inlined) and fallback paths
        // agree on the fall-off-end result.
        let mut leaves_value = false;
        for (i, statement) in statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            if is_last {
                if let Statement::Return(_) = statement {
                    last_is_return = true;
                }
                leaves_value = matches!(statement, Statement::Expr { .. } | Statement::Let(_) | Statement::Return(_));
            }
            self.compile_statement_with_pop_control(statement, !is_last)?;
        }

        let max_slots = self.functions.last().unwrap().max_slots;
        self.end_scope(EmptySourceRange);

        if !last_is_return {
            if self.functions.last().unwrap().is_constructor {
                self.emit(Bytecode::GetLocal(0), EmptySourceRange);
            } else if !leaves_value {
                self.emit(Bytecode::Nil, EmptySourceRange);
            }
            self.emit(Bytecode::Return, EmptySourceRange);
        }

        let func = self.functions.pop().unwrap();
        let callable = Callable {
            chunk: func.chunk,
            max_slots,
            num_upvalues: func.upvalues.len(),
            upvalues: func.upvalues,
            arity: params.len(),
            name_sym,
        };

        let closure = self.vm.heap.alloc(Object::Closure(ClosureObject {
            callable,
            module: self.module,
            upvalues: Vec::new(),
        }));
        Ok(closure)
    }

    pub(crate) fn compile(mut self, program: Program) -> PhResult<ObjRef> {
        let len = program.statements.len();
        let mut last_is_return = false;
        for (i, statement) in program.statements.into_iter().enumerate() {
            let is_last = i == len - 1;
            // Check if last statement is a return
            if is_last {
                if let Statement::Return(_) = statement {
                    last_is_return = true;
                }
            }
            self.compile_statement_with_pop_control(statement, !is_last)?;
        }

        if !last_is_return {
            self.emit(Bytecode::Return, EmptySourceRange);
        }

        let name_sym = self.vm.heap.module(self.module).name_sym;
        let func = self.functions.pop().unwrap();
        let callable = Callable {
            chunk: func.chunk,
            max_slots: func.max_slots,
            num_upvalues: 0,
            upvalues: Vec::new(),
            arity: 0,
            name_sym,
        };

        let closure = self.vm.heap.alloc(Object::Closure(ClosureObject {
            callable,
            module: self.module,
            upvalues: Vec::new(),
        }));

        Ok(closure)
    }

    pub(crate) fn compile_statement_with_pop_control(&mut self, statement: Statement, emit_pop: bool) -> Result<(), CompilerError> {
        match statement {
            Statement::Expr { expr, range } => {
                // A bare-statement expression whose value is about to be
                // popped is an "unused" position (U-CORE-2): pass that
                // through so a one-armed sacred conditional can skip its
                // `Some`-wrap allocation — see `compile_expr_want`.
                self.compile_expr_want(expr, !emit_pop)?;
                if emit_pop {
                    // println!("[Compiler] Emitting Pop");
                    self.emit(Bytecode::Pop, range);
                }
            }
            Statement::Let(binding) => {
                let range = binding.range;
                // `var` is mutable and may be left uninitialized; `let` is
                // immutable and *requires* an initializer (ADR-0014). A
                // destructuring pattern (`Pattern::Tuple`/`Pattern::List`)
                // always requires an initializer regardless of `let`/`var`
                // (U14, open-questions.md Q7, ADR-0046 §2) — there is nothing
                // to unpack from an absent value.
                let mutable = matches!(binding.kind, BindingKind::Var);

                match binding.value {
                    Some(expr) => self.compile_expr(expr)?,
                    None => {
                        let Pattern::Name { name, .. } = &binding.pattern else {
                            return Err(CompilerError::DestructuringWithoutInitializer(binding.pattern.range()));
                        };
                        if !mutable {
                            // `let x` with no initializer is a compile error.
                            return Err(CompilerError::LetWithoutInitializer(name.clone()));
                        }
                        // `var x` with no initializer is backed by the private
                        // `nil` sentinel; the VM surfaces every read of that
                        // slot as the surface `None` value (ADR-0007/ADR-0010).
                        self.emit(Bytecode::Nil, range);
                    }
                }

                // The initializer's single evaluated value is sitting on top
                // of the operand stack; bind it — positionally, through the
                // sub-patterns' `at(_)` reads, for a destructuring pattern —
                // as a local or a module global depending on where this `let`
                // appears (ADR-0014's existing local-vs-global rule, threaded
                // through every leaf of the pattern).
                let as_global = self.functions.last().unwrap().scope_depth == 0;
                self.compile_pattern_bind_top_of_stack(&binding.pattern, mutable, as_global)?;
            }
            Statement::Import(import_stmt) => {
                self.compile_import(import_stmt)?;
            }
            Statement::Return(return_stmt) => {
                let range = return_stmt.range;
                if self.functions.last().unwrap().is_constructor {
                    if return_stmt.value.is_some() {
                        return Err(CompilerError::ReturnValueFromInitializer);
                    }
                    self.emit(Bytecode::GetLocal(0), range);
                } else {
                    if let Some(expr) = return_stmt.value {
                        self.compile_expr(expr)?;
                    } else {
                        self.emit(Bytecode::Nil, range);
                    }
                }
                // A `return` inside a block literal is a *non-local* return: it
                // unwinds to the block's enclosing method activation, not just
                // the block's own frame (blocks.md §5, ADR-0013). A method or
                // constructor body keeps the ordinary single-frame `Return`.
                // Constructors are never block bodies (`is_constructor` always
                // accompanies `is_method`), so `is_block` is the sole gate.
                if self.functions.last().unwrap().is_block {
                    self.emit(Bytecode::ReturnNonLocal, range);
                } else {
                    self.emit(Bytecode::Return, range);
                }
            }
            Statement::Class(class_def) => {
                self.compile_class(class_def)?;
            }
            Statement::For(for_stmt) => {
                // A `for` is a statement consumed for effect (U-ITER spec
                // §1.2): it leaves no value, so `emit_pop` is irrelevant.
                self.compile_for(for_stmt)?;
            }
            Statement::Break { range } => {
                self.compile_break(range)?;
            }
            Statement::Continue { range } => {
                self.compile_continue(range)?;
            }
            Statement::Throw { expr, range } => {
                // The compile-time half of error-handling.md §1: a
                // syntactically-literal non-`Error` operand is rejected before
                // lowering (the runtime half — a genuine `doesNotUnderstand`
                // miss on `raise()` — already exists via U-CORE-6's `Error`
                // primitive, and covers everything this static check can't
                // prove, e.g. `throw someVariable`).
                if is_non_error_literal(&expr) {
                    return Err(CompilerError::ThrowNonError(range));
                }
                // `throw expr` is surface sugar for `expr.raise()`
                // (ADR-0031 §1) — reuse the ordinary `MethodCall` lowering
                // rather than hand-rolling an `Invoke`, so it gets the same
                // selector encoding / IC treatment as a written `.raise()`
                // send. `raise()` always unwinds (never returns normally), so
                // its "value" never materialises; `emit_pop` is honored purely
                // to keep the stack discipline uniform with every other
                // statement, not because a value is meaningfully produced.
                let call = Expr::MethodCall(Box::new(MethodCallExpr {
                    object: expr,
                    method: "raise".to_string(),
                    args: Vec::new(),
                    range,
                }));
                self.compile_expr_want(call, !emit_pop)?;
                if emit_pop {
                    self.emit(Bytecode::Pop, range);
                }
            }
        }
        Ok(())
    }

    /// Lowers `import "path" as Name` (U15, DEC-U15 A+A) to a
    /// [`Bytecode::Import`] carrying the raw path, immediately followed by
    /// the same [`Bytecode::DefineGlobal`] a module-level `let Name = …`
    /// would emit — the `as Name` binding is an ordinary immutable global
    /// ([ADR-0014](../../../docs/adr/0014-let-var-bindings.md)), not a new
    /// binding kind.
    ///
    /// Restricted to a compilation unit's own top level (`self.functions`
    /// has exactly the module body's [`FunctionState`] and its scope depth
    /// is 0) — mirroring how `class` is a program-shape construct rather
    /// than an ordinary statement. `import` inside a method/block/class body
    /// is a [`CompilerError::ImportNotAtTopLevel`] compile error.
    ///
    /// # Errors
    ///
    /// Returns [`CompilerError::ImportNotAtTopLevel`] if `import_stmt`
    /// appears anywhere but the module's own top level.
    fn compile_import(&mut self, import_stmt: phalcom_ast::ast::ImportStatement) -> Result<(), CompilerError> {
        if self.functions.len() > 1 || self.functions.last().unwrap().scope_depth > 0 {
            return Err(CompilerError::ImportNotAtTopLevel);
        }

        let range = import_stmt.range;
        let path_sym = self.vm.interner.intern(&import_stmt.path);
        let path_idx = self.add_constant(Value::Symbol(path_sym));
        self.emit(Bytecode::Import(path_idx), range);

        // `as Name` is always an immutable module-level global — the whole
        // binding is `let`-shaped (ADR-0014), never a local (import is
        // top-level-only, see the guard above).
        let name_sym = self.vm.interner.intern(&import_stmt.binding);
        self.immutable_globals.insert(name_sym);
        let name_idx = self.add_constant(Value::Symbol(name_sym));
        self.emit(Bytecode::DefineGlobal(name_idx), range);
        Ok(())
    }
}

/// Reports whether `expr` is a syntactically detectable non-`Error` literal —
/// a `Number`/`String`/`Boolean` literal, none of which can ever answer
/// `raise()` (only `Error` and its subclasses do). Guards `throw`'s
/// compile-time half of error-handling.md §1 ("`throw "oops"` is a compile
/// error"); a non-literal operand (a variable, a list/map-literal send, a
/// user `Error` construction) cannot be statically classified and defers to
/// the runtime `doesNotUnderstand` miss on `raise()` instead — deliberately no
/// flow typing (U-ERR plan §2.2).
fn is_non_error_literal(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Number { .. } | Expr::String { .. } | Expr::Boolean { .. } | Expr::Symbol(_)
    )
}

#[cfg(test)]
mod tests {

    // #[test]
    // fn test_primitive_class() {
    //     // Number
    //     let result = run_test("return 123.class;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Number"),
    //         _ => panic!("Expected Value::Class for 123.class"),
    //     }
    //     // String
    //     let result = run_test("return \"abc\".class;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "String"),
    //         _ => panic!("Expected Value::Class for string.class"),
    //     }
    //     // Boolean
    //     let result = run_test("return true.class;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Bool"),
    //         _ => panic!("Expected Value::Class for true.class"),
    //     }
    //     // Nil
    //     let result = run_test("return nil.class;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Nil"),
    //         _ => panic!("Expected Value::Class for nil.class"),
    //     }
    // }

    // #[test]
    // fn test_primitive_superclass() {
    //     // Number superclass
    //     let result = run_test("return 123.class.superclass;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
    //         _ => panic!("Expected Value::Class for 123.class.superclass"),
    //     }
    //     // String superclass
    //     let result = run_test("return \"abc\".class.superclass;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
    //         _ => panic!("Expected Value::Class for \"abc\".class.superclass"),
    //     }
    // }

    // #[test]
    // fn test_primitive_name() {
    //     // Number name
    //     let result = run_test("return 123.name;").unwrap();
    //     println!("{:?}", result);

    //     match result {
    //         Value::String(ref s) => assert_eq!(&*s.borrow().as_str(), "Number"),
    //         _ => panic!("Expected Value::String for 123.name"),
    //     }
    //     // String name
    //     let result = run_test("\"abc\".name;").unwrap();
    //     match result {
    //         Value::String(ref s) => assert_eq!(&*s.borrow().as_str(), "String"),
    //         _ => panic!("Expected Value::String for string.name"),
    //     }
    // }

    // #[test]
    // fn test_class_identity() {
    //     // .class.class should be Class
    //     let result = run_test("return 123.class.class;").unwrap();
    //     println!("{:?}", result);

    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Class"),
    //         _ => panic!("Expected Value::Class for 123.class.class"),
    //     }
    //     // .class.superclass should be Object
    //     let result = run_test("return 123.class.superclass;").unwrap();
    //     match result {
    //         Value::Class(c) => assert_eq!(&*c.borrow().name().borrow().as_str(), "Object"),
    //         _ => panic!("Expected Value::Class for 123.class.superclass"),
    //     }
    // }
    // use super::*;

    // fn run_test(source: &str) -> Result<Value, PhError> {
    //     let mut vm = VM::new();
    //     let closure = compile(&mut vm, source)?;
    //     let module = vm.create_module_from_stdin();
    //     vm.run_module(module, closure)
    // }

    // #[test]
    // fn test_compile_number() {
    //     let result = run_test("123;").unwrap();
    //     assert_eq!(result, Value::Number(123.0));
    // }

    // #[test]
    // fn test_compile_string() {
    //     let result = run_test("\"hello\";").unwrap();
    //     assert_eq!(result, Value::string_from("hello".to_string()));
    // }

    // #[test]
    // fn test_compile_boolean() {
    //     let result = run_test("return true;").unwrap();
    //     assert_eq!(result, Value::Bool(true));
    //     let result = run_test("return false;").unwrap();
    //     assert_eq!(result, Value::Bool(false));
    // }

    // #[test]
    // fn test_compile_nil() {
    //     let result = run_test("return nil;").unwrap();
    //     assert_eq!(result, Value::Nil);
    // }

    // #[test]
    // fn test_compile_binary_expr() {
    //     let result = run_test("return 1 + 2;").unwrap();
    //     assert_eq!(result, Value::Number(3.0));
    // }

    // #[test]
    // fn test_compile_binary_mult() {
    //     let result = run_test("return 4 * 3;").unwrap();
    //     assert_eq!(result, Value::Number(12.0));
    // }

    // #[test]
    // fn test_compile_unary_expr() {
    //     let result = run_test("-10;").unwrap();
    //     assert_eq!(result, Value::Number(-10.0));
    // }

    // #[test]
    // fn test_compile_global_let() {
    //     let result = run_test("let a = 10; return a;").unwrap();
    //     assert_eq!(result, Value::Number(10.0));
    // }

    // #[test]
    // fn test_compile_global_assignment() {
    //     let result = run_test("let a = 10; a += 20; return a;").unwrap();
    //     assert_eq!(result, Value::Number(30.0));
    // }

    // #[test]
    // fn test_complex_global_assignment() {
    //     let source = "
    //         let a = 5;
    //         let b = 10;
    //         a += b; // a should be 15
    //         return a;           // return 15
    //     ";
    //     let result = run_test(source).unwrap();
    //     assert_eq!(result, Value::Number(15.0));
    // }

    // #[test]
    // fn test_compile_precedence() {
    //     let result = run_test("return 1 + 2 * 3;").unwrap();
    //     assert_eq!(result, Value::Number(7.0));
    // }

    // #[test]
    // fn test_compile_return() {
    //     let result = run_test("return 15; 20;").unwrap();
    //     assert_eq!(result, Value::Number(15.0));

    //     let result = run_test("return;").unwrap();
    //     assert_eq!(result, Value::Nil);
    // }

    // #[test]
    // fn test_compile_method_expr() {
    //     let result = run_test("return 123.class.name.class;").unwrap();
    //     println!("{:?}", result);
    // }

    // #[test]
    // fn test_compile_class_add_call() {
    //     let result = run_test("return 123.class + true.class;").unwrap();
    //     println!("{:?}", result);
    // }
}
