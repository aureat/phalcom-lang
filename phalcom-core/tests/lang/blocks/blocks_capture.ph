// area: blocks
// spec: blocks.md §5; functions.md §2
// status: PASS
const x = 10
const addX = n => n + x

System.print(addX.call(5))
