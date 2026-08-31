// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// `clear()` empties a Map in place.

const a = Map.new()
a.at(1, put: 1)
a.at(2, put: 2)
a.at(3, put: 3)
a.clear
System.print(a)
System.print(a.size)
