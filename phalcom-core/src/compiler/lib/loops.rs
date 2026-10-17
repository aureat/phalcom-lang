use crate::bytecode::Bytecode;
use crate::modules::semantic_lowering::LoweringSiteKind;
use crate::value::Value;
use phalcom_ast::ast::{ForStatement, Pattern};
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

    /// Declares a loop local named `name` and returns its slot. `mutable`
    /// records whether user code may reassign it: synthetic loop temporaries
    /// pass `true`, the loop variable passes `false` (it behaves as
    /// a per-iteration `let`, iteration.md §2) — the compiler still rebinds it
    /// each step through a direct [`Bytecode::SetLocal`], which bypasses the
    /// user-facing immutability check.
    pub(super) fn declare_loop_local(&mut self, name: &str, mutable: bool) -> Result<usize, CompilerError> {
        let sym = self.vm.interner.intern(name);
        self.add_local(sym, mutable)?;
        let slot = self.functions.last().unwrap().num_locals - 1;
        self.emit(Bytecode::ReserveScratchLocal(slot as u16), SourceRange::default());
        Ok(slot)
    }

    /// Lowers a `for` statement from preselected canonical semantic lane
    /// products. Missing canonical lowering specifications are rejected.
    pub(super) fn compile_for(&mut self, for_stmt: ForStatement) -> Result<(), CompilerError> {
        let canonical_specs = for_stmt
            .lanes
            .iter()
            .map(|lane| {
                self.lowering()
                    .and_then(|lowering| lowering.for_iteration_lanes.get(&lane.iter.range()))
                    .cloned()
            })
            .collect::<Option<Vec<_>>>();
        let Some(specs) = canonical_specs else {
            return Err(CompilerError::Message("missing required canonical for-iteration lowering spec".into()));
        };
        let generalized_single = for_stmt.lanes.len() == 1 && (for_stmt.lanes[0].index.is_some() || !matches!(for_stmt.lanes[0].pattern, Pattern::Name { .. }));
        if for_stmt.lanes.len() > 1 || generalized_single {
            return self.compile_canonical_for_lanes(for_stmt, specs);
        }
        self.compile_canonical_for(for_stmt, specs[0].clone())
    }

    /// Lowers a proven canonical `Iterable` lane to a stateful iterator loop.
    /// The source is evaluated once, `iterator` and `next` are each selected by
    /// semantic lowering, and the returned `Option` is retained long enough for
    /// the direct `None` branch and `Some` payload extraction.
    fn compile_canonical_for(
        &mut self,
        for_stmt: ForStatement,
        spec: crate::modules::semantic_lowering::ForIterationLoweringSpec,
    ) -> Result<(), CompilerError> {
        let range = for_stmt.range;
        let phalcom_ast::ast::ForStatement { lanes, body, range: _ } = for_stmt;
        let lane = lanes
            .into_iter()
            .next()
            .ok_or_else(|| CompilerError::Message("for requires at least one lane".into()))?;
        let Pattern::Name { name, .. } = lane.pattern else {
            return Err(CompilerError::Message("canonical single-lane lowering requires a name pattern".into()));
        };

        self.begin_scope();
        self.compile_expr(lane.iter)?;
        let source_slot = self.declare_loop_local("$for_source", true)?;
        self.emit(Bytecode::SetLocal(source_slot as u16), range);
        self.emit(Bytecode::Pop, range);

        self.emit(Bytecode::GetLocal(source_slot as u16), range);
        self.emit_for_iteration_requirement(&spec.iterator, range)?;
        let iterator_slot = self.declare_loop_local("$for_iterator", true)?;
        self.emit(Bytecode::SetLocal(iterator_slot as u16), range);
        self.emit(Bytecode::Pop, range);

        self.emit(Bytecode::Nil, range);
        let binding_slot = self.declare_loop_local(&name, false)?;
        self.emit(Bytecode::Pop, range);

        self.push_loop_context();
        let loop_start = self.chunk_len();
        self.emit(Bytecode::GetLocal(iterator_slot as u16), range);
        self.emit_for_iteration_requirement(&spec.next, range)?;
        self.emit(Bytecode::Dup, range);
        let exit_on_none = self.emit_forward_jump(Bytecode::JumpIfNone, range);
        self.emit(Bytecode::GetVariantPayload(0), range);
        self.emit(Bytecode::SetLocal(binding_slot as u16), range);
        self.emit(Bytecode::Pop, range);

        self.begin_scope();
        for statement in body {
            self.compile_statement_with_pop_control(statement, true)?;
        }
        self.end_scope(range);

        let step_label = self.chunk_len();
        if self.functions.last().unwrap().locals[binding_slot].is_captured {
            self.emit(Bytecode::CloseUpvalue(binding_slot as u16), range);
        }
        self.emit_backward_loop(loop_start, range);

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

    /// Canonical multi-lane lowering. All sources are evaluated first, then
    /// every iterator is advanced exactly once before exhaustion is inspected.
    fn compile_canonical_for_lanes(
        &mut self,
        for_stmt: ForStatement,
        specs: Vec<crate::modules::semantic_lowering::ForIterationLoweringSpec>,
    ) -> Result<(), CompilerError> {
        let phalcom_ast::ast::ForStatement { lanes, body, range } = for_stmt;
        if lanes.is_empty() || lanes.len() != specs.len() {
            return Err(CompilerError::Message("for requires at least one canonical lane".into()));
        }
        self.begin_scope();

        let mut iterators = Vec::with_capacity(lanes.len());
        for (number, lane) in lanes.iter().enumerate() {
            self.compile_expr(lane.iter.clone())?;
            let source = self.declare_loop_local(&format!("$for_source_{number}"), true)? as u16;
            self.emit(Bytecode::SetLocal(source), lane.range);
            self.emit(Bytecode::Pop, lane.range);
            iterators.push(source);
        }
        for (number, lane) in lanes.iter().enumerate() {
            self.emit(Bytecode::GetLocal(iterators[number]), lane.range);
            self.emit_for_iteration_requirement(&specs[number].iterator, lane.range)?;
            let iterator = self.declare_loop_local(&format!("$for_iterator_{number}"), true)? as u16;
            self.emit(Bytecode::SetLocal(iterator), lane.range);
            self.emit(Bytecode::Pop, lane.range);
            iterators[number] = iterator;
        }

        let ordinal = self.declare_loop_local("$for_ordinal", true)? as u16;
        let zero = self.add_constant(Value::int(0), range)?;
        self.emit(Bytecode::Constant(zero), range);
        self.emit(Bytecode::SetLocal(ordinal), range);
        self.emit(Bytecode::Pop, range);
        let mut indices = Vec::with_capacity(lanes.len());
        for lane in &lanes {
            indices.push(if let Some(index) = &lane.index {
                let slot = self.declare_loop_local(&index.name, true)? as u16;
                self.emit(Bytecode::Nil, index.range);
                self.emit(Bytecode::SetLocal(slot), index.range);
                self.emit(Bytecode::Pop, index.range);
                Some(slot)
            } else {
                None
            });
        }
        let mut lane_specs = Vec::with_capacity(lanes.len());
        for lane in &lanes {
            let spec = self.pattern_site_spec(LoweringSiteKind::ForLanePattern, lane.pattern.range())?;
            lane_specs.push(spec);
        }

        let mut results = Vec::with_capacity(lanes.len());
        let mut values = Vec::with_capacity(lanes.len());
        for (number, lane) in lanes.iter().enumerate() {
            results.push(self.declare_loop_local(&format!("$for_result_{number}"), true)? as u16);
            values.push(self.reserve_pack_scratch(&format!("$for_value_{number}"), lane.range)?);
        }

        let mut lane_frames = Vec::with_capacity(lanes.len());
        for (lane, spec) in lanes.iter().zip(lane_specs.iter()) {
            let frame = self.setup_pattern_frame(&spec.bindings, &spec.pattern, lane.pattern.range())?;
            lane_frames.push(frame);
        }

        self.push_loop_context();
        let loop_start = self.chunk_len();
        for (number, lane) in lanes.iter().enumerate() {
            self.emit(Bytecode::GetLocal(iterators[number]), lane.range);
            self.emit_for_iteration_requirement(&specs[number].next, lane.range)?;
            self.emit(Bytecode::SetLocal(results[number]), lane.range);
            self.emit(Bytecode::Pop, lane.range);
        }

        self.emit(Bytecode::GetLocal(results[0]), range);
        let first_none = self.emit_forward_jump(Bytecode::JumpIfNone, range);
        let mut later_none = Vec::new();
        for result in results.iter().skip(1) {
            self.emit(Bytecode::GetLocal(*result), range);
            later_none.push(self.emit_forward_jump(Bytecode::JumpIfNone, range));
        }
        let all_some = self.emit_forward_jump(Bytecode::Jump, range);
        let first_none_label = self.chunk_len();
        self.patch_forward_jump_to(first_none, first_none_label);
        let mut mismatch = Vec::new();
        for result in results.iter().skip(1) {
            self.emit(Bytecode::GetLocal(*result), range);
            let none = self.emit_forward_jump(Bytecode::JumpIfNone, range);
            let live = self.emit_forward_jump(Bytecode::Jump, range);
            self.patch_forward_jump_to(none, self.chunk_len());
            mismatch.push(live);
        }
        let all_none_skip_error = self.emit_forward_jump(Bytecode::Jump, range);
        let error = self.chunk_len();
        for jump in mismatch {
            self.patch_forward_jump_to(jump, error);
        }
        self.emit_strict_zip_error(range)?;
        let all_some_label = self.chunk_len();
        self.patch_forward_jump_to(all_some, all_some_label);
        for jump in later_none {
            self.patch_forward_jump_to(jump, error);
        }

        let temporary_base = self.functions.last().unwrap().num_locals;
        let mut pattern_failures = Vec::new();
        for (number, lane) in lanes.iter().enumerate() {
            self.emit(Bytecode::GetLocal(results[number]), lane.range);
            self.emit(Bytecode::GetVariantPayload(0), lane.range);
            self.emit(Bytecode::SetLocal(values[number]), lane.range);
            self.emit(Bytecode::Pop, lane.range);

            self.emit_executable_pattern(
                &lane_specs[number].pattern,
                values[number],
                &lane_frames[number],
                &mut pattern_failures,
                lane.pattern.range(),
            )?;
        }
        let after_pattern = self.emit_forward_jump(Bytecode::Jump, range);
        let pattern_failure = self.chunk_len();
        for jump in pattern_failures {
            self.patch_forward_jump_to(jump, pattern_failure);
        }
        let temporary_count = self.functions.last().unwrap().num_locals - temporary_base;
        // This branch is emitted before the success path; keep compiler-local
        // bookkeeping intact until the success-path cleanup below.
        self.emit_release_scratch_range(temporary_base as u16, temporary_count, range);
        self.emit_pattern_mismatch_raise("for pattern did not match iterable value".into(), range)?;
        self.patch_forward_jump_to(after_pattern, self.chunk_len());

        for (number, lane) in lanes.iter().enumerate() {
            self.commit_pattern_frame(&lane_frames[number], lane.pattern.range())?;
            if let Some(index) = indices[number] {
                self.emit(Bytecode::GetLocal(ordinal), lane.range);
                self.emit(Bytecode::SetLocal(index), lane.range);
                self.emit(Bytecode::Pop, lane.range);
            }
        }
        self.release_pack_scratch_from(temporary_base as u16, temporary_count, range);

        self.begin_scope();
        for statement in body {
            self.compile_statement_with_pop_control(statement, true)?;
        }
        self.end_scope(range);
        let step = self.chunk_len();
        for spec in &lane_specs {
            for b in spec.bindings.iter() {
                let symbol = self.vm.interner.intern(&b.name);
                if let Some(slot) = self.resolve_local(symbol)
                    && self.functions.last().unwrap().locals[slot].is_captured
                {
                    self.emit(Bytecode::CloseUpvalue(slot as u16), range);
                }
            }
        }
        self.emit(Bytecode::GetLocal(ordinal), range);
        let one = self.add_constant(Value::int(1), range)?;
        self.emit(Bytecode::Constant(one), range);
        self.emit_operator_send("+", 1, range)?;
        self.emit(Bytecode::SetLocal(ordinal), range);
        self.emit(Bytecode::Pop, range);
        self.emit_backward_loop(loop_start, range);

        let loop_exit = self.chunk_len();
        self.patch_forward_jump_to(all_none_skip_error, loop_exit);
        let (breaks, continues) = self.pop_loop_context();
        for jump in breaks {
            self.patch_forward_jump_to(jump, loop_exit);
        }
        for jump in continues {
            self.patch_forward_jump_to(jump, step);
        }
        self.end_scope(range);
        Ok(())
    }

    fn emit_strict_zip_error(&mut self, range: SourceRange) -> Result<(), CompilerError> {
        let error_sym = self.vm.interner.intern("ZipLengthMismatchError");
        self.emit_global_reference(error_sym, range)?;
        let message = self.vm.alloc_string_value("strict zip lanes ended at different times".to_string());
        let message_idx = self.add_constant(message, range)?;
        self.emit(Bytecode::Constant(message_idx), range);
        self.emit_operator_send("new", 1, range)?;
        self.emit_operator_send("raise", 0, range)?;
        self.emit(Bytecode::Pop, range);
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
    fn emit_deopt_block_control_trap(&mut self, range: SourceRange) -> Result<(), CompilerError> {
        let error_sym = self.vm.interner.intern("Error");
        self.emit_global_reference(error_sym, range)?;
        let message = self.vm.alloc_string_value(
            "`break`/`continue` reached through a materialized block (a deopt fallback or a real \
             block-arg closure) cannot leave its enclosing loop — non-local break/continue across a \
             materialized block is not supported (U-REOPEN-FIX; see docs/forge/DEFERRED.md)."
                .to_string(),
        );
        let message_idx = self.add_constant(message, range)?;
        self.emit(Bytecode::Constant(message_idx), range);
        self.emit_operator_send("new", 1, range)?;
        self.emit_operator_send("raise", 0, range)?;
        // `raise` never returns normally (it unwinds the stack), so this `Pop`
        // is unreachable dead code — kept only so the chunk's static stack
        // shape stays balanced like any other statement.
        self.emit(Bytecode::Pop, range);
        Ok(())
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
            self.emit_deopt_block_control_trap(range)?;
        }
        Ok(())
    }

    /// Lowers a `continue` statement (ADR-0035 §3, iteration.md §3, U-ITER
    /// specification §4): an unconditional forward [`Bytecode::Jump`] recorded
    /// on the innermost [`LoopContext`], backpatched to the next-step label
    /// by [`Self::compile_for`] — or, when
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
            self.emit_deopt_block_control_trap(range)?;
        }
        Ok(())
    }
}
