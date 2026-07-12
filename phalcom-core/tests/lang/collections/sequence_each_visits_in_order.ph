// area: collections
// spec: U-CORE-5 as-built.md §2.1, §3.3(b); R-INV-5.2 (L6)
// status: PASS
// `each(_:)` visits `at(0)..at(size-1)` exactly once, in insertion order.

let l = List.new()
l.add(10)
l.add(20)
l.add(30)
l.each { x => System.print(x) }
