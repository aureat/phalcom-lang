// area: blocks
// spec: blocks.md §5 (shared mutation while the captured slot is open)
// status: PASS
var count = 0
let increment = { count = count + 1 }

increment.call()
increment.call()
System.print(count)
