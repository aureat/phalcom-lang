from ..choices.example import Example
from ..choices.provider import ReplayChoiceProvider
from ..choices.data import DrawData
from ..choices.buffer import ChoiceBuffer
from ..core.failure import Failure
from ..core.result import PropertyResult
from ..core.statistics import Statistics
from ..core.phase import Phase
from ..core.status import ExampleStatus
from ..property.specification import PropertySpec
from ..property.evaluator import Evaluator
from ..strategies.strategy import Strategy
from .complexity import ExampleComplexity
from .shrink_pass import IntegerShrinkPass

/// Reduces a falsifying example while preserving its failure origin.
class Shrinker<T, S>
    where S |= Strategy<T>
{
  _pass: IntegerShrinkPass
  _specification: PropertySpec<T, S>

  @constructor
  new(specification: PropertySpec<T, S>, pass: IntegerShrinkPass) {
    _specification = specification
    _pass = pass
  }

  /// Searches smaller choice tapes and confirms the final failure twice.
  shrink(
    initialValue: T,
    initialFailure: Failure,
    initialExample: Example,
    maxShrinkSteps: Int,
    generatedExamples: Int,
    initialDiscards: Int
  ) -> PropertyResult<T>
  {
    let evaluator = Evaluator<T, S>.new()
    let mut currentValue = initialValue
    let mut currentFailure = initialFailure
    let mut currentExample = initialExample
    let mut accepted = 0
    let mut evaluations = 0
    let mut discards = initialDiscards
    let mut changed = true

    while changed and accepted < maxShrinkSteps {
      changed = false
      let currentComplexity = ExampleComplexity.of(
        example: currentExample
      )
      let proposals = self.candidates(example: currentExample)
      let mut proposalIndex = 0

      while proposalIndex < proposals.size and not changed {
        let proposal = proposals[proposalIndex]

        if ExampleComplexity.of(example: proposal).lessThan(
          other: currentComplexity
        ) {
          let drawData = DrawData<ReplayChoiceProvider>.new(
            provider: ReplayChoiceProvider.new(example: proposal),
            buffer: ChoiceBuffer.new()
          )

          match evaluator.evaluate(
            specification: _specification,
            drawData: drawData,
            exampleIndex: initialFailure.exampleIndex,
            phase: Phase::Shrink
          ) {
            ExampleStatus::Falsified(
              value,
              failure,
              replayedExample
            ) => {
              evaluations += 1

              if failure.sameOrigin(other: currentFailure) {
                currentValue = value
                currentFailure = failure
                currentExample = replayedExample
                accepted += 1
                changed = true
              }
            }
            ExampleStatus::Passed(_, _) => {
              evaluations += 1
            }
            ExampleStatus::Discarded(_) => {
              evaluations += 1
              discards += 1
            }
            ExampleStatus::Errored(_) => {
              evaluations += 1
            }

            _ => {
              evaluations += 1
            }
          }
        }

        proposalIndex += 1
      }
    }

    // Replaying twice distinguishes a stable minimized failure from a flaky one.
    let verifyDataA = DrawData<ReplayChoiceProvider>.new(
      provider: ReplayChoiceProvider.new(example: currentExample),
      buffer: ChoiceBuffer.new()
    )
    let verifyA = evaluator.evaluate(
      specification: _specification,
      drawData: verifyDataA,
      exampleIndex: initialFailure.exampleIndex,
      phase: Phase::Verify
    )

    let verifyDataB = DrawData<ReplayChoiceProvider>.new(
      provider: ReplayChoiceProvider.new(example: currentExample),
      buffer: ChoiceBuffer.new()
    )
    let verifyB = evaluator.evaluate(
      specification: _specification,
      drawData: verifyDataB,
      exampleIndex: initialFailure.exampleIndex,
      phase: Phase::Verify
    )

    if match verifyA {
      ExampleStatus::Discarded(_) => true
      _ => false
    } {
      discards += 1
    }
    if match verifyB {
      ExampleStatus::Discarded(_) => true
      _ => false
    } {
      discards += 1
    }

    let statistics = Statistics(
      generatedExamples: generatedExamples,
      shrinkEvaluations: evaluations,
      acceptedShrinks: accepted,
      discards: discards,
      verificationReplays: 2
    )

    match verifyA {
      ExampleStatus::Falsified(_, failureA, _) => {
        match verifyB {
          ExampleStatus::Falsified(_, failureB, _) => {
            if failureA.sameOrigin(other: currentFailure) and
              failureB.sameOrigin(other: currentFailure) {
              PropertyResult<T>::Falsified(
                currentValue,
                failureA,
                currentExample,
                statistics
              )
            } else {
              PropertyResult<T>::Flaky(currentValue, currentExample, statistics)
            }
          }

          ExampleStatus::Errored(reason) =>
            PropertyResult<T>::Errored(reason, statistics)

          _ => PropertyResult<T>::Flaky(currentValue, currentExample, statistics)
        }
      }

      ExampleStatus::Passed(_, _) =>
        PropertyResult<T>::Flaky(currentValue, currentExample, statistics)

      ExampleStatus::Errored(reason) =>
        PropertyResult<T>::Errored(reason, statistics)

      _ => PropertyResult<T>::Flaky(currentValue, currentExample, statistics)
    }
  }

  /// Builds one-choice-at-a-time candidate tapes for the configured pass.
  candidates(example: Example) -> List<Example> {
    let mut output = List.new()
    let mut index = 0

    while index < example.size {
      let choices = _pass.candidates(choice: example.at(index: index))
      let mut candidateIndex = 0

      while candidateIndex < choices.size {
        output.append(
          example.replace(index: index, choice: choices[candidateIndex])
        )
        candidateIndex += 1
      }

      index += 1
    }

    output
  }
}

export Shrinker
