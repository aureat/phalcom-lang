// area: sequence
// spec: D.1 eager traversal + E.1 explicit iterator pipeline
// status: PASS
// Iterator map applies its function during traversal; no direct collection lazy API is involved.

let source = [1, 2, 3]
let view = source.iter.map |x| { x * 2 }
let result = []
for (x in view) {
  result.append(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
