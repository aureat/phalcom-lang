// area: list
// spec: U-LIST-plan.md §7; ADR-0020
// status: PASS
// `toString` renders as `"[e1, e2, e3]"`. A native primitive this unit (not
// `.ph`-defined over `each`) — see the U-LIST return contract for why.

let l = List.new()
l.add(1)
l.add(2)
l.add(3)
System.print(l.toString)
