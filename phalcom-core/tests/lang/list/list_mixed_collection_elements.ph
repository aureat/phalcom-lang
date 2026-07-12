// area: list
// spec: U-LIST-plan.md; U-COLLTYPES plan.md; ADR-0007; ADR-0020
// status: PASS
// Cross-kind: a List holding heterogeneous native collection elements
// (Tuple, Map, Set) — `at(_, put:)` overwriting one slot's element type
// leaves neighboring elements/kinds untouched.

let l = List.new()
l.add(Tuple.fromList(List.new().add(1).add(2)))
l.add(Map.new().at("k", put: 1))
l.add(Set.new().add(1).add(2))
System.print(l.size)
System.print(l.at(0))
System.print(l.at(1))
System.print(l.at(2).size)
System.print(l.at(2).includes(1))

l.at(0, put: Tuple.fromList(List.new()))
System.print(l.at(0).size)
System.print(l.at(1))
System.print(l.at(2).size)
