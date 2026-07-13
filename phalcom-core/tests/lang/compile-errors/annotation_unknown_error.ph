// area: compile-errors
// spec: contract-annotations.md
// status: NEGATIVE
// contract: an attribute name with no registered expander is a compile-time error

class Math {
  @bogus(1)
  abs(x) {
    return x
  }
}
