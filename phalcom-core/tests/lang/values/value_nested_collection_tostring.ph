// area: values
// spec: values-and-absence.md; U-CORE-4; U-LIST `toString` (list.rs)
// status: PASS
// Adversarial: `List#toString` renders each element via that element's OWN
// `toString`, recursively — a `List` element renders as `[..]`, so a list of
// lists nests correctly. A `Some` wrapping a `List` composes the same way:
// `Option#toString`'s `"Some(" + v.toString + ")"` picks up the list's own
// bracketed rendering for its inner value.

const inner = []
inner.append(1)
inner.append(2)
const outer = []
outer.append(inner)
outer.append(3)
System.print(outer.toString)
System.print(Some.new(inner).toString)
