// A consumed reference is removed after one selection and cannot be reused.
import Assert from hypothesis
import Bundle from hypothesis
import Gen from hypothesis
import Rule from hypothesis
import When from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

const Tickets = Bundle<Int>.new(#ticket)

class ConsumingMachine is StateMachine {
  @constructor
  new() { _consumed = Set.new() }

  @Rule(Gen.int(min: 1, max: 10), Tickets.publish)
  issue(value: Int) -> Int { return value }

  hasTicket -> Bool => _consumed.size == 0

  @When(#hasTicket)
  @Rule(Tickets.consume)
  spend(value: Int) {
    Assert.false(_consumed.includes(value))
    _consumed.add(value)
  }
}

Stateful.check(ConsumingMachine, with: Settings.standard.statefulSteps(20))
