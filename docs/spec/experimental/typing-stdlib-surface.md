# Typing the committed surface — root protocol, `==`, variadics, catch, literals (proposed)

- Status: **Proposed** (experimental; not ratified)
- Axis: typing ⊗ standard-library surface
- Resolves: [typing.md](typing.md) Tier-2 gaps #5 (variadics), #6 (equality), #8 (catch) and Tier-3 (root protocol, collection literals, interpolation)
- Related: [equality-and-hash.md](equality-and-hash.md), [iteration-protocol.md](iteration-protocol.md), [numeric-and-string-indexing.md](numeric-and-string-indexing.md), [error-handling.md](../error-handling.md), [messages-and-selectors.md](../messages-and-selectors.md), [ADR-0012](../../adr/0012-selector-signature-encoding-and-dispatch.md)

## Problem

[typing.md](typing.md) types `List` and the core forms but skips several *committed*
surface features and the root of the type lattice: the universal `Object` protocol
(what `Any` can do), how `==` is typed, variadic rest params (`*xs`, ADR-0012), the
`catch` binding, and the non-`List` collection / interpolation literals. Each is a
small but load-bearing decision; left implicit they will be improvised inconsistently.

## Decision

### Root `Object` protocol — what makes `Any` ≠ `?`

Every value (hence `Any`) understands exactly this minimal protocol:

| Selector | Type |
|----------|------|
| `==(_: Any)` | `Bool` |
| `hash` | `Int` |
| `toString` | `String` |
| `class` | `Self class` |

Sending anything *else* to an `Any`-typed receiver is a **type error** — this is the
concrete difference from `?` ([typing.md §4](typing.md)): `Any` remembers it is *some*
value and permits only universal messages; `?` defers all checking to runtime.

### Equality takes `Any`, not `Self`

```phalcom
==(other: Any) -> Bool
```

Deliberately **not** `Self`. Cross-type comparison (`1 == "x"`) is legal and `false`,
matching the runtime ladder in [equality-and-hash.md](equality-and-hash.md). Making
`==` take `Self` would turn `1 == "x"` into a *type error*, contradicting that ladder
and surprising every programmer. A class overriding `==` narrows the *implementation*;
the *type* stays `Any`-accepting. (Arithmetic `+` still takes `Self` — equality and
arithmetic differ precisely here.)

### Variadic rest parameters

```phalcom
sum(*xs: Int) -> Int            //  xs : List<Int>  (read-only)
```

- A rest parameter typed `T` binds as `List<out T>` in the body.
- Call site accepts zero-or-more `T` arguments; a spread `sum(*ys)` requires
  `ys : List<T>` (or an `Iterable<T>`, → [iteration-protocol.md](iteration-protocol.md)).
- The *type* is orthogonal to the variadic dispatch-table key (DEC-B /
  [messages §4](../messages-and-selectors.md)) and is erased like every other
  annotation.

### `catch` binding — a typed handler, not checked exceptions

```phalcom
try { … } catch (e) { … }              // e : Error
try { … } catch (e: NetworkError) { … } // e : NetworkError; runtime class test
```

- Un-annotated `catch (e)` binds `e : Error` (the throwable root — only `Error`
  subclasses are throwable, [error-handling.md](../error-handling.md)).
- `catch (e: NetworkError)` binds `e : NetworkError` **and** filters at runtime by
  class; non-matching throws propagate.
- **Not an erasure violation.** `try`/`catch` already desugars to the `on(_)(_)` block
  protocol, where the error *class* is a runtime operand that selects the handler. The
  annotation merely *names* the class the existing dispatch already tests — the class
  test exists with or without the type. The throw *channel* stays an untracked effect
  ([typing.md §5.11](typing.md)); only the handler *binding* is typed.

### Collection & interpolation literals

| Literal | Type |
|---------|------|
| `[a, b]` | `List<LUB(a, b)>` |
| `{k: v, …}` | `Map<LUB(keys), LUB(values)>` (disjoint values ⇒ `Map<Symbol, ?>`) |
| `(a, b)` | `Tuple<A, B>` — fixed-arity heterogeneous product |
| `1..5` | `Range<Int>` |
| `Set(a, b)` | `Set<LUB(a, b)>` |
| `"{e}"` | `String`; constrains `e` to the root protocol's `toString` — so **any** value interpolates (interpolation is total) |

Indexing types (`List<T>.at(_) -> Option<T>`, codepoint strings, integral indices)
are **out of scope here** — they live in
[numeric-and-string-indexing.md](numeric-and-string-indexing.md).

## Precludes

- **`==` typed as `Self`** — would break heterogeneous comparison and contradict the
  equality ladder. Locked to `Any`.
- **Checked exceptions** — `catch` types the handler binding only; the throw channel
  remains an untracked effect. No method carries a `throws T` in its type.
- **A second variadic in one selector** — the rest-param type does not rescue the
  dispatch-key ambiguity; DEC-B's "reject a 2nd same-name variadic at definition"
  still governs.
