// Initializers execute in stable selector order before any normal rule.
import Assert from hypothesis
import Initialize from hypothesis
import Rule from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

class InitializedMachine is StateMachine {
  @constructor
  new() { _order = List.new() }

  @Initialize
  aPrepare() { _order.add(#a) }

  @Initialize
  bPrepare() { _order.add(#b) }

  @Rule
  verifyInitialized() { Assert.equal(const [#a, #b], _order) }
}

Stateful.check(InitializedMachine, with: Settings.standard.statefulSteps(1))
