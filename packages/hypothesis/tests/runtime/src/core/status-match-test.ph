import hypothesis.core.failure as Failures
import hypothesis.core.phase as Phases
from hypothesis.core.status import ExampleStatus
import hypothesis.choices.choice as Choices
import hypothesis.choices.example as Examples
import hypothesis_tests.support.test as Testing

let choices: List<Choices.Choice> = List.new()
let example = Examples.Example.new(choices: choices)
let failure = Failures.Failure(
  exampleIndex: 0,
  phase: Phases.Phase.Generate,
  origin: #property,
  message: "expected failure"
)

let passed: ExampleStatus<Int> = ExampleStatus::Passed(7, example)
match passed {
  ExampleStatus::Passed(value, recorded) => {
    Testing.Test.expect(value == 7, "passed status must retain its value")
    Testing.Test.expect(recorded.size == 0, "passed status must retain its example")
  }
  ExampleStatus::Discarded(_) => { throw Error.new("expected passed status") }
  ExampleStatus::Falsified(_, _, _) => { throw Error.new("expected passed status") }
}

let discarded: ExampleStatus<Int> = ExampleStatus::Discarded("assumption")
match discarded {
  ExampleStatus::Passed(_, _) => { throw Error.new("expected discarded status") }
  ExampleStatus::Discarded(reason) =>
    Testing.Test.expect(reason == "assumption", "discarded status must retain its reason")
  ExampleStatus::Falsified(_, _, _) => { throw Error.new("expected discarded status") }
}

let falsified: ExampleStatus<Int> = ExampleStatus::Falsified(7, failure, example)
match falsified {
  ExampleStatus::Passed(_, _) => { throw Error.new("expected falsified status") }
  ExampleStatus::Discarded(_) => { throw Error.new("expected falsified status") }
  ExampleStatus::Falsified(value, observed, recorded) => {
    Testing.Test.expect(value == 7, "falsified status must retain its value")
    Testing.Test.expect(observed.sameOrigin(failure), "falsified status must retain the failure")
    Testing.Test.expect(recorded.size == 0, "falsified status must retain its example")
  }
}

Testing.Test.pass("core.status_match")
