// area: compile-errors
// spec: contract-annotations.md
// status: NEGATIVE
// contract: Impure expressions are rejected in contracts

class Math {
  @requires(x = 10)
  abs(_ x) {
    return x
  }
}
