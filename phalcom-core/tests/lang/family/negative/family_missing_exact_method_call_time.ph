// area: family/negative
// spec: docs/spec/callables/family.md §1–3
// status: NEGATIVE
// Exact Family construction does not probe the receiver. The missing method
// reaches ordinary doesNotUnderstand only when called with its exact shape.

class Foo {}
const f = Foo.new()
const g = f::#typo(_)
System.print(g(1))
