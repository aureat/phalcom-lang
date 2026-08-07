// area: reflection
// spec: core/decisions.md Q5; core/core-classes.md; ADR-0023
// status: PASS
// The Q5 hash contract split across collection mutability: `Tuple` is
// immutable ⇒ hashable-by-value, so two independently-built structurally
// equal tuples hash equal. `List` is mutable ⇒ NOT hashable-by-value — it
// inherits `Object#hash` (identity), so two structurally equal-but-distinct
// lists hash UNEQUAL even though `==` reports them equal.

const t1 = Tuple.__fromList(List.new().add(1).add(2))
const t2 = Tuple.__fromList(List.new().add(1).add(2))
System.print(t1 == t2)
System.print(t1.hash == t2.hash)

const l1 = List.new().add(1).add(2)
const l2 = List.new().add(1).add(2)
System.print(l1 == l2)
System.print(l1.hash == l2.hash)
System.print(l1.hash == l1.hash)
