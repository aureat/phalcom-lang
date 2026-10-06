// A broken counter must shrink to exactly one decrement action.
import Assert from hypothesis
import Rule from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

class BrokenCounter is StateMachine {
  @constructor
  new() { _count = 0 }

  @Rule
  decrement() {
    _count--
    Assert.true(_count >= 0, because: #counterNeverNegative)
  }
}

const result = Stateful.check(
  BrokenCounter,
  with: Settings.standard.statefulSteps(20).seed(20260723)
)
Assert.equal(1, result.error.statefulScenario.normalActionCount)
Assert.equal(#decrement, result.error.statefulScenario.actions.at(0).selector)
