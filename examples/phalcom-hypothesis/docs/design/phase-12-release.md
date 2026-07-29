# Phase 12 Final Integration and Release Design

## Release boundary

Phase 12 changes packaging and ownership, not search semantics. The 0.1.0 release contains one authoritative implementation for choices, strategies, search, properties, reporting, persistence, stateful testing, and reflected derivation. The historical monolith, migration generator, and temporary compatibility modules are excluded.

Compatibility names remain only where they are inexpensive direct aliases. They cannot own state, wrap dispatch, or create a second behavior path.

## Verification model

A release is accepted only when:

1. every Phase 01–12 source/static verifier passes;
2. the Phase 11 mutation verifier rejects all five mutations;
3. the complete release gate passes twice in independent processes;
4. every documented root export is imported by `tests/integration/package_loads.ph`;
5. active source contains no retired syntax or placeholder implementation markers;
6. `SHA256SUMS` covers every project file except itself;
7. a clean extraction of the final ZIP reproduces the same results.

When a real Phalcom executable is available, `phalcom test --all` and every example are additional required evidence. Without that executable, runtime and benchmark behavior remain explicitly unobserved.

## Required final summary

```text
All tests passed.
No legacy syntax found.
No placeholder implementations found.
All public façade imports resolved.
```

In a source-only environment, “tests” in this summary refers only to the observed source/static gates; the checkpoint report must state that limitation adjacent to the summary.
