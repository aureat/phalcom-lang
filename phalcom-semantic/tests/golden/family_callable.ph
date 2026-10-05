// LAW CHAIN
// 1. Family capture records receiver and selector without activating a Method.
// 2. Exact capture routes Int; pattern capture routes labeled and nullary calls.
// 3. Router publishes all call results as String.

class Formatter {
  render(_ value: Int) -> String { "int" }
  render(value: String) -> String { value }
  render() -> String { "empty" }
}

class Router {
  @class
  use(_ family: (Int) -> String, _ value: Int) -> String {
    family(value)
  }
}

class Service {
  @class
  run(_ formatter: Formatter) {
    let mut exact = &formatter.render(_)
    let mut pattern = &formatter.render

    let mut a = Router.use(exact, 42)
    let mut b = pattern(value: "x")
    let mut c = pattern()

    (a, b, c)
  }
}

class Probe {
  @class
  run(_ formatter: Formatter) {
    Service.run(formatter)
  }
}
