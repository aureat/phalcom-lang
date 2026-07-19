// area: collections
// spec: tuple-and-range.md §2; ADR-0035 §1; iteration.md §1; ADR-0048
// status: PASS
// Wren range/iterate.wren + range/iterator_value.wren, ported onto
// Phalcom's bare-index/`None`-end-sentinel cursor protocol (post-Route-B,
// U-ITERABLE, ADR-0048 — not Wren's raw null/int/false, and not the earlier
// Option-wrapped-cursor idiom either): `iterate(None)` starts at `0`,
// `iteratorValue(_)` is `start + cursor` (never bounds- or type-checked, so
// it round-trips a raw out-of-range cursor too — same "doesn't bother to
// check" contract Wren pins).

const r = Range.new(1, 3, true)
System.print(r.iterate(None))
System.print(r.iterate(0))
System.print(r.iterate(1))
System.print(r.iterate(2))

const e = Range.new(1, 3, false)
System.print(e.iterate(None))
System.print(e.iterate(0))
System.print(e.iterate(1))

const empty = Range.new(1, 1, false)
System.print(empty.iterate(None))

System.print(r.iteratorValue(0))
System.print(r.iteratorValue(2))
System.print(r.iteratorValue(-2))
System.print(r.iteratorValue(5))

let c = r.iterate(None)
while (c != None) {
  System.print(r.iteratorValue(c))
  c = r.iterate(c)
}
