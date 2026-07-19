// area: sequence
// spec: iteration.md §5; ADR-0035
// status: PASS
// toList materializes Range into List

let list = Range.new(1, 4, false).toList
System.print(list.class.name)
System.print(list.size)
System.print(list.at(0))
System.print(list.at(1))
System.print(list.at(2))
