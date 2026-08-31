// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// Iterator skip skips first n elements

let source = [1, 2, 3, 4, 5]
let view = source.iter.skip(2)
let result = []
for x in view {
  result.append(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
