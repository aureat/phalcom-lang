// area: absence
// spec: ADR-0018 (sacred selector inliner) amendment "Allocation elision";
//   invariant-requirements.md R-INV-2.2
// status: PASS
// U-CORE-2: R-INV-2.2. In statement position, the compiler's `want_value`
// thread (`compiler/lib.rs` `compile_statement_with_pop_control` ->
// `compile_expr_want` -> `compile_sacred_call_want`) elides the `WrapSome`
// emission for a discarded `ifTrue`/`ifFalse` result. This pins that the
// elision is purely an allocation optimization: the taken arm's body still
// runs exactly once, the untaken arm prints nothing and never errors, and —
// over the identical body — the value-position twin (where `WrapSome` is
// NOT elided) fires the same side effect once and still yields an
// observable `Some` (`isSome == true`).

// statement position -> WrapSome elided; body must still run exactly once
true.ifTrue || { System.print("taken") }
false.ifTrue || { System.print("skip")  }
true.ifFalse || { System.print("skip")  }
false.ifFalse{ System.print("takenF") }
// value position (WrapSome present) over the SAME shape of body: side
// effect fires once, Some still observable
System.print(true.ifTrue || { System.print("effect"); 1 }.isSome)
