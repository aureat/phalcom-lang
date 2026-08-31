// area: list
// spec: U-LIST-plan.md §7; ADR-0020
// status: PASS
// `toString` renders as `"[e1, e2, e3]"`. A native primitive this unit (not
// `.ph`-defined over `each`) — see the U-LIST return contract for why.

const l = []
l.append(1)
l.append(2)
l.append(3)
System.print(l.toString)
