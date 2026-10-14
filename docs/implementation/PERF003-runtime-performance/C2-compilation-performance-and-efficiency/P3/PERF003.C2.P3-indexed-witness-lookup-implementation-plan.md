---
id: PERF003.C2.P3
checkpoint: PERF003.C2
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# PERF003.C2.P3 — indexed witness lookup increment

Scope authorized by the user's request to continue optimizations from the attached
response, especially section 15.1. This record bounds the implemented first P3 slice;
it does not adopt all speculative settlement proposals as completed architecture.

T1: Replace per-requirement global callable-signature scans with exact
`ImplId + DispatchSide + Selector` lookup. Live `CallableId` is already exactly
`CallableOwnerId + Selector + DispatchSide`; use the existing canonical index,
without another candidate cache or first-candidate policy. IMPLEMENTED.

T2: Bound effective inherent-surface signature enumeration to the target module's
already maintained callable index, retaining exact declaration/variant filtering.
IMPLEMENTED.

T3: Preserve compatibility/body/evidence/generic rules and table edit behavior.
One focused owner/side/clone/removal regression passed, including final module-index
assertions. Broader conformance suites deferred by the user.

Remaining P3 work: instantiated requirement caching with complete dependencies,
structural versus return-sensitive witness completion, SCC inferred-return settlement,
and dependency-sensitive generic proof reuse. These are not implemented by T1/T2.
