// Map capabilities
let m = Map.new()
m.at("a", put: 1)
m.at("b", put: 2)
System.print(m.isEmpty) // false
let m_map = m.map { k => k + k }.toList
System.print(m_map.size) // 2
System.print(m_map.at(0)) // "aa"
System.print(m_map.at(1)) // "bb"

let m_filtered = m.filter { k => k == "b" }
System.print(m_filtered.size) // 1
System.print(m_filtered.at(0)) // "b"

let m_reduced = m.reduce("", { acc, k => acc + k })
System.print(m_reduced) // "ab"

// Set capabilities
let s = Set.new().add(10).add(20)
System.print(s.isEmpty) // false
System.print(s.includes(10)) // true
System.print(s.includes(15)) // false
let s_map = s.map { x => x * 10 }.toList
System.print(s_map.at(0)) // 100
System.print(s_map.at(1)) // 200

// Tuple capabilities
let t = Tuple.fromList(List.new().add("hello").add("world"))
System.print(t.isEmpty) // false
System.print(t.includes("hello")) // true
System.print(t.includes("world")) // true
System.print(t.includes("nope")) // false
let t_map = t.map { x => x + "!" }.toList
System.print(t_map.at(0)) // "hello!"
System.print(t_map.at(1)) // "world!"

// Range capabilities
let r = Range.new(1, 4, true) // 1, 2, 3, 4 (Wait, is inclusive true start and end inclusive? Let's check: 1,2,3,4, size 4. Yes, range_inclusive_exclusive_and_laziness.ph has: let inc = Range.new(1, 5, true) inc.size is 5. So let's write: let r = Range.new(1, 3, true) to get 1, 2, 3)
let r = Range.new(1, 3, true)
System.print(r.isEmpty) // false
let r_map = r.map { x => x * 2 }.toList
System.print(r_map.at(0)) // 2
System.print(r_map.at(1)) // 4
System.print(r_map.at(2)) // 6

let r_reduced = r.reduce(0, { acc, x => acc + x })
System.print(r_reduced) // 6
