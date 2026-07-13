// area: list
// spec: ADR-0035 §1; ADR-0048; iteration.md §1; U-LIST-plan.md §7
// status: PASS
// Ported from Wren `test/core/list/iterate.wren`, adapted to the Phalcom
// cursor protocol (ADR-0048's bare-cursor/`None` end sentinel replaces
// Wren's raw `null`/index/`false` triad — Invariant 4, no surface `nil`):
// `iterate(_)` takes the previous cursor (`None` to start) and returns
// `Some(nextIndex)` while elements remain, `None` once exhausted; an empty
// list yields `None` immediately.

let a = List.new()
a.add("one")
a.add("two")
a.add("three")
a.add("four")
System.print(a.iterate(None))
System.print(a.iterate(Some.new(0)))
System.print(a.iterate(Some.new(1)))
System.print(a.iterate(Some.new(2)))
System.print(a.iterate(Some.new(3)))

// Nothing to iterate in an empty list.
System.print(List.new().iterate(None))
