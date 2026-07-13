// area: functions
// spec: functions.md §1-2; ADR-0006
// status: NEGATIVE
// Ported from Wren `test/core/function/call_extra_arguments.wren`: Wren
// silently discards extra call arguments beyond a block's declared arity.
// Phalcom's `Block#call` is strict (`primitive/block.rs`'s `block_call`) —
// any arity mismatch, including too many arguments, raises
// `RuntimeError::Arity` rather than truncating.

let f0 = { System.print("zero") }
f0.call("a")
