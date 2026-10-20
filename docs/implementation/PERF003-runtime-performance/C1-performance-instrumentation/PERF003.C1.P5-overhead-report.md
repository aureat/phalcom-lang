---
id: PERF003.C1.P5
kind: observer-overhead-report
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
baseline_head: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
---

# PERF003.C1.P5 observer-overhead report

## Result

The B2 canonical semantic-baseline workload completed in five separate
processes for each of the five modes. All 25 processes exited successfully.
The table reports each mode's median elapsed time and full observed range;
overhead is calculated from the ratio of mode medians to the feature-disabled
median.

| Mode | Successful samples | Median | Observed range | Median overhead vs. feature-disabled |
|---|---:|---:|---:|---:|
| Feature-disabled | 5/5 | 48.915 ms | 48.523–61.416 ms | Reference |
| Feature-enabled Off | 5/5 | 49.553 ms | 48.884–69.691 ms | +1.30% |
| Summary | 5/5 | 49.155 ms | 47.894–70.431 ms | +0.49% |
| Detailed | 5/5 | 49.110 ms | 48.139–70.792 ms | +0.40% |
| Trace | 5/5 | 51.827 ms | 48.711–71.160 ms | +5.95% |

All reports identify `BASELINE_HEAD=e2ae4a0fd360e4075ce8ad229a3e39e1531fb771`,
source and execution revision `cde1062260a74dc97a43344b1b29466c8529bfee`,
branch `perf/universe-characterization`, a clean worktree, and toolchain
`nightly-2026-07-10-aarch64-apple-darwin`. They classify the workload as
`canonical_semantic_baseline` with `process_cold` lifecycle.

After the investigation was pinned, `main` advanced from the recorded baseline
to `b9c8ddd9` with changes limited to TYPEAUD001 planning/checkpoint documents.
That commit was not incorporated into this worktree. It does not change the
measured semantic/compiler source; later main changes to those paths should be
characterized in a separate run.

This is diagnostic evidence for B2 only. Its wide per-mode ranges show
substantial process-level timing variation, and the workload is not one of the
plan's tiny-package, medium-package, or warm-incremental acceptance cases. The
observed Summary median therefore does not establish the plan's overhead target
across its required workload set.

## Collection method

The five-mode run used `PHALCOM_PERF_BASELINE_HEAD=e2ae4a0fd360e4075ce8ad229a3e39e1531fb771`
with `benchmarks/perf003/run-overhead-matrix.sh semantic-baseline 5`, followed
by a metadata-aligned rerun of the feature-disabled samples using
`benchmarks/perf003/run-profile.sh semantic-baseline disabled 5`. The
feature-disabled profile currently supports B2 only. Each sample is a separate
process and includes its lifecycle, baseline revision, source revision,
branch, dirty state, and toolchain. Runtime Off retains only an elapsed-time
clock.

Raw JSON reports are retained in `samples/PERF003.C1.P5-T14-B2-<mode>-<nn>.json`
for modes `disabled`, `off`, `summary`, `detailed`, and `trace`. The matching
process outcomes are in `samples/PERF003.C1.P5-T14-B2-exit-codes.csv`.

Compare medians only for successful samples of the same workload and lifecycle.
Keep correctness outcomes and incomplete reports visible. The matrix runner
records each process exit code. Do not infer Summary overhead from a workload
that stops at a correctness failure or compare modes with different work. B2
provides a diagnostic cold-baseline comparison; it does not satisfy the plan's
required tiny-package, medium-package, and warm-incremental workload coverage.

## Acceptance state

T14 remains incomplete until equivalent five-mode evidence covers the required
tiny-package, medium-package, and warm-incremental workloads. B7–B8 are still
unimplemented, B6's medium-size status still needs acceptance, and the
feature-disabled harness currently supports only B2. If Summary overhead
exceeds the plan target on the required workloads, change only the
observability implementation and repeat the same matrix.
