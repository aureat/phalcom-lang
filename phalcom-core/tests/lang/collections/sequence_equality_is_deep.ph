// area: collections
// spec: U-CORE-5 as-built.md §2.2, §3.2, §3.3(b); R-INV-5.3 (E1,E2)
// status: PASS
// Structural equality recurses into nested lists (deep equality), is
// order-sensitive, holds for two empty lists, and is `false` across kinds
// (a `List` is never `==` to a non-`List`, without raising).

const nestedA = List.new()
const innerA = List.new()
innerA.add(1)
innerA.add(2)
nestedA.add(innerA)
nestedA.add(3)

const nestedB = List.new()
const innerB = List.new()
innerB.add(1)
innerB.add(2)
nestedB.add(innerB)
nestedB.add(3)

System.print(nestedA == nestedB)

const reordered = List.new()
const innerC = List.new()
innerC.add(2)
innerC.add(1)
reordered.add(innerC)
reordered.add(3)
System.print(nestedA == reordered)

const emptyA = List.new()
const emptyB = List.new()
System.print(emptyA == emptyB)

System.print(nestedA == 3)
