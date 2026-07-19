// area: classes
// spec: next/is-tests.md
// status: NEGATIVE
// `is` is non-chaining: its result is a `Bool`, so `a is B is C` (a second
// `is` immediately following the first) is a compile-time parse error, not
// a silently-accepted `(a is B) is C`. Parenthesize to force it.

let a = 3
class B {}
class C {}
System.print(a is B is C)
