// First-class in-process reproduction tokens. The token retains the exact
// immutable example and original settings; replay only restricts phases to
// reuse so no new random example can replace the recorded one.

import Example from "choices/example"
import Settings from "core/settings"
import Phase from "core/phase"
import engineSpec from "engine/specification"
import engineSearch from "engine/engine"
import propertyTarget from "property/target"
import reportingReporter from "reporting/reporter"

@data
@immutable
class ReproductionToken {
  const _propertyId: Any
  const _example: Example
  const _settings: Settings
  const _text: String

  @class
  create(
    propertyId: Any,
    example: Example,
    settings: Settings
  ) -> ReproductionToken {
    return ReproductionToken.new(
      propertyId: propertyId,
      example: example,
      settings: settings,
      text: _ReproductionText.token(
        propertyId: propertyId,
        example: example,
        settings: settings
      )
    )
  }

  toString -> String { _text }
}

class Reproduction {
  @class
  statefulExecutable(failure: Any) -> Option<String> {
    if failure.error.respondsTo(#statefulScenario) {
      return Some.new(failure.error.statefulScenario.executable)
    }
    return None
  }

  @class
  fromRun(run: Any) -> Option<ReproductionToken> {
    if run.explicitFailure || {
      return None
    }

    return run.result.match(
      passed: |_| { None },
      falsified: |value| {
        Some.new(
          ReproductionToken.create(
            propertyId: run.id,
            example: value.failure.example,
            settings: run.settings
          )
        )
      },
      inconclusive: |_| { None },
      errored: |_| { None }
    )
  }

  @class
  replay(
    token: ReproductionToken,
    strategies: List<Any>,
    target: Any
  ) -> Any {
    const replaySettings = token.settings
      .phases(const [Phase.Reuse])
      .maxShrinks(0)

    const spec = engineSpec.PropertySpec.check(
      id: token.propertyId,
      target: propertyTarget._BlockTarget.new(target),
      strategies: strategies,
      explicitExamples: const [],
      reuseExamples: const [token.example],
      parameterNames: const [],
      settings: replaySettings
    )

    return engineSearch.SearchEngine.new().check(
      spec,
      reporter: reportingReporter.NullReporter.new()
    )
  }
}

class _ReproductionText {
  @class
  token(
    propertyId: Any,
    example: Example,
    settings: Settings
  ) -> String {
    return "phalcom-hypothesis:v1:" +
      propertyId.toString + ":" +
      settings.maxChoices.toString + ":" +
      settings.maxShrinks.toString + ":" +
      example.signature
  }
}
