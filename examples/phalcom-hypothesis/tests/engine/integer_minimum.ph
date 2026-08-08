// Phase 05 — structural integer shrinking preserves the property and finds
// the exact boundary counterexample rather than merely the shrink target.

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import specification from "engine/specification"
import engine from "engine/engine"

const Choice = choice.Choice
const Example = example.Example
const PropertySpec = specification.PropertySpec
const SearchEngine = engine.SearchEngine

const seedExample = Example.from(
  choices: const [
    Choice.integer(value: 57, min: 0, max: 100, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 7
)
const settings = Settings.standard.phases(
  const [Phase.Reuse, Phase.Shrink]
)
const spec = PropertySpec.check(
  id: #integerMinimum,
  target: |n| { Assert.isTrue(n < 10) },
  strategies: const [Gen.int(min: 0, max: 100)],
  explicitExamples: const [],
  reuseExamples: const [seedExample],
  settings: settings
)

const result = SearchEngine.new().check(spec)
Assert.equal(10, result.args.at(0))
Assert.equal(10, result.tape.unwrap.at(0).value)

System.print("PASS engine integer minimum")
