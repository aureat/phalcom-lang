// Phase 05 — find uses the shared engine and shrinker, returns the minimal
// satisfying value, and never encodes success as an exception.

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import engine from "engine/engine"

const seedExample = example.Example.from(
  choices: const [
    choice.Choice.integer(value: 40, min: 0, max: 100, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 6
)
const found = engine.SearchEngine.new().find(
  strategy: Gen.int(min: 0, max: 100),
  predicate: { n => n >= 10 },
  settings: Settings.standard.phases(const [Phase.Reuse, Phase.Shrink]),
  reuseExamples: const [seedExample]
)
Assert.equal(Some.new(10), found)

System.print("PASS engine find minimal")
