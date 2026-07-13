// area: runtime-errors
// spec: ADR-0021 (no truthiness enforcement)
// status: NEGATIVE
// Ported from Wren `test/core/object/not.wren`: Wren's `!` is truthiness-
// based and `!Foo.new()` evaluates to `false` (every object is truthy).
// Under ADR-0021 Phalcom's `not` (`Bool#not()`) is defined ONLY on real
// `Bool` receivers — there is no generic truthiness coercion — so applying
// `not` to a plain instance is a hard does-not-understand, never a silent
// `false`. Companion to the already-landed `runtime_list_not_operator_dnu.ph`
// (a `List` receiver) and `absence/negative/absence_none_not_operator_dnu.ph`
// (a `None` receiver) — this one exercises a plain user-class instance.
// (U-NEG: prefix `!` retired; `not` is the sole prefix-negation surface.)

class Foo {
  construct new() {}
}
System.print(not Foo.new())
