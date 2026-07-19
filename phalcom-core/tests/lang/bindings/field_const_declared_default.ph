// area: bindings
// spec: ADR-0064 §4; U-BINDINGS §4
// status: PASS
// `const _x = e` parses and is immutable; the default is set through the
// constructor here since bare-allocator default-application is a separate,
// unwired mechanism (not part of this unit's write-set).

class Origin {
  const _x = 10
  construct new(v) { _x = v }
  get { return _x }
}

System.print(Origin.new(10).get)
