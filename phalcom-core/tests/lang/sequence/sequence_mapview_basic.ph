// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// Iterator map applies its function during traversal

let source = [1, 2, 3]
let view = source.iter.map |x| { x * 2 }
let result = []
for (x in view) {
  result.append(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
