// Phase 05 — a shrink remains interesting only when it preserves the exact
// source-aware FailureOrigin, not merely the error class.

import Assert from hypothesis
import Settings from hypothesis
import Phase from hypothesis
import Gen from hypothesis
import Failure from hypothesis
import failure from "core/failure"
import choice from "choices/choice"
import example from "choices/example"
import specification from "engine/specification"
import engine from "engine/engine"

class OriginError is Error {
  @constructor
  new(origin: failure.FailureOrigin) {
    _origin = origin
  }

  failureOrigin -> failure.FailureOrigin => _origin
}

const early = failure.FailureOrigin.new(
  errorType: OriginError,
  module: #engineTests,
  selector: #originBoundary,
  line: 10,
  column: 3,
  label: Some.new(#early)
)
const late = failure.FailureOrigin.new(
  errorType: OriginError,
  module: #engineTests,
  selector: #originBoundary,
  line: 20,
  column: 3,
  label: Some.new(#late)
)
const seedExample = example.Example.from(
  choices: const [
    choice.Choice.integer(value: 30, min: 0, max: 100, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 5
)
const spec = specification.PropertySpec.check(
  id: #originBoundary,
  target: { n =>
    if n >= 20 {
      throw OriginError.new(late)
    }
    if n >= 10 {
      throw OriginError.new(early)
    }
  },
  strategies: const [Gen.int(min: 0, max: 100)],
  explicitExamples: const [],
  reuseExamples: const [seedExample],
  settings: Settings.standard.phases(const [Phase.Reuse, Phase.Shrink])
)

const result = engine.SearchEngine.new().check(spec)
Assert.equal(20, result.args.at(0))
Assert.isTrue(result.match(
  passed: { _ => false },
  falsified: { value => value.failure.origin.sameSite(late) },
  inconclusive: { _ => false },
  errored: { _ => false }
))

System.print("PASS engine failure origin preservation")
