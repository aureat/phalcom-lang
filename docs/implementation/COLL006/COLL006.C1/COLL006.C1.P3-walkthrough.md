---
id: COLL006.C1.P3
program: COLL006
checkpoint: COLL006.C1
kind: walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# COLL006.C1.P3 — Completion walkthrough

P3 completed its bounded Vector and inference-publication scope in the
`telos-collections-upgrade` worktree.

## Implemented

- Vector static generic formation retains exact `StaticInt` identity and rejects negative concrete extents during canonical annotation formation.
- Array public source contracts type index arguments, element replacements, checked reads, defaults, and callbacks without changing the native primitive floor.
- Sequence publication is checker-owned and fail-closed: accepted owners commit products; rejected, unresolved, deferred, and invalid owners do not.
- Deferred scalar/row readiness, exact-case conformance, union shared arguments, independent siblings, and static extraction are covered.
- Generic conflict diagnostics preserve failing argument ranges and explanation parents.
- Incremental invalid→valid→invalid publication transitions agree with cold analysis and retain an unchanged independent callable.

## Focused evidence

All commands used `RUSTFLAGS='' RUSTC_WRAPPER=''` where applicable.

| Scope | Result |
|---|---:|
| `phalcom-semantic` COLL006 filter | 46 passed |
| publication unit tests | 3 passed |
| generic proof-integrity | 8 passed |
| record-row inference | 6 passed |
| generic conformance | 21 passed |
| diagnostic golden suite | 8 passed |
| publication incremental parity | 1 passed |
| core Vector suite | 8 passed |
| core Array suite | 10 passed |
| exact static integer/static-domain/negative extent tests | passed |
| workspace all-targets check | passed |
| native surface `--check` | 406 declarations, passed |

## Deferred

Broader metadata/reflection failures recorded in checkpoint D1/D2 remain
classified baseline residuals. Workspace tests, clippy, and release
certification remain outside P3 and belong to later certification work.

P4 owns static homogeneous sequence values, const materialization, and the
atomic bare-bracket default/migration cutover.
