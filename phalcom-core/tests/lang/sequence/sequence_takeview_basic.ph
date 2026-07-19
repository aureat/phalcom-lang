// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// TakeView takes first n elements

let source = [1, 2, 3, 4, 5]
let view = TakeView.new(source, 3)
let result = []
for (x in view) {
  result.add(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
