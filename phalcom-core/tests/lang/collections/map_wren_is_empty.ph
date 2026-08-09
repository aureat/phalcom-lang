// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// `Map` implements the shared `Iterable#isEmpty` predicate.

System.print(Map.new().isEmpty)
const m = Map.new()
m.at(1, put: 1)
System.print(m.isEmpty)
