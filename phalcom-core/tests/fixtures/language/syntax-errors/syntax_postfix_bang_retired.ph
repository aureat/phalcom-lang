// area: errors
// spec: next/is-tests.md
// status: NEGATIVE
// A trailing bare `!` after an expression is not a postfix operator either
// (prefix `!` is fully retired, not merely reordered) — `!` only survives as
// the first half of `!=`.

System.print(true !)
