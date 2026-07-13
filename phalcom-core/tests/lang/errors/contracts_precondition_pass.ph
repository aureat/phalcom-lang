// area: errors
// spec: contract-annotations.md
// status: PASS
// contract: a @requires-only method called with a valid argument runs normally

class Divider {
  construct new() { }

  @requires(divisor > 0)
  divide(value, divisor) {
    return value / divisor
  }
}

let d = Divider.new()
let result = d.divide(10, 2)
System.print(result)
System.print("ran without exception")
