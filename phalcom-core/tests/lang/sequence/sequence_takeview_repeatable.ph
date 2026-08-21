// area: sequence
// spec: iteration.md §5; ADR-0035; collection-protocol.md law 2
// status: PASS
// Iterator take can be traversed repeatedly with identical results (law-2 compliance)

let source = [1, 2, 3, 4, 5]
let view = source.iter.take(3)

let result1 = []
for x in view {
  result1.append(x)
}

let result2 = []
for x in view {
  result2.append(x)
}

System.print(result1.at(0) == result2.at(0))
System.print(result1.at(1) == result2.at(1))
System.print(result1.at(2) == result2.at(2))
