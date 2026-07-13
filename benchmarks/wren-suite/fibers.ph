// Ported from wren/test/benchmark/fibers.wren. `fibers[i + 1]` postfix index
// unsupported (no `[]` read operator, only list *literal* syntax) — replaced
// with `.at(i + 1)`. `0...100000` range-in-for replaced with a while-counter.
var fibers = List.new()
var sum = 0

var i = 0
while (i < 100000) {
  var captured = i
  fibers.add(Fiber.new {
    sum = sum + captured
    if (captured < 99999) { fibers.at(captured + 1).call() }
  })
  i = i + 1
}

fibers.at(0).call()
System.print(sum)
