// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// MapView iterates with function applied to each element

let source = [1, 2, 3]
let view = MapView.new(source, |x| { x * 2 })
let result = []
for (x in view) {
  result.add(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
