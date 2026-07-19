// area: family/negative
// spec: selectors.md §3 error table ("Empty family — obj::typo where no
//   method named typo exists — Error at reference time, naming the class");
//   ADR-0047, U16-Pinned
// status: NEGATIVE
// `obj::#typo(_)` on a class with no `typo(_)` method and no
// `doesNotUnderstand` override is an error at `::` reference time, checked
// against the exact pinned selector (not just the base name).

class Foo {}
const f = Foo.new()
const g = f::#typo(_)
System.print(g)
