use crate::bytecode::Bytecode;
use phalcom_common::range::SourceRange;

use super::Compiler;

impl<'vm> Compiler<'vm> {
    /// Returns the current end of the function-under-compilation's chunk (the
    /// index the next emitted opcode will occupy) — a jump/loop label.
    pub(super) fn chunk_len(&self) -> usize {
        self.functions.last().unwrap().chunk.code.len()
    }

    /// Emits a placeholder single-`i32`-operand jump/guard instruction built
    /// by `make` — [`Bytecode::Jump`], [`Bytecode::JumpIfFalse`],
    /// [`Bytecode::GuardBool`], [`Bytecode::GuardBlock`] are all usable
    /// directly as an `fn(i32) -> Bytecode` tuple-variant constructor —
    /// returning its chunk index for a later [`Self::patch_forward_jump`]/
    /// [`Self::patch_forward_jump_to`].
    ///
    /// Shared by [`super::loops`]'s `compile_for` direct jump skeleton and every
    /// sacred-inliner guarded fast path (U-ITER-FIX item 4: `lib.rs` and
    /// `compiler/inliner.rs` used to each keep an independent copy of this
    /// helper set, since neither module's private items are visible to the
    /// other — now that a single unit co-edits both, they share one copy).
    pub(crate) fn emit_forward_jump(&mut self, make: fn(i32) -> Bytecode, range: SourceRange) -> usize {
        self.emit(make(0), range);
        self.chunk_len() - 1
    }

    /// Backpatches the forward jump/guard placeholder at chunk index `idx` so
    /// its offset lands exactly at the **current** end of the chunk — the
    /// common case, used whenever the jump/guard's target is simply "wherever
    /// compilation naturally continues to next" (every sacred-inliner guard/
    /// jump, and a `for`/`while` loop's condition-false exit).
    pub(crate) fn patch_forward_jump(&mut self, idx: usize) {
        let target = self.chunk_len();
        self.patch_forward_jump_to(idx, target);
    }

    /// Backpatches the forward jump/guard placeholder at chunk index `idx` so
    /// it lands at the explicit absolute chunk index `target` (the
    /// [`Bytecode::Jump`] offset convention: relative to `ip` already
    /// advanced past the jump). Needed whenever the target is **not** simply
    /// "here" — unlike [`Self::patch_forward_jump`]'s implicit current-chunk-
    /// end target — such as a `continue`'s cursor-step label or `while`'s
    /// condition-retest label, both earlier in the chunk than `idx`.
    ///
    /// # Panics
    ///
    /// Panics if `idx` is not one of the four offset-carrying jump/guard
    /// opcodes ([`Bytecode::Jump`], [`Bytecode::JumpIfFalse`],
    /// [`Bytecode::GuardBool`], [`Bytecode::GuardBlock`]) — a
    /// compiler-internal invariant, never user-reachable.
    pub(crate) fn patch_forward_jump_to(&mut self, idx: usize, target: usize) {
        let offset = target as i32 - (idx as i32 + 1);
        match &mut self.functions.last_mut().unwrap().chunk.code[idx] {
            Bytecode::Jump(o) | Bytecode::JumpIfFalse(o) | Bytecode::JumpIfNone(o) | Bytecode::GuardBool(o) | Bytecode::GuardBlock(o) => *o = offset,
            other => unreachable!("patch_forward_jump_to on a non-jump opcode: {other:?}"),
        }
    }

    /// Emits a backward [`Bytecode::Loop`] to the absolute chunk index
    /// `loop_start`, closing one iteration of a `for` cursor loop or an
    /// inlined `whileTrue` (U-ITER-FIX item 4 — shared with the inliner for
    /// the same reason as [`Self::emit_forward_jump`]).
    pub(crate) fn emit_backward_loop(&mut self, loop_start: usize, range: SourceRange) {
        let idx = self.chunk_len() as i32;
        self.emit(Bytecode::Loop(loop_start as i32 - (idx + 1)), range);
    }
}
