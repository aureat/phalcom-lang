// area: functions
// spec: functions.md; selectors.md
// status: PENDING

class Greeter {
  greet(name) {
    return "Hello, " + name;
  }
}
let g = Greeter.new()
let bound = g.methodFor(#greet(_)).bind(g)
System.print(bound("World"))
