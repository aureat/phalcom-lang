// area: metaclass
// spec: object-model.md
// status: PASS
// `isA(_:)` walked across a 3-level chain: an instance of the most-derived
// class `C` is-a `A` (true, via `C -> B -> A`), but is NOT a `String` (false,
// unrelated hierarchy branch).

class A {
}
class B is A {
}
class C is B {
}
const c = C.new()
System.print(c.isA(A))
System.print(c.isA(String))
