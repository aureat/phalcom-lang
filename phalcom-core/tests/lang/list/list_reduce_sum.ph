// area: list
// spec: catalog-delta.md §2.4; ADR-0020; U-STD §2.6; DEFERRED.md #25
// status: PASS
// U-STD: `reduce(init, f)` folds `f(acc, x)` left-to-right from `init`.
// Selector `reduce(_:_:)` — the trailing block is the second positional
// argument. This is the exact accumulator shape `blocks_argument_to_method`
// was waiting on.

let l = List.new()
l.add(1)
l.add(2)
l.add(3)
System.print(l.reduce(0) { acc, x => acc + x })
