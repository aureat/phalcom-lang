// area: functions
// spec: functions.md §1-2; ADR-0006
// status: NEGATIVE
// Ported from Wren `test/core/function/call_missing_arguments.wren`: Wren
// pads missing call arguments with `null`. Phalcom's `Block#call` is
// strict — too few arguments also raises `RuntimeError::Arity`, matching
// the extra-arguments case (`functions_call_extra_arguments.ph`).

const f2 = |a, b| { System.print(a + b) }
f2.call("a")
