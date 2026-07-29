// Phase 05 — final replay that does not reproduce the same failure is an
// engine error, never a falsified property.

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import errors from "core/errors"
import choice from "choices/choice"
import example from "choices/example"
import specification from "engine/specification"
import engine from "engine/engine"

class FlakyTarget {
  @constructor
  new() {
    _calls = 0
  }

  invoke(arguments: List<Any>) -> None {
    _calls++
    if _calls == 1 {
      throw Error.new("first execution only")
    }
  }
}

const seedExample = example.Example.from(
  choices: const [
    choice.Choice.integer(value: 1, min: 0, max: 1, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 1
)
const spec = specification.PropertySpec.check(
  id: #flakyVerification,
  target: FlakyTarget.new(),
  strategies: const [Gen.int(min: 0, max: 1)],
  explicitExamples: const [],
  reuseExamples: const [seedExample],
  settings: Settings.standard.phases(const [Phase.Reuse])
)
const result = engine.SearchEngine.new().check(spec)
Assert.isTrue(result.match(
  passed: { _ => false },
  falsified: { _ => false },
  inconclusive: { _ => false },
  errored: { value => value.error.isA(errors._FlakyFailure) }
))

System.print("PASS engine flaky verification")
