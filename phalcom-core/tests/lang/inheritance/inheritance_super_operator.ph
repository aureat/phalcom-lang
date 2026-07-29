// area: inheritance
// spec: method-lookup.md §1.14; messages-and-selectors.md §3 (overloadable operators)
// status: PASS
// U-ERR-FIX SUPER-OP-SYNTAX: `super.<operator>(...)` parses and dispatches —
// operator methods are overridable and now super-callable too.

class Vec {
  @constructor
  new(x) { _x = x }
  x => _x
  +(other) { return Vec.new(_x + other.x) }
}
class Vec3 is Vec {
  +(other) {
    const r = super.+(other);
    return r.x;
  }
}
const a = Vec3.new(2)
const b = Vec3.new(3)
System.print(a.+(b))
