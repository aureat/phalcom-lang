class Test {
  @class
  expect(_ condition: Bool, _ message: String) {
    if not condition { Error.new("FAILED: " + message).raise() }
  }

  @class
  pass(_ name: String) {
    System.print("PASS " + name)
  }
}

export Test
