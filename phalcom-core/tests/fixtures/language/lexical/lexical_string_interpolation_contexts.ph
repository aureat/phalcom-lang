// area: lexical/literals
// spec: lexical-structure.md §5; ADR-0022
// status: PASS
// Interpolation must work as a complete expression in bindings, return values,
// argument positions, field reads, nested sends, arithmetic, and at string
// boundaries. `\\(` remains literal text.

class Counter {
  mut _value
  @constructor
  new(_ value) { _value = value }

  render(_ prefix) {
    return "\(prefix): \(_value + 1)"
  }
}

let counter = Counter.new(4)
let summary = counter.render("count")
System.print(summary)
System.print("argument: \(counter.render("next"))")
System.print("\(1)|left\(2)|right\(3)")
System.print("literal \\( stays; nested \(String.new("ok"))")
