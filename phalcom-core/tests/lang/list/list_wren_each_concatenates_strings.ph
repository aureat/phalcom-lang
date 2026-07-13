// area: list
// spec: U-LIST-plan.md §7; ADR-0020
// status: PASS
// Ported from Wren `test/core/list/each.wren`: `each(_:)` visits elements in
// insertion order, driving a side-effecting accumulation outside the block
// (string concatenation, not a numeric sum — the existing
// `list_each_sums_elements` case already covers the numeric shape).

let l = List.new()
l.add("One")
l.add("Two")
l.add("Three")
var words = ""
l.each { word => words = words + word }
System.print(words)
