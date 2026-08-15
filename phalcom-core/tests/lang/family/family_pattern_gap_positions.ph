// area: family
// spec: docs/spec/callables/family.md §§1–3
// status: PASS
// Exercises every named selector-pattern gap position through live Family
// dispatch. The gap is zero-or-more slots; fixed prefix/suffix slots remain
// structural constraints rather than call-time hints.

class Router {
  route() { "zero" }
  route(_ a) { "one" }
  route(_ a, _ b) { "two" }
  route(foo) { "foo" }
  route(_ a, foo) { "prefix+foo" }
  route(_ a, mid, foo) { "prefix+mid+foo" }
  route(_ a, _ b, mid, foo) { "prefix+two+mid+foo" }
  route(bar) { "bar" }
}

const r = Router.new()
const any = r::route...
const methods = r::route(...)
const prefix = r::route(_, ...)
const suffix = r::route(..., foo)
const sandwich = r::route(_, ..., foo)

System.print(any())
System.print(any(1))
System.print(methods(foo: 1))
System.print(prefix(1))
System.print(prefix(1, 2))
System.print(suffix(foo: 1))
System.print(suffix(1, foo: 2))
System.print(sandwich(1, foo: 2))
System.print(sandwich(1, mid: 2, foo: 3))
System.print(sandwich(1, 2, mid: 3, foo: 4))
