//! The AST-to-bytecode compiler.
//!
//! Lowers a parsed [`Program`] into a [`ClosureObject`] whose [`Chunk`] the VM
//! executes. String, method and superclass constants are materialized onto the
//! [`Heap`](crate::heap::Heap) as the compiler emits them — the compiler already
//! holds `&mut VM` and therefore the heap — and are referenced from the constant
//! pool by `Copy` [`Value::Obj`] handles
//! ([ADR-0009](../../../docs/adr/0009-handle-arena-heap.md),
//! [ADR-0010](../../../docs/adr/0010-tagged-value-enum.md)).

use crate::bytecode::Bytecode;
use crate::callable::{Callable, UpvalueDescriptor};
use crate::chunk::Chunk;
use crate::compiler::inliner;
use crate::closure::ClosureObject;
use crate::error::PhResult;
use crate::heap::{ObjRef, Object};
use crate::interner::Symbol;
use crate::method::{encode_selector, make_signature, MethodKind, MethodObject, SignatureKind};
use crate::value::Value;
use crate::vm::VM;
use phalcom_ast::ast::{Argument, BinaryOp, BindingKind, BlockExpr, ClassMember, Expr, ForStatement, Program, Statement, UnaryOp};
use phalcom_ast::error::SyntaxError;
use phalcom_common::range::{EmptySourceRange, SourceRange};
use std::collections::HashSet;
use thiserror::Error;
use tracing::debug;
use indexmap::IndexMap;

/// An error raised while lowering the AST to bytecode.
#[derive(Error, Debug, Clone)]
pub enum CompilerError {
    /// A catch-all for otherwise-unclassified compilation failures.
    #[error("Unknown error during compilation.")]
    Unknown,

    /// A reference to a variable the compiler cannot resolve.
    #[error("Undefined variable '{0}'.")]
    UndefinedVariable(String),

    /// An assignment whose left-hand side is not an assignable target.
    #[error("Invalid assignment target.")]
    InvalidAssignmentTarget,

    /// A reassignment of a `let`-bound name (local, upvalue or global).
    ///
    /// `let` bindings are immutable per
    /// [ADR-0014](../../../docs/adr/0014-let-var-bindings.md); only `var`
    /// bindings may be reassigned. The offending name is carried for the
    /// diagnostic.
    #[error("Cannot reassign immutable `let` binding '{0}'; declare it with `var` to allow mutation.")]
    AssignToImmutable(String),

    /// A `let` binding written without an initializer.
    ///
    /// `let x` with no `= expr` is rejected at compile time
    /// ([ADR-0014](../../../docs/adr/0014-let-var-bindings.md)); an
    /// uninitialized binding must use `var x`, which reads the surface `None`
    /// value ([ADR-0007](../../../docs/adr/0007-option-type.md)). The offending
    /// name is carried for the diagnostic.
    #[error("`let` binding '{0}' requires an initializer; use `var {0}` for an uninitialized binding.")]
    LetWithoutInitializer(String),

    /// A branch condition that is a syntactically detectable `Option` literal.
    ///
    /// `Option` has no truth value: `if (None)`, `if (Some.new(x))` and the
    /// like are compile errors, and any non-`Bool` condition is a hard runtime
    /// type error (no coercion). Reach through `.isSome`/`.isNone` or use
    /// `ifSome`/`ifNone` instead
    /// ([ADR-0007](../../../docs/adr/0007-option-type.md),
    /// values-and-absence §3.5; BD-U6-1 enforcement = typed branch +
    /// literal-only compile check).
    #[error("An `Option` value has no truth value; use `.isSome`/`.isNone` or `ifSome`/`ifNone` instead of a boolean condition.")]
    OptionTruthiness,

    /// A syntax error surfaced from the front-end parser.
    #[error(transparent)]
    Parse(#[from] SyntaxError),

    /// A free-form compiler diagnostic.
    #[error("{0}")]
    Message(String),

    /// A field read whose name is in no assignment set in the class (ADR-0011).
    #[error("Read-before-write: field '{0}' is used before being assigned anywhere in this class.")]
    ReadBeforeWrite(String),

    /// An explicit value returned from a construct initializer.
    #[error("Cannot return a value from an initializer.")]
    ReturnValueFromInitializer,

    /// A `super.sel(…)` send written where there is no enclosing class body to
    /// anchor the walk (top level, or a free function).
    ///
    /// `super` starts method lookup at the *superclass of the defining class*
    /// (method-lookup.md §1.14, U-INH §3.4); with no defining class there is no
    /// superclass to start from.
    #[error("`super` cannot be used outside a method: there is no defining class to start the lookup above.")]
    SuperOutsideMethod,

    /// A bare `super` that is not the receiver of a message send.
    ///
    /// `super` is only meaningful as `super.sel(…)` — it names the current
    /// receiver but redirects the lookup start, so it has no value on its own
    /// (U-INH §3.4). `super` no longer silently evaluates to `nil`.
    #[error("`super` may only be used as the receiver of a message send, e.g. `super.method(...)`.")]
    BareSuper,

    /// A `break` written outside any enclosing loop.
    ///
    /// `break`/`continue` are resolved lexically against the compiler's
    /// loop-context stack (ADR-0035 §3, iteration.md §3, U-ITER specification
    /// §4); with the stack empty there is no loop to leave, so this is a
    /// compile error (C-ITER-7). The [`SourceRange`] is the offending keyword's
    /// span.
    #[error("`break` outside of a loop: `break` may only appear inside a `for` loop body.")]
    BreakOutsideLoop(SourceRange),

    /// A `continue` written outside any enclosing loop.
    ///
    /// The `continue` counterpart of [`CompilerError::BreakOutsideLoop`]
    /// (ADR-0035 §3, C-ITER-7). The [`SourceRange`] is the offending keyword's
    /// span.
    #[error("`continue` outside of a loop: `continue` may only appear inside a `for` loop body.")]
    ContinueOutsideLoop(SourceRange),
}

// impl From<CompilerError> for PhError {
//     fn from(err: CompilerError) -> Self {
//         PhError::Compile(err)
//     }
// }
//
// impl From<SyntaxError> for PhError {
//     fn from(err: SyntaxError) -> Self {
//         PhError::Compile(err.into())
//     }
// }

/// A lexically-scoped local variable inside a single function/block body.
struct Local {
    /// The interned name the local was declared with.
    name: Symbol,
    /// The nesting depth of the scope that declared it.
    depth: usize,
    /// Whether a nested block captured this local as an upvalue, which forces
    /// the enclosing frame to close (heap-promote) it on scope exit
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
    is_captured: bool,
    /// Whether the binding may be reassigned.
    ///
    /// `let` locals are immutable (`false`); `var` locals, method parameters
    /// and the receiver slot are mutable (`true`). The assignment path rejects
    /// a store to an immutable local
    /// ([ADR-0014](../../../docs/adr/0014-let-var-bindings.md)).
    is_mutable: bool,
}

/// The mutable per-function compilation state for one closure body.
///
/// Every method, getter, setter, block literal and the top-level module gets
/// its own `FunctionState`. The [`Compiler`] keeps them on a **stack**
/// ([`Compiler::functions`]) so upvalue resolution can walk from the innermost
/// block outward through its enclosing functions using plain indices — with no
/// aliasing `&mut` references and no raw parent pointers
/// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
pub(crate) struct FunctionState {
    /// The bytecode chunk being emitted for this body.
    pub(crate) chunk: Chunk,
    /// The live local variables, innermost last.
    locals: Vec<Local>,
    /// The current lexical scope nesting depth.
    scope_depth: usize,
    /// The number of live locals (mirrors `locals.len()`).
    num_locals: usize,
    /// The peak number of locals seen, used as the callable's slot count.
    max_slots: usize,
    /// The upvalue capture descriptors resolved for this body.
    upvalues: Vec<UpvalueDescriptor>,
    /// Whether this body compiles a constructor initializer (ADR-0011).
    is_constructor: bool,
    /// Whether this body compiles a **block literal** (rather than a method or
    /// constructor body).
    ///
    /// Set to `!is_method` in [`Compiler::compile_block`]. Gates
    /// [`Statement::Return`]'s opcode choice: a `return` in a block body emits
    /// [`Bytecode::ReturnNonLocal`] (unwind to the enclosing method), while a
    /// `return` in a method body keeps [`Bytecode::Return`] (blocks.md §5,
    /// [ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
    is_block: bool,
}

/// A single enclosing loop's control-flow context (ADR-0035 §3,
/// iteration.md §3, U-ITER specification §4).
///
/// One frame is pushed onto the compiler's loop-context stack while a `for`
/// body is compiled and popped afterwards; `break`/`continue` in the body
/// resolve to the **innermost** (top) frame. The frame records the chunk
/// indices of the forward [`Bytecode::Jump`] placeholders each `break`/
/// `continue` emits, so they can be backpatched to the loop's exit / cursor-
/// step labels once those are known. An empty stack at a `break`/`continue` is
/// the out-of-loop compile error (C-ITER-7).
struct LoopContext {
    /// The compiler function-nesting depth (`functions.len()`) at which this
    /// loop was entered.
    ///
    /// `break`/`continue` only bind to this loop when emitted **in the same
    /// function** (its inlined body). A control keyword reached inside a
    /// *deeper* function — e.g. the sacred inliner's deopt-fallback
    /// materialization of an `if` block into a closure, or a real block literal
    /// — cannot statically jump into this chunk, so it is compiled to a
    /// harmless dead no-op instead of corrupting this loop's patch list. This
    /// is what keeps `for (x in xs) { if (…) { break } }` sound: the guarded
    /// fast path splices the `break` inline (same function → real jump), while
    /// the never-taken fallback closure's copy becomes an inert `Jump(0)`.
    func_depth: usize,
    /// Chunk indices of `break` jumps, backpatched to the loop-exit label.
    break_jumps: Vec<usize>,
    /// Chunk indices of `continue` jumps, backpatched to the cursor-step label.
    continue_jumps: Vec<usize>,
}

impl FunctionState {
    /// Creates an empty function-compilation state for a body that is a
    /// constructor initializer (`is_constructor`) and/or a block literal
    /// (`is_block`); a plain method body and the top-level module body pass
    /// `false` for both.
    fn new(is_constructor: bool, is_block: bool) -> Self {
        FunctionState {
            chunk: Chunk::default(),
            locals: Vec::new(),
            scope_depth: 0,
            num_locals: 0,
            max_slots: 0,
            upvalues: Vec::new(),
            is_constructor,
            is_block,
        }
    }
}

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

    /// Emits `opcode` into the current function's chunk.
    pub(crate) fn emit(&mut self, opcode: Bytecode, range: SourceRange) {
        self.functions.last_mut().unwrap().chunk.add_instruction(opcode, range);
    }

    /// Adds `value` to the current function's constant pool, returning its index.
    pub(crate) fn add_constant(&mut self, value: Value) -> u16 {
        self.functions.last_mut().unwrap().chunk.add_constant(value)
    }

    /// Emits a read of the receiver `self` for the current body.
    ///
    /// In a method/module body `self` is the frame receiver, emitted as
    /// [`Bytecode::GetSelf`]. Inside a block, `self` is not the block object —
    /// it is the enclosing method's receiver, captured as an ordinary upvalue
    /// (functions.md §2), so it resolves through [`Self::resolve_upvalue`] and is
    /// emitted as [`Bytecode::GetUpvalue`].
    fn emit_self(&mut self, range: SourceRange) {
        if self.functions.len() > 1 {
            let self_sym = self.vm.interner.intern("self");
            if let Some(upvalue) = self.resolve_upvalue(self_sym) {
                self.emit(Bytecode::GetUpvalue(upvalue as u16), range);
                return;
            }
        }
        self.emit(Bytecode::GetSelf, range);
    }

    /// Lowers a `super.sel(args)` send to [`Bytecode::SuperSend`] (U-INH §3.4,
    /// method-lookup.md §1.14).
    ///
    /// Emits the original receiver (`self`), then the arguments, then a
    /// `SuperSend` carrying `selector_sym`, `argc`, and the **defining class
    /// name** ([`Self::current_class`]) so the VM starts the walk above that
    /// class at dispatch time (DEC-INH-B). A `super` with no enclosing class —
    /// at top level or in a free function — has no defining class to anchor the
    /// walk and is rejected with [`CompilerError::SuperOutsideMethod`].
    fn compile_super_send(&mut self, selector_sym: Symbol, args: Vec<Argument>, argc: u8, range: SourceRange) -> Result<(), CompilerError> {
        let defining = self.current_class.ok_or(CompilerError::SuperOutsideMethod)?;
        self.emit_self(range);
        for arg in args {
            self.compile_expr(arg.expr)?;
        }
        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
        let defining_idx = self.add_constant(Value::Symbol(defining));
        self.emit(Bytecode::SuperSend(argc, selector_idx, defining_idx), range);
        Ok(())
    }

    pub(crate) fn begin_scope(&mut self) {
        self.functions.last_mut().unwrap().scope_depth += 1;
    }

    /// Closes the innermost scope, promoting captured locals to heap cells.
    ///
    /// The function-body scope is cleaned up by [`Bytecode::Return`], which
    /// truncates the whole frame window (see the VM's `Return`/`run_until`
    /// handling). We therefore never `Pop` locals here — doing so would discard
    /// the return value sitting on top of them (the calculator-getter bug). We
    /// only emit [`Bytecode::CloseUpvalue`] for captured locals so their heap
    /// cells are promoted (`Open` -> `Closed`) before the slot is reclaimed
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)); the VM's
    /// `Return` closes them again idempotently for the explicit-return path.
    pub(crate) fn end_scope(&mut self, range: SourceRange) {
        let func = self.functions.last_mut().unwrap();
        func.scope_depth -= 1;
        let scope_depth = func.scope_depth;
        let mut to_close = Vec::new();
        while func.num_locals > 0 && func.locals[func.num_locals - 1].depth > scope_depth {
            func.num_locals -= 1;
            let local = func.locals.pop().unwrap();
            if local.is_captured {
                to_close.push(func.num_locals as u16);
            }
        }
        for slot in to_close {
            self.emit(Bytecode::CloseUpvalue(slot), range);
        }
    }

    /// Declares a new local named `name` in the current function.
    ///
    /// `is_mutable` records whether the binding may later be reassigned: `var`
    /// locals (and the synthetic receiver/parameter slots) pass `true`, while
    /// `let` locals pass `false` so the assignment path can reject stores to
    /// them ([ADR-0014](../../../docs/adr/0014-let-var-bindings.md)).
    fn add_local(&mut self, name: Symbol, is_mutable: bool) {
        let func = self.functions.last_mut().unwrap();
        debug!("[Compiler] Adding local at depth {}", func.scope_depth);
        func.locals.push(Local { name, depth: func.scope_depth, is_captured: false, is_mutable });
        func.num_locals += 1;
        if func.num_locals > func.max_slots {
            func.max_slots = func.num_locals;
        }
    }

    /// Resolves `name` as a local in the current function, returning its slot.
    fn resolve_local(&self, name: Symbol) -> Option<usize> {
        self.resolve_local_in(self.functions.len() - 1, name)
    }

    /// Resolves `name` as a local in the function at `func_idx`, returning its
    /// slot index (which doubles as the runtime stack slot within the frame).
    fn resolve_local_in(&self, func_idx: usize, name: Symbol) -> Option<usize> {
        let func = &self.functions[func_idx];
        (0..func.num_locals).rev().find(|&i| func.locals[i].name == name)
    }


    /// Resolves `name` as an upvalue captured by the current function.
    ///
    /// Walks the enclosing function-compilation states (via [`Self::functions`]
    /// indices — no aliasing borrows) marking the captured local so the
    /// enclosing frame closes it on scope exit
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)).
    fn resolve_upvalue(&mut self, name: Symbol) -> Option<usize> {
        self.resolve_upvalue_in(self.functions.len() - 1, name)
    }

    /// Resolves `name` as an upvalue of the function at `func_idx`, recursing
    /// into the enclosing function when the variable is itself an upvalue there.
    fn resolve_upvalue_in(&mut self, func_idx: usize, name: Symbol) -> Option<usize> {
        if func_idx == 0 {
            return None;
        }
        let enclosing = func_idx - 1;

        // 1. Resolve as a local in the enclosing function -> capture it directly.
        if let Some(slot) = self.resolve_local_in(enclosing, name) {
            self.functions[enclosing].locals[slot].is_captured = true;
            return Some(self.add_upvalue(func_idx, slot, true));
        }

        // 2. Otherwise resolve recursively as an upvalue of the enclosing
        //    function and chain through it.
        if let Some(upvalue_idx) = self.resolve_upvalue_in(enclosing, name) {
            return Some(self.add_upvalue(func_idx, upvalue_idx, false));
        }

        None
    }

    /// Records (deduplicating) an upvalue descriptor on the function at
    /// `func_idx`, returning its index in that function's upvalue list.
    fn add_upvalue(&mut self, func_idx: usize, index: usize, is_local: bool) -> usize {
        let upvalues = &mut self.functions[func_idx].upvalues;
        for (i, upval) in upvalues.iter().enumerate() {
            if upval.index == index && upval.is_local == is_local {
                return i;
            }
        }
        upvalues.push(UpvalueDescriptor { is_local, index });
        upvalues.len() - 1
    }

    /// Emits an ordinary `Invoke` send for operator selector `name` at
    /// `arity` (control-flow.md §1; U5-plan.md §4.1). Always builds the
    /// selector through [`encode_selector`] — never a hand-rolled string —
    /// so the compiler and the primitive registrations in `universe.rs`
    /// stay in lockstep (ADR-0012).
    fn emit_operator_send(&mut self, name: &str, arity: u8, range: SourceRange) {
        let labels = vec![None; arity as usize];
        let selector = encode_selector(name, &labels, SignatureKind::Method(arity));
        let selector_sym = self.vm.interner.intern(&selector);
        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
        self.emit(Bytecode::Invoke(arity, selector_idx), range);
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
    /// [`compile_inline_block_body`]: Self::compile_inline_block_body
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
                // immutable and *requires* an initializer (ADR-0014).
                let mutable = matches!(binding.kind, BindingKind::Var);

                match binding.value {
                    Some(expr) => self.compile_expr(expr)?,
                    None => {
                        if !mutable {
                            // `let x` with no initializer is a compile error.
                            return Err(CompilerError::LetWithoutInitializer(binding.name));
                        }
                        // `var x` with no initializer is backed by the private
                        // `nil` sentinel; the VM surfaces every read of that
                        // slot as the surface `None` value (ADR-0007/ADR-0010).
                        self.emit(Bytecode::Nil, range);
                    }
                }

                let name_sym = self.vm.interner.intern(&binding.name);
                if self.functions.last().unwrap().scope_depth > 0 {
                    // Local variable — record its mutability for the
                    // assignment path (ADR-0014).
                    self.add_local(name_sym, mutable);
                    let slot = self.functions.last().unwrap().num_locals - 1;
                    self.emit(Bytecode::SetLocal(slot as u16), range);
                } else {
                    // Module-level (global) variable. Globals never appear in
                    // `functions`, so an immutable `let` is tracked in a
                    // side set the assignment path consults (ADR-0014).
                    if !mutable {
                        self.immutable_globals.insert(name_sym);
                    }
                    let name_idx = self.add_constant(Value::Symbol(name_sym));
                    self.emit(Bytecode::DefineGlobal(name_idx), range);
                }
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
                let range = class_def.range;
                let name_sym = self.vm.interner.intern(&class_def.name);
                let name_idx = self.add_constant(Value::Symbol(name_sym));

                // 1. Whole-class field collection pass
                let mut own_instance_fields = Vec::new();
                let mut own_static_fields = Vec::new();

                // Pass 1: Collect static fields
                for member in &class_def.members {
                    match member {
                        ClassMember::Method(m) if m.is_static => {
                            let mut fields = Vec::new();
                            for stmt in &m.body {
                                collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                            }
                            for f in fields {
                                if !own_static_fields.contains(&f) {
                                    own_static_fields.push(f);
                                }
                            }
                        }
                        ClassMember::Getter(g) if g.is_static => {
                            if g.name.starts_with('_') {
                                let f = self.vm.interner.intern(&g.name);
                                if !own_static_fields.contains(&f) {
                                    own_static_fields.push(f);
                                }
                            }
                            let mut fields = Vec::new();
                            for stmt in &g.body {
                                collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                            }
                            for f in fields {
                                if !own_static_fields.contains(&f) {
                                    own_static_fields.push(f);
                                }
                            }
                        }
                        ClassMember::Setter(s) if s.is_static => {
                            let mut fields = Vec::new();
                            for stmt in &s.body {
                                collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                            }
                            for f in fields {
                                if !own_static_fields.contains(&f) {
                                    own_static_fields.push(f);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // Pass 2: Collect instance fields (only if not static)
                for member in &class_def.members {
                    match member {
                        ClassMember::Method(m) if !m.is_static => {
                            let mut fields = Vec::new();
                            for stmt in &m.body {
                                collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                            }
                            for f in fields {
                                if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                    own_instance_fields.push(f);
                                }
                            }
                        }
                        ClassMember::Getter(g) if !g.is_static => {
                            if g.name.starts_with('_') {
                                let f = self.vm.interner.intern(&g.name);
                                if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                    own_instance_fields.push(f);
                                }
                            }
                            let mut fields = Vec::new();
                            for stmt in &g.body {
                                collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                            }
                            for f in fields {
                                if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                    own_instance_fields.push(f);
                                }
                            }
                        }
                        ClassMember::Setter(s) if !s.is_static => {
                            let mut fields = Vec::new();
                            for stmt in &s.body {
                                collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                            }
                            for f in fields {
                                if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                    own_instance_fields.push(f);
                                }
                            }
                        }
                        ClassMember::Construct(c) => {
                            let mut fields = Vec::new();
                            for stmt in &c.body {
                                collect_assigned_fields_stmt(stmt, &mut fields, &mut self.vm.interner);
                            }
                            for f in fields {
                                if !own_static_fields.contains(&f) && !own_instance_fields.contains(&f) {
                                    own_instance_fields.push(f);
                                }
                            }
                        }
                        _ => {}
                    }
                }

                // 2. Build the ClassLayout and store it in VM.
                //
                // A subclass's own fields stack on top of the superclass's fields
                // (ADR-0011, U-INH §3.5): own instance/static slots begin at the
                // superclass's field count, so inherited slots keep their offsets
                // and are never aliased. The superclass's counts are resolved at
                // COMPILE time — from a reopened class already in `vm.classes`,
                // from the `extends` clause (looked up in the accumulating
                // `field_layouts`/`classes` metadata, since a *user* superclass
                // has not been created at runtime yet), or the implicit `Object`
                // root.
                let (sc_field_count, sc_meta_field_count) = if let Some(&existing_class) = self.vm.classes.get(&name_sym) {
                    // Reopening an existing class: keep its established superclass.
                    match self.vm.heap.class(existing_class).superclass {
                        Some(sc_id) => {
                            let meta = self.vm.heap.class(sc_id).class;
                            (self.vm.heap.class(sc_id).field_count, self.vm.heap.class(meta).field_count)
                        }
                        None => (0, 0),
                    }
                } else if let Some(sc_ref) = &class_def.superclass {
                    let sc_sym = self.vm.interner.intern(&sc_ref.name);
                    // Self-inheritance and unknown/forward superclasses are
                    // rejected here (U-INH §3.2): a class cannot appear in its own
                    // superclass chain (that would make method lookup
                    // non-terminating), and the single top-down compile pass
                    // requires the superclass to be defined earlier. A longer
                    // cycle is rejected transitively — the earlier class in the
                    // cycle refers forward to a not-yet-defined name.
                    if sc_sym == name_sym {
                        return Err(CompilerError::Message(format!(
                            "A class cannot extend itself: `{}` names itself as its superclass.",
                            class_def.name
                        )));
                    }
                    let counts = if let Some(layout) = self.vm.field_layouts.get(&sc_sym) {
                        (layout.field_count, layout.static_field_count)
                    } else if let Some(&sc_id) = self.vm.classes.get(&sc_sym) {
                        let meta = self.vm.heap.class(sc_id).class;
                        (self.vm.heap.class(sc_id).field_count, self.vm.heap.class(meta).field_count)
                    } else {
                        return Err(CompilerError::Message(format!(
                            "Unknown superclass `{}`: it must be a class defined before `{}`.",
                            sc_ref.name, class_def.name
                        )));
                    };
                    // Record the compile-time superclass edge (U-INH follow-on)
                    // ONLY here — past the self-check, on a known/validated
                    // superclass. The reopen branch above and the self/unknown
                    // error paths deliberately do not populate `class_parents`, so
                    // no self- or dangling edge can enter the map (the VM persists
                    // across REPL lines, so a stale edge would otherwise make the
                    // guard/alias chain-walks spin). Edges normally point only to a
                    // strictly-earlier-defined class; the one residual way to form a
                    // back-edge is a reopen-redefinition within a unit (`class A {}`,
                    // `class B extends A`, then `class A extends B`), which the
                    // `visited` guard in both chain-walks handles without spinning.
                    self.vm.class_parents.insert(name_sym, sc_sym);
                    counts
                } else {
                    // Implicit `Object` root.
                    let object_class = self.vm.universe.classes.object_class;
                    let meta = self.vm.heap.class(object_class).class;
                    (self.vm.heap.class(object_class).field_count, self.vm.heap.class(meta).field_count)
                };

                let mut field_slots = IndexMap::new();
                for (i, f) in own_instance_fields.iter().enumerate() {
                    field_slots.insert(*f, (sc_field_count as usize + i) as u16);
                }
                let field_count = sc_field_count + own_instance_fields.len() as u16;

                let mut static_field_slots = IndexMap::new();
                for (i, f) in own_static_fields.iter().enumerate() {
                    static_field_slots.insert(*f, (sc_meta_field_count as usize + i) as u16);
                }
                let static_field_count = sc_meta_field_count + own_static_fields.len() as u16;

                let layout = crate::vm::ClassLayout {
                    name: name_sym,
                    field_slots,
                    field_count,
                    static_field_slots,
                    static_field_count,
                };
                self.vm.field_layouts.insert(name_sym, layout);

                self.current_class = Some(name_sym);

                if let Some(&existing_class) = self.vm.classes.get(&name_sym) {
                    let class_idx = self.add_constant(Value::Obj(existing_class));
                    self.emit(Bytecode::Constant(class_idx), range);
                } else {
                    // Push the superclass onto the stack for the `Class` handler
                    // to consume (vm.rs `Bytecode::Class` pops it and wires both
                    // `superclass` and the parallel metaclass via `create_class`,
                    // ADR-0002 rule 4). An explicit `extends S` resolves `S` as an
                    // ordinary global at runtime; with no `extends` the class
                    // implicitly inherits from `Object`.
                    if let Some(sc_ref) = &class_def.superclass {
                        let sc_sym = self.vm.interner.intern(&sc_ref.name);
                        let sc_name_idx = self.add_constant(Value::Symbol(sc_sym));
                        self.emit(Bytecode::GetGlobal(sc_name_idx), sc_ref.range);
                    } else {
                        let object_class = self.vm.universe.classes.object_class;
                        let superclass_idx = self.add_constant(Value::Obj(object_class));
                        self.emit(Bytecode::Constant(superclass_idx), range);
                    }
                    self.emit(Bytecode::Class(name_idx), range);
                }

                // The class object is now on top of the stack. Iterate through members.
                for member in class_def.members {
                    match member {
                        ClassMember::Method(method_def) => {
                            let range = method_def.range;

                            let arity = method_def.params.len();
                            // At most one parameter may be the rest parameter, and only
                            // as the list's last entry — enforced by
                            // `parse_param_list` for every OTHER position, but a rest
                            // parameter that isn't last can still reach here if it is
                            // the only param (`self.is_rest` false paths aside, this
                            // guards the compiler's own invariant defensively, per U9's
                            // write-set: "reject any other param in the list with
                            // is_rest set").
                            if let Some(bad) = method_def.params.iter().take(arity.saturating_sub(1)).find(|p| p.is_rest) {
                                return Err(CompilerError::Message(format!(
                                    "rest parameter \"*{}\" must be the last parameter of \"{}\"",
                                    bad.name, method_def.name
                                )));
                            }
                            let is_variadic = method_def.params.last().is_some_and(|p| p.is_rest);

                            let sig_kind = if is_variadic {
                                // The rest parameter itself occupies one local slot
                                // beyond the `F` fixed parameters; `F` is the payload
                                // U9 corrections §0 point 3 requires for
                                // `SignatureKind::Variadic`, never spelled into the
                                // selector text.
                                let fixed_arity = (arity - 1) as u8;
                                SignatureKind::Variadic(fixed_arity)
                            } else {
                                SignatureKind::Method(arity as u8)
                            };

                            let labels: Vec<Option<String>> = method_def.params.iter().map(|p| p.label.clone()).collect();
                            let selector = encode_selector(&method_def.name, &labels, sig_kind);
                            let selector_sym = self.vm.interner.intern(&selector);

                            let param_names: Vec<String> = method_def.params.iter().map(|p| p.name.clone()).collect();
                            self.is_static_context = method_def.is_static;
                            let closure = self.compile_block(method_def.body, selector_sym, param_names, true, false)?;

                            debug!("[Compiler] Compiling method: {} (static: {})", selector, method_def.is_static);

                            let method_obj = self.vm.heap.alloc(Object::Method(MethodObject::new_single(
                                selector_sym,
                                sig_kind,
                                MethodKind::Closure(closure),
                            )));

                            let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                            self.emit(Bytecode::Constant(method_obj_idx), range);

                            let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                            self.emit(Bytecode::Method(selector_idx, method_def.is_static), range);
                        }
                        ClassMember::Getter(getter_def) => {
                            let range = getter_def.range;

                            if getter_def.name.starts_with('_') {
                                let layout = self.vm.field_layouts.get(&name_sym).unwrap().clone();
                                let field_name_sym = self.vm.interner.intern(&getter_def.name);
                                let slot = if getter_def.is_static {
                                    *layout.static_field_slots.get(&field_name_sym).ok_or_else(|| {
                                        CompilerError::Message(format!("Static field slot not found: {}", getter_def.name))
                                    })?
                                } else {
                                    *layout.field_slots.get(&field_name_sym).ok_or_else(|| {
                                        CompilerError::Message(format!("Instance field slot not found: {}", getter_def.name))
                                    })?
                                };

                                self.emit(Bytecode::Dup, range);
                                if let Statement::Expr { expr, .. } = &getter_def.body[0] {
                                    self.compile_expr(expr.clone())?;
                                } else {
                                    return Err(CompilerError::Message("Invalid field initializer body".to_string()));
                                }
                                self.emit(Bytecode::SetField(slot), range);
                                self.emit(Bytecode::Pop, range);
                            } else {
                                let selector = make_signature(&getter_def.name, SignatureKind::Getter);
                                let selector_sym = self.vm.interner.intern(&selector);

                                self.is_static_context = getter_def.is_static;
                                let closure = self.compile_block(getter_def.body, selector_sym, Vec::new(), true, false)?;

                                debug!("[Compiler] Compiling getter: {} (static: {})", selector, getter_def.is_static);

                                let method_obj =
                                    self.vm.heap.alloc(Object::Method(MethodObject::new_single(selector_sym, SignatureKind::Getter, MethodKind::Closure(closure))));

                                let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                                self.emit(Bytecode::Constant(method_obj_idx), range);

                                let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                                self.emit(Bytecode::Method(selector_idx, getter_def.is_static), range);
                            }
                        }
                        ClassMember::Setter(setter_def) => {
                            let range = setter_def.range;

                            let selector = make_signature(&setter_def.name, SignatureKind::Setter);
                            let selector_sym = self.vm.interner.intern(&selector);

                            self.is_static_context = setter_def.is_static;
                            let closure = self.compile_block(setter_def.body, selector_sym, vec![setter_def.param.clone()], true, false)?;

                            debug!("[Compiler] Compiling setter: {} (static: {})", selector, setter_def.is_static);

                            let method_obj =
                                self.vm.heap.alloc(Object::Method(MethodObject::new_single(selector_sym, SignatureKind::Setter, MethodKind::Closure(closure))));

                            let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                            self.emit(Bytecode::Constant(method_obj_idx), range);

                            let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                            self.emit(Bytecode::Method(selector_idx, setter_def.is_static), range);
                        }
                        ClassMember::Construct(construct_def) => {
                            let range = construct_def.range;

                            let arity = construct_def.params.len();
                            let labels: Vec<Option<String>> = construct_def.params.iter().map(|p| p.label.clone()).collect();
                            let selector = encode_selector(&construct_def.name, &labels, SignatureKind::Initializer(arity as u8));
                            let selector_sym = self.vm.interner.intern(&selector);

                            // Register the call-site alias (ADR-0011): user
                            // source calls a constructor as an ordinary send
                            // (`Counter.new()`), which encodes to the
                            // `SignatureKind::Method` selector below, not the
                            // `Initializer` selector installed above. Without
                            // this alias the call would silently resolve to
                            // the inherited `Object::new` bare-allocation
                            // primitive instead of this constructor.
                            let call_site_selector = encode_selector(&construct_def.name, &labels, SignatureKind::Method(arity as u8));
                            let call_site_sym = self.vm.interner.intern(&call_site_selector);
                            let class_name_sym = self.current_class.expect("construct is only compiled within a class body");
                            self.vm.constructor_aliases.insert((class_name_sym, call_site_sym), selector_sym);
                            if construct_def.name == "new" {
                                self.vm.has_new_construct.insert(class_name_sym);
                            }

                            let param_names: Vec<String> = construct_def.params.iter().map(|p| p.name.clone()).collect();
                            
                            self.is_static_context = false;
                            let closure = self.compile_block(construct_def.body, selector_sym, param_names, true, true)?;

                            debug!("[Compiler] Compiling constructor: {}", selector);

                            let method_obj = self.vm.heap.alloc(Object::Method(MethodObject::new_single(
                                selector_sym,
                                SignatureKind::Initializer(arity as u8),
                                MethodKind::Closure(closure),
                            )));

                            let method_obj_idx = self.add_constant(Value::Obj(method_obj));
                            self.emit(Bytecode::Constant(method_obj_idx), range);

                            let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                            self.emit(Bytecode::Method(selector_idx, true), range);
                        }
                    }
                }

                self.current_class = None;

                // After defining all methods, the class is still on the stack.
                // Define it as a global variable.
                self.emit(Bytecode::DefineGlobal(name_idx), range);
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
        }
        Ok(())
    }

    /// Reports whether `class_sym` or any of its compile-time ancestors
    /// (via [`VM::class_parents`](crate::vm::VM::class_parents)) declares a
    /// `new`-named `construct`.
    ///
    /// Inheritance-aware form of a bare `has_new_construct` membership test.
    /// A subclass that *inherits* a `new` constructor but declares none still
    /// has no user-visible bare allocator, so a mis-arity `Sub.new(...)` must
    /// error rather than silently fall through to the `Object.class::new`
    /// bare allocator and yield an uninitialized instance (U-INH follow-on;
    /// `docs/forge/DEFERRED.md` correctness entry). The walk terminates at the
    /// implicit `Object` root, which never has an edge in `class_parents`.
    fn inherits_new_construct(&self, mut class_sym: Symbol) -> bool {
        // `class_parents` is normally a strict-ancestry DAG, but a reopen-
        // redefinition within a unit can form a back-edge (see the populate site);
        // the `visited` guard makes the walk terminate regardless, so the compiler
        // never spins.
        let mut visited = std::collections::HashSet::new();
        while visited.insert(class_sym) {
            if self.vm.has_new_construct.contains(&class_sym) {
                return true;
            }
            match self.vm.class_parents.get(&class_sym) {
                Some(&parent) => class_sym = parent,
                None => return false,
            }
        }
        false
    }

    /// Resolves the `construct` call-site alias for `selector_sym` sent to
    /// `class_sym`, walking the compile-time superclass chain
    /// ([`VM::class_parents`](crate::vm::VM::class_parents)) so an *inherited*
    /// constructor — declared only on an ancestor — is redirected to its
    /// installed `Initializer` selector at the subclass call site, exactly as
    /// a locally declared one is (ADR-0011). Returns `None` if no class in the
    /// chain declares a `construct` matching `selector_sym`.
    fn lookup_constructor_alias(&self, mut class_sym: Symbol, selector_sym: Symbol) -> Option<Symbol> {
        // Nearest-declared wins: the walk checks each class before its parent, so
        // a subclass `construct` shadows an ancestor's of the same selector. The
        // `visited` guard terminates the walk even on a reopen-redefinition back-
        // edge (see [`Self::inherits_new_construct`]).
        let mut visited = std::collections::HashSet::new();
        while visited.insert(class_sym) {
            if let Some(&alias) = self.vm.constructor_aliases.get(&(class_sym, selector_sym)) {
                return Some(alias);
            }
            match self.vm.class_parents.get(&class_sym) {
                Some(&parent) => class_sym = parent,
                None => return None,
            }
        }
        None
    }

    /// Returns the current end of the function-under-compilation's chunk (the
    /// index the next emitted opcode will occupy) — a jump/loop label.
    fn chunk_len(&self) -> usize {
        self.functions.last().unwrap().chunk.code.len()
    }

    /// Emits a placeholder forward jump built by `make` (one of
    /// [`Bytecode::Jump`] / [`Bytecode::JumpIfFalse`]), returning its chunk
    /// index for a later [`Self::patch_forward_jump_to`].
    ///
    /// This is the loop-lowering counterpart of the sacred inliner's private
    /// `emit_jump` (`compiler/inliner.rs`); it is re-implemented here rather
    /// than shared because that helper is module-private to the inliner and
    /// U-ITER's write-set does not include `inliner.rs`.
    fn emit_forward_jump(&mut self, make: fn(i32) -> Bytecode, range: SourceRange) -> usize {
        self.emit(make(0), range);
        self.chunk_len() - 1
    }

    /// Backpatches the forward jump at chunk index `idx` so it lands at the
    /// absolute chunk index `target` (the [`Bytecode::Jump`] offset convention:
    /// relative to `ip` already advanced past the jump). Unlike the inliner's
    /// `patch_jump`, which always targets the current chunk end, `continue`
    /// must target the earlier cursor-step label, so this takes an explicit
    /// `target`.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is not a forward-jump opcode — a compiler-internal
    /// invariant, never user-reachable.
    fn patch_forward_jump_to(&mut self, idx: usize, target: usize) {
        let offset = target as i32 - (idx as i32 + 1);
        match &mut self.functions.last_mut().unwrap().chunk.code[idx] {
            Bytecode::Jump(o) | Bytecode::JumpIfFalse(o) => *o = offset,
            other => unreachable!("patch_forward_jump_to on a non-jump opcode: {other:?}"),
        }
    }

    /// Emits a backward [`Bytecode::Loop`] to the absolute chunk index
    /// `loop_start`, closing a `for` iteration (the loop-lowering counterpart
    /// of the inliner's private `emit_loop`).
    fn emit_backward_loop(&mut self, loop_start: usize, range: SourceRange) {
        let idx = self.chunk_len() as i32;
        self.emit(Bytecode::Loop(loop_start as i32 - (idx + 1)), range);
    }

    /// Emits a 0-arity getter send for `name` — the raw-name selector the
    /// getter is installed under (matching the [`Expr::GetProperty`] path), not
    /// an [`encode_selector`] method spelling. Used for `Option#isSome` in the
    /// `for` condition.
    fn emit_getter_send(&mut self, name: &str, range: SourceRange) {
        let sym = self.vm.interner.intern(name);
        let idx = self.add_constant(Value::Symbol(sym));
        self.emit(Bytecode::Invoke(0, idx), range);
    }

    /// Declares a loop local named `name` and returns its slot. `mutable`
    /// records whether user code may reassign it: the synthetic cursor/receiver
    /// temporaries pass `true`, the loop variable passes `false` (it behaves as
    /// a per-iteration `let`, iteration.md §2) — the compiler still rebinds it
    /// each step through a direct [`Bytecode::SetLocal`], which bypasses the
    /// user-facing immutability check.
    fn declare_loop_local(&mut self, name: &str, mutable: bool) -> usize {
        let sym = self.vm.interner.intern(name);
        self.add_local(sym, mutable);
        self.functions.last().unwrap().num_locals - 1
    }

    /// Lowers `for (binding in iter) { body }` to an inlined cursor `while`
    /// (ADR-0035 §2, iteration.md §2, U-ITER specification §3.1).
    ///
    /// The iterable is evaluated **exactly once** into a synthetic local; the
    /// loop then drives the two-selector protocol — `iterate(_)` /
    /// `iteratorValue(_)` as ordinary (never-inlined) sends — under a jump
    /// skeleton emitted **directly** as [`Bytecode::JumpIfFalse`] /
    /// [`Bytecode::Loop`] (D-ITER-2), **not** via a synthesized `whileTrue`
    /// send and **never** via `coll.each { … }`. This is load-bearing: the
    /// emitted chunk contains **no `block_call`** on the taken path, so a `for`
    /// body inside a fiber can `yield` freely (U-ITER specification §7.1,
    /// guarded by C-ITER-4). A single lowering path serves both the plain and
    /// the `break`/`continue` cases (D-ITER-3): a [`LoopContext`] is always
    /// pushed so control keywords resolve.
    ///
    /// The desugar realized (U-ITER specification §3.1), with `$coll`/`$cursor`
    /// synthetic locals:
    ///
    /// ```text
    /// $coll   = iter
    /// $cursor = $coll.iterate(None)
    /// loop:   if !$cursor.isSome -> exit
    ///         binding = $coll.iteratorValue($cursor.unwrapOr(0))
    ///         <body>                       ; break -> exit, continue -> step
    /// step:   $cursor = $coll.iterate($cursor)
    ///         Loop -> loop
    /// exit:
    /// ```
    ///
    /// `$cursor.unwrapOr(0)` extracts the live index from the `Some` the loop
    /// condition just proved present — this surface has no bare `Option#unwrap`,
    /// and the `0` default is never observed.
    ///
    /// # Errors
    ///
    /// Propagates any error compiling the iterable expression or the body.
    fn compile_for(&mut self, for_stmt: ForStatement) -> Result<(), CompilerError> {
        let range = for_stmt.range;
        // A fresh scope keeps the synthetic temporaries and the loop variable
        // out of the enclosing scope after the loop.
        self.begin_scope();

        // 1. Evaluate the iterable exactly once into `$coll`.
        self.compile_expr(for_stmt.iter)?;
        let coll_slot = self.declare_loop_local("$for_coll", true);
        self.emit(Bytecode::SetLocal(coll_slot as u16), range);

        // 2. `$cursor = $coll.iterate(None)` — `Bytecode::Nil` pushes the
        //    surface `None` singleton that starts the cursor.
        self.emit(Bytecode::GetLocal(coll_slot as u16), range);
        self.emit(Bytecode::Nil, range);
        self.emit_operator_send("iterate", 1, range);
        let cursor_slot = self.declare_loop_local("$for_cursor", true);
        self.emit(Bytecode::SetLocal(cursor_slot as u16), range);

        // 3. Declare the loop variable once (rebound each step); placeholder.
        self.emit(Bytecode::Nil, range);
        let binding_slot = self.declare_loop_local(&for_stmt.binding, false);

        // 4. Enter the loop context so body `break`/`continue` resolve here.
        self.loop_contexts.push(LoopContext {
            func_depth: self.functions.len(),
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });

        // loop_start: the condition test `$cursor.isSome`.
        let loop_start = self.chunk_len();
        self.emit(Bytecode::GetLocal(cursor_slot as u16), range);
        self.emit_getter_send("isSome", range);
        let exit_on_false = self.emit_forward_jump(Bytecode::JumpIfFalse, range);

        // Bind the loop variable: `binding = $coll.iteratorValue($cursor.unwrapOr(0))`.
        self.emit(Bytecode::GetLocal(coll_slot as u16), range);
        self.emit(Bytecode::GetLocal(cursor_slot as u16), range);
        let zero_idx = self.add_constant(Value::Number(0.0));
        self.emit(Bytecode::Constant(zero_idx), range);
        self.emit_operator_send("unwrapOr", 1, range);
        self.emit_operator_send("iteratorValue", 1, range);
        self.emit(Bytecode::SetLocal(binding_slot as u16), range);
        self.emit(Bytecode::Pop, range);

        // Body — a nested scope; each statement's value is discarded.
        self.begin_scope();
        for stmt in for_stmt.body {
            self.compile_statement_with_pop_control(stmt, true)?;
        }
        self.end_scope(range);

        // step: (the `continue` target) advance the cursor.
        let step_label = self.chunk_len();
        self.emit(Bytecode::GetLocal(coll_slot as u16), range);
        self.emit(Bytecode::GetLocal(cursor_slot as u16), range);
        self.emit_operator_send("iterate", 1, range);
        self.emit(Bytecode::SetLocal(cursor_slot as u16), range);
        self.emit(Bytecode::Pop, range);
        self.emit_backward_loop(loop_start, range);

        // exit: (the `break` and condition-false target).
        let exit_label = self.chunk_len();
        self.patch_forward_jump_to(exit_on_false, exit_label);

        let ctx = self.loop_contexts.pop().expect("loop context was pushed above");
        for jump in ctx.break_jumps {
            self.patch_forward_jump_to(jump, exit_label);
        }
        for jump in ctx.continue_jumps {
            self.patch_forward_jump_to(jump, step_label);
        }

        self.end_scope(range);
        Ok(())
    }

    /// Lowers a `break` statement (ADR-0035 §3, iteration.md §3, U-ITER
    /// specification §4): an unconditional forward [`Bytecode::Jump`] recorded
    /// on the innermost [`LoopContext`], backpatched to the loop-exit label by
    /// [`Self::compile_for`].
    ///
    /// # Errors
    ///
    /// Returns [`CompilerError::BreakOutsideLoop`] (with the keyword span) when
    /// no loop encloses the `break` (C-ITER-7).
    fn compile_break(&mut self, range: SourceRange) -> Result<(), CompilerError> {
        let Some(ctx) = self.loop_contexts.last() else {
            return Err(CompilerError::BreakOutsideLoop(range));
        };
        let same_function = ctx.func_depth == self.functions.len();
        let jump = self.emit_forward_jump(Bytecode::Jump, range);
        if same_function {
            self.loop_contexts.last_mut().unwrap().break_jumps.push(jump);
        }
        // A deeper function (a materialized block / deopt fallback) leaves the
        // `Jump(0)` as inert dead code — see [`LoopContext::func_depth`].
        Ok(())
    }

    /// Lowers a `continue` statement (ADR-0035 §3, iteration.md §3, U-ITER
    /// specification §4): an unconditional forward [`Bytecode::Jump`] recorded
    /// on the innermost [`LoopContext`], backpatched to the cursor-step label
    /// (so the next `iterate(_)` runs) by [`Self::compile_for`].
    ///
    /// # Errors
    ///
    /// Returns [`CompilerError::ContinueOutsideLoop`] (with the keyword span)
    /// when no loop encloses the `continue` (C-ITER-7).
    fn compile_continue(&mut self, range: SourceRange) -> Result<(), CompilerError> {
        let Some(ctx) = self.loop_contexts.last() else {
            return Err(CompilerError::ContinueOutsideLoop(range));
        };
        let same_function = ctx.func_depth == self.functions.len();
        let jump = self.emit_forward_jump(Bytecode::Jump, range);
        if same_function {
            self.loop_contexts.last_mut().unwrap().continue_jumps.push(jump);
        }
        // A deeper function (a materialized block / deopt fallback) leaves the
        // `Jump(0)` as inert dead code — see [`LoopContext::func_depth`].
        Ok(())
    }

    /// Compiles `expr`, always leaving exactly one value on the stack.
    /// Equivalent to `compile_expr_want(expr, true)` — see that method for
    /// `want_value`.
    pub(crate) fn compile_expr(&mut self, expr: Expr) -> Result<(), CompilerError> {
        self.compile_expr_want(expr, true)
    }

    /// Compiles `expr`, always leaving exactly one value on the stack.
    /// `want_value` is `false` only when the immediate caller is about to
    /// discard that value with a `Pop` right after
    /// (`compile_statement_with_pop_control`'s bare-statement case) — it lets
    /// a recognized one-armed sacred conditional (`ifTrue`/`ifFalse`) skip
    /// its `Some`-wrap allocation, since the wrap is unobservable when the
    /// value is popped unread (U-CORE-2; see
    /// [`Self::compile_sacred_call_want`]). Every other expression shape
    /// ignores `want_value` — it still pushes its one value as normal.
    pub(crate) fn compile_expr_want(&mut self, expr: Expr, want_value: bool) -> Result<(), CompilerError> {
        match expr {
            Expr::MethodCall(method_call) => {
                // A `super.sel(args)` send lowers to `SuperSend`, never an
                // ordinary `Invoke` — and must be intercepted *before* the
                // sacred inliner, so a `super.ifTrue { … }` is a real dispatch
                // starting above the defining class, not an inlined fast path
                // keyed on the receiver's static type (U-INH §3.4).
                if matches!(&method_call.object, Expr::SuperVar { .. }) {
                    let mc = *method_call;
                    let argc = mc.args.len();
                    let labels: Vec<Option<String>> = mc.args.iter().map(|a| a.label.clone()).collect();
                    let selector = encode_selector(&mc.method, &labels, SignatureKind::Method(argc as u8));
                    let selector_sym = self.vm.interner.intern(&selector);
                    return self.compile_super_send(selector_sym, mc.args, argc as u8, mc.range);
                }

                // U5 Layer 1 (control-flow.md §3, ADR-0018): a sacred
                // selector sent with literal-block arguments compiles to a
                // guarded inline fast path instead of a plain send. Every
                // other call — including a sacred selector with a
                // *non*-literal block argument — falls through unchanged.
                let range = method_call.range;
                match inliner::recognize(*method_call) {
                    Ok(sacred) => {
                        // BD-U6-1 (ADR-0007, values-and-absence §3.5): a
                        // conditional's condition that is a syntactically
                        // detectable `Option` literal (`if (None) { … }`,
                        // `if (Some.new(x)) { … }`, `None and …`) is a compile
                        // error — `Option` has no truth value. General
                        // non-`Bool` conditions are a hard runtime type error
                        // via the branch opcode's `Bool` requirement.
                        if let Some(condition) = branch_condition_of(&sacred)
                            && is_option_literal(condition)
                        {
                            return Err(CompilerError::OptionTruthiness);
                        }
                        return self.compile_sacred_call_want(sacred, range, want_value);
                    }
                    Err(method_call) => {
                        // A literal `ClassName.method(...)` receiver may name
                        // a `construct` (ADR-0011): redirect the call-site
                        // selector to the `Initializer` selector it was
                        // actually installed under, so `Counter.new()` reaches
                        // the constructor instead of the inherited
                        // `Object::new` bare-allocation primitive.
                        let receiver_class_sym = match &method_call.object {
                            Expr::Var { value, .. } => Some(self.vm.interner.intern(value)),
                            _ => None,
                        };

                        let arity = method_call.args.len();
                        let labels: Vec<Option<String>> = method_call.args.iter().map(|a| a.label.clone()).collect();
                        let selector = encode_selector(&method_call.method, &labels, SignatureKind::Method(arity as u8));
                        let selector_sym = self.vm.interner.intern(&selector);
                        let alias = receiver_class_sym.and_then(|class_sym| self.lookup_constructor_alias(class_sym, selector_sym));

                        // U7-plan §6 negative: a class with a `new`
                        // constructor has no user-visible bare allocator —
                        // a `new(...)` call whose arity/labels match no
                        // declared `construct` must not silently fall
                        // through to the inherited `Object::new` primitive.
                        if alias.is_none()
                            && method_call.method == "new"
                            && let Some(class_sym) = receiver_class_sym
                            && self.inherits_new_construct(class_sym)
                        {
                            return Err(CompilerError::Message(format!(
                                "No constructor `{}.new(...)` matches this call: arity/labels don't match any declared `construct`",
                                self.vm.resolve_symbol(class_sym)
                            )));
                        }

                        self.compile_expr(method_call.object)?;
                        for arg in &method_call.args {
                            self.compile_expr(arg.expr.clone())?;
                        }
                        let selector_sym = alias.unwrap_or(selector_sym);
                        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                        self.emit(Bytecode::Invoke(method_call.args.len() as u8, selector_idx), method_call.range);
                    }
                }
            }
            Expr::GetProperty(get_prop) => {
                // `super.prop` is a zero-arg super send (U-INH §3.4); the
                // getter/no-arg selector is the bare property name, matching the
                // ordinary getter dispatch below.
                if matches!(&get_prop.object, Expr::SuperVar { .. }) {
                    let selector_sym = self.vm.interner.intern(&get_prop.property);
                    return self.compile_super_send(selector_sym, Vec::new(), 0, get_prop.range);
                }
                self.compile_expr(get_prop.object)?;
                let selector_sym = self.vm.interner.intern(&get_prop.property);
                let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                self.emit(Bytecode::Invoke(0, selector_idx), get_prop.range);
            }
            Expr::SetProperty(set_prop) => {
                self.compile_expr(set_prop.object)?;
                self.compile_expr(set_prop.value)?;
                let selector = make_signature(&set_prop.property, SignatureKind::Setter);
                let selector_sym = self.vm.interner.intern(&selector);
                let selector_idx = self.add_constant(Value::Symbol(selector_sym));
                self.emit(Bytecode::Invoke(1, selector_idx), set_prop.range);
            }
            Expr::Number { value, range } => {
                let idx = self.add_constant(Value::Number(value));
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::String { value, range } => {
                let string_obj = self.vm.alloc_string_value(value);
                let idx = self.add_constant(string_obj);
                self.emit(Bytecode::Constant(idx), range);
            }
            Expr::Boolean { value, range } => {
                if value {
                    self.emit(Bytecode::True, range);
                } else {
                    self.emit(Bytecode::False, range);
                }
            }
            Expr::Var { value, range } => {
                let name_sym = self.vm.interner.intern(&value);
                if let Some(slot) = self.resolve_local(name_sym) {
                    self.emit(Bytecode::GetLocal(slot as u16), range);
                } else if let Some(upvalue) = self.resolve_upvalue(name_sym) {
                    self.emit(Bytecode::GetUpvalue(upvalue as u16), range);
                } else {
                    let name_idx = self.add_constant(Value::Symbol(name_sym));
                    self.emit(Bytecode::GetGlobal(name_idx), range);
                }
            }
            Expr::Field { value, range } => {
                let name_sym = self.vm.interner.intern(&value);
                let class_sym = self.current_class.ok_or_else(|| {
                    CompilerError::Message(format!("Fields can only be accessed within a class: {}", value))
                })?;
                let layout = self.vm.field_layouts.get(&class_sym).cloned().ok_or_else(|| {
                    CompilerError::Message(format!("No layout registered for class: {}", self.vm.resolve_symbol(class_sym)))
                })?;

                if let Some(&slot) = layout.static_field_slots.get(&name_sym) {
                    if self.is_static_context {
                        self.emit_self(range);
                    } else {
                        self.emit_self(range);
                        let class_sym = self.vm.interner.intern("class");
                        let class_idx = self.add_constant(Value::Symbol(class_sym));
                        self.emit(Bytecode::Invoke(0, class_idx), range);
                    }
                    self.emit(Bytecode::GetField(slot), range);
                } else if let Some(&slot) = layout.field_slots.get(&name_sym) {
                    self.emit_self(range);
                    self.emit(Bytecode::GetField(slot), range);
                } else {
                    return Err(CompilerError::ReadBeforeWrite(value.clone()));
                }
            }
            Expr::Assignment(assign_expr) => {
                match *assign_expr.name {
                    Expr::Var { value, range } => {
                        let name_sym = self.vm.interner.intern(&value);
                        // Enforce `let` immutability (ADR-0014) before
                        // evaluating the RHS. Resolution order mirrors the
                        // emit order below: current-function local, then an
                        // enclosing captured local (upvalue), then a global.
                        if let Some(slot) = self.resolve_local(name_sym) {
                            if !self.functions.last().unwrap().locals[slot].is_mutable {
                                return Err(CompilerError::AssignToImmutable(value));
                            }
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetLocal(slot as u16), range);
                        } else if let Some(upvalue) = self.resolve_upvalue(name_sym) {
                            // NOTE: reassignment of a *captured* `let` (an outer
                            // binding reached through an upvalue) is not yet
                            // rejected here — U6's stated scope is the
                            // current-function local and the module global.
                            // Tracked in DEFERRED.md.
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetUpvalue(upvalue as u16), range);
                        } else {
                            if self.immutable_globals.contains(&name_sym) {
                                return Err(CompilerError::AssignToImmutable(value));
                            }
                            self.compile_expr(assign_expr.value)?;
                            let name_idx = self.add_constant(Value::Symbol(name_sym));
                            self.emit(Bytecode::SetGlobal(name_idx), range);
                        }
                    }
                    Expr::Field { value, range } => {
                        let name_sym = self.vm.interner.intern(&value);
                        let class_sym = self.current_class.ok_or_else(|| {
                            CompilerError::Message(format!("Fields can only be accessed within a class: {}", value))
                        })?;
                        let layout = self.vm.field_layouts.get(&class_sym).cloned().ok_or_else(|| {
                            CompilerError::Message(format!("No layout registered for class: {}", self.vm.resolve_symbol(class_sym)))
                        })?;

                        if let Some(&slot) = layout.static_field_slots.get(&name_sym) {
                            if self.is_static_context {
                                self.emit_self(range);
                            } else {
                                self.emit_self(range);
                                let class_sym = self.vm.interner.intern("class");
                                let class_idx = self.add_constant(Value::Symbol(class_sym));
                                self.emit(Bytecode::Invoke(0, class_idx), range);
                            }
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetField(slot), range);
                        } else if let Some(&slot) = layout.field_slots.get(&name_sym) {
                            self.emit_self(range);
                            self.compile_expr(assign_expr.value)?;
                            self.emit(Bytecode::SetField(slot), range);
                        } else {
                            return Err(CompilerError::Message(format!("Field not collected in layout: {}", value)));
                        }
                    }
                    _ => return Err(CompilerError::InvalidAssignmentTarget),
                }
            }
            Expr::Binary(binary_expr) => {
                // U5 (control-flow.md §1): every binary operator is an
                // ordinary `Invoke` send — none of these are opcodes anymore.
                // `and`/`or` are the two *lazy* exceptions (control-flow.md
                // §2): their right operand compiles as a 0-arity block
                // literal, not a plain expression, so `Bool::and(_)`/`or(_)`
                // can choose whether to evaluate it at all. That literal
                // block is compiler-synthesized here but is exactly as
                // "literal" as a user-written `{ ... }` — U5-plan.md §4.2's
                // "literal block at the call site" inlining condition is
                // about the block's *shape*, not its origin — so `a and b`
                // is built directly as a recognized `SacredCall::And` and
                // handed to the same guarded-jump emitter `a.and { b }`
                // uses (`inliner.rs`), not a plain send.
                let range = binary_expr.range;
                match binary_expr.op {
                    BinaryOp::And => {
                        // The left operand is a branch condition (control-flow
                        // .md §2); reject a literal `Option` there (BD-U6-1).
                        if is_option_literal(&binary_expr.left) {
                            return Err(CompilerError::OptionTruthiness);
                        }
                        let rhs_block = wrap_expr_as_lazy_block(binary_expr.right, range);
                        return self.compile_sacred_call(inliner::SacredCall::And { receiver: binary_expr.left, rhs_block }, range);
                    }
                    BinaryOp::Or => {
                        if is_option_literal(&binary_expr.left) {
                            return Err(CompilerError::OptionTruthiness);
                        }
                        let rhs_block = wrap_expr_as_lazy_block(binary_expr.right, range);
                        return self.compile_sacred_call(inliner::SacredCall::Or { receiver: binary_expr.left, rhs_block }, range);
                    }
                    op => {
                        self.compile_expr(binary_expr.left)?;
                        self.compile_expr(binary_expr.right)?;
                        self.emit_operator_send(binary_op_selector_name(&op), 1, range);
                    }
                }
            }
            Expr::Unary(unary_expr) => {
                // U5: `-x`/`!x` lower to 0-arg sends (`negated()`/`not()`)
                // via the single `encode_selector` helper, replacing the
                // hand-rolled `"-"`/`"not"` lookup strings the old opcode
                // handlers used (ADR-0012 — "do not hand-roll a divergent
                // encoder", the F8 lesson).
                self.compile_expr(unary_expr.expr)?;
                let range = unary_expr.range;
                let name = match unary_expr.op {
                    UnaryOp::Negate => "negated",
                    UnaryOp::Not => "not",
                };
                self.emit_operator_send(name, 0, range);
            }
            Expr::SelfVar { range } => {
                self.emit_self(range);
            }
            Expr::SuperVar { range: _ } => {
                // A bare `super` that reaches here is not the receiver of a
                // message send (the `super.sel(…)` forms are intercepted in the
                // `MethodCall`/`GetProperty` arms). `super` has no value on its
                // own — it only redirects a send's lookup start (U-INH §3.4).
                return Err(CompilerError::BareSuper);
            }
            Expr::Block(block_expr) => {
                let name_sym = self.vm.interner.intern("<block>");
                let closure = self.compile_block(block_expr.body, name_sym, block_expr.params, false, false)?;
                let idx = self.add_constant(Value::Obj(closure));
                self.emit(Bytecode::Closure(idx), block_expr.range);
            }
            // Expr::Call(call_expr) => {
              //     // TODO: Implement function call compilation
              //     self.compile_expr(call_expr.callee)?;
              //     for arg in call_expr.args {
              //         self.compile_expr(arg)?;
              //     }
              //     // For now, push Nil as a placeholder for the return value
              //     self.emit(Bytecode::Nil);
              // }
        }
        Ok(())
    }
}

fn collect_assigned_fields(expr: &Expr, fields: &mut Vec<Symbol>, interner: &mut crate::interner::Interner) {
    match expr {
        Expr::Assignment(assign) => {
            if let Expr::Field { value, .. } = &*assign.name {
                let sym = interner.intern(value);
                if !fields.contains(&sym) {
                    fields.push(sym);
                }
            }
            collect_assigned_fields(&assign.value, fields, interner);
        }
        Expr::Unary(unary) => {
            collect_assigned_fields(&unary.expr, fields, interner);
        }
        Expr::Binary(binary) => {
            collect_assigned_fields(&binary.left, fields, interner);
            collect_assigned_fields(&binary.right, fields, interner);
        }
        Expr::MethodCall(call) => {
            collect_assigned_fields(&call.object, fields, interner);
            for arg in &call.args {
                collect_assigned_fields(&arg.expr, fields, interner);
            }
        }
        Expr::GetProperty(get_prop) => {
            collect_assigned_fields(&get_prop.object, fields, interner);
        }
        Expr::SetProperty(set_prop) => {
            collect_assigned_fields(&set_prop.object, fields, interner);
            collect_assigned_fields(&set_prop.value, fields, interner);
        }
        Expr::Block(block) => {
            for stmt in &block.body {
                collect_assigned_fields_stmt(stmt, fields, interner);
            }
        }
        _ => {}
    }
}

fn collect_assigned_fields_stmt(stmt: &Statement, fields: &mut Vec<Symbol>, interner: &mut crate::interner::Interner) {
    match stmt {
        Statement::Expr { expr, .. } => {
            collect_assigned_fields(expr, fields, interner);
        }
        Statement::Let(binding) => {
            if let Some(ref val) = binding.value {
                collect_assigned_fields(val, fields, interner);
            }
        }
        Statement::Return(ret) => {
            if let Some(ref val) = ret.value {
                collect_assigned_fields(val, fields, interner);
            }
        }
        Statement::For(for_stmt) => {
            // A field assigned only inside a `for` body must still be
            // collected into the class layout (ADR-0035; U-ITER), or its
            // first read would trip `ReadBeforeWrite`.
            collect_assigned_fields(&for_stmt.iter, fields, interner);
            for body_stmt in &for_stmt.body {
                collect_assigned_fields_stmt(body_stmt, fields, interner);
            }
        }
        _ => {}
    }
}

/// Returns the branch-condition sub-expression of a recognized [`SacredCall`],
/// or `None` for the forms whose condition is a block rather than a plain
/// expression (`whileTrue`).
///
/// The condition is the receiver of the inlined conditional/short-circuit
/// selectors (`ifTrue:`/`ifFalse:`/`ifTrue:ifFalse:`/`and:`/`or:`); it is the
/// value the branch opcode tests, so it is exactly where the "no `Option`
/// truthiness" rule applies (BD-U6-1, values-and-absence §3.5).
fn branch_condition_of(sacred: &inliner::SacredCall) -> Option<&Expr> {
    match sacred {
        inliner::SacredCall::IfTrue { receiver, .. }
        | inliner::SacredCall::IfFalse { receiver, .. }
        | inliner::SacredCall::IfTrueIfFalse { receiver, .. }
        | inliner::SacredCall::And { receiver, .. }
        | inliner::SacredCall::Or { receiver, .. } => Some(receiver),
        inliner::SacredCall::WhileTrue { .. } => None,
    }
}

/// Reports whether `expr` is a syntactically detectable `Option` literal.
///
/// Matches the surface forms of the two `Option` cases that carry no truth
/// value ([ADR-0007](../../../docs/adr/0007-option-type.md)):
///
/// - the `None` singleton, which lexes to `Var { value: "None" }`; and
/// - a `Some.new(…)` construction — an [`Expr::MethodCall`] of `new` on the
///   `Some` class (Phalcom has no bare `Some(x)` call syntax, so construction
///   is always the explicit static `Some.new(x)` send).
///
/// This is the literal-only half of BD-U6-1's `if (opt)` compile check; every
/// non-literal, non-`Bool` condition is caught at runtime by the branch
/// opcode's `Bool` requirement.
fn is_option_literal(expr: &Expr) -> bool {
    match expr {
        Expr::Var { value, .. } => value == "None",
        Expr::MethodCall(call) => {
            call.method == "new" && matches!(&call.object, Expr::Var { value, .. } if value == "Some")
        }
        _ => false,
    }
}

/// Wraps `expr` in a synthetic 0-parameter, expression-bodied block literal
/// spanning `range`, for `and`/`or`'s lazily-evaluated right-hand side
/// (control-flow.md §2: `a and b` ≡ `a.and { b }`).
fn wrap_expr_as_lazy_block(expr: Expr, range: SourceRange) -> Expr {
    Expr::Block(Box::new(BlockExpr {
        params: Vec::new(),
        body: vec![Statement::Expr { expr, range }],
        expr_body: true,
        range,
    }))
}

/// Maps a non-lazy [`BinaryOp`] to the base selector name `emit_operator_send`
/// encodes it under. `And`/`Or` are handled separately (lazy — see
/// [`Compiler::compile_lazy_block_operand`]) and never reach this function.
///
/// # Panics
///
/// Panics if called with `BinaryOp::And`/`BinaryOp::Or` — a compiler-internal
/// invariant violation, not a user-reachable error.
fn binary_op_selector_name(op: &BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Modulo => "%",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::LessThan => "<",
        BinaryOp::LessThanOrEqual => "<=",
        BinaryOp::GreaterThan => ">",
        BinaryOp::GreaterThanOrEqual => ">=",
        BinaryOp::And | BinaryOp::Or => unreachable!("and/or are lazy and compiled separately"),
    }
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
