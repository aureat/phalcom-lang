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
//! directly into the enclosing function's bytecode — no [`crate::closure::ClosureObject`]
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
//! ([ADR-0012](../../../docs/adr/0012-selector-encoding-and-dispatch.md))
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

use super::lib::Compiler;
use crate::bytecode::Bytecode;
use crate::method::{encode_selector, SignatureKind};
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
            Ok(SacredCall::IfTrue { receiver: call.object, then_block: args.remove(0).expr })
        }
        ("ifFalse", 1) if is_literal_block(&call.args[0]) => {
            let mut args = call.args;
            Ok(SacredCall::IfFalse { receiver: call.object, else_block: args.remove(0).expr })
        }
        ("ifTrue", 2)
            if is_literal_block(&call.args[0])
                && call.args[1].label.as_deref() == Some("ifFalse")
                && matches!(call.args[1].expr, Expr::Block(_)) =>
        {
            let mut args = call.args;
            let else_block = args.remove(1).expr;
            let then_block = args.remove(0).expr;
            Ok(SacredCall::IfTrueIfFalse { receiver: call.object, then_block, else_block })
        }
        ("and", 1) if is_literal_block(&call.args[0]) => {
            let mut args = call.args;
            Ok(SacredCall::And { receiver: call.object, rhs_block: args.remove(0).expr })
        }
        ("or", 1) if is_literal_block(&call.args[0]) => {
            let mut args = call.args;
            Ok(SacredCall::Or { receiver: call.object, rhs_block: args.remove(0).expr })
        }
        ("whileTrue", 1) if is_literal_block(&call.args[0]) && matches!(call.object, Expr::Block(_)) => {
            let mut args = call.args;
            Ok(SacredCall::WhileTrue { cond_block: call.object, body_block: args.remove(0).expr })
        }
        _ => Err(call),
    }
}

impl<'vm> Compiler<'vm> {
    /// Compiles a recognized [`SacredCall`], dispatching to the per-selector
    /// emitter.
    pub(crate) fn compile_sacred_call(&mut self, call: SacredCall, range: SourceRange) -> Result<(), CompilerError> {
        match call {
            SacredCall::IfTrue { receiver, then_block } => self.compile_if_true(receiver, then_block, range),
            SacredCall::IfFalse { receiver, else_block } => self.compile_if_false(receiver, else_block, range),
            SacredCall::IfTrueIfFalse { receiver, then_block, else_block } => {
                self.compile_if_true_if_false(receiver, then_block, else_block, range)
            }
            SacredCall::And { receiver, rhs_block } => self.compile_and(receiver, rhs_block, range),
            SacredCall::Or { receiver, rhs_block } => self.compile_or(receiver, rhs_block, range),
            SacredCall::WhileTrue { cond_block, body_block } => self.compile_while_true(cond_block, body_block, range),
        }
    }

    /// Emits a placeholder single-`i32`-operand jump/guard instruction built
    /// by `make` (one of [`Bytecode::Jump`], [`Bytecode::JumpIfFalse`],
    /// [`Bytecode::GuardBool`], [`Bytecode::GuardBlock`] — each usable
    /// directly as an `fn(i32) -> Bytecode` tuple-variant constructor),
    /// returning its index in the current function's chunk for a later
    /// [`Self::patch_jump`].
    fn emit_jump(&mut self, make: fn(i32) -> Bytecode, range: SourceRange) -> usize {
        self.emit(make(0), range);
        self.functions.last().unwrap().chunk.code.len() - 1
    }

    /// Backpatches the placeholder jump/guard at chunk index `idx` so its
    /// offset lands exactly at the current end of the chunk (the
    /// [`Bytecode::Jump`] offset convention: relative to `ip` already
    /// advanced past the jump instruction itself).
    ///
    /// # Panics
    ///
    /// Panics if `idx` is not one of the four offset-carrying jump/guard
    /// opcodes — an inliner-internal invariant violation, never
    /// user-reachable.
    fn patch_jump(&mut self, idx: usize) {
        let target = self.functions.last().unwrap().chunk.code.len() as i32;
        let offset = target - (idx as i32 + 1);
        let code = &mut self.functions.last_mut().unwrap().chunk.code;
        match &mut code[idx] {
            Bytecode::Jump(o) | Bytecode::JumpIfFalse(o) | Bytecode::GuardBool(o) | Bytecode::GuardBlock(o) => *o = offset,
            other => unreachable!("patch_jump on a non-jump opcode: {other:?}"),
        }
    }

    /// Emits a backward [`Bytecode::Loop`] jumping to the absolute chunk
    /// index `loop_start`, closing an inlined `whileTrue` iteration.
    fn emit_loop(&mut self, loop_start: usize, range: SourceRange) {
        let idx = self.functions.last().unwrap().chunk.code.len() as i32;
        self.emit(Bytecode::Loop(loop_start as i32 - (idx + 1)), range);
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
    /// no new [`super::lib::FunctionState`], no [`crate::closure::ClosureObject`]
    /// allocation, no call frame (spec §3's zero-allocation invariant). A
    /// `return` inside therefore compiles to the enclosing method's ordinary
    /// [`Bytecode::Return`] and unwinds to the home method exactly as the
    /// non-inlined send form's frame-token non-local return would
    /// ([ADR-0013](../../../docs/adr/0013-block-closure-upvalues.md)) — this
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
    /// Jump→end; else: Nil; Jump→end; fallback: ⟨block A⟩; Invoke
    /// ifTrue(_:); end:`. The `else` arm's `Nil` is the "no branch taken"
    /// placeholder (U5 guardrail: `Value::Nil` stays as-is, U6's job) that
    /// keeps the stack depth identical to the taken-branch arm.
    fn compile_if_true(&mut self, receiver: Expr, then_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let then_block_fallback = then_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_jump(Bytecode::GuardBool, range);
        let to_else = self.emit_jump(Bytecode::JumpIfFalse, range);
        self.compile_inline_block_body(Self::expect_block(then_block))?;
        let to_end_1 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(to_else);
        self.emit(Bytecode::Nil, range);
        let to_end_2 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(guard);
        self.compile_expr(then_block_fallback)?;
        self.emit_sacred_send("ifTrue", &[None], range);
        self.patch_jump(to_end_1);
        self.patch_jump(to_end_2);
        Ok(())
    }

    /// `cond.ifFalse { A }` — control-flow.md §3, the mirror of
    /// [`Self::compile_if_true`] for the `false` branch.
    fn compile_if_false(&mut self, receiver: Expr, else_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let else_block_fallback = else_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_jump(Bytecode::GuardBool, range);
        let to_body = self.emit_jump(Bytecode::JumpIfFalse, range);
        // Condition is `true`: the `ifFalse` arm does not run.
        self.emit(Bytecode::Nil, range);
        let to_end_1 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(to_body);
        self.compile_inline_block_body(Self::expect_block(else_block))?;
        let to_end_2 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(guard);
        self.compile_expr(else_block_fallback)?;
        self.emit_sacred_send("ifFalse", &[None], range);
        self.patch_jump(to_end_1);
        self.patch_jump(to_end_2);
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
        let guard = self.emit_jump(Bytecode::GuardBool, range);
        let to_else = self.emit_jump(Bytecode::JumpIfFalse, range);
        self.compile_inline_block_body(Self::expect_block(then_block))?;
        let to_end_1 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(to_else);
        self.compile_inline_block_body(Self::expect_block(else_block))?;
        let to_end_2 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(guard);
        self.compile_expr(then_fallback)?;
        self.compile_expr(else_fallback)?;
        self.emit_sacred_send("ifTrue", &[None, Some("ifFalse")], range);
        self.patch_jump(to_end_1);
        self.patch_jump(to_end_2);
        Ok(())
    }

    /// `a.and { b }` — control-flow.md §2. Short-circuits to `false` without
    /// evaluating `b` when `a` is `false`; otherwise evaluates `b` inline and
    /// returns its value as-is (matching `primitive/boolean.rs`'s `bool_and`
    /// fallback exactly).
    fn compile_and(&mut self, receiver: Expr, rhs_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let rhs_fallback = rhs_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_jump(Bytecode::GuardBool, range);
        let to_short_circuit = self.emit_jump(Bytecode::JumpIfFalse, range);
        self.compile_inline_block_body(Self::expect_block(rhs_block))?;
        let to_end_1 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(to_short_circuit);
        self.emit(Bytecode::False, range);
        let to_end_2 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(guard);
        self.compile_expr(rhs_fallback)?;
        self.emit_sacred_send("and", &[None], range);
        self.patch_jump(to_end_1);
        self.patch_jump(to_end_2);
        Ok(())
    }

    /// `a.or { b }` — control-flow.md §2, the mirror of [`Self::compile_and`]:
    /// short-circuits to `true` without evaluating `b` when `a` is `true`.
    fn compile_or(&mut self, receiver: Expr, rhs_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let rhs_fallback = rhs_block.clone();
        self.compile_expr(receiver)?;
        let guard = self.emit_jump(Bytecode::GuardBool, range);
        let to_rhs = self.emit_jump(Bytecode::JumpIfFalse, range);
        self.emit(Bytecode::True, range);
        let to_end_1 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(to_rhs);
        self.compile_inline_block_body(Self::expect_block(rhs_block))?;
        let to_end_2 = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(guard);
        self.compile_expr(rhs_fallback)?;
        self.emit_sacred_send("or", &[None], range);
        self.patch_jump(to_end_1);
        self.patch_jump(to_end_2);
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
    fn compile_while_true(&mut self, cond_block: Expr, body_block: Expr, range: SourceRange) -> Result<(), CompilerError> {
        let cond_fallback = cond_block.clone();
        let body_fallback = body_block.clone();
        let guard = self.emit_jump(Bytecode::GuardBlock, range);
        let loop_start = self.functions.last().unwrap().chunk.code.len();
        self.compile_inline_block_body(Self::expect_block(cond_block))?;
        let to_exit = self.emit_jump(Bytecode::JumpIfFalse, range);
        self.compile_inline_block_body(Self::expect_block(body_block))?;
        self.emit(Bytecode::Pop, range); // discard the per-iteration body result
        self.emit_loop(loop_start, range);
        self.patch_jump(to_exit);
        self.emit(Bytecode::Nil, range);
        let to_end = self.emit_jump(Bytecode::Jump, range);
        self.patch_jump(guard);
        self.compile_expr(cond_fallback)?;
        self.compile_expr(body_fallback)?;
        self.emit_sacred_send("whileTrue", &[None], range);
        self.patch_jump(to_end);
        Ok(())
    }
}
