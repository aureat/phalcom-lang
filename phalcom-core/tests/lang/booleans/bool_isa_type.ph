// area: booleans
// spec: object-model.md; values-and-absence.md
// status: PASS
// Ported from Wren `test/core/bool/type.wren`: `true is Bool`/`true is
// Object`/`true is Num` become `isA(_)` sends (Phalcom has no `is` surface
// operator yet — is-tests.md is an unratified proposal); `.type` becomes
// `.class`.

System.print(true.isA(Bool))
System.print(true.isA(Object))
System.print(true.isA(Number))
System.print(true.class == Bool)
