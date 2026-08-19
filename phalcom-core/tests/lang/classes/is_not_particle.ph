// area: classes
// spec: next/is-tests.md
// status: PASS
// U-IS: the `not` negation particle. `is not`/`is! not` are compile-time
// `.not` wraps over the base `is`/`is!` send (no `isNot` selector),
// consuming `not` greedily right after `is`/`is!` (Python's `is not` rule) —
// never a prefix on the RHS.

System.print(3 is not Number)
System.print(3 is! not Number)
