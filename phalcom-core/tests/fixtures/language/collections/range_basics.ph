// area: collections
// spec: tuple-and-range.md §2; U-COLLTYPES plan.md Phase 3; ADR-0039
// status: PASS
// Range surface protocol: inclusive/exclusive bound parity, size/first/last/
// includes, each/toList, structural == over independently-built ranges.

let r = Range.new(1, 5, true)
System.print(r.size)
System.print(r.first)
System.print(r.last)
System.print(r.includes(5))
System.print(r.includes(6))
System.print(r.toList)

let e = Range.new(1, 5, false)
System.print(e.size)
System.print(e.last)
System.print(e.includes(5))

let r2 = Range.new(1, 5, true)
System.print(r == r2)
System.print(r.hash == r2.hash)
