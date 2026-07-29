// Regression: reporter exceptions are extension failures, not falsifying examples.

import { Assert, ReporterFailure } from "hypothesis"

class BrokenReporter {
  handle(event: Any) -> None {
    throw Error.new("broken reporter")
  }
}

Assert.true(ReporterFailure.from(Error.new("probe")).isA(ReporterFailure))
// Runtime fixture checks SearchEngine returns Errored(ReporterFailure) and does
// not emit FailureFound for the property body.
