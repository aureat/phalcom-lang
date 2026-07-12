# Result

Part of the [Phalcom Language Specification](README.md). Status: Normative
(model fixed by [ADR-0008](../../adr/0008-layered-exceptions-and-result.md);
shape mirrors [ADR-0007](../../adr/0007-option-as-abstract-with-some-none.md)).

`Result` is the **value** channel for *expected, local* failure — the sibling of
[`Option`](values-and-absence.md) for cases where the caller wants the failure
*and its reason* visible in the type, rather than unwinding the stack. It composes
with the exception channel through cheap bridges
([Error Handling §5](error-handling.md)).

## 1. Shape — mirrors `Option`

`Result` is an **abstract superclass** with exactly two concrete subclasses, laid
out identically to `Option`/`Some`/`None` (ADR-0007), so the same machinery and
mental model apply:

| `Option` | `Result` | Holds |
|---|---|---|
| abstract `Option` | abstract `Result` | — |
| `Some(value)` | `Ok(value)` | a success value |
| `None` (singleton) | `Err(error)` | a failure reason (an `Error`, or any value) |

`Ok`/`Err` are ordinary heap instances (a single `_value` / `_error` field), like
`Some` — the object model stays uniform.

## 2. Protocol

| Selector | On | Meaning |
|---|---|---|
| `isOk` / `isErr` | `Result` | which arm → `Bool` |
| `map(_)` | `Result` | transform the `Ok` value; `Err` passes through unchanged |
| `mapErr(_)` | `Result` | transform the `Err` reason; `Ok` passes through |
| `andThen(_)` | `Result` | chain an `Ok`→`Result` function (flat-map); short-circuits on `Err` |
| `unwrap` | `Result` | the `Ok` value, or **`throw`** the `Err` ([ADR-0008](../../adr/0008-layered-exceptions-and-result.md)) |
| `unwrapOr(_)` | `Result` | the `Ok` value, or a default |
| `unwrapErr` | `Result` | the `Err` reason, or `throw` if `Ok` |
| `ok()` | `Result` | to `Option`: `Ok(v)` → `Some(v)`, `Err(_)` → `None` |
| `okOr(_)` | `Option` | to `Result`: `Some(v)` → `Ok(v)`, `None` → `Err(err)` |

`map`/`mapErr`/`andThen` never raise — they are pure value transforms; only
`unwrap`/`unwrapErr` cross into the exception channel.

```phalcom
Int.parse(input)                       // Result<Int, Error>
  .map { n => n * 2 }
  .unwrapOr(0)
```

## 3. Bridges to the exception channel

The two channels are interconvertible in one send each — the reason layering works
([Error Handling §5](error-handling.md)):

| Direction | Form | Meaning |
|---|---|---|
| throw → value | `{ risky() }.attempt()` | run the block, capturing a `throw` into `Err(e)`; success is `Ok(v)` |
| value → throw | `result.unwrap()` | the `Ok` value, or re-`throw` the contained `Err` |
| absence ↔ error | `option.okOr(err)` / `result.ok()` | fill in / drop the failure reason |

## 4. Bootstrap & status

`Result`/`Ok`/`Err` are kernel value classes that bootstrap **alongside**
`Option`/`Some`/`None`, reusing the ADR-0007 abstract-root-plus-two-subclasses
machinery (no new mechanism). They are **reserved** by U-CORE-6 and implemented by
a later value-classes unit ([Deferred & Future Work §3](deferred-work.md)); this
spec is the ratified design that unit builds to.

## 5. Non-goals

- **A third arm** (e.g. `Pending`) — `Result` is exactly two-armed, mirroring
  `Option`.
- **Auto-propagation sugar** (a `?` operator) — deferred; `andThen`/`unwrap` cover
  the cases explicitly for now.
