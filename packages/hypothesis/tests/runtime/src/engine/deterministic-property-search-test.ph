from hypothesis.choices.choice import Choice
import hypothesis.choices.provider as Providers
from hypothesis.core.phase import Phase
from hypothesis.core.result import PropertyResult
from hypothesis.core.settings import Settings
from hypothesis.property.property import Property
from hypothesis.property.specification import PropertySpec
from hypothesis.strategies.primitives import IntStrategy
import hypothesis_tests.support.test as Testing

let mut script = List.new()
script.append(Choice::Integer(37, 0, 100, 0))

let specification = PropertySpec<Int, IntStrategy>(
  strategy: IntStrategy(min: 0, max: 100, shrinkTowards: 0),
  property: |value| { value < 10 },
  identity: #integerBelow10
)
let settings = Settings(maxExamples: 1, maxShrinkSteps: 100)
let factory = Providers.ScriptedChoiceProviderFactory.new(script: script)
let result = Property.check(
  specification: specification,
  settings: settings,
  providers: factory.providerFactory
)

match result {
  PropertyResult::Falsified(value, failure, example, statistics) => {
    Testing.Test.expect(value == 10, "threshold failure must shrink to 10")
    Testing.Test.expect(
      failure.phase == Phase::Verify,
      "returned failure must come from a verification replay"
    )
    Testing.Test.expect(example.size == 1, "shrink must preserve the single-choice tape")
    match example.at(index: 0) {
      Choice::Integer(choice, _, _, _) =>
        Testing.Test.expect(choice == 10, "replayed tape must contain the minimized choice")
      _ => { throw Error.new("expected an integer choice in the minimized example") }
    }
    Testing.Test.expect(statistics.generatedExamples == 1, "search must count the initial example")
    Testing.Test.expect(statistics.shrinkEvaluations > 0, "search must evaluate shrink candidates")
    Testing.Test.expect(statistics.acceptedShrinks > 0, "search must accept a smaller failure")
    Testing.Test.expect(
      statistics.verificationReplays == 2,
      "search must verify the minimized failure twice"
    )
  }

  PropertyResult::Passed(_) =>
    { throw Error.new("threshold property unexpectedly passed") }
  PropertyResult::Flaky(_, _, _) =>
    { throw Error.new("deterministic threshold property became flaky") }
  PropertyResult::GaveUp(_, _) =>
    { throw Error.new("search gave up before finding the scripted failure") }
  PropertyResult::Errored(reason, _) =>
    { throw Error.new("search errored: " + reason) }

  _ => { throw Error.new("search returned an unrecognized result") }
}

Testing.Test.pass("engine.deterministic_property_search")
