# Annotations — layout tier (`@construct`, field declarations)

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Depends on: [annotations-core.md](annotations-core.md)
- Related: ADR-0011 (fixed slot layout), ADR-0014 (`let`/`var` bindings), classes.md §1–2, selectors.md §1 (R3 label order = identity)

## Context

`@construct` is **not** a body-weave — it generates a whole member from the field
set and therefore touches instance layout (ADR-0011's fixed slot vector). It is
the **layout-derive** tier and has two prerequisites absent from the current tree.

## Decision

`@construct` is gated behind two new pieces of machinery; the derive itself is
trivial once they exist.

### Prerequisite 1 — field-declaration syntax

Fields are today implicit-by-assignment (classes.md §2); there is no
declaration-only form for `@construct` to read. Add:

```rust
ClassMember::Field(FieldDef { name, mutable, default: Option<Expr>, attributes, range })
```

so `var _x` / `let _x` at **class-body position** fixes the slot layout at
class-definition time. The parser must disambiguate `var` at class-body
(field decl) vs statement position (local binding, ADR-0014) — same keyword, two
roles.

### Prerequisite 2 — `construct` semantics

Per classes.md §1 a constructor emits allocation, binds `self`, returns `self`
implicitly. `parse_class_member` (`parser.rs` L548) has **no `construct`
handling** today — only `static`/name/`=`/params/body — so the `construct`
keyword is itself unbuilt. `@construct` sits downstream of it.

### The derive (once prerequisites land)

Read declared fields → emit one constructor member:

```phalcom
@construct
class Point { var _x; var _y }
```
⇒
```rust
MethodDef {
    name: "new",
    params: vec![labeled("x"), labeled("y")],   // ParameterDef.label already exists (ast.rs:37)
    body:   vec![assign("_x", var("x")), assign("_y", var("y"))],
    is_constructor: true,                         // routes through the construct path
    ..
}
```

### `@get`/`@set`

Derive an accessor pair for a field field-member. `@get var _label` ⇒ `label =>
_label`; `@set` ⇒ `label=(value) { _label = value }`. Collision with a
hand-written accessor of the same selector is a **compile error** (ADR-0012:
selector is the sole dispatch key — no last-wins). `@get(priv)` is **advisory
naming only**, never enforcement (selectors.md §5 rejects visibility syntax).

## Hazards

- **Field order is API.** R3 (selectors.md §1) makes label order identity, so
  reordering `var _x; var _y` silently changes `new(x:,y:)`'s calling
  convention. This is the Swift memberwise-init cost; **documented, not routed
  around** — no keyword-reordering exists elsewhere, so it is consistent.
- **Layout growth.** `@construct`/`@get`/`@set` and any future `@observable`
  reserve or read declared slots — this is exactly why they are the layout tier
  and gated here, separate from the method-table-macro contracts.

## What this precludes

Splitting field-declaration into its own grammar does not preclude the implicit-
by-assignment path continuing for non-annotated classes; both coexist. Making
`@get(priv)` advisory precludes ever giving it enforcement without reopening
selectors.md §5's no-visibility-syntax commitment.
