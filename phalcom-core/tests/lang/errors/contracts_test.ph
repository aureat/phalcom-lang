// area: errors
// spec: contract-annotations.md
// status: PASS
// contract: contracts validation under @requires, @ensures, and @invariant

class Counter {
  @invariant(self.val >= 0)
  
  construct new(init) {
    _val = init
  }
  
  val => _val
  
  @requires(amount > 0)
  @ensures(self.val == old(self.val) + amount)
  increment(amount) {
    _val = _val + amount
  }
  
  @requires(amount > 0)
  @ensures(self.val == old(self.val) - amount)
  decrement(amount) {
    _val = _val - amount
  }
}

const c = Counter.new(10)
System.print(c.val)

c.increment(5)
System.print(c.val)

c.decrement(3)
System.print(c.val)

// Test precondition violation
try {
  c.increment(-5)
} catch e {
  System.print("PreconditionError: " + e.message)
}

// Test invariant violation
try {
  c.decrement(20)
} catch e {
  System.print("InvariantError: " + e.message)
}
