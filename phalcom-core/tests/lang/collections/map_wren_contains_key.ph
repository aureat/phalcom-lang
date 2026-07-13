// area: collections
// spec: map-and-set.md §2; ADR-0039
// status: PASS
// Ported from wren/test/core/map/contains_key.wren (Wren's `containsKey` is
// Phalcom's `includes(_)`, the shared collection-protocol membership test).

let m = Map.new()
m.at("one", put: 1)
m.at("two", put: 2)
m.at("three", put: 3)

System.print(m.includes("one"))
System.print(m.includes("two"))
System.print(m.includes("three"))
System.print(m.includes("four"))
System.print(m.includes("five"))
