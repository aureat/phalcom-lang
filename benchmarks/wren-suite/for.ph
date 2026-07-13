// Ported from wren/test/benchmark/for.wren. `0...1000000` range-in-for has no
// parser production — replaced with a while-counter.
var list = List.new()

var i = 0
while (i < 1000000) {
  list.add(i)
  i = i + 1
}

var sum = 0
for (x in list) {
  sum = sum + x
}

System.print(sum)
