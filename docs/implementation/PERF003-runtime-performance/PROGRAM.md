---
id: PERF003
category: PERF
kind: optimization
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# PERF003 — runtime performance

This program owns runtime performance instrumentation and implementation
records. PERF003.C1 is partially implemented; its instrumentation follow-up
has a run-scoped report substrate, compiler/VM spans, semantic-update phase
spans and committed-work counters, exhaustive query-kind classification and
per-kind query lifecycle reporting, and successful canonical semantic-baseline,
ordinary semantic-update, and canonical source-backed semantic-update
profiles, callable-body pass counts, and reverse-closure traversal totals. T1
report round-trip and T6 query lifecycle tests remain outstanding.
Canonical compile and full-VM characterization remain blocked by recorded
baseline failures.
