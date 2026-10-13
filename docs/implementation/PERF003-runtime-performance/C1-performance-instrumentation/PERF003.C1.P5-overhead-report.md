---
id: PERF003.C1.P5
kind: observer-overhead-report
status: IN_PROGRESS
completion: NOT_STARTED
verification: UNVERIFIED
baseline_head: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
---

# PERF003.C1.P5 observer-overhead report

## Result

No A/B/C/D observer-overhead samples have been collected. This report makes no
overhead acceptance claim.

## Collection method

Use `benchmarks/perf003/run-overhead-matrix.sh semantic-baseline 5` to launch
five process-isolated samples each at `off`, `summary`, `detailed`, and `trace`
levels. Set `PHALCOM_PERF_BASELINE_HEAD` to the full pinned baseline SHA. Each
sample report includes its process lifecycle, baseline revision, source
revision, branch, dirty state, and toolchain.

Compare medians only for successful samples of the same workload and lifecycle.
Keep correctness outcomes and incomplete reports visible. The matrix runner
records each process exit code. Do not infer Summary overhead from a workload
that stops at a correctness failure or compare modes with different work.

## Acceptance state

T14 is unverified until the matrix is run and the four mode distributions,
sample counts, outcomes, and observed overhead are recorded here. If Summary
overhead exceeds the plan target, change only the observability implementation
and repeat the same workload matrix.
