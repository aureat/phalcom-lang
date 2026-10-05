// area: collections
// spec: map-and-set.md §2; U-COLLTYPES plan.md Phase 1; ADR-0039
// status: PASS
// Nesting: a Map value may itself be a List (or Map) — mutation through the
// retrieved reference is visible on the next lookup (Map stores a reference,
// not a snapshot).

let inner = [1, 2]
let m = Map.new()
m.at("nums", put: inner)
System.print(m["nums"])
m["nums"].append(3)
System.print(m["nums"])
System.print(m["nums"].size)

let outer = Map.new()
outer.at("inner", put: Map.new().at("x", put: 1))
System.print(outer["inner"]["x"])
System.print(outer["inner"])
