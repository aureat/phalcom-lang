import ..choices.example as Examples
import ..choices.provider as Providers
import ..choices.data as ChoiceData
import ..choices.buffer as Buffers
import ..core.failure as Failures
import ..core.result as Results
import ..core.statistics as StatisticsModule
import ..core.phase as Phases
import ..core.status as StatusModule
import ..property.specification as Specifications
import ..property.evaluator as Evaluators
import ..strategies.strategy as StrategyModule
import .complexity as Complexities
import .shrink_pass as Passes

/// Reduces a falsifying example while preserving its failure origin.
class Shrinker {
  _pass: Passes.IntegerShrinkPass

  @constructor
  new(pass: Passes.IntegerShrinkPass) {
    _pass = pass
  }

  /// Searches smaller choice tapes and confirms the final failure twice.
  shrink<T, S>(
    specification: Specifications.PropertySpec<T, S>,
    initialValue: T,
    initialFailure: Failures.Failure,
    initialExample: Examples.Example,
    maxShrinkSteps: Int,
    generatedExamples: Int,
    initialDiscards: Int
  ) -> Results.PropertyResult<T>
      where S |= StrategyModule.Strategy<T>
  {
    let evaluator = Evaluators.Evaluator<T, S>.new()
    let mut currentValue = initialValue
    let mut currentFailure = initialFailure
    let mut currentExample = initialExample
    let mut accepted = 0
    let mut evaluations = 0
    let mut discards = initialDiscards
    let mut changed = true

    while changed and accepted < maxShrinkSteps {
      changed = false
      let currentComplexity = Complexities.ExampleComplexity.of(currentExample)
      let proposals = self.candidates(currentExample)
      let mut proposalIndex = 0

      while proposalIndex < proposals.size and not changed {
        let proposal = proposals[proposalIndex]

        if Complexities.ExampleComplexity.of(proposal).lessThan(
          currentComplexity
        ) {
          let drawData = ChoiceData.DrawData<Providers.ReplayChoiceProvider>.new(
            provider: Providers.ReplayChoiceProvider.new(example: proposal),
            buffer: Buffers.ChoiceBuffer.new()
          )

          match evaluator.evaluate(
            specification,
            drawData,
            initialFailure.exampleIndex,
            Phases.Phase::Shrink
          ) {
            StatusModule.ExampleStatus::Falsified(
              value,
              failure,
              replayedExample
            ) => {
              evaluations += 1

              if failure.sameOrigin(currentFailure) {
                currentValue = value
                currentFailure = failure
                currentExample = replayedExample
                accepted += 1
                changed = true
              }
            }
            StatusModule.ExampleStatus::Passed(_, _) => {
              evaluations += 1
            }
            StatusModule.ExampleStatus::Discarded(_) => {
              evaluations += 1
              discards += 1
            }
            StatusModule.ExampleStatus::Errored(_) => {
              evaluations += 1
            }
          }
        }

        proposalIndex += 1
      }
    }

    // Replaying twice distinguishes a stable minimized failure from a flaky one.
    let verifyDataA = ChoiceData.DrawData<Providers.ReplayChoiceProvider>.new(
      provider: Providers.ReplayChoiceProvider.new(example: currentExample),
      buffer: Buffers.ChoiceBuffer.new()
    )
    let verifyA = evaluator.evaluate(
      specification,
      verifyDataA,
      initialFailure.exampleIndex,
      Phases.Phase::Verify
    )

    let verifyDataB = ChoiceData.DrawData<Providers.ReplayChoiceProvider>.new(
      provider: Providers.ReplayChoiceProvider.new(example: currentExample),
      buffer: Buffers.ChoiceBuffer.new()
    )
    let verifyB = evaluator.evaluate(
      specification,
      verifyDataB,
      initialFailure.exampleIndex,
      Phases.Phase::Verify
    )

    if match verifyA {
      StatusModule.ExampleStatus::Discarded(_) => true
      _ => false
    } {
      discards += 1
    }
    if match verifyB {
      StatusModule.ExampleStatus::Discarded(_) => true
      _ => false
    } {
      discards += 1
    }

    let statistics = StatisticsModule.Statistics(
      generatedExamples: generatedExamples,
      shrinkEvaluations: evaluations,
      acceptedShrinks: accepted,
      discards: discards,
      verificationReplays: 2
    )

    match verifyA {
      StatusModule.ExampleStatus::Falsified(value, failureA, _) => {
        match verifyB {
          StatusModule.ExampleStatus::Falsified(_, failureB, _) => {
            if failureA.sameOrigin(currentFailure) and
              failureB.sameOrigin(currentFailure) {
              Results.PropertyResult::Falsified(
                value,
                failureA,
                currentExample,
                statistics
              )
            } else {
              Results.PropertyResult::Flaky(value, currentExample, statistics)
            }
          }

          StatusModule.ExampleStatus::Errored(reason) =>
            Results.PropertyResult::Errored(reason, statistics)

          _ => Results.PropertyResult::Flaky(value, currentExample, statistics)
        }
      }

      StatusModule.ExampleStatus::Passed(_, _) =>
        Results.PropertyResult::Flaky(currentValue, currentExample, statistics)

      StatusModule.ExampleStatus::Errored(reason) =>
        Results.PropertyResult::Errored(reason, statistics)

      _ => Results.PropertyResult::Flaky(currentValue, currentExample, statistics)
    }
  }

  /// Builds one-choice-at-a-time candidate tapes for the configured pass.
  candidates(example: Examples.Example) -> List<Examples.Example> {
    let mut output = List.new()
    let mut index = 0

    while index < example.size {
      let choices = _pass.candidates(example.at(index))
      let mut candidateIndex = 0

      while candidateIndex < choices.size {
        output.add(example.replace(index, choices[candidateIndex]))
        candidateIndex += 1
      }

      index += 1
    }

    output
  }
}

export Shrinker
