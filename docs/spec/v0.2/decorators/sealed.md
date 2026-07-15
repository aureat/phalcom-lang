# `@sealed` + `@variant` — sealed hierarchies and variant arms

- Status: **Implemented**
- Unit: U-ANNOT-LAYOUT
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `SealedExpander` (L564-578)
  and `VariantExpander` (L591-605), registered at L648-649; `expand_variants`
  (L1265-1371), called from `expand_class_attributes` L1599. `@sealed`'s own
  bookkeeping (recording the class in `VM::sealed_classes`) runs in
  `phalcom-core/src/compiler/lib/class_decl.rs` L751-754, and the cross-unit
  subclass rejection at L357-364. Fixtures:
  `tests/lang/compile-errors/annotation_variant_requires_sealed.ph`,
  `tests/lang/errors/annotation_variant_visitor_exhaustive.ph`.
- Tier: **Compile / generate**, `runtime: false`.
- Depends on: [README.md](README.md) · [annotations-data.md](../experimental/annotations-data.md)
- Related: **[drafts/sealed-classes.md](../drafts/sealed-classes.md) — the full
  mechanism, exhaustiveness story, and design space. Read that first; this file is
  the as-built decorator surface only.** · [data.md](data.md)

One file for both: **`@variant` requires `@sealed`**, and a `@variant` arm with no
enclosing `@sealed` is a compile error (L1277-1282).

## What it does

```phalcom
@sealed class Shape {
  @variant Circle(r)
  @variant Rect(w, h)
}
// -> sibling top-level `class Circle extends Shape` (carrying @data), likewise Rect
// -> Shape#match(circle:, rect:)  — the generated visitor
```

Both are `Class`-target (`@sealed`, L566-568) / `Variant`-target (`@variant`,
L593-595); both expanders' own `expand` is a deliberate no-op, since the real work
needs the whole class.

As built:

- **`@variant` arms become sibling top-level classes**, not members —
  `expand_variants` returns `Vec<Statement>` and `expand_class_attributes`'s return
  type widened to `(ClassDef, Vec<Statement>)` (DEC-ANNOT-G) to carry them. The
  caller compiles each sibling immediately after the enclosing class. Each generated
  variant `extends` the sealed class and carries `@data` (L1332), so it gets the
  record protocol for free.
- **Exhaustiveness is free, not checked.** The generated `match(k1:, k2:, ...)` takes
  one keyword per variant, in declaration order, and double-dispatches to each
  variant's own `__matchArm` override (L1301-1368). A call site that omits or
  misnames an arm is an ordinary **missing-keyword dispatch miss** — no new
  diagnostic. Variant names lower-case their first letter to form the label
  (`Circle` → `circle:`, `lower_first`, L1231).
- **`@sealed` is enforced across compilation units** via `VM::sealed_classes`,
  checked at the *subclass's* compile time (class_decl.rs L357-364).

## Not built

- **A dedicated `@variant`-without-`@sealed` diagnostic.** The error is raised as
  **`attr.illegal_target`** with a bespoke message (L1278-1281), reusing the legality
  code rather than a purpose-named one. Fixture:
  `annotation_variant_requires_sealed.ph`.
- Everything [drafts/sealed-classes.md](../drafts/sealed-classes.md) marks as draft.
