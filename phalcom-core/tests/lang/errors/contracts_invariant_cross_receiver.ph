// area: errors
// spec: contract-annotations.md, ADR-0052 Bug-1 regression
// status: PASS
// contract: A's public method calling B's public method must not suppress B's own @invariant check

class B {
  @invariant(self.val >= 0)

  @constructor
  new(_ init) {
    _val = init
  }

  val { _val }

  drain(_ amount) {
    _val = _val - amount
  }
}

class A {
  @constructor
  new(_ b) {
    _b = b
  }

  poke(_ amount) {
    _b.drain(amount)
  }
}

const b = B.new(10)
const a = A.new(b)

a.poke(3)
System.print(b.val)

try {
  a.poke(20)
} catch e {
  System.print("InvariantError: " + e.message)
}
