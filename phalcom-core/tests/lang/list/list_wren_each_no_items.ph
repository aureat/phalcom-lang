// area: list
// spec: U-LIST-plan.md §7; ADR-0020
// status: PASS
// Ported from Wren `test/core/list/each_no_items.wren`: `each(_:)` on an
// empty list never invokes the block — zero iterations, not a panic on a
// zero-length receiver.

const empty = List.new()
let count = 0
empty.each |item| { count = count + 1 }
System.print(count)
