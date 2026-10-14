/// Minimal assertions shared by Nano ID tests.
class Test {
  @class
  expect(_ condition: Bool, _ message: String) -> Unit {
    if not condition {
      throw Error.new(message)
    }
  }

  @class
  expectEqual(_ actual: Dynamic, _ expected: Dynamic, _ label: String) -> Unit {
    if actual != expected {
      throw Error.new("\(label): expected \(expected), got \(actual)")
    }
  }

  @class
  pass(_ name: String) -> Unit {
    System.print("PASS " + name)
  }
}

export Test
