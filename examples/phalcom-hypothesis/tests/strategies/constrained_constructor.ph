// Phase 10: arbitrary constructor contracts are never translated into filters.

import Assert from hypothesis
import StrategyRegistry from hypothesis
import StrategyResolutionError from hypothesis
import arbitrary from hypothesis

@arbitrary
@immutable
class Interval {
  const _start: Int
  const _end: Int

  @constructor
  @requires(start <= end)
  new(start: Int, end: Int) {
    _start = start
    _end = end
  }
}

const outcome = { StrategyRegistry.standard.forType(Interval) }.attempt()
outcome.match(
  ok: |_| { Assert.fail("expected constrained derivation to fail") },
  error: |error| {
    Assert.true(error.isA(StrategyResolutionError))
    Assert.true(error.message.unwrap.includes("constrained constructor"))
    Assert.true(error.message.unwrap.includes("custom strategy"))
  }
)
