# PERF003.C1.P5 profile harness

`run-profile.sh` builds the instrumented example in release mode and starts a
fresh operating-system process for each sample. Set
`PHALCOM_PERF_BASELINE_HEAD` to the exact committed baseline SHA before running
it. Reports and per-sample exit codes are written under
`target/perf/profiles/`; they contain revision and lifecycle metadata but no
source text.

For example, from the repository root:

```sh
PHALCOM_PERF_BASELINE_HEAD=e2ae4a0fd360e4075ce8ad229a3e39e1531fb771 \
  benchmarks/perf003/run-profile.sh semantic-baseline detailed 5
```

`run-overhead-matrix.sh` runs the same workload at `off`, `summary`, `detailed`,
and `trace` levels. Each sample is a separate process. Use B2 for overhead
characterization while B1 is blocked by the baseline VM startup failure.

The current B1–B9 implementation and blocker ledger is
[`p5-workloads.json`](p5-workloads.json). B7–B8 still need workload bodies;
B5 is backed by the standalone-module fixture. B6 currently uses a four-source
module project and remains a candidate until its medium-workload cardinality is
accepted. B7–B8 still need workload bodies; B9 depends on B1 reaching canonical
VM initialization.
