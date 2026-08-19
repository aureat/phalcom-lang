// area: functions
// spec: object-model.md; ADR-0006
// status: PASS
// Ported from Wren `test/core/function/type.wren`: `Fn.new || { 0 } is Fn`
// becomes `isA(Closure)` — Phalcom's concrete callable is `Closure`, with the
// abstract `Function` as its root (ADR-0006); `.type` becomes `.class`.

System.print(|| { 0 }.is(Closure))
System.print(|| { 0 }.is(Object))
System.print(|| { 0 }.is(String))
System.print(|| { 0 }.class == Closure)
