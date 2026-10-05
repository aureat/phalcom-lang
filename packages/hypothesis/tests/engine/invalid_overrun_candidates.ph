// Phase 05 — invalid and overrun replay candidates are ignored during
// shrinking; only a strictly smaller stable interesting candidate is accepted.

import Assert from hypothesis
import choice from "choices/choice"
import example from "choices/example"
import status from "core/status"
import failure from "core/failure"
import shrinkPass from "engine/shrink_pass"
import shrinker from "engine/shrinker"

const origin = failure.FailureOrigin.new(
  errorType: Error,
  module: #engineTests,
  selector: #candidateClassification,
  line: 1,
  column: 1,
  label: None
)

class CandidatePass {
  name -> Symbol { #candidateClassification }

  candidates(current: example.Example) -> List<example.Example> {
    const out = List.new()
    for value in const [9, 8, 7] {
      out.add(current.replace(0, current.at(0).withValue(value)))
    }
    return out
  }
}

class CandidateEvaluator {
  replay(candidate: example.Example) -> Any {
    const value = candidate.at(0).value
    if value == 9 {
      return status.ExampleStatus.invalid(
        reason: Error.new("invalid"), example: candidate,
        arguments: const [value], context: None
      )
    }
    if value == 8 {
      return status.ExampleStatus.overrun(
        reason: Error.new("overrun"), example: candidate,
        arguments: const [value], context: None
      )
    }
    return status.ExampleStatus.interesting(
      failure: failure.Failure.new(
        origin: origin,
        error: Error.new("stable"),
        example: candidate,
        arguments: const [value],
        notes: const []
      ),
      example: candidate,
      arguments: const [value],
      context: None
    )
  }
}

const initialExample = example.Example.from(
  choices: const [
    choice.Choice.integer(value: 10, min: 0, max: 10, shrinkTowards: 0, label: None)
  ],
  spans: const [],
  generationSize: 1
)
const initial = status.ExampleStatus.interesting(
  failure: failure.Failure.new(
    origin: origin,
    error: Error.new("stable"),
    example: initialExample,
    arguments: const [10],
    notes: const []
  ),
  example: initialExample,
  arguments: const [10],
  context: None
)
const worker = shrinker.Shrinker.new(passes: const [CandidatePass.new()])
const minimal = worker.shrinkFailure(
  initial: initial,
  evaluator: CandidateEvaluator.new(),
  maxShrinks: 10,
  statistics: None
)
Assert.equal(7, minimal.args.at(0))
Assert.equal(1, worker.acceptedComplexities.size)

System.print("PASS engine invalid overrun candidates")
