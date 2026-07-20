// area: bytes
// spec: bytes.md law 5, §8; collection-protocol laws 3/4; PDR-0011 rulings 4/6
// status: PASS
// Structural ==/!= (List#=='s shape), non-Bytes argument unequal not error,
// constant-time compare (equal / unequal / length mismatch -> false),
// toTuple as the value-hashable Map-key escape hatch.

const a = Bytes.fromList([1, 2])
const b = Bytes.fromList([1, 2])
const c = Bytes.fromList([1, 3])
System.print(a == b)
System.print(a == c)
System.print(a != c)
System.print(a == "nope")
System.print(a.equalsConstantTime(b))
System.print(a.equalsConstantTime(c))
System.print(a.equalsConstantTime(Bytes.fromList([1, 2, 3])))
const m = Map.new()
m.at(a.toTuple, put: "ok")
System.print(m.at(b.toTuple))
