// area: concurrency
// spec: concurrency.md; ADR-0030 §4
// status: PASS
// C-FIB-3: `Fiber.yield` under a GENUINE re-entrant native call frame raises
// `CannotYieldAcrossNativeFrame` instead of corrupting the fiber's suspended
// position. Under C2.P1-R1 native suspension continuations, `.on(_)` and
// `.ensure(_)` are suspension-transparent; the restricted-yield guard preserves
// isolation for genuinely synchronous host algorithms (Map/Set hashing/equality).

class Probe {
  hash {
    Fiber.yield(1)
    return 0
  }
}

const f = Fiber.new || {
  let s = Set.new()
  s.add(Probe.new())
}
const result = f.try()
System.print(result.class.name)
