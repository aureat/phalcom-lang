// area: blocks
// spec: blocks.md §5; functions.md §2
// status: PASS
let x = 10
let addX = n => n + x

System.print(addX.call(5))
