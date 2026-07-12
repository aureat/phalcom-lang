# Values & Absence

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADRs:**
[ADR-0007](../adr/0007-option-as-abstract-with-some-none.md) (Option as abstract with Some/None) ·
[ADR-0010](../adr/0010-tagged-value-enum.md) (tagged Value enum) ·
[ADR-0014](../adr/0014-let-and-var-bindings.md) (let and var bindings)

## 1. Value types

| Type | Notes |
|------|-------|
| `Int` / `Float` | Abstract `Number` split into exact unbounded `Int` + `f64` `Float` ([ADR-0024](../adr/0024-numeric-surface-split-int-float-and-division.md)) |
| `String` | Immutable, interpolating |
| `Bool` | A real class; `ifTrue`/`ifFalse` are stdlib methods, not VM builtins |
| `Block` | First-class closure ([Blocks](blocks.md)) |
| `Tuple` | Fixed-arity product type, `(3, 4)` |
| `List` | `[1, 2, 3]` |
| `Map` | `{ a: 1 }` |
| `Set` | `Set(1, 2)` |
| `Range` | `1..5` |
| `Option` | `Some(v)` / `None` — the only way to express absence |
| `Result` | `Ok(v)` / `Err(e)` — an expected failure as a value ([Error Handling](error-handling.md)) |
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

`match(some,none)` is the one primitive that leaves Option-world with a value.
Every other extractor is defined in terms of it.

```phalcom
opt.match(
  some: { v => "got \(v)" },
  none: { "empty" }
)                                             // -> the common type of both blocks
```

### 3.3 Core protocol

Grouped by what they return.

**Transform** — stay inside `Option`:

| Selector | Returns | Meaning |
|----------|---------|---------|
| `map(_)` | `Option<U>` | `Some(f(v))` / `None` |
| `flatMap(_)` | `Option<U>` | monadic bind; `f` already returns an `Option`, so no nesting |
| `filter(_)` | `Option<T>` | `Some(v)` if the predicate holds, else `None` |
| `orElse(_)` | `Option<T>` | `Some` passes through; `None` becomes the block's `Option` |
| `zip(_)` | `Option<(T, U)>` | `Some((a, b))` iff both are `Some` |

**Extract** — leave `Option`:

| Selector | Returns | Meaning |
|----------|---------|---------|
| `unwrapOr(_)` | `T` | the value, or the given default |
| `unwrapOrElse(_)` | `T` | the value, or the block's result |
| `unwrap()` | `T` | the value; sends `doesNotUnderstand`-style error on `None` |
| `match(some,none)` | `T` | the eliminator (§3.2) |

**Effect** — run a block for its side effect, return `self` so calls chain:

| Selector | Returns | Meaning |
|----------|---------|---------|
| `ifSome(_)` | `Option<T>` (self) | runs the block with the value when `Some` |
| `ifNone(_)` | `Option<T>` (self) | runs the block when `None` |

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
| `contains(_)` | `Bool` — `Some(x)` where `x == arg` |

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

### 3.7 The `Result` bridge

`Result` (§4) is `Option`'s sibling for *failure that carries a reason*. `map` /
`flatMap` / `unwrapOr` carry identical meaning on both, and the two convert freely:
`opt.okOr(err) -> Result` and `result.ok() -> Option`.

## 4. `Result` — expected failure as a value

`Result` is the value channel of [error handling](error-handling.md): use it for
*expected, local* failures (parse, validate, lookup), and reserve `throw` for the
exceptional. It mirrors `Option` exactly.

### 4.1 Class shape

Like `Option` ([§3.1](#31-class-shape), [ADR-0007](../adr/0007-option-as-abstract-with-some-none.md)),
`Result` is abstract with two concrete subclasses:

| Class | Kind | State |
|-------|------|-------|
| `Result` | abstract | — |
| `Ok` | subclass | one field, `_value` |
| `Err` | subclass | one field, `_error` (an [`Error`](object-model.md)) |

Combinators are two method definitions each — `Ok>>map`, `Err>>map` — so dispatch
replaces branching, and `Ok`/`Err` are meaningful classes to user code.

### 4.2 Protocol

Parallel to `Option`, grouped by what they return.

**Transform** — stay inside `Result`:

| Selector | Returns | Meaning |
|----------|---------|---------|
| `map(_)` | `Result<U, E>` | `Ok(f(v))` / `Err` passes through |
| `flatMap(_)` | `Result<U, E>` | monadic bind; `f` returns a `Result` |
| `mapErr(_)` | `Result<T, F>` | transform the error; `Ok` passes through |
| `orElse(_)` | `Result<T, F>` | `Ok` passes through; `Err` becomes the block's `Result` |

**Extract** — leave `Result`:

| Selector | Returns | Meaning |
|----------|---------|---------|
| `unwrapOr(_)` | `T` | the value, or the given default |
| `unwrap()` | `T` | the value; **`throw`s** the contained `Err` on failure |
| `match(ok,err)` | `T` | the eliminator |

**Query:** `isOk`, `isErr` → `Bool`.

### 4.3 Bridges

| From → to | Form |
|-----------|------|
| `Result` → `Option` | `result.ok()` — `Some(v)` / `None` (discards the error) |
| `Option` → `Result` | `opt.okOr(err)` — `Ok(v)` / `Err(err)` |
| `throw` → `Result` | `{ risky() }.attempt()` ([Error Handling §5](error-handling.md)) |
| `Result` → `throw` | `result.unwrap()` |

`match(ok,err)` and the effecting/query forms follow `Option`'s conventions
([§3.3](#33-core-protocol)); `Result` has no truthiness and no implicit coercion
for the same reasons ([§3.5](#35-no-truthiness)).
</content>
