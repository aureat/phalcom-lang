// area: list
// spec: collection-protocol.md §2
// status: PASS
// `count` and `count(where:)` belong to the shared sequence protocol.

System.print([].count)
System.print([1].count)

const a = []
a.append(1)
a.append(2)
a.append(3)
System.print(a.count(where: |x| { x > 3 }))
System.print(a.count(where: |x| { x > 1 }))
System.print([].count(where: |x| { true }))
