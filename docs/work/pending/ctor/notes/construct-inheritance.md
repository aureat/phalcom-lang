# Annotations — `@construct` inheritance, collisions, field defaults

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Depends on: [construct-derive.md](construct-derive.md)
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
- The super-param list is read off the parent's **reflected constructor
  signature** — its `MethodDef.params` label list — not off whether the parent
  used `@construct`. A constructor's labeled-parameter list is statically known
  at the parent's definition time regardless of whether it was hand-written or
  `@construct`-derived (`ParameterDef.label` already exists, `ast.rs:37`), so
  requiring the parent to *also* use `@construct` was never load-bearing for
  correctness — it only restricted *which already-known signatures* the
  subclass could read.

```phalcom
class Animal { construct new(name:) { _name = name } }   // hand-written parent
@construct class Dog : Animal { var _breed }
// Dog.new(name:, breed:)  ⇒  super.new(name); _breed = breed
```

**Ambiguity, not ancestry, is the failure case.** `@construct` on a subclass is
a compile error (`construct.super_ambiguous`) only if the superclass has **more
than one** constructor selector (overloaded `new` with different label sets) —
there is then no single signature to infer from, and the subclass must
hand-write its constructor and pick one explicitly via `super.new(...)`. A
superclass with **exactly one** constructor selector — hand-written,
`@construct`-derived, or itself inferred this same way — always has a total,
unambiguous signature to read, so `@construct` composes freely up any
hierarchy depth without requiring uniform derivation at every level.

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

- `@construct` derivation stays total for the common case (exactly one
  superclass constructor selector): the expander never has to guess a
  hand-written super's signature, it reads it — the same reflection surface
  `Family`/`perform` already need to exist. It only degrades to a compile
  error in the genuinely ambiguous case (an overloaded superclass
  constructor), not merely because the superclass didn't itself use
  `@construct`.
- Default-bearing fields shrink the constructor signature — reordering or adding a
  defaulted field changes the param list, so the R3 field-order-is-API caveat
  (annotations-construct.md) extends to defaults.
- `@construct` is now usable on the *first* subclass of any hand-written
  `core.ph` kernel class with a single constructor selector, removing the
  Draft 0.1 blocker that otherwise made `@construct` unusable on day one for
  the common case of extending an unannotated base class.

## What this precludes

Signature-inference-from-reflection precludes nothing `@construct` could do
before; it only removes the artificial "every ancestor must also use
`@construct`" restriction. The genuine limit that remains — an overloaded
superclass constructor has no single signature to infer — is inherent to
inference itself, not a Draft 0.1 simplification to be lifted later.
