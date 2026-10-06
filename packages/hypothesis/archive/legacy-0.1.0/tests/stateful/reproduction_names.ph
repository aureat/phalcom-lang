// Reproduction uses stable assignment names and distinguishes references from literals.
import Assert from hypothesis
import Bundle from hypothesis
import Gen from hypothesis
import Rule from hypothesis
import Settings from hypothesis
import StateMachine from hypothesis
import Stateful from hypothesis

const ReproductionKeys = Bundle<Bytes>.new(#key)

class ReproductionMachine is StateMachine {
  @Rule(Gen.bytes, ReproductionKeys.publish)
  createKey(value: Bytes) -> Bytes { return value }

  @Rule(ReproductionKeys)
  delete(key: Bytes) { Assert.fail("delete failed", because: #reproduction) }
}

const result = Stateful.check(ReproductionMachine, with: Settings.standard.statefulSteps(5))
const text = result.error.statefulScenario.executable
Assert.true(text.includes("key1 = state.createKey"))
Assert.true(text.includes("state.delete(key: key1)"))
