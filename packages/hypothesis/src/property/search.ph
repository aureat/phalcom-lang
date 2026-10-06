import ..core.settings as SettingsModule
import ..core.statistics as StatisticsModule
import ..core.result as Results
import ..core.phase as Phases
import ..choices.data as ChoiceData
import ..choices.provider as Providers
import ..choices.buffer as Buffers
import .specification as Specifications
import .evaluator as Evaluators
import ..core.status as StatusModule
import ..engine.shrinker as Shrinkers
import ..engine.shrink_pass as ShrinkPasses
import ..strategies.strategy as StrategyModule

/// Evaluates examples until the budget is used or a failure is found.
class SearchEngine<T, S>
    where S |= StrategyModule.Strategy<T>
{
  /// Runs bounded generation and shrinks the first falsifying example.
  run<F>(
    specification: Specifications.PropertySpec<T, S>,
    settings: SettingsModule.Settings,
    factory: F
  ) -> Results.PropertyResult<T>
      where
        F |= Providers.ChoiceProviderFactory,
        F::Provider |= Providers.ChoiceProvider
  {
    let evaluator = Evaluators.Evaluator<T, S>.new()
    let mut exampleIndex = 0
    let mut examples = 0
    let mut discards = 0

    if settings.maxExamples <= 0 or settings.maxShrinkSteps < 0 {
      let statistics = StatisticsModule.Statistics(
        generatedExamples: 0,
        shrinkEvaluations: 0,
        acceptedShrinks: 0,
        discards: 0,
        verificationReplays: 0
      )

      return Results.PropertyResult::GaveUp(
        if settings.maxExamples <= 0 {
          "maxExamples must be positive"
        } else {
          "maxShrinkSteps must not be negative"
        },
        statistics
      )
    }

    while exampleIndex < settings.maxExamples {
      let provider: F::Provider = factory.create(exampleIndex)
      let drawData = ChoiceData.DrawData<F::Provider>.new(
        provider: provider,
        buffer: Buffers.ChoiceBuffer.new()
      )
      let status = evaluator.evaluate(
        specification,
        drawData,
        exampleIndex,
        Phases.Phase::Generate
      )
      examples += 1

      match status {
        StatusModule.ExampleStatus::Passed(_, _) => ()

        StatusModule.ExampleStatus::Discarded(_) => {
          discards += 1
        }

        StatusModule.ExampleStatus::Falsified(value, failure, example) => {
          let shrinker = Shrinkers.Shrinker.new(
            pass: ShrinkPasses.IntegerShrinkPass.new()
          )

          return shrinker.shrink(
            specification,
            value,
            failure,
            example,
            settings.maxShrinkSteps,
            examples,
            discards
          )
        }

        StatusModule.ExampleStatus::Errored(reason) => {
          let statistics = StatisticsModule.Statistics(
            generatedExamples: examples,
            shrinkEvaluations: 0,
            acceptedShrinks: 0,
            discards: discards,
            verificationReplays: 0
          )

          return Results.PropertyResult::Errored(reason, statistics)
        }
      }

      exampleIndex += 1
    }

    let statistics = StatisticsModule.Statistics(
      generatedExamples: examples,
      shrinkEvaluations: 0,
      acceptedShrinks: 0,
      discards: discards,
      verificationReplays: 0
    )

    if discards == examples {
      Results.PropertyResult::GaveUp(
        "no non-discarded examples were evaluated",
        statistics
      )
    } else {
      Results.PropertyResult::Passed(statistics)
    }
  }
}

export SearchEngine
