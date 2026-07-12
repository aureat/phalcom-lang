// C-ITER-9 (U-ITER, ADR-0035 §1, iteration.md §1): the two-selector cursor
// protocol on `List`, driven directly (before `for` exists) by a hand-written
// `while` over `iterate(_)`/`iteratorValue(_)`. Proves the `.ph` contract:
// `iterate(None)` starts at `Some(0)`, advances, and reports `None` past the
// end; `iteratorValue(_)` round-trips `at(_)`.
let xs = List.new().add(7).add(8)
System.print(xs.iterate(None))          // Some(0)
System.print(xs.iterate(Some.new(0)))   // Some(1)
System.print(xs.iterate(Some.new(1)))   // None (past end)
System.print(xs.iteratorValue(0))       // 7
System.print(xs.iteratorValue(1))       // 8

// Full traversal via the cursor protocol.
var c = xs.iterate(None)
while (c.isSome) {
  System.print(xs.iteratorValue(c.unwrapOr(0)))
  c = xs.iterate(c)
}
