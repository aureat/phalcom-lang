// area: absence
// spec: ADR-0021 (no truthiness enforcement); values-and-absence.md
// status: NEGATIVE
// Ported from Wren `test/core/null/not.wren`: Wren's `!` is truthiness-based
// and `!null` evaluates to `true` (null is falsy). Under ADR-0021 Phalcom's
// `not` (`Bool#not`) is defined ONLY on real `Bool` receivers — there is no
// generic truthiness coercion, and `None` is not an exception — so applying
// `not` to `None` is a hard does-not-understand, never a silent `true`.
// (U-NEG: prefix `!` retired; `not` is the sole prefix-negation surface.)

System.print(not None)
