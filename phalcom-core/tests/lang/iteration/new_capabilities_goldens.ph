// Map capabilities
const m = Map.new()
m.at("a", put: 1)
m.at("b", put: 2)
System.print(m.isEmpty) // false
const m_map = m.map { k => k + k }.toList
System.print(m_map.size) // 2
System.print(m_map.at(0)) // "aa"
System.print(m_map.at(1)) // "bb"

const m_filtered = m.filter { k => k == "b" }
System.print(m_filtered.size) // 1
System.print(m_filtered.at(0)) // "b"

const m_reduced = m.reduce("", { acc, k => acc + k })
System.print(m_reduced) // "ab"

// Set capabilities
const s = Set.new().add(10).add(20)
System.print(s.isEmpty) // false
System.print(s.includes(10)) // true
System.print(s.includes(15)) // false
const s_map = s.map { x => x * 10 }.toList
System.print(s_map.at(0)) // 100
System.print(s_map.at(1)) // 200

// Tuple capabilities
const t = Tuple.__fromList(List.new().add("hello").add("world"))
System.print(t.isEmpty) // false
System.print(t.includes("hello")) // true
System.print(t.includes("world")) // true
System.print(t.includes("nope")) // false
const t_map = t.map { x => x + "!" }.toList
System.print(t_map.at(0)) // "hello!"
System.print(t_map.at(1)) // "world!"

// Range capabilities
// Range.new(1, 3, true) is inclusive on both ends: 1, 2, 3 (size 3).
const r = Range.new(1, 3, true)
System.print(r.isEmpty) // false
const r_map = r.map { x => x * 2 }.toList
System.print(r_map.at(0)) // 2
System.print(r_map.at(1)) // 4
System.print(r_map.at(2)) // 6

const r_reduced = r.reduce(0, { acc, x => acc + x })
System.print(r_reduced) // 6
