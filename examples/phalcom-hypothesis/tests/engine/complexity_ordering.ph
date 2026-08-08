// Phase 05 — every accepted candidate is strictly smaller under the engine's
// total complexity ordering.

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import specification from "engine/specification"
import evaluator from "engine/evaluator"
import shrinker from "engine/shrinker"

const Choice = choice.Choice
const Example = example.Example
const seedExample = Example.from(
  choices: const [
    Choice.integer(value: 80, min: 0, max: 100, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 8
)
const spec = specification.PropertySpec.check(
  id: #complexityOrdering,
  target: |n| { Assert.isTrue(n < 10) },
  strategies: const [Gen.int(min: 0, max: 100)],
  explicitExamples: const [],
  reuseExamples: const [seedExample],
  settings: Settings.standard.phases(const [Phase.Reuse, Phase.Shrink])
)
const worker = evaluator._Evaluator.new(spec)
const initial = worker.replay(seedExample).status
const shrinking = shrinker.Shrinker.standard
shrinking.shrinkFailure(
  initial: initial,
  evaluator: worker,
  maxShrinks: 1000,
  statistics: None
)

const accepted = shrinking.acceptedComplexities
let index = 1
while index < accepted.size {
  Assert.isTrue(accepted.at(index).lessThan(accepted.at(index - 1)))
  index++
}

System.print("PASS engine complexity ordering")
