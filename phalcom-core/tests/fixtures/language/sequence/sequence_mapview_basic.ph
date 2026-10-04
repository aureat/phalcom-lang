// area: sequence
// spec: D.1 eager traversal + E.1 explicit iterator pipeline
// status: PASS
// Iterator map applies its function during traversal; no direct collection lazy API is involved.

let mut source = [1, 2, 3]
let mut view = source.iter.map |x| { x * 2 }
let mut result = []
for x in view {
  result.append(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
