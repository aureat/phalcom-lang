// area: control-flow
// spec: catalog-delta.md §4.2; ADR-0018 (sacred selector inliner) amendment;
//   invariant-requirements.md R-INV-2.3
// status: PASS
// U-CORE-2: R-INV-2.3. The Some-lift is one-armed only: the paired
// conditional `ifTrue(_, ifFalse:)` and the lazy `and`/`or` sends return
// their block result RAW, never Some-lifted. A `Number` message is sent to
// each result — if it were `Some`-wrapped, `Some` would `dnu '+(_)'` instead
// of arithmetic succeeding. Also exercises the inliner's paired/`and`/`or`
// compile paths (`compiler/inliner.rs` `compile_if_true_if_false`,
// `compile_and`, `compile_or`), which emit no `WrapSome`, matching the
// `bool_if_true_if_false`/`bool_and`/`bool_or` primitives.

System.print((3 > 2).ifTrue(|| { 10 }, ifFalse: || { 20 }) + 1)
System.print((2 > 3).ifTrue(|| { 10 }, ifFalse: || { 20 }) + 1)
System.print((true and 5) + 1)
System.print((false or 7) + 1)
System.print(true and 5)
System.print(false or 7)
