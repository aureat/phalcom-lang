// area: list
// spec: catalog-delta.md §2.4; ADR-0020; U-STD §2.6; DEFERRED.md #25
// status: PASS
// Ported from Wren `test/core/list/reduce.wren`: `reduce(init, f)` folds an
// arbitrary 2-arity function across the elements, left-to-right — proved
// here with two different folds (a max-tracking fold built over the atomic
// `ifTrue(_, ifFalse:)`, and a `+`-based sum/concat fold reused for both
// `Number` and `String` elements) and varying seeds, mirroring Wren's
// `max`/`sum` `Fn` objects (stored here as blocks instead, `Fn`'s Phalcom
// analog).

const a = List.new()
a.add(1)
a.add(4)
a.add(2)
a.add(1)
a.add(5)

const max = { x, y => (x > y).ifTrue({ x }, ifFalse: { y }) }
const sum = { x, y => x + y }

System.print(a.reduce(0) { acc, x => max.call(acc, x) })
System.print(a.reduce(10) { acc, x => max.call(acc, x) })
System.print(a.reduce(0) { acc, x => sum.call(acc, x) })
System.print(a.reduce(-1) { acc, x => sum.call(acc, x) })

const b = List.new()
b.add("W")
b.add("o")
b.add("r")
b.add("l")
b.add("d")
System.print(b.reduce("Hello ") { acc, x => sum.call(acc, x) })

// An empty seed list still applies `f` zero times, returning `init` as-is —
// the Phalcom `reduce(init, f)` shape always requires an explicit seed, so
// there is no "empty sequence" raise (unlike Wren's no-init `reduce(_:)`).
System.print(List.new().reduce(1) { acc, x => 42 })
