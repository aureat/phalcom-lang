---
id: LANG007.C4.P4.R1-A1
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
recorded: 2026-10-06
implementation_revision: 09ac903b
consolidated_branch: main
---

# R1 A1 — Shared match equality authority walkthrough

A1 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`, committed on local `main`
as `09ac903b`. It started at `33916182`; TEL-1 work was preserved and subsequently
consolidated separately as `c2d72fed`.

## Implementation

Match synthesis consumes `GenericEvidenceEnvironment::refine_type`; the private
`apply_ambient_equalities` solver is removed. Known, Refined, and Unavailable
results distinguish successful refinement from typed projection uncertainty.
The query reuses the existing equivalence components and equivalence proof;
it preserves parameter owners and introduces no inference or local-rigid identity.
Parameter-only components remain symbolic. Structural cycles, cancellation,
budget exhaustion, and ambiguous concrete endpoints remain unavailable.
Coverage retains its original root on failure and calls `retain_blocked`.

Independent review identified a transitive projection gap. The shared
`concrete_refinement` helper now selects the unique concrete endpoint of a
proven component. Generic projection normalization consumes it after the
qualifying-conformance gate, retaining its existing active-cycle guard.
The plan's projection seam therefore includes a small production change in
`types/projection.rs`, rather than inspection only. No architectural boundary
was redesigned, and no consultation escalation was triggered.

## Evidence

- Pre-change exhaustiveness baseline: 29 passed, 5 existing ignored.
- Semantic crate check: passed before the final projection correction.
- Final `cargo test -p phalcom-semantic --lib refinement_tests`: 6 passed.
- Final `cargo test -p phalcom-semantic --test semantic adts::matching`:
  153 passed, 16 existing ignored.
- Source regressions assert equality-chain direction/order parity, symbolic
  parameter cycles, exact variant candidates, payload types, and concrete
  GADT mismatch impossibility. Unit tests cover projection parity under assumed
  conformance, unresolved projections, owner isolation, cycles, cancellation,
  and exhausted budgets.
- Scoped whitespace review passed and the private solver is absent.

The pre-change source suite was green: the missing shared authority and shared
API were the pre-fix observations, not a failing source reproducer. Tests were
run with `RUSTFLAGS` and `RUSTC_WRAPPER` cleared. Existing deprecation warnings
remain. No full semantic suite, workspace/release gates, or additional tests
ran during the subsequent Git consolidation. Delivery is not certification.

A2 is the next R1 issue. R1 and parent P4 remain partial; TEL-1's passing project
compile and remaining generic-rest call-site failures are recorded separately.
