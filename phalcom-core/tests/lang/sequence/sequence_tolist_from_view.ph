// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// toList materializes a view into List

let view = WhereView.new([1, 2, 3, 4], { x => x > 2 })
let list = view.toList
System.print(list.class.name)
System.print(list.size)
System.print(list.at(0))
System.print(list.at(1))
