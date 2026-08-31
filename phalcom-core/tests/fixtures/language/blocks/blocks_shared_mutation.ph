// area: blocks
// spec: blocks.md §5 (shared mutation while the captured slot is open)
// status: PASS
let count = 0
const increment = || { count = count + 1 }

increment.call()
increment.call()
System.print(count)
