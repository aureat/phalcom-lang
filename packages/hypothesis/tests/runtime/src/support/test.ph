/// Minimal assertions shared by executable Hypothesis runtime tests.
class Test {
  @class
  expect(_ condition: Bool, _ message: String) -> Unit {
    if not condition {
      throw Error.new(message)
    }
  }

  @class
  pass(_ name: String) -> Unit {
    System.print("PASS " + name)
  }
}

export Test
