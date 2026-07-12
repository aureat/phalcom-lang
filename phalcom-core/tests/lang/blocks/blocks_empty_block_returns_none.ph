// area: blocks
// spec: blocks.md §5 (block value)
// status: PASS
// An empty block body `{}` has no expression to yield, so calling it must
// surface the `None` singleton (U6: surface `nil` is gone; ADR-0007/ADR-0010).
let empty = {}
System.print(empty.call())
