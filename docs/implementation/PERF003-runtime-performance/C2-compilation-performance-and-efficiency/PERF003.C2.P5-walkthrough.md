---
id: PERF003.C2.P5
program: PERF003
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# PERF003.C2.P5 — Partial Implementation Walkthrough

Continuation over `0b40dbbb` on `perf/universe-characterization`, 2026-10-13.
This is a progress record, not plan closeout. Source is committed as `e71057d9`;
lifecycle records accompany it on the same branch.

The continuation replaces a Universe-wide dependency bypass with an exact,
image-owned structural fingerprint index. Canonical dependency evidence stays
separate from frozen-query readiness, with local-state precedence and no mutable
canonical query copies. Query freeze rejects structural rows changed by final
return settlement and their dependent closure. Existing generic hierarchy
fingerprints preserve validated superclass source syntax.

Parent overlays now cover dispatch surfaces/conditional members/type
registrations, data, enums/variants, associated surfaces, enum requirements,
and type-member declarations. The snapshot shares dispatch's surface map, and
the resolver retains associated/type-member views without flattening parents.
First-update factories now use empty local overlays. Core map consumers were
adapted mechanically to the new merged iteration API.

Imported alias formation now fingerprints the resolved alias result before
cache reuse. Cycle quarantine removes the affected alias component and its
transitive alias dependents, cached declaration/static products and retained
forms. Unaffected canonical/local aliases survive; the original alias-shell
republication assertion still passes. Three global-empty alias fixture
assumptions were corrected to exact invalid identity checks under the adviser
ruling, retaining cycle diagnostics and declaration-absence checks.

Adviser rulings clarified aggregate dependency authority, final-content freeze
eligibility, affected-component alias quarantine, and foreign inherent target
ownership. Independent focused review found a computed-key readiness consumer,
which now uses exact dependency evidence; no remaining findings were reported.

Focused evidence: 13 recomputation, 18 workspace, seven alias capability,
12 canonical-image library, 10 parent library and four canonical-image
integration tests passed, plus one computed Record-key signature regression
(65 selected tests total). Core with observability and LSP checks passed.
CHECKPOINT.md records the exact targets, timings and limits. New tests cover
canonical mutation/collision safety, inherited dispatch and removal, exact
aggregate dependency recording/missing-key/mismatched-fingerprint rejection,
stale freeze closure, canonical sharing and alias cycle recovery.

P5 remains PARTIAL. Residual canonical lookup maps, persistence descriptors,
query eligibility extraction, observability closure, P4-dependent metadata
composition and final certification remain open. No `.telosc` is emitted and
cold source-built acquisition remains. Broad release suites, full-workspace
formatting and performance certification were deliberately deferred. The
updated baseline-failure-fix handoff contains the next bounded work.
