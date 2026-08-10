// area: errors
// spec: contract-annotations.md, ADR-0052 fiber-switch hazard
// status: NEGATIVE
// BUG (found, not yet fixed): `Fiber.yield` inside an `@invariant`-guarded
// method body currently hard-errors, because `__invariantEnter`/
// `__invariantExit` are native primitives wrapping the body call, and the
// interpreter forbids a fiber switch across a native call frame (same
// restriction as `.each || { }`). ADR-0052 never named this hazard. This test
// pins CURRENT (broken) behavior so a fix is visible as a diff here, not a
// silent regression.

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

fiberA.call()
