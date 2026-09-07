# Phalcom Benchmarks source snapshot

> Source: Local repository snapshot at `benchmarks/`
> Collected: 2026-09-07
> Published: Unknown

> Note: This snapshot captures the benchmark documentation and result/catalog schemas used by the article; executable corpus programs and committed history records remain in the source directory.

## `benchmarks/annotations/README.md`

~~~~markdown
# Annotation showcase (design-only, **not** a runnable benchmark)

This directory exists to show the **full Phalcom annotation surface** in one
place, documented with [Phaldoc](../../docs/spec/experimental/doc-comments-phaldoc.md).
It is deliberately **not** under [`benchmarks/math/`](../math/), because
[`run.sh`](../math/run.sh) globs every `*.ph` in that directory and would try to
execute these files — which cannot lex today.

## Why it doesn't run

Two independent things are unbuilt on the current tree:

1. **Code-level `@`-attributes** (`@requires`, `@construct`, `@data`, …) need
   `Token::At` in the lexer and the desugar pass
   ([annotations-core.md](../../docs/spec/experimental/annotations-core.md)) — all
   still *Proposed*. A `@requires(x > 0)` line lexes as `/` `@` … garbage today.
2. **The Phaldoc doc generator** (`phalcom doc`) does not exist yet. The `///` /
   `//!` comments *are* inert (they are `//`-prefixed trivia — see
   [`lexer.rs:88`](../../phalcom-ast/src/lexer.rs)), but nothing harvests them.

So [`showcase.ph`](showcase.ph) is a **visual specimen**: it is what idiomatic,
fully-annotated Phalcom is intended to look like once both land. Do not add it to
CI or `run.sh` until the `@` lexer is in.

## The one rule it demonstrates

Phaldoc §8.1 — **prose and attributes never restate each other**:

- `///` prose says *what and why* (intent a machine can't infer).
- `@…` attributes say the *checkable facts* (which inputs are legal, what's
  guaranteed, what's generated).

`deposit`'s `///` never says "amount must be positive" — that fact lives only in
`@requires(amount > 0)`, and the doc tool harvests it (Eiffel's contract view).

## What `phalcom doc` would render for `deposit(_)`

Harvesting the two layers per Phaldoc §8.2 produces:

```
BankAccount ▸ deposit(_)
  Add funds.

  Requires   amount > 0                              [checked: debug+release]
  Ensures    _balance == old(_balance) + amount      [checked: debug]
  Invariant  _balance >= 0                            (class, all public methods)
  Raises     PreconditionError   — precondition violated   (derived)
             PostconditionError  — postcondition violated  (derived)
  Example
             let a = BankAccount.opened(100)
             a.deposit(50)          // ok
             a.deposit(0 - 1)       // raises PreconditionError
```

Every line under `Requires`/`Ensures`/`Invariant`/`Raises` is **harvested**, not
authored — the author wrote only the summary and the `@example`. The
`[checked: …]` badges come from the compile-mode table
([annotations-contract-semantics.md](../../docs/spec/experimental/annotations-contract-semantics.md)):
`@ensures` is stripped in `release`, so the doc says so; the *specification* is
shown regardless because contract metadata is retained in every mode
(D-contract-1).

## Files

| File | Shows |
|---|---|
| [`showcase.ph`](showcase.ph) | `@requires`/`@ensures`/`@invariant` (DbC), `@construct` + `@get`/`@set` (layout), `@data`/`@sealed`/`@variant` (ADTs + exhaustive `match`), `@observable`/`@computed` (reactive) — each wrapped in Phaldoc `///`/`//!` |
~~~~

## `benchmarks/math/README.md`

~~~~markdown
# Math benchmark corpus

A set of **self-verifying** Phalcom programs that stress the language with real
mathematics. They are deliberately **not** wired into `cargo test` — they are a
staging corpus, meant to be promoted into the golden suite as implementation
phases land. Each program checks **mathematical identities** (e.g. `sin²+cos²=1`,
`φ²=φ+1`, `gcd·lcm=a·b`) rather than hardcoded decimals, so a passing run is
genuine evidence of correctness, not just "it printed something."

Every program prints a column of `true` values (one per assertion). Any `false`
is a failure and names the broken identity in the adjacent comment.

## Running

The workspace tree must build first. Run one file directly:

```sh
cargo run -p phalcom-core --bin phalcom -- benchmarks/math/<file>.ph
```

Or run the whole corpus with the bundled harness, which builds the CLI once and
asserts that every output line is `true`:

```sh
benchmarks/math/run.sh            # all files; Tier-2 files report PENDING, not FAIL
benchmarks/math/run.sh --strict   # also require Tier-2 files to pass
benchmarks/math/run.sh vectors.ph # only the named file(s)
```

A file PASSES iff it exits 0 and prints nothing but `true`. Tier-2 files (needing
`var`/collections) are expected to fail today, so they surface as `PENDING` and
don't fail the run until `--strict`.

> As of this writing the working tree is mid-U6 and does not compile; the last
> green commit exercising the Tier-1 feature set is `83c908a` (U5). These files
> are **unexecuted drafts** — expect to fix a selector name or two on first run,
> especially in the Tier-2 files, whose stdlib surface is still deferred.

## The programs

| File | Tier | Requires | What it tests |
|---|---|---|---|
| `numeric_core.ph` | 1 | U5 | Newton `sqrt`, Euclid `gcd`/`lcm`, factorial (rec vs iter), fast exponentiation, trial-division primality |
| `transcendental.ph` | 1 | U5 | Taylor `exp`/`sin`/`cos`/`atan`; Machin's π; Pythagorean & exponential-addition identities |
| `integration.ph` | 1 | U5 | Composite Simpson's rule generic over a **block-valued** integrand; π via `∫4/(1+x²)`; exactness on cubics; linearity |
| `number_theory.ph` | 1 | U5 | Fast modular exponentiation, Fermat's little theorem, perfect/abundant numbers, Collatz step counts, π(30) |
| `continued_fractions.ph` | 1 | U5 | Fixed-point iteration of continued fractions for φ and √2, checked against their defining identities |
| `rationals.ph` | 1.5 | U5 (object model) | Exact `Rational` class: operator methods `+ - * / ==`, gcd reduction, telescoping sums — exact `==` is legitimate here |
| `monte_carlo.ph` | 1 | U5 | In-language MINSTD PRNG; π by darts & by 3D ball, `e` by expected-crossing, LLN (mean→½, var→1/12), MC integration of a block-valued `f` |
| `random_walk.ph` | 1 | U5 | Random walks off the same PRNG: unbiasedness `E[S_n]→0`, 1D/2D diffusion `E[S_n²]→n`, fair-coin rate |
| `stats.ph` | 2 | U6 `var` + collections + stdlib | mean/variance/stddev/median over a list; two variance formulas cross-checked; shift-invariance |
| `vectors.ph` | 2 | U6 `var` + collections + stdlib | dot/norm/cosine; `dot(a,a)=‖a‖²`, Cauchy–Schwarz, orthogonality + Pythagoras |

### Tiers

- **Tier 1** — pure recursion / closures / control flow / arithmetic. Every
  construct was observed working at U5 (`83c908a`). These should run today on a
  buildable tree and are the first candidates for promotion into the golden set.
- **Tier 1.5** — adds the object model (instance fields, operator methods,
  static factories). Also U5-era; grouped separately only because exact rational
  arithmetic makes `==` assertions meaningful.
- **Tier 2** — needs `var` (U6, [ADR-0014](../../docs/adr/accepted/0014-let-and-var-bindings.md)),
  list/map literals ([lexical-structure.md §4/§6](../../docs/spec/lexical-structure.md)),
  the iteration protocol ([iteration-protocol.md](../../docs/spec/experimental/iteration-protocol.md)),
  and a standard-library collection surface that is **still deferred**
  ([typing-stdlib-surface.md](../../docs/spec/experimental/typing-stdlib-surface.md)).
  Selector names (`reduce`, `sorted`, `map`, list `+`) are best-effort guesses at
  the eventual surface and should be reconciled when U-STD is specified.

## Promotion checklist (per file, when its tier lands)

1. Run it; confirm the output column is all `true`.
2. Move it under `phalcom-core/tests/lang/math/` with the standard header
   (`// area:`, `// spec:`, `// status: PASS`).
3. Capture a golden snapshot of stdout so regressions are caught.
4. Delete the corresponding row above (or mark it **LANDED**).
~~~~

## `benchmarks/results/README.md`

~~~~markdown
# Benchmark Results Store

This directory contains durable performance history and baseline indexes for the Phalcom VM.

- `baselines.json` maps named baselines (e.g. `main`) to specific run IDs in `history/`.
- `history/` stores committed `BenchmarkRun` JSON records.
- `schema-v1.json` documents the Schema v1 layout for `BenchmarkRun` and `ComparisonRun`.
~~~~

## `benchmarks/results/baselines.json`

~~~~json
{
  "baselines": {
    "main": "value16-ab_cand",
    "pre-value16": "value16-ab_base"
  }
}
~~~~

## `benchmarks/results/schema-v1.json`

~~~~json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Phalcom BenchmarkRun Schema v1",
  "type": "object",
  "required": [
    "schema_version",
    "run_id",
    "timestamp",
    "git",
    "build",
    "host",
    "layouts",
    "command",
    "suite",
    "resource_quality",
    "cases",
    "summary"
  ],
  "properties": {
    "schema_version": { "type": "integer", "const": 1 },
    "run_id": { "type": "string" },
    "timestamp": { "type": "integer" },
    "resource_quality": { "type": "string", "enum": ["full", "wall_only"] }
  }
}
~~~~

## `benchmarks/suites/representation.json`

~~~~json
{
  "schema_version": 1,
  "name": "representation",
  "cases": [
    {
      "id": "vm/bare_send",
      "path": "benchmarks/vm/bare_send.ph",
      "tags": ["dispatch", "quick"],
      "heavy": false,
      "default_samples": 5,
      "default_warmup": 1,
      "verification": {
        "kind": "stdout_exact",
        "expected": "0"
      }
    },
    {
      "id": "vm/arith_send",
      "path": "benchmarks/vm/arith_send.ph",
      "tags": ["dispatch", "quick"],
      "heavy": false,
      "default_samples": 5,
      "default_warmup": 1,
      "verification": {
        "kind": "stdout_exact",
        "expected": "3"
      }
    },
    {
      "id": "vm/rest_fallback_send",
      "path": "benchmarks/vm/rest_fallback_send.ph",
      "tags": ["dispatch"],
      "heavy": false,
      "default_samples": 5,
      "default_warmup": 1,
      "verification": {
        "kind": "stdout_exact",
        "expected": "6000000"
      }
    },
    {
      "id": "vm/fiber_spawn",
      "path": "benchmarks/vm/fiber_spawn.ph",
      "tags": ["fibers"],
      "heavy": false,
      "default_samples": 5,
      "default_warmup": 1,
      "verification": {
        "kind": "stdout_exact",
        "expected": "0"
      }
    },
    {
      "id": "wren/for",
      "path": "benchmarks/wren-suite/for.ph",
      "tags": ["list", "memory"],
      "heavy": false,
      "default_samples": 5,
      "default_warmup": 1,
      "verification": {
        "kind": "stdout_exact",
        "expected": "499999500000"
      }
    },
    {
      "id": "wren/map_numeric",
      "path": "benchmarks/wren-suite/map_numeric.ph",
      "tags": ["map", "memory"],
      "heavy": false,
      "default_samples": 5,
      "default_warmup": 1,
      "verification": {
        "kind": "stdout_exact",
        "expected": "2000001000000"
      }
    },
    {
      "id": "wren/fib",
      "path": "benchmarks/wren-suite/fib.ph",
      "tags": ["numeric"],
      "heavy": false,
      "default_samples": 5,
      "default_warmup": 1,
      "verification": {
        "kind": "stdout_exact",
        "expected": "317811\n317811\n317811\n317811\n317811"
      }
    },
    {
      "id": "concurrency/skynet",
      "path": "benchmarks/concurrency/skynet.ph",
      "tags": ["fibers", "heavy"],
      "heavy": true,
      "default_samples": 3,
      "default_warmup": 1,
      "verification": {
        "kind": "stdout_exact",
        "expected": "Result: 499999500000"
      }
    }
  ]
}
~~~~

## `benchmarks/vm/BASELINE.md`

~~~~markdown
# Tier 0 performance baseline (U-BENCH)

Reproducible, attributed measurement backing the [performance strategy](../../docs/spec/current/performance.md)
([ADR-0051](../../docs/adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)).
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
~~~~

## `benchmarks/wren-suite/README.md`

~~~~markdown
# Wren benchmark suite, ported

Direct ports of `wren/test/benchmark/*.wren` (from the sibling
`/Users/altunhasanli/dev/repos/wren` checkout) to Phalcom, for cross-language
throughput comparison. Every file's printed output matches its Wren original
exactly (cross-checked below) — these are correctness-verified, not just
"it ran."

Not ported: `api_call.wren`/`api_foreign_method.wren` (exercise the embedder C
API, not expressible as a standalone script), `fannkuch.*` (no `.wren` source
in the benchmark dir — only lua/py/rb), `delta_blue.wren` (blocked — needs
`List#removeAt`/stack-pop and index-write `list[i]=`, neither of which exist
on Phalcom's `List` today; see Porting notes).

## Running

```sh
cargo build -r -p phalcom-core --bin phalcom
time ./target/release/phalcom benchmarks/wren-suite/<file>.ph
```

## Results (release build, this machine, best-of-3 — not statistically rigorous)

Re-measured 2026-07-14 at `debadfa` (after U-PRIM-ABI cut 001, U-GC Win A cut
002, U-TRACE cut 003, U-GC's collector, and U-HOTPATH). The `(was …)` column
is the original measurement this table carried, taken before those cuts; every
row improved, `for` and `map_string` by an order of magnitude. Each Phalcom
run's stdout is compared byte-for-byte against its Wren original's (minus
Wren's `elapsed:` line) — a row is only reported if the output matches, since
a benchmark that computes the wrong answer is not a measurement.

| Benchmark | Wren | Phalcom | Slowdown | (was) | Output matches |
|---|---|---|---|---|---|
| `fib.wren` (fib(28) ×5) | 0.18s | 0.87s | 4.8x | 8.6x | ✓ 317811 ×5 |
| `for.wren` (1M list build+sum) | 0.05s | 0.73s | 13.6x | ~144x | ✓ 499999500000 |
| `fibers.wren` (100k chained) | 0.03s | 0.13s | 4.0x | ~12x | ✓ 4999950000 |
| `method_call.wren` (2M dispatch) | 0.09s | 0.53s | 5.7x | 8.2x | ✓ true / false |
| `string_equals.wren` (10M compares) | 0.11s | 1.10s | 10.0x | 17x | ✓ 3000000 |
| `binary_trees.wren` | 0.18s | 0.85s | 4.9x | 7.3x | ✓ all check lines |
| `binary_trees_gc.wren` | 0.56s* | 0.84s | 1.5x* | 2x* | ✓ all check lines |
| `map_numeric.wren` (2M map ops) | 0.86s | 3.85s | 4.5x | 5.9x | ✓ 2000001000000 |
| `map_string.wren` (~193k-key map) | 0.10s | 0.71s | 6.9x | 46x | ✓ 12799920000 |

\* `binary_trees_gc.wren`'s Wren time includes explicit `System.gc()` calls
(no Phalcom equivalent exists — dropped, see the file's header comment), so
this row is not apples-to-apples; Phalcom's number here is really closer to
plain `binary_trees.ph` (1.25s vs 1.32s, consistent).

`for.wren` was the outlier at 144x and is now 13.6x — the gap was allocation
and object size (cuts 001/002), not the list protocol, which is why no
`List`-specific unit was ever needed. `map_string`'s 46x → 6.9x collapsed the
same way. The suite now sits in a **4–14x** band with `for` and
`string_equals` at the top; the remaining spread, not any single row, is what
Tier 3 (U-IC) should be sized against.

**Re-measure with** `benchmarks/vm/compare-wren.py` (best-of-N, output-verified
against Wren, per-row slowdown), or run a single row by name.

## Porting notes (surface gaps hit, translated around)

- **No range-literal parser production.** `..`/`...` are lexed
  (`DotDot`/`DotDotDot`) but never consumed by an expression production —
  `for (i in 0...n)` is a hard parse error. Every range-based `for` below is
  a `while`-counter instead. (Same finding as skynet/mandelbrot.)
- **No postfix `[]` index operator**, read or write — only `[a, b, c]` list
  *literal* syntax exists. `list[i]` → `list.at(i)`; `map[k] = v` →
  `map.at(k, put: v)`; `map[k]` → `map.at(k)`.
- **`{}` is the empty-block literal, not empty-map** (spec §6 — Phalcom's
  own doc comment says so explicitly). `var map = {}` → `var map = Map.new()`.
- **Multi-line list literal parser bug**, found porting `map_string.ph`:
  `[a, b\n]` (no trailing comma before the newline-then-`]`) is a hard parse
  error; `[a, b,\n]` (trailing comma) parses fine. Every multi-line list in
  `map_string.ph` carries an added trailing comma to route around it. Looks
  like `parse_comma_exprs` skips newlines after a comma but not before the
  closing-bracket check — worth a real fix, not just a workaround.
- **`this` → `self`.** Phalcom's self-reference keyword.
- **`is` → `extends`** for inheritance; bare `super(args)` constructor call
  → `super.new(args)` (Phalcom dispatches constructors by selector, so the
  super initializer needs naming).
- **`%(expr)` → `\(expr)`** string interpolation (ADR-0022).
- **`System.clock` unimplemented** (pending fixture) — timing done via shell
  `time`, not in-language; every `elapsed:` print line is dropped.
- **No `System.gc()`** — dropped from `binary_trees_gc.ph` (see caveat above).
- Everything else — `_field` instance vars, `@constructor
new(...)`, getters
  (`value => _state` / `value { _state }`), setters (`name=(value) { }`),
  `super.propName` (no-paren super property read), closures, `Fiber.new{}`/
  `.call()`, `while`, string `+` concat, `List.new().add(...)`, `Map`'s
  `.at`/`.at(_,put:)`/`.remove`/`.includes` — ported with **zero** semantic
  change from the Wren source.
~~~~


