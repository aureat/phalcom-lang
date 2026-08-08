// area: functions
// spec: object-model.md; ADR-0006
// status: PASS
// Ported from Wren `test/core/function/type.wren`: `Fn.new || { 0 } is Fn`
// becomes `isA(Block)` — Phalcom's concrete callable is `Block`, with the
// abstract `Function` as its root (ADR-0006); `.type` becomes `.class`.

System.print(|| { 0 }.isA(Block))
System.print(|| { 0 }.isA(Object))
System.print(|| { 0 }.isA(String))
System.print(|| { 0 }.class == Block)
