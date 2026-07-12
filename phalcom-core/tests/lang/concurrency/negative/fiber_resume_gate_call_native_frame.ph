// area: concurrency
// spec: concurrency.md §6; ADR-0030 §4
// status: NEGATIVE
// U-FIBER reviewer follow-on #2: `Fiber#call` attempted underneath a native
// re-entrant call frame (here, `List#each`'s block-call) is rejected by the
// restricted-switch guard — the diagnostic names the actual violated action
// (a resume, not a yield) instead of reusing the yield-specific wording
// `Fiber.yield` gets for the same underlying restriction.

let f = Fiber.new { 1 }
[1].each { x => f.call() }
