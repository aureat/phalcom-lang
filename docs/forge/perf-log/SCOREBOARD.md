# Performance scoreboard — best measured result per change

> **Purpose.** One file that answers, at any commit: *how fast is Phalcom, how many
> times slower than Wren is it, what does one operation cost in nanoseconds, and
> which function is eating the time?* [`README.md`](README.md) records **why** a cut
> landed; [`findings.md`](findings.md) records **what was learned**. This file
> records **the numbers**, and only the numbers, over time.
>
> Numbers here are intended to graduate into the source: once a per-operation or
> per-function cost is stable and reproduced, it belongs in the rustdoc of the
> function that owns it (`# Performance` section), citing the row below.

**Laws this file inherits** (ADR-0051, `performance.md` §2, law P1):

- **No oral numbers.** Every cell is a measurement with a commit, a workload and an
  instrument, or it says `not measured` — never a guess, never an interpolation.
- **Whole-process lifetime is the unit**, not steady-state throughput. Peak RSS is
  co-equal with wall-clock; a row without RSS is half a row.
- **Read `user`, not `real`, on fiber workloads.** Skynet's `real` spread was 5.4 s
  *within one binary*; `user` held to ±0.4 s. (README §Method)
- **Best-of-N, matched commits, same session.** An A/B against a working tree that
  drifted two commits reported the wrong *sign* once (HANDOFF §Traps).
- A `~` prefix means derived (e.g. ns/send = mean ÷ send count), not directly timed.

Current HEAD when last updated: **`45ffe76`** (2026-07-14).

> **§1's suite table and §3b are one commit behind** (`2997d0b`, pre-F12); §2, §3a,
> §3c and §5 are at `39d9042`. F12 moved `for` 0.729 → 0.69 s and `method_call`
> 0.535 → 0.519 s in the post-landing verification run, but that run was not
> best-of-5 — the table is re-run at the next cut rather than mixing instruments.
> Skynet's §1 row **is** updated (best-of-3, `user`).

---

## 1. Headline — how many times slower than Wren

Best measured Phalcom time per benchmark, against Wren on the same machine. Every
row is output-verified (Phalcom stdout diffed byte-for-byte against Wren's) —
a row that computes the wrong answer is not a measurement and is not reported.

Instrument: `REPS=5 benchmarks/vm/compare-wren.py` (best-of-5, output-verified,
exits non-zero on any stdout mismatch). Measured at **`2997d0b` (HEAD)**, 2026-07-14.

| Benchmark | Work | Wren | Phalcom (best) | **Slowdown** | At `debadfa` | At origin |
|---|---|---|---|---|---|---|
| `binary_trees_gc` | alloc + explicit GC | 0.552 s\* | 0.824 s | **1.5×**\* | 1.5×\* | 2×\* |
| `fibers` | 100k chained fibers | 0.032 s | 0.122 s | **3.8×** | 4.0× | ~12× |
| `binary_trees` | alloc-heavy tree | 0.176 s | 0.829 s | **4.7×** | 4.9× | 7.3× |
| `fib` | fib(28) ×5, recursion | 0.173 s | 0.864 s | **5.0×** | 4.8× | 8.6× |
| `map_numeric` | 2M map ops | 0.750 s | 3.913 s | **5.2×** | 4.5× | 5.9× |
| `method_call` | 2M dispatch | 0.093 s | 0.535 s | **5.8×** | 5.7× | 8.2× |
| `map_string` | ~193k-key string map | 0.099 s | 0.691 s | **7.0×** | 6.9× | 46× |
| `string_equals` | 10M compares | 0.109 s | 1.107 s | **10.1×** | 10.0× | 17× |
| `for` | 1M list build + sum | 0.053 s | 0.729 s | **13.7×** | 13.6× | ~144× |
| **`skynet`** | **1.11M fibers, depth-6 ×10 fan-out** | **0.61 s `user`** | **1.79 s `user`** | **2.9×** | — | **~19–20×** |

All 9 suite rows `ok` (stdout byte-identical to Wren's). \* `binary_trees_gc`'s Wren
time includes `System.gc()` calls that Phalcom's port drops — not apples-to-apples.

**Band: 1.5–13.7×, centre of mass ~5×.** `for` and `string_equals` are the tail.
The oral "~29× on Skynet" is **superseded** — never reproduced; measured origin was
~19–20× (F1) and HEAD is **3.2×**.

**The `debadfa` column is not stale — verified, not assumed.** The obvious prediction
(F13's +175 ms inflated that table, so HEAD should be ~24% better on a 0.73 s row) is
**wrong**: `debadfa` *predates* `3b2dd97`, so it never carried the regression, which
landed after it and was gone by `0274f10`. HEAD ≈ `debadfa` on every row, and that is
the correct expectation. `map_numeric`'s 4.5× → 5.2× drift is the **Wren** side
(0.86 → 0.75 s); Phalcom moved 3.85 → 3.91 s, i.e. noise. Do not read it as a
regression.

## 2. Memory — peak RSS vs Wren

Instrument: `/usr/bin/time -l`, best-of-3, at **`2997d0b` (HEAD)**. Four columns per
law: `real` is unusable on fiber workloads (see below), `user` is the signal, `sys` is
where allocation/paging effects hide, RSS is co-equal with time.

| Workload | `real` | `user` | `sys` | **Peak RSS** |
|---|---|---|---|---|
| `skynet` — Phalcom | 2.01–2.72 s | **1.79–1.86 s** | 0.21–0.30 s | **1.322 GB** |
| `skynet` — Wren | 0.72–0.75 s | **0.61–0.62 s** | 0.10–0.12 s | **0.667 GB** |
| **`skynet` — ratio** | ~2.8× | **~2.9×** | ~2.1× | **~2.0×** |
| `fiber_churn` (500k spawn→drop) — Phalcom | 0.26–0.27 s | **0.22 s** | 0.03 s | **264 MB** |
| `fiber_churn` — Wren | *not ported* (hole H4) | — | — | — |
| `bootstrap` (`System.print(1)`) — Phalcom | 0.00 s | **0.00 s** (~5 ms) | 0.00 s | **6.8 MB** |

**Both fiber rows beat every number previously logged**, because F11's tables were
measured on a *prototype* at `1e1b101` — which still carried F13's +175 ms — while
`0274f10` and `4f2eed8` both landed after:

| | logged | **HEAD** | Δ |
|---|---|---|---|
| skynet `user` | 2.13 s (F11 prototype) / 2.4–2.5 s (README) | **1.94 s** | **−9% / −20%** |
| skynet RSS | 1.534 GB (F11) / 1.44 GB (README) | **1.322 GB** | **−14% / −8%** |
| fiber_churn `user` | 0.46 s (F11) | **0.28 s** | **−39%** |
| fiber_churn RSS | 420 MB (F11) | **264 MB** | **−37%** |

fiber_churn's −39% is almost exactly F13's constant (0.46 − 0.175 ≈ 0.285) — on a
0.28 s workload the bootstrap fix *is* the benchmark. Its −37% RSS is not explained
by that and is unattributed (**hole H12**).

Derived density, at HEAD:

| | Wren | Phalcom | Ratio |
|---|---|---|---|
| skynet, per fiber (1.11M fibers) | ~0.6 KB | **~1.19 KB** | **~2.0×** |

`size_of::<Object>()` = **40 B** (was 280 B before cut 002 boxed six fat variants —
F7). Fiber density is the open memory lever: `FiberObject` is 176 B *before* its two
buffers, untouched since Win A. Note per-fiber is now ~1.19 KB, down from F11's
~1.4 KB — the ladder moved without anyone pulling that lever.

## 3. Per-operation cost — nanoseconds per unit of work

**This is the table that graduates into rustdoc.** Each row = one program's mean
time ÷ the operations it performs. Derived, so read it as an order-of-magnitude
budget per op, not a cycle-accurate figure.

### 3a. Criterion micro-benches (`phalcom-core/benches/vm_bench.rs`)

**At `39d9042` (HEAD, F12 landed)**, `cargo bench -p phalcom-core --bench vm_bench`:

| Benchmark | Program | Ops | Mean | **Per-op** | Criterion CI | `2997d0b` | Origin | **Δ vs origin** |
|---|---|---|---|---|---|---|---|---|
| `bare_send` | static, arg-free send to a user method (full `CallFrame` push + `return 0`) | 200,000 sends | **34.70 ms** | **~174 ns/send** | [34.65, 34.75] ms | 42.27 ms / ~211 ns | 65.7 ms / ~329 ns | **−47.2%** |
| `arith_send` | primitive `1 + 2` send (`number_add`, no frame push, per-call arg `Vec`) | 200,000 sends | **30.25 ms** | **~151 ns/send** | [30.17, 30.36] ms | 35.35 ms / ~177 ns | 72.1 ms / ~361 ns | **−58.0%** |
| `fiber_spawn` | `Fiber.new{}` + `.call()` + `Fiber.yield` | 20,000 spawns | **12.31 ms** | **~615 ns/spawn** | [12.03, 12.70] ms | 15.40 ms / ~770 ns | 24.2 ms / ~1.21 µs | **−49.1%** |
| `variadic_send` | variadic `name(*)` dispatch (added `8ba87ec`) | 2,000,000 sends | **681.03 ms** | **~341 ns/send** | [677.7, 685.0] ms | 742.65 ms / ~371 ns | *(post-dates origin)* | — |

**The headline per-op numbers: a user-method send costs ~174 ns; a primitive
arithmetic send ~151 ns; a fiber spawn+call+yield ~615 ns.** Every one is now
better than half its origin cost.

Note the **inversion since origin**: arith_send was *slower* than bare_send at origin
(361 vs 329 ns) and is now *faster* (177 vs 211 ns). Cut 001 killed the per-send arg
`Vec` that made the primitive path lose; what remains on bare_send's path — a real
`CallFrame` push plus a bytecode body — is now the more expensive of the two. Any
model that still says "primitives are the slow path" is out of date.

Origin (`757d88a`) CIs for reference: `bare_send` [64.6, 67.1] ms; `arith_send`
[68.2, 77.7] ms; `fiber_spawn` [23.0, 25.9] ms.

> Criterion's `change:` line on this run compared against a stored baseline of unknown
> provenance (it read `p = 0.50`, `p = 0.06`, `p = 0.00`) — **ignored**. The Δ column
> above is against `BASELINE.md`'s recorded origin, which is the only matched
> comparison available. README §Method: criterion's p-value covers within-run variance
> only and has certified noise at `p = 0.00` on this hardware twice.
>
> Bootstrap (~5 ms at HEAD) is *inside* each criterion iteration — each runs
> `Interpreter::new()` — so these carried F13's +175 ms between `3b2dd97` and
> `0274f10`. At HEAD that contamination is ~5 ms on a 42 ms bench: ~12%, still not
> nothing. Isolating it is unclaimed work.

### 3b. Whole-process per-op (derived from the wren-suite at HEAD, `2997d0b`)

Whole-process, so each row carries bootstrap (~5 ms) and process start — for `fibers`
(0.122 s) that is ~4% of the row; for `map_numeric` it is negligible.

| Benchmark | Ops | Phalcom | **Per-op** | Wren per-op | Gap |
|---|---|---|---|---|---|
| `method_call` | 2M dispatches | 0.535 s | **~268 ns/send** | ~47 ns | 5.8× |
| `for` | 1M iterations (build+sum) | 0.729 s | **~729 ns/iteration** | ~53 ns | 13.7× |
| `string_equals` | 10M compares | 1.107 s | **~111 ns/compare** | ~11 ns | 10.1× |
| `map_numeric` | 2M map ops | 3.913 s | **~1.96 µs/op** | ~375 ns | 5.2× |
| `skynet` | 1.11M fibers (spawn+run+join) | 1.94 s `user` | **~1.75 µs/fiber** | ~549 ns | 3.2× |
| `fiber_churn` | 500k spawn→run→drop | 0.28 s `user` | **~560 ns/fiber** | *not ported* (H4) | — |

Cross-check: `method_call`'s ~268 ns/send (whole-process) against `bare_send`'s
~211 ns/send (criterion) — same order, and the gap is the right sign, since
`method_call.ph` passes arguments and `bare_send.ph` does not. Two independent
instruments agreeing to ~25% is the closest thing to corroboration this harness has.

`for`'s ~729 ns/iteration is the worst per-op number in the suite and the one with a
named mechanism: its inner loop pays **four SipHash global probes per iteration**
(`i` read ×2, `list` read, `i` write) — F12. `fiber_churn`'s ~560 ns/fiber vs
`fiber_spawn`'s ~770 ns/spawn differ by the `Fiber.yield` the latter adds.

### 3bb. Per-**instruction** cost (H3, F17) — `45ffe76`, 2026-07-14

**What one Phalcom instruction costs: ~10–13 ns on hot loops (~75–96 Minstr/s).**

Instrument: `benchmarks/vm/opcode-cost.py`. **Two builds on purpose** — counts from
a `--features opcode-histogram` build, wall-clock from a **default** build, divided.
Counting costs an increment per instruction (the same per-opcode work `vm-trace`'s
span cost 18.2% of arith wall — 003), so a timing from a counting build is wrong.
Counts are deterministic, so this keeps the counter out of the number it produces.
Do not collapse it into one run.

| Benchmark | Wall | Instructions retired | **ns/instr** | Minstr/s | Top opcodes |
|---|---|---|---|---|---|
| `method_call` | 0.512 s | 49,334,099 | **10.4** | 96.3 | GetSelf 23%, Invoke 15%, GetField 12% |
| `for` | 0.712 s | 68,000,706 | **10.5** | 95.5 | Invoke 20%, GetLocal 19%, GetGlobal 8% |
| `string_equals` | 1.095 s | 99,000,674 | **11.1** | 90.4 | Constant 25%, Invoke 15%, Pop 11% |
| `arith_send` | 0.035 s | 3,000,674 | **11.6** | 86.4 | Constant 26%, Invoke 19%, GetGlobal 13% |
| `bare_send` | 0.039 s | 3,200,683 | **12.2** | 82.2 | Constant 18%, GetGlobal 18%, Invoke 18% |
| `fib` | 0.883 s | 69,421,572 | **12.7** | 78.6 | Invoke 25%, GetLocal 18%, Constant 14% |
| `binary_trees` | 0.798 s | 61,541,242 | **13.0** | 77.1 | Invoke 13%, GetLocal 13%, GetSelf 12% |
| `variadic_send` | 0.668 s | 50,000,686 | **13.4** | 74.9 | Invoke 23%, Constant 20%, GetGlobal 15% |
| `fibers` | 0.115 s | 4,800,676 | **24.0** | 41.7 | Invoke 22%, GetGlobal 14%, Constant 8% |
| `fiber_churn` | 0.276 s | 11,000,674 | **25.1** | 39.9 | GetGlobal 22%, Invoke 22%, Constant 9% |
| `map_numeric` | 3.696 s | 126,000,699 | **29.3** | 34.1 | GetGlobal 22%, Invoke 20%, Pop 12% |
| `bootstrap` | 0.007 s | 662 | *(11,241)* | 0.1 | Constant 31%, Method 26%, GetGlobal 9% |

**Spread (execution-bound rows): 10.4 → 29.3 ns/instr, 2.8×.**

**This is a new attribution axis**, and the reason it matters: it separates *"runs
more instructions"* from *"each instruction does more work"* — which wall-clock
against Wren cannot distinguish. `map_numeric`, `fiber_churn` and `fibers` cost
**2.3–2.8× more per instruction** than the tight-loop rows. Their instructions are
not more numerous; they are individually heavier (allocation, GC, hashing). Any
future cut aimed at them should target per-instruction work, not instruction count.

`bootstrap` is reported but **excluded from the spread**: at ~660 instructions its
ns/instr prices the *compiler* (~5 ms compiling `core.ph`), not the dispatch loop.
That it lands 1000× off the others is the tell that the row measures something else.

**Independent corroboration.** `bare_send` retires **16 instructions per send**
(3,200,683 ÷ 200,000). Net of the ~5 ms bootstrap, that derives **170 ns/send** —
against criterion's separately measured **~174 ns/send** (§3a). Two unrelated
instruments, 2.3% apart. This is the only cross-check in the harness where an
error in either would show up as disagreement.

**What the mean does not buy.** `wall / total` is a true mean over each program's
*executed mix*, not a price per opcode — a `Loop` and an `Invoke` land in the same
average. Pricing one opcode needs a differential (two programs differing by a known
count of a single opcode). The histogram makes that constructible; it does not
perform it. **Do not quote a row below as "the cost of `Invoke`".**

### 3c. Fixed costs

| Cost | Value | Measured at | Note |
|---|---|---|---|
| **Bootstrap** (`VM::new`, recompiles `core.ph`) | **~5 ms**, 6.8 MB RSS | **`2997d0b`** | On *every* process, every golden test, every criterion iteration. Confirmed at HEAD |
| Bootstrap, regressed | 180 ms | `3b2dd97`…`0274f10` | **35× regression, passed every gate** (F13) — nothing measured bootstrap |
| Bootstrap, pre-U-STRING | ~5 ms | `debadfa` | |
| `size_of::<Object>()` | **40 B** | post-`7480d75` | was 280 B (F7) |
| Fiber shell | 176 B + 2 buffers | `1e1b101` | before operand/frame `Vec` growth |
| Pool cost per fiber (**flag OFF, do not use**) | +~450 B/fiber | `ad4a215` | F10 — linear, measured negative |

### 3d. Compile time vs conditional nest depth (F13)

Source length held **linear**; compile time was **2^depth** before `0274f10`.

| nest depth | 8 | 10 | 12 | 14 | 16 | 18 | 20 | 26 |
|---|---|---|---|---|---|---|---|---|
| before `0274f10` (over baseline) | ~0 | 0.04 s | ~0 | 0.03 s | 0.17 s | 0.70 s | **2.8 s** | **70.9 s** |
| after `0274f10` | — | — | — | — | — | — | — | **0.022 s** |

A 20-line source method could hang the compiler for minutes. `--test lang`:
**122 s → 2.8 s**.

## 4. Function-level attribution — where the ticks go

Leaf-frame ("top of stack") tick counts, macOS `sample <pid> <secs>` @1 ms. These
are the rows that become `# Performance` rustdoc on the named function.

### 4a. `bare_send` (40M sends) @ `1e1b101`

| Function / mechanism | Ticks | Share |
|---|---|---|
| `hash_one` + `sip::Hasher::write` (**global name resolution**) | 317 | **~13%** |
| `vm::send::call_method` (arg-buffer build + frame setup) | 353 | **~14%** |
| `Value::class` | 92 | ~4% |

### 4b. `for` (4M) @ `1e1b101`

| Function / mechanism | Ticks | Share |
|---|---|---|
| `hash_one` + `sip::Hasher::write` | 168 | **~7%** |

### 4c. `skynet` @ `1e1b101` (post-U-GC, post-cut-004) — **GC-bound**

| Function / mechanism | Ticks | Share |
|---|---|---|
| `vm::dispatch::run_until_inner` (interpreter loop) | 259 | 35% |
| **`trace_object`** (GC mark) | 145 | **~20%** |
| malloc/free family | ~130 | ~17% |
| slotmap slot drop + `Heap::collect` | 53 | 7% |
| `vm::send::call_method` | 48 | 6% |
| `hash_one` | 13 | ~1% |

**GC family ≈ 30%**, and pre-`4f2eed8` nearly all of it was wasted: ~1.1M fibers all
live, every cycle traced everything and reclaimed ~nothing, then `GC_GROW_FACTOR=1.5`
re-triggered.

### 4d. Historical — origin profiles @ `757d88a` (kept for the delta)

`arith_send` ×20M (2,429 leaf ticks):

| Mechanism | Ticks | Share | Status |
|---|---|---|---|
| `vm::dispatch::run_until_inner` | 814 | 33.5% | — |
| malloc+free family (**per-send `Vec`**) | 478 | **19.7%** | **killed — cut 001** |
| `mach_absolute_time` (tracing span) | 445 | **18.3%** | **killed — cut 003** |
| Dispatch lookup (`IndexMap::get`, `hash_one`, `lookup_method_in_hierarchy`) | 338 | 13.9% | **cached — U-IC `f5e41f1`** |
| `number_add` / `number_lt` (primitive body) | 176 | 7.2% | irreducible-ish |
| `vm::send::call_method` | 116 | 4.8% | open — DEC-PRIM-B |
| `_platform_memmove` | 42 | 1.7% | — |

`skynet` @ `757d88a` (2,624 leaf ticks):

| Mechanism | Ticks | Share | Status |
|---|---|---|---|
| malloc+free family | 740 | **28.2%** | U-GC + cut 004 |
| `vm::dispatch::run_until_inner` | 727 | 27.7% | — |
| `_platform_memmove` (per-fiber `Vec` growth-realloc — **not** `mem::take`, F3) | 541 | **20.6%** | open |
| Dispatch lookup | 204 | 7.8% | U-IC |
| `vm::send::call_method` | 96 | 3.7% | open |
| `SlotMap::try_insert_with_key` (heap arena) | 58 | 2.2% | — |
| `close_upvalues_from` | 26 | 1.0% | — |
| `block_call` | 19 | 0.7% | cut 004 |

**Shares re-normalize as cuts land.** Cut 001 removed most of the malloc share and
003 removed 18.3%; every remaining share is inflated relative to these tables. Never
size a unit from an origin-profile percentage — re-profile first (design-notes O5).

## 5. Timeline — best result after each change

One row per landed cut. **Δ is against the immediately preceding commit**, matched
same-session A/B unless noted.

| Commit | Change | Best measured result | RSS | Golden diff |
|---|---|---|---|---|
| `757d88a` | U-BENCH Tier 0 harness + BASELINE | *origin*: skynet 13.7–15.6 s, **~19–20× Wren**; bare_send ~329 ns/send | 4.65–6.09 GB (**~7–9× Wren**) | — |
| `37f31c9` | **Cut 001** — on-stack arg buffer replaces per-send heap `Vec` | **arith_send −41.5%**, bare_send −33.8% | — | none |
| `ad4a215` | F5 fiber-pool — reverted | null result (code reverted, finding kept) | — | none |
| `7480d75` | **Cut 002** — box the six fat `Object` variants | `for.ph` **−43%** wall, skynet **−34%** wall (`sys` 2–4× less); `bare_send` **+5%** | wash | none |
| | | `size_of::<Object>()` **280 B → 40 B** | | |
| `1ef999b` | **Cut 003** — `vm-trace` feature gates the per-opcode span + 3 `debug!`s | **arith_send −16.7%** (whole-process); skynet ≈−1% `user` | — | none |
| `f5e41f1` | **U-IC** — monomorphic IC on `Invoke`, guarded `(class, world_version)` | method lookup no longer the top dispatch cost | — | none |
| `1531070` | **Cut 004** — share block-literal `Callable` via `Rc` | **skynet −30% `user`**, `sys` −94%; fiber_churn −16%. **Costs +5–7% on send-heavy** | **−63%** (3.73 → **1.37 GB**) | none |
| | | ⇒ skynet now **2.4–2.5 s / 1.44 GB** = **~3.2× wall / ~2.2× RSS** vs Wren | | |
| `debadfa` | Cut 004 Change 2 — memoize the variadic `name(*)` selector | **variadic_send −28%** (1.00 → **0.72 s**) | — | none |
| `3b2dd97` | *(U-STRING core rework)* | ⚠ **bootstrap 5 ms → 180 ms** — a 35× regression that passed every gate | — | — |
| `8ba87ec` | Harness repair — benches verify **answers**; `compare-wren.py`; `fiber_churn.ph` | found `method_call.ph` had silently stopped parsing | — | — |
| `0274f10` | **Inliner fix (F13)** — suppress inlining inside deopt-fallback copies | **bootstrap 0.18 s → 0.005 s** (~175 ms back to *every* process) | — | none |
| | | nest depth 26: **70.9 s → 0.022 s**; `--test lang` **122 s → 2.8 s** | | |
| `4f2eed8` | **Yield-adaptive GC (F11)** — grow `next_gc` 4× when yield <10%, else 1.5× | **skynet −11.7% `user`**; **fiber_churn −7.4%** | skynet **−8%**; fiber_churn **−15%** (450 → 420 MB) | none |
| | | `cargo test --workspace` **fully green** — first time; was red since ≥`bd3f492` | | |
| `2997d0b` | Handoff doc | — | — | — |
| `2c775ac` | SCOREBOARD + HEAD re-measure; F12 probe | *(docs only)* — closed holes H1/H2, unblocked F12 | — | — |
| **`39d9042`** | **F12 — per-callsite global-resolution cache, guarded by `ModuleObject.globals_version`** | **bare_send −17.9%**, arith_send −14.4%, **fiber_spawn −20.1%**, variadic_send −8.3%, **skynet −7.7% `user`**, **fiber_churn −21.4%**, `for` −3.6% | unchanged (1.322 GB skynet, 264 MB fiber_churn) | none |
| | | ⇒ per-send **~211 → ~174 ns**; skynet **2.9× Wren** — under 3× for the first time | | |

**Investigated, not landed** (do not re-litigate):

| Candidate | Verdict | Numbers |
|---|---|---|
| Fiber-stack pool (`fiber-pool` flag) | **NEGATIVE — flag stays, stays OFF, do not use** (owner ruling) | +72–86% RSS (linear ~450 B/fiber); **+37% `user`** at 1M fibers. Skynet a wash — which is why F5 saw nothing |
| U-HOTPATH Change 3 — reorder `Value::class` arms | dropped pre-landing | no measurable change; LLVM already ordered the match |
| `Option`/`Some` escape analysis | premise falsified | `List/Map.at` already zero-alloc |
| **F12 global slot cache (prototype)** | **measurement only — NOT landable** | **bare_send −15.3%, arith_send −14.2%, `for` −5.0%, method_call −3.6%** (net of F13's constant, bare_send ≈ **−18%**). Gets shadowing semantics **wrong** |

## 6. Open holes — what is empty, and how to fill it

Ranked by how much they distort the numbers above.

| # | Hole | Why it matters | How to fill |
|---|---|---|---|
| ~~**H1**~~ | ~~§1/§2 measured at `debadfa`, not HEAD~~ | **CLOSED `2997d0b`.** And the hole's own premise was **wrong**: `debadfa` predates F13's regression, so no row was inflated. HEAD ≈ `debadfa`, verified on all 9 rows | — |
| ~~**H2**~~ | ~~§3a per-send ns are the ORIGIN baseline~~ | **CLOSED `2997d0b`.** ~211 ns/send (bare), ~177 ns (arith), ~770 ns (spawn). Surfaced the bare/arith **inversion** vs origin | — |
| ~~**H3**~~ | ~~No per-opcode / per-instruction cost anywhere~~ | **CLOSED `45ffe76`** — `opcode-histogram` feature + `benchmarks/vm/opcode-cost.py`. **~10–13 ns/instruction**, spread 2.8× (§3bb). Option (a) as predicted; (b) would indeed have answered nothing | — |
| **H13** | **No per-opcode *price*.** §3bb gives a mean over each program's mix, not the cost of `Invoke` vs `GetLocal` | The histogram says `Invoke` is 13–25% of every hot program's mix but not what it costs. Sizing DEC-PRIM-B or the variadic-IC refill wants a price, not a share | Build a **differential**: two programs whose instruction counts differ by a known quantity of exactly one opcode, timed on a default build. The histogram makes the "known quantity" verifiable, which is what makes the subtraction sound. Least-squares over the 11 execution-bound rows is the cheaper alternative but is underdetermined at 35 opcodes — do not fit it and report the coefficients as prices |
| **H4** | **No fiber-churn ÷ Wren row.** `fiber_churn.ph` has no `.wren` twin | It is the *only* Phalcom fiber row with no Wren ratio, and high-turnover is the workload F5/F10 turned on. §3b's ~560 ns/fiber has nothing to divide by | Port `fiber_churn.ph` → `fiber_churn.wren` (12 lines; `Fiber.new{}`/`.call()` exist in both). Then per-spawn ns = wall ÷ 500,000 both sides |
| **H5** | **RSS is missing on all 9 wren-suite rows.** Only skynet/fiber_churn/bootstrap have it | Law: "a row without RSS is half a row." Cut 002 was nearly abandoned because RSS effects were invisible to the chosen instrument | `compare-wren.py` should shell `/usr/bin/time -l` and add peak-RSS + `sys` columns per row for both runtimes. **Partially closed**: skynet/fiber_churn/bootstrap now carry all four columns (§2) |
| **H12** | **fiber_churn's −37% RSS (420 → 264 MB) is unattributed** | It is a large memory win nobody claimed. F13's constant explains its −39% `user`, but **not** the RSS | A/B `0274f10` vs `4f2eed8` on fiber_churn RSS. Suspect adaptive GC (F11 measured −15% there); if it is bigger than that, something else moved |
| **H6** | **No CPython column.** BASELINE §1 records `not measured` deliberately (DEC-BENCH-B) | `performance.md` §3 makes CPython parity the intermediate checkpoint — currently unfalsifiable | Port `skynet.ph` → generators/`asyncio`, or nominate a different recursive/alloc microbenchmark as the checkpoint. Needs a ruling on which, not a silent pick |
| **H7** | **Bootstrap has no tripwire in the harness.** F13's 35× regression passed all three gates | It will happen again. `bootstrap.ph` exists (added `8ba87ec`) but nothing asserts on it | Add a whole-process `bootstrap.ph` assertion to `run.sh` with a ceiling (e.g. fail >20 ms). One line |
| **H8** | **No `sys` time column.** Cut 002's real signal was `sys` 2–4× less; cut 004's was `sys` −94% | Allocation/paging effects hide in `sys` and are invisible in `user` | `/usr/bin/time -l` already prints it — record `user`/`sys`/`real`/RSS as four columns, not one |
| **H9** | **`memmove` 20.6% on skynet (F3: per-fiber `Vec` growth-realloc) was never re-profiled** after cuts 001–004 | It was the largest non-hypothesized mechanism at origin and its current share is unknown | `sample` skynet at HEAD; if it survived, presizing fiber buffers is a ~10-line probe |
| **H10** | **`math/`, `rendering/mandelbrot`, `annotations/` benchmarks are in the tree but in no table** | `mandelbrot.wren` exists next to `mandelbrot.ph` — a free comparison row that nobody runs | Add them to `compare-wren.py`'s row list. `math/run.sh` exists; wire its output in or delete the directory |
| **H11** | **No variance discipline recorded per row.** Most rows are single-run or best-of-3 | Criterion certified noise at `p=0.00` twice; the same binary vs the same baseline reported +8.8% then +1.3% | Record N and spread per row. For effects <10%: alternating same-session A/B, read the **sign across pairs** |

### F12 — unblocked (probe run 2026-07-14 at `2997d0b`)

F12 (global slot cache, **−15% bare_send**) was blocked on a language ruling: may a
later `define` shadow a name an earlier callsite already resolved? **The probe the
handoff left unrun has now been run, and the ruling is not needed.** Measured
semantics at HEAD:

| Operation | Effect on a callsite's resolution | Cache impact |
|---|---|---|
| `var X = …` shadowing a **core** name | main slot created; value **changes** (`List` → `42`) | **the only stale case** |
| `class X {}` on a core name | **reopens** — `before == after` is `true`, same object | value identical; invisible |
| forward ref read **before** its define | **hard error** — `Undefined variable 'later'.` | no entry ever cached |
| redefinition (`var x = 2`) / assignment (`x = 3`) | `declare` returns the existing slot; slots append-only | **cache stays valid** |
| `X = v` with no main definition | error — `SetGlobal` has **no** core fallback | n/a |

**Reachability in real code: 0 of 672 `.ph` files.** All 4 core-name collisions
(`class Bool` ×2, `class Option`, `class ArgumentError`) are **reopenings** that
preserve object identity, not shadowings. `var`-shadowing of a core name occurs only
in synthetic probes.

**Two corrections to the handoff's reasoning** (its numbers were right; the inference
was not):

1. **Handoff claim (3) is wrong as argued.** "Forward references work and are
   ordinary… so late binding is not an exotic corner" — but a forward global read
   *before* its define **errors**. It works only when read *after*, at which point
   the callsite's first successful resolution is already main-module. **Forward
   references do not share the shadowing machinery** and cannot produce a stale entry.
2. **The bump condition is narrower than proposed.** The cache stores `(module, slot)`
   and reads *through* the slot, so `set_global` needs **no** invalidation and
   `define` on an existing name reuses its slot. Only a `define` that **allocates a
   new slot** can invalidate anything.

**Conclusion: no language ruling required.** A version-guarded cache preserves today's
semantics exactly, so "make shadowing illegal" is a semantics change bought for one
integer compare — it earns nothing. Implement the guard, keep late-binding. Spec Q4's
lean (kernel names as core exports the import system may re-scope) is preserved rather
than pre-empted.

---

## Maintaining this file

1. **Land a cut → add a §5 row in the same commit.** Δ vs the immediately preceding
   commit, matched same-session A/B, with RSS. A cut without a row is not landed.
2. **Any new ns/op or tick-share → §3 or §4, with commit + workload + instrument.**
3. **Re-measure §1/§2 whenever ≥2 cuts land**, and stamp the new commit at the top.
4. **When a number goes stale, mark it stale — never delete it.** The origin rows are
   what make the deltas legible.
5. **A number that survives two independent measurements graduates to rustdoc** on
   the function it describes, as a `# Performance` section citing its row here.
