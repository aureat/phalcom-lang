import ..core.settings as SettingsModule
from ..core.statistics import Statistics
from ..core.result import PropertyResult
from ..core.phase import Phase
from ..choices.data import DrawData
from ..choices.provider import ChoiceProvider, ChoiceProviderFactory
from ..choices.buffer import ChoiceBuffer
import .specification as Specifications
from .evaluator import Evaluator
from ..core.status import ExampleStatus
from ..engine.shrinker import Shrinker
from ..engine.shrink_pass import IntegerShrinkPass
import ..strategies.strategy as StrategyModule

/// Evaluates examples until the budget is used or a failure is found.
class SearchEngine<T, S>
    where S |= StrategyModule.Strategy<T>
{
  _specification: Specifications.PropertySpec<T, S>

  @constructor
  new(specification: Specifications.PropertySpec<T, S>) {
    _specification = specification
  }

  /// Runs bounded generation and shrinks the first falsifying example.
  run<P>(
    settings: SettingsModule.Settings,
    factory: ChoiceProviderFactory<P>
  ) -> PropertyResult<T>
      where
        P |= ChoiceProvider
  {
    let evaluator = Evaluator<T, S>.new()
    let mut exampleIndex = 0
    let mut examples = 0
    let mut discards = 0

    let maxExamples = settings.maxExamples
    if maxExamples <= 0 or settings.maxShrinkSteps < 0 {
      let statistics = Statistics(
        generatedExamples: 0,
        shrinkEvaluations: 0,
        acceptedShrinks: 0,
        discards: 0,
        verificationReplays: 0
      )

      return PropertyResult<T>::GaveUp(
        if maxExamples <= 0 {
          "maxExamples must be positive"
        } else {
          "maxShrinkSteps must not be negative"
        },
        statistics
      )
    }

    while exampleIndex < maxExamples {
      let provider = factory.create(exampleIndex: exampleIndex)
      let drawData = DrawData<P>.new(
        provider: provider,
        buffer: ChoiceBuffer.new()
      )
      let status = evaluator.evaluate(
        specification: _specification,
        drawData: drawData,
        exampleIndex: exampleIndex,
        phase: Phase::Generate
      )
      examples += 1

      match status {
        ExampleStatus::Passed(_, _) => ()

        ExampleStatus::Discarded(_) => {
          discards += 1
        }

        ExampleStatus::Falsified(value, failure, example) => {
          let shrinker = Shrinker.new(
            specification: _specification,
            pass: IntegerShrinkPass.new()
          )

          return shrinker.shrink(
            initialValue: value,
            initialFailure: failure,
            initialExample: example,
            maxShrinkSteps: settings.maxShrinkSteps,
            generatedExamples: examples,
            initialDiscards: discards
          )
        }

        ExampleStatus::Errored(reason) => {
          let statistics = Statistics(
            generatedExamples: examples,
            shrinkEvaluations: 0,
            acceptedShrinks: 0,
            discards: discards,
            verificationReplays: 0
          )

          return PropertyResult<T>::Errored(reason, statistics)
        }

        _ => {
          let statistics = Statistics(
            generatedExamples: examples,
            shrinkEvaluations: 0,
            acceptedShrinks: 0,
            discards: discards,
            verificationReplays: 0
          )

          return PropertyResult<T>::Errored(
            "evaluator returned an unknown status",
            statistics
          )
        }
      }

      exampleIndex += 1
    }

    let statistics = Statistics(
      generatedExamples: examples,
      shrinkEvaluations: 0,
      acceptedShrinks: 0,
      discards: discards,
      verificationReplays: 0
    )

    if discards == examples {
      PropertyResult<T>::GaveUp(
        "no non-discarded examples were evaluated",
        statistics
      )
    } else {
      PropertyResult<T>::Passed(statistics)
    }
  }
}

export SearchEngine
