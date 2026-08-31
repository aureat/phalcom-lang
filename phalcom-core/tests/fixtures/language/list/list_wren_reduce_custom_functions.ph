// area: list
// spec: D.1 §14; ADR-0020
// status: PASS
// Ported from Wren `test/core/list/reduce.wren`: `fold(initial:using:)` folds an
// arbitrary 2-arity function across the elements, left-to-right — proved
// here with two different folds (a max-tracking fold built over the atomic
// `ifTrue(_, ifFalse:)`, and a `+`-based sum/concat fold reused for both
// `Number` and `String` elements) and varying seeds, mirroring Wren's
// `max`/`sum` `Fn` objects (stored here as blocks instead, `Fn`'s Phalcom
// analog).

const a = []
a.append(1)
a.append(4)
a.append(2)
a.append(1)
a.append(5)

const max = |x, y| { (x > y).ifTrue(|| { x }, ifFalse: || { y }) }
const sum = |x, y| { x + y }

System.print(a.fold(initial: 0, using: |acc, x| { max.call(acc, x) }))
System.print(a.fold(initial: 10, using: |acc, x| { max.call(acc, x) }))
System.print(a.fold(initial: 0, using: |acc, x| { sum.call(acc, x) }))
System.print(a.fold(initial: -1, using: |acc, x| { sum.call(acc, x) }))

const b = []
b.append("W")
b.append("o")
b.append("r")
b.append("l")
b.append("d")
System.print(b.fold(initial: "Hello ", using: |acc, x| { sum.call(acc, x) }))

// An empty seed list still applies `f` zero times, returning `init` as-is —
// `fold(initial:using:)` always requires an explicit seed, so
// there is no "empty sequence" raise (unlike Wren's no-init `reduce(_:)`).
System.print([].fold(initial: 1, using: |acc, x| { 42 }))
