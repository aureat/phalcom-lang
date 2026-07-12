// area: system
// spec: system.md; values-and-absence.md §3.2; U-COLLTYPES plan.md
// status: PASS
// `System.print` on every collection kind renders via that value's OWN
// `toString`, no wrapper — `List` brackets, the `{k: v}` map literal's
// `#sym: val` rendering, `Set(...)`, tuple parens, and `Range`'s `a..b`
// syntax-literal-shaped render, plus `None`/`Some(_)` for good measure.

let l = List.new()
l.add(1)
l.add(2)
System.print(l)

let m = {a: 1, b: 2}
System.print(m)

let s = Set.new()
s.add(1)
s.add(2)
System.print(s)

let t = Tuple.fromList(List.new().add(3).add(4))
System.print(t)

let r = Range.new(1, 3, true)
System.print(r)

System.print(None)
System.print(Some.new(5))
