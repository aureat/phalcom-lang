// area: list
// spec: docs/spec/v0.2/core/collection-protocol.md §2 (U-SEQ breadth: `count`
//   / `count(f)`); docs/spec/v0.2/core/core-classes.md §"Iterable"
// status: PENDING
// Ported from Wren `test/core/list/count.wren` + `count_predicate.wren`.
// U-SEQ (deps U-ITERABLE, not yet landed) adds `count` as a total-size
// synonym (mirroring `size`) and `count(f)` as a predicate-counting
// combinator over the shared cursor protocol. Pinning the intended surface,
// not today's behavior — `List` has no `count` selector yet (`size` is the
// only element-count getter currently defined).

System.print([].count)
System.print([1].count)

const a = []
a.append(1)
a.append(2)
a.append(3)
System.print(a.count |x| { x > 3 })
System.print(a.count |x| { x > 1 })
System.print([].count |x| { true })
