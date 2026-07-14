// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// Lazy pipeline: where(p).map(f).take(3) composes view wrappers, allocates only wrappers until iteration

var coll = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
var view = coll.where { x => x > 3 }
var mapped = view.map { x => x * 2 }
var limited = mapped.take(3)
var result = []
for (x in limited) {
  result.add(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
