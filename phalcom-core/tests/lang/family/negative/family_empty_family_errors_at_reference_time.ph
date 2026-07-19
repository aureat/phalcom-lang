// area: family/negative
// spec: selectors.md §3 error table ("Empty family — obj::typo where no
//   method named typo exists — Error at reference time, naming the class");
//   ADR-0047
// status: NEGATIVE
// `obj::typo` on a class with no `typo` method and no `doesNotUnderstand`
// override is an error at `::` reference time (not deferred to a later
// call), naming the class.

class Foo {}
const f = Foo.new()
const g = f::typo
System.print(g)
