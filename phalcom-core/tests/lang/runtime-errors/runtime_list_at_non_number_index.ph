// area: runtime-errors
// spec: U-LIST-plan.md §3/§7; ADR-0020
// status: NEGATIVE
// A non-Int index to `List::at(_:)` is a hard type error, never a silent
// coercion or panic.

const l = List.new()
l.add(1)
System.print(l.at("x"))
