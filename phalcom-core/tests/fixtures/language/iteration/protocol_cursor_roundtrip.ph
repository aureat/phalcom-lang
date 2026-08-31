// C-ITER-9 (U-ITER, ADR-0035 §1, iteration.md §1): the two-selector cursor
// protocol on `List`, driven directly (before `for` exists) by a hand-written
// `while` over `iterate(_)`/`iteratorValue(_)`. Proves the `.ph` contract:
// `iterate(None)` starts at 0, advances, and reports `None` past the
// end; `iteratorValue(_)` round-trips `at(_)`.
const xs = [7, 8]
System.print(xs.iterate(None))          // 0
System.print(xs.iterate(0))             // 1
System.print(xs.iterate(1))             // None (past end)
System.print(xs.iteratorValue(0))       // 7
System.print(xs.iteratorValue(1))       // 8

// Full traversal via the cursor protocol.
let c = xs.iterate(None)
while (c != None) {
  System.print(xs.iteratorValue(c))
  c = xs.iterate(c)
}
