// area: bindings
// spec: ADR-0064 §5; U-BINDINGS §5
// status: PASS
// A `const` field with no default is assignable inside its constructor.

class Id {
  const _id

  construct new(v) {
    _id = v
  }

  get { return _id }
}

System.print(Id.new(7).get)
