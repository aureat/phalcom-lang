use crate::bytecode::Bytecode;
use crate::value::Value;
use phalcom_ast::ast::ForStatement;
use phalcom_common::range::SourceRange;

use super::Compiler;
use super::error::CompilerError;
use super::state::LoopContext;

impl<'vm> Compiler<'vm> {
    /// Pushes a fresh [`LoopContext`] for a loop entered at the current
    /// function-nesting depth (ADR-0035 §3, U-ITER specification §4).
    ///
    /// Shared by [`Self::compile_for`] and the inliner's `compile_while_true`
    /// (U-ITER-FIX item 2, spec §3.2) — both push one context per loop, so
    /// `break`/`continue` in either construct's body resolve against it via
    /// [`Self::compile_break`]/[`Self::compile_continue`].
    pub(crate) fn push_loop_context(&mut self) {
        self.loop_contexts.push(LoopContext {
            func_depth: self.functions.len(),
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });
    }

    /// Pops the innermost [`LoopContext`], returning its recorded `break`
    /// jump chunk indices and `continue` jump chunk indices (in that order)
    /// for the caller to backpatch to its own exit and step/retest labels.
    ///
    /// # Panics
    ///
    /// Panics if no loop context is on the stack — an internal invariant:
    /// every [`Self::push_loop_context`] caller pops exactly once, right
    /// after compiling its loop body.
    pub(crate) fn pop_loop_context(&mut self) -> (Vec<usize>, Vec<usize>) {
        let ctx = self.loop_contexts.pop().expect("loop context was pushed above");
        (ctx.break_jumps, ctx.continue_jumps)
    }

    /// Clones the innermost [`LoopContext`]'s recorded `break`/`continue` jump
    /// chunk indices **without** popping it.
    ///
    /// The inliner's `compile_while_true` (U-ITER-FIX item 2) needs this: the
    /// fast-path loop body and the deopt-fallback's materialized closure body
    /// are the *same* source block compiled twice, so the loop context must
    /// stay pushed across both — a `break`/`continue` reached through the
    /// fallback closure is a deeper function depth than the context's
    /// `func_depth` and so takes [`Self::emit_deopt_block_control_trap`]
    /// rather than recording a jump here, but it still needs a non-empty
    /// [`Self::loop_contexts`] stack to avoid a spurious
    /// [`CompilerError::BreakOutsideLoop`]/[`CompilerError::ContinueOutsideLoop`].
    /// The fast path's jumps are backpatched from this snapshot immediately
    /// after the fast-path loop is compiled; [`Self::pop_loop_context`] is
    /// called once more, after the fallback compiles too, to finally discard
    /// the context (its jumps are ignored there — already patched here).
    ///
    /// # Panics
    ///
    /// Panics if no loop context is on the stack — same invariant as
    /// [`Self::pop_loop_context`].
    pub(crate) fn peek_loop_context_jumps(&self) -> (Vec<usize>, Vec<usize>) {
        let ctx = self.loop_contexts.last().expect("loop context was pushed above");
        (ctx.break_jumps.clone(), ctx.continue_jumps.clone())
    }

    /// Emits a 0-arity getter send for `name` — the raw-name selector the
    /// getter is installed under (matching the [`phalcom_ast::ast::Expr::GetProperty`] path), not
    /// an [`crate::method::encode_selector`] method spelling. Used for `Option#isSome` in the
    /// `for` condition.
    pub(super) fn emit_getter_send(&mut self, name: &str, range: SourceRange) {
        let sym = self.vm.interner.intern(name);
        let idx = self.add_constant(Value::symbol(sym));
        self.emit(Bytecode::Invoke(0, idx), range);
    }

    /// Declares a loop local named `name` and returns its slot. `mutable`
    /// records whether user code may reassign it: the synthetic cursor/receiver
    /// temporaries pass `true`, the loop variable passes `false` (it behaves as
    /// a per-iteration `let`, iteration.md §2) — the compiler still rebinds it
    /// each step through a direct [`Bytecode::SetLocal`], which bypasses the
    /// user-facing immutability check.
    fn declare_loop_local(&mut self, name: &str, mutable: bool) -> Result<usize, CompilerError> {
        let sym = self.vm.interner.intern(name);
        self.add_local(sym, mutable)?;
        Ok(self.functions.last().unwrap().num_locals - 1)
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
    pub(super) fn compile_for(&mut self, for_stmt: ForStatement) -> Result<(), CompilerError> {
        let range = for_stmt.range;
        // A fresh scope keeps the synthetic temporaries and the loop variable
        // out of the enclosing scope after the loop.
        self.begin_scope();

        // 1. Evaluate the iterable exactly once into `$coll`.
        self.compile_expr(for_stmt.iter)?;
        let coll_slot = self.declare_loop_local("$for_coll", true)?;
        self.emit(Bytecode::SetLocal(coll_slot as u16), range);

        // 2. `$cursor = $coll.iterate(None)` — `Bytecode::Nil` pushes the
        //    immediate `None` value that starts the cursor.
        self.emit(Bytecode::GetLocal(coll_slot as u16), range);
        self.emit(Bytecode::Nil, range);
        self.emit_operator_send("iterate", 1, range);
        let cursor_slot = self.declare_loop_local("$for_cursor", true)?;
        self.emit(Bytecode::SetLocal(cursor_slot as u16), range);

        // 3. Declare the loop variable once (rebound each step); placeholder.
        self.emit(Bytecode::Nil, range);
        let binding_slot = self.declare_loop_local(&for_stmt.binding, false)?;

        // 4. Enter the loop context so body `break`/`continue` resolve here.
        self.push_loop_context();

        // loop_start: the condition test `$cursor == None`.
        let loop_start = self.chunk_len();
        self.emit(Bytecode::GetLocal(cursor_slot as u16), range);
        let exit_on_none = self.emit_forward_jump(Bytecode::JumpIfNone, range);

        // Bind the loop variable: `binding = $coll.iteratorValue($cursor)`.
        self.emit(Bytecode::GetLocal(coll_slot as u16), range);
        self.emit(Bytecode::GetLocal(cursor_slot as u16), range);
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

        // U-ITER-FIX item 3 (spec §3.3): the loop variable is one local slot
        // rebound every iteration via `SetLocal` below — without this, every
        // closure the body captured it in would share the *same* open
        // upvalue cell and all observe the loop's final value. Closing it
        // here, before the rebind, promotes this iteration's cell (if any
        // closure actually opened one) to an immutable heap copy; the next
        // `SetLocal` then writes to a plain stack slot with no attached
        // upvalue, so a closure captured on the *next* iteration lazily opens
        // a brand-new cell instead of reusing this one — each iteration gets
        // its own snapshot (matches inlined-`while`'s per-statement capture
        // behavior). `continue` lands at `step_label`, i.e. **at** this close,
        // so a closure captured before a `continue` still gets its own cell.
        // Only emitted when the body statically captured the binding at all
        // (`Local::is_captured`, set by `Self::resolve_upvalue_in` while
        // compiling the body above) — a harmless no-op close is still
        // avoided for the common case that never captures it.
        if self.functions.last().unwrap().locals[binding_slot].is_captured {
            self.emit(Bytecode::CloseUpvalue(binding_slot as u16), range);
        }

        self.emit(Bytecode::GetLocal(coll_slot as u16), range);
        self.emit(Bytecode::GetLocal(cursor_slot as u16), range);
        self.emit_operator_send("iterate", 1, range);
        self.emit(Bytecode::SetLocal(cursor_slot as u16), range);
        self.emit(Bytecode::Pop, range);
        self.emit_backward_loop(loop_start, range);

        // exit: (the `break` and condition-false target).
        let exit_label = self.chunk_len();
        self.patch_forward_jump_to(exit_on_none, exit_label);

        let (break_jumps, continue_jumps) = self.pop_loop_context();
        for jump in break_jumps {
            self.patch_forward_jump_to(jump, exit_label);
        }
        for jump in continue_jumps {
            self.patch_forward_jump_to(jump, step_label);
        }

        self.end_scope(range);
        Ok(())
    }

    /// Emits a runtime trap for a `break`/`continue` reached through a
    /// **materialized** block — a compiler function nested deeper than the
    /// loop body that owns the innermost [`LoopContext`]: the sacred
    /// inliner's deopt fallback for a non-`Bool` `if` condition, or an
    /// ordinary block-arg closure (`each { break }`) (U-ITER-FIX item 1(a);
    /// `docs/forge/DEFERRED.md`).
    ///
    /// A jump emitted here cannot statically reach the enclosing loop's own
    /// chunk — it lives in a different [`super::state::FunctionState`] entirely — so rather
    /// than the silent no-op U-ITER shipped with, this compiles an
    /// unconditional `Error.new(message).raise()` send carrying a descriptive
    /// message (U-REOPEN-FIX): the rare cross-block case now fails **loudly**
    /// at runtime instead of quietly falling through past the loop-control
    /// intent. Full non-local break/continue (threading the target across
    /// [`super::state::FunctionState`] frames so the jump truly escapes the closure) is a
    /// larger follow-on left for a future unit.
    fn emit_deopt_block_control_trap(&mut self, range: SourceRange) {
        let error_sym = self.vm.interner.intern("Error");
        let error_idx = self.add_constant(Value::symbol(error_sym));
        self.emit(Bytecode::GetGlobal(error_idx), range);
        let message = self.vm.alloc_string_value(
            "`break`/`continue` reached through a materialized block (a deopt fallback or a real \
             block-arg closure) cannot leave its enclosing loop — non-local break/continue across a \
             materialized block is not supported (U-REOPEN-FIX; see docs/forge/DEFERRED.md)."
                .to_string(),
        );
        let message_idx = self.add_constant(message);
        self.emit(Bytecode::Constant(message_idx), range);
        self.emit_operator_send("new", 1, range);
        self.emit_operator_send("raise", 0, range);
        // `raise` never returns normally (it unwinds the stack), so this `Pop`
        // is unreachable dead code — kept only so the chunk's static stack
        // shape stays balanced like any other statement.
        self.emit(Bytecode::Pop, range);
    }

    /// Lowers a `break` statement (ADR-0035 §3, iteration.md §3, U-ITER
    /// specification §4): an unconditional forward [`Bytecode::Jump`] recorded
    /// on the innermost [`LoopContext`], backpatched to the loop-exit label by
    /// [`Self::compile_for`] — or, when reached through a materialized block,
    /// [`Self::emit_deopt_block_control_trap`] (U-ITER-FIX item 1(a)).
    ///
    /// # Errors
    ///
    /// Returns [`CompilerError::BreakOutsideLoop`] (with the keyword span) when
    /// no loop encloses the `break` (C-ITER-7).
    pub(super) fn compile_break(&mut self, range: SourceRange) -> Result<(), CompilerError> {
        let Some(ctx) = self.loop_contexts.last() else {
            return Err(CompilerError::BreakOutsideLoop(range));
        };
        if ctx.func_depth == self.functions.len() {
            let jump = self.emit_forward_jump(Bytecode::Jump, range);
            self.loop_contexts.last_mut().unwrap().break_jumps.push(jump);
        } else {
            self.emit_deopt_block_control_trap(range);
        }
        Ok(())
    }

    /// Lowers a `continue` statement (ADR-0035 §3, iteration.md §3, U-ITER
    /// specification §4): an unconditional forward [`Bytecode::Jump`] recorded
    /// on the innermost [`LoopContext`], backpatched to the cursor-step label
    /// (so the next `iterate(_)` runs) by [`Self::compile_for`] — or, when
    /// reached through a materialized block,
    /// [`Self::emit_deopt_block_control_trap`] (U-ITER-FIX item 1(a)).
    ///
    /// # Errors
    ///
    /// Returns [`CompilerError::ContinueOutsideLoop`] (with the keyword span)
    /// when no loop encloses the `continue` (C-ITER-7).
    pub(super) fn compile_continue(&mut self, range: SourceRange) -> Result<(), CompilerError> {
        let Some(ctx) = self.loop_contexts.last() else {
            return Err(CompilerError::ContinueOutsideLoop(range));
        };
        if ctx.func_depth == self.functions.len() {
            let jump = self.emit_forward_jump(Bytecode::Jump, range);
            self.loop_contexts.last_mut().unwrap().continue_jumps.push(jump);
        } else {
            self.emit_deopt_block_control_trap(range);
        }
        Ok(())
    }
}
