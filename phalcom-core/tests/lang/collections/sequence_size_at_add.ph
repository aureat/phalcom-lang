// area: collections
// spec: U-CORE-5 as-built.md §2.1, §3.3(b); R-INV-5.1 (L1,L2,L3,L5)
// status: PASS
// The shared sequence-protocol contract, exercised against `List` as its
// reference implementation: `size`/`at(_:)`/`add(_:)` round-trip and `add`
// returns `self` for chaining.

const l = List.new()
System.print(l.size)
l.add(1).add(2).add(3)
System.print(l.size)
System.print(l.at(0))
System.print(l.at(1))
System.print(l.at(2))
