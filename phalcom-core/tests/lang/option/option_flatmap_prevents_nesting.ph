// area: option
// spec: values-and-absence.md §3.3; PDR-0033
// status: PASS
// `map` adds a layer when its callback returns `Some`; `flatMap` uses that
// callback result directly.

const nested = Some(1).map |x| { Some(x + 1) }
const flat = Some(1).flatMap |x| { Some(x + 1) }
System.print(nested)
System.print(flat)
