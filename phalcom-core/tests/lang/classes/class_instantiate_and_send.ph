// area: classes
// spec: classes.md; messages-and-selectors.md
// status: PASS

class Counter {
  value => _v
  construct new(v) {
    _v = v
  }
  inc() {
    return Counter.new(_v + 1)
  }
}
System.print(Counter.new(0).inc().inc().inc().value)
