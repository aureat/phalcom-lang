// area: collections
// spec: tuple-and-range.md §2; ADR-0035 §1; iteration.md §1
// status: PASS
// Wren range/iterate.wren + range/iterator_value.wren, ported onto
// Phalcom's Option-based cursor protocol (not Wren's raw
// null/int/false): `iterate(None)` starts at `Some(0)`,
// `iteratorValue(_)` is `start + cursor` (never bounds- or type-checked, so
// it round-trips a raw out-of-range cursor too — same "doesn't bother to
// check" contract Wren pins).

let r = Range.new(1, 3, true)
System.print(r.iterate(None))
System.print(r.iterate(Some.new(0)))
System.print(r.iterate(Some.new(1)))
System.print(r.iterate(Some.new(2)))

let e = Range.new(1, 3, false)
System.print(e.iterate(None))
System.print(e.iterate(Some.new(0)))
System.print(e.iterate(Some.new(1)))

let empty = Range.new(1, 1, false)
System.print(empty.iterate(None))

System.print(r.iteratorValue(0))
System.print(r.iteratorValue(2))
System.print(r.iteratorValue(-2))
System.print(r.iteratorValue(5))

var c = r.iterate(None)
while (c.isSome) {
  System.print(r.iteratorValue(c.unwrapOr(0)))
  c = r.iterate(c)
}
