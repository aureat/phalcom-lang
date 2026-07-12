// area: functions
// spec: functions.md §3; ADR-0006 (Function/Method/Block tower);
//   ADR-0028 (Method reflection floor amendment)
// status: PASS
// U-CORE-3: `Object#methodFor(_)` reifies a `Method` by selector,
// `Method#invokeOn(_,_)` applies it to an explicit receiver + argument List,
// `Method#bind(_)` closes it over a receiver so `bound.call(...)` matches
// (R-INV-3.3), and `arity`/`name`/`selector` read its reflective surface.
// Written entirely in already-supported syntax: `Symbol.new("...")` interns
// the same symbol the compiler produced (so `methodFor` hits), and
// `List.new().add(_)` builds the argument list — no `#...`/`[...]` literals
// needed (those are U-LEX-gated, see the `pending/` fixtures in this
// directory).
class Greeter {
  greet(name) {
    return "Hello, " + name
  }
}
let g = Greeter.new()
let m = g.methodFor(Symbol.new("greet(_:)"))
let args = List.new().add("World")
System.print(m.invokeOn(g, args))
let bound = m.bind(g)
System.print(bound.call("World"))
System.print(m.arity)
System.print(m.name)
System.print(m.selector.toString)
System.print(g.methodFor(Symbol.new("nope")).isNone)
