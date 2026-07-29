import Given from hypothesis
import WithSettings from hypothesis
import Settings from hypothesis
import MemoryDatabase from hypothesis
import PropertySuite from hypothesis
import PropertyRunner from hypothesis

const database = MemoryDatabase.new(maxEntries: 8)

class StaleReplayProperties is PropertySuite {
  @WithSettings(Settings.standard.database(database))
  @Given(Gen.int(min: 0, max: 10))
  alwaysPasses(value: Int) {
    Assert.true(value >= 0)
  }
}

PropertyRunner.run(const [StaleReplayProperties], with: Settings.standard.database(database))
PropertyRunner.run(const [StaleReplayProperties], with: Settings.standard.database(database))
Assert.equal(0, database.entryCount)
