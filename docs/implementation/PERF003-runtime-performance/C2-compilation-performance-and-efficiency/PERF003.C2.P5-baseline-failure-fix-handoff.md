# PERF003.C2.P5 — Canonical Image Continuation Handoff

**Program:** PERF003
**Checkpoint:** PERF003.C2
**Plan:** PERF003.C2.P5
**Status:** IN_PROGRESS
**Completion:** PARTIAL
**Verification:** FOCUSED_TESTED
**Date:** 2026-10-13
**Branch:** `perf/universe-characterization`
**Worktree:** `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
**Source revision:** `e71057d9dcb559a9777ff3c51f568497c83a3683` (continuation over `0b40dbbb`).
**Delivery:** source committed; lifecycle records accompany it on `perf/universe-characterization`.

## Established continuation

The earlier Universe-wide dependency bypass has been replaced with exact image
aggregate fingerprints. `CanonicalUniverseFormalImage::dependency_fingerprint`
consults frozen roots and its derived immutable structural index. Hierarchy
entries retain validated source-sensitive query fingerprints and require exact
agreement with final declaration/parent identity. Other approved structural
families use final aggregate fingerprints. Catalog/frozen overlap must agree.

`SemanticDb::dependency_fingerprint` preserves local-state precedence. Edge
recording, validation and immutable-leaf indexes use exact membership. An
aggregate-only dependency does not become a Ready/reusable query or typed query
product. Formal replay and computed-key readiness use exact evidence; invented
Universe identities fail. Query freezing excludes structural rows changed by
final settlement and rejects their dependent closure.

`ParentMap` and `with_parent` factories now cover dispatch/surfaces, conditional
member sets/type registrations, data, enums/variants, associated surfaces,
enum requirements and type-member declarations. Snapshot surfaces share the
dispatch map; associated resolver views avoid flattening canonical maps.
First updates use base overlays; subsequent updates clone local contributions.
Declaration/signature/hierarchy overlays remain established from Slice 1.

Alias result-kind inference from the inherited fix is retained. Static-definition
fingerprints now include formed alias semantics, and both publication variants
compare the freshly formed fingerprint before reuse. The imported-alias A6
regression passes with its original shell-publication assertion intact.
Cycles quarantine participants and transitive alias dependents, removing alias
products, declarations, static definitions/cached shells and retained forms.
Unrelated aliases survive. Stale global-empty test assertions now check exact
invalid identities; diagnostics and declaration-absence checks remain.

## Invariants and rulings

See CHECKPOINT.md for adopted Level 1 adviser clarifications. Frozen query
eligibility and aggregate dependency authority are distinct; never restore a
Universe-wide bypass or synthesize Ready rows. Parent mutation/collision policy
stays strict: foreign inherent impls of canonical targets are forbidden by
`docs/specs/objects/impl.md`. Independent focused review has no remaining
findings after correcting a computed-key readiness consumer.

## Completed verification

- Incremental recomputation filter: 13 passed (120.97 s), including all four
  inherited failures.
- Workspace integration filter: 18 passed (44.89 s), including new
  valid→cycle→valid quarantine/preservation regression.
- Alias capability filter: 7 passed (6.57 s).
- Final canonical-image library filter: 12 passed (41.84 s), including exact
  aggregate dependency evidence and final-settlement freeze rejection.
- Parent library filter: 10 passed (0.06 s).
- Canonical-image integration: 4 passed (123.33 s), including source parity,
  provider isolation and actual canonical sharing across local replacements.
- Core with observability and LSP checks passed; LSP retains an unrelated
  unused import warning.
- Computed Record-key callable-signature regression passed (one selected).
- Scoped diff whitespace check passed. Full format/workspace/release and
  benchmark certification were deliberately deferred.

## Next work

Continue residual P5.T6 Slice 4 canonical lookup maps (alias forms/static
products/callable definitions and remaining copied aggregates), then T7–T9
read/persistence contracts and observability. Do not claim zero copies across
all product families yet. T10/final acceptance remain gated on P4 metadata
composition. P5 does not persist `.telosc` and still source-builds the image
on first process access.

Keep unrelated worktree state intact. Commit and push were authorized on
2026-10-13. Broad workspace/release gates and speedup certification were
deliberately deferred. Delivery does not promote P5 to COMPLETE.
