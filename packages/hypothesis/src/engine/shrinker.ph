import ..choices.example as Examples
import ..choices.provider as Providers
import ..choices.data as ChoiceData
import ..choices.buffer as Buffers
import ..core.failure as Failures
import ..core.result as Results
import ..core.statistics as StatisticsModule
import ..core.status as StatusModule
import ..property.specification as Specifications
import ..property.evaluator as Evaluators
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
  shrink<T>(
    specification: Specifications.PropertySpec<T>,
    initialValue: T,
    initialFailure: Failures.Failure,
    initialExample: Examples.Example,
    maxShrinks: Int,
    exampleCount: Int
  ) -> Results.PropertyResult<T> {
    let evaluator = Evaluators.Evaluator<T>.new()
    let mut currentValue = initialValue
    let mut currentFailure = initialFailure
    let mut currentExample = initialExample
    let mut accepted = 0
    let mut evaluations = 0
    let mut changed = true

    while changed and accepted < maxShrinks {
      changed = false
      let currentComplexity = Complexities.ExampleComplexity.of(currentExample)
      let proposals = self.candidates(currentExample)
      let mut proposalIndex = 0

      while proposalIndex < proposals.size and not changed {
        let proposal = proposals[proposalIndex]

        if Complexities.ExampleComplexity.of(proposal).lessThan(
          currentComplexity
        ) {
          let drawData = ChoiceData.DrawData.new(
            provider: Providers.ReplayChoiceProvider.new(example: proposal),
            buffer: Buffers.ChoiceBuffer.new()
          )

          match evaluator.evaluate(
            specification,
            drawData,
            initialFailure.exampleIndex
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
            _ => {
              evaluations += 1
            }
          }
        }

        proposalIndex += 1
      }
    }

    // Replaying twice distinguishes a stable minimized failure from a flaky one.
    let verifyDataA = ChoiceData.DrawData.new(
      provider: Providers.ReplayChoiceProvider.new(example: currentExample),
      buffer: Buffers.ChoiceBuffer.new()
    )
    let verifyA = evaluator.evaluate(
      specification,
      verifyDataA,
      initialFailure.exampleIndex
    )

    let verifyDataB = ChoiceData.DrawData.new(
      provider: Providers.ReplayChoiceProvider.new(example: currentExample),
      buffer: Buffers.ChoiceBuffer.new()
    )
    let verifyB = evaluator.evaluate(
      specification,
      verifyDataB,
      initialFailure.exampleIndex
    )

    evaluations += 2

    let statistics = StatisticsModule.Statistics(
      examples: exampleCount + evaluations,
      shrinks: accepted,
      discards: 0
    )

    match verifyA {
      StatusModule.ExampleStatus::Falsified(value, failureA, _) => {
        match verifyB {
          StatusModule.ExampleStatus::Falsified(_, failureB, _) => {
            if failureA.sameOrigin(failureB) {
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

          _ => Results.PropertyResult::Flaky(value, currentExample, statistics)
        }
      }

      StatusModule.ExampleStatus::Passed(value, _) =>
        Results.PropertyResult::Flaky(value, currentExample, statistics)

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
