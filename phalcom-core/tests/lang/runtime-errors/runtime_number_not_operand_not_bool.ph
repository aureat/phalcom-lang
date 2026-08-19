// area: errors
// spec: control-flow.md; messages-and-selectors.md
// status: NEGATIVE
// ported from wren/test/core/number/not.wren: Wren's `!` coerces any
// receiver to a boolean and negates it, but Phalcom's `not` is a Bool-only
// send (mirrors `and`/`or`, see runtime_and_non_boolean_operand.ph) — a
// Number receiver has no `not` getter. (U-NEG: prefix `!` retired; `not`
// is the sole prefix-negation surface.)

System.print(not 123)
