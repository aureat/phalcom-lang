# Values & Absence

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

## 1. Value types

| Type | Notes |
|------|-------|
| `Number` | Single numeric type ([open question](open-questions.md) re: int/float split) |
| `String` | Immutable, interpolating |
| `Bool` | A real class; `ifTrue`/`ifFalse` are stdlib methods, not VM builtins |
| `Block` | First-class closure ([Blocks](blocks.md)) |
| `Tuple` | Fixed-arity product type, `(3, 4)` |
| `List` | `[1, 2, 3]` |
| `Map` | `{ a: 1 }` |
| `Set` | `Set(1, 2)` |
| `Range` | `1..5` |
| `Option` | `Some(v)` / `None` — the only way to express absence |
| `Class` | Classes are objects; classes have metaclasses ([Object Model](object-model.md)) |
| `Message` | Reified send ([Method Lookup](method-lookup.md)) |

## 2. `nil` is private

There is no `nil`, `null`, or `undefined` in user-facing Phalcom.

`nil` exists in the VM as an implementation detail — uninitialized slots and
internal sentinels. It has **no surface syntax, no literal, and cannot be produced
by user code** (Invariant 4).

## 3. Absence is `Option`

```phalcom
Some(42)
None
```

- `var x` with no initializer is `None`.
- A declared-but-unassigned field reads as `None` ([Classes §Fields](classes.md)).
- `ifTrue` / `ifFalse` **return an `Option`** — they are semantically a `map` over
  a boolean:

```phalcom
(x > 0).ifTrue { "positive" }                 // Some("positive") or None
(x > 0).ifTrue { "pos" }.unwrapOr("non-pos")  // extract with a default
```

### 3.1 Class shape

`Option` is an abstract kernel class with two concrete subclasses, exactly
mirroring `Bool` / `True` / `False` ([ADR-0004](../adr/), [Object Model](object-model.md)):

| Class | Kind | State |
|-------|------|-------|
| `Option` | abstract | — |
| `Some` | subclass | one field, `_value` |
| `None` | subclass | singleton instance |

`None` is a single shared instance (like `true` / `false`): identity-comparable
and zero-allocation. Only `Some` allocates. Every combinator is two method
definitions — `Some>>map` and `None>>map` — so dispatch replaces branching; there
is no variant tag to test.

`Some(v)` is an ordinary construction send; `None` is a global bound to the
singleton.

### 3.2 The eliminator

`match(some:, none:)` is the one primitive that leaves Option-world with a value.
Every other extractor is defined in terms of it.

```phalcom
opt.match(
  some: { v => "got {v}" },
  none: { "empty" }
)                                             // -> the common type of both blocks
```

### 3.3 Core protocol

Grouped by what they return.

**Transform** — stay inside `Option`:

| Selector | Returns | Meaning |
|----------|---------|---------|
| `map(_:)` | `Option<U>` | `Some(f(v))` / `None` |
| `flatMap(_:)` | `Option<U>` | monadic bind; `f` already returns an `Option`, so no nesting |
| `filter(_:)` | `Option<T>` | `Some(v)` if the predicate holds, else `None` |
| `orElse(_:)` | `Option<T>` | `Some` passes through; `None` becomes the block's `Option` |
| `zip(_:)` | `Option<(T, U)>` | `Some((a, b))` iff both are `Some` |

**Extract** — leave `Option`:

| Selector | Returns | Meaning |
|----------|---------|---------|
| `unwrapOr(_:)` | `T` | the value, or the given default |
| `unwrapOrElse(_:)` | `T` | the value, or the block's result |
| `unwrap()` | `T` | the value; sends `doesNotUnderstand`-style error on `None` |
| `match(some:, none:)` | `T` | the eliminator (§3.2) |

**Effect** — run a block for its side effect, return `self` so calls chain:

| Selector | Returns | Meaning |
|----------|---------|---------|
| `ifSome(_:)` | `Option<T>` (self) | runs the block with the value when `Some` |
| `ifNone(_:)` | `Option<T>` (self) | runs the block when `None` |

`ifSome` / `ifNone` **never extract** — extraction is only `unwrapOr` / `unwrapOrElse`
/ `unwrap` / `match`. This keeps their return type unambiguous and makes them
freely chainable:

```phalcom
opt.ifSome { v => log(v) }.ifNone { warn("missing") }   // -> opt, unchanged
```

**Query:**

| Selector | Returns |
|----------|---------|
| `isSome` | `Bool` |
| `isNone` | `Bool` |
| `contains(_:)` | `Bool` — `Some(x)` where `x == arg` |

### 3.4 `??` and `?.`

Both desugar to Option sends and short-circuit. Tokenization and precedence live
in [Lexical Structure §9](lexical-structure.md):

```phalcom
a ?? b            // a.orElse { b }
opt?.foo          // opt.map { x => x.foo }
opt?.bar(baz)     // opt.map { x => x.bar(baz) }
```

`?.` gives JavaScript-style member access over `Option`; chained, each hop stays
inside `Option` and the first `None` short-circuits the rest.

### 3.5 No truthiness

`Option` is not `Bool`. `if (opt) { … }` is a **compile error**; a condition must
be a `Bool`. Reach through `.isSome` / `.isNone`, or use `ifSome` / `ifNone`. This
is a deliberate, signposted deviation from JavaScript (Invariant 6) — the absence
of `nil` already makes truthiness meaningless here.

### 3.6 Equality and iteration

- `None == None` by identity; `Some(a) == Some(b)` iff `a == b` (delegates to the
  inner value's `==`).
- `Option` conforms to `Iterable`, yielding zero or one element, so `opt.each { … }`
  and `for x in opt` work ([Collections](open-questions.md) — protocol finalized
  with the iteration work).

### 3.7 The `Result` bridge (reserved)

`Result` is not yet designed ([Open Questions §9](open-questions.md)). This section
only reserves shared vocabulary so it slots in without churn: `map` / `flatMap` /
`unwrapOr` carry identical meaning on both, and the bridges are
`opt.okOr(err) -> Result` and `result.ok() -> Option`.
</content>
