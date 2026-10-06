import .specification as Specifications
import ..choices.data as ChoiceData
from ..core.failure import Failure, FailureOrigin
from ..core.phase import Phase
from ..core.status import ExampleStatus
import ..choices.provider as Providers
from ..strategies.strategy import Strategy

data DrawOutcome<T> {
  Value(_ value: T)
  Errored(_ reason: String)
}

data PredicateOutcome {
  Returned(_ passed: Bool)
  Raised(_ reason: String)
}

class Evaluator<T, S> {
  /// Evaluates a single generated value and records the consumed example.
  evaluate<P>(
    specification: Specifications.PropertySpec<T, S>,
    drawData: ChoiceData.DrawData<P>,
    exampleIndex: Int,
    phase: Phase
  ) -> ExampleStatus<T>
      where S |= Strategy<T>, P |= Providers.ChoiceProvider
  {
    let mut drawOutcome: DrawOutcome<T> =
      DrawOutcome::Errored("strategy evaluation did not complete")
    try {
      drawOutcome = DrawOutcome::Value(
        specification.strategy.draw(drawData: drawData)
      )
    } catch error {
      drawOutcome = DrawOutcome::Errored(error.message)
    }

    let value = match drawOutcome {
      DrawOutcome::Value(value) => value
      DrawOutcome::Errored(reason) => {
        return ExampleStatus::Errored(reason)
      }
    }
    let example = drawData.example

    // Generation and replay must agree on the number of consumed choices.
    if drawData.consumedChoices != example.size {
      return ExampleStatus::Errored(
        "provider consumption does not match the captured example"
      )
    }
    if not drawData.fullyConsumed {
      return ExampleStatus::Errored(
        "provider tape contains unconsumed choices"
      )
    }

    let mut predicateOutcome: PredicateOutcome =
      PredicateOutcome::Raised("property evaluation did not complete")
    try {
      predicateOutcome = PredicateOutcome::Returned(
        specification.property(value)
      )
    } catch error {
      predicateOutcome = PredicateOutcome::Raised(error.message)
    }

    match predicateOutcome {
      PredicateOutcome::Returned(passed) => {
        if passed {
          ExampleStatus::Passed(value, example)
        } else {
          let failure = Failure(
            exampleIndex: exampleIndex,
            phase: phase,
            origin: FailureOrigin(
              kind: #propertyReturnedFalse,
              property: specification.identity
            ),
            message: "property returned false"
          )

          ExampleStatus::Falsified(value, failure, example)
        }
      }

      PredicateOutcome::Raised(reason) => {
        let failure = Failure(
          exampleIndex: exampleIndex,
          phase: phase,
          origin: FailureOrigin(
            kind: #propertyRaised,
            property: specification.identity
          ),
          message: "property raised: " + reason
        )

        ExampleStatus::Falsified(value, failure, example)
      }
    }
  }
}

export Evaluator
