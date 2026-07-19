// area: bindings
// spec: open-questions.md Q7; ADR-0046 §1
// status: PASS
// U14: the RHS is evaluated exactly ONCE, into a scratch temp, before any
// sub-pattern reads it — `const (a, b) = counterTuple()` must invoke the
// producer exactly once, not once per binding.
let count = 0

class Producer {
  static counterTuple() {
    count = count + 1
    return (1, 2)
  }
}

const (a, b) = Producer.counterTuple()
System.print(a)
System.print(b)
System.print(count)
