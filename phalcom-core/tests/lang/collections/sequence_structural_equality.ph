// area: collections
// spec: U-CORE-5 as-built.md §2.2, §3.2, §3.3(b); R-INV-5.3 (E1,E3,E6)
// status: PASS
// `List#==` is structural (element-wise, order-sensitive), and `List#!=`
// routes through it rather than staying identity-based (the `==`/`!=`
// decoupling hazard).

let a = List.new()
a.add(1)
a.add(2)
a.add(3)
let b = List.new()
b.add(1)
b.add(2)
b.add(3)
let c = List.new()
c.add(1)
c.add(9)
c.add(3)
System.print(a == b)
System.print(a == c)
System.print(a != c)
System.print(a == a)
