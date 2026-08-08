// area: inheritance
// spec: method-lookup.md §2; object-model.md
// status: PASS
// Selector identity across a class boundary: the parent defines a 1-arg
// `foo(x)` (selector `foo(_:)`), the child defines a 0-arg `foo` getter
// (selector `foo`) with the SAME bare name but a DIFFERENT selector. Both
// coexist on the subclass instance and are dispatched independently — the
// child's zero-arg definition does not shadow or collide with the inherited
// one-arg definition.

class Base {
  foo(_ x) { return "Base.foo(" + x.toString + ")" }
}
class Sub is Base { foo => "Sub.foo"
}
const s = Sub.new()
System.print(s.foo)
System.print(s.foo(5))
