---
id: ARTF001.C1.P2.R4
kind: walkthrough
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
date: 2026-10-24
---

# Query-owner test migration

Assessment baseline: `84e58af20` on `feat/artifact-universe-consolidation`.
The initial local checkout was behind that revision; reconciliation preserves
all intervening remote fixes. This record covers the query-test slice, while
P2.R4 remains open.

## Implementation

Query-owner regressions now exercise `resolve_query_key_v3` with an explicit
`StableOwnerCatalog` inside `artifact::query::query_owner_catalog_regressions`.
The compact fixture lifts V2 to V5. Module, declaration, callable, field and
type-member owners retain exact assertions. Mismatched, lossy and absent
owners fail closed. The overlapping root-policy case stays covered by the
existing policy suite.

The obsolete query integration target and its Cargo registration are removed.
Focused CI selects the owner regression module. The compatibility importer
remains test-only; no production API is restored.

## Reconciliation and consultation

The remote already fixed the type-export test context, receipt fixture,
immutable session parents, provenance joins, and migrated Universe behavior.
Those changes are retained. A read-only adviser confirmed that the supported
session constructors at the updated head attach the correct immutable parent.
Earlier advice based on the stale local checkout is superseded. No material
plan amendment or extra production-authority index is included.

## Verification

Current-head evidence after reconciliation:

- Semantic library and integration-target checks passed.
- Combined library unit gates passed 35/35: owner reconstruction 4,
  compatibility importer 20, query policy 6, required-root export 1,
  immutable leaves 1, and image-backed publication 3.
- Image-session and zero-copy integration gates passed 4/4.
- Compiler artifact round trips passed 13/13.
- `git diff --check` passed.

Commands use the pinned toolchain with empty `RUSTFLAGS` and `RUSTC_WRAPPER`.
Existing warnings remain. The slice is `IMPLEMENTED / FOCUSED_TESTED`;
P2.R4 remains incomplete. Real SlotMap full-package execution, package
determinism/hostile controls and SHA-1 known answers follow. Broad
certification was deliberately deferred until those focused paths pass.
