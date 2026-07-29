// area: metaclass
// spec: method-lookup.md §2; object-model.md
// status: PASS
// `respondsTo(_:)` walked across a 3-level chain: `greet` is defined only on
// the top ancestor `A`, and the most-derived class `C` neither defines nor
// overrides it. `respondsTo` must succeed by walking the full chain (true),
// while an unrelated selector correctly reports false — without ever
// triggering `doesNotUnderstand(_:)`.

class A {
  greet => "hi"
}
class B is A {
}
class C is B {
}
const c = C.new()
System.print(c.respondsTo(Symbol.new("greet")))
System.print(c.respondsTo(Symbol.new("missing")))
