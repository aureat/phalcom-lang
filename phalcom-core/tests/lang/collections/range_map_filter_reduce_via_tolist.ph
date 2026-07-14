// area: collections
// spec: tuple-and-range.md §2; U-CORE-5 as-built.md §2.1; catalog-delta.md
// status: PASS
// Wren range/map.wren + range/where.wren + range/reduce.wren. `Range` is not
// itself a `Sequence`/`Iterable` root (that generic combinator layer is
// future work — U-ITERABLE/U-SEQ, unlanded); it carries no `map`/`where`/
// `reduce` of its own (tuple-and-range.md §2's selector table lists only
// `each`/`includes`/`size`/`toList`/`first`/`last`). The idiomatic Phalcom
// bridge is `toList` (RG-2's explicit materialization escape hatch) into
// `List`'s spec-anchored combinators (`map`/`filter`/`reduce`,
// catalog-delta.md) — `filter` is Phalcom's name for Wren's `where`.

let r = Range.new(1, 3, true)
let mapped = r.toList.map { x => x + 1 }.toList
System.print(mapped)
let filtered = r.toList.filter { x => x > 1 }
System.print(filtered)
let emptyFiltered = r.toList.filter { x => x > 10 }
System.print(emptyFiltered)
let summed = Range.new(1, 10, true).toList.reduce(0) { acc, x => acc + x }
System.print(summed)
let smallest = Range.new(1, 10, true).toList.reduce(100) { acc, x => (acc < x).ifTrue({ acc }, ifFalse: { x }) }
System.print(smallest)
