//! The sacred-selector inliner (control-flow.md §3, ADR-0012, ADR-0018).
//!
//! `recognize` decides, purely from the *shape* of a
//! [`phalcom_ast::ast::MethodCallExpr`] at compile time, whether a send is
//! eligible for inlining: the selector must be one of the sacred set
//! (`ifTrue(_)`, `ifFalse(_)`, `ifTrue(_:ifFalse:)` — see the note on that
//! spelling below —, `and(_)`, `or(_)`, `whileTrue(_)`) **and** every
//! block-typed argument (and, for `whileTrue`, the receiver too) must be a
//! **literal block** [`Expr::Block`] node at the call site, never a variable
//! holding a block (U5-plan.md §4.2). This is a purely syntactic,
//! zero-runtime-cost check — the actual soundness guard against override and
//! type mismatch is the [`crate::bytecode::Bytecode::GuardBool`] /
//! [`crate::bytecode::Bytecode::GuardBlock`] opcode emitted alongside the
//! inlined fast path, not this recognizer.
//!
//! Each `compile_*` method below emits **both** paths for its construct: a
//! guarded, zero-allocation inline fast path (the block body is spliced
//! directly into the enclosing function's bytecode — no [`crate::heap::ClosureObject`]
//! allocation, no call frame, spec §3's "zero closure allocation and zero
//! call frames on the common path") and a fallback that materializes the
//! block literal(s) and performs the exact same [`crate::bytecode::Bytecode::Invoke`]
//! send the non-inlined form would have compiled to. The deopt guard chooses
//! between them at runtime; the two paths are built to be **observationally
//! identical** in every case a Phalcom program can detect — see the
//! per-selector primitives in `primitive/boolean.rs`/`primitive/block.rs`,
//! which both paths ultimately agree with.
//!
//! ## The paired `ifTrue(_:ifFalse:)` selector
//!
//! control-flow.md §3 lists the paired conditional as `ifTrue(_)ifFalse(_)`,
//! modeled on Smalltalk's independently-worded `ifTrue:ifFalse:` keyword
//! message. Phalcom's actual selector model
//! ([ADR-0012](../../../docs/adr/accepted/0012-selector-encoding-and-dispatch.md))
//! has no such shape: a selector is **one** base name plus per-argument
//! *labels* on that name (`encode_selector`), not several independently
//! named keyword parts chained together. U5 realizes the same semantics as
//! `ifTrue(_:ifFalse:)` — one base name (`ifTrue`), a positional first
//! argument, and an `ifFalse:`-labeled second argument — which
//! `encode_selector` already expresses exactly (see
//! `Universe::install_primitives`' `bool_if_true_if_false` registration).
//! `if (c) { A } else { B }` desugars to this selector directly
//! (`phalcom-ast/src/parser.rs`'s `parse_if`), **not** to the spec's
//! illustrative `ifTrue{}.ifNone{}` `Option` chain — that chain depends on
//! U6's `Option`, which lands after U5, and (per the overlay's own hazard
//! note) breaks chaining in general (`ifTrue{None}` is indistinguishable
//! from the branch not taken). The paired selector is U6-independent and
//! directly inlinable.
//!
//! ## v0.2 conditional-surface decision — READ BEFORE CHANGING THE `Some`-LIFT
//!
//! The one-armed `ifTrue(_)`/`ifFalse(_)` forms return an `Option`
//! (`Some(A)` on the taken branch, `None` otherwise — U-CORE-2, ADR-0007),
//! and *that* is the sole reason `Compiler::compile_sacred_call_want`
//! carries a `want_value` flag and this module emits a **dual**
//! [`Bytecode::WrapSome`] path (wrap when the result is consumed, elide when
//! it is popped unread). The elision is a real optimization — `Some`
//! allocates, and one-armed `ifTrue` in *statement* position (effect-only
//! guards like `pred.ifTrue { xs.add(x) }`) is the common case — but the
//! flag is threaded from the caller rather than derived from a single
//! authoritative "is this value demanded" attribute, which is the known
//! fragility (the value-demand fact lives in two places: here, and
//! `compile_statement_with_pop_control`'s `emit_pop`).
//!
//! **The decision, for v0.2: leave this exactly as it is.** The *canonical*
//! two-armed conditional surface is the paired `ifTrue(_, ifFalse:_)` send
//! documented above — atomic, both arms present, returns the arm value `R`
//! directly, never sends a message to an `Option`, so it sidesteps the
//! chaining hazard entirely. The one-armed `Option`-returning forms are
//! **retained unchanged** because real code depends on them for their value
//! (`core.ph`'s cursor protocol, `(next < size).ifTrue { next }` →
//! `Some(next)`/`None`) and for effect (the guards above). We are not
//! renaming them, not switching them to `Result`, not returning a bespoke
//! false-branch sentinel, not returning the receiver `Bool`, and not adding
//! infix `.ifTrue{}.ifFalse{}` chaining sugar — each was considered and
//! rejected (a `Result<A, _>` degenerates to `Option<A>` with a dead error
//! payload and miscolors "false" as "failure"; a sentinel or receiver-return
//! reintroduces the half-`Option` collapse or forces presence-protocol
//! dispatch onto `Object`, contradicting the truthiness ban of ADR-0007
//! §3.5; a forced infix pair cannot be *forced* in a dynamic message-send
//! language and leaks a half-built conditional). See the session decision
//! record `iftrue-conditional-surface-decision` in the project memory.
//!
//! **Deferred fix (NOT scheduled, do not implement speculatively):** collapse
//! the `want_value` dual path by making value-demand a single inherited
//! codegen attribute threaded once from the statement boundary
//! (destination-driven / context-threading codegen), and/or introduce a
//! Bool-only `when`/`unless` effect-sibling so effect-only callers leave
//! `ifTrue` mono-purpose. Either removes the divergence risk without changing
//! `Option` semantics or representation. Until one lands, the dual path below
//! is correct and load-bearing — every edit must keep the wrapped and elided
//! arms observationally identical to the `bool_if_true` primitive fallback.

use super::lib::Compiler;
use crate::bytecode::Bytecode;
use crate::method::{SignatureKind, encode_selector};
use crate::value::Value;
use phalcom_ast::ast::{Argument, BlockExpr, Expr, MethodCallExpr, Statement};
use phalcom_common::range::SourceRange;

use super::lib::CompilerError;

/// A sacred-selector call site recognized by [`recognize`], holding the
/// receiver and block-argument AST nodes the corresponding `compile_*`
/// method needs. Consumes the original [`MethodCallExpr`]'s pieces rather
/// than re-deriving them, so recognition and compilation never disagree
/// about which arguments are the sacred blocks.
pub(crate) enum SacredCall {
    /// `cond.ifTrue { A }` — control-flow.md §3.
    IfTrue { receiver: Expr, then_block: Expr },
    /// `cond.ifFalse { A }` — control-flow.md §3.
    IfFalse { receiver: Expr, else_block: Expr },
    /// `cond.ifTrue(A, ifFalse: B)` — `if`/`else`'s desugar target; see the
    /// module doc for why this, not Smalltalk's `ifTrue:ifFalse:`, is the
    /// paired selector's actual shape.
    IfTrueIfFalse { receiver: Expr, then_block: Expr, else_block: Expr },
    /// `a.and { b }` — control-flow.md §2 (lazy).
    And { receiver: Expr, rhs_block: Expr },
    /// `a.or { b }` — control-flow.md §2 (lazy).
    Or { receiver: Expr, rhs_block: Expr },
    /// `{ cond }.whileTrue { body }` — receiver **and** argument are both
    /// literal blocks (control-flow.md §1/§3).
    WhileTrue { cond_block: Expr, body_block: Expr },
}

/// Recognizes whether `call` is a sacred-selector call site eligible for
/// inlining (control-flow.md §3, U5-plan.md §4.2): the `(method, args)`
/// shape must match one of the sacred selectors exactly, with every
/// block-typed argument a literal [`Expr::Block`] — a variable holding a
/// block (`let b = { ... }; cond.ifTrue(b)`) is deliberately **not**
/// recognized, since the inliner would then have no compile-time block body
/// to splice; that case falls through to an ordinary [`crate::bytecode::Bytecode::Invoke`]
/// send, which is correct, just not fast.
///
/// Consumes `call` and returns either the recognized [`SacredCall`] or the
/// original [`MethodCallExpr`] unchanged (`Err`) so the caller can fall
/// through to the ordinary send path with no cloning on the common
/// (non-sacred) case.
pub(crate) fn recognize(call: MethodCallExpr) -> Result<SacredCall, MethodCallExpr> {
    let is_literal_block = |a: &Argument| a.label.is_none() && matches!(a.expr, Expr::Block(_));
    match (call.method.as_str(), call.args.len()) {
        ("ifTrue", 1) if is_literal_block(&call.args[0]) => {
            let mut args = call.args;
            Ok(SacredCall::IfTrue {
                receiver: call.object,
                then_block: args.remove(0).expr,
            })
        }
        ("ifFalse", 1) if is_literal_block(&call.args[0]) => {
            let mut args = call.args;
            Ok(SacredCall::IfFalse {
                receiver: call.object,
                else_block: args.remove(0).expr,
            })
        }
        ("ifTrue", 2) if is_literal_block(&call.args[0]) && call.args[1].label.as_deref() == Some("ifFalse") && matches!(call.args[1].expr, Expr::Block(_)) => {
            let mut args = call.args;
            let else_block = args.remove(1).expr;
            let then_block = args.remove(0).expr;
            Ok(SacredCall::IfTrueIfFalse {
                receiver: call.object,
                then_block,
                else_block,
            })
        }
        ("and", 1) if is_literal_block(&call.args[0]) => {
            let mut args = call.args;
            Ok(SacredCall::And {
                receiver: call.object,
                rhs_block: args.remove(0).expr,
            })
        }
        ("or", 1) if is_literal_block(&call.args[0]) => {
            let mut args = call.args;
            Ok(SacredCall::Or {
                receiver: call.object,
                rhs_block: args.remove(0).expr,
            })
        }
        ("whileTrue", 1) if is_literal_block(&call.args[0]) && matches!(call.object, Expr::Block(_)) => {
            let mut args = call.args;
            Ok(SacredCall::WhileTrue {
                cond_block: call.object,
                body_block: args.remove(0).expr,
            })
        }
        _ => Err(call),
    }
}

impl<'vm> Compiler<'vm> {
    /// Compiles a recognized [`SacredCall`], dispatching to the per-selector
    /// emitter. Equivalent to `compile_sacred_call_want(call, range, true)` —
    /// see that method for `want_value`.
    pub(crate) fn compile_sacred_call(&mut self, call: SacredCall, range: SourceRange) -> Result<(), CompilerError> {
        self.compile_sacred_call_want(call, range, true)
    }

    /// Compiles a recognized [`SacredCall`], dispatching to the per-selector
    /// emitter. `want_value` is `false` only when the caller is about to
    /// discard the result immediately (a bare-statement `Pop`,
    /// `compile_statement_with_pop_control`'s `emit_pop` case) — the only
    /// place a `false` value is meaningful today is [`Self::compile_if_true`]/
    /// [`Self::compile_if_false`], whose taken arm can then skip the
    /// `Some`-wrap allocation entirely, since nothing observes `Some(A)` vs.
    /// `A` when both are popped unread on the next instruction (U-CORE-2, the
    /// ADR-0018 amendment). Every other sacred selector ignores it.
    pub(crate) fn compile_sacred_call_want(&mut self, call: SacredCall, range: SourceRange, want_value: bool) -> Result<(), CompilerError> {
        match call {
            SacredCall::IfTrue { receiver, then_block } => self.compile_if_true(receiver, then_block, range, want_value),
            SacredCall::IfFalse { receiver, else_block } => self.compile_if_false(receiver, else_block, range, want_value),
            SacredCall::IfTrueIfFalse {
                receiver,
                then_block,
                else_block,
            } => self.compile_if_true_if_false(receiver, then_block, else_block, range),
            SacredCall::And { receiver, rhs_block } => self.compile_and(receiver, rhs_block, range),
            SacredCall::Or { receiver, rhs_block } => self.compile_or(receiver, rhs_block, range),
            SacredCall::WhileTrue { cond_block, body_block } => self.compile_while_true(cond_block, body_block, range),
        }
    }

    /// Emits an ordinary `Invoke` send for a sacred selector's *fallback*
    /// path: `name` plus `labels` (one entry per positional argument, `None`
    /// for unlabeled, `Some(label)` for a labeled one, e.g. `[None,
    /// Some("ifFalse")]` for the paired conditional) — always through
    /// [`encode_selector`], the single source of truth for selector spelling
    /// (ADR-0012), so the inliner's fallback and `universe.rs`'s primitive
    /// registrations can never drift apart.
    fn emit_sacred_send(&mut self, name: &str, labels: &[Option<&str>], range: SourceRange) {
        let arity = labels.len() as u8;
        let owned_labels: Vec<Option<String>> = labels.iter().map(|l| l.map(str::to_string)).collect();
        let selector = encode_selector(name, &owned_labels, SignatureKind::Method(arity));
        let selector_sym = self.vm.interner.intern(&selector);
        let selector_idx = self.add_constant(Value::Symbol(selector_sym));
        self.emit(Bytecode::Invoke(arity, selector_idx), range);
    }

    /// Compiles `block`'s statements **inline** into the current function —
    /// no new [`super::lib::FunctionState`], no [`crate::heap::ClosureObject`]
    /// allocation, no call frame (spec §3's zero-allocation invariant). A
    /// `return` inside therefore compiles to the enclosing method's ordinary
    /// [`Bytecode::Return`] and unwinds to the home method exactly as the
    /// non-inlined send form's frame-token non-local return would
    /// ([ADR-0013](../../../docs/adr/accepted/0013-block-closure-upvalues.md)) — this
    /// is the "for free" non-local-return transparency U5-plan.md §4.2
    /// calls the highest-value correctness assertion in the unit.
    ///
    /// Guarantees exactly **one** value is left on the operand stack when
    /// control falls off the end normally (never after a `return`, which
    /// exits the frame instead): the last statement's expression value if it
    /// is an [`Statement::Expr`] or a *local* [`Statement::Let`] (always
    /// local here — this method opens its own scope first), or a pushed
    /// [`Bytecode::Nil`] placeholder otherwise (an empty body, or a body
    /// ending in a [`Statement::Class`], which — like every class
    /// declaration — always binds a global and leaves nothing behind).
    /// Every inlined arm relies on this exact contract to keep the stack
    /// depth identical across the guard's fast and fallback paths.
    ///
    /// # Errors
    ///
    /// Propagates any error compiling the block's statements.
    fn compile_inline_block_body(&mut self, block: BlockExpr) -> Result<(), CompilerError> {
        self.begin_scope();
        let range = block.range;
        let len = block.body.len();
        let mut leaves_value = false;
        for (i, statement) in block.body.into_iter().enumerate() {
            let is_last = i == len - 1;
            if is_last {
                leaves_value = matches!(statement, Statement::Expr { .. } | Statement::Let(_) | Statement::Return(_));
            }
            let emit_pop = !(is_last && leaves_value);
            self.compile_statement_with_pop_control(statement, emit_pop)?;
        }
        if !leaves_value {
            self.emit(Bytecode::Nil, range);
        }
        self.end_scope(range);
        Ok(())
    }

    /// Unwraps a recognized-literal-block [`Expr::Block`], panicking
    /// otherwise.
    ///
    /// # Panics
    ///
    /// Panics if `expr` is not [`Expr::Block`] — [`recognize`] already
    /// guaranteed this for every block this module compiles inline; a panic
    /// here means recognition and compilation disagreed, an inliner-internal
    /// bug, never a user-reachable error.
    fn expect_block(expr: Expr) -> BlockExpr {
        match expr {
            Expr::Block(b) => *b,
            other => unreachable!("inliner: expected a literal block, got {other:?}"),
        }
    }

    /// `cond.ifTrue { A }` — control-flow.md §3.
    ///
    /// Fast path: `⟨cond⟩; GuardBool→fallback; JumpIfFalse→else; ⟨inline A⟩;
    /// [WrapSome]; Jump→end; else: Nil; Jump→end; fallback: ⟨block A⟩; Invoke
    /// ifTrue(_:); end:`. The `else` arm's `Nil` (surfaced to `None`) is the
    /// "no branch taken" placeholder that keeps the stack depth identical to
    /// the taken-branch arm.
    ///
    /// `ifTrue` returns `Option` (U-CORE-2, ADR-0007): the taken arm's value
    /// is `Some`-lifted via [`Bytecode::WrapSome`] so the fast path matches
    /// the `bool_if_true` primitive fallback exactly (both arms are then
    /// `Some(A) ∪ None`, a well-formed `Option`). When `want_value` is
    /// `false` — the caller is about to `Pop` the result unread — the
    /// `WrapSome` is elided; see [`Self::compile_sacred_call_want`].
    fn compile_if_true(&mut self, receiver: Expr, then_block: Expr, range: SourceRange, want_value: bool) -> Result<(), CompilerError> {
        let then_block_fallback = then_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_forward_jump(Bytecode::GuardBool, range);
        let to_else = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        self.compile_inline_block_body(Self::expect_block(then_block))?;
        // Some-lift the taken arm so this fast path yields the same `Option`
        // as the `bool_if_true` fallback. Elided when the result is popped
        // unread — the load-bearing `want_value` dual path; see the module
        // doc's "v0.2 conditional-surface decision" before touching this.
        if want_value {
            self.emit(Bytecode::WrapSome, range);
        }
        let to_end_1 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(to_else);
        self.emit(Bytecode::Nil, range);
        let to_end_2 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(guard);
        self.with_deopt_fallback(|c| c.compile_expr(then_block_fallback))?;
        self.emit_sacred_send("ifTrue", &[None], range);
        self.patch_forward_jump(to_end_1);
        self.patch_forward_jump(to_end_2);
        Ok(())
    }

    /// `cond.ifFalse { A }` — control-flow.md §3, the mirror of
    /// [`Self::compile_if_true`] for the `false` branch. See that method's
    /// doc for the `Some`-lift and `want_value` elision.
    fn compile_if_false(&mut self, receiver: Expr, else_block: Expr, range: SourceRange, want_value: bool) -> Result<(), CompilerError> {
        let else_block_fallback = else_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_forward_jump(Bytecode::GuardBool, range);
        let to_body = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        // Condition is `true`: the `ifFalse` arm does not run.
        self.emit(Bytecode::Nil, range);
        let to_end_1 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(to_body);
        self.compile_inline_block_body(Self::expect_block(else_block))?;
        // Mirror of `compile_if_true`'s Some-lift; same load-bearing
        // `want_value` elision — see the module doc's "v0.2 conditional-surface
        // decision" before touching this.
        if want_value {
            self.emit(Bytecode::WrapSome, range);
        }
        let to_end_2 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(guard);
        self.with_deopt_fallback(|c| c.compile_expr(else_block_fallback))?;
        self.emit_sacred_send("ifFalse", &[None], range);
        self.patch_forward_jump(to_end_1);
        self.patch_forward_jump(to_end_2);
        Ok(())
    }

    /// `cond.ifTrue(A, ifFalse: B)` — `if`/`else`'s desugar target (see the
    /// module doc for the selector-spelling note). Both arms are covered, so
    /// unlike [`Self::compile_if_true`]/[`Self::compile_if_false`] there is
    /// no separate "no branch taken" `Nil` placeholder.
    fn compile_if_true_if_false(&mut self, receiver: Expr, then_block: Expr, else_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let then_fallback = then_block.clone();
        let else_fallback = else_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_forward_jump(Bytecode::GuardBool, range);
        let to_else = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        self.compile_inline_block_body(Self::expect_block(then_block))?;
        let to_end_1 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(to_else);
        self.compile_inline_block_body(Self::expect_block(else_block))?;
        let to_end_2 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(guard);
        self.with_deopt_fallback(|c| {
            c.compile_expr(then_fallback)?;
            c.compile_expr(else_fallback)
        })?;
        self.emit_sacred_send("ifTrue", &[None, Some("ifFalse")], range);
        self.patch_forward_jump(to_end_1);
        self.patch_forward_jump(to_end_2);
        Ok(())
    }

    /// `a.and { b }` — control-flow.md §2. Short-circuits to `false` without
    /// evaluating `b` when `a` is `false`; otherwise evaluates `b` inline and
    /// returns its value as-is (matching `primitive/boolean.rs`'s `bool_and`
    /// fallback exactly).
    fn compile_and(&mut self, receiver: Expr, rhs_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let rhs_fallback = rhs_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_forward_jump(Bytecode::GuardBool, range);
        let to_short_circuit = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        self.compile_inline_block_body(Self::expect_block(rhs_block))?;
        let to_end_1 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(to_short_circuit);
        self.emit(Bytecode::False, range);
        let to_end_2 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(guard);
        self.with_deopt_fallback(|c| c.compile_expr(rhs_fallback))?;
        self.emit_sacred_send("and", &[None], range);
        self.patch_forward_jump(to_end_1);
        self.patch_forward_jump(to_end_2);
        Ok(())
    }

    /// `a.or { b }` — control-flow.md §2, the mirror of [`Self::compile_and`]:
    /// short-circuits to `true` without evaluating `b` when `a` is `true`.
    fn compile_or(&mut self, receiver: Expr, rhs_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let rhs_fallback = rhs_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_forward_jump(Bytecode::GuardBool, range);
        let to_rhs = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        self.emit(Bytecode::True, range);
        let to_end_1 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(to_rhs);
        self.compile_inline_block_body(Self::expect_block(rhs_block))?;
        let to_end_2 = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(guard);
        self.with_deopt_fallback(|c| c.compile_expr(rhs_fallback))?;
        self.emit_sacred_send("or", &[None], range);
        self.patch_forward_jump(to_end_1);
        self.patch_forward_jump(to_end_2);
        Ok(())
    }

    /// `{ cond }.whileTrue { body }` — control-flow.md §1/§3.
    ///
    /// The deopt guard is checked **once**, before the loop, not every
    /// iteration: the receiver is always a compiler-materialized block
    /// literal (never a runtime-uncertain value), so the only thing that can
    /// go stale is whether `Block>>whileTrue(_)` itself was redefined —
    /// [`Bytecode::GuardBlock`] tests exactly that (see its doc). The
    /// condition is re-evaluated inline every iteration and its result is
    /// type-checked by [`Bytecode::JumpIfFalse`] itself (raising a runtime
    /// type error on a non-`Bool` condition) — this is what gives `while`'s
    /// "no truthiness" floor without a second guard opcode.
    ///
    /// A [`super::lib::Compiler::push_loop_context`]-pushed loop context wraps
    /// the fast-path loop body (U-ITER-FIX item 2, spec §3.2), so `break`/
    /// `continue` bind inside a bare `while` exactly as they do inside a
    /// `for` body: `break` jumps to the same "loop is over" label the
    /// condition-false exit already targets, `continue` jumps back to
    /// `loop_start` to retest the condition (there is no separate step label —
    /// re-evaluating the condition block *is* the step).
    fn compile_while_true(&mut self, cond_block: Expr, body_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let cond_fallback = cond_block.clone();
        let body_fallback = body_block.clone();
        let guard = self.emit_forward_jump(Bytecode::GuardBlock, range);
        let loop_start = self.functions.last().unwrap().chunk.code.len();
        // The context stays pushed across *both* the fast-path body below and
        // the deopt-fallback body compiled further down — they are the same
        // source block compiled twice — see
        // [`super::lib::Compiler::peek_loop_context_jumps`]'s doc for why.
        self.push_loop_context();
        self.compile_inline_block_body(Self::expect_block(cond_block))?;
        let to_exit = self.emit_forward_jump(Bytecode::JumpIfFalse, range);
        self.compile_inline_block_body(Self::expect_block(body_block))?;
        self.emit(Bytecode::Pop, range); // discard the per-iteration body result
        self.emit_backward_loop(loop_start, range);
        let (break_jumps, continue_jumps) = self.peek_loop_context_jumps();
        self.patch_forward_jump(to_exit);
        for jump in break_jumps {
            self.patch_forward_jump(jump);
        }
        for jump in continue_jumps {
            self.patch_forward_jump_to(jump, loop_start);
        }
        self.emit(Bytecode::Nil, range);
        let to_end = self.emit_forward_jump(Bytecode::Jump, range);
        self.patch_forward_jump(guard);
        self.with_deopt_fallback(|c| {
            c.compile_expr(cond_fallback)?;
            c.compile_expr(body_fallback)
        })?;
        self.emit_sacred_send("whileTrue", &[None], range);
        self.patch_forward_jump(to_end);
        // Discard the context now that both bodies are compiled (its jumps
        // were already patched from the peek above).
        self.pop_loop_context();
        Ok(())
    }
}
