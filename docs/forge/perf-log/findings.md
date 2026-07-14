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
- **The reverted code is unrecoverable.** It was never staged, so it never entered
  the git object DB (`git fsck --lost-found` shows no matching dangling blob across
  6203 objects). Only the design above survives — ~1h to rebuild from it. Nothing is
  gained by hunting for the diff.

## F6 — U-GC's normative tables had two free-a-live-object bugs

Step 0 of U-GC regenerated `memory-management.md` §2.1 (roots) and §2.3 (outgoing
edges) against HEAD 2026-07-14. Both were written 2026-07-13 and had drifted. The
collector would have been built on them.

**Two missed edges, each of which frees a reachable object:**

- **`Object::Block` was absent from §2.3 entirely.** `BlockObject.closure: ObjRef` is
  a real edge — and a `Block` is the *only* retainer of its closure in a program that
  passes a `[…]` around. (Its `home_frame_token` is an index+generation, not a handle.)
- **`Upvalue::Open` carries a `fiber: ObjRef`.** §2.3 asserted an `Open` cell "aliases a
  live stack slot already traced as a root" — false on HEAD. The slot lives on *that*
  fiber's stack, which is the `VM` mirror **only while that fiber is current**;
  otherwise it is parked inside the `FiberObject`. A cross-fiber open upvalue was
  therefore untraced.

**Two claimed edges that do not exist** (harmless over-retention, but they signalled
the table was not written from the code): `ClassObject.name` is a Rust `String`, not a
heap object; `MethodObject` owns no chunk — its `MethodKind::Closure(ObjRef)` does, and
the constants live in `ClosureObject.callable.chunk`.

**Three missed roots** — `VM::sealed_classes: HashMap<Symbol, ObjRef>` (U-ANNOT-LAYOUT),
`VM::checking: HashSet<ObjRef>` (U-ANNOT-CONTRACTS), and `VM::ready_queue:
VecDeque<ObjRef>`. `sealed_classes` is the subtlest: it sits among four *genuine*
non-roots (`field_layouts`, `class_parents`, `constructor_aliases`,
`has_new_construct`) that hold only `Symbol`s, and reads like a peer of them — but its
values are handles.

**`ready_queue` is the important one, because the step-0 audit itself missed it.** It
holds fibers `System.schedule(_)` has enqueued but not yet resumed — reachable from
nowhere else, so an unrooted `ready_queue` frees a scheduled fiber. It was missed
because the audit grepped `^\s+pub [a-z_]+:` and `ready_queue` is `pub(crate)`. A
regex over field declarations is not an audit; it silently under-reports by exactly
the visibility modifier you forgot to type.

**The fix is structural, not another audit.** `VM::collect_roots`,
`Universe::each_handle` and `CoreClasses::each_handle` are now **exhaustive
destructures** — a new field fails to compile until classified as root or non-root.
The spec table documents the classification; the code enforces it. `ready_queue` was
caught the moment the destructure was written, before any test ran.

**Five edges that landed after the table was written**, all from the annotation work:
`Class.attributes`, `Method.attributes`, `Method.contracts`, `Module.attributes`,
`Fiber.checking`.

**Consequences:**
- §2.3 is now **field-level and exhaustive** over all 16 variants, and states what is
  *not* an edge — so the next drift is visible rather than silent.
- The exhaustive `match` (ADR-0050 §3) catches a new **variant** at compile time but
  **not a new field on an existing variant** — which is exactly how all five annotation
  edges got in.
- **The real lesson is that documenting the fix was not the fix.** Step 0 regenerated
  the table by hand and *still* missed `ready_queue`. What actually worked was making
  the compiler the enforcer: exhaustive destructures in `collect_roots` /
  `each_handle`, and a wildcard-free `match` in `trace_object`. Prefer a construct
  that fails the build over a table that asks the next reader to be careful.
- Generalises past GC: a spec table written from a design rather than from the code
  drifts silently under concurrent unit work. Cheap to re-derive, expensive to trust.

## F8 — the first object Phalcom ever reclaimed was leaked at bootstrap

Writing the step-2 tests surfaced this: a freshly bootstrapped `VM` is **not
garbage-free**. `core.ph`'s top-level `Closure` is unreachable the moment bootstrap
returns — the core `ModuleObject::closure` field is already `None`, and a full
reverse-scan over the arena (every live object traced, asking who points at it)
finds **zero holders**. It has been leaking since ADR-0009 deferred reclamation, and
it is the first object the collector reclaimed.

Harmless in itself (one closure per process). It matters for two reasons:

1. **Any test asserting an exact live count must baseline *after* a collection**
   (`settled_vm()` in `tests/gc.rs`), or it measures this closure rather than its own
   fixture. Four of the six step-2 tests failed on exactly this, off by one, before
   the baseline was fixed.
2. **The off-by-one was indistinguishable from a missed root** — the M3 failure mode.
   The only way to tell "correctly collected garbage" from "freed a live object" is
   to identify the object and prove nothing holds it. The reverse-scan probe that did
   so is worth rebuilding when step 4's temp-root audit hits the same ambiguity;
   `Heap::iter_handles_for_test`/`kind_of_for_test` exist for it.

## F7 — `size_of::<Object>()` grew to 280 B; Win A is six variants, not "the driver"

ADR-0050 measured 256 B. HEAD 2026-07-14 measures **280 B**
(`cargo +nightly rustc -p phalcom-core --lib -- -Zprint-type-sizes`), because
`ClassObject` gained `attributes: Vec<Value>` (+24 B) + `attributes_frozen` under
U-ANNOT. `ClassObject` alone *is* the 280 B, and the `SlotMap` sizes every slot to it —
so every 32 B string and 16 B tuple pays 280 B on the hot `heap.get` path.

Ladder: Class 280 · Fiber 176 · Module 168 · Closure 160 · Method 88 · Map/Set 72 ·
Range 40 · Str 32 · {Instance, Block, List, BoundMethod, Upvalue, Family} ≤24 · Tuple 16.

Two consequences for U-GC Win A:
- **Six variants must be boxed** (Class, Fiber, Module, Closure, Method, Map/Set) for
  `Range` (40 B) to cap the enum and the plan's pinned `<= 48` bound to hold. ADR-0050
  §9's list predates the measurement.
- **Do not box `Instance`.** At 24 B it is already under the floor, and it is the
  most-allocated variant — boxing it adds an indirection plus an allocation for zero
  size win. The obvious-looking move is the wrong one.

Post-boxing target: 280 → ~40 B, a **7×** arena density win, independent of the
collector and landable first.
