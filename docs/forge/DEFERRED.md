# Deferred improvements register

Out-of-scope optimizations / DX / speed / security observations noticed while
landing a forge unit, but deliberately not implemented in that unit. Each
entry: file:line, category, one-line rationale.

## Open entries

| file:line | Category | Rationale |
|---|---|---|
| `docs/spec/v0.2/object-model.md` §5:210-211; `docs/spec/.../implementation-status.md` | docs-drift | Note claims "every metaclass's superclass wired to `Class`, breaking it" — stale pre-U2; native tower now satisfies ADR-0002 rule 4 (tested) and U-INH extends the same rule to user classes. Re-point both. |
| `phalcom-core/src/bytecode.rs` (`SuperSend`) | perf / IC follow-on | `SuperSend` is uncached (DEC-INH-F). Wire the inline-cache seam **with U15/U16** so a `superclass=` (U15) / override-epoch bump (ADR-0018) invalidates a cached `SuperSend` the same way it invalidates `Invoke`. |
| `docs/forge/units/README.md`, phase INDEX | docs roster | Add the `U-INH` roster row (landed). Not edited in-unit — shared-file concurrent-session hazard. |
| `phalcom-core/src/compiler/lib.rs:~1081` (`has_new_construct` guard) | correctness | Guard is keyed on the receiver class name only, **not** inheritance-aware. A subclass that *inherits* a `new`-constructor but declares none is absent from `has_new_construct`, so a wrong-arity `Sub.new(...)` (e.g. `B.new()` when the only ancestor ctor is `new(t)`) silently falls through to the `Object.class::new` bare allocator and returns an **uninitialized** instance instead of the "No constructor matches" error the declaring class raises. Unique to `new` (named ctors have no bare-allocator fallback → they dNU, safe). Matching-arity inherited ctors already resolve correctly via `value.rs:128` `lookup_method`'s `init `-prefix metaclass walk — this is *only* the guard gap. Fix: walk the superclass chain in both the guard **and** the `constructor_aliases` lookup; needs a compile-time name→parent map (populate at ClassDef superclass resolution, `lib.rs:~764`). Owning unit: U13 (hierarchy policy). |
| `phalcom-core/tests/lang/iteration/pending/` (not yet created) | test / cross-unit (U-FIBER) | U-ITER step 5 was cut: the PENDING generator fixtures `for_generator_suspends.ph` (C-ITER-8 — `Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }` suspends and yields `1,2,3`) and `each_generator_raises.ph` (`.each { Fiber.yield }` → `CannotYieldAcrossNativeFrame`) graduate with the **U-FIBER** landing. The `for` disasm golden (C-ITER-4) already proves the compile-time half (no `block_call` in the `for` chunk); these pin the runtime half. |
| `phalcom-core/core/core.ph` `class List` (`each`/`map`/`filter`/`reduce`/`includes`) | std-lib follow-on (U-STD) | **DEC-ITER-A** (resolved, user 2026-07-12): migrate the combinators off `size`/`at` onto the `iterate(_)`/`iteratorValue(_)` protocol as `.ph` defaults (ADR-0035 §5). Out of U-ITER scope — U-ITER's `core.ph` edit is limited to `List#iterate`/`iteratorValue`; both mechanisms are correct in parallel meanwhile. Owning unit: U-STD. |
| `phalcom-core/src/compiler/lib.rs` `compile_for` (loop-variable slot) | semantics / correctness | The loop variable is one reused local rebound each iteration via `SetLocal`, so a closure captured in the body over it observes the loop's **final** value, not the per-step value (spec §3.3 wants per-iteration freshness). Matches the existing inlined-`while` capture behavior; not exercised by C-ITER-1..7. Fix needs a fresh cell per iteration (a `CloseUpvalue`-per-step in the loop body). |
| `phalcom-core/src/compiler/inliner.rs` `compile_while_true` | feature parity (out of write-set) | `break`/`continue` bind only inside a `for` body: a `while` lowers via the inliner's `compile_while_true`, which pushes no `LoopContext`, so `break`/`continue` inside a bare `while` currently raise the out-of-loop compile error. Spec §3.2 wants `while`+`break`/`continue` too; realizing it needs `inliner.rs` (outside U-ITER's write-set) to push/pop a loop context around its jump loop. |
| `phalcom-core/src/compiler/lib.rs:~1043` (`patch_forward_jump_to`) vs `inliner.rs:167` (`emit_jump`) | dedup / DX | U-ITER re-implements the jump/patch/loop helpers (`emit_forward_jump`/`patch_forward_jump_to`/`emit_backward_loop`) because the inliner's equivalents are module-private and `inliner.rs` was outside the write-set. Once both are co-editable, hoist a shared jump-emission helper set onto `Compiler` and drop the duplicates. |
| `phalcom-core/src/compiler/lib.rs` `compile_break`/`compile_continue` (`func_depth` guard, ~L1246/L1272) | semantics / correctness (reviewer-found, adjudicated) | `break`/`continue` reached through a **materialized** block — the sacred inliner's deopt fallback for a non-Bool `if` condition, or an ordinary block-arg closure (`each { break }`) — compiles the jump into the closure's own chunk (`same_function` false), where it silently no-ops instead of leaving the loop. The common `if (Bool) { break }` path is unaffected: the inliner's fast path handles it and the dead deopt twin is never taken for real Bools. Pinned as PENDING (`iteration/pending/{break,continue}_across_materialized_block.ph`, intended output). **Adjudicated by user 2026-07-12:** ship U-ITER with the common case working + this documented, rather than (a) the reviewer's compile-error fix — infeasible, the deopt twin is compiled for *every* `if`-block so a compile-time reject regresses `if (Bool) { break }`; or (b) a full non-local-break implementation — out of scope. Real fix (follow-on): thread break/continue targets across `FunctionState` frames, **or** emit a runtime `Error.raise` trap (`primitive/error.rs`, `RuntimeError::Raise`) in the materialized twin so the rare case fails loudly instead of silently. |

## Homed entries

Every other deferral has been homed in its owning unit's plan — each carries an
**Adopted debt** note in its write-set section:

| Debt | Owning unit |
|---|---|
| `primitive/number.rs:~34` — type-error message hardcodes `"value"` | [U12](units/U12/plan.md) §3 |
| `primitive/nil.rs:~64` — broken rustdoc link → private `wrap_some` | [U-ERR](units/U-ERR/plan.md) §3 |
| `core/README.md` — stale floor baseline (80/64, should track 88) | [U-ERR](units/U-ERR/plan.md) §3 |

Add a new entry here **only** when a debt has no plausible owning unit; otherwise
fold it into the relevant `units/<U>/plan.md` write-set as an **Adopted debt** note.
