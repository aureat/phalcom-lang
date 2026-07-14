// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// SkipView skips first n elements

var source = [1, 2, 3, 4, 5]
var view = SkipView.new(source, 2)
var result = []
for (x in view) {
  result.add(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
