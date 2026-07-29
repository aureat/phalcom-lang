// Stable schema-versioned JSON-lines rendering of report events.

import ReportEvent from "reporting/event"

class JsonReporter {
  @constructor
  new() {
    _records = List.new()
    _jsonLines = List.new()
  }

  handle(event: ReportEvent) -> None {
    const record = _JsonEvent.record(event)
    _records.add(record)
    _jsonLines.add(_JsonEvent.line(event))
  }

  records -> List<Map<String, Any>> {
    const copied = List.new()
    for record in _records {
      copied.add(record)
    }
    return copied
  }

  jsonLines -> List<String> {
    const copied = List.new()
    for line in _jsonLines {
      copied.add(line)
    }
    return copied
  }

  text -> String => _jsonLines.join("\n")
}

class _JsonEvent {
  @class
  record(event: ReportEvent) -> Map<String, Any> {
    const record = Map.new()
    record.at("schemaVersion", put: 1)
    event.match(
      suiteStarted: { value =>
        record.at("type", put: "suite_started")
        record.at("total", put: value.total)
      },
      propertyStarted: { value =>
        record.at("type", put: "property_started")
        record.at("property", put: value.id.toString)
      },
      phaseStarted: { value =>
        record.at("type", put: "phase_started")
        record.at("property", put: value.id.toString)
        record.at("phase", put: value.phase.toString)
      },
      exampleAccepted: { value =>
        record.at("type", put: "example_accepted")
        record.at("property", put: value.id.toString)
        record.at("index", put: value.index)
      },
      exampleRejected: { value =>
        record.at("type", put: "example_rejected")
        record.at("property", put: value.id.toString)
        record.at("index", put: value.index)
        record.at("reason", put: value.reason.toString)
      },
      failureFound: { value =>
        record.at("type", put: "failure_found")
        record.at("property", put: value.id.toString)
        record.at("error", put: value.failure.error.class.name.toString)
        if value.failure.error.respondsTo(#statefulScenario) {
          record.at(
            "statefulScenario",
            put: value.failure.error.statefulScenario.executable
          )
          record.at(
            "primaryError",
            put: value.failure.error.primaryError.class.name.toString
          )
        }
      },
      shrinkAccepted: { value =>
        record.at("type", put: "shrink_accepted")
        record.at("property", put: value.id.toString)
        record.at("before", put: value.before.signature)
        record.at("after", put: value.after.signature)
      },
      healthCheckFailed: { value =>
        record.at("type", put: "health_check_failed")
        record.at("property", put: value.id.toString)
        record.at("message", put: value.error.message.unwrapOr(value.error.toString))
      },
      propertyFinished: { value =>
        record.at("type", put: "property_finished")
        record.at("property", put: value.run.id.toString)
        record.at("outcome", put: self.outcome(value.run.result))
      },
      suiteFinished: { value =>
        record.at("type", put: "suite_finished")
        record.at("passed", put: value.result.passedCount)
        record.at("failed", put: value.result.failedCount)
      }
    )
    return record
  }

  @class
  line(event: ReportEvent) -> String {
    const record = self.record(event)
    let out = "{\"schemaVersion\":1"
    for key in _JsonOrdering.keys(record) {
      if key != "schemaVersion" {
        out += ",\"" + _Json.escape(key) + "\":" +
          _Json.value(record.at(key))
      }
    }
    return out + "}"
  }

  @class
  outcome(result: Any) -> String {
    return result.match(
      passed: { _ => "passed" },
      falsified: { _ => "falsified" },
      inconclusive: { _ => "inconclusive" },
      errored: { value =>
        let error = value.error
        if error.respondsTo(#primaryError) {
          error = error.primaryError
        }
        if error.class.name.toString == "_HealthCheckFailure" {
          return "health_check"
        }
        if error.class.name.toString == "_FlakyFailure" {
          return "flaky"
        }
        return "errored"
      }
    )
  }
}

class _Json {
  @class
  value(value: Any) -> String {
    if value.isA(String) or value.isA(Symbol) {
      return "\"" + self.escape(value.toString) + "\""
    }
    if value == true {
      return "true"
    }
    if value == false {
      return "false"
    }
    if value == None {
      return "null"
    }
    return value.toString
  }

  @class
  escape(value: String) -> String {
    return value
      .replace("\\", with: "\\\\")
      .replace("\"", with: "\\\"")
      .replace("\n", with: "\\n")
      .replace("\r", with: "\\r")
      .replace("\t", with: "\\t")
  }
}

class _JsonOrdering {
  @class
  keys(values: Map<String, Any>) -> List<String> {
    return values.keys.toList.sorted { left, right => left < right }
  }
}
