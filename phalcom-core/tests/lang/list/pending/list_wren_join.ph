// area: list
// spec: docs/spec/v0.2/core/collection-protocol.md §2 (U-SEQ breadth: `join`
//   / `join(sep)`); docs/spec/v0.2/core/core-classes.md §"Iterable"
// status: PENDING
// Ported from Wren `test/core/list/join.wren`. U-SEQ adds `join` (no
// separator) and `join(sep)` (element `toString` joined by `sep`) over the
// shared cursor protocol — not yet landed (`List` has no `join` selector).
// Pinning the Wren-mirrored default (no-arg `join` behaves as `join("")`)
// as the intended surface.

System.print(List.new().join(",") == "")

const a = List.new()
a.add(1)
a.add(2)
a.add(3)
System.print(a.join(""))
System.print(a.join())
