# 004 — U-HOTPATH: share block-literal `Callable` via `Rc` (Tier 2)

Status: **landed** · Unit: [U-HOTPATH](../units/U-HOTPATH/implementation-spec.md) (Tier 2) · Spec: [performance.md §4 Tier 2](../../spec/v0.2/performance.md), [ADR-0051](../../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md) · Behavior-invariant (no ADR, no floor change)

Two of the unit's four changes landed (`1531070`, `debadfa`). **The unit's win is
Change 4 alone**; Change 2 is a wash and Change 3 was dropped before landing. This
entry separates them, because the commit pair reads as one optimization and only
one of the two is a measured win.

## The cost (Change 4)

`Bytecode::Closure` materializes a block literal by cloning its `ClosureObject` —
which owned `Callable` **by value**, and `Callable` owns the block's `Chunk`
(code, constants, line table). Every evaluation of a `{ … }` literal therefore
deep-copied the entire compiled body: three heap allocations plus the copy, on a
path that runs once per block *evaluation*, not once per block *in the source*.

Skynet evaluates one block literal per fiber — ~1.1M deep chunk copies for a
body that is identical every time.

## The cut

`ClosureObject.callable: Rc<Callable>`; the two compiler construction sites wrap
in `Rc::new`, and `dispatch.rs`'s `Bytecode::Closure` arm does `Rc::clone` — a
refcount bump instead of a chunk copy. The GC tracer needed no change (`Rc` derefs
transparently).

## Measured

Whole-process, release, alternating A/B/C across three binaries built from
`bd3f492` (base, pre-unit), `1531070` (Change 4), `debadfa` (HEAD, + Change 2);
3 reps each; `user` time is the estimator per the README's method note (Skynet's
`real` cannot resolve single-digit effects).

| Benchmark | base | +Change 4 | +Change 2 (HEAD) | Δ Change 4 |
|---|---|---|---|---|
| **Skynet** (1.1M fibers, 1 block literal each) | 3.11s user / 3.73 GB / 3.28s sys | **2.19s / 1.37 GB / 0.19s sys** | 2.19s / 1.44 GB | **−30% user, −63% RSS, −94% sys** |
| **fiber_churn** (500k spawn→Done→respawn) | 0.36s user | **0.30s** | 0.30s | **−16%** |
| `binary_trees` | 0.80s user | 0.86s | 0.84s | **+7% (regression)** |
| `bare_send` (5M sends) | 0.85s user | 0.90s | 0.89s | **+5% (regression)** |
| `arith_send` (5M sends) | 0.76s user | 0.81s | 0.80s | **+6% (regression)** |

Skynet's `real` fell 8.0s → 2.5s, but the honest number is `user` (−30%); most of
the `real` collapse is the 3.28s → 0.19s `sys` drop, i.e. the page-fault cost of
allocating 1.1M chunk copies. **The RSS win is the headline**: 3.73 GB → 1.37 GB,
and it lands in the one place the perf log had concluded only the GC collector
could help ([F5](findings.md#f5--fiber-stack-pool-implemented-measured-reverted-null-result)).

## The regression is real and is Change 1's job

The +5–7% on send-heavy programs is consistent in sign across every pair, on three
independent benchmarks: `Rc<Callable>` adds a pointer hop to the chunk read the
dispatch loop performs **per instruction**. Change 4 trades a per-instruction hop
for a per-block-evaluation copy — an excellent trade wherever blocks are
evaluated, a small loss in a loop that evaluates none.

[Change 1 (hoist the chunk pointer out of the dispatch loop)](../units/U-HOTPATH/implementation-spec.md)
is what pays this back, and the unit deferred it as "optional — only if it
measures". **That call should be reversed**: Change 1 is not an independent
candidate, it is Change 4's other half. Change 4's `Rc` is precisely the
foundation that makes the hoist clean (the chunk outlives the frame by refcount).

## Change 2 (memoize derived selectors) — a **−28% win**, on a path no benchmark had

`VM::variadic_selector_cache` replaces the `Invoke` variadic probe's
`decode_selector` + `format!` + re-intern with a `HashMap<Symbol, Option<Symbol>>`
lookup.

**This entry first recorded Change 2 as "a wash, not a win" and called for its
revert. That was wrong, and the error was in the harness, not the change.** The
variadic probe sits behind *two* misses (`dispatch.rs`: IC miss → exact-selector
miss → variadic probe). Every benchmark in this repo dispatches through the IC or
the exact probe, so **not one of them executed the cached line even once**. The
measurement said "no change" because it never ran the code.

`benchmarks/vm/variadic_send.ph` (new — 2M sends to a `sum(*args)` variadic, the
only shape that reaches the probe), clean A/B `1531070` → `debadfa`:

| | user (min of 3) |
|---|---|
| without cache (`1531070`) | 0.99–1.01 s |
| with cache (`debadfa`) | **0.72–0.74 s** |

**−28%.** The `format!("{name}(*)")` + re-intern it removes ran *per variadic call*.

Two things this exposes, both bigger than the cut:

1. **Variadic dispatch never refills the IC.** The refill at `dispatch.rs:857` is on
   the exact-hit branch only; a variadic hit dispatches without caching. So every
   variadic call re-walks: IC miss → full exact-probe hierarchy walk → variadic
   probe → *second* full hierarchy walk. Change 2 removed the string work from that
   path and left both walks. **Caching the variadic resolution in the IC is the
   larger remaining win on this path** — and it is U-IC's natural follow-on, not a
   new unit.
2. **A benchmark suite with a hole reports a win as noise.** The suite had no
   variadic coverage at all, so the path was unmeasurable by construction. `wash`
   and `never executed` are indistinguishable to a harness that only reports time —
   which is the same lesson as F9 (attribution ≠ mechanism), one level up: before
   concluding "no effect", confirm the code ran.

`init_selector_cache` **is** still dead code (`intern` needs `&mut`, `lookup_method`
has `&VM`) — that part of the original criticism stands, and it is a separate,
smaller cleanup.

## Change 3 (reorder `Value::class` arms) — dropped

Benchmarked, no measurable change on `bare_send`/`arith_send`; LLVM was already
ordering the match. Reverted before landing, per the spec's own "drop if it
washes" rule.

## Verification

`cargo test --workspace` green; zero golden diff; every wren-suite program still
prints its Wren-identical output (`benchmarks/vm/compare-wren.py`, which now
checks that mechanically rather than by eye).
