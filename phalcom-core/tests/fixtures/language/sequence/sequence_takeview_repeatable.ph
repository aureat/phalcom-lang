// area: sequence
// spec: iteration.md §5; ADR-0035; collection-protocol.md law 2
// status: PASS
// Iterator take can be traversed repeatedly with identical results (law-2 compliance)

let mut source = [1, 2, 3, 4, 5]
let mut view = source.iter.take(3)

let mut result1: List<Int> = []
for x in view {
  result1.append(x)
}

let mut result2: List<Int> = []
for x in view {
  result2.append(x)
}

System.print(result1.at(0) == result2.at(0))
System.print(result1.at(1) == result2.at(1))
System.print(result1.at(2) == result2.at(2))
