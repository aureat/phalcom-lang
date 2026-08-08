import {
  Given,
  Gen,
  PropertyRunner,
  PropertySuite,
  RecordingReporter,
  ReportEvent,
  Settings
} from "hypothesis"

class PassingProperties is PropertySuite {
  @Given(Gen.int(min: 0, max: 1))
  accepts(value: Int) {
    self.assertTrue(value >= 0)
  }
}

const reporter = RecordingReporter.new()
const suite = PropertyRunner.run(
  const [PassingProperties],
  with: Settings.standard.maxExamples(2),
  reporter: reporter
)

Assert.true(suite.passed)
Assert.true(reporter.events.at(0).isSuiteStarted)
Assert.true(reporter.events.at(1).isPropertyStarted)
Assert.true(reporter.events.any |event| { event.isPhaseStarted })
Assert.true(reporter.events.any |event| { event.isExampleAccepted })
Assert.true(reporter.events.at(reporter.events.size - 2).isPropertyFinished)
Assert.true(reporter.events.at(reporter.events.size - 1).isSuiteFinished)
