// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// Lazy pipeline: where(p).map(f).take(3) composes view wrappers, allocates only wrappers until iteration

let coll = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let view = coll.where |x| { x > 3 }
let mapped = view.map |x| { x * 2 }
let limited = mapped.take(3)
let result = []
for (x in limited) {
  result.add(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
