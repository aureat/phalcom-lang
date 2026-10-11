// area: bytes
// spec: bytes.md law 5, §8; collection-protocol laws 3/4; PDR-0011 rulings 4/6
// status: PASS
// Structural ==/!= (List#=='s shape), non-Bytes argument unequal not error,
// constant-time compare (equal / unequal / length mismatch -> false),
// toTuple as the value-hashable Map-key escape hatch.

let aValues: List<Int> = [1, 2]
let bValues: List<Int> = [1, 2]
let cValues: List<Int> = [1, 3]
let a = Bytes.fromList(aValues)
let b = Bytes.fromList(bValues)
let c = Bytes.fromList(cValues)
System.print(a == b)
System.print(a == c)
System.print(a != c)
System.print(a == "nope")
System.print(a.equalsConstantTime(b))
System.print(a.equalsConstantTime(c))
let longerValues: List<Int> = [1, 2, 3]
System.print(a.equalsConstantTime(Bytes.fromList(longerValues)))
let m: Map<Dynamic, Dynamic> = Map<Dynamic, Dynamic>.new()
m.at(a.toTuple, put: "ok")
System.print(m[b.toTuple])
