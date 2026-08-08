// area: sequence
// spec: iteration.md §5; ADR-0035; iteration.md §3.2
// status: PASS
// Views over Map correctly yield keys (Map's for yields keys, not values)

let map = {a: 1, b: 2, c: 3}
let view = map.iter.map |x| { x }
let result = []
for (x in view) {
  result.append(x)
}
System.print(result.size == 3)
