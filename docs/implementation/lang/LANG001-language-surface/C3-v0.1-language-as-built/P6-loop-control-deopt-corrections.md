# U-ITER-FIX — Work order: U-ITER loop-control follow-ons (deopt break/continue · `while` break/continue · loop-var freshness · jump-helper dedup)

_Self-contained plan for **one** implementer. **Reviewer ON** — it edits the compiler spine
(`compiler/lib.rs`, `compiler/inliner.rs`), which is load-bearing for every loop. Green gate:
`./scripts/verify.sh` exits 0 + `cargo doc --workspace --no-deps` clean. In-tree on `main`, NO
worktree (STATE stash hazard). Grounds in **spec §3.2/§3.3** (loop control), the U-ITER as-built,
and the `DEFERRED.md` entries this unit closes. All four items are already registered in
`DEFERRED.md`; this plan gives the homeless ones (1/2/3/6) an owning unit._

## Write-set (serialize with U-ERR — shared `compiler/lib.rs`)
`phalcom-core/src/compiler/lib.rs`, `phalcom-core/src/compiler/inliner.rs`,
`phalcom-core/tests/lang/iteration/` (+ graduate the two PENDING fixtures), `tests/lang/MANIFEST.md`.
**Collision:** contends `compiler/lib.rs` with **U-ERR** → never co-schedule; one compiler-spine
writer at a time. Disjoint from U-COLL (`phalcom-ast/parser.rs`) and U-COLLTYPES (`heap.rs`/`core.ph`).

## Items (each an independently-green slice)

### 1. Deopt-block break/continue silent no-op (the fresh find, correctness — highest priority)
`break`/`continue` reached through a **materialized** block — the sacred inliner's deopt fallback for
a non-Bool `if` condition, or an ordinary block-arg closure (`each { break }`) — compiles the jump
into the closure's own chunk (`same_function` false in `compile_break`/`compile_continue`,
`lib.rs:~L1246/L1272`), where it silently no-ops instead of leaving the loop. The common
`if (Bool) { break }` path is unaffected (inliner fast path; dead deopt twin never taken for real
Bools). **Adjudicated (user 2026-07-12): ship U-ITER with the common case + this documented.**
Real fix, pick one:
- **(a) runtime trap (smaller, recommended):** emit an `Error.raise`/`RuntimeError::Raise` in the
  materialized twin so the rare cross-block case fails **loudly** instead of silently no-op'ing.
  Depends on U-CORE-6 unwind (**landed**). Graduates
  `iteration/pending/{break,continue}_across_materialized_block.ph` to their intended (now: error) output.
- **(b) full non-local break (larger):** thread break/continue targets across `FunctionState`
  frames so the jump escapes the closure to the real loop. Correct per spec but a deeper change.
Recommend (a) for this unit; leave (b) as a documented future option.

### 2. `while` + break/continue (feature parity, spec §3.2)
Bare `while` lowers via inliner `compile_while_true`, which pushes **no `LoopContext`**, so
`break`/`continue` inside a `while` body raises the out-of-loop compile error. Fix: push/pop a
`LoopContext` around the inliner's jump loop (mirrors `for`'s context), so break/continue bind inside
`while` too. New PASS goldens `iteration/while_break.ph` / `while_continue.ph`.

### 3. Loop-variable capture freshness (semantics, spec §3.3)
`compile_for` reuses one local rebound each iteration via `SetLocal`, so a closure captured over the
loop var observes the loop's **final** value, not the per-step value. Matches the inlined-`while`
capture behavior; not exercised by C-ITER-1..7. Fix: a fresh cell per iteration
(`CloseUpvalue`-per-step in the loop body). Golden: closure-in-loop collects `[0,1,2]` not `[3,3,3]`.

### 4. Jump-helper dedup (DX, do last)
U-ITER re-implemented `emit_forward_jump`/`patch_forward_jump_to`/`emit_backward_loop`
(`lib.rs:~L1043`) because the inliner's equivalents (`inliner.rs:167`) are module-private. Once both
are co-editable (this unit edits both), hoist a shared jump-emission helper set onto `Compiler` and
drop the duplicates. Pure refactor — no behavior change; green gate is the proof.

## Not in this unit
- **Item 4 combinator migration** (List `each`/`map`/`filter`/`reduce`/`includes` off `size`/`at`
  onto `iterate`/`iteratorValue`, DEC-ITER-A resolved) → **U-STD** (edits `core.ph`, different spine).
- **Item 5 generator fixtures** (`for_generator_suspends.ph` C-ITER-8, `each_generator_raises.ph`) →
  **U-FIBER test follow-on**: these fixtures were never created; author them + verify against landed
  U-FIBER (tests-only, `tests/lang/{iteration,concurrency}/`). Parallel-safe with any src writer.

## Build order
Slice 1 (trap) → 2 (`while`) → 3 (freshness) → 4 (dedup, last). Each its own green commit;
`graphify update . --no-cluster` before each. Stage only explicit paths (`git status --short`,
never `git add -A`).
