// area: runtime-errors
// spec: ADR-0021 (no truthiness enforcement); U-LIST-plan.md §7
// status: NEGATIVE
// Ported from Wren `test/core/list/not.wren`: Wren's `!` is truthiness-based
// and `![1, 2]`/`![]` both evaluate to `false` (every list is truthy). Under
// ADR-0021 Phalcom's `not` (`Bool#not`) is defined ONLY on real `Bool`
// receivers — there is no generic truthiness coercion — so applying `not` to
// a `List` is a hard does-not-understand, never a silent `false`. (U-NEG:
// prefix `!` retired; `not` is the sole prefix-negation surface.)

const l = []
System.print(not l)
