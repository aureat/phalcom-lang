// area: messages/selectors
// spec: messages-and-selectors.md; object-model.md
// status: PASS

class Adder {
  @class
  add(_ a, _ b) {
    return a + b;
  }
}
System.print(Adder.add(2, 3))
