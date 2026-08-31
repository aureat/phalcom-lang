// area: rest
// spec: F.3-rest-capture-and-rest-pattern-dispatch-amended.md inheritance fallback
// status: PASS

class ParentRest {
  route(*items) { return 100 + items.size }
}

class ChildRest is ParentRest {
  route(_ fixed, **extra) { return 200 + extra.size }
}

const child = ChildRest.new()
System.print(child.route(1, 2))

const positional = (1, 2)
System.print(child.route(*positional))

System.print(child.route(1, debug: 2))
