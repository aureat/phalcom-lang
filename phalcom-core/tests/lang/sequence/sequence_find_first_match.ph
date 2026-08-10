// area: list
// spec: collection-protocol.md §2
// status: PASS
// `find(where:)` returns the first matching element or `None`.

const list = []
list.append(0)
list.append(1)
list.append(2)
list.append(3)
list.append(4)
System.print(list.find(where: |x| { x == 4 }))
System.print(list.find(where: |x| { x == 2 }))
System.print(list.find(where: |x| { x == 100 }))
