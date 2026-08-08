// area: dispatch
// spec: messages-and-selectors.md §5; ADR-0012
// status: PASS
// `perform(selector, args)` dispatches through the same lookup path as a
// static send (reflective parity). `#"+(_)"` symbol-literal syntax is U-LEX;
// the selector is built with `Symbol.new("...")`. `3.perform(+, [4]) == 3 + 4`.

const a = []
a.append(4)
System.print(3.perform(Symbol.new("+(_)"), a))
System.print(3.perform(Symbol.new("negated()"), []))
System.print(3.perform(Symbol.new("+(_)"), a) == 7)
