---
id: PERF003.C1.P5
kind: benchmark-methodology
status: IN_PROGRESS
baseline_head: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
---

# PERF003.C1.P5 benchmark methodology

## Revisions

All evidence in this plan is pinned to `BASELINE_HEAD=e2ae4a0fd360e4075ce8ad229a3e39e1531fb771`.
The implementation worktree began at that committed revision and records the
execution-tree HEAD separately. Do not move the worktree when `main` advances;
start a separate run for a later baseline.

## Profile collection

`benchmarks/perf003/run-profile.sh <workload> <level> [samples]` builds the
feature-enabled example in release mode and starts a distinct OS process for
each sample. It requires `PHALCOM_PERF_BASELINE_HEAD`, records the committed
HEAD, branch, dirty state, and active Rust toolchain, and writes JSON reports
and exit codes under `target/perf/profiles/`. Profile output omits source text.

`benchmarks/perf003/run-overhead-matrix.sh <workload> [samples-per-level]`
runs the same workload independently in a feature-disabled build, then with
the feature enabled at runtime `off`, `summary`, `detailed`, and `trace`.
Feature-disabled profiling currently supports B2 only. Use a successful
representative case such as B2 for diagnostic comparison; it does not satisfy
the plan's tiny, medium, and warm-incremental workload coverage. The matrix
wrapper does not decide acceptance; record and interpret the actual observed
overhead.

## Workload state

[`benchmarks/perf003/p5-workloads.json`](../../../../benchmarks/perf003/p5-workloads.json)
is the B1–B9 inventory. Every report includes a typed lifecycle classification
when the harness can establish one. B3 performs the canonical semantic-baseline
warm-up before installing the recorder and labels the measured session
instantiation `session_warm`. Every run from the script is process-isolated.

The inventory records current completion and blockers. In particular, B7 and
B8 are unfinished, B6 is only a four-module candidate, and B9 depends on the
blocked B1 path. These cases must not be presented as a complete or accepted
suite.

## Stored profiles

Capture a process-level `phalcom-perf` sample, then attach the report with
`phalcom-perf profile capture <profile.json>` stores a profile-only wall-time
sample. `phalcom-perf profile attach <sample-run-id> --report <profile.json>`
stores a sidecar beside an external process-level run and checks revision and
dirty-state identity. `phalcom-perf profile show` renders bounded top-N
summaries; `phalcom-perf profile compare` reports compatibility, elapsed time,
phase, counter, and query deltas plus derived metrics. Two separately captured
B2 samples were successfully shown and compared; this was one pair, not a
performance conclusion.

## Current verification boundary

The runner scripts passed syntax validation and the observability library,
profile example, and default/feature-enabled `phalcom-perf` targets passed
`cargo check`. The T12 stored-profile capture/show/compare workflow was exercised
with two B2 reports. The five-mode runner is available, but its diagnostic B2
matrix and final characterization have not yet been run; T13–T15 remain
unverified. The baseline blocks full VM
startup at ProductKind source indexing and inline compiler projection at legacy
Record-remainder metadata export.
