// area: absence
// spec: values-and-absence.md; ADR-0007; object-model.md
// status: PASS
// Ported from Wren `test/core/null/type.wren`: `null is Null`/`null is
// Object`/`null is Bool` become `isA(_)` sends against the `None` singleton
// (Phalcom's absence value, ADR-0007 — there is no bare `nil` surface, so
// `Null` becomes `None`'s own class); `.type` becomes `.class`.

System.print(None.isA(Object))
System.print(None.isA(Bool))
System.print(None.class == None.class)
