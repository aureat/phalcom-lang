// area: collections
// spec: map-and-set.md §2; U-COLLTYPES plan.md Phase 1; ADR-0039
// status: PASS
// Nesting: a Map value may itself be a List (or Map) — mutation through the
// retrieved reference is visible on the next lookup (Map stores a reference,
// not a snapshot).

let inner = List.new().add(1).add(2)
let m = Map.new()
m.at("nums", put: inner)
System.print(m.at("nums"))
m.at("nums").add(3)
System.print(m.at("nums"))
System.print(m.at("nums").size)

let outer = Map.new()
outer.at("inner", put: Map.new().at("x", put: 1))
System.print(outer.at("inner").at("x"))
System.print(outer.at("inner"))
