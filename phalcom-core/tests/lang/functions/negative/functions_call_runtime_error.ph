// area: functions
// spec: functions.md §1-2; messages-and-selectors.md
// status: NEGATIVE
// Ported from Wren `test/core/function/call_runtime_error.wren`: an error
// raised inside a called block's body propagates out of `call` unchanged —
// here, `+` sent to a `Bool` receiver is a plain does-not-understand.

const f1 = { a, b => a + b }
f1.call(true, false)
