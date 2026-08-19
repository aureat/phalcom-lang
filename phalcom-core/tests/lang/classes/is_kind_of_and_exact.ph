// area: classes
// spec: next/is-tests.md
// status: PASS
// U-IS: the base `is`/`is!` surface — kind-of vs exact-class. Numeric
// literals are `Int` instances, so `3` is a kind of abstract `Number` but is
// not exactly `Number`. `3 is "str"` / `3 is 4` pin I-4 (a
// non-class RHS returns `false`, never raises).

System.print(3 is Number) // true
System.print(3 is! Number) // false
System.print(3 is Int) // true
System.print(3 is! Int) // true
System.print(3.is(Number)) // true
System.print(3.is!(Number)) // false
System.print(3.is(Int)) // true
System.print(3.is!(Int)) // true
System.print(3 is "str") // false
System.print(3 is 4) // false
System.print(3.is("str")) // false
System.print(3.is!(4)) // false
