// Invariants run once after all initialization and after every normal rule.
import Assert from hypothesis
import Initialize from hypothesis
import Rule from hypothesis
import StateInvariant from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

class InvariantMachine is StateMachine {
  @constructor
  new() {
    _initialized = 0
    _rules = 0
    _checks = 0
  }

  @Initialize
  aInitialize() { _initialized++ }

  @Initialize
  bInitialize() { _initialized++ }

  @StateInvariant
  initializedBeforeChecking() {
    Assert.equal(2, _initialized)
    Assert.equal(_rules + 1, ++_checks)
  }

  @Rule
  step() { _rules++ }
}

Stateful.check(InvariantMachine, with: Settings.standard.statefulSteps(3))
