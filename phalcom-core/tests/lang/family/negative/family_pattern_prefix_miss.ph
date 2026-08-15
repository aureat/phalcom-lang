// area: family/negative
// spec: docs/spec/callables/family.md §3
// status: NEGATIVE
// A prefix-constrained pattern does not silently widen to the same base name.

class Router {
  route() { 0 }
  route(_ value) { value }
}
const family = Router.new()::route(_, ...)
System.print(family())
