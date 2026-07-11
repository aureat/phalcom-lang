// area: dispatch
// spec: messages-and-selectors.md; lexical-structure.md
// status: PENDING

class Adder {
  add(a, b, c) {
    return a + b + c;
  }
}
let args = [1, 2, 3]
System.print(Adder.new().add(*args))
