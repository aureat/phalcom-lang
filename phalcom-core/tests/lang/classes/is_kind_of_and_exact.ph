// area: classes
// spec: next/is-tests.md
// status: PASS
// U-IS: the base `is`/`is!` surface — kind-of vs exact-class. Numeric
// literals are direct `Number` instances (no `Int` subclass over the floor),
// so `3`'s kind-of and exact tests against `Number` agree — see is-tests.md's
// "worked-example discrepancy" note. `3 is "str"` / `3 is 4` pin I-4 (a
// non-class RHS returns `false`, never raises).

System.print(3 is Number)
System.print(3 is! Number)
System.print(3 is "str")
System.print(3 is 4)
