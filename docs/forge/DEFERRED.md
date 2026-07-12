# Deferred improvements register

Out-of-scope optimizations / DX / speed / security observations noticed while
landing a forge unit, but deliberately not implemented in that unit. Each
entry: file:line, category, one-line rationale.

## Open entries

| file:line | Category | Rationale |
|---|---|---|
| `phalcom-core/tests/lang/iteration/pending/` (not yet created) | test / cross-unit (U-FIBER) | U-ITER step 5 was cut: the PENDING generator fixtures `for_generator_suspends.ph` (C-ITER-8 — `Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }` suspends and yields `1,2,3`) and `each_generator_raises.ph` (`.each { Fiber.yield }` → `CannotYieldAcrossNativeFrame`) graduate with the **U-FIBER** landing. The `for` disasm golden (C-ITER-4) already proves the compile-time half (no `block_call` in the `for` chunk); these pin the runtime half. |
| `phalcom-core/core/core.ph` `class List` (`each`/`map`/`filter`/`reduce`/`includes`) | std-lib follow-on (U-STD) | **DEC-ITER-A** (resolved, user 2026-07-12): migrate the combinators off `size`/`at` onto the `iterate(_)`/`iteratorValue(_)` protocol as `.ph` defaults (ADR-0035 §5). Out of U-ITER scope — U-ITER's `core.ph` edit is limited to `List#iterate`/`iteratorValue`; both mechanisms are correct in parallel meanwhile. Owning unit: U-STD. |
| `phalcom-core/src/compiler/lib.rs` `compile_for` (loop-variable slot) | semantics / correctness | The loop variable is one reused local rebound each iteration via `SetLocal`, so a closure captured in the body over it observes the loop's **final** value, not the per-step value (spec §3.3 wants per-iteration freshness). Matches the existing inlined-`while` capture behavior; not exercised by C-ITER-1..7. Fix needs a fresh cell per iteration (a `CloseUpvalue`-per-step in the loop body). |
| `phalcom-core/src/compiler/inliner.rs` `compile_while_true` | feature parity (out of write-set) | `break`/`continue` bind only inside a `for` body: a `while` lowers via the inliner's `compile_while_true`, which pushes no `LoopContext`, so `break`/`continue` inside a bare `while` currently raise the out-of-loop compile error. Spec §3.2 wants `while`+`break`/`continue` too; realizing it needs `inliner.rs` (outside U-ITER's write-set) to push/pop a loop context around its jump loop. |
| `phalcom-core/src/compiler/lib.rs:~1043` (`patch_forward_jump_to`) vs `inliner.rs:167` (`emit_jump`) | dedup / DX | U-ITER re-implements the jump/patch/loop helpers (`emit_forward_jump`/`patch_forward_jump_to`/`emit_backward_loop`) because the inliner's equivalents are module-private and `inliner.rs` was outside the write-set. Once both are co-editable, hoist a shared jump-emission helper set onto `Compiler` and drop the duplicates. |

| `phalcom-core/src/compiler/lib.rs` `compile_break`/`compile_continue` (`func_depth` guard, ~L1246/L1272) | semantics / correctness (reviewer-found, adjudicated) | `break`/`continue` reached through a **materialized** block — the sacred inliner's deopt fallback for a non-Bool `if` condition, or an ordinary block-arg closure (`each { break }`) — compiles the jump into the closure's own chunk (`same_function` false), where it silently no-ops instead of leaving the loop. The common `if (Bool) { break }` path is unaffected: the inliner's fast path handles it and the dead deopt twin is never taken for real Bools. Pinned as PENDING (`iteration/pending/{break,continue}_across_materialized_block.ph`, intended output). **Adjudicated by user 2026-07-12:** ship U-ITER with the common case working + this documented, rather than (a) the reviewer's compile-error fix — infeasible, the deopt twin is compiled for *every* `if`-block so a compile-time reject regresses `if (Bool) { break }`; or (b) a full non-local-break implementation — out of scope. Real fix (follow-on): thread break/continue targets across `FunctionState` frames, **or** emit a runtime `Error.raise` trap (`primitive/error.rs`, `RuntimeError::Raise`) in the materialized twin so the rare case fails loudly instead of silently. |

## Homed entries

Every other deferral is homed in its owning unit's plan.
