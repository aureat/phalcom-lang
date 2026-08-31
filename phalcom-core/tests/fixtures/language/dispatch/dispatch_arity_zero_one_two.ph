// area: dispatch
// spec: method-lookup.md; messages-and-selectors.md; object-model.md
// status: PASS

class Overload {
  m() {
    return 0;
  }
  m(_ a) {
    return a;
  }
  m(_ a, _ b) {
    return a + b;
  }
}
const o = Overload.new()
System.print(o.m())
System.print(o.m(7))
System.print(o.m(2, 3))
