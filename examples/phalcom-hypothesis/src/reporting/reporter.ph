// Stable reporter protocol and output-independent implementations.
//
// Delivery is synchronous and ordered. _CheckedReporter converts an exception
// from an extension into ReporterFailure so it is never classified as a user
// property counterexample.

import ReportEvent from "reporting/event"
import ReporterFailure from "core/errors"

protocol Reporter {
  handle(event: ReportEvent) -> None
}

class NullReporter {
  handle(event: ReportEvent) -> None {
    None
  }
}

class RecordingReporter {
  @constructor
  new() {
    _events = List.new()
  }

  handle(event: ReportEvent) -> None {
    _events.add(event)
  }

  events -> List<ReportEvent> {
    const copied = List.new()
    for event in _events {
      copied.add(event)
    }
    return copied
  }
}

class CompositeReporter {
  @constructor
  new(reporters: List<Reporter>) {
    _reporters = List.new()
    for reporter in reporters {
      _reporters.add(reporter)
    }
  }

  handle(event: ReportEvent) -> None {
    for reporter in _reporters {
      reporter.handle(event)
    }
  }
}

class _CheckedReporter {
  @constructor
  new(reporter: Reporter) {
    _reporter = reporter
    _failure = None
  }

  handle(event: ReportEvent) -> None {
    if _failure.isSome {
      return None
    }
    const delivered = {
      _reporter.handle(event)
    }.attempt()
    if delivered.isErr {
      const error = delivered.unwrapErr
      let failure = ReporterFailure.from(error)
      if error.isA(ReporterFailure) {
        failure = error
      }
      _failure = Some.new(failure)
      throw failure
    }
    return None
  }
}
