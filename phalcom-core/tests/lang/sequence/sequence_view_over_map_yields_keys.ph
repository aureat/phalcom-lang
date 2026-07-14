// area: sequence
// spec: iteration.md §5; ADR-0035; iteration.md §3.2
// status: PASS
// Views over Map correctly yield keys (Map's for yields keys, not values)

var map = {a: 1, b: 2, c: 3}
var view = MapView.new(map, { x => x })
var result = []
for (x in view) {
  result.add(x)
}
System.print(result.size == 3)
