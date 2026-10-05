// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// Iterator skip skips first n elements

let mut source = [1, 2, 3, 4, 5]
let mut view = source.iter.skip(2)
let mut result = []
for x in view {
  result.append(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
