// area: string
// spec: object-model.md; selectors.md
// status: PASS
// Wren precedent: test/core/string/type.wren. Wren's `is` type-test operator
// has no Phalcom equivalent (selectors.md; reflection/reflection_isa_respondsto_representative.ph
// already establishes the pattern) — ported onto `isA(_)` (walks the
// superclass chain) and `.class` (the receiver's own defining class) instead
// of `is`/`.type`.
System.print("s".is(String))
System.print("s".is(Object))
System.print("s".is(Number))
System.print("s".class == String)
