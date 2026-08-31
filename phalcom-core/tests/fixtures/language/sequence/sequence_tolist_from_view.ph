// area: sequence
// spec: D.1 eager traversal + E.1 explicit iterator pipeline
// status: PASS
// toList materializes a view into List

let view = [1, 2, 3, 4].iter.filter |x| { x > 2 }
let list = view.toList
System.print(list.class.name)
System.print(list.size)
System.print(list.at(0))
System.print(list.at(1))
