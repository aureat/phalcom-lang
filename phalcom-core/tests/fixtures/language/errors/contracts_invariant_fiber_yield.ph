// area: errors
// spec: contract-annotations.md, ADR-0052 fiber-switch hazard
// status: PASS
// Proves `Fiber.yield` inside an `@invariant`-guarded method conditional
// executes cleanly without native re-entry frame restriction under C2.P1-R1.

class Guard {
  @invariant(self.n >= 0)

  @constructor
  new(_ n) {
    _n = n
  }

  n { _n }

  bump(_ shouldYield) {
    if (shouldYield) {
      Fiber.yield(0)
    }
    _n = _n + 1
  }
}

const x = Guard.new(1)

const fiberA = Fiber.new || {
  x.bump(true)
}

System.print("yield: " + fiberA.call().toString)
System.print("resumed: " + fiberA.call().toString)
System.print("n: " + x.n.toString)
