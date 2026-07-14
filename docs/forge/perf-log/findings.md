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

## F13 — bootstrap went 5 ms → 180 ms: the `ifTrue` inliner is exponential in nest depth

Every Phalcom process pays a **fixed +175 ms** at `debadfa` → `3b2dd97`
(U-STRING's core rework). Bootstrap (`VM::new`, which recompiles `core.ph`)
measured on `System.print(1)`, whole-process:

| | user |
|---|---|
| `debadfa` | **0.00 s** (~5 ms) |
| `3b2dd97` (clean) | **0.18 s** |

This is what looked, at first, like a uniform ~20% throughput regression across
`bare_send` (+22%), `arith_send` (+23%) and `for` (+28%). It is not a throughput
regression at all — it is a **constant**, and it fooled a percentage-based reading
because the benchmarks run for ~0.9 s. `bare_send` touches no strings, which is what
gave it away.

**Mechanism — a compiler bug, not a string bug.** `core.ph`'s new `codePointAt(i)`
is a 32-`ifTrue` nest (UTF-8 continuation-byte decode), ~14 levels deep. Compiling
that **one method** costs ~200 ms (0.42 s vs the 0.215 s bootstrap baseline). Compile
time doubles per nesting level with **source length held linear**:

| nest depth | 8 | 10 | 12 | 14 | 16 | 18 | 20 |
|---|---|---|---|---|---|---|---|
| compile (over baseline) | ~0 | 0.04 s | ~0 | 0.03 s | 0.17 s | 0.70 s | **2.8 s** |

(First attempt at this table put the nested body in *both* arms, making the *source*
exponential — it "confirmed" the hypothesis and proved nothing. The table above nests
only the `ifFalse` arm, which is the shape `core.ph` actually has.)

The `ifTrue`/`ifFalse` inliner (`compiler/inliner.rs`) evidently duplicates its
continuation into each arm, so nested conditionals multiply rather than add: 2^depth
compiled code from depth-linear source.

**Consequences:**
- **A 20-line source method can hang the compiler.** Depth 20 is 2.8 s; depth ~26 is
  minutes. This is reachable by ordinary user code, not just `core.ph` — and
  [the two-armed `ifTrue(_, ifFalse:_)` surface is locked by decision](../../adr/), so
  deep nesting is the *idiomatic* way to write a multi-way conditional today. The
  language's own conditional form is a compiler-blowup hazard.
- **This is the cheapest large win available.** Bootstrap is on every process,
  including every one of the ~200 golden tests and every criterion iteration (each
  bench iteration runs `Interpreter::new`). Fixing the inliner returns ~175 ms per
  process and makes Tier 5 (bootstrap) largely moot at current `core.ph` size.
- **It landed unnoticed because nothing measures bootstrap.** `run.sh`'s gate asks
  "did it run", the criterion benches amortize bootstrap inside a 0.9 s program, and
  the wren-suite table is single-run. A 35× bootstrap regression passed all three.
  **A bootstrap-only tripwire (`System.print(1)`, whole-process) belongs in the
  harness** — it is one line and would have caught this on the commit.

## F12 — module globals are a SipHash probe per access; the IC does not cover them

Now that U-IC (`f5e41f1`) caches method lookup, **the top non-loop cost on
send-heavy code is not dispatch — it is reading a variable.** `Bytecode::GetGlobal`
/ `SetGlobal` (`vm/dispatch.rs:494-536`) resolve the name through
`ModuleObject.name_to_slot`, a `HashMap<Symbol, usize>` with the **default SipHash**
hasher, on **every access**. `sample` leaf ticks at `1e1b101`:

| workload | `hash_one` + `sip::Hasher::write` | share |
|---|---|---|
| `bare_send` (40M sends) | 235 + 82 = **317** | **~13%** |
| `for` (4M) | 131 + 37 = **168** | **~7%** |

`for`'s inner loop (`while (i < N) { list.add(i); i = i + 1 }`) reads `i` twice,
reads `list`, writes `i` — **four SipHash probes per iteration**, next to an IC'd
send. Wren, the comparison target, resolves module variables to slot indices at
compile time and pays zero.

**Prototype measured** (per-callsite `(module, slot)` cache in `Chunk.gcaches`,
parallel to the existing `caches` IC vector; ~40 lines, all 46 lang tests green,
every wren-suite output still matching):

| | bare_send | arith_send | for | method_call |
|---|---|---|---|---|
| Δ user | **−15.3%** | **−14.2%** | **−5.0%** | **−3.6%** |

Understated: both binaries carry F13's +175 ms bootstrap constant, which dilutes
every percentage. Net of it, `bare_send` is ≈ **−18%**.

**Consequences:**
- **This is the next send-path unit, ahead of anything else on the ranking.** It is
  a bigger, cheaper win than what U-IC's remaining polish offers, and it is
  orthogonal to it.
- **The real fix is probably compile-time slot resolution, not a cache.** The
  prototype caches because slots are append-only per module (`declare` only pushes),
  so a resolved `(module, slot)` stays valid. That invariant is exactly what would
  let the *compiler* emit `GetGlobalSlot(u16)` directly. The open question is
  late-binding: a `GetGlobal` that resolves to `core` today can be shadowed by a
  later `define` in the main module, and the prototype does **not** handle that (it
  is a measurement, not a candidate to land as-is). Whether that shadowing is
  reachable, and whether it should be legal, is a language question — flag for
  decision, do not silently pick one.
- **`Value::class` (92 ticks, ~4%) and `call_method` (353, ~14%) are the next two**
  on the same profile; `call_method`'s share is the arg-buffer + frame setup that
  DEC-PRIM-B's arithmetic fast path targets.

## F11 — skynet is now GC-bound, and its collections free nothing

Post-U-GC, post-cut-004, the skynet profile is unrecognizable from F1's. `sample`
leaf ticks at `1e1b101`, mid-run:

| mechanism | ticks | share |
|---|---|---|
| interpreter loop | 259 | 35% |
| **`trace_object` (GC mark)** | **145** | **~20%** |
| malloc/free family | ~130 | ~17% |
| `call_method` | 48 | 6% |
| slotmap slot drop + `Heap::collect` | 53 | 7% |

**The GC family is ~30% of skynet, and nearly all of it is wasted work.** Skynet's
~1.1M fibers are *all live* until the very end — every collection traces the entire
live set and frees almost nothing, then `next_gc` grows by only
`GC_GROW_FACTOR = 1.5` (`heap/mod.rs:99-100`), so the next cycle re-traces ~1.5×
as much, again for nothing.

**Measured fix — back off harder when a cycle is unproductive** (~15 lines: if a
collection reclaims <10% of the heap, grow the threshold 4× instead of 1.5×):

| | user | peak RSS |
|---|---|---|
| skynet, `1e1b101` | 2.36 s | 1.577 GB |
| skynet, adaptive | **2.13 s (−10%)** | **1.534 GB (−3%)** |
| fiber_churn (real garbage — the regression check) | 0.51 s → **0.46 s (−10%)** | 450 → **420 MB** |

Both workloads improve and RSS improves with them, which is the surprise: the
naive fear (a laxer threshold trades memory for time) does not materialize, because
the collections being skipped were freeing nothing anyway.

**Consequences:**
- **A yield-adaptive threshold is a cheap, real unit** — 15 lines for −10% on the
  two most allocation-heavy workloads in the repo, no correctness surface beyond the
  trigger policy. Precedent is standard (V8/CPython both scale the next threshold by
  reclamation yield).
- **`GC_GROW_FACTOR = 1.5` with an all-live heap is the pathological case** the
  constant was never chosen for: cost scales with the *live* set while the benefit
  scales with the *garbage*, and skynet has none until it ends.
- Do not read this as the whole fiber story: 1.5 GB for 1.1M fibers is ~1.4 KB each
  against Wren's ~0.6 KB. Object density (F7's ladder — `Fiber` is 176 B before its
  buffers) is a separate, unexercised lever.

## F10 — the fiber pool is not neutral, it is negative — and F5 measured the wrong workload

F5 rebuilt-and-reverted the fiber-stack pool as a **null result** and left the door
open: "would only pay off under **high fiber turnover** … revisit only with such a
benchmark." The code was later rebuilt behind the off-by-default `fiber-pool`
Cargo feature (`phalcom-core/Cargo.toml`, U-GC step 5), with the flag's own comment
recording it as "measured net negative … kept so the experiment can be re-run."

**It has now been re-run against the workload F5 asked for** — and the flag's
comment is right for a reason nobody has written down: it is not a wash.

`benchmarks/vm/fiber_churn.ph` (new — spawn → run to `Done` → drop, in a loop) is
the high-turnover probe F5 said was missing. Skynet spawns 1.1M fibers and lets
them *live*; it hits the pool's only recycle site (the `Done` path in
`vm/dispatch.rs`) once per fiber with no reuse pressure. `fiber_churn` hits it every
iteration with a pool that is never empty — the pool's best case.

Same-machine A/B, release, `nopool` vs `--features fiber-pool`, 3 reps:

| fibers | user (nopool → pool) | peak RSS (nopool → pool) | ΔRSS |
|---|---|---|---|
| 100k | 0.06 → 0.06 s | 52.7 MB → 98.0 MB | **+86%** |
| 500k | 0.31 → 0.31 s | 309 MB → 539 MB | **+74%** |
| 1M | 0.62 → **0.85 s (+37%)** | 635 MB → 1090 MB | **+72%** |

Skynet, by contrast, is a wash on every axis (user 2.19 vs 2.20–2.31 s, RSS 1.437
vs 1.437 GB) — which is why F5, measuring Skynet, saw nothing.

**The RSS cost is linear in fibers created: ~450 B per fiber, dead on.** (45 MB /
100k, 229 MB / 500k, 455 MB / 1M.) That is not pool bookkeeping — the pool is
bounded at 100 entries and can cost at most a few hundred KB. A per-fiber cost from
a fixed-size pool means **recycled capacity is being retained per fiber, not
reused**: a buffer drawn from the pool carries a previous fiber's grown capacity
into a shell that outlives its own run (fiber shells are only freed when the
collector sweeps them, and the arena never shrinks). The pool converts a
`Vec::new()` (capacity 0) into a capacity-carrying buffer on an object that leaks
until the next collection. At 1M fibers that also costs time (+37% user, sys 0.09 →
0.23 s) — the RSS is paid for in page faults.

**This mechanism is a hypothesis; the numbers are not.** Confirming it means
instrumenting where a pooled buffer's capacity ends up, and that work is only worth
doing if someone wants to revive the flag.

**Consequences:**
- **F5's "null result" is too generous, and its stated revival condition is
  refuted.** The pool is not "neutral, might pay off under turnover" — under
  turnover it is *worse*, on the exact workload F5 nominated as its proving ground.
  A revival needs a new mechanism (recycle capacity, don't hand it to a shell that
  outlives its run), not a new benchmark.
- **Ruled: the flag stays, stays off, and is not to be used** (owner's call,
  2026-07-14). It is not a live option — do not enable it, do not benchmark against
  it, do not cite it as an available tuning knob. Reviving it needs a **new
  mechanism** (recycle capacity without handing it to a shell that outlives its run),
  not a re-run: the experiment has now been run twice, with a worse answer each time.
  The recommendation on the table was to delete the six `#[cfg]` sites across
  `vm/mod.rs`, `vm/bootstrap.rs`, `vm/gc.rs`, `vm/dispatch.rs`, `primitive/fiber.rs`
  and `heap/fiber.rs` (including the duplicate `FiberObject::new_entry_with_buffers`
  constructor and the `collect_roots` destructure arm); that was declined in favour
  of keeping the experiment reconstructable in-tree.
- **It carries a live GC hazard while it exists.** `fiber_pool` is classified a
  **non-root** in `collect_roots` (`fiber_pool: _`) — correct only because the single
  push site in `dispatch.rs` calls `.clear()` on both buffers first. A future second
  recycle site that forgets the `clear()` hands the collector a pool full of
  reachable-but-untraced `Value`s: F6's free-a-live-object failure mode, in a feature
  that is off, untested in CI, and therefore will not fail anything until someone
  turns it on. An off-by-default flag is not free; it is unmeasured surface that the
  root-classification argument still has to hold for.
- **Generalises F5's own lesson one step further.** F5 concluded "no current
  benchmark exercises high turnover." The follow-through was to *write* one
  (~10 lines), not to leave the question open behind a flag. A candidate parked
  behind a feature gate looks decided and is not: this one sat as "measured net
  negative, re-run later" while the cheap experiment that would settle it stayed
  unwritten.

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

## F9 — the "tracing span" cost was real, but the mechanism was wrong twice

_Numbered F8 while in flight; `1ef999b` and `3d6d45f`'s commit messages say "F8" and mean
this finding. Renumbered on landing — F8 was already taken by the bootstrap-leak finding
(`bbc12d6`), which was absent from this README's index and so invisible to a grep._

Cut [003](003-vm-trace-feature-gate.md) closed F1's Tier 1 candidate. F1's *size*
estimate held (18.3% predicted, 18.2% measured on the A/B), but both natural
explanations of *why* were falsified — and each would have produced a worse fix.

**Refuted #1 — "it's a subscriber misconfiguration."** `bin/phalcom/main.rs:15` reads
`registry().with(layer.with_filter(LevelFilter::OFF))`. The tidy story: the registry's
default `register_callsite` returns `Interest::always()`, the filtered layer returns
`never`, so the callsite resolves to **`sometimes`** and every opcode pays a dynamic
`enabled()` check. Tested first, because it promised a one-line fix touching no VM
code. **Measured: −0.4%.** Interest already resolves to `never` in both placements.
`main.rs` was reverted, untouched.

**Refuted #2 — "it's the span."** F1, `performance.md` §2, and the README's own
ranking all name the *span* ("Tier 1 — tracing span"). It is half the cost:

| Removed | Δ |
|---|---|
| span only | −8.4% |
| the three `debug!`s only | −8.4% |
| both | −18.2% |

Even and additive — generic per-callsite overhead, not a span-guard optimization
barrier. **A unit dispatched from the README's framing would have gated the span,
booked −8.4%, and closed the candidate** with half the win left on the floor and a
false mechanism recorded as fact.

**The transferable point:** a correct *attribution* (which line is hot) does not carry
a correct *mechanism* (why), and only the mechanism tells you the shape of the fix.
The profiler named the span because the span is where the samples landed; the
`debug!`s two lines away cost the same and were invisible to the framing. Cheap to
check — build one variant per suspect — and the check is what sized the fix.
Corollary, already in the README's method: **no runtime filter can remove code.**
`LevelFilter::OFF` was set the whole time and bought nothing; only `#[cfg]` did.
