// area: collections
// spec: tuple-and-range.md §1; U-COLLTYPES plan.md Phase 2; ADR-0039
// status: PASS
// Boundary: fixed-arity construction at 0/1/2/3 elements, and independent
// value-equality — two separately-built tuples of the same arity/elements
// compare `==` (and `!=` correctly negates), while a same-elements arity
// mismatch is unequal.

const t0 = ()
System.print(t0 == ())

const t1 = (9,)
System.print(t1.size)
System.print(t1.at(0))

const t2a = (1, 2)
const t2b = (1, 2)
System.print(t2a == t2b)
System.print(t2a != t2b)

const t3 = (1, 2, 3)
System.print(t3.size)
System.print(t3 == t2a)
System.print(t3 != t2a)
