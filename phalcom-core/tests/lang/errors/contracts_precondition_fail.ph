// area: errors
// spec: contract-annotations.md
// status: PASS
// contract: a @requires violation raises a catchable PreconditionError

class Divider {
  @constructor new() { }

  @requires(divisor > 0)
  divide(value, divisor) {
    return value / divisor
  }
}

const d = Divider.new()

try {
  d.divide(10, 0)
} catch e {
  System.print("PreconditionError: " + e.message)
}
