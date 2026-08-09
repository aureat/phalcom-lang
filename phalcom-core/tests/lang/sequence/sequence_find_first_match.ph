// area: list
// spec: collection-protocol.md §2
// status: PASS
// `find(_)` returns the first matching element or `None`.

const list = []
list.append(0)
list.append(1)
list.append(2)
list.append(3)
list.append(4)
System.print(list.find |x| { x == 4 })
System.print(list.find |x| { x == 2 })
System.print(list.find |x| { x == 100 })
