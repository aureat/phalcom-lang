// area: reflection
// spec: messages-and-selectors.md §5; ADR-0012
// status: PASS
// `perform(selector, args)` reflectively dispatches a USER-DEFINED
// two-argument method (distinct from dispatch/'s built-in `+`/`negated`
// parity fixture): the selector's arity-encoded name (`add(_,_)`) picks the
// right overload, and the returned value composes normally with the rest of
// the program.

class Adder {
  add(a, b) => a + b
  zero() => 0
}
const obj = Adder.new()
const args = List.new()
args.add(3)
args.add(4)
System.print(obj.perform(Symbol.new("add(_,_)"), args))
System.print(obj.perform(Symbol.new("zero()"), List.new()))
System.print(obj.perform(Symbol.new("add(_,_)"), args) == 7)
