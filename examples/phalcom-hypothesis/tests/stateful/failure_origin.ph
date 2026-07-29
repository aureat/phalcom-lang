// Stateful wrappers preserve the original assertion or invariant FailureOrigin.
import Assert from hypothesis
import Rule from hypothesis
import StateInvariant from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

class OriginMachine is StateMachine {
  @constructor
  new() { _value = 0 }

  @Rule
  decrement() { _value-- }

  @StateInvariant
  nonNegative() { Assert.true(_value >= 0, because: #statefulOrigin) }
}

const result = Stateful.check(OriginMachine, with: Settings.standard.statefulSteps(3))
Assert.equal(Some.new(#statefulOrigin), result.error.failureOrigin.label)
