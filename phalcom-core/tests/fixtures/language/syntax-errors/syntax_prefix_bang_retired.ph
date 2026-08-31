// area: errors
// spec: next/is-tests.md
// status: NEGATIVE
// U-NEG retires prefix `!` as an expression operator; `not x` is the sole
// boolean-negation prefix. A bare `!` in expression position is a parse
// error (only `!=` remains a valid `!`-containing token).

System.print(!true)
