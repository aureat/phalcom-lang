import .specification as Specifications
import ..choices.data as ChoiceData
import ..core.failure as Failures
import ..core.phase as Phases
import ..core.status as StatusModule
import ..choices.provider as Providers
import ..strategies.strategy as StrategyModule

data DrawOutcome<T> {
  Value(_ value: T)
  Errored(_ reason: String)
}

data PredicateOutcome {
  Returned(_ passed: Bool)
  Raised(_ reason: String)
}

class Evaluator<T, S>
    where S |= StrategyModule.Strategy<T>
{
  /// Evaluates a single generated value and records the consumed example.
  evaluate<P>(
    specification: Specifications.PropertySpec<T, S>,
    drawData: ChoiceData.DrawData<P>,
    exampleIndex: Int,
    phase: Phases.Phase
  ) -> StatusModule.ExampleStatus<T>
      where P |= Providers.ChoiceProvider
  {
    let drawOutcome = try {
      DrawOutcome::Value(specification.strategy.draw(drawData))
    } catch error {
      DrawOutcome::Errored(error.message)
    }

    let value = match drawOutcome {
      DrawOutcome::Value(value) => value
      DrawOutcome::Errored(reason) => {
        return StatusModule.ExampleStatus::Errored(reason)
      }
    }
    let example = drawData.example

    // Generation and replay must agree on the number of consumed choices.
    if drawData.consumedChoices != example.size {
      return StatusModule.ExampleStatus::Errored(
        "provider consumption does not match the captured example"
      )
    }
    if not drawData.fullyConsumed {
      return StatusModule.ExampleStatus::Errored(
        "provider tape contains unconsumed choices"
      )
    }

    let predicateOutcome = try {
      PredicateOutcome::Returned(specification.property(value))
    } catch error {
      PredicateOutcome::Raised(error.message)
    }

    match predicateOutcome {
      PredicateOutcome::Returned(true) =>
        StatusModule.ExampleStatus::Passed(value, example)

      PredicateOutcome::Returned(false) => {
        let failure = Failures.Failure(
          exampleIndex: exampleIndex,
          phase: phase,
          origin: Failures.FailureOrigin(
            kind: #propertyReturnedFalse,
            property: specification.identity
          ),
          message: "property returned false"
        )

        StatusModule.ExampleStatus::Falsified(value, failure, example)
      }

      PredicateOutcome::Raised(reason) => {
        let failure = Failures.Failure(
          exampleIndex: exampleIndex,
          phase: phase,
          origin: Failures.FailureOrigin(
            kind: #propertyRaised,
            property: specification.identity
          ),
          message: "property raised: " + reason
        )

        StatusModule.ExampleStatus::Falsified(value, failure, example)
      }
    }
  }
}

export Evaluator
