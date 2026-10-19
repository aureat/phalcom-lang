---
id: PERF003.C2.P8
kind: handoff
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# PERF003.C2.P8 takeover handoff

## Readiness

P8 takeover is unblocked after independent P7 certification. No P8 implementation has started.

- P7 implementation revision measured: `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`.
- P7 R.G7 test revision: `c1524361b9606dc537671997fe5069f2aa66035b`.
- R.G6: PASS; R.G7: PASS.
- Evidence report: `docs/perf003-p7-audit/REPORT.md`.
- Case matrix: `docs/perf003-p7-audit/R.G6-R.G7-CERTIFICATION.md`.
- Evidence publication commit: see the PERF003.C2 checkpoint and final reconciliation delivery record.
- Detailed P8 plan: `PERF003.C2.P8-portable-executable-universe-detailed-implementation-plan.md`.

## Inherited P7 contracts

- Artifact-backed compiler initialization is source-free. Semantic, type, static-value, and query authority comes from validated artifact sections.
- Whole-load allocation N is 373,069,634 bytes. N−1 returns the typed budget failure; exact-N retry succeeds.
- The tested artifact is 15,936,496 bytes with BLAKE3-256 digest `b91ccc4e6011c2789c6e20861e6d6d98908074bb45461d54b5bd8885ca9665bf`.
- Package source/artifact compiler parity passes for `packages/slotmap` (4 modules) and `packages/slotmap/tests/runtime` (8 modules). The SlotMap runtime smoke passes.
- Runtime bytecode portability is outside P7. The current R.T6 runtime suite has 20 `RUNTIME_ERROR` rows with empty error detail; it supplies no runtime performance claim.
- Workspace semantic integration tests still have 412 inherited API fixture compile errors, reproduced at clean predecessor `08e2b2f5`. Do not migrate those fixtures as part of P8 unless a named P8 acceptance invariant requires them.

## Next action

Begin P8 planning/execution from the detailed plan. Preserve the P7 compiler-image ownership, identity, allocation, and source-free invariants while defining the executable image boundary.
