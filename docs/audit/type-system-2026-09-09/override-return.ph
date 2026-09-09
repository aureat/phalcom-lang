class Base { value() -> Int { return 1 } }
class Child is Base {
  @constructor
  new() {}
  value() -> String { return "wrong" }
}
class Probe {
  @class
  consume(_ x: Base) -> Int { return x.value() }
}
System.print(Probe.consume(Child.new()))
