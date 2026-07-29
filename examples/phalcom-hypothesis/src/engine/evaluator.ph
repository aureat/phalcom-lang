// Complete-property evaluation through generation or deterministic replay.

import DrawData from "choices/data"
import ChoiceProvider from "choices/provider"
import providers from "choices/provider"
import Example from "choices/example"
import ExampleStatus from "core/status"
import Failure from "core/failure"
import coreContext from "core/context"
import errors from "core/errors"
import search from "engine/search"

class _Evaluator {
  @constructor
  new(spec: Any) {
    _spec = spec
  }

  explicit(arguments: List<Any>) -> search._SearchResult<Any> {
    const context = coreContext._PropertyContext.new()
    const outcome = {
      coreContext._propertyContexts.with(context) {
        self.invokeTarget(arguments)
      }
    }.attempt()
    return search._SearchResult.evaluated(
      self.classify(
        outcome: outcome,
        example: Example.empty,
        arguments: arguments,
        context: context
      )
    )
  }

  generated(provider: ChoiceProvider, size: Int) -> search._SearchResult<Any> {
    return self.withData(
      DrawData.generate(
        provider: provider,
        generationSize: size,
        maxChoices: _spec.settings.maxChoices
      )
    )
  }

  // Compatibility entry for direct internal callers retained through Phase 11.
  generated(random: Random, size: Int) -> search._SearchResult<Any> {
    return self.generated(
      providers.SystemRandomChoiceProvider.new(random),
      size
    )
  }

  replay(example: Example) -> search._SearchResult<Any> {
    return self.withData(
      DrawData.replay(
        example: example,
        maxChoices: _spec.settings.maxChoices
      )
    )
  }

  withData(data: DrawData) -> search._SearchResult<Any> {
    const context = coreContext._PropertyContext.new()
    const arguments = List.new()
    const outcome = {
      for strategy in _spec.strategies {
        arguments.add(strategy.draw(data))
      }

      return coreContext._propertyContexts.with(context) {
        if _spec.findMode {
          return _spec.predicate.call(arguments.at(0))
        }
        return self.invokeTarget(arguments)
      }
    }.attempt()

    const completed = data.example
    if _spec.findMode and outcome.isOk and outcome.unwrap {
      return search._SearchResult.found(
        value: arguments.at(0),
        example: completed,
        arguments: arguments,
        context: context
      )
    }

    return search._SearchResult.evaluated(
      self.classify(
        outcome: outcome,
        example: completed,
        arguments: arguments,
        context: context
      )
    )
  }

  invokeTarget(arguments: List<Any>) -> Any {
    if _spec.target.respondsTo(#invoke) {
      return _spec.target.invoke(arguments)
    }
    return _spec.target.callWith(arguments)
  }

  classify(
    outcome: Result<Any, Error>,
    example: Example,
    arguments: List<Any>,
    context: Any
  ) -> ExampleStatus {
    if outcome.isOk {
      return ExampleStatus.valid(
        example: example,
        arguments: arguments,
        context: context
      )
    }

    const error = outcome.unwrapErr
    if error.isA(errors._RejectedExample) {
      return ExampleStatus.invalid(
        reason: error,
        example: example,
        arguments: arguments,
        context: context
      )
    }

    if error.isA(errors._EngineOverrun) or error.isA(errors._HealthCheckFailure) {
      return ExampleStatus.overrun(
        reason: error,
        example: example,
        arguments: arguments,
        context: context
      )
    }

    return ExampleStatus.interesting(
      failure: Failure.from(
        error: error,
        example: example,
        arguments: arguments,
        notes: context.notes
      ),
      example: example,
      arguments: arguments,
      context: context
    )
  }
}
