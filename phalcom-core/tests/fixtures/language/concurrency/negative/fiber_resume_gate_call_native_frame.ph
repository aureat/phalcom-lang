// area: concurrency
// spec: concurrency.md §6; ADR-0030 §4
// status: NEGATIVE
// U-FIBER reviewer follow-on #2: `Fiber#call` attempted underneath a native
// re-entrant call frame is rejected by the restricted-switch guard — the
// diagnostic names the actual violated action (a resume, not a yield).
// Under C2.P1-R1 native suspension continuations, `.on(_)` and `.ensure(_)`
// are suspension-transparent; the restricted-switch guard preserves isolation
// for genuinely synchronous host algorithms (Map/Set hashing/equality).

class Probe {
  hash {
    const f = Fiber.new || { 1 }
    f.call()
    return 0
  }
}

let s = Set.new()
s.add(Probe.new())
