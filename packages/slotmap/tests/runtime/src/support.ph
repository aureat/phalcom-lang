class Test {
  @class
  expect(_ condition: Bool, _ message: String) {
    if not condition {
      throw Error.new("test failure: \(message)")
    }
    ()
  }

  @class
  expectEqual<T>(_ actual: T, _ expected: T) {
    if actual != expected {
      throw Error.new("test failure: expected \(expected), got \(actual)")
    }
    ()
  }

  @class
  pass(_ name: String) {
    System.print("ok: \(name)")
    ()
  }
}

export Test
