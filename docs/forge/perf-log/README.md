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
| [003](003-vm-trace-feature-gate.md) | U-TRACE / Tier 1 | `vm-trace` feature compiles the dispatch loop's per-opcode span + `debug!`s out by default | arith_send (5M, whole-process) −16.7%; Skynet ≈−1% on `user`, unresolvable on `real` | none | `1ef999b` |
| [004](004-hotpath-rc-callable.md) | U-HOTPATH / Tier 2 | Share block-literal `Callable` via `Rc` instead of deep-cloning its `Chunk` per evaluation | Skynet −30% `user`, **−63% RSS** (3.73 → 1.37 GB), `sys` −94%; fiber_churn −16%. **Costs +5–7% on send-heavy programs** (`Rc` hop per instruction) — Change 1 (chunk hoist) is what repays it | none | `1531070` |
| [004](004-hotpath-rc-callable.md) | U-HOTPATH / Tier 2 | Memoize the `Invoke` variadic probe's derived `name(*)` selector (kills a `format!` + re-intern per variadic call) | **variadic_send −28%** (1.00 → 0.72 s). Invisible to every other benchmark — the probe sits behind two misses and no other program reaches it | none | `debadfa` |
| [005](../perf-log/SCOREBOARD.md#5--timeline--best-result-after-each-change) | F12 / Tier 3 | Per-callsite `(module, slot)` cache for `GetGlobal`/`SetGlobal` in `Chunk.gcaches`, guarded by `ModuleObject.globals_version` (bumped only when `declare` allocates a **new** slot) | **bare_send −17.9%**, arith_send −14.4%, **fiber_spawn −20.1%**, variadic_send −8.3%, skynet −7.7% `user` (**2.9× Wren**), fiber_churn −21.4%, `for` −3.6%. RSS unchanged | none | `39d9042` |
| [006](../perf-log/SCOREBOARD.md#5--timeline--best-result-after-each-change) | F14 S2 / Tier 2 | Drop `spans[ip]` from the dispatch loop's read-decode — the span is discarded on the happy path. Re-read in `Invoke` (inside the borrow the IC probe already takes, so the send path pays nothing) and in `SuperSend`, its only two consumers | **`for` −6.8%**, method_call −5.6%, variadic_send −5.2%, arith_send −3.0%, bare_send −2.8%, skynet −2.8% `user`. RSS unchanged | none | `916be0a` |

## Investigated, not landed

| Candidate | Result | Why | Ref |
|-----------|--------|-----|-----|
| Fiber-stack pool (U-GC "Win B") | **negative — flag kept, stays OFF, do not use** | Rebuilt behind the off-by-default `fiber-pool` feature after F5's revert. Re-measured against the high-turnover workload F5 asked for (`fiber_churn.ph`): **+72–86% peak RSS**, linear at ~450 B/fiber, and +37% `user` at 1M fibers. Worse, not neutral. Owner ruled the flag stays for reconstructability; reviving it needs a new mechanism, not a re-run | [findings F10](findings.md#f10--the-fiber-pool-is-not-neutral-it-is-negative--and-f5-measured-the-wrong-workload), [F5](findings.md#f5--fiber-stack-pool-implemented-measured-reverted-null-result), `ad4a215` |
| U-HOTPATH Change 3 (reorder `Value::class` arms) | dropped pre-landing | No measurable change; LLVM was already ordering the match | [004](004-hotpath-rc-callable.md) |
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
- **F8** — a freshly bootstrapped `VM` is **not garbage-free**: `core.ph`'s top-level
  `Closure` is unreachable the moment bootstrap returns, so the first collection on any
  VM legitimately sweeps one object. Surfaced by U-GC's step-2 tests.
- **F17** — **an instruction costs ~10–13 ns** (~75–96 Minstr/s), and the **2.8× spread**
  (10.4 `method_call` → 29.3 `map_numeric`) is per-instruction *work*, not instruction
  count — a distinction wall-clock-vs-Wren cannot make. Closes H3 via an off-by-default
  `opcode-histogram` feature + `benchmarks/vm/opcode-cost.py` (two builds: counts from
  the counting build, time from the default one). Corroborated: `bare_send` retires 16
  instructions/send → 170 ns/send vs criterion's ~174.
- **F13** — **bootstrap regressed 5 ms → 180 ms** (`debadfa` → `3b2dd97`), a fixed tax on
  every process. Not a throughput regression: the `ifTrue` inliner is **exponential in
  nest depth**, and `core.ph`'s new 14-deep `codePointAt` costs ~200 ms to compile *by
  itself*. A 20-line source method can hang the compiler for seconds. Nothing in the
  harness measures bootstrap, so a 35× regression passed every gate.
- **F12** — **module globals are a SipHash `HashMap` probe per access** (`GetGlobal`/
  `SetGlobal`); the IC does not cover them. Now the top non-loop cost on send-heavy code
  (~13% of `bare_send` ticks; `for`'s inner loop pays four probes per iteration).
  Prototyped per-callsite slot cache: **bare_send −15%, arith_send −14%, for −5%**.
- **F11** — **skynet is GC-bound and its collections free nothing**: `trace_object` is
  ~20% of ticks, the GC family ~30%. Its ~1.1M fibers are all live, so every cycle
  traces everything and reclaims ~nothing, then `GC_GROW_FACTOR = 1.5` re-triggers.
  Yield-adaptive threshold (~15 lines): **skynet −10% user with RSS also −3%**;
  fiber_churn −10%.
- **F10** — the fiber pool (now the off-by-default `fiber-pool` feature) is **negative,
  not null**: re-measured against the high-turnover workload F5 asked for, it costs
  **+72–86% peak RSS** (linear, ~450 B per fiber) and +37% `user` at 1M fibers. F5's
  revival condition is refuted, the flag should be deleted, and while it exists its
  `fiber_pool: _` non-root classification holds only because one push site calls
  `.clear()`.
- **F14** — **the dispatch loop re-derives every frame field on every opcode**: a 96 B
  `CallFrame` copy, 2–3 SlotMap lookups, a `spans[ip]` load discarded on the happy
  path, and a per-opcode safepoint. Wren hoists all of it into locals and reloads only
  on call/return. **This, not method lookup, is where 174 ns/send vs Wren's ~47 ns
  lives** — and it is why `run_until_inner` has held 33–35% of ticks across every cut.
  Not a new lever: it **resizes lever 4** below.
- **F15** — **`Value` is 16 B against Wren's NaN-boxed 8 B, and that 2.0× *is* skynet's
  2.0× RSS** (a fiber is a stack of `Value`s) — superseding F7's arena framing; the
  arena is fine at 40 B/slot. **NaN-boxing is blocked**: a NaN payload holds ~48–51
  bits and `ObjRef` is a full 64 (`slotmap` `u32` index + `u32` version). Shrinking the
  key first is a correctness review (version-wraparound), likely more work than the
  boxing. `CallFrame` is 96 B against Wren's 24 B.
- **F16** — **superinstructions: no, defer.** They would amortize F14's re-derivation
  rather than fix it; and the F13 inliner already covers the classic arithmetic win.
  Its second reason is **narrowed but not retired** by F17: H3 is closed, so a histogram
  now exists — but it counts **single opcodes, not adjacent pairs**, and fusion
  selection needs pair frequencies. Extend `opcode_stats::record` to `(prev, cur)`
  (~10 lines) before re-asking. Reason 1 (do F14's S1 first) is load-bearing regardless.
- **F9** — Tier 1's size held (18.2% measured vs 18.3% predicted) but **both**
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
- **"What does one instruction cost" → `benchmarks/vm/opcode-cost.py`, never `sample`**
  (F17). The dispatch loop is one `match` in one function, so the profiler books every
  opcode arm to `run_until_inner` and prices none of them. The script is **two builds
  on purpose** — counts from `--features opcode-histogram`, wall-clock from a default
  build — because counting costs an increment per instruction, i.e. the same per-opcode
  work 003 gated `vm-trace` for. Counts are deterministic, which is what makes the split
  sound. Never read a timing from a histogram build.
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

> **The ranking basis is stale — see [design-notes O5](../../design-notes/optimization-method-and-harness-fidelity.md).**
> These shares come from one profile of a binary that no longer exists: cut 001 removed most of
> the malloc share and cut 003 removed the ~18.2% tracing share, which re-normalizes every
> remaining share upward. The order below may well still hold, but **do not size or justify U-IC
> from "13.9%"** — re-profile at HEAD first. Also open: [O4](../../design-notes/optimization-method-and-harness-fidelity.md)
> — `fiber_spawn` ~37% vs the previous run, unattributed, plausibly U-GC's collector.

Ranked by attributed cost on the arith micro-bench + Skynet, after cut 001:

1. ~~**Tier 1 — tracing span (~18.3% arith).**~~ **Done — [cut 003](003-vm-trace-feature-gate.md)
   ([U-TRACE](../units/U-TRACE/plan.md)), arith −16.7%.** The attribution was right about the
   size and wrong about the mechanism twice over: the cost is **not** a subscriber
   misconfiguration (fixing that bought −0.4%), and it is **not** span-specific — it splits
   evenly with the loop's three `debug!`s, so the fix had to gate all four callsites, not the
   span alone.
2. ~~**Tier 4 — U-GC collector (malloc 28.2% Skynet).**~~ **Landed** (non-moving
   mark-sweep, ADR-0050), together with [cut 004](004-hotpath-rc-callable.md)'s
   `Rc<Callable>`. Skynet is now **2.4–2.5 s / 1.44 GB** against Wren's 0.7–0.8 s /
   0.67 GB — **~3.2× wall, ~2.2× RSS**, from F1's ~19–20× / ~7–9×.
3. ~~**Tier 3 — U-IC.**~~ **Landed** (`f5e41f1`) — monomorphic IC on `Invoke`, guarded
   by `(class, world_version)`. Method lookup is no longer the top dispatch cost;
   re-profiled below.

**Re-ranked from fresh `sample` profiles at `1e1b101`** (2026-07-14). Every item
below is measured on a prototype, not hypothesized:

1. **Fix the exponential `ifTrue` inliner (F13).** Returns **~175 ms to every
   process** and removes a compiler-hang hazard reachable from ordinary source. The
   cheapest large win on the list, and the only one that is also a correctness/DoS
   issue.
2. ~~**Slot-resolve module globals (F12).**~~ **Landed `39d9042`** — version-guarded
   per-callsite cache, **bare_send −17.9%**, fiber_spawn −20.1%, fiber_churn −21.4%,
   skynet −7.7% (**2.9× Wren**). The late-binding/shadowing question needed **no
   ruling**: the guard preserves current semantics exactly, and the shadowing it
   guards against is reachable in **0 of 672** `.ph` files. See [F12](findings.md#f12--module-globals-are-a-siphash-probe-per-access-the-ic-does-not-cover-them).
3. **Yield-adaptive GC threshold (F11).** **−10% skynet user, −10% fiber_churn, RSS
   better on both**, ~15 lines. Skynet is GC-bound and its collections free nothing.
4. **U-HOTPATH Change 1 — hoist the *frame* out of the dispatch loop.** Not optional
   and not a fresh candidate: cut 004 traded a per-instruction pointer hop for a
   per-block-evaluation chunk copy and left a **measured +5–7% regression on
   send-heavy programs** on the table. This is the half that pays it back.
   **[F14](findings.md#f14--the-dispatch-loop-re-derives-every-frame-field-on-every-opcode)
   resizes this item: it is not "a chunk pointer".** The loop re-derives the *whole*
   frame every opcode — a 96 B copy, 2–3 SlotMap lookups, a discarded `spans[ip]` load,
   a per-opcode safepoint — and that is where 174 ns/send vs Wren's ~47 ns lives. Sized
   at *est* −30–45%/send (S1), with S2 (drop `spans[ip]`, *est* −3–8%) worth landing
   **first** as an isolated A/B before S1 makes it unmeasurable. Estimates, not
   measurements — law P1.
5. **Cache variadic resolution in the IC (004).** A variadic hit never refills the
   IC, so every variadic call pays **two** full hierarchy walks. Change 2 removed
   the string work from that path and left both walks.
6. **DEC-PRIM-B arithmetic fast path.** `call_method` is ~14% of `bare_send` ticks
   (arg-buffer build + frame setup); `Value::class` another ~4%.
7. ~~**Object density (F7).**~~ **Reframed by [F15](findings.md#f15--value-is-2-wrens-and-objref-blocks-nan-boxing)
   — this was never an arena question.** The arena is fine at 40 B/slot; the bytes are
   in the fiber's two `Vec`s. At HEAD skynet is ~1.19 KB/fiber vs Wren's ~0.6 KB, and
   **that 2.0× is `Value`'s 2.0×** (16 B vs Wren's NaN-boxed 8 B) — a fiber is a stack
   of `Value`s. Split into the real ladder, all *estimates* (law P1):
   - **Presize the two fiber `Vec`s — this is [F3](findings.md#f3--memmove-206-skynet-is-vec-growth-not-memtake)/H9,
     ~10 lines, *est* −10–15% skynet `user`. Highest gain-per-effort item on the whole
     list**, open since origin, never re-profiled after cuts 001–004.
   - Box the fiber side tables (`BTreeMap` + `HashSet` sit inline in every
     `FiberObject`, empty for ~every skynet fiber): shell 176 → ~104 B.
   - `CallFrame` 96 → ~32 B (S4; Wren's is 24 B).
   - `Value` 16 → 8 B: **blocked** — a NaN payload holds ~48–51 bits and `ObjRef` is a
     full 64. Shrinking the key is a version-wraparound correctness review, likely more
     work than the boxing. *Est* RSS 1.32 → ~0.85 GB (~1.3× Wren, **not** under it).

## Session ledger (2026-07-14)

- `757d88a` — U-BENCH Tier 0 harness (criterion + BASELINE), concurrent session.
- `37f31c9` — **U-PRIM-ABI cut 001** (arith −41.5%). Real win.
- `ad4a215` — F5 fiber-pool null result (code reverted, finding kept).
- `94b6bbf` — U-GC steps 3–4 (`System.gc`, safepoint latch); step 5 fiber pool a
  second null result, kept in `git stash@{0}`, not shipped.
- `1ef999b` — **U-TRACE cut 003** (arith −16.7%). Real win. Two mechanisms falsified
  on the way (F9); `main.rs` deliberately left untouched.
- `1531070` — **U-HOTPATH cut 004** (Skynet −30% user, −63% RSS). Real win, with a
  measured +5–7% send-path regression that Change 1 must repay.
- `debadfa` — U-HOTPATH Change 2. Sound, green, **unmeasured** — see 004.

## Harness note (2026-07-14)

Two gaps closed after the whole suite was re-measured at `debadfa`:

- **The criterion benches checked only that the program did not error.** They now
  read back each program's loop counter and value checksum from the `main` module
  and fail on a wrong answer (`benches/vm_bench.rs`). A build that skips the loop or
  mis-dispatches is fast and wrong; the old gate would have booked it as a win.
- **The wren-suite comparison was verified by eye, once, and had gone stale by up to
  20×.** `benchmarks/vm/compare-wren.py` now runs each pair best-of-N and diffs
  Phalcom's stdout against Wren's mechanically, exiting non-zero on any mismatch. It
  immediately found `method_call.ph` failing to parse (`!x` — the surface moved to
  `not x` under a later unit and the port was never updated), i.e. a suite row that
  had silently stopped being a benchmark at all.
