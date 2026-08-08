// area: dispatch
// spec: messages-and-selectors.md; lexical-structure.md
// status: PENDING

class Adder {
  add(_ a, _ b, _ c) {
    return a + b + c;
  }
}
const args = [1, 2, 3]
System.print(Adder.new().add(*args))
