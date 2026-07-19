// area: bindings
// spec: ADR-0064 §4; U-BINDINGS §4
// status: PASS
// Mutable fields take no keyword — bare `_x`.

class Counter {
  _n
  construct new() { _n = 0 }
  bump { _n = _n + 1 }
  get { return _n }
}

const c = Counter.new()
c.bump
c.bump
System.print(c.get)
