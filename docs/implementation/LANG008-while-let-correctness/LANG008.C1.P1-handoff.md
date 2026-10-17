---
id: LANG008.C1.P1
category: LANG
program: LANG008
checkpoint: LANG008.C1
kind: handoff
status: SUPERSEDED (plan completed — see LANG008.C1.P1-walkthrough.md and the checkpoint verification state)
branch: feat/integration-universe
worktree: /Users/altunhasanli/dev/phalcom/phalcom (main checkout — NOT .worktrees/collections)
prepared: 2026-10-14, end of session
---

# LANG008.C1.P1 — handoff to fresh session (while let correctness)

Read `AGENTS.md`, `docs/workflow/implementation-record-lifecycle-convention.md`,
`docs/implementation/LANG008-while-let-correctness/LANG008.C1-CHECKPOINT.md`
(this checkpoint owns the full root-cause record), then this file.

## Where you are

Branch `feat/integration-universe`, main checkout. Uncommitted work in ~19
files. The tree **does not currently compile** — you are mid-refactor of
`phalcom-core/src/compiler/lib/patterns.rs`. Everything before the refactor
compiled (`cargo check --workspace` green) and the semantic-layer changes are
believed complete and stable.

## What is done (do not redo)

1. **AST + parser** (`phalcom-ast`): `IfLetExpr`/`WhileLetExpr` gained
   `pub annotation: Option<TypeAnnotation>`; `parse_if`/`parse_while` parse the
   optional `: Type` after the pattern (mirroring `parse_let`). Compiles clean.
2. **Semantic checker** (`phalcom-semantic/src/checker/expression.rs`): both
   `Expr::IfLet` and `Expr::WhileLet` now elaborate the optional annotation
   via `ctx.elaborate_type_annotation` and pass
   `ExpectedType::proper_from/inference_from(...)` into scrutinee
   `analyze_expression` (while-let computes it once before building
   `evaluate_step`, and the closure captures `&expected_scrutinee`). Compiles clean.
3. **source_index builder**: `IfLet`/`WhileLet` now use
   `declare_pattern_with_annotation(..., annotation.is_some())`. Occurrence
   visitor needs no change (it already ignored `Statement::Let` annotations).
4. **compile_while_let** (`phalcom-core/src/compiler/lib/expr.rs`):
   - `SetLocal(value_slot)` inside the loop is now followed by `Bytecode::Pop`
     (the original runtime bug).
   - `compile_if_let` and `compile_while_let` both call
     `setup_pattern_frame(&spec.bindings, &spec.pattern, ...)` (new 3rd arg).
   - while-let no longer uses per-iteration `temporary_base/Count` +
     `release_pack_scratch_from` + failure-landing logic — that scheme is
     superseded by pre-allocated frame temps (item 5). The if-let site at
     line ~2657 computes `pattern_scratch_count` AFTER `emit_executable_pattern`
     (order there is fine under the new scheme because nothing reserves during
     emission anymore).
5. **patterns.rs refactor — PARTIALLY APPLIED** (this is what you must finish):
   - `PatternExecutionFrame` has new field `temp_slots: Vec<u16>`.
   - New free fn `count_pattern_temp_depth(&ExecutablePattern) -> usize` (depth
     = max concurrent temps; Record/Map entries cost 2 — lookup temp + value
     temp; list-rest counter uses depth+1).
   - `setup_pattern_frame(bindings, pattern, range)` now pre-reserves
     `count_pattern_temp_depth(pattern)` slots named `$pattern_temp`.
   - `emit_executable_pattern` is now a thin wrapper delegating to new
     `emit_executable_pattern_at_depth(..., depth, ...)`; Variant/Or/Tuple/List/
     Record/Map arms rewritten to take temps from `frame.temp_slots.get(depth)`
     (falling back to `reserve_pack_scratch` only when a slot is missing) and
     recursing with `depth + 1` (Record/Map children at `depth + 2`).
   - **NOT yet done**: the old non-depth helper methods still exist unchanged
     (`claim_pattern_temp`, `emit_element_temp`, `emit_option_value_temp`,
     `emit_lookup_temp`, `emit_list_rest_temp`, `emit_record_rest_temp`), and
     the new `*_at_depth` variants referenced by the rewritten emitter were
     never written. Also `emit_executable_pattern_at_depth` still calls
     `self.emit_element_temp_at_depth / emit_list_rest_temp_at_depth /
     emit_lookup_temp_at_depth / emit_option_value_temp_at_depth`, which do not
     exist yet.

## Exact remaining work (mechanical, ~30–60 min)

1. `phalcom-core/src/compiler/lib/patterns.rs`:
   - Add `claim_pattern_temp_at_depth(&self, frame, depth, prefix, range)`:
     `slot = frame.temp_slots.get(depth).copied().unwrap_or_else(|| reserve_pack_scratch(prefix, range))`,
     then emit `SetLocal(slot)` + `Pop` (i.e. old `claim_pattern_temp` body).
   - Add `emit_element_temp_at_depth(frame, depth, value_slot, index, range)` =
     `emit_element_read(...)` + `claim_pattern_temp_at_depth(frame, depth, "$pattern_element", ...)`.
   - Add `emit_option_value_temp_at_depth(frame, depth, value_slot, range)` =
     old `emit_option_value_temp` body but claiming via `*_at_depth`
     (`"$pattern_option_value"`).
   - Add `emit_lookup_temp_at_depth(frame, depth, value_slot, key, range)` =
     old `emit_lookup_temp` body but claiming via `*_at_depth`
     (`"$pattern_lookup"`).
   - Add `emit_list_rest_temp_at_depth(frame, depth, value_slot, fixed_count, range)`:
     copy the old `emit_list_rest_temp` body exactly (List-new, rest_slot =
     claim at `depth` with `"$pattern_rest"`, inner counter claim at `depth + 1`
     with `"$pattern_rest_index"`, the size/append/`+` loop,
     `emit_backward_loop`, patch done, `end_scope`), only changing which claim
     helper is used. Keep `emit_operator_send("append", ...)` as in the current
     file (an earlier draft wrote "add" — the file says "append"; trust the file).
   - In `count_pattern_temp_depth`, fix the one type error: `Record` and `Map`
     have *different* entry types (`(Box<str>, ExecutablePattern)` vs
     `(MapPatternKey, ExecutablePattern)`), so they cannot share one arm.
     Split into two arms with the same `2 + depth` costing.
   - Optionally rename or delete the now-unused old helpers
     (`claim_pattern_temp`, `emit_element_temp`, `emit_option_value_temp`,
     `emit_lookup_temp`, `emit_list_rest_temp`) — BUT note
     `emit_record_rest_temp`, `emit_lookup_temp`, `emit_option_value_temp`,
     `claim_pattern_temp` are still used by the legacy *irrefutable* binding
     lowering lower in the same file (lines ~620–701: `$destructure_variant`,
     record-rest, map/record destructuring) and by `match_expr.rs` line ~207
     (`$variant_child` reserve). Do NOT delete those; only remove helpers that
     end up with zero callers after the rewrite (check each with grep).
2. Update the two remaining `setup_pattern_frame` callers to the new 3-arg form
   (pass the executable pattern):
   - `phalcom-core/src/compiler/lib/loops.rs:235` —
     `self.setup_pattern_frame(&spec.bindings, &lane_specs/lane spec .pattern, lane.pattern.range())`
     (inside `compile_canonical_for_lanes`; the per-lane spec's
     `.pattern` field — the lane's executable pattern is `lane_specs[number].pattern`,
     available in that loop via the zip with `lane_specs`).
   - `phalcom-core/src/compiler/lib/match_expr.rs:49` and `:110` —
     pass `&arm_spec.pattern`.
   - `loops.rs` also has its own `temporary_base/temporary_count` +
     `release_pack_scratch_from` scheme around its per-lane
     `emit_executable_pattern` (lines ~278–314). Under the new scheme
     `emit_executable_pattern` reserves nothing, so `temporary_count` will
     compute 0 and the existing code already handles 0 (guards
     `if temporary_count > 0` on emit; `release_pack_scratch_from` with 0 is
     also a no-op, but verify the debug_assert in
     `release_pack_scratch_from` tolerates count 0 — it should, since
     first_slot + 0 == num_locals only if first_slot == num_locals; check and
     if it trips, guard the call). Leave the surrounding logic intact otherwise.
3. `compile_while_let` in `expr.rs` — verify final shape matches:
   pre-loop `Reserve` of value slot + Nil seed (already there), frame setup with
   pattern (done), loop body = compile value → `SetLocal` → `Pop` →
   `emit_executable_pattern` (temps come from frame; nothing reserved inside)
   → `commit_pattern_frame` → body → CloseUpvalue for captured bindings →
   backward loop; exit = failure/break target; after exit, release
   `pattern_scratch_count` slots from `frame.scope_start` (already emitted) and
   the value slot, then `end_scope`, `Nil`. Remove any leftover
   `temporary_base/temporary_count` locals if they still exist (an earlier edit
   added them; the frame-temp scheme replaces them — grep and clean).
4. Build: `RUSTFLAGS='' cargo check --workspace` → fix remaining mechanical
   errors (expect only the ones above: 3×E0061 callers, 1×E0308 Record/Map,
   7×E0599 missing `*_at_depth` methods, possibly unused-fn warnings for
   retired helpers — delete only truly-dead ones).

## Acceptance gates (run in this order)

1. `RUSTFLAGS='' cargo test -p phalcom-core --test core language::golden::fixture_control_flow_if_let_while_let`
   — new focused golden test; expected stdout is in the fixture's `.expected`
   sidecar (includes sections: some/none, tuple/record, counter `while let`,
   nested tuple destructure `while let`, annotated `while let : Option<Int>`).
   Run the fixture manually with `target/debug/phalcom
   phalcom-core/tests/fixtures/language/control-flow/control_flow_if_let_while_let.ph`
   if it fails — output must match the sidecar exactly.
2. `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus control_flow`
   (scans all 14 control-flow fixtures; budget ~2–4 min; bootstrap is slow in
   debug, ~90 s per fresh canonical Universe compile).
3. `RUSTFLAGS='' cargo test -p phalcom-core --test cli-smoke`
   (exercises `successful_file_run...`; previously failed on
   `interval.ph` inference conflict and on
   `Chain.==`/`Chain.drop` `NoTypeEvidence` — both believed fixed by the
   annotation + `rawAt` changes already in the tree; this is the gate that
   tells you whether any `match` in the canonical Universe is still blocked).
4. `RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic adts::behavior`
   (quick semantic smoke).
5. If gates 1–4 pass: the checkpoint's remaining item is dogfooding — confirm
   `folds.ph` `while let` forms execute (they are compiled as part of the
   canonical Universe in every gate above, so gates passing already exercises
   them) and optionally modernize `packages/universe` twins to match
   `core/universe` (both are already annotated/`while let`-modernized in this
   tree — just diff-check the twins match).
6. Then update `LANG008.C1-CHECKPOINT.md` plan ledger (P1 → COMPLETE /
   IMPLEMENTED + FOCUSED_TESTED), and write P1 walkthrough.

## Known gotchas

- Fixture uses `Helper.makeNone` class-method indirection for the `None` case:
  a direct `let mut optNone: Option<Int> = Option::None` makes the checker
  refine the scrutinee to `Option::None` exactly, and the `if let Some` arm
  correctly reports `MatchPatternImpossible`. That is intended behavior, not a
  bug — keep the helper indirection in the fixture.
- `Array.at` returns union `T | None`; `.unwrap` does NOT exist on that union
  (errors: "union receiver arm `None` does not support selector `unwrap()`",
  and remember zero-arg method syntax is `.unwrap`, no parens). Size-checked
  accesses should use `.rawAt(i)` — that is what `interval.ph` now does.
- Unqualified `Some(...)`/`None` in patterns fail with "variant `Some` has no
  declaration-backed owner" in this fixture context; qualified
  `Option::Some(...)` is the reliable form here.
- Long-running gates: corpus/cli-smoke can take 90–140 s each in debug; the
  binary run of the fixture is also slow (~45 s canonical Universe compile).
  Use generous timeouts (≥180 s) or you'll see spurious tool timeouts.
- `emit_backward_loop(loop_start, range)` (not `emit_backward_jump`) is the
  correct call for the loop backedge — match the existing file.
- Scratch slot mechanics (for review, no action needed): `ReserveScratchLocal`
  inserts Nil at `stack_offset+slot`; `ReleaseScratchLocal` removes at the same
  index; `SetLocal` copies without popping. This is why the frame-temp
  pre-allocation exists: keeping reserve/release outside the loop keeps stack
  indices stable across iterations.
- Do NOT weaken `ensure_executable_match_coverage` (canonical Universe
  fail-closed gate). If a Universe `match` still blocks, fix the Universe source
  with annotations or exact accessors (see gotchas) — the gate is correct.

## Deliverables when done

- Green gates 1–4 above (record in walkthrough with exact commands + results).
- Scoped diff containing: ast/parser annotation support, semantic expected-type
  propagation, source_index annotation-aware pattern declaration,
  patterns.rs frame-temp scheme, expr.rs Pop fix + frame shape, loops.rs and
  match_expr.rs caller updates, fixture + sidecar + golden test, Universe
  source annotation/`rawAt` fixes.
- Updated CHECKPOINT.md ledger, P1 walkthrough + this handoff refreshed.
- Do not commit unless the user asks; the tree is intentionally uncommitted
  mid-work.
