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
(x > 0).ifTrue { "positive" }            // Some("positive") or None
(x > 0).ifTrue { "pos" }.ifNone { "non-pos" }
```

### Core `Option` protocol

`ifSome(_:)`, `ifNone(_:)`, `map(_:)`, `orElse(_:)`, `unwrapOr(_:)`, `isSome`,
`isNone`.

`??` is sugar for `orElse` and short-circuits ([Control Flow](control-flow.md)):

```phalcom
a ?? b        // a.orElse { b }
```
</content>
