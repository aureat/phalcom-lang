// Phase 10: an opt-in immutable data class derives from reflected constructor parameters.

import Assert from hypothesis
import StrategyRegistry from hypothesis
import arbitrary from hypothesis
import data from "choices/data"

@arbitrary
@data
@immutable
class Coordinate {
  const _x: Int
  const _y: Int
}

const registry = StrategyRegistry.standard
const generated = registry.forType(Coordinate).draw(
  data.DrawData.generate(
    random: Random.new(seed: 10),
    generationSize: 4,
    maxChoices: 32
  )
)
Assert.true(generated.isA(Coordinate))
Assert.true(registry.forType(Coordinate).fingerprint.includes("Coordinate"))
