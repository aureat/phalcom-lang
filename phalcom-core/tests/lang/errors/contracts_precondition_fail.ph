// area: errors
// spec: contract-annotations.md
// status: PASS
// contract: a @requires violation raises a catchable PreconditionError

class Divider {
  construct new() { }

  @requires(divisor > 0)
  divide(value, divisor) {
    return value / divisor
  }
}

let d = Divider.new()

try {
  d.divide(10, 0)
} catch e {
  System.print("PreconditionError: " + e.message)
}
