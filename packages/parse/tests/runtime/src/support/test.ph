from parse.core.result import RunResult

/// Minimal assertion support shared by runtime test fixtures.
class Test {
  @class
  expect(_ condition: Bool, _ message: String) -> Unit {
    if not condition {
      throw Error.new("FAILED: " + message)
    }
  }

  @class
  pass(_ name: String) -> Unit {
    System.print("PASS " + name)
  }

  /// Renders a parse outcome for string-equality assertions.
  @class
  show<T>(_ outcome: RunResult<T>) -> String {
    match outcome {
      RunResult::Ok(value) => {
        "ok:" + value.toString
      }
      RunResult::Err(error) => {
        "err:" + error.describe
      }
    }
  }

  /// Renders an outcome through a value renderer.
  @class
  showWith<T>(_ outcome: RunResult<T>, _ render: (T) -> String) -> String {
    match outcome {
      RunResult::Ok(value) => {
        "ok:" + render.call(value)
      }
      RunResult::Err(error) => {
        "err:" + error.describe
      }
    }
  }

  /// Renders `actual` and asserts it equals `expected`, throwing with
  /// both renderings on mismatch.
  @class
  expectEqual(_ actual: String, _ expected: String) -> Unit {
    if actual != expected {
      throw Error.new("FAILED: expected \"" + expected + "\" but got \"" + actual + "\"")
    }
  }
}

export Test
