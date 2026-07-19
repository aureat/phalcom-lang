// area: sequence
// spec: iteration.md §5; ADR-0035; collection-protocol.md law 2
// status: PASS
// TakeView can be traversed repeatedly with identical results (law-2 compliance)

let source = [1, 2, 3, 4, 5]
let view = TakeView.new(source, 3)

let result1 = []
for (x in view) {
  result1.add(x)
}

let result2 = []
for (x in view) {
  result2.add(x)
}

System.print(result1.at(0) == result2.at(0))
System.print(result1.at(1) == result2.at(1))
System.print(result1.at(2) == result2.at(2))
