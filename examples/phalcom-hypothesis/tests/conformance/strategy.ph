// Phase 11: custom strategies can inherit reusable combinators or conform structurally.

import { Assert, Strategy, StrategyBase } from "hypothesis"
import DrawData from "choices/data"
import ScriptedChoiceProvider from "choices/provider"
import Choice from "choices/choice"

class ConstantStrategy is StrategyBase<Int> {
  draw(data: DrawData) -> Int => 7
  fingerprint -> String => "constant(7)"
}

const strategy: Strategy<Int> = ConstantStrategy.new()
const mapped = strategy.map { value => value + 1 }
const data = DrawData.new(
  provider: ScriptedChoiceProvider.new(const []),
  generationSize: 0,
  maxChoices: 1
)
Assert.equal(8, mapped.draw(data))
Assert.equal("constant(7)", strategy.fingerprint)
