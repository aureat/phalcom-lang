// area: list
// spec: U-LIST-plan.md §7; ADR-0020
// status: PASS
// `each(_:)` iterates 0..size, calling the block with each element — proves
// block-calling into List iteration works.

const l = List.new()
l.add(1)
l.add(2)
l.add(3)
let sum = 0
l.each(|v| { sum = sum + v })
System.print(sum)
