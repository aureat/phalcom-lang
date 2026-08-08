// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// Iterator take takes first n elements

let source = [1, 2, 3, 4, 5]
let view = source.iter.take(3)
let result = []
for (x in view) {
  result.append(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
