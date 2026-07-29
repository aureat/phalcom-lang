import { Given, Gen, Property, PropertyRunner, PropertySuite, RecordingReporter, Settings } from "hypothesis"

class NoteProperties is PropertySuite {
  @Given(Gen.int(min: 0, max: 20))
  failsAtTen(value: Int) {
    Property.note("candidate=" + value.toString)
    Property.note(value)
    self.assertTrue(value < 10)
  }
}

const reporter = RecordingReporter.new()
const suite = PropertyRunner.run(
  const [NoteProperties],
  with: Settings.standard.seed(7),
  reporter: reporter
)
const failure = suite.runs.at(0).result.match(
  passed: { _ => throw Error.new("expected failure") },
  falsified: { value => value.failure },
  inconclusive: { value => throw value.reason },
  errored: { value => throw value.error }
)
Assert.equal(2, failure.notes.size)
Assert.equal(10, failure.notes.at(1))
