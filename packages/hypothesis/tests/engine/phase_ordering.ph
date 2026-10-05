// Phase 05 — explicit examples execute before reuse and generation. A failing
// explicit example is reported directly and is never shrunk.

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import specification from "engine/specification"
import engine from "engine/engine"

class RecordingTarget {
  @constructor
  new(calls: List<Int>) {
    _calls = calls
  }

  invoke(arguments: List<Any>) -> None {
    const value = arguments.at(0)
    _calls.add(value)
    if value == 1 {
      throw Error.new("explicit failure")
    }
  }
}

const calls = List.new()
const reused = example.Example.from(
  choices: const [
    choice.Choice.integer(value: 2, min: 0, max: 10, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 1
)
const spec = specification.PropertySpec.check(
  id: #phaseOrdering,
  target: RecordingTarget.new(calls),
  strategies: const [Gen.int(min: 0, max: 10)],
  explicitExamples: const [const [1]],
  reuseExamples: const [reused],
  settings: Settings.standard
)
const result = engine.SearchEngine.new().check(spec)
Assert.equal(const [1], calls)
Assert.equal(1, result.args.at(0))
Assert.equal(0, result.stats.shrinks)

System.print("PASS engine phase ordering")
