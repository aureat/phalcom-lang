// area: collections
// spec: map-and-set.md §2/§3; U-COLLTYPES plan.md Phase 1; ADR-0039
// status: PASS
// Map/Set surface protocol: construction, keyed put/get/overwrite, includes,
// remove idempotence, keys/values in iteration order, Set add idempotence.

const m = Map.new()
System.print(m.size)
m.at(1, put: 10)
m.at(2, put: 20)
System.print(m.size)
System.print(m.at(1))
m.at(1, put: 99)
System.print(m.at(1))
System.print(m.includes(2))
System.print(m.includes(5))
m.remove(2)
System.print(m.size)
System.print(m.keys)
System.print(m.values)

const s = Set.new()
s.add(1).add(2).add(1)
System.print(s.size)
System.print(s.includes(2))
s.remove(1)
System.print(s.size)
System.print(s.includes(1))
