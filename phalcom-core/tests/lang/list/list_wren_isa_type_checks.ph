// area: list
// spec: U-CORE-1 as-built.md (`isA(_)`); U-LIST-plan.md §7
// status: PASS
// Ported from Wren `test/core/list/type.wren`, adapted to Phalcom's `isA(_)`
// reflection selector (no `is`/`Sequence` keyword surface yet — `List` does
// not extend a shared `Iterable` root in this build, so the `Sequence`
// membership check has no Phalcom analog and is dropped here).

const l = List.new()
System.print(l.isA(List))
System.print(l.isA(Object))
System.print(l.isA(Bool))
