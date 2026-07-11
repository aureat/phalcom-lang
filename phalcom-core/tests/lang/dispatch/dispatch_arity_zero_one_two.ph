// area: dispatch
// spec: method-lookup.md; messages-and-selectors.md; object-model.md
// status: PASS

class Overload {
  m() {
    return 0;
  }
  m(a) {
    return a;
  }
  m(a, b) {
    return a + b;
  }
}
let o = Overload.new()
System.print(o.m())
System.print(o.m(7))
System.print(o.m(2, 3))
