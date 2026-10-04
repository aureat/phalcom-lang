// area: family
// spec: docs/spec/callables/family.md §§1–3
// status: PASS

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

let r = Router.new()
let any = (Router >> #route...).bind(r)
let methods = (Router >> #route(...)).bind(r)
let prefix = (Router >> #route(_, ...)).bind(r)
let suffix = (Router >> #route(..., foo)).bind(r)
let sandwich = (Router >> #route(_, ..., foo)).bind(r)

System.print(any())
System.print(any(1))
System.print(methods(foo: 1))
System.print(prefix(1))
System.print(prefix(1, 2))
System.print(prefix(1, foo: 2))
System.print(suffix(foo: 1))
System.print(suffix(1, foo: 2))
System.print(sandwich(1, foo: 2))
System.print(sandwich(1, mid: 2, foo: 3))
System.print(sandwich(1, 2, mid: 3, foo: 4))
