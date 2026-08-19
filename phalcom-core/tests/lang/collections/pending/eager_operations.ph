// area: collections
// spec: D.1
// status: PASS

// 22.1 Eagerness
let counter = 0
let list_123 = [1, 2, 3]
let map_res = list_123.map |x| {
  counter = counter + 1
  x * 2
}
System.print(counter)
System.print(map_res.is(List))
System.print(map_res.at(0))
System.print(map_res.at(1))
System.print(map_res.at(2))

// 22.2 Transform order/content
// map of empty/singleton/multiple
System.print([].map |x| { x })
System.print([5].map |x| { x * 10 })
// filter retain none/some/all
let filtered = list_123.filter |x| { x > 1 }
System.print(filtered.at(0))
System.print(filtered.at(1))
System.print(list_123.filter |x| { x > 5 })
System.print(list_123.filter |x| { x > 0 })
// flatMap empty inner values, multiple inner values, nested encounter order
let flat_res = list_123.flatMap |x| { [x, x * 10] }
System.print(flat_res.at(0))
System.print(flat_res.at(1))
System.print(flat_res.at(2))
System.print(flat_res.at(3))
System.print(flat_res.at(4))
System.print(flat_res.at(5))

let flat_empty = list_123.flatMap |x| { [] }
System.print(flat_empty)

// stored/mapped None
let map_none = [1].map |x| { None }
System.print(map_none.at(0))

// 22.3 Indexed variants
// map(indexed:) and each(indexed:)
let map_idx = list_123.map(indexed: |i, x| { i.toString + ":" + x.toString })
System.print(map_idx.at(0))
System.print(map_idx.at(1))
System.print(map_idx.at(2))

let each_idx_res = []
list_123.each(indexed: |i, x| {
  each_idx_res.append(i.toString + "-" + x.toString)
})
System.print(each_idx_res.at(0))
System.print(each_idx_res.at(1))
System.print(each_idx_res.at(2))

// non-List iterable: Range (since Range is Iterable)
let range_idx = (1..3).map(indexed: |i, x| { i.toString + ":" + x.toString })
System.print(range_idx.at(0))
System.print(range_idx.at(1))
System.print(range_idx.at(2))

// 22.4 Query identities and short circuit
let any_calls = 0
let any_match = list_123.any(where: |x| {
  any_calls = any_calls + 1
  x == 2
})
System.print(any_match)
System.print(any_calls)

let all_calls = 0
let all_match = list_123.all(where: |x| {
  all_calls = all_calls + 1
  x == 1
})
System.print(all_match)
System.print(all_calls)

let none_calls = 0
let none_match = list_123.none(where: |x| {
  none_calls = none_calls + 1
  x == 1
})
System.print(none_match)
System.print(none_calls)

let find_calls = 0
let find_match = list_123.find(where: |x| {
  find_calls = find_calls + 1
  x == 2
})
System.print(find_match.unwrapOr(None))
System.print(find_calls)

let index_calls = 0
let index_match = list_123.index(where: |x| {
  index_calls = index_calls + 1
  x == 2
})
System.print(index_match.unwrapOr(None))
System.print(index_calls)

let count_calls = 0
let count_match = list_123.count(where: |x| {
  count_calls = count_calls + 1
  x > 1
})
System.print(count_match)
System.print(count_calls)

// empty identities
System.print([].any(where: |x| { true }))
System.print([].all(where: |x| { false }))
System.print([].none(where: |x| { false }))

// 22.5 Fold/reduce
// fold empty returns exact initial object
let initial_obj = [9, 9]
System.print([].fold(initial: initial_obj, using: |acc, x| { acc }) == initial_obj)
// fold order with three values
let fold_val = list_123.fold(initial: 100, using: |acc, x| { acc - x })
System.print(fold_val)

// reduce empty -> None
System.print([].reduce(using: |a, b| { a + b }))
// reduce singleton -> Some(element), callback count 0
let red_calls = 0
let red_single = [42].reduce(using: |a, b| {
  red_calls = red_calls + 1
  a + b
})
System.print(red_single.unwrapOr(None))
System.print(red_calls)

// reduce multiple -> Some(result)
let red_mult = list_123.reduce(using: |a, b| { a + b })
System.print(red_mult.unwrapOr(None))

// reduce one-element absence surface None -> Some(None)
let red_none = [None].reduce(using: |a, b| { a })
System.print(red_none.unwrapOr(None))

// 22.6 Map migration
let m = Map.new()
m.at("k1", put: "v1")
m.at("k2", put: "v2")

let map_each_res = []
m.each |x| {
  map_each_res.append(x)
}
System.print(map_each_res.at(0))
System.print(map_each_res.at(1))

let entries_each_res = []
m.entries.each |entry| {
  entries_each_res.append(entry.key + "=" + entry.value)
}
System.print(entries_each_res.at(0))
System.print(entries_each_res.at(1))

// first and last on List, Tuple, Bytes
let t = (10, 20, 30)
System.print(list_123.first.unwrapOr(None))
System.print(list_123.last.unwrapOr(None))
System.print(t.first.unwrapOr(None))
System.print(t.last.unwrapOr(None))

let b = Bytes.new(2)
b[0] = 65
b[1] = 66
System.print(b.first.unwrapOr(None))
System.print(b.last.unwrapOr(None))

System.print([].first)
System.print([].last)
