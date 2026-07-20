// area: bytes
// spec: bytes.md §6, law 8; ADR-0048; bytes.md §3.1
// status: PASS
// Index-order iteration through the inherited Iterable protocol (for and
// each), empty-buffer termination, fromList/toList inverses.

let sum = 0
for (x in Bytes.fromList([1, 2, 3])) { sum = sum + x }
System.print(sum)
Bytes.fromList([10, 20]).each { x => System.print(x) }
let empty_visits = 0
Bytes.new(0).each { x => empty_visits = empty_visits + 1 }
System.print(empty_visits)
const l = Bytes.fromList([4, 5]).toList
System.print(l.size)
System.print(l.at(1))
System.print(Bytes.fromList(l) == Bytes.fromList([4, 5]))
