// area: collections
// spec: eager operations
// status: PASS

System.print("--- fold ---")
const list1 = [1, 2, 3, 4]
System.print(list1.fold(0, |acc, x| { acc + x }))
System.print(list1.fold(10, using: |acc, x| { acc + x }))

System.print("--- group(by:) ---")
// empty
const emptyList = []
System.print(emptyList.group(by: |x| { x }).size)

// non-empty, repeat keys, encounter order
const list2 = [1, 2, 3, 4, 5, 6]
let countGroup = 0
const groupMap = list2.group(by: |x| {
  countGroup = countGroup + 1
  x % 2
})
System.print(groupMap.size)
System.print(groupMap[0])
System.print(groupMap[1])
System.print(countGroup)

System.print("--- partition(where:) ---")
// empty
const pEmpty = emptyList.partition(where: |x| { true })
System.print(pEmpty.at(0).size)
System.print(pEmpty.at(1).size)
System.print(pEmpty.at(0) == pEmpty.at(1))

// all true
const pAllTrue = [1, 2, 3].partition(where: |x| { true })
System.print(pAllTrue.at(0))
System.print(pAllTrue.at(1))

// all false
const pAllFalse = [1, 2, 3].partition(where: |x| { false })
System.print(pAllFalse.at(0))
System.print(pAllFalse.at(1))

// mixed
const pMixed = [1, 2, 3, 4].partition(where: |x| { x % 2 == 0 })
System.print(pMixed.at(0))
System.print(pMixed.at(1))

System.print("--- toList ---")
const origList = [1, 2, 3]
const copiedList = origList.toList
System.print(copiedList == origList)
copiedList.append(4)
System.print(origList.size)
System.print(copiedList.size)

System.print("--- toSet ---")
const listWithDupes = [1, 2, 2, 3, 1, 4]
const setVal = listWithDupes.toSet
System.print(setVal.size)
System.print(setVal.includes(1))
System.print(setVal.includes(2))
System.print(setVal.includes(3))
System.print(setVal.includes(4))
System.print(setVal.includes(5))

System.print("--- toMap ---")
const entries = [Entry.new("a", 1), Entry.new("b", 2)]
const mapResult = entries.toMap
System.print(mapResult.isOk)
const m1 = mapResult.unwrap
System.print(m1.size)
System.print(m1["a"])
System.print(m1["b"])

const entriesWithDupes = [
  Entry.new("a", 1),
  Entry.new("b", 2),
  Entry.new("a", 3),
  Entry.new("b", 4)
]
const badResult = entriesWithDupes.toMap
System.print(badResult.isErr)
const err = badResult.unwrapErr
System.print(err.class == DuplicateKeyError)
System.print(err.key)
System.print(err.message)

const entriesWithNone = [Entry.new("a", None)]
const noneMapRes = entriesWithNone.toMap
System.print(noneMapRes.isOk)
System.print(noneMapRes.unwrap["a"] == None)

System.print("--- toMap(merging:) ---")
const entriesForMerge = [
  Entry.new("a", 1),
  Entry.new("b", 2),
  Entry.new("a", 10),
  Entry.new("c", 3),
  Entry.new("b", 20)
]
const mergedMap = entriesForMerge.toMap(merging: |v1, v2| { v1 + v2 })
System.print(mergedMap)
System.print(mergedMap["a"])
System.print(mergedMap["b"])
System.print(mergedMap["c"])
