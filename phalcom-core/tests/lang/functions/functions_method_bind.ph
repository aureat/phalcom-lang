// area: functions
// spec: functions.md; selectors.md
// status: PASS
// Graduated by U-LEX-HASH: `#greet(_)` now lexes as a selector symbol,
// unblocking `Object#methodFor(_)` + `Method#bind(_)` over a real `#`-literal
// selector (previously only reachable via `Symbol.new("greet(_)")`, see
// `functions_method_reflection.ph`).

class Greeter {
  greet(_ name) {
    return "Hello, " + name;
  }
}
const g = Greeter.new()
const bound = g.methodFor(#greet(_)).bind(g)
System.print(bound("World"))
