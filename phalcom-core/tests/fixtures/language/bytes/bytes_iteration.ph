// area: bytes
// spec: bytes.md §6, law 8; ADR-0048; bytes.md §3.1
// status: PASS
// Index-order iteration through the inherited Iterable protocol (for and
// each), empty-buffer termination, fromList/toList inverses.

let mut sum = 0
let iterationValues: List<Int> = [1, 2, 3]
for x in Bytes.fromList(iterationValues) { sum = sum + x }
System.print(sum)
let eachValues: List<Int> = [10, 20]
Bytes.fromList(eachValues).each |x| { System.print(x) }
let mut empty_visits = 0
Bytes.new(0).each |x| { empty_visits = empty_visits + 1 }
System.print(empty_visits)
let roundTripValues: List<Int> = [4, 5]
let l = Bytes.fromList(roundTripValues).toList
System.print(l.size)
System.print(l.at(1))
System.print(Bytes.fromList(l) == Bytes.fromList(roundTripValues))
