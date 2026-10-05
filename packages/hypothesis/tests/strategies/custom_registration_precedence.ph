// Phase 10: exact registrations remain the highest-precedence registry source.

import Assert from hypothesis
import Gen from hypothesis
import StrategyRegistry from hypothesis
import arbitrary from hypothesis

@arbitrary
@data
@immutable
class Coordinate {
  const _x: Int
  const _y: Int
}

const registry = StrategyRegistry.standard
registry.register(
  type: Coordinate,
  strategy: Gen.just(Coordinate.new(x: 0, y: 0))
)
Assert.equal("just(Coordinate)", registry.forType(Coordinate).fingerprint)
