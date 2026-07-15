# `@construct` — derive a constructor from declared fields

- Status: **Implemented**
- Unit: U-ANNOT-LAYOUT
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `ConstructExpander`
  (L475-489), registered at L644; `derive_construct` (L715-772);
  `strip_leading_underscore` (L663). Fixtures:
  `phalcom-core/tests/lang/classes/class_attribute_construct_get_set.ph`,
  `tests/lang/errors/annotation_construct_own_fields.ph`.
- Tier: spec says **Layout**; as built it is an ordinary **Compile / generate**
  derive (there is no Layout tier — see [README.md](README.md)).
- Depends on: [README.md](README.md) · [annotations-construct.md](../experimental/annotations-construct.md)
- Related: [accessors.md](accessors.md) · [data.md](data.md) (reuses this derive)

## What it does

`@construct` on a class derives `construct new(...)` binding its declared fields,
in declaration order.

```phalcom
@construct class Point { var _x; var _y }
// -> construct new(x:, y:) { _x = x; _y = y }
```

Each field's parameter name and label is the field name with one leading
underscore stripped (`_x` → `x`).

As built:

- Legal target is **`Class` only** (L477-479). `ConstructExpander::expand` is a
  deliberate no-op — the derive needs the whole class's fields, so it runs from
  `expand_class_attributes` (L1555-1557).
- Emits a real `ConstructDef`/`ClassMember::Construct` (L764) — **not** a
  `MethodDef` named `new` (which would silently be a non-constructor). It is then
  compiled through the same path a hand-written `construct` takes, so it gets the
  same selector encoding, `constructor_aliases` registration, and
  `has_new_construct` bare-allocator-guard interaction.
- **Defaulted fields are omitted from the parameter list** (L725). Every defaulted
  field's default is assigned **first**, in declaration order, before the labeled
  parameter assignments (L753-762) — so a later field's default observes prior
  defaults already applied.
- **Collision**: `attr.accessor_collision` if the class already hand-writes a
  `construct` of the *exact same derived selector* (L736-740). A
  differently-selectored hand-written constructor (e.g. `construct anonymous()`)
  coexists unaffected — the check is selector-keyed, not "any construct present".

## Not built

- **Superclass chaining.** `derive_construct` is **own-fields-only** (L684-688): a
  `@construct` class that `extends` a superclass with its own constructor gets only
  its own fields' assignments, never a `super.new(...)` call. The inheritance-aware
  fix (`annotations-construct-inheritance.md`'s F-fix) is a separate build-order
  step that has not been done. Fixture pinning the current behavior:
  `tests/lang/errors/annotation_construct_own_fields.ph`.
- **Slot reservation.** Nothing reserves a slot; `@construct`'s "Layout" tier
  classification is aspirational (see [README.md](README.md)).
