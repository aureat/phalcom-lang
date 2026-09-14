---
plan: LANG005.C4.P2
checkpoint: LANG005.C4
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
next_plan: LANG005.C4.P3
---

# Walkthrough — LANG005.C4.P2 Witness Resolution, Defaults, and Evidence

## Result

P2 is complete and focused-tested for the source witness/default plan, exact
evidence boundary, terminal applicability states, C2 applicability evidence,
and incremental parity. Conformance members now have source-owned callable
identities, signatures, body analyses, deterministic source selection, and a
snapshot-owned `ConformanceWitnessPlan`. Exact evidence performs unique P1
head lookup, requires a complete source plan, binds the exact impl
environment, and never reselects witnesses.

## Implemented

1. `CallableOwnerId::Conformance(ImplId)` distinguishes witness callables from
   target-owned inherent and trait-owned default callables.
2. Cold and incremental semantic-shard body fingerprints use the same
   conformance owner and `ImplId` identity.
3. Conformance signatures and bodies are published from `BehaviorMember`
   source, checked with target-relative `Self`, and kept out of ordinary
   declaration surfaces and dispatch return projections.
4. Explicit members are matched to `TraitRequirementId`; unmatched,
   duplicate, and bodyless members diagnose and invalidate the source plan.
5. `InstantiatedTraitRequirement` projects trait parameters and `Self` through
   one `TypeEnvironment`/`TypeView` path without mutating `TraitSurface`.
6. `WitnessCompatibility` preserves relation terminal states and checks
   callable variance, generic shape, directional generic constraints, rest
   role, and explicit visibility coverage; data components use a
   return-contract check without synthetic getters.
7. Source plan selection implements explicit witness, compatible effective
   inherent callable, readable data component, and trait-default precedence.
8. The effective inherent query layers ordinary/inherited dispatch over the
   existing exact-case and conditional applicability product, preserving
   terminal applicability states instead of reducing them to absence.
9. Conditional inherent selections retain canonical C2 specialization
   evidence, and exact evidence specializes that product through the exact
   conformance environment.
10. Source plans and exact evidence publish semantic fingerprints; exact data
   selections specialize target declaration parameters.
11. `SemanticSnapshot` owns source plans and exposes exact
   `ConformanceEvidence` resolution from `ConformanceIndex` plus the selected
   plan.

## Focused evidence

- semantic trait lane: 16 passed;
- semantic impl/conformance lane: 70 passed;
- terminal conditional-applicability regression: passed;
- directional generic-constraint regression: passed;
- conformance plan/evidence/data/default/bodyless/unmatched tests: passed;
- instantiated requirement, incompatible witness, exact generic data, and
  fingerprint assertions: passed;
- semantic test compilation/check: passed;
- format check: passed.

## Remaining P2 work

None. P3 ordinary trait-evidenced dispatch, lowering, and runtime behavior
remain out of scope for this plan.
