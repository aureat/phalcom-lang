// area: collections
// spec: U-CORE-5 as-built.md §2.2, §3.2, §3.3(b); R-INV-5.3 (E1,E2)
// status: PASS
// Structural equality recurses into nested lists (deep equality), is
// order-sensitive, holds for two empty lists, and is `false` across kinds
// (a `List` is never `==` to a non-`List`, without raising).

const nestedA = []
const innerA = []
innerA.append(1)
innerA.append(2)
nestedA.append(innerA)
nestedA.append(3)

const nestedB = []
const innerB = []
innerB.append(1)
innerB.append(2)
nestedB.append(innerB)
nestedB.append(3)

System.print(nestedA == nestedB)

const reordered = []
const innerC = []
innerC.append(2)
innerC.append(1)
reordered.append(innerC)
reordered.append(3)
System.print(nestedA == reordered)

const emptyA = []
const emptyB = []
System.print(emptyA == emptyB)

System.print(nestedA == 3)
