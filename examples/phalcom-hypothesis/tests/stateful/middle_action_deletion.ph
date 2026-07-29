// Structural shrinking deletes an irrelevant middle action and retains a later action.
import Assert from hypothesis
import Rule from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

class MiddleDeletionMachine is StateMachine {
  @constructor
  new() {
    _first = false
    _noise = false
  }

  @Rule
  first() { _first = true }

  @Rule
  irrelevant() { _noise = true }

  @Rule
  last() { if _first { Assert.fail("first and last fail", because: #middleDeletion) } }
}

const result = Stateful.check(
  MiddleDeletionMachine,
  with: Settings.standard.statefulSteps(8).seed(20260723)
)
Assert.false(result.error.statefulScenario.selectors.includes(#irrelevant))
Assert.true(result.error.statefulScenario.selectors.includes(#last))
