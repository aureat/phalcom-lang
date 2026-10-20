---
id: PERF003.C1.P5
kind: observability-schema
status: IN_PROGRESS
schema_version: 3
baseline_head: e2ae4a0fd360e4075ce8ad229a3e39e1531fb771
---

# PERF003.C1.P5 observability schema

The canonical serializable report model is `telos_observability::PerfReport`.
It contains run metadata, elapsed wall time, hierarchical phase spans, stable
counters, semantic-query lifecycle aggregates, and bounded distributions.
Source text is not part of the model.

## Compatibility

The current schema version is 3. Decoding accepts versions 1 through 3. Query
metrics and distributions default to empty for older reports. The optional
`metadata.lifecycle` field is additive and records `process_cold`,
`process_warm`, `session_cold`, `session_warm`, or
`vm_cold_inside_warm_process` when the harness can establish that state. Reports
without this field remain readable and must be rendered as unclassified.

Phase identifiers are enum-backed serialized names. New phases are additive;
renaming a serialized phase requires an explicit schema decision. Counters are
also enum-backed and aggregate deterministically. Detailed/Trace distributions
are bounded count/sum/max summaries, not unbounded event retention.

## Derived values

`phalcom-perf profile show` calculates ratios only when the report contains the
required counters and a nonzero denominator:

- source traversal: statement visits / unique top-level statements;
- dependency fanout: reverse-closure nodes visited / seed nodes;
- query recomputation: recomputed / requested queries.

Callable-body amplification remains unavailable until a stable unique-body
denominator is emitted. Copy amplification remains unavailable without direct
physical-copy volume and a retained-size denominator. Universe wall-time share
is explicitly labeled partial when calculated from scope-attributed spans.
Unavailable values carry a reason rather than a fabricated zero.

## Profile attachment and comparison

`phalcom-perf profile capture` stores a profile-only elapsed-time sample under
`target/perf/profiles/`. A profile may also be attached as
`internal-profile.json` in a process-sample directory under `target/perf/runs/`;
attachment checks the report's source revision and dirty state against the
sample metadata. Comparison reports elapsed time, per-phase durations,
counters, and query lifecycle deltas. Workload, observability-level, lifecycle,
or pinned-baseline mismatches make the pair structurally incompatible while
preserving the measured deltas for inspection.

Capture, show, and comparison were exercised on two compatible B2 profiles.
This confirms the CLI data path only; it is one timing pair and not an
acceptance measurement.
