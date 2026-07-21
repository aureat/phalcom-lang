# `@get` / `@set` — derive field accessors

- Status: **Implemented**
- Unit: U-ANNOT-LAYOUT
- Evidence: `phalcom-core/src/compiler/attributes.rs` — `GetExpander` (L498-512),
  `SetExpander` (L515-529), registered at L645-646; `derive_accessors` (L859-905),
  called from `expand_class_attributes` L1593; `check_accessor_collision` (L821).
  Fixture: `phalcom-core/tests/lang/classes/class_attribute_construct_get_set.ph`.
- Tier: **Compile / generate**, `runtime: false`.
- Depends on: [README.md](README.md) · [annotations-construct.md](../experimental/annotations-construct.md)
- Related: [construct.md](construct.md) · [data.md](data.md) (backfills getters)

One file for both: they are a pair, share one derive function, and differ only in
which member kind they emit.

## What it does

```phalcom
@get var _name          // -> name => _name
@set var _name          // -> name=(value) { _name = value }
```

Both may be stacked on one field to get a read/write pair. The accessor's base name
is the field name with one leading underscore stripped (`_name` → `name`).

As built:

- Legal target is **`Field` only** for both (L500-502, L517-519). Anywhere else is
  `attr.illegal_target`.
- Both expanders' own `expand` is a deliberate no-op: the derive appends a
  **sibling** member next to the field, which `AttributeExpander::expand`'s
  mutate-in-place signature cannot do. The real work is `derive_accessors`, which
  runs **before** the member-level attribute loop consumes each field's attributes
  (L1591-1593).
- The setter's parameter is always literally named `value` (L889).
- **Collision**: `attr.accessor_collision` if the class already hand-writes a
  member of the same derived selector (getter for `@get`, setter for `@set`).
  Selector-keyed via `check_selector_collision` (L807) — `name` (getter) and
  `name=` (setter) are distinct selectors, so a hand-written getter does not
  collide with a derived setter.

## Not built

- **`@get(priv)`** — the bare argument is **parsed but never inspected**; nothing
  gates on it (L857-858). It is advisory naming only, so `@get(priv)` today derives
  exactly the same public getter as bare `@get`. **Divergence: the documented
  visibility argument has no effect.**
- **Static accessors** — every derived accessor is `is_static: false` (L881, L897).
