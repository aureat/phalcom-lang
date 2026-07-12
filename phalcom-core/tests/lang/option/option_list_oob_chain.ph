// area: option
// spec: values-and-absence.md §3.3; U-LIST return contract; catalog-delta.md §2.2
// status: PASS
// Adversarial: `List#at(_)` is only "Option over an element" on the MISS
// path — an out-of-range read is the `None` singleton (raw `rawAt` shape),
// which then chains through `map`/`unwrapOr` like any other `Option`. An
// in-range read returns the raw element value directly (NOT `Some`-wrapped),
// so it is printed as-is and does not itself support `.map`/`.isNone`.

let xs = List.new()
xs.add(10)
xs.add(20)
System.print(xs.at(0))
System.print(xs.at(5))
System.print(xs.at(5).map { v => v + 1 }.unwrapOr(-1))
System.print(xs.at(5).isNone)
