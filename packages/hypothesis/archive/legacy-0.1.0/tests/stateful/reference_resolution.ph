// Later actions resolve a ResultReference produced by an earlier action.
import Assert from hypothesis
import Bundle from hypothesis
import Gen from hypothesis
import Rule from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

const References = Bundle<Int>.new(#value)

class ReferenceMachine is StateMachine {
  @constructor
  new() { _seen = List.new() }

  @Rule(Gen.int(min: 1, max: 10), References.publish)
  create(value: Int) -> Int {
    _seen.add(value)
    return value
  }

  @Rule(References)
  use(value: Int) { Assert.true(_seen.includes(value)) }
}

const result = Stateful.check(ReferenceMachine, with: Settings.standard.statefulSteps(20))
Assert.true(result.respondsTo(#stats))
