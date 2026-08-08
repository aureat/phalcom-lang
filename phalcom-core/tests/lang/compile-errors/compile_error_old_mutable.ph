// area: compile-errors
// spec: contract-annotations.md
// status: NEGATIVE
// contract: old() operand must not be the whole receiver (aliases the mutable object)

class Math {
  @ensures(old(_ self) == self)
  abs(_ x) {
    return x
  }
}
