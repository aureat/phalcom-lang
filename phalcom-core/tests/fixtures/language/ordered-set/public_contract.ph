// area: collections
// spec: docs/specs/collection-families.md §4
// status: PASS

let mut values = OrderedSet<Int>.new()
values.add(10)
values.add(20)
values.add(10)
System.print(values.size)
System.print(values.contains(20))
System.print(values.indexOf(10).unwrapOr(-1))
System.print(values.at(-1).unwrapOr(-1))

values.insert(30, at: -1)
System.print(values.at(2).unwrapOr(-1))
values.move(2, to: 0)
System.print(values.at(0).unwrapOr(-1))

let replaced = values.replaceAt(1, with: 40)
System.print(replaced.unwrapOr(-1))
System.print(values.at(1).unwrapOr(-1))
let removed = values.removeAt(-1)
System.print(removed.unwrapOr(-1))
values.remove(20)
System.print(values.contains(20))

let mut iterator = values.iterator
System.print(iterator.next.unwrapOr(-1))
System.print(iterator.next.unwrapOr(-1))
System.print(iterator.next.isNone)

values.clear
System.print(values.size)
