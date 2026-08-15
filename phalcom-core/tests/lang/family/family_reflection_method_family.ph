// area: family
// spec: docs/spec/callables/reflection.md §2
// status: PASS
// Pattern extraction returns MethodFamily. Binding the snapshot routes exact
// captured implementations without receiver-side reselection.

class Router {
  route() { "zero" }
  route(_ value) { "one" }
  route(to) { "to" }
}

const methods = Router >> #route(...)
System.print(methods.size)
const bound = methods.bind(Router.new())
System.print(bound())
System.print(bound(7))
System.print(bound(to: 8))
