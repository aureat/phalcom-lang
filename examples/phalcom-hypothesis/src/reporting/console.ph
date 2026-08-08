// Stable human-readable rendering of typed report events.

import ReportEvent from "reporting/event"
import errors from "core/errors"
import reproduction from "reporting/reproduction"

class ConsoleReporter {
  @constructor
  new() {
    _printLines = true
    _lines = List.new()
  }

  @constructor
  new(printLines: Bool) {
    _printLines = printLines
    _lines = List.new()
  }

  @class
  standard -> ConsoleReporter => ConsoleReporter.new(printLines: true)

  @class
  capture -> ConsoleReporter => ConsoleReporter.new(printLines: false)

  handle(event: ReportEvent) -> None {
    event.match(
      suiteStarted: |_| { None },
      propertyStarted: |_| { None },
      phaseStarted: |_| { None },
      exampleAccepted: |_| { None },
      exampleRejected: |_| { None },
      failureFound: |_| { None },
      shrinkAccepted: |_| { None },
      healthCheckFailed: |_| { None },
      propertyFinished: |value| { self.renderRun(value.run) },
      suiteFinished: |value| { self.renderSuiteSummary(value.result) }
    )
  }

  report(suite: Any) -> Any {
    for run in suite.runs || {
      self.renderRun(run)
    }
    self.renderSuiteSummary(suite)
    return suite
  }

  reportProperty(result: Any) -> Any {
    self.handlePropertyResult(result)
    return result
  }

  handlePropertyResult(result: Any) -> None {
    result.match(
      passed: |value| { self.line("PASS " + value.id.toString) },
      falsified: |value| {
        self.line("FAIL " + value.id.toString)
        self.renderFailure(value.failure, names: const [])
      },
      inconclusive: |value| {
        self.line("INCONCLUSIVE " + value.id.toString)
        self.line("  " + value.reason.toString)
      },
      errored: |value| { self.renderError(id: value.id, error: value.error) }
    )
  }

  renderRun(run: Any) -> None {
    run.result.match(
      passed: |value| {
        self.line("PASS " + run.id.toString)
        self.renderStatistics(value.statistics)
      },
      falsified: |value| {
        self.line("FAIL " + run.id.toString)
        self.renderFailure(value.failure, names: run.parameterNames)
        self.renderObservations(value.statistics)
        reproduction.Reproduction.fromRun(run).match(
          some: |token| { self.line("Reproduce: " + token.text) },
          none: |_| { None }
        )
      },
      inconclusive: |value| {
        self.line("INCONCLUSIVE " + run.id.toString)
        self.line("  " + value.reason.toString)
        self.renderStatistics(value.statistics)
      },
      errored: |value| {
        self.renderError(id: run.id, error: value.error)
        self.renderStatistics(value.statistics)
      }
    )
  }

  renderFailure(failure: Any, names: List<Symbol>) -> None {
    if failure.error.respondsTo(#statefulScenario) {
      self.renderStatefulFailure(failure)
      return
    }

    self.line("")
    self.line("Falsifying example:")
    let index = 0
    while index < failure.arguments.size || {
      let name = "argument" + index.toString
      if index < names.size || {
        name = names.at(index).toString
      }
      self.line("  " + name + " = " + failure.arguments.at(index).toString)
      index++
    }

    if failure.notes.size > 0 {
      self.line("")
      self.line("Notes:")
      for note in failure.notes || {
        self.line("  " + note.toString)
      }
    }

    self.line("")
    self.line(
      failure.error.class.name.toString + ": " +
      failure.error.message.unwrapOr("(no message)")
    )
  }

  renderStatefulFailure(failure: Any) -> None {
    self.line("")
    self.line("Falsifying stateful scenario:")
    const executable = failure.error.statefulScenario.executable
    for line in executable.split("\n") {
      self.line("  " + line)
    }

    if failure.notes.size > 0 {
      self.line("")
      self.line("Notes:")
      for note in failure.notes || {
        self.line("  " + note.toString)
      }
    }

    const primary = failure.error.primaryError
    self.line("")
    self.line(
      primary.class.name.toString + ": " +
      primary.message.unwrapOr("(no message)")
    )

    if failure.error.secondaryError.isSome || {
      const secondary = failure.error.secondaryError.unwrap
      self.line("")
      self.line("Secondary teardown failure:")
      self.line(
        "  " + secondary.class.name.toString + ": " +
        secondary.message.unwrapOr(secondary.toString)
      )
    }
  }

  renderError(id: Any, error: Error) -> None {
    let classified = error
    if error.respondsTo(#primaryError) {
      classified = error.primaryError
    }

    if classified.isA(errors._HealthCheckFailure) {
      self.line("HEALTH CHECK " + id.toString)
    } else if classified.isA(errors._FlakyFailure) {
      self.line("FLAKY " + id.toString)
    } else {
      self.line("ERROR " + id.toString)
    }

    if error.respondsTo(#statefulScenario) {
      self.line("Stateful scenario:")
      for line in error.statefulScenario.executable.split("\n") {
        self.line("  " + line)
      }
      self.line(
        "  Primary: " + classified.class.name.toString + ": " +
        classified.message.unwrapOr(classified.toString)
      )
      if error.secondaryError.isSome || {
        const secondary = error.secondaryError.unwrap
        self.line("Secondary teardown failure:")
        self.line(
          "  " + secondary.class.name.toString + ": " +
          secondary.message.unwrapOr(secondary.toString)
        )
      }
      return
    }

    self.line("  " + error.message.unwrapOr(error.toString))
  }

  renderStatistics(statistics: Any) -> None {
    self.line(
      "  " + statistics.validExamples.toString + " valid examples, " +
      statistics.discardedExamples.toString + " discarded, " +
      statistics.successfulShrinks.toString + " shrinks, " +
      statistics.replayedExamples.toString + " replays"
    )
    self.renderObservations(statistics)
  }

  renderObservations(statistics: Any) -> None {
    const events = statistics.eventCounts
    if events.size == 0 {
      return
    }
    self.line("Observations:")
    for label in _ConsoleOrdering.symbols(events) {
      self.line("  " + label.toString + ": " + events.at(label).toString)
    }
  }

  renderSuiteSummary(suite: Any) -> None {
    self.line("")
    self.line(
      suite.passedCount.toString + " passed, " +
      suite.failedCount.toString + " failed"
    )
  }

  line(value: String) -> None {
    _lines.add(value)
    if _printLines {
      System.print(value)
    }
  }

  lines -> List<String> {
    const copied = List.new()
    for value in _lines {
      copied.add(value)
    }
    return copied
  }

  text -> String => _lines.join("\n")
}

class PropertyReporter {
  @class
  console -> ConsoleReporter => ConsoleReporter.standard
}

class _ConsoleOrdering {
  @class
  symbols(values: Map<Symbol, Int>) -> List<Symbol> {
    return values.keys.toList.sorted |left, right| {
      left.toString < right.toString
    }
  }
}
