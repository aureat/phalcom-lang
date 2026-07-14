# Profiling findings

Measured facts that reshape the performance plan. Each is grounded in code +
U-BENCH numbers, not hypothesis.

## F1 — Measured baseline supersedes the oral 29×

U-BENCH Tier 0 (`benchmarks/vm/BASELINE.md`) measured Skynet at **~19–20× Wren
wall-clock, ~7–9× RSS** (Phalcom 13.7–15.6 s / 4.65–6.09 GB vs local `wren_test`
0.68–0.79 s / ~667 MB). The oral "~29×" (ADR-0051 context) is revised **down** to a
measured ~19–20×.

**Attribution re-ranks [performance.md §2](../../spec/v0.2/performance.md).**
malloc/free is the single largest attributable mechanism on **both** workloads
(arith 19.7%, Skynet 28.2%) — larger than the tracing span (18.3%) and larger than
dispatch lookup (13.9% arith / 7.8% Skynet). Confirms ADR-0051's rejection of a
dispatch-first ordering: **allocation is the top lever, not the inline cache.**
→ drove cut [001](001-prim-abi-inline-args.md).

## F2 — `Option`-escape optimization: premise falsified

A pasted senior review proposed escape-analysis / scalar-replacement of `Some` as
a killer win ("`map.at(k)` returns `Some(v)` on every lookup"). **Not true in this
codebase:**

- `List#at`/`Map#at` return the raw value on hit and the preallocated `None`
  singleton on miss — **zero allocation** (`primitive/list.rs`, `primitive/map.rs`;
  `None` singleton at `universe/core_classes.rs`).
- The compiler already **elides** the `Some` wrap when the result is discarded
  (`want_value` gate, `compiler/inliner.rs`).
- The *only* live `Some` allocation is `WrapSome` from a one-armed
  `ifTrue`/`ifFalse` whose value is consumed, plus explicit `Some.new(_)`.

Remaining opportunity (a transient `Some` in `cond.ifTrue { X }.ifSome { … }`
chains) is too narrow to justify a unit. **No unit filed.**

## F3 — memmove (20.6% Skynet) is `Vec` growth, not `mem::take`

The attribution flagged `memmove` = 20.6% of Skynet leaf ticks, hypothesized as
fiber `mem::take` churn. **Mechanism corrected:** `mem::take(&mut Vec<T>)` swaps the
3-word `(ptr,len,cap)` header — O(1), copies no elements
(`primitive/fiber.rs:30-32,37,51-54` are all O(1) swaps).

The real source is **per-fiber `Vec` growth-reallocation**: every fiber starts
`stack: Vec::new()` / `frames: Vec::new()` (capacity 0, `heap/fiber.rs`), and each
push past current capacity as the fiber runs triggers a `memmove` of live elements.
There is **zero fiber-buffer pooling today** — dead fibers' buffers are cleared
(length→0) but never freed or reused (`vm/dispatch.rs` clears on `Failed`; `Heap`
has no dealloc path). Over Skynet's ~1M fibers that is ~2M+ fresh allocations plus
millions of small memmoves.

**Fix = fiber-stack pool** — already named in [U-GC plan §3.7 + DEC-GC-C](../units/U-GC/plan.md)
as "Win B", explicitly independent of the mark-sweep collector. → extract as
**U-GC-POOL** (a free-list of `Vec<Value>`/`Vec<CallFrame>` handed out at fiber
creation, returned on `Finished`/`Failed`). Next measured lever after 001.

## F4 — U-IC preconditions (for when Tier 3 comes)

- `Symbol(u32)` (`interner.rs`) is a **single mixed namespace** (vars/fields/
  selectors); no `SelectorId` type exists in source. A selector-only interner is
  U-IC's build-order step 1, not a separate pre-unit.
- The IC seam is a **comment only** today (`vm/dispatch.rs`, "IC → exact-probe …");
  `lookup_method_in_hierarchy` (`heap/class.rs`) is an unconditional `IndexMap`
  hash-probe walked per superclass level. `ClassObject` has **no epoch/version
  field** yet, and there is no global `world_version` — U-IC introduces the first
  epoch primitive.
- The existing override epoch (`bool_sacred_pristine`/`block_sacred_pristine`,
  ADR-0018) is a coarse global one-shot bit. Per [PLAN-DECORATORS](../PLAN-DECORATORS.md),
  the IC guard must read that bit **alongside** the `(class_id, SelectorId)`
  compare. Mutation-site enumeration for epoch bumps (esp. `superclass=`) is still
  open.

## F5 — fiber-stack pool: implemented, measured, reverted (null result)

The [F3](#f3--memmove-206-skynet-is-vec-growth-not-memtake) memmove finding
pointed at a fiber-stack pool (U-GC "Win B"). It was **built and measured**, then
**reverted** — it shows no reliable win.

Implementation (correct, behavior-invariant, all fiber/concurrency tests green): a
bounded free-list of `Vec<Value>`/`Vec<CallFrame>` on the VM; a spawned fiber
(`new_fiber_ref`) takes recycled capacity-retained buffers; a fiber reaching
`FiberStatus::Done` returns its buffers before the resumer's `load_live_from` drops
them. Only the `Done` path recycles (park/`yield` keeps its buffers; the rare
`Failed` cascade is left as-is).

Same-machine A/B on Skynet (release; cleanest run-3-WITH vs two WITHOUT, each right
after a rebuild):

| | wall | peak RSS |
|---|------|----------|
| without pool | 15.23 s, 15.29 s | 5.66 GB, 5.95 GB |
| with pool | 15.48 s | 5.78 GB |

Indistinguishable. `fiber_spawn` criterion likewise flat (p = 0.65). **Why:**

1. **Skynet RSS is dominated by the ~1M immortal `FiberObject` shells** in the
   heap slotmap (never freed — no GC), not the stack/frames buffers. Pooling
   buffers cannot move that; only the real collector (**U-GC**) reclaiming dead
   fiber objects will. That is the actual Tier-4 RSS lever.
2. The memmove was 20.6% of *CPU ticks*, but removing it does not move wall-clock
   out of run-to-run noise on this workload.

**Consequences:**
- Per measure-first (P2/P3), an unproven optimization does not land in the
  contention-prone fiber cascade. Reverted.
- **Redirects U-GC:** the Skynet memory win is *freeing fiber shells* (the
  collector), not buffer pooling. Do not split "Win B" out ahead of the collector —
  it does not stand alone on the evidence.
- A fiber-stack pool would only pay off under **high fiber turnover** (rapid
  spawn→Done→respawn). No current benchmark exercises that; revisit only with such
  a benchmark on a quiet machine.
