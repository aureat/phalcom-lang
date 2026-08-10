// area: inheritance
// spec: object-model.md §5.1; ADR-0011; ADR-0040
// status: PASS
// U-INH follow-on: an inherited `construct` resolves at a subclass call site
// with NO redeclaration. `Point3` declares no constructor; `Point3.new(x)`
// redirects to `Point2`'s inherited `@constructor new(v)` (via the compile-time
// superclass-chain alias walk) and runs it on the `Point3` instance, filling
// the inherited slot. Both matching-arity `new` and named ctors inherit.

class Point2 {
  @constructor
  new(_ v) { _v = v }
  @constructor
  named(_ w) { _v = w }
  v { _v }
}
class Point3 is Point2 {
}
const a = Point3.new(11)
System.print(a.v)
const b = Point3.named(22)
System.print(b.v)
