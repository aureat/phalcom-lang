# Annotations — `@construct` inheritance, collisions, field defaults

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Depends on: [annotations-construct.md](annotations-construct.md)
- Resolves: construct gaps — super-construct chaining, hand-written collision, `let` vs `var` fields, default timing
- Related: classes.md §2 (private, non-inherited fields), ADR-0011 (fixed slots), ADR-0014 (`let`/`var`)

## Context

annotations-construct.md derives a constructor from own fields but is silent on
inheritance. Because fields are **private and non-inherited** (classes.md §2), a
derived class's `@construct` cannot touch the parent's slots — so subclass
construction has no defined path.

## Decision

### Super-construct chaining

`@construct` initializes **only the declaring class's own fields**. A subclass
must be able to initialize the parent. Rule:

- If the class has a superclass with its own fields, the derived `@construct`'s
  synthesized params are `super-params ++ own-params`, and the synthesized body
  begins with `super.new(<super-params>)` before assigning own fields.
- The super-param list is the parent's `@construct` param list (or, for a
  hand-written parent constructor, a **compile error** — `@construct` cannot infer
  a hand-written super's signature; the subclass must hand-write its constructor).

```phalcom
@construct class Animal { var _name }
@construct class Dog : Animal { var _breed }
// Dog.new(name:, breed:)  ⇒  super.new(name); _breed = breed
```

Draft 0.1 conservative fallback: **`@construct` is legal only when every ancestor
up the chain also uses `@construct`** (uniform derivation). Mixed hand-written /
derived hierarchies require a hand-written constructor at the mix point. This
keeps super-signature inference total.

### Collision with a hand-written `construct`

A class carrying `@construct` **and** a hand-written `construct new(...)` of the
same selector is a **compile error** (ADR-0012: selector is sole dispatch key, no
last-wins) — identical policy to the `@get`/`@set` collision rule. `@construct`
plus a *differently-selectored* hand-written constructor (e.g. `construct
anonymous()`) is fine; they coexist.

### `let` vs `var` fields as parameters

Both `let _x` (immutable) and `var _x` (mutable) field declarations become
constructor parameters — `@construct` sets each once at construction. The
`let`/`var` distinction governs **post-construction** mutability (a later
`_x = …` in another method is a compile error for `let` fields), not
participation in the constructor.

### Field default timing

`var _x = expr` / `let _x = expr` — the default `expr` is evaluated **per
instance, at construct time, before the `@construct` body runs**, in field
declaration order. A field with a default is **omitted** from the synthesized
constructor's parameter list (it is not caller-supplied); supply-and-default is
mutually exclusive per field. A field without a default and without a construct
param reads as `None` (classes.md §2 / open-Q1) until assigned.

## Consequences

- `@construct` derivation stays total: the uniform-derivation fallback means the
  expander never has to reverse-engineer a hand-written super.
- Default-bearing fields shrink the constructor signature — reordering or adding a
  defaulted field changes the param list, so the R3 field-order-is-API caveat
  (annotations-construct.md) extends to defaults.

## What this precludes

The uniform-derivation fallback precludes `@construct` on a subclass of a
hand-written-constructor parent (until a signature-annotation escape hatch is
added) — a deliberate Draft 0.1 simplification, not a permanent limit.
