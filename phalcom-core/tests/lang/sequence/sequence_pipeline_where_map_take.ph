// area: sequence
// spec: D.1 eager traversal + E.1 explicit iterator pipeline
// status: PASS
// Explicit iterator pipeline: filter(p).map(f).take(3) composes lazy stages.

let coll = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
let view = coll.iter.filter |x| { x > 3 }
let mapped = view.map |x| { x * 2 }
let limited = mapped.take(3)
let result = []
for x in limited {
  result.append(x)
}
System.print(result.at(0))
System.print(result.at(1))
System.print(result.at(2))
