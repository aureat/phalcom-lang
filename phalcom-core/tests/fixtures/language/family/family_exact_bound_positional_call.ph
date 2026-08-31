// area: family
// spec: docs/spec/callables/family.md §1 and §2
// status: PASS

class Adder {
  move(_ a, _ b) { return a + b }
}
const f = (Adder >> #move(_,_)).bind(Adder.new())
System.print(f(3, 4))
