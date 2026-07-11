// area: list
// spec: catalog-delta.md §2.4; ADR-0020; U-STD §2.6
// status: PASS
// U-STD: `includes(_)` is `true` iff some element is `== x`; `isEmpty` is
// `self.size == 0`. Both conditions are well-formed `Bool`s (ADR-0021).

let l = List.new()
l.add(1)
l.add(2)
l.add(3)
System.print(l.includes(2))
System.print(l.includes(9))
System.print(List.new().isEmpty)
System.print(l.isEmpty)
