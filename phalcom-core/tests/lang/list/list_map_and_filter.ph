// area: list
// spec: catalog-delta.md §2.4; ADR-0020; U-STD §2.6
// status: PASS
// U-STD: `map(_)` builds a new List of `f(x)`; `filter(_)` keeps elements for
// which `pred(x)` holds. Rendering the whole List via its native `toString`
// (safe path — element values via `list_to_string`), never the `toString`
// *message* on an individual element (the class-name trap, DEFERRED.md #19).

const l = List.new()
l.add(1)
l.add(2)
l.add(3)
System.print(l.map |x| { x * 2 }.toList.toString)
System.print(l.filter |x| { x > 1 }.toString)
