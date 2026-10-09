// area: list
// spec: U-LIST-plan.md §7; ADR-0020
// status: PASS
// `each(_:)` iterates 0..size, calling the block with each element — proves
// block-calling into List iteration works.

let l: List<Int> = []
l.append(1)
l.append(2)
l.append(3)
let mut sum = 0
l.each(|v| { sum = sum + v })
System.print(sum)
