# Annotations — legality, grammar, and the expander interface

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Depends on: [annotations-core.md](annotations-core.md)
- Resolves: core gaps — `Target` undefined, no legality table, unknown-attribute behavior, newline binding, arg grammar
- Related: ADR-0016 (newlines are tokens), ADR-0012 (selector identity)

## Context

annotations-core.md references `Target` in the `AttributeExpander` trait and
describes the parser change in prose, but never pins the grammar, the
attribute-position legality, or unknown-attribute behavior. The registry and
parser cannot be built without these.

## Decision

### Grammar

Attributes are **class-member-only** in Draft 0.1 — no statement- or
expression-level attributes.

```ebnf
class-member  := attribute* [ "static" ] member-decl
attribute     := "@" ident [ "(" attr-args? ")" ] NEWLINE*
attr-args     := attr-arg { "," attr-arg }
attr-arg      := expr | ident            (* bare ident e.g. `priv`; see below *)
member-decl   := field-decl | method | getter | setter   (* field-decl: annotations-construct.md *)
```

- **`attr-arg` is an expression by default.** A bare identifier that is not a
  legal standalone expression in context (e.g. `priv` in `@get(priv)`) is parsed
  as `Expr::Var` and interpreted by the expander, not the grammar. Expanders that
  take flag-style args (`@get(priv)`) match on `Expr::Var` names; expanders that
  take predicates (`@requires(x > 0)`) evaluate the full expr.
- **`old(...)`** is ordinary call syntax at parse time; it is recognized as a
  pseudo-selector only by the `@ensures` expander (annotations-contracts.md).

### Newline binding (ADR-0016)

An attribute binds to the **next member**, skipping any number of newlines
between them:

```phalcom
@requires(amount > 0)

deposit(amount) { ... }        // binds — blank lines allowed
```

A `}` or EOF before a member is a **dangling-attribute error** with the
attribute's span. This mirrors `parse_class_body`'s existing `skip_newlines`
(`parser.rs` L527).

### `Target` and the legality table

```rust
enum Target { Class, Method, Getter, Setter, Field }
```

| Attribute | Legal on | Illegal on → error |
|-----------|----------|--------------------|
| `@construct` | Class | anything else |
| `@data`, `@sealed` | Class | member |
| `@variant` | Class-nested variant decl | plain method |
| `@get`, `@set` | Field | method/getter/setter/class |
| `@requires`, `@ensures` | Method, Getter, Setter | Class, Field |
| `@invariant` | Class (member-position predicate) | Method, Field |

`@requires`/`@ensures` **are** legal on `static` methods (they weave the body
like any other). `@invariant` is **not** applied to static methods
(annotations-contracts.md).

### Unknown / misplaced attributes

- **Unknown name** (`@typo`) → expansion-time error naming the attribute, listing
  registered names. Never silently ignored (contrast Java's ignore-unknown).
- **Known name, illegal target** → error citing the legality table row.

Both carry the `Attribute.range` span (D3, annotations-core.md).

## Consequences

- The `AttributeExpander` registry gains a `legal_targets() -> &[Target]` method;
  the pass checks position before calling `expand`, so every expander assumes a
  valid target.
- Class-member-only scope keeps `@` out of the expression grammar entirely — no
  interaction with `::` method-refs or `#` symbols.

## What this precludes

Class-member-only forecloses Python-style `@decorator`-on-a-`def`-anywhere and
attribute-on-expression forms without a grammar extension — acceptable for Draft
0.1, revisitable behind a new production.
