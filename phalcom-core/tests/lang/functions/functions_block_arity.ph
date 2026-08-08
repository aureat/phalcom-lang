// area: functions
// spec: functions.md; blocks.md; ADR-0006
// status: PASS
// Ported from Wren `test/core/function/arity.wren`: `Fn.new { … }.arity`
// becomes a block literal's own `.arity` — Phalcom blocks are first-class
// values already (`{ }`), with no `Fn.new` wrapper needed.

System.print({ 0 }.arity)
System.print(|a| { a }.arity)
System.print(|a, b| { a }.arity)
