// area: family/negative
// spec: docs/spec/callables/family.md §1–3
// status: NEGATIVE
// Exact Family construction does not probe the receiver. The missing getter
// reaches ordinary doesNotUnderstand only when the Family is called.

class Foo {}
const f = Foo.new()
const g = f::typo
System.print(g.get())
