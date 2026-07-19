// area: list
// spec: docs/spec/v0.2/core/collection-protocol.md §2 (U-SEQ breadth:
//   `find(f)` → `Option`)
// status: PENDING
// Adapted from Wren `test/core/list/index_of.wren` (Wren's `indexOf(_)`
// returns a raw index or `-1`; Phalcom's planned analog is `find(f)` →
// `Option` — the first element satisfying the predicate, `None` on no
// match, never a sentinel `-1`, ADR-0007/Invariant 4). Not yet landed
// (`List` has no `find` selector).

const list = List.new()
list.add(0)
list.add(1)
list.add(2)
list.add(3)
list.add(4)
System.print(list.find { x => x == 4 })
System.print(list.find { x => x == 2 })
System.print(list.find { x => x == 100 })
