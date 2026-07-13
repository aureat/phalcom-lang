// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PENDING
// Ported from wren/test/core/map/is_empty.wren. `List` has `isEmpty`
// (`self.size == 0`, core.ph), but `Map` does not define it — `{}.isEmpty`
// currently raises `does not understand 'isEmpty'`. Pinning the intended
// surface (a `Map#isEmpty` sibling to `List#isEmpty`).

System.print(Map.new().isEmpty)
let m = Map.new()
m.at(1, put: 1)
System.print(m.isEmpty)
