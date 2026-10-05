// area: system
// spec: system.md; values-and-absence.md §3.2; U-COLLTYPES plan.md
// status: PASS
// `System.print` on every collection kind renders via that value's OWN
// `toString`, no wrapper — `List` brackets, the `Map {k: v}` map literal's
// `#sym: val` rendering, `Set(...)`, tuple parens, and `Range`'s `a..b`
// syntax-literal-shaped render, plus `None`/`Some(_)` for good measure.

let l = []
l.append(1)
l.append(2)
System.print(l)

let m = Map {a: 1, b: 2}
System.print(m)

let s = #{1, 2}
System.print(s)

let t = (3, 4)
System.print(t)

let r = 1..=3
System.print(r)

System.print(None)
System.print(Some(5))
