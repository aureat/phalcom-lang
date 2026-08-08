// area: list
// spec: catalog-delta.md §2.4; ADR-0020; U-STD §2.6
// status: PASS
// U-STD: `includes(_)` is `true` iff some element is `== x`; `isEmpty` is
// `self.size == 0`. Both conditions are well-formed `Bool`s (ADR-0021).

const l = []
l.append(1)
l.append(2)
l.append(3)
System.print(l.includes(2))
System.print(l.includes(9))
System.print([].isEmpty)
System.print(l.isEmpty)
