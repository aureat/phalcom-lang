// area: collections
// spec: docs/specs/collection-families.md §3
// status: PASS

let mut dict = IndexedDict<Int, String>.new()
dict.insert("one", for: 1)
dict.insert("two", for: 2)
System.print(dict.size)
System.print(dict[1])
System.print(dict.get(3).isNone)
System.print(dict.contains(2))
System.print(dict.keyAt(-1).unwrapOr(-1))
System.print(dict.valueAt(-2).unwrapOr("missing"))
System.print(dict.indexOfKey(1).unwrapOr(-1))

let previous = dict.insert("ONE", for: 1)
System.print(previous.unwrapOr("missing"))
System.print(dict.keyAt(0).unwrapOr(-1))
System.print(dict.valueAt(0).unwrapOr("missing"))

dict.insert((key: 3, value: "three"), at: -1)
dict.move(0, to: 2)
System.print(dict.keyAt(2).unwrapOr(-1))
System.print(dict.entryAt(-1).isSome)
System.print(dict.entryAt(3).isNone)

let removed = dict.removeAt(-1)
System.print(removed.isOk)
dict.remove(2)
System.print(dict.contains(2))
dict.clear
System.print(dict.size)

