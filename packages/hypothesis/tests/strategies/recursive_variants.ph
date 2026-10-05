// Phase 10: recursive variants use the existing size-aware recursive combinator.

import Assert from hypothesis
import StrategyRegistry from hypothesis
import arbitrary from hypothesis
import data from "choices/data"

@arbitrary
@data
@sealed
class Expression {
  @variant Literal(value: Int)
  @variant Negate(value: Expression)
  @variant Add(left: Expression, right: Expression)
}

const expression = StrategyRegistry.standard.forType(Expression).draw(
  data.DrawData.generate(
    random: Random.new(seed: 10),
    generationSize: 0,
    maxChoices: 32
  )
)
Assert.true(expression.isA(Expression))
Assert.true(StrategyRegistry.standard.forType(Expression).fingerprint.includes("recursive-sealed"))
