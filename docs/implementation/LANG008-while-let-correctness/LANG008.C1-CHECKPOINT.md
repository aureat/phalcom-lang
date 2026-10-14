---
id: LANG008.C1
program: LANG008
checkpoint: LANG008.C1
kind: checkpoint-record
status: IN_PROGRESS
branch: feat/integration-universe
created: 2026-10-14
---

# LANG008.C1 — `while let` runtime correctness, scrutinee inference, and fixture coverage

## Objective

Make `while let PAT = expr { ... }` (and the parallel `if let` form) compile and
execute correctly for all pattern shapes, including inside generic Universe code,
so idiomatic `while let Option::Some(v) = iterator.next` replaces the
`let mut step: T = ...; while step.isSome { match step ... }` workaround across
`functional/folds.ph` and friends.

## Root causes (established, evidence in P1 session)

1. **VM operand-stack leak** — `compile_while_let`
   (`phalcom-core/src/compiler/lib/expr.rs`) emitted `SetLocal(value_slot)`
   without a following `Bytecode::Pop` inside the loop. `SetLocal` copies the
   stack top without popping, so every iteration leaked one scrutinee onto the
   VM stack. Consequences: `ReleaseScratchLocal` (`stack.remove(idx)`) removed
   wrong-position entries, method receivers (`stack.len() - 1 - arity`) and
   `Return` (`stack.pop()`) read loop garbage; observed as corruption/hangs.
   `compile_if_let` and `compile_match_expr` always paired `SetLocal` + `Pop`;
   only `compile_while_let` omitted it.
2. **Nested-pattern scratch churn inside the loop** — `emit_executable_pattern`
   reserved scratch locals (`$variant_child`, `$pattern_element`, ...) *inside*
   the loop body via `reserve_pack_scratch`, and released them per iteration.
   Failure jumps could bypass reserves, so releases removed slots reserved in a
   previous pass; per-iteration `Reserve/ReleaseScratchLocal` also shifts open
   upvalue indices. Fix in flight: pre-allocate all temp slots in
   `setup_pattern_frame` from a static `count_pattern_temp_depth(pattern)` so
   the loop body is slot-count-neutral (matches how `compile_canonical_for_lanes`
   keeps reserves outside its loop).
3. **Scrutinee analyzed with `ExpectedType::None`** — for
   `iterator.next` typed through associated projections
   (`Option<<I as Iterable>::Item>`), synthesis-mode inference left the
   projection underconstrained; the fail-closed canonical-Universe gate
   (`canonical_universe.rs::ensure_executable_match_coverage`) then rejected any
   *`match`* in the same body with
   `blocked coverage proof: UnknownType(UnderconstrainedTypeVariable)`.
   Note: `while let` pattern sites themselves are NOT subject to that gate
   (it inspects `match_resolutions` only) — the workaround existed because the
   rewritten `match step` form *was* gated. Grammar also offered no way to
   annotate the scrutinee.
4. **Fixture blind spot** — `control_flow_while_let.ph` contains a plain
   `while (i < 3)` loop (misnamed); `control_flow_if_let_while_let.ph` exercises
   the real forms and is picked up by corpus directory scan
   (`corpus::control_flow` → `check_pass("control-flow")`), so MANIFEST row
   counts never reflected it. A focused golden test is being added.

## Durable invariants (established or being established)

- `SetLocal` must always be paired with `Pop` (or an equivalent stack consume)
  at every emission site that stores a transient value.
- Loop bodies must not change the function's local-slot count or the operand
  stack depth at the backedge; scratch reservations for pattern matching belong
  to `setup_pattern_frame` (outside the loop), released at loop exit.
- `if let` / `while let` must elaborate an optional `: Type` annotation and pass
  it as `ExpectedType` to scrutinee synthesis (mirrors `Statement::Let`).
- Canonical Universe remains fail-closed on unproven `match` coverage; this
  checkpoint does not weaken it.

## Plan ledger

| Plan | Status | Notes |
|---|---|---|
| LANG008.C1.P1 | IMPLEMENTED + FOCUSED_TESTED | walkthrough/handoff in `LANG008.C1.P1-walkthrough.md` / `LANG008.C1.P1-handoff.md` |

## Related fixes riding along (same working tree, this branch)

- `interval.ph` `subtract`: `rems.at(0)` → `rems.rawAt(0)` — `Array.at` returns
  union `T | None` which triggered a generic-inference conflict at variant
  payloads; size is pre-checked so `rawAt` is exact.
- `functional/{chain,folds,effects,journal}.ph` (both `core/universe` and
  `packages/universe` twins): explicit accumulator annotations
  (`let mut result: Chain<T> = Chain<T>::Empty` etc.) to unblock coverage
  proofs; `folds.ph` already uses idiomatic `while let` on this branch.
- Fixture `control_flow_if_let_while_let.ph` rewritten to qualified
  `Option::Some/None` + typed `next -> Option<Int>` methods + new annotated
  `while let ... : Option<Int>` section; sidecar `.expected` extended;
  focused golden test added in `tests/core/language/golden.rs`.

## Verification state

- `RUSTFLAGS='' cargo check --workspace --all-targets`: PASS (clean, no warnings).
- `fixture_control_flow_if_let_while_let` golden test: PASS (~155s debug) —
  the primary acceptance discriminator; the pre-fix 120s hang is gone.
- `language-corpus corpus::control_flow_while_break_continue`: PASS.
- `language-corpus corpus::control_flow`: FAIL on `control_flow_for.ph`
  ("missing required canonical for-iteration lowering spec") — **BASELINE_BLOCKED,
  not caused by this patch**: at clean HEAD the same suite dies earlier
  (Universe bootstrap failure in `functional.chain`); this patch fixes the
  bootstrap crash and unmasks the latent for-lane defect. Diagnosis of the
  for-lane issue continues separately (suspects: commit `1404e08f` moved
  Iterator/Iterable protocols to `collections/iteration` in the
  `packages/universe` twin only; semantic lane resolver hardcodes
  `collections.iterator_laws`; instrumentation confirmed the
  `collections.iterator_laws.Iterable` trait surface is absent from the
  registry despite the embedded twin still having the traits).

## Deferred / out of scope

- For-lane publication defect (see Verification state): pre-existing, masked
  by the bootstrap crash; tracked as follow-up diagnosis, out of LANG008.C1
  acceptance.
- Collections worktree (`.worktrees/collections`, branch `feat/telos-collections`)
  carries an earlier parallel version of these fixes plus two fixes not yet on
  this branch: `expose .capabilities ...` lines in both
  `numbers/package.ph` files (unblocks `universe.numbers.capabilities` import)
  and the `is_legacy_symbol_key_domain` union-domain acceptance in
  `phalcom-semantic/src/metadata/export.rs` (unblocks
  `NonExportableRecordKeyDomain`). Port if the corresponding failures appear
  here; they were observed in that worktree, not yet on this branch.
- MANIFEST.md control-flow row count refresh (3 → 14 files actually scanned) —
  cosmetic, optional.
