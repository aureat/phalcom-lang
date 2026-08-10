// area: inheritance
// spec: method-lookup.md §1.14; object-model.md
// status: PASS
// Classic OO "virtual call from constructor" hazard: `A`'s constructor calls
// `self.label` while `self` is already bound to the most-derived runtime
// instance. When constructing a `B`, that self-send resolves DYNAMICALLY to
// `B#label`, not statically to `A#label`, even though the call originates
// inside `A`'s own constructor body.

class A {
  @constructor
  new() { _tag = self.label }
  label { "A" }
  report { _tag }
}
class B is A {
  @constructor
  new() { super.new() }
  label { "B" }
}
const b = B.new()
System.print(b.report)
