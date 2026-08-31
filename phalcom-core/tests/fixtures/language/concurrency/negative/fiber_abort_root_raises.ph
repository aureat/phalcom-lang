// area: concurrency
// spec: concurrency.md §2 rule 7; §6; ADR-0030
// status: NEGATIVE
// U-FIBER reviewer follow-on #1: `Fiber.abort(_)` had no root-fiber guard,
// so aborting the root propagated the given error straight to the host
// instead of raising the spec §2-rule-7/§6 "cannot abort the root fiber"
// runtime error — mirrors the existing `Fiber.yield` root guard.

Fiber.abort("boom")
