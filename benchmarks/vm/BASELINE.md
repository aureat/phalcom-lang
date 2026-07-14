# Tier 0 performance baseline (U-BENCH)

Reproducible, attributed measurement backing the [performance strategy](../../docs/spec/v0.2/performance.md)
([ADR-0051](../../docs/adr/0051-performance-strategy-measure-first-tiered-optimization.md)).
This replaces the oral "~29× slower than Wren on Skynet" figure with a
measured, in-repo number, and gives every later tier (`U-HOTPATH`,
`U-PRIM-ABI`, `U-IC`, `U-GC`, `U-COMPILE`) a re-measurable baseline (law P1).

**Reproduce everything in this file:** `benchmarks/vm/run.sh` (whole-process
Skynet + micro-program execution gate + criterion) or
`cargo bench -p phalcom-core --bench vm_bench` (criterion only).

All numbers below: macOS (Darwin, this repo's dev machine), release build
(`cargo build -r`), **single run, not statistically rigorous** — the same
caveat `benchmarks/wren-suite/README.md` already carries for its numbers.
Wall-clock varied ~13.7–15.6 s for Phalcom Skynet across repeated runs on
this machine (background load, page-fault/GC-adjacent noise); the criterion
numbers below carry real confidence intervals (100 samples) and are the
more trustworthy figures for regression-tripwire purposes.

## 1. Skynet: Phalcom vs Wren vs CPython (whole-process)

Skynet ([`benchmarks/concurrency/skynet.ph`](../concurrency/skynet.ph),
ported from [`skynet.wren`](../concurrency/skynet.wren)) spawns 1,111,111
fibers in a depth-6, 10-way fan-out tree; `System.print`s the fixed checksum
`499999500000`. Measured with `/usr/bin/time -l` (whole-process wall-clock +
peak RSS — performance.md §2: "whole-process lifetime is the unit of
measurement", not steady-state throughput).

| Runtime | Wall-clock (real) | Peak RSS | Provenance |
|---|---|---|---|
| Phalcom (release) | **13.7–15.6 s** | **4.65–6.09 GB** | measured, this run, `target/release/phalcom` |
| Wren (release, real fibers) | **0.68–0.79 s** | **~667 MB** | measured, this run, `~/dev/repos/wren/bin/wren_test` (local checkout build) |
| CPython | not measured | not measured | Skynet's fiber-fan-out has no committed CPython port in this repo (would need a generator/coroutine translation); out of this unit's write-set (`benchmarks/vm/` + `benches/` only). See [DEC-BENCH-B](#dec-bench-b) — no number is invented. |

**Slowdown vs Wren: ~19–20× wall-clock, ~7–9× peak RSS**, taking the
run.sh figures (13.70 s / 4.65 GB vs 0.68 s / 0.667 GB) as representative.
This *revises* the oral "~29×" down to a measured **~19–20×** — the oral
figure was apparently a single worse-case run (or a different machine/build);
either way it is now superseded by this reproducible number, not asserted
from memory.

CPython parity is `performance.md` §3's intermediate checkpoint, expected
from Tiers 1–3; a CPython Skynet port (or an equivalent recursive/allocation
microbenchmark) is left for whichever later tier first wants that
comparison point — recording "not measured" here rather than fabricating a
number (DEC-BENCH-B).

## 2. Criterion micro-benches (send / arith / fiber)

`phalcom-core/benches/vm_bench.rs`, `cargo bench -p phalcom-core --bench
vm_bench`. Each iteration runs one whole program
(`Interpreter::new()` + `interpret_source`) to completion — bootstrap
(`VM::new`'s `core.ph` recompile, Tier 5's target) is *not* isolated out,
but is a small fraction of each program's total loop-bound time (confirmed:
`bare_send.ph`/`arith_send.ph` each run 200,000 iterations; `fiber_spawn.ph`
runs 20,000).

| Benchmark | Program | Sends | Mean time | Per-send |
|---|---|---|---|---|
| `bare_send` | [`bare_send.ph`](bare_send.ph) — static, argument-free user-method send | 200,000 | **65.7 ms** | ~329 ns |
| `arith_send` | [`arith_send.ph`](arith_send.ph) — primitive `1 + 2` send | 200,000 | **72.1 ms** | ~361 ns |
| `fiber_spawn` | [`fiber_spawn.ph`](fiber_spawn.ph) — `Fiber.new{}` + `.call()` + `Fiber.yield` | 20,000 | **24.2 ms** | ~1.21 µs |

(Criterion CI: `bare_send` [64.6, 67.1] ms; `arith_send` [68.2, 77.7] ms;
`fiber_spawn` [23.0, 25.9] ms, 100 samples each.)

**Caveat on the send/arith delta:** `bare_send` dispatches to a
*user-defined* method (full `CallFrame` push + bytecode body execution of
`return 0`), while `arith_send` dispatches to a *native primitive*
(`number_add`, no frame push, but a per-call `Vec<Value>` allocation,
`vm.rs:626`). The two are not a clean decomposition of "dispatch tax alone"
vs "allocation tax alone" — they isolate *allocation-bound* vs
*dispatch/frame-bound* sends of comparable mechanism weight, which is
exactly what the attribution profile below breaks down further by mechanism
rather than by these two programs alone.

## 3. Attribution profile (by mechanism)

Captured with macOS's built-in sampling profiler (`sample <pid> <secs>`,
1 ms interval) on two workloads: a 20,000,000-iteration extension of
`arith_send.ph` (long enough for a stable 3 s sample) and Skynet itself
(sampled mid-run). Leaf-frame ("top of stack") tick counts, which
approximate CPU-time share per mechanism.

### 3a. Arithmetic loop (`1 + 2` × 20M, dispatch- and allocation-bound)

| Mechanism | Symbol(s) | Ticks | Share |
|---|---|---|---|
| Interpreter loop overhead | `vm::dispatch::run_until_inner` | 814 | 33.5% |
| **Tracing span / timestamp** | `mach_absolute_time` | **445** | **18.3%** |
| **Per-send allocation** (malloc+free family) | `_xzm_xzone_malloc_tiny`, `_malloc_zone_malloc`, `_xzm_free`, `_free`, `_xzm_xzone_malloc`, misc | **478** | **19.7%** |
| Dispatch lookup (hash probe + resolution) | `IndexMap::get`, `hash_one`/`Hasher::write`, `lookup_method_in_hierarchy`, `Value::lookup_method`, `Value::class` | 338 | 13.9% |
| `call_method` mechanics (arg-vec build, frame setup) | `vm::send::call_method` | 116 | 4.8% |
| Primitive body itself | `number_add`, `number_lt` | 176 | 7.2% |
| memmove | `_platform_memmove` | 42 | 1.7% |

(2,429 total leaf ticks.)

### 3b. Skynet (mid-run, fiber- and allocation-bound)

| Mechanism | Symbol(s) | Ticks | Share |
|---|---|---|---|
| Interpreter loop overhead | `vm::dispatch::run_until_inner` | 727 | 27.7% |
| **memmove** (fiber stack/container churn — `fiber.rs` `mem::take` of operand/frame `Vec`s) | `_platform_memmove` | **541** | **20.6%** |
| **Allocation** (malloc+free family, incl. new fiber objects) | `_xzm_xzone_malloc_tiny`, `_xzm_xzone_malloc_freelist_outlined`, `_malloc_zone_malloc`, `_xzm_free`, `_free`, misc | **740** | **28.2%** |
| Dispatch lookup | `IndexMap::get`, `hash_one`/write, `lookup_method_in_hierarchy`, `Value::lookup_method`, `Value::class` | 204 | 7.8% |
| `call_method` mechanics | `vm::send::call_method` | 96 | 3.7% |
| Heap arena insert (**unbounded heap** — new object slot allocation) | `slotmap::basic::SlotMap::try_insert_with_key` | 58 | 2.2% |
| Closure/upvalue teardown | `vm::dispatch::close_upvalues_from` | 26 | 1.0% |
| Block invocation | `primitive::block::block_call` | 19 | 0.7% |
| Module/symbol bookkeeping | `BTreeMap` insert/remove (interner or module registry) | 47 | 1.8% |

(2,624 total leaf ticks. No `mach_absolute_time` samples in the top 30 —
tracing overhead's *share* is much smaller here because each send does far
more other work per unit wall-clock time; the fixed per-opcode tracing cost
is diluted, not absent.)

## 4. Ratify or re-rank? — `performance.md` §2's suspect order

§2 orders the suspects (Part II / the tier sequence) roughly: **tracing
span → per-send `Vec` allocation (args) → dispatch lookup (`IndexMap`
probe) → unbounded heap**. This profile:

- **Ratifies**: every one of the four is real and measurable — none is a
  phantom. `mach_absolute_time` alone is 18.3% of samples on the arithmetic
  loop; the malloc+free family is 20–28% on both workloads; the dispatch
  hash probe is a consistent 8–14%; the heap-arena insert is measurable and
  directly explains Skynet's peak RSS gap (Phalcom 4.65 GB vs Wren's
  0.667 GB — the unbounded heap **is** the RSS story, even though its raw
  CPU-tick share is small: it's a memory-footprint cost, not primarily a
  cycle cost, which is exactly why a throughput-only harness would have
  missed it — the reason performance.md §2 mandates peak-RSS measurement).

- **Re-ranks**: **per-send allocation (malloc+free) is the single largest
  attributable mechanism on both workloads (19.7% arithmetic-loop, 28.2%
  Skynet)** — larger than the tracing span and larger than dispatch lookup.
  Dispatch lookup itself is smaller than the oral hypothesis implied
  (13.9% arithmetic-loop, 7.8% Skynet) — the `IndexMap` hash probe is real
  but not dominant. This **numerically confirms ADR-0051's explicit
  rejection of "dispatch-first ordering"**: rewriting `lookup_method_in_hierarchy`
  before touching allocation would have targeted a ~8–14% mechanism while
  leaving a ~20–28% one untouched. The committed tier sequence — Tier 1
  (tracing-gate, cheap) then Tier 2 (kill allocation) then Tier 3
  (dispatch/IC) — remains the right order; if anything this data makes the
  case for Tier 2 *slightly stronger relative to Tier 1* than the oral
  narrative suggested, since allocation outweighs tracing on the
  fiber-heavy (i.e. more Skynet-representative) workload.

- **New, non-hypothesized finding**: `memmove` is 20.6% of Skynet's samples
  — not named in §2's suspect list at all. This is consistent with (but not
  proof of) the fiber operand/frame `Vec` churn `mem::take`s around every
  fiber switch (`fiber.rs:29-51`) copying/moving container contents;
  ADR-0051 already rules out the *switch* itself as the target (it's O(1)),
  but this suggests the *containers moved during* switch-adjacent sends are
  a real, previously unattributed cost worth a closer look in Tier 2/4 —
  flagged here, not investigated further (out of this unit's scope).

## 5. Write-set / floor confirmation

- No `phalcom-core/src/*.rs` runtime file touched. Confirmed: `git diff --stat`
  against this unit's changes touches only `phalcom-core/Cargo.toml` (dev-dependency +
  `[[bench]]` stanza), `phalcom-core/benches/vm_bench.rs` (new), and
  `benchmarks/vm/*` (new).
- No golden `.ph` corpus change. Floor: **+0**.
