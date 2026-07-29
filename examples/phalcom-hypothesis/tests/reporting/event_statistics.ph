import { Given, Gen, Property, PropertyRunner, PropertySuite, Settings } from "hypothesis"

class ObservationProperties is PropertySuite {
  @Given(Gen.int(min: 0, max: 3))
  records(value: Int) {
    Property.event(#generated)
    Property.classify(value == 0, as: #zero)
    Property.classify(value > 0, as: #positive)
    self.assertTrue(value >= 0)
  }
}

const suite = PropertyRunner.run(
  const [ObservationProperties],
  with: Settings.standard.maxExamples(20).seed(3)
)
const statistics = suite.runs.at(0).result.stats
Assert.equal(20, statistics.events.at(#generated))
Assert.equal(20, statistics.events.at(#zero) + statistics.events.at(#positive))
