// area: compile-errors
// spec: contract-annotations.md
// status: NEGATIVE
// contract: Requires cannot contain old() expressions

class Math {
  @requires(old(x) > 0)
  abs(x) {
    return x
  }
}
