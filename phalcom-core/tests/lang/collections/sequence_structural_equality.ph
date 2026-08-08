// area: collections
// spec: U-CORE-5 as-built.md §2.2, §3.2, §3.3(b); R-INV-5.3 (E1,E3,E6)
// status: PASS
// `List#==` is structural (element-wise, order-sensitive), and `List#!=`
// routes through it rather than staying identity-based (the `==`/`!=`
// decoupling hazard).

const a = []
a.append(1)
a.append(2)
a.append(3)
const b = []
b.append(1)
b.append(2)
b.append(3)
const c = []
c.append(1)
c.append(9)
c.append(3)
System.print(a == b)
System.print(a == c)
System.print(a != c)
System.print(a == a)
