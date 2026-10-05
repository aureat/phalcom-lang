import .specification as Specifications
import ..choices.data as ChoiceData
import ..core.failure as Failures
import ..core.phase as Phases
import ..core.status as StatusModule

class Evaluator<T> {
  /// Evaluates a single generated value and records the consumed example.
  evaluate(
    specification: Specifications.PropertySpec<T>,
    drawData: ChoiceData.DrawData,
    exampleIndex: Int
  ) -> StatusModule.ExampleStatus<T> {
    let value = specification.strategy.draw(drawData)
    let example = drawData.example

    // Generation and replay must agree on the number of consumed choices.
    if drawData.consumedChoices != example.size {
      throw Error.new("provider consumption does not match the captured example")
    }

    if specification.property(value) {
      StatusModule.ExampleStatus::Passed(value, example)
    } else {
      let failure = Failures.Failure(
        exampleIndex: exampleIndex,
        phase: Phases.Phase::Generate,
        origin: #propertyReturnedFalse,
        message: "property returned false"
      )

      StatusModule.ExampleStatus::Falsified(value, failure, example)
    }
  }
}

export Evaluator
