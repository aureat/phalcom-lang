// area: errors
// spec: contract-annotations.md
// status: PASS
// contract: a @requires-only method called with a valid argument runs normally

class Divider {
  @constructor
  new() { }

  @requires(divisor > 0)
  divide(_ value, _ divisor) {
    return value / divisor
  }
}

const d = Divider.new()
const result = d.divide(10, 2)
System.print(result)
System.print("ran without exception")
