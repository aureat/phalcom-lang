---
id: TEST002.C1.P2
category: TEST
program: TEST002
checkpoint: TEST002.C1
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - TEST002.C1.P1
---

# TEST002.C1.P2 — Generic Override Compatibility and Symbolic Inheritance Closure

## Goal

Restore the authoritative BL-19 symbolic inherited-capability regression to an analyzable state without weakening the new ordinary override-certification surface or bypassing it for synthetic fixtures.

## Starting state

`TEST002.C1.P1` completed BL-17 and BL-18. It also constrained the newly introduced override validator to source declarations whose provider provenance is the canonical Universe URI; actual canonical Universe support contracts are therefore not evaluated by the new user-facing relation. Ordinary synthetic fixtures remain evaluated.

BL-19 cannot reach its intended `Monad -> Applicative -> Functor` symbolic-evidence trace because generic `Monad`/`Applicative` methods in the typing fixture are rejected first by four `IncompatibleOverride` diagnostics (`map`, `pure`, `map2`, and `flatMap`). Migrating the fixture from `ModuleId::universe_root()` to a synthetic module identity did not change those diagnostics. The migration was reverted.

## Fixed invariants

- BL-19 remains authoritative: `Functor.map` must resolve through inherited generic capability traversal with `Assumed` symbolic evidence.
- Do not weaken the test, mark it Dynamic, add `Monad`/`Functor` name cases, or bypass generic inference.
- Do not suppress override validation for ordinary synthetic source or selectors.
- Canonical Universe support exclusion remains provenance-based, not a loose module/path/name heuristic.
- Any generic override relation must preserve ordinary parameter contravariance, return covariance, callable identity, and accepted constraint semantics.

## Tasks

### T1 — Specify and isolate generic override compatibility

Trace the first incorrect compatibility decision for the monad fixture's `map`, `pure`, `map2`, and `flatMap` overrides. Compare their declared generic signatures, inherited specialized signatures, constraints, type-parameter owners, and variance relations. Produce an accepted implementation decision before modifying the compatibility relation.

### T2 — Repair the owning generic override relation

Implement only the general compatibility rule identified by T1. Preserve the existing user-level override probes and add focused coverage for the generic relation. Do not introduce fixture, selector, or capability-name exceptions.

### T3 — Resume the original BL-19 evidence trace

With the fixture free of unrelated override diagnostics, reproduce the `Functor.map` failure, trace the first `Known(symbolic, Assumed) -> Unknown(NoTypeEvidence)` transition, and repair the owning general specialization/inference rule. Run the BL-19 test and nearby monad body regressions.

### T4 — Complete checkpoint certification

Run the remaining G4 coverage and the one-time G5 semantic/core broad targets. Classify any new failures under the baseline policy and complete TEST002.C1 records only if BL-01 through BL-19 are resolved or correctly migrated.

**Disposition:** Deferred to the checkpoint's remaining certification work. P2's
generic override and BL-19 acceptance objective is complete; it does not own a
second broad-gate run for unrelated P1 changes.

## Required first commands

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core \
  traverse_body_resolves_symbolic_inherited_capabilities -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  audit_probes::override_certification -- --nocapture
```

## STOP / CONSULT

Stop before editing if a correct generic override relation requires a new published type representation, changes variance/constraint semantics, or reveals that the fixture's expected inheritance behavior conflicts with accepted language semantics.
