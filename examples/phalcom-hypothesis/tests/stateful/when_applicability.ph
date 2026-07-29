// @When removes unavailable rules before selection and does not reject examples.
import Assert from hypothesis
import Rule from hypothesis
import When from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

class GuardedCounter is StateMachine {
  @constructor
  new() { _count = 0 }

  canDecrement -> Bool => _count > 0

  @Rule
  increment() { _count++ }

  @When(#canDecrement)
  @Rule
  decrement() { _count-- }
}

const result = Stateful.check(
  GuardedCounter,
  with: Settings.standard.statefulSteps(30).examples(20).seed(20260723)
)
Assert.equal(0, result.stats.discardedExamples)
