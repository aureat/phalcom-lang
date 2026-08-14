// area: family
// spec: docs/spec/callables/family.md §1 and §2
// status: PASS
// An exact positional Family retains its selector shape. The receiver
// expression is itself a MethodCall, proving `::` composes as a postfix.

class Adder {
  move(_ a, _ b) { return a + b }
}
const f = Adder.new()::move(_,_)
System.print(f(3, 4))
