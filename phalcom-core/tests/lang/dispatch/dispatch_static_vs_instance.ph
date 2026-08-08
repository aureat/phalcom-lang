// area: dispatch
// spec: method-lookup.md; object-model.md
// status: PASS

class Calc {
  @class
  double(_ n) {
    return n * 2;
  }
  triple(_ n) {
    return n * 3;
  }
}
System.print(Calc.double(5))
System.print(Calc.new().triple(5))
