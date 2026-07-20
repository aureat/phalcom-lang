// area: concurrency
// spec: concurrency.md §6; ADR-0030 §4
// status: NEGATIVE
// U-FIBER reviewer follow-on #2: `Fiber#call` attempted underneath a native
// re-entrant call frame is rejected by the restricted-switch guard — the
// diagnostic names the actual violated action (a resume, not a yield).
// Post flat-entry (U-BYTES follow-on, bytes.md §3.1): `List#each`'s block
// call — this fixture's original vehicle — no longer creates a native frame
// and is legal; the guard's remaining territory is block invocation from
// inside a native primitive, here an `.on(_)` error handler.

const f = Fiber.new { 1 }
{ throw Error.new("boom") }.on(Error) { e => f.call() }
