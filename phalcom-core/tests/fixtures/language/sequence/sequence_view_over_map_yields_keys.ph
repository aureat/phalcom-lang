// area: sequence
// spec: D.1 Map one-value traversal + E.1 explicit iterator pipeline
// status: PASS
// Views over Map correctly yield keys (Map's for yields keys, not values)

let mut map = Map {a: 1, b: 2, c: 3}
let mut view = map.iter.map |x| { x }
let mut result = []
for x in view {
  result.append(x)
}
System.print(result.size == 3)
