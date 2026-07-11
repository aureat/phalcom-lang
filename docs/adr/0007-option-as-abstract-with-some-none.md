# 7. Represent absence as abstract `Option` + `Some`/`None`

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/values-and-absence.md` §3; [ADR-0004](0004-boolean-as-abstract-bool-with-true-false.md); `phalcom-core/src/nil.rs`, `value.rs`

## Context

Phalcom has no surface `nil` (Invariant 4): the VM keeps a private `nil` for
uninitialized slots, but user code expresses absence exclusively through `Option`.
`values-and-absence.md` originally listed `Option` as a value type with a handful
of combinators, leaving three things unsettled:

1. **Class shape.** Is `Option` one class with a variant tag, or an abstract
   superclass with `Some`/`None` subclasses?
2. **Combinator semantics.** The spec's own examples used `ifNone` both as a
   side-effecting hook (`ifSome { … }` for effect) and as an extractor
   (`.ifNone { "non-pos" }` producing a bare value). Those return types conflict.
3. **JS ergonomics.** JavaScript programmers reach for `?.` and `??` constantly;
   only `??` was specified.

## Decision

**Recommendation (pending approval):** model `Option` exactly like `Bool`
([ADR-0004](0004-boolean-as-abstract-bool-with-true-false.md)).

- `Option` is abstract; `Some` and `None` are its concrete subclasses. `Some`
  carries one field `_value`; `None` is a single shared singleton instance
  (identity-comparable, zero-allocation). Combinators are two method definitions
  each — `Some>>map`, `None>>map` — so **dispatch replaces branching**; there is no
  variant tag to test.
- `match(some:, none:)` is the sole eliminator that leaves Option-world with a
  value; `unwrapOr` / `unwrapOrElse` / `unwrap` are defined over it.
- `ifSome` / `ifNone` are **effecting** — they run a block and return `self`, never
  extract. This resolves the return-type conflict and makes them chainable.
- The protocol gains monadic bind `flatMap` (absent before), without which any
  `map` returning an `Option` nests into `Option<Option>`.
- Surface sugar `opt?.foo ≡ opt.map { x => x.foo }`, alongside the existing
  `a ?? b ≡ a.orElse { b }`. Both short-circuit.
- `Option` is not `Bool`: `if (opt)` is a **compile error**. No implicit
  truthiness — a signposted deviation from JavaScript (Invariant 6), coherent
  because the absence of `nil` already makes truthiness meaningless.

Like ADR-0004, this is a refinement toward a finished language, not a minimum
correctness fix, and can follow the kernel work.

## Consequences

- Absence handling is ordinary polymorphic dispatch, uniform with the rest of the
  object model; `Some`/`None` are meaningful classes to user code.
- `None` as a shared singleton keeps the common case (every unassigned field reads
  `None`) allocation-free; only `Some` allocates.
- `?.` adds one token and desugaring to the grammar
  ([lexical-structure.md §9](../spec/lexical-structure.md)); it must be threaded
  into the precedence table in the grammar pass.
- Two extra kernel classes to bootstrap and keep invariant-checked, plus the
  private VM `nil` must never leak into a `Some`.

## Alternatives considered

- **Single `Option` class with a tag field.** Fewer classes, but every combinator
  becomes an internal `if tag == Some` test — special-cased instead of dispatched,
  inconsistent with `Bool`/`Number`/`Function`.
- **`ifSome`/`ifNone` as extractors** (per the original example). Overloads them
  with an ambiguous return type and collides with `unwrapOr`; rejected in favor of
  effect-and-return-`self`.
- **Implicit truthiness** (`if (opt)`). Familiar to JS, but silently re-introduces
  nil-like coercion the model exists to remove.
