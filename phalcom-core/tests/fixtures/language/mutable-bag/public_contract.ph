// area: collections
// spec: docs/specs/collection-families.md §2
// status: PASS

let mut bag = MutableBag<Int>.new()
bag.add(4, count: 3)
bag.add(2)
System.print(bag.size)
System.print(bag.totalCount)
System.print(bag.distinctCount)
System.print(bag.count(4))
System.print(bag.contains(2))

bag.remove(4, count: 2)
System.print(bag.count(4))
bag.remove(4, count: 20)
System.print(bag.count(4))
System.print(bag.size)

let mut iterator = bag.iterator
System.print(iterator.next.unwrapOr(-1))
System.print(iterator.next.unwrapOr(-1))
System.print(iterator.next.unwrapOr(-1))
System.print(iterator.next.isNone)

bag.clear
System.print(bag.size)
