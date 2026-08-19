// area: classes
// spec: next/is-tests.md
// status: PASS
// U-IS: the `not` negation particle. `is not`/`is! not` are compile-time
// `.not` wraps over the base `is`/`is!` send (no `isNot` selector),
// consuming `not` greedily right after `is`/`is!` (Python's `is not` rule) —
// never a prefix on the RHS.

System.print(3 is not Number ) // false
System.print(3 is! not Number) // true
System.print(3 is not Int ) // false
System.print(3 is! not Int) // false

System.print(3.is(Number).not ) // false
System.print(3.is!(Number).not) // true
System.print(3.is(Int).not ) // false
System.print(3.is!(Int).not) // false

System.print(3 is not "str" ) // true
System.print(3 is! not "str") // true
System.print(3 is not 4 ) // true
System.print(3 is! not 4) // true

System.print(3.is("str").not ) // true
System.print(3.is!("str").not) // true
System.print(3.is(4).not ) // true
System.print(3.is!(4).not) // true
