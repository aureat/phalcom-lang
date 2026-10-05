import ..core.settings as SettingsModule
import ..core.statistics as StatisticsModule
import ..core.result as Results
import ..choices.data as ChoiceData
import ..choices.provider as Providers
import ..choices.buffer as Buffers
import .specification as Specifications
import .evaluator as Evaluators
import ..core.status as StatusModule
import ..engine.shrinker as Shrinkers
import ..engine.shrink_pass as ShrinkPasses

/// Evaluates examples until the budget is used or a failure is found.
class SearchEngine<T> {
  /// Runs bounded generation and shrinks the first falsifying example.
  run(
    specification: Specifications.PropertySpec<T>,
    settings: SettingsModule.Settings,
    factory: Providers.ChoiceProviderFactory
  ) -> Results.PropertyResult<T> {
    let evaluator = Evaluators.Evaluator<T>.new()
    let mut exampleIndex = 0
    let mut examples = 0
    let mut discards = 0
    let mut last: Option<T> = None

    if settings.maxExamples <= 0 {
      let statistics = StatisticsModule.Statistics(
        examples: 0,
        shrinks: 0,
        discards: 0
      )

      return Results.PropertyResult::GaveUp(
        "maxExamples must be positive",
        statistics
      )
    }

    while exampleIndex < settings.maxExamples {
      let provider = factory.create(exampleIndex)
      let drawData = ChoiceData.DrawData.new(
        provider: provider,
        buffer: Buffers.ChoiceBuffer.new()
      )
      let status = evaluator.evaluate(specification, drawData, exampleIndex)
      examples += 1

      match status {
        StatusModule.ExampleStatus::Passed(value, _) => {
          last = Some(value)
        }

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
            settings.maxShrinks,
            examples
          )
        }
      }

      exampleIndex += 1
    }

    let statistics = StatisticsModule.Statistics(
      examples: examples,
      shrinks: 0,
      discards: discards
    )

    match last {
      Some(value) => Results.PropertyResult::Passed(value, statistics)
      None => Results.PropertyResult::GaveUp(
        "no non-discarded examples were evaluated",
        statistics
      )
    }
  }
}

export SearchEngine
