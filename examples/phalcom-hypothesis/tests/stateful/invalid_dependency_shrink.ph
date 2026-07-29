// Deleting a producer invalidates a dependent candidate; it is ignored as overrun.
import Assert from hypothesis
import Bundle from hypothesis
import Gen from hypothesis
import Rule from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

const DependencyValues = Bundle<Int>.new(#value)

class DependencyMachine is StateMachine {
  @Rule(Gen.int, DependencyValues.publish)
  produce(value: Int) -> Int { return value }

  @Rule(DependencyValues)
  consume(value: Int) { if value == 0 { Assert.fail("dependent failure") } }
}

const result = Stateful.check(DependencyMachine, with: Settings.standard.statefulSteps(10))
Assert.true(result.failed)
