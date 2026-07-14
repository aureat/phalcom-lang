# Performance log

Running, measured record of optimization cuts and profiling findings for the
Phalcom VM. One entry per landed cut; findings that reshape the plan live in
[`findings.md`](findings.md).

Governed by [`performance.md`](../../spec/v0.2/performance.md) +
[ADR-0051](../../adr/0051-performance-strategy-measure-first-tiered-optimization.md)
(measure-first, tiered, behavior-invariant). Every entry cites a **before/after
number from the U-BENCH harness** (`benchmarks/vm/`, criterion target
`phalcom-core/benches/vm_bench.rs`) — no oral numbers (law P1). A cut lands only on
a measured win + zero golden diff; an unproven optimization is reverted and
recorded as a finding, not shipped.

## Cuts (landed)

| # | Unit / tier | Cut | Measured | Golden diff | Commit |
|---|-------------|-----|----------|-------------|--------|
| [001](001-prim-abi-inline-args.md) | U-PRIM-ABI / Tier 2 | On-stack arg buffer replaces the per-send heap `Vec` in the primitive dispatch path | arith_send −41.5%, bare_send −33.8% | none | `37f31c9` |
| [002](002-gc-win-a-box-fat-variants.md) | U-GC Win A / Tier 4 | Box the six fat `Object` variants — `size_of::<Object>()` 280 B → 40 B | `for.ph` −43% wall, `skynet` −34% wall (`sys` 2–4× less); RSS a wash; `bare_send` +5% | none | `7480d75` |
| [003](003-vm-trace-feature-gate.md) | U-TRACE / Tier 1 | `vm-trace` feature compiles the dispatch loop's per-opcode span + `debug!`s out by default | arith_send (5M, whole-process) −16.7%; Skynet ≈−1% on `user`, unresolvable on `real` | none | `<pending>` |

## Investigated, not landed

| Candidate | Result | Why | Ref |
|-----------|--------|-----|-----|
| Fiber-stack pool (U-GC "Win B") | **reverted — null result** | Skynet RSS is dominated by ~1M immortal `FiberObject` shells (no GC), not stack/frames buffers; A/B indistinguishable | [findings F5](findings.md#f5--fiber-stack-pool-implemented-measured-reverted-null-result), `ad4a215` |
| Escape-analysis of `Option` (`Some`) | not built — premise falsified | `List/Map.at` already zero-alloc (`None` singleton); discarded `Some` already elided | [findings F2](findings.md#f2--option-escape-optimization-premise-falsified) |

## Findings

Full write-ups in [`findings.md`](findings.md):

- **F1** — measured baseline supersedes the oral 29×: Skynet **~19–20× Wren**,
  ~7–9× RSS. Attribution re-ranks perf.md §2 — **malloc/free is the #1 mechanism**
  (arith 19.7%, Skynet 28.2%), above tracing span and dispatch lookup.
- **F2** — `Option`-escape optimization premise falsified (lookups already
  zero-alloc). No unit.
- **F3** — memmove (20.6% Skynet) is per-fiber `Vec` growth-realloc, **not**
  `mem::take` (which is O(1)). Zero fiber pooling today.
- **F4** — U-IC preconditions: `Symbol` is one mixed namespace (needs selector-only
  interner first); IC seam is a comment only; no `ClassObject` epoch / global
  `world_version` yet.
- **F5** — fiber-stack pool built, measured, **reverted** (null result). Redirects
  the Skynet memory win to the **U-GC collector** (freeing shells), not pooling.
  The reverted code never entered the git object DB — design survives in F5, ~1h
  to rebuild.
- **F6** — U-GC's normative tables had **two free-a-live-object bugs**
  (`Block.closure` absent; `Upvalue::Open.fiber` wrongly assumed root-traced), two
  missed roots (`sealed_classes`, `checking`), and five edges added by the
  annotation work. §2.3 regenerated field-level over all 16 variants. The
  exhaustive `match` catches a new *variant*, never a new *field* — the table is
  the only defence.
- **F7** — `size_of::<Object>()` **grew 256 → 280 B** (`ClassObject.attributes`).
  Win A is **six** boxed variants, not "the driver"; **do not box `Instance`**
  (24 B, most-allocated). Target 280 → ~40 B, a 7× arena density win.
- **F8** — Tier 1's size held (18.2% measured vs 18.3% predicted) but **both**
  mechanisms were wrong: not a subscriber misconfiguration (−0.4%), and not the span
  (half the cost — the three `debug!`s are the other half). A fix dispatched from the
  README's "tracing span" framing would have booked −8.4% and closed the candidate.
  Attribution ≠ mechanism; only mechanism gives the fix its shape. Closed by cut 003.

## Method — instrument selection (learned the hard way in 002)

**Match the instrument to where the effect lives.** Cut 002 was nearly abandoned
because criterion micro-benches were used to judge a representation change: they
measure the per-send pointer chase it *costs* and are blind to the per-allocation
arena growth it *removes*. The real workloads showed −34%/−43% where the
micro-benches showed a regression.

- **Representation / allocation changes** → `for.ph`, `skynet`, `/usr/bin/time -l`.
  Watch `sys` time, not just wall.
- **Dispatch / send-path changes** → criterion (`benchmarks/vm/`) — **except anything
  `tracing`-sensitive** (003). `benches/vm_bench.rs` drives `Interpreter` directly and
  installs **no subscriber**, so callsite interest caches as `never`: a cheaper path
  than the real binary, which installs a registry. Criterion is *blind* to per-opcode
  tracing cost and showed ~no win where whole-process showed −16.7%.
- **Skynet: read `user`, not `real`** (003). Its `real` spread was 5.4 s **within one
  binary** (13.98–19.40) — it cannot resolve a single-digit-percent effect. The
  variance lives in `sys` (page faults, allocation); `user` held to ±0.4 s across the
  same runs. `BASELINE.md`'s own 13.7–15.6 s range says the same thing.
- **Criterion's p-value covers within-run variance only.** On this hardware it
  certified noise as `p = 0.00` significance twice; the *same binary* against the
  *same saved baseline* reported +8.8% and then +1.3%. For effects under ~10%, use
  alternating same-session A/B and read the sign across pairs, not the p-value.
- **Never run `cargo build` inside a measurement loop** — it contends with the bench
  (two runs of 002's A/B came back at 2× normal).
- **A hypothesis fitted to one noisy run will appear to confirm.** 002 built one,
  tested it, "confirmed" it, and had to discard it. Reproduce the *baseline
  observation* before explaining it.

## Next measured levers

Ranked by attributed cost on the arith micro-bench + Skynet, after cut 001:

1. ~~**Tier 1 — tracing span (~18.3% arith).**~~ **Done — [cut 003](003-vm-trace-feature-gate.md)
   ([U-TRACE](../units/U-TRACE/plan.md)), arith −16.7%.** The attribution was right about the
   size and wrong about the mechanism twice over: the cost is **not** a subscriber
   misconfiguration (fixing that bought −0.4%), and it is **not** span-specific — it splits
   evenly with the loop's three `debug!`s, so the fix had to gate all four callsites, not the
   span alone.
2. **Tier 4 — U-GC collector (malloc 28.2% Skynet).** The dominant Skynet cost is
   allocating + retaining ~1M `FiberObject` shells. This is the real RSS lever
   (F5), a large unit (non-moving mark-sweep, ADR-0050).
3. **Tier 3 — U-IC (dispatch lookup 13.9% arith).** Monomorphic inline cache;
   needs the selector-only interner first (F4). Also carries the arithmetic
   fast-path deferred from U-PRIM-ABI (DEC-PRIM-B).

## Session ledger (2026-07-14)

- `757d88a` — U-BENCH Tier 0 harness (criterion + BASELINE), concurrent session.
- `37f31c9` — **U-PRIM-ABI cut 001** (arith −41.5%). Real win.
- `ad4a215` — F5 fiber-pool null result (code reverted, finding kept).
