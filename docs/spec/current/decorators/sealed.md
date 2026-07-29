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
// -> sibling top-level `class Circle is Shape` (carrying @data), likewise Rect
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
  variant `is` the sealed class and carries `@data` (L1332), so it gets the
  record protocol for free.
- **Exhaustiveness is free, not checked.** The generated `match(k1:, k2:, ...)` takes
  one keyword per variant, in declaration order, and double-dispatches to each
  variant's own `__matchArm` override (L1301-1368). A call site that omits or
  misnames an arm is an ordinary **missing-keyword dispatch miss** — no new
  diagnostic. Variant names lower-case their first letter to form the label
  (`Circle` → `circle:`, `lower_first`, L1231).
- **`@sealed` is enforced across compilation units** via `VM::sealed_classes`,
  checked at the *subclass's* compile time (class_decl.rs L357-364) — **but see
  "Not built": for a *user* class that check is currently unreachable.**
- **The `@variant` gate reads the union of two sources** (2026-07-15, DEFERRED CB-3):
  `sealed_by_attr || sealed_by_table`. Neither is complete on its own — a user's own
  `@sealed class` is not yet in `VM::sealed_classes` while its body expands
  (`class_decl.rs` inserts it *after* the body compiles), and bootstrap-sealed
  `Option`/`Some`/`None` carry no `@sealed` attribute at all. Reading either alone is
  wrong for the other case. Fixture:
  `compile-errors/annotation_variant_in_bootstrap_sealed_class.ph`.

## Not built

- **The headline cross-unit enforcement is unreachable for a *user* class.** This is the
  decorator's advertised purpose, and today it does nothing for user code — verified
  2026-07-15 (DEFERRED CB-3 / [drafts/sealed-classes.md](../drafts/sealed-classes.md) S-2).
  `attr.sealed_violation` cannot fire for a user class on two independent grounds:
  1. **Ordering.** `is` resolves its superclass at *compile* time; `import` binds the
     module at *runtime*. Give an imported module a `System.print` side effect and it never
     runs — the `Unknown superclass` error fires first. An imported class cannot be a
     superclass at all, sealed or not.
  2. **Naming.** `is S.Shape` does not parse (`is` takes a bare identifier, not a
     member access), and [ADR-0045](../../../adr/accepted/0045-module-import-relative-path-whole-module-binding.md)'s
     whole-module binding leaks no globals.

  So module structure already supplies the protection `@sealed` advertises, and the check
  is live only for classes present in every unit's globals at compile time — the
  bootstrap-sealed kernel (`Option`/`Some`/`None`). **`@sealed`'s only live effect on a user
  class today is gating `@variant`.** The decorator is not useless — it is future-proofing
  for the day cross-module class references land — but do not describe it as enforcing
  anything for user code yet. Fixture pinning this:
  `compile-errors/decorators_sealed_cross_unit_needs_isolation.ph`, which **must change** if
  that day comes.
- **A dedicated `@variant`-without-`@sealed` diagnostic.** The error is raised as
  **`attr.illegal_target`** with a bespoke message (L1278-1281), reusing the legality
  code rather than a purpose-named one. Fixture:
  `annotation_variant_requires_sealed.ph`.
- **One property, still two representations.** The union-read above makes the *gate* correct
  without unifying the underlying sources. Collapsing them (bootstrap carries `@sealed` too,
  attributes become the single source) is filed as DEFERRED #35; the blocker is that `None`
  has no `.ph` reopen to carry an annotation, plus a seal-ownership question when both paths
  write the table. See [drafts/sealed-classes.md](../drafts/sealed-classes.md) S-1.
- Everything [drafts/sealed-classes.md](../drafts/sealed-classes.md) marks as draft.
