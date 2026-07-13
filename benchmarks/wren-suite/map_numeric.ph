// Ported from wren/test/benchmark/map_numeric.wren. `map[i] = i` / `map[i]`
// index-assign/read has no Phalcom parser production at all (no postfix `[]`
// operator, only `[a, b, c]` literal syntax) — replaced with the documented
// Map surface `.at(k, put: v)` / `.at(k)` (phalcom-core/core/core.ph, class
// Map). `{}` is the empty-BLOCK literal in Phalcom, not empty-map (spec §6)
// — `{}` -> `Map.new()`. `1..2000000` (inclusive) range-in-for replaced with
// a while-counter.
var map = Map.new()

var i = 1
while (i <= 2000000) {
  map.at(i, put: i)
  i = i + 1
}

var sum = 0
i = 1
while (i <= 2000000) {
  sum = sum + map.at(i)
  i = i + 1
}
System.print(sum)

i = 1
while (i <= 2000000) {
  map.remove(i)
  i = i + 1
}
