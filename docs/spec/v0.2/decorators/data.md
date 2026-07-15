# `@data` — derive the record protocol

- Status: **Implemented**
- Unit: U-ANNOT-LAYOUT
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `DataExpander` (L539-553),
  registered at L647; `derive_data` (L1111-1214); builders `build_data_eq` (L920),
  `build_data_hash` (L947), `build_data_to_string` (L974), `build_data_with`
  (L1006). Fixtures: `tests/lang/errors/annotation_data_derive_full.ph`,
  `annotation_data_with_shallow_copy.ph`, `annotation_data_unaffected_inferred_class.ph`,
  `tests/lang/compile-errors/annotation_data_eq_hash_collision.ph`.
- Tier: **Compile / generate**, `runtime: false`.
- Depends on: [README.md](README.md) · [annotations-data.md](../experimental/annotations-data.md)
- Related: [construct.md](construct.md) (reused unchanged) · [sealed.md](sealed.md)
  (every `@variant` arm is generated carrying `@data`) ·
  [equality-and-hash.md](../experimental/equality-and-hash.md)

## What it does

```phalcom
@data class Money { var _cents; var _currency }
// derives, in order: construct new(cents:, currency:)
//                    cents / currency  (getter backfill)
//                    ==(other) · hash · toString · with(cents:, currency:)
```

Legal target is **`Class` only** (L541-543); the derive runs from
`expand_class_attributes` (L1557-1559).

As built:

- **Constructor**: reuses [`derive_construct`](construct.md) verbatim, and is
  **skipped entirely** if a `Construct` named `new` is already present — hand-written
  or already derived by a `@construct` earlier in the same pass (L1121-1124). Same
  own-fields-only limitation applies.
- **`==`/`hash` are derived together or not at all** (L1130-1136). Hand-writing
  exactly one is `attr.accessor_collision`; hand-writing **both** is a silent no-op
  (hand-written wins); hand-writing **neither** derives both. This enforces
  `a == b ⇒ a.hash == b.hash`.
- **Getter backfill**: deriving `==` also backfills a plain getter for any field
  lacking one (L1138-1151) — `build_data_eq` reads `other.<name>` through a real
  getter send, since `Expr::Field` is always implicit-`self` and there is no other
  way to read a sibling instance's field. A hand-written accessor is reused as-is,
  never a collision.
- **`toString` and `with(...)`** are independently no-op-if-already-present (silent
  skip, no error). `with(...)` is **omitted entirely** when there are no
  non-defaulted fields (L1189) — a zero-parameter `with()` carries no useful
  functional-update surface.
- **`with(...)` is shallow** by construction: every field value is copied by
  reference, never cloned. Fixture: `annotation_data_with_shallow_copy.ph`.

## Divergences from `annotations-data.md`'s pseudocode

All three are the Rubric's own "build the equivalent manually" fallback, taken
because the pseudocode's primitives do not exist on HEAD:

| Pseudocode | As built | Why |
|---|---|---|
| `hash => _a.hash.combine(_b.hash)` | left-folded `acc * 31 + field.hash` (L947) | no `combine` primitive or `core.ph` method exists |
| `toString => "Money(\(_cents))"` | hand-built `+`-chain of `String.new(field)` segments (L974) | interpolation desugars at *parse* time; there is no `Expr::StringInterp` node left to synthesize |
| `with`: `<param>.orElse { self.<field> }` | `(param == None).ifTrue({ self.<field> }, ifFalse: { param })` (L1006-1047) | `orElse` only works if `param` is already an `Option`, but `with(...)`'s whole point is accepting a **raw** replacement value (`money.with(cents: 700)`, not `Some(700)`) |

Also: **every `with(...)` label is required at the call site**. Keyword-argument
omission is an ordinary different-selector dispatch miss under
[ADR-0012](../../../adr/accepted/0012-selector-signature-encoding-and-dispatch.md)'s
selector-identity model — there is no partial-application sugar. A caller passes
`None` for every field left unchanged.

A field-less `@data` class derives a vacuous `true` for `==` and a constant `0` for
`hash`.

## Not built

- **Superclass field chaining** — inherited from `derive_construct`'s
  own-fields-only shape; see [construct.md](construct.md).
- **`@data` on an inferred class** — no effect; fixture
  `annotation_data_unaffected_inferred_class.ph` pins this.
