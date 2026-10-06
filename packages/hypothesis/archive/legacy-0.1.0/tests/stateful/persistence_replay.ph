// Accepted minimal stateful examples replay deterministically through the existing database.
import Assert from hypothesis
import DirectoryDatabase from hypothesis
import Rule from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

class PersistentMachine is StateMachine {
  @Rule
  fail() { Assert.fail("persistent", because: #statefulPersistence) }
}

const database = DirectoryDatabase.new(root: ".tmp/stateful-persistence")
const settings = Settings.standard.withDatabase(database).statefulSteps(5).seed(20260723)
const first = Stateful.check(PersistentMachine, with: settings)
const second = Stateful.check(PersistentMachine, with: settings)
Assert.equal(first.tape.unwrap.signature, second.tape.unwrap.signature)
Assert.true(second.stats.replayedExamples > 0)
