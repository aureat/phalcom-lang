// area: values
// spec: values-and-absence.md; U-CORE-4; U-LIST `toString` (list.rs)
// status: PASS
// Adversarial: `List#toString` renders each element via that element's OWN
// `toString`, recursively — a `List` element renders as `[..]`, so a list of
// lists nests correctly. A `Some` wrapping a `List` composes the same way:
// `Option#toString`'s `"Some(" + v.toString + ")"` picks up the list's own
// bracketed rendering for its inner value.

let inner = List.new()
inner.add(1)
inner.add(2)
let outer = List.new()
outer.add(inner)
outer.add(3)
System.print(outer.toString)
System.print(Some.new(inner).toString)
