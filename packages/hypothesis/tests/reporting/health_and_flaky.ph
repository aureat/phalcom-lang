import {
  ConsoleReporter,
  PropertyResult,
  FlakyFailure,
  HealthCheckFailure,
  PropertyId,
  Statistics
} from "hypothesis"

const id = PropertyId.new(module: #tests, suite: #Health, selector: #check)
const health = PropertyResult.errored(
  id: id,
  error: HealthCheckFailure.new("too many filtered examples"),
  statistics: Statistics.empty
)
const flaky = PropertyResult.errored(
  id: id,
  error: FlakyFailure.new("counterexample did not reproduce"),
  statistics: Statistics.empty
)
const reporter = ConsoleReporter.capture()
reporter.handlePropertyResult(health)
reporter.handlePropertyResult(flaky)
Assert.true(reporter.text.includes("HEALTH CHECK"))
Assert.true(reporter.text.includes("FLAKY"))
Assert.false(reporter.text.includes("Falsifying example"))
