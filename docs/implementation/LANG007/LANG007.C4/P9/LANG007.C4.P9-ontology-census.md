# LANG007.C4.P9 — Public Ontology Census

**Audit date:** 2026-10-14  
**Scope:** Consolidated specifications, shipped P1–P8 semantics, public reflection vs internal compiler domains.  
**Authority:** `docs/specs/README.md`; `docs/specs/structural-products.md`; `docs/specs/type-level-domains.md`.

---

## 1. Classification Categories

- **A — Canonical public structural API:** Primary normative types and members authored in user Telos code.
- **B — Canonical public kind/binder domain:** Static/type-level kinded domains that parameterize generic declarations.
- **C — Compatibility facade:** Deprecated or legacy aliases maintained for backward compatibility.
- **D — Obsolete public API:** Stale reflection surfaces slated for retirement in P9.
- **E — Tooling/test regression guard:** Intentional regression tests for surface separation and parser rules.
- **F — Internal implementation:** Semantic solver state, AST node representations, runtime descriptors.
- **G — Historical/superseded documentation:** Prior documentation notes preserving historical evolution.
- **H — Unrelated coincidental spelling:** Independent symbols sharing a naming substring.

---

## 2. Census and Disposition Table

| Symbol / Term | Canonical Path | Category | Role & Meaning | P9 Action |
|---|---|---|---|---|
| `TypePack` | `collections/tuple.ph`, `specs/type-level-domains.md` | B | Canonical C3 kinded positional sequence of proper types | Keep |
| `LabeledTypePack` | `collections/tuple.ph`, `specs/type-level-domains.md` | B | Canonical C3 kinded ordered `(Symbol, Type)` lane | Keep |
| `CompleteTypePack` | `collections/pack.ph`, `specs/type-level-domains.md` | B | Canonical C3 complete two-lane sequence anchor | Keep |
| `RecordFields` | `collections/record.ph`, `specs/type-level-domains.md` | B | Contextual kinded binder domain for `Record<**Fs>` | Keep binder kind; retire reflection member |
| `Pack` | `collections/pack.ph`, `specs/structural-products.md` | A | First-class ordinary Pack type with proper `TypeId` | Keep |
| `PositionalPack<T>` | `collections/pack.ph`, `specs/type-level-domains.md` | A | Proper positional Pack specialization `Pack<*: T>` | Keep |
| `LabeledPack<T>` | `collections/pack.ph`, `specs/type-level-domains.md` | A | Proper labeled Pack specialization `Pack<**: T>` | Keep |
| `CompletePack<T>` | `collections/pack.ph`, `specs/type-level-domains.md` | A | Proper complete Pack specialization `Pack<***: T>` | Keep |
| `PositionalPackView<T>` | `collections/pack.ph` | A | One-step positional Pack decomposition view | Keep |
| `LabeledPackView<T>` | `collections/pack.ph` | A | One-step labeled Pack decomposition view | Keep |
| `PositionalTypePackView` | `collections/pack.ph` | C | Dynamic-valued alias of `PositionalPackView<Dynamic>` | Keep as compatibility facade |
| `LabeledTypePackView` | `collections/pack.ph` | C | Dynamic-valued alias of `LabeledPackView<Dynamic>` | Keep as compatibility facade |
| `UnorderedProduct` | `reflection/structural.ph`, `specs/structural-products.md` | A | Public trait for unordered product types | Keep |
| `UnorderedShape` | `reflection/structural.ph`, `specs/structural-products.md` | A | Normative reflection data for unordered product shapes | Keep |
| `FieldKey` | `collections/record.ph`, `specs/structural-products.md` | A | Canonical heterogeneous key for Record fields (Symbol, String, Int, Bool) | Keep |
| `FieldKeyRef` | `phalcom-type-meta/src/field_key.rs` | A | Stable wire representation of FieldKey in schema v13 | Keep |
| `FieldInfo` | `reflection/structural.ph`, `specs/structural-products.md` | A | Component type and Presence metadata | Keep |
| `Presence` | `reflection/structural.ph`, `specs/structural-products.md` | A | Required vs Optional structural presence | Keep |
| `ProductField` | `reflection/structural.ph`, `specs/structural-products.md` | A | FieldKey + FieldInfo entry in UnorderedShape | Keep |
| `UnorderedTail` | `reflection/structural.ph`, `specs/structural-products.md` | A | Closed, Typed, Unconstrained, Captured tail | Keep |
| `FieldLookup` | `reflection/structural.ph`, `specs/structural-products.md` | A | Required, Optional, Absent, Unknown lookup outcome | Keep |
| `UnorderedShapeView` | `reflection/structural.ph`, `specs/structural-products.md` | A | One-step decomposition of UnorderedShape | Keep |
| `Product` | `reflection/structural.ph`, `specs/structural-products.md` | A | Base reflection trait for all product forms | Keep |
| `OrderedProduct` | `reflection/structural.ph`, `specs/structural-products.md` | A | Reflection trait for Tuple and Pack | Keep |
| `OrderedShape` | `reflection/structural.ph`, `specs/structural-products.md` | A | Reflected shape of ordered products | Keep |
| `Sum` | `reflection/structural.ph`, `specs/structural-sums.md` | A | Reflection trait for Sum / Enum types | Keep |
| `Variant` | `reflection/structural.ph`, `specs/structural-sums.md` | A | Individual reflected variant with Symbol name and shape | Keep |
| `Variants` | `reflection/structural.ph`, `specs/structural-sums.md` | A | PositionalPack of Variant entries | Keep |
| `VariantNames` | `reflection/structural.ph`, `specs/structural-sums.md` | A | PositionalPack of Symbol variant names | Keep |
| `Record::Fields` | `collections/record.ph` | D | Obsolete public type-member facade | Retire public facade; use `UnorderedShape` |
| `RecordFields::Map` | `collections/record.ph` | D | Obsolete public type-member mapping facade | Retire public facade; keep internal `RecordRowMap` |
| `RecordRow` / `RecordRowId` | `phalcom-semantic/src/types/row.rs` | F | Internal solver row representation | Keep internal |
| `RecordRowMap` | `phalcom-semantic/src/types/type_member.rs` | F | Internal solver operation | Keep internal |
| `let`, `let mut`, `const` | Language syntax | A | Canonical binding and phase model | Keep |
