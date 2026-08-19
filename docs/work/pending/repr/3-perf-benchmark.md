# Implementation Specification 3 — Reproducible Performance Measurement, Structured Benchmark History, Comparison Tooling, and Ergonomic Benchmark CLI

**Status:** Ratified
**Applies after:** Implementation Specifications 1 and 2
**Primary implementation:** existing `phalcom-perf` tool
**Purpose:** make performance changes measurable, comparable, reviewable, and durable

# 1. Objective

Turn the existing collection of useful but partially separate Phalcom performance instruments into one coherent benchmark system.

The system must answer four different questions without conflating them:

1. **Is the program still correct?**
2. **How fast is this build on representative whole-process workloads?**
3. **How much memory/system work does it consume?**
4. **Did a candidate change improve or regress relative to a comparable baseline?**

The result must be useful to both:

- a developer running benchmarks interactively;
- an implementation/review agent inspecting machine-readable historical results.

The benchmark infrastructure must preserve the repository's existing measurement principles:

- whole-process performance matters;
- an erroneous benchmark is not a valid measurement;
- never invent unavailable numbers;
- microbenchmarks and real workloads answer different questions;
- build steps never occur inside the timing loop;
- small effects require alternating guarded A/B measurement rather than faith in a single run;
- contaminated measurements are discarded, not “corrected.”

---

# 2. Current state to preserve and evolve

The repository already has four useful pieces.

## `phalcom-core/bin/perf/main.rs`

Current `phalcom-perf`:

- runs acceptance corpus and benchmark programs;
- times via `Instant`;
- checks correctness;
- writes JSON-lines under `target/perf-logs`;
- reports per-label timings.

Keep the correctness logic and CLI identity.

Replace the single-file implementation with modular internals.

## `scripts/bench.sh`

This is already the ergonomic top-level developer entry point.

Keep:

```bash
scripts/bench.sh ...
```

as the canonical shell entry point.

## `benchmarks/vm/run.sh`

This already encodes:

- release-build requirement;
- bootstrap tripwire;
- whole-process Skynet timing;
- Criterion lane.

Do not silently discard these concepts.

## `benchmarks/vm/ab-guarded.py`

This contains a particularly valuable measurement rule:

```text
if machine contention invalidates an A/B run,
abort the run and emit no believable result.
```

Do not replace this with “more repetitions will average it out.”

---

# 3. Context-budget contract for the implementing agent

The benchmark work is deliberately isolated so the implementation agent does not need to reread runtime internals.

## Initial reads only

Read:

```text
phalcom-core/bin/perf/main.rs
scripts/bench.sh
benchmarks/vm/run.sh
benchmarks/vm/ab-guarded.py
benchmarks/vm/BASELINE.md:1–180
phalcom-core/benches/vm_bench.rs
phalcom-core/Cargo.toml:1–110
Cargo.toml:workspace.dependencies
```

Read the benchmark program source only for cases explicitly listed in the new suite manifest.

Do not reread:

```text
vm/send.rs
vm/dispatch.rs
value/*
heap/*
compiler/*
```

for benchmark implementation.

The benchmark tool links `phalcom-core` and can directly ask Rust for representation sizes.

## Compaction

Before writing code, compact those source reads into:

```text
current CLI
current correctness model
current A/B law
current whole-process law
current Criterion lane
new schema/CLI requirements from this spec
```

Then implement from that compact state.

---

# 4. Do not create a new benchmark crate

Keep the existing binary:

```text
phalcom-perf
```

inside `phalcom-core`.

The benchmark tool is already established and avoids an unnecessary workspace-level migration.

Refactor its implementation into modules.

## New files

Create:

```text
phalcom-core/bin/perf/model.rs
phalcom-core/bin/perf/env.rs
phalcom-core/bin/perf/measure.rs
phalcom-core/bin/perf/suite.rs
phalcom-core/bin/perf/store.rs
phalcom-core/bin/perf/compare.rs
```

Keep:

```text
phalcom-core/bin/perf/main.rs
```

as CLI/orchestration only.

Target approximate responsibility:

| Module | Responsibility |
|---|---|
| `model.rs` | serialized schema and aggregate statistics |
| `env.rs` | git/build/host/layout metadata |
| `measure.rs` | whole-process execution/resource measurement |
| `suite.rs` | benchmark discovery, manifests, correctness |
| `store.rs` | persistent run storage, promotion, baseline index |
| `compare.rs` | comparison, paired A/B analysis, regression policy |
| `main.rs` | clap parsing and command dispatch |

Do not build a generic framework. These modules exist specifically for Phalcom.

---

# 5. Dependencies

Use `serde` and `serde_json` for durable machine-readable results.

At workspace root add:

```toml
serde_json = "1"
```

to `[workspace.dependencies]`.

In `phalcom-core/Cargo.toml`, add the required direct dependencies.

If practical without excessive Cargo restructuring, gate result-serialization-only dependencies behind a `perf-tools` feature and make `phalcom-perf` require it.

If that complicates the existing binary substantially, use ordinary direct dependencies; correctness and maintainability are more important than saving one already-small tooling dependency graph.

Do not hand-build JSON strings as the current runner does.

---

# 6. New CLI model

The canonical user surface becomes:

```bash
scripts/bench.sh perf <command> ...
```

Commands:

```text
run
ab
compare
show
list
layout
baseline
```

## 6.1 `run`

Examples:

```bash
scripts/bench.sh perf run
```

Run the default representation/performance suite.

```bash
scripts/bench.sh perf run --suite representation
```

Explicit suite.

```bash
scripts/bench.sh perf run \
  --suite representation \
  --samples 9 \
  --warmup 1
```

```bash
scripts/bench.sh perf run \
  --case vm/bare_send
```

```bash
scripts/bench.sh perf run \
  --heavy
```

Include expensive workloads such as Skynet/map stress.

```bash
scripts/bench.sh perf run \
  --record \
  --name post-value16
```

Persist as durable repository history instead of local-only results.

### Required `run` options

```text
--suite NAME
--case FILTER
--samples N
--warmup N
--heavy
--binary PATH
--record
--name NAME
--top N
--json
```

Preserve current corpus-oriented options where useful:

```text
--corpus-only
--bench-only
--pending
--label
```

Do not break existing developer muscle memory unnecessarily.

## 6.2 `ab`

Canonical representation-change command:

```bash
scripts/bench.sh perf ab \
  --baseline-bin /tmp/phalcom-before \
  --candidate-bin target/release/phalcom \
  --suite representation \
  --pairs 15
```

Fast smoke:

```bash
scripts/bench.sh perf ab \
  --baseline-bin /tmp/phalcom-before \
  --candidate-bin target/release/phalcom \
  --suite representation \
  --quick \
  --pairs 7
```

This must alternate execution order.

For pair index:

```text
even pair:
    baseline -> candidate

odd pair:
    candidate -> baseline
```

Builds are prepared **before** entering `ab`.

`phalcom-perf` must never call Cargo from inside an A/B measurement loop.

## 6.3 `compare`

Compare stored runs:

```bash
scripts/bench.sh perf compare \
  baseline:value-pre16 \
  latest
```

or:

```bash
scripts/bench.sh perf compare \
  20260816T...-abc123 \
  20260817T...-def456
```

Machine output:

```bash
scripts/bench.sh perf compare \
  baseline:value-pre16 \
  latest \
  --json
```

Optional gate:

```bash
scripts/bench.sh perf compare \
  baseline:main \
  latest \
  --gate
```

## 6.4 `list`

```bash
scripts/bench.sh perf list
scripts/bench.sh perf list --limit 50
```

Display:

```text
run id
date
git SHA
dirty/clean
suite
host
status
recorded/local
optional name
```

## 6.5 `show`

```bash
scripts/bench.sh perf show RUN_ID
scripts/bench.sh perf show baseline:main
scripts/bench.sh perf show RUN_ID --json
```

## 6.6 `layout`

```bash
scripts/bench.sh perf layout
scripts/bench.sh perf layout --json
```

This reports static representation facts without running programs.

Required types:

```text
Value
ObjRef
Object
RuntimeError
PhError
Result<Value, PhError>
```

Example human output:

```text
Value                  16 B  align 8
ObjRef                   8 B  align 8
Object                   40 B align 8
RuntimeError             96 B align 8
PhError                  96 B align 8
Result<Value, PhError>   96 B align 8
```

Do not hard-code expected numbers except `Value == 16`; read them with `size_of`/`align_of`.

Every `run` and `ab` result embeds these layout measurements automatically.

## 6.7 `baseline`

Commands:

```bash
scripts/bench.sh perf baseline list
```

```bash
scripts/bench.sh perf baseline promote RUN_ID --name main
```

```bash
scripts/bench.sh perf baseline promote RUN_ID --name pre-value16
```

Promotion copies the run into committed historical storage and updates the baseline index.

---

# 7. `scripts/bench.sh` backward compatibility

Update the `perf)` lane.

If no subcommand is supplied, run:

```text
run
```

If the first argument starts with `-`, insert:

```text
run
```

before existing arguments.

Thus this remains valid:

```bash
scripts/bench.sh perf --bench-only
```

and becomes internally:

```bash
phalcom-perf run --bench-only
```

Update usage examples without removing the other existing lanes:

```text
vm
criterion
wren
math
one
```

---

# 8. Structured suite manifests

Create:

```text
benchmarks/suites/representation.json
```

Schema:

```json
{
  "schema_version": 1,
  "name": "representation",
  "description": "VM workloads sensitive to Value density, dispatch, allocation, collections and fibers.",
  "cases": []
}
```

Each case:

```json
{
  "id": "vm/bare_send",
  "path": "benchmarks/vm/bare_send.ph",
  "tags": ["dispatch", "quick"],
  "default_samples": 9,
  "default_warmup": 1,
  "heavy": false,
  "verification": {
    "kind": "stdout_exact",
    "value": "0"
  },
  "work": {
    "unit": "send",
    "count": 200000
  }
}
```

The manifest should include at least:

```text
vm/bare_send
vm/arith_send
vm/rest_fallback_send
vm/fiber_spawn
wren/for
wren/map_numeric
wren/fib
concurrency/skynet
```

Tag Skynet and very large map workloads as:

```json
"heavy": true
```

so routine iteration remains ergonomic.

## Why `wren/for`

Current `benchmarks/wren-suite/for.ph` creates a list containing one million `Value`s and traverses it.

That is directly sensitive to:

- `Value` slot size;
- list backing-array growth;
- memory bandwidth;
- GC/heap interaction around the workload.

It is therefore one of the most important measurements for the 24 → 16 byte representation change.

## Why `map_numeric`

It places millions of numeric values into Map storage and exercises:

- Value copying;
- hashing;
- equality;
- collection allocation;
- memory footprint.

## Why retain bare send

A representation change can improve allocation-heavy workloads while regressing a pure dispatch path.

The repository has already observed exactly this class of trade-off with boxed heap object variants.

The benchmark system must reveal both sides.

---

# 9. Correctness is part of every sample

A benchmark sample that exits incorrectly does not have a valid timing.

For `stdout_exact`:

1. process must exit zero;
2. it must not panic;
3. normalized stdout must match expected text.

For acceptance negative tests retain current negative-test semantics.

For A/B:

```text
baseline output == expected
candidate output == expected
baseline output == candidate output
```

If outputs differ:

```text
abort comparison
do not emit a performance verdict
exit with harness error
```

Never label a semantically different program “faster.”

---

# 10. Mandatory measurements

Every measured process sample stores:

```rust
pub struct Sample {
    pub index: u32,
    pub order: Option<SampleOrder>,

    pub wall_ns: u64,

    pub user_ns: Option<u64>,
    pub sys_ns: Option<u64>,

    pub peak_rss_bytes: Option<u64>,

    pub minor_page_faults: Option<u64>,
    pub major_page_faults: Option<u64>,

    pub voluntary_context_switches: Option<u64>,
    pub involuntary_context_switches: Option<u64>,

    pub exit_code: Option<i32>,
    pub status: SampleStatus,
}
```

Mandatory on every platform:

```text
wall_ns
exit/correctness
```

Required when supported by the platform resource meter:

```text
user_ns
sys_ns
peak_rss_bytes
```

Optional:

```text
fault counts
context switches
```

If a metric is unavailable:

```json
null
```

Never:

```json
0
```

unless the actual measured value is zero.

---

# 11. Canonical wall-clock measurement

Use:

```rust
std::time::Instant
```

around child process execution as the canonical wall time.

Do not use the external `time` command's wall field as the canonical value.

External resource tools supply:

```text
user
sys
RSS
faults
context-switch metrics
```

`Instant` remains the consistent wall-clock source.

---

# 12. Platform resource meter

Implement in:

```text
phalcom-core/bin/perf/measure.rs
```

Use an enum:

```rust
enum ResourceMeter {
    MacOsTime,
    LinuxGnuTime,
    WallOnly,
}
```

## macOS

When `/usr/bin/time` exists, invoke it with BSD/macOS resource reporting and write its own statistics to a temporary file under:

```text
target/perf/tmp/
```

Parse at least:

```text
user time
system time
maximum resident set size
```

macOS maximum RSS is normalized to bytes.

## Linux

Use GNU:

```text
/usr/bin/time -v
```

with output redirected to the temporary resource file.

Parse:

```text
User time (seconds)
System time (seconds)
Maximum resident set size (kbytes)
Minor page faults
Major page faults
Voluntary context switches
Involuntary context switches
```

Convert RSS:

```text
kB -> bytes
```

before storing it.

## Fallback

If `/usr/bin/time` is missing or the parser cannot recognize the platform output:

1. preserve the valid `Instant` wall measurement;
2. set resource metrics to `null`;
3. record:

```json
"resource_quality": "wall_only"
```

4. print a clear warning.

Do not fail an otherwise useful developer smoke run solely because RSS measurement is unavailable.

However, a comparison explicitly gated on RSS must refuse to issue an RSS verdict when the field is unavailable.

---

# 13. Statistical aggregation

Store **every raw sample**.

Do not store only averages.

For each metric calculate:

```text
n
minimum
median / p50
maximum
MAD (median absolute deviation)
p90 when n is large enough
```

Representation:

```rust
pub struct MetricSummary {
    pub samples: usize,
    pub min: u64,
    pub median: u64,
    pub max: u64,
    pub mad: u64,
    pub p90: Option<u64>,
}
```

`CaseAggregate` contains one optional `MetricSummary` per metric.

The human table should make median the default reported number.

Retain minimum because historical Phalcom performance investigations have sometimes used best-of-N to distinguish scheduler noise, but do not make minimum the sole comparison statistic.

---

# 14. A/B comparison statistics

A/B is different from two unrelated stored runs.

For each pair `i`:

```text
ratio_i =
    candidate.wall_ns_i / baseline.wall_ns_i
```

Record:

```text
median paired ratio
median percentage delta
candidate-faster pair count
candidate-slower pair count
ties
```

Example:

```text
for
baseline median    4.810 s
candidate median   3.220 s
paired delta      -33.1%
sign               14/15 faster
```

For effects below roughly 10%, the **sign across pairs** must be prominently displayed.

Do not allow a pretty four-decimal-place average to hide that pair signs are inconsistent.

---

# 15. Preserve and integrate the quiet-machine guard

Do not discard `benchmarks/vm/ab-guarded.py`.

Refactor it minimally so its guard can be reused by the Rust orchestrator.

Add a mode conceptually equivalent to:

```bash
benchmarks/vm/ab-guarded.py \
  --check-only \
  --where "vm/bare_send pair 4"
```

Exit:

```text
0 = quiet
3 = contaminated / busy
2 = guard/tool error
```

`phalcom-perf ab` must run the guard:

```text
before the entire session
before every pair
after the entire session
```

If the guard returns `3`:

- abort;
- mark the run `contaminated`;
- do not emit a “candidate +X%” verdict;
- keep diagnostic metadata explaining why the session was discarded.

A contaminated run may be stored locally for investigation but cannot be promoted as a baseline.

The user-facing message should say plainly:

```text
benchmark aborted: machine became busy; no performance verdict recorded
```

Do not pause and resume the same run.

---

# 16. No build operations inside timing sessions

This is a hard rule.

Before:

```bash
phalcom-perf ab
```

starts its first timing:

- both binaries must exist;
- both must be executable;
- metadata must be captured;
- no Cargo/rustc operation remains.

The harness must never do:

```text
build baseline
time baseline
build candidate
time candidate
...
```

inside the pair loop.

Recommended developer workflow:

```bash
# clean baseline worktree
cargo build --release -p phalcom-core --bin phalcom
cp target/release/phalcom /tmp/phalcom-before

# candidate tree
cargo build --release -p phalcom-core --bin phalcom

scripts/bench.sh perf ab \
  --baseline-bin /tmp/phalcom-before \
  --candidate-bin target/release/phalcom \
  --suite representation \
  --pairs 15
```

---

# 17. Run metadata schema

Create serialized types in `model.rs`.

Top-level:

```rust
pub struct BenchmarkRun {
    pub schema_version: u32,
    pub run_id: String,

    pub started_at_unix_ms: u64,

    pub name: Option<String>,

    pub git: GitMetadata,
    pub build: BuildMetadata,
    pub host: HostMetadata,
    pub layouts: LayoutMetadata,

    pub command: RunCommandMetadata,
    pub suite: SuiteMetadata,

    pub resource_quality: ResourceQuality,

    pub cases: Vec<CaseResult>,
    pub summary: RunSummary,
}
```

## Git metadata

Store:

```text
full SHA
short SHA
branch
dirty boolean
```

Determine dirty state with tracked-file status.

Do not silently label a dirty tree as equivalent to its commit SHA.

## Build metadata

Store:

```text
binary path
binary file size
profile
target triple where available
rustc -Vv
cargo -V
features label / unknown
```

For the canonical `scripts/bench.sh perf run` path:

```text
profile = release
features = default
```

must be known.

For arbitrary `--binary`, allow:

```text
profile = unknown
features = unknown
```

rather than guessing.

## Host metadata

Store:

```text
OS
OS version where cheaply available
architecture
CPU model
logical CPU count
```

Construct a human-readable host key such as:

```text
darwin-arm64-Apple_M3_Max-16
```

Comparisons use the structured fields, not the exact string spelling.

Do not store private user paths or environment contents beyond explicitly relevant benchmark metadata.

---

# 18. Embed representation layout in every run

`env.rs` should directly use:

```rust
std::mem::{size_of, align_of};
```

and capture:

```rust
pub struct TypeLayout {
    pub size_bytes: usize,
    pub align_bytes: usize,
}
```

For:

```text
Value
ObjRef
Object
RuntimeError
PhError
Result<Value, PhError>
```

This makes future agent inspection extremely useful.

An agent can later answer:

```text
"When did Value become 16 B?"
"When did RuntimeError grow by 24 B?"
"Did a performance regression coincide with Object size growth?"
```

without reconstructing old source trees.

---

# 19. Stable result storage

Use two levels.

## 19.1 Automatic local storage

Every valid or diagnostic run writes:

```text
target/perf/runs/<RUN_ID>/run.json
```

A/B additionally writes:

```text
target/perf/runs/<RUN_ID>/comparison.json
```

This is automatic and git-ignored through `target/`.

Also place human summary:

```text
target/perf/runs/<RUN_ID>/summary.txt
```

This is useful for both developers and agents that prefer concise text.

## 19.2 Durable repository history

Create:

```text
benchmarks/results/README.md
benchmarks/results/schema-v1.json
benchmarks/results/baselines.json
benchmarks/results/history/
```

Do **not** automatically dirty the repository on every routine benchmark.

Only:

```bash
--record
```

or:

```bash
perf baseline promote
```

copies a canonical `run.json` into:

```text
benchmarks/results/history/<RUN_ID>.json
```

This file is intended to be committed.

That provides:

- ephemeral high-frequency local runs;
- curated durable historical data.

---

# 20. Baseline index

Initial:

```json
{
  "schema_version": 1,
  "baselines": {}
}
```

Promotion:

```bash
scripts/bench.sh perf baseline promote \
  RUN_ID \
  --name pre-value16
```

writes:

```json
{
  "schema_version": 1,
  "baselines": {
    "pre-value16": {
      "run_id": "...",
      "path": "history/...json"
    }
  }
}
```

Never use symlinks for baseline identity.

A baseline name is metadata, not a filesystem trick.

---

# 21. Comparison compatibility rules

Before presenting percentage changes, validate comparability.

## Hard mismatches by default

Refuse a performance verdict when:

```text
different architecture
different OS family
different benchmark case identity
different benchmark semantic output
different build profile
```

For host CPU model mismatch, refuse by default but support:

```bash
--allow-host-mismatch
```

for exploratory comparisons.

Display a prominent warning when overridden.

## Layout changes are allowed

A representation benchmark is specifically intended to compare:

```text
Value 24 B
vs
Value 16 B
```

so layout mismatch is informative, not a compatibility failure.

Include it in the comparison header.

---

# 22. Stored-run comparison algorithm

For independently recorded runs:

```text
delta =
    candidate.median / baseline.median - 1
```

Display:

```text
absolute delta
percentage delta
baseline MAD
candidate MAD
```

Do not hard-fail small noisy cases by default.

Recommended default gating rules when `--gate` is explicitly requested:

### Wall

Candidate regression is a gate failure only if:

```text
relative regression > 5%
AND
absolute regression > 1 ms
```

### Peak RSS

Fail if:

```text
median peak RSS regression > 10%
AND
absolute RSS increase > 8 MiB
```

### System time

Report by default.

Gate only when a suite manifest explicitly defines a threshold, because system time can be unusually host-sensitive.

Thresholds belong in the suite/case definition, not scattered through comparison code.

---

# 23. Paired A/B gate

For guarded paired A/B, use stronger evidence.

A wall-time regression gate should require both:

```text
median paired regression exceeds case threshold
```

and:

```text
candidate is slower in at least two-thirds of valid pairs
```

Example with 15 pairs:

```text
>= 10 / 15 slower
```

Likewise, an improvement should be described as “strongly supported” only when its sign is similarly consistent.

Otherwise report:

```text
inconclusive / noisy
```

rather than forcing a win/loss.

---

# 24. Exit codes

Standardize:

```text
0
    command succeeded;
    no requested gate failed

1
    valid benchmark/comparison completed,
    but a requested regression gate failed

2
    usage error, invalid suite, missing binary,
    correctness failure, incomparable runs,
    malformed stored data

3
    benchmark invalidated by machine contention;
    no performance verdict
```

This preserves the established meaning of guarded A/B exit `3`.

CI/agents can distinguish:

```text
regression
tool failure
contaminated measurement
```

without parsing prose.

---

# 25. Human comparison output

Example:

```text
Phalcom benchmark comparison

baseline   pre-value16  abc1234
candidate  post-value16 def5678
host       darwin-arm64 / Apple M3 Max / 16 logical CPUs

layout
  Value                 24 B -> 16 B   -33.3%
  RuntimeError         104 B ->  96 B    -7.7%
  Object                40 B ->  40 B     0.0%

case                       wall median       delta       RSS median      delta
vm/bare_send              65.8 -> 66.7 ms    +1.4%      18 -> 18 MiB     0.0%
vm/arith_send             72.0 -> 68.1 ms    -5.4%      19 -> 18 MiB    -5.3%
wren/for                   5.2 -> 4.1 s      -21.2%     3.4 -> 2.5 GiB  -26.5%
wren/map_numeric          ...               ...        ...             ...
concurrency/skynet        ...               ...        ...             ...

verdict
  0 gated regressions
  3 improvements
  1 neutral
```

Do not color-code machine-readable output.

TTY human output may use restrained coloring.

---

# 26. JSON comparison output

`comparison.json` should contain:

```json
{
  "schema_version": 1,
  "comparison_id": "...",
  "baseline_run_id": "...",
  "candidate_run_id": "...",
  "compatible": true,
  "layout_delta": {},
  "cases": [],
  "verdict": {
    "regressions": 0,
    "improvements": 3,
    "neutral": 1,
    "inconclusive": 0
  }
}
```

For each case include:

```text
baseline aggregate
candidate aggregate
absolute delta
percentage delta
gate threshold
gate result
paired statistics if A/B
```

This allows future tooling or an agent to compare runs without scraping terminal tables.

---

# 27. Preserve Criterion as a separate instrument

Do not subsume Criterion timings into whole-process run statistics.

Keep:

```bash
scripts/bench.sh criterion
```

and:

```text
phalcom-core/benches/vm_bench.rs
```

Criterion remains useful for:

- fixed micro-mechanism tripwires;
- distributions/confidence intervals;
- local detailed comparisons.

`phalcom-perf` records whole-process workloads.

A future optional importer may attach Criterion JSON/artifacts, but this spec does not require parsing Criterion output.

---

# 28. Preserve opcode histogram as a separate non-timing build

The existing `opcode-histogram` feature intentionally changes per-opcode cost.

Never time a binary compiled with it and report those timings as production performance.

If future benchmark tooling invokes opcode counts, record them as an auxiliary artifact:

```text
instrument = opcode_histogram
timing_valid = false
```

The run metadata must make clear that counts and timing came from separate binaries.

---

# 29. Tests for benchmark infrastructure

The benchmark tool itself needs substantial deterministic tests.

## `model.rs`

Round-trip:

```text
BenchmarkRun -> JSON -> BenchmarkRun
```

Verify schema version.

## `measure.rs`

Test parser fixtures for:

```text
macOS /usr/bin/time -l
Linux GNU /usr/bin/time -v
malformed output
missing optional fields
```

Store representative raw `time` output strings directly in test constants.

Do not invoke long real benchmarks in unit tests.

## `store.rs`

Using `tempfile`:

- save run;
- load run;
- promote run;
- update baseline index;
- reject missing run;
- reject malformed schema;
- resolve `latest`;
- resolve `baseline:name`.

## `compare.rs`

Unit-test:

```text
delta calculations
median
MAD
pair sign counts
2/3 rule
absolute-noise floor
RSS threshold
host mismatch
missing metric
```

## `suite.rs`

Test manifest parsing and duplicate case ID rejection.

A duplicate case ID is a hard configuration error.

## CLI

Add lightweight command tests where practical for:

```text
layout --json
list
show
compare
```

Do not require running Skynet from the test suite.

---

# 30. Benchmark session for the 16-byte Value migration

After all three specs are implemented, perform the first durable comparison as follows.

## Step 1 — identify baseline commit

Use the commit immediately before the 16-byte `Value` implementation.

Record its SHA.

## Step 2 — build baseline in an isolated clean tree/worktree

```bash
cargo build \
  --release \
  -p phalcom-core \
  --bin phalcom
```

Copy it out:

```bash
cp target/release/phalcom \
   /tmp/phalcom-pre-value16
```

Do not rebuild this path later.

## Step 3 — build candidate

On the completed branch:

```bash
cargo build \
  --release \
  -p phalcom-core \
  --bin phalcom \
  --bin phalcom-perf
```

## Step 4 — smoke suite

```bash
scripts/bench.sh perf ab \
  --baseline-bin /tmp/phalcom-pre-value16 \
  --candidate-bin target/release/phalcom \
  --suite representation \
  --quick \
  --pairs 7
```

This proves the harness, correctness, and output equivalence.

## Step 5 — full quiet A/B

On a quiet machine:

```bash
scripts/bench.sh perf ab \
  --baseline-bin /tmp/phalcom-pre-value16 \
  --candidate-bin target/release/phalcom \
  --suite representation \
  --pairs 15 \
  --heavy \
  --record \
  --name value16-ab
```

## Step 6 — inspect expected hypotheses

The implementation should **not** require these results, but the analysis should ask:

### `wren/for`

Does the 33% smaller `Value` materially reduce:

```text
wall
sys
peak RSS
```

for its million-element List?

### `map_numeric`

Does denser key/value storage help RSS and allocation/reallocation behavior?

### `fiber_spawn` / Skynet

Do smaller stack `Vec<Value>` buffers reduce fiber memory and copy pressure?

### `bare_send`

Did the extra tag/depth decoding make pure dispatch measurably slower?

### `arith_send`

Do smaller Values improve argument/stack traffic despite equivalent language semantics?

## Step 7 — promote durable baseline

Promote the pre-change and post-change runs with explicit names, for example:

```bash
scripts/bench.sh perf baseline promote \
  PRE_RUN \
  --name pre-value16
```

```bash
scripts/bench.sh perf baseline promote \
  POST_RUN \
  --name value16
```

Commit:

```text
benchmarks/results/history/<pre>.json
benchmarks/results/history/<post>.json
benchmarks/results/baselines.json
```

along with a concise human performance note if the project continues using `docs/perf-log`.

---

# 31. What not to do

Do not:

- time debug binaries and call them representative;
- build inside timing loops;
- compare different hosts silently;
- average away machine contention;
- report unavailable RSS as zero;
- write only human Markdown results;
- write only machine JSON with no readable summary;
- make every local run dirty Git;
- replace Criterion;
- discard the existing A/B guard;
- use one single run as a regression gate;
- gate a 1 ms benchmark on a 1% difference;
- measure a semantically failing program;
- commit massive stdout/stderr blobs;
- auto-promote a contaminated run;
- treat `Value == 16 B` itself as proof that the VM became faster.

---

# 32. Definition of done

The benchmark system is complete when:

- `scripts/bench.sh perf run` is ergonomic and documented;
- `scripts/bench.sh perf ab` performs alternating guarded A/B;
- every sample is correctness checked;
- wall/user/sys/RSS are captured where supported;
- unavailable metrics are explicitly null;
- raw samples and statistical aggregates are retained;
- every run embeds Git/build/host metadata;
- every run embeds core Rust representation sizes;
- run files use a versioned serde JSON schema;
- routine runs save automatically under `target/perf/runs`;
- explicit promotion records durable history under `benchmarks/results/history`;
- named baselines are managed through a structured index;
- `compare` can compare two stored runs without re-running benchmarks;
- host/build incompatibility is detected;
- regression gates use relative + absolute thresholds;
- A/B results include pair-sign evidence;
- machine contention exits with code 3 and no performance verdict;
- Criterion remains available independently;
- old `scripts/bench.sh perf --bench-only` style invocations remain usable;
- the first pre/post 16-byte `Value` comparison is recorded in durable history.