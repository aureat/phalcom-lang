use crate::callable::UpvalueDescriptor;
use crate::chunk::Chunk;
use crate::interner::Symbol;

/// A lexically-scoped local variable inside a single function/block body.
pub(super) struct Local {
    /// The interned name the local was declared with.
    pub(super) name: Symbol,
    /// The nesting depth of the scope that declared it.
    pub(super) depth: usize,
    /// Whether a nested block captured this local as an upvalue, which forces
    /// the enclosing frame to close (heap-promote) it on scope exit
    /// ([ADR-0013](../../../docs/adr/accepted/0013-block-closure-upvalues.md)).
    pub(super) is_captured: bool,
    /// Whether the binding may be reassigned.
    ///
    /// `let` locals are immutable (`false`); `var` locals, method parameters
    /// and the receiver slot are mutable (`true`). The assignment path rejects
    /// a store to an immutable local
    /// ([ADR-0014](../../../docs/adr/accepted/0014-let-var-bindings.md)).
    pub(super) is_mutable: bool,
}

/// The mutable per-function compilation state for one closure body.
///
/// Every method, getter, setter, block literal and the top-level module gets
/// its own `FunctionState`. The [`super::Compiler`] keeps them on a **stack**
/// ([`super::Compiler::functions`]) so upvalue resolution can walk from the innermost
/// block outward through its enclosing functions using plain indices — with no
/// aliasing `&mut` references and no raw parent pointers
/// ([ADR-0013](../../../docs/adr/accepted/0013-block-closure-upvalues.md)).
pub(crate) struct FunctionState {
    /// The bytecode chunk being emitted for this body.
    pub(crate) chunk: Chunk,
    /// The live local variables, innermost last.
    pub(super) locals: Vec<Local>,
    /// The current lexical scope nesting depth.
    pub(super) scope_depth: usize,
    /// The number of live locals (mirrors `locals.len()`).
    pub(super) num_locals: usize,
    /// The peak number of locals seen, used as the callable's slot count.
    pub(super) max_slots: usize,
    /// The upvalue capture descriptors resolved for this body.
    pub(super) upvalues: Vec<UpvalueDescriptor>,
    /// Whether this body compiles a constructor initializer (ADR-0063).
    pub(super) is_constructor: bool,
    /// Source constructor name whose body this initializer executes, if any.
    ///
    /// This lets `super.<constructor-name>(...)` lower to the generated hidden
    /// instance-side initializer without changing ordinary `super` sends made
    /// from constructor bodies.
    pub(super) constructor_name: Option<String>,
    /// Whether this body compiles a **block literal** (rather than a method or
    /// constructor body).
    ///
    /// Set to `!is_method` in [`super::Compiler::compile_block`]. Gates
    /// [`phalcom_ast::ast::Statement::Return`]'s opcode choice: a `return` in a block body emits
    /// [`crate::bytecode::Bytecode::ReturnNonLocal`] (unwind to the enclosing method), while a
    /// `return` in a method body keeps [`crate::bytecode::Bytecode::Return`] (blocks.md §5,
    /// [ADR-0013](../../../docs/adr/accepted/0013-block-closure-upvalues.md)).
    pub(super) is_block: bool,
    /// All local variable names declared inside this function.
    pub(super) local_names: Vec<Symbol>,
}

/// A single enclosing loop's control-flow context (ADR-0035 §3,
/// iteration.md §3, U-ITER specification §4).
///
/// One frame is pushed onto the compiler's loop-context stack while a `for`
/// body is compiled and popped afterwards; `break`/`continue` in the body
/// resolve to the **innermost** (top) frame. The frame records the chunk
/// indices of the forward [`crate::bytecode::Bytecode::Jump`] placeholders each `break`/
/// `continue` emits, so they can be backpatched to the loop's exit / cursor-
/// step labels once those are known. An empty stack at a `break`/`continue` is
/// the out-of-loop compile error (C-ITER-7).
pub(super) struct LoopContext {
    /// The compiler function-nesting depth (`functions.len()`) at which this
    /// loop was entered.
    ///
    /// `break`/`continue` only bind to this loop when emitted **in the same
    /// function** (its inlined body). A control keyword reached inside a
    /// *deeper* function — e.g. the sacred inliner's deopt-fallback
    /// materialization of an `if` block into a closure, or a real block literal
    /// — cannot statically jump into this chunk, so it is compiled to a
    /// runtime trap (U-ITER-FIX item 1(a): [`super::Compiler::emit_deopt_block_control_trap`])
    /// instead of corrupting this loop's patch list. This is what keeps
    /// `for (x in xs) { if (…) { break } }` sound: the guarded fast path
    /// splices the `break` inline (same function → real jump), while the
    /// never-taken fallback closure's copy becomes a loud runtime error —
    /// never silently swallowed — should it somehow execute.
    pub(super) func_depth: usize,
    /// Chunk indices of `break` jumps, backpatched to the loop-exit label.
    pub(super) break_jumps: Vec<usize>,
    /// Chunk indices of `continue` jumps, backpatched to the cursor-step label.
    pub(super) continue_jumps: Vec<usize>,
}

impl FunctionState {
    /// Creates an empty function-compilation state for a body that is a
    /// constructor initializer (`is_constructor`) and/or a block literal
    /// (`is_block`); a plain method body and the top-level module body pass
    /// `false` for both.
    pub(super) fn new(is_constructor: bool, is_block: bool, constructor_name: Option<String>) -> Self {
        FunctionState {
            chunk: Chunk::default(),
            locals: Vec::new(),
            scope_depth: 0,
            num_locals: 0,
            max_slots: 0,
            upvalues: Vec::new(),
            is_constructor,
            constructor_name,
            is_block,
            local_names: Vec::new(),
        }
    }
}
