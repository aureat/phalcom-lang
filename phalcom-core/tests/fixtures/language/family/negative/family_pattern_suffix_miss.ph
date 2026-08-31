// area: family/negative
// spec: docs/spec/callables/family.md §3
// status: NEGATIVE
// A suffix-constrained pattern must keep the fixed trailing label.

class Router {
  route(foo) { foo }
  route(bar) { bar }
}
const family = (Router >> #route(..., foo)).bind(Router.new())
System.print(family(bar: 1))
