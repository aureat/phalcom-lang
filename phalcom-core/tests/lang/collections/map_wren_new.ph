// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// Ported from wren/test/core/map/new.wren — `Map.new()` starts empty; the
// native `toString` renders it `{}` (Wren's own expectation, unchanged).

let m = Map.new()
System.print(m.size)
System.print(m)
