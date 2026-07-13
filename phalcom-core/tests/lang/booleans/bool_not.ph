// area: booleans
// spec: control-flow.md; values-and-absence.md; next/is-tests.md
// status: PASS
// Ported from Wren `test/core/bool/not.wren`: `Bool#not()` (surface `not`,
// U-NEG — prefix `!` retired, `not` is the sole prefix-negation surface).

System.print(not true)
System.print(not false)
System.print(not not true)
