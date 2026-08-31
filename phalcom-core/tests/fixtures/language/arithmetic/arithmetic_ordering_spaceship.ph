// area: arithmetic
// spec: Ordering and <=>

class NumWrapper {
  @get _n
  @constructor new(_ n) { _n = n }

  compare(_ other) {
    if (other is NumWrapper) {
      return _n <=> other.n
    }
    if (other is Int) {
      return _n <=> other
    }
    unsupported
  }

  toString { "NumWrapper(\(_n))" }
}

let w1 = NumWrapper.new(10)
let w2 = NumWrapper.new(20)
let w3 = NumWrapper.new(10)

System.print(w1 <=> w2)
System.print(w2 <=> w1)
System.print(w1 <=> w3)

// Reflected comparison: 15 <=> w1 (where w1 is RHS, handles comparison and reverses result)
System.print(15 <=> w1)
System.print(5 <=> w1)

// Derived < <= > >= on Object
System.print(w1 < w2)
System.print(w1 <= w3)
System.print(w2 > w1)
System.print(w1 >= w2)
