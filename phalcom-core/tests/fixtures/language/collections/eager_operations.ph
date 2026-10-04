// area: collections
// spec: eager operations
// status: PASS

System.print("--- fold ---")
let list1 = [1, 2, 3, 4]
System.print(list1.fold(initial: 0, using: |acc, x| { acc + x }))
System.print(list1.fold(initial: 10, using: |acc, x| { acc + x }))

System.print("--- group(by:) ---")
// empty
let emptyList = []
System.print(emptyList.group(by: |x| { x }).size)

// non-empty, repeat keys, encounter order
let list2 = [1, 2, 3, 4, 5, 6]
let mut countGroup = 0
let groupMap = list2.group(by: |x| {
  countGroup = countGroup + 1
  x % 2
})
System.print(groupMap.size)
System.print(groupMap[0])
System.print(groupMap[1])
System.print(countGroup)

System.print("--- partition(where:) ---")
// empty
let pEmpty = emptyList.partition(where: |x| { true })
System.print(pEmpty.at(0).size)
System.print(pEmpty.at(1).size)
System.print(pEmpty.at(0) == pEmpty.at(1))

// all true
let pAllTrue = [1, 2, 3].partition(where: |x| { true })
System.print(pAllTrue.at(0))
System.print(pAllTrue.at(1))

// all false
let pAllFalse = [1, 2, 3].partition(where: |x| { false })
System.print(pAllFalse.at(0))
System.print(pAllFalse.at(1))

// mixed
let pMixed = [1, 2, 3, 4].partition(where: |x| { x % 2 == 0 })
System.print(pMixed.at(0))
System.print(pMixed.at(1))

System.print("--- toList ---")
let origList = [1, 2, 3]
let copiedList = origList.toList
System.print(copiedList == origList)
copiedList.append(4)
System.print(origList.size)
System.print(copiedList.size)

System.print("--- toSet ---")
let listWithDupes = [1, 2, 2, 3, 1, 4]
let setVal = listWithDupes.toSet
System.print(setVal.size)
System.print(setVal.includes(1))
System.print(setVal.includes(2))
System.print(setVal.includes(3))
System.print(setVal.includes(4))
System.print(setVal.includes(5))

System.print("--- toMap ---")
let entries = [Entry.new("a", 1), Entry.new("b", 2)]
let mapResult = entries.toMap
System.print(mapResult.isOk)
let m1 = mapResult.unwrap
System.print(m1.size)
System.print(m1["a"])
System.print(m1["b"])

let entriesWithDupes = [
  Entry.new("a", 1),
  Entry.new("b", 2),
  Entry.new("a", 3),
  Entry.new("b", 4)
]
let badResult = entriesWithDupes.toMap
System.print(badResult.isErr)
let err = badResult.unwrapErr
System.print(err.class == DuplicateKeyError)
System.print(err.key)
System.print(err.message)

let entriesWithNone = [Entry.new("a", None)]
let noneMapRes = entriesWithNone.toMap
System.print(noneMapRes.isOk)
System.print(noneMapRes.unwrap["a"] == None)

System.print("--- toMap(merging:) ---")
let entriesForMerge = [
  Entry.new("a", 1),
  Entry.new("b", 2),
  Entry.new("a", 10),
  Entry.new("c", 3),
  Entry.new("b", 20)
]
let mergedMap = entriesForMerge.toMap(merging: |v1, v2| { v1 + v2 })
System.print(mergedMap)
System.print(mergedMap["a"])
System.print(mergedMap["b"])
System.print(mergedMap["c"])
