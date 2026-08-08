// area: functions
// spec: values-and-absence.md; ADR-0006
// status: PASS
// Ported from Wren `test/core/function/to_string.wren`: Wren renders a
// `Fn` as `<fn>`; Phalcom's `Block` renders as `<Block>` (values.md's
// per-type `toString` catalog).

System.print(|| {})
