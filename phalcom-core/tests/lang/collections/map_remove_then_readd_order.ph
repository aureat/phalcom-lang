// area: collections
// spec: map-and-set.md §2; U-COLLTYPES plan.md Phase 1; ADR-0039
// status: PASS
// Adversarial: remove a key, then re-add it — pins whatever iteration order
// the backing store actually produces afterward (swap-remove reshuffles the
// probe sequence; this is a corner the thin corpus never exercised).

let m = Map.new()
m.at(1, put: "a")
m.at(2, put: "b")
m.at(3, put: "c")
m.remove(2)
System.print(m.size)
System.print(m.keys)
System.print(m.values)
m.at(2, put: "d")
System.print(m.size)
System.print(m.keys)
System.print(m.values)
System.print(m.includes(2))
System.print(m.at(2))
