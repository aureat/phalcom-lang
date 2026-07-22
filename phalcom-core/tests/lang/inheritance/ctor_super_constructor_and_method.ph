// area: inheritance
// spec: ADR-0063 §5 (constructor desugar and super-initializer rewrite)
// status: PASS
// A constructor's matching `super.new` targets the hidden initializer, while
// an unrelated `super.mark` remains an ordinary instance-side super-send.
class Base {
  @constructor
  new(value) { _base = value }
  mark(value) { _marked = value + 10 }
  base => _base
  marked => _marked
}
class Derived extends Base {
  @constructor
  new(value) {
    super.new(value)
    super.mark(value)
  }
}
let d = Derived.new(5)
System.print(d.base)
System.print(d.marked)
