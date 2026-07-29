# Phalcom Generator Semantics — Recorded Decisions Pending Reconciliation

**Status:** Provisional design record; not yet normative  
**Important:** Phalcom already has generator specifications whose semantics may differ from the assumptions recorded here. This document preserves accepted recommendations for later comparison and reconciliation. It must not silently override existing generator semantics.

---

## 1. Recorded first-version model

The recorded recommendation is that the first generator model be unidirectional:

- the generator yields values of one type;
- callers do not send values back into the suspended generator;
- generator completion has no exposed payload and therefore uses unit internally;
- invocation returns an iterator object.

Illustrative syntax:

```phalcom
generator numbers() yields Int {
  yield 1
  yield 2
  yield 3
}
```

Conceptual invocation type:

```phalcom
() -> Iterator<Int>
```

The exact declaration syntax, keywords, and runtime object names must be reconciled with Phalcom's existing generator specification.

---

## 2. Completion semantics

A first-version generator completes without an exposed payload.

The following terminate the generator normally:

```phalcom
return
```

```phalcom
return ()
```

```phalcom
// reaching the end of the body
```

The internal completion value is `()`.

A non-unit generator return is rejected in the first version:

```phalcom
return 42
// invalid in the recorded V1 model
```

Rationale:

- no public API exposes the completion payload;
- accepting it would create a value that disappears;
- observable generator return values introduce a more complex `Generator<Yield, Return>` model;
- bidirectional send values would add a third type parameter.

A generalized generator model may be considered later.

---

## 3. Iterator protocol

The recorded iterator protocol is:

```phalcom
@protocol
class Iterator<out T> {
  next() -> Option<T>
}
```

Each `next()` call returns:

```phalcom
Some(value)
```

when another element exists, or:

```phalcom
None
```

when the iterator is exhausted.

This cleanly separates:

```text
Some(value) = an element is present
None        = no further element exists
()          = successful completion without a payload
```

Iteration exhaustion is represented by absence because `next()` asks whether another element exists.

---

## 4. Type of `yield`

In the recorded unidirectional model:

```phalcom
yield value
```

has type:

```phalcom
()
```

It emits a value, suspends execution, and later resumes without receiving a payload from the caller.

This permits ordinary sequencing:

```phalcom
generator values() yields Int {
  yield 1
  yield 2
}
```

A later bidirectional model could give `yield` the type of the value sent back into the generator, but that is explicitly deferred.

---

## 5. Empty generators

An empty generator is valid:

```phalcom
generator empty() yields Int {
}
```

Invocation returns an iterator whose first `next()` produces `None`.

A generator yielding `Never` is also meaningful:

```phalcom
generator empty() yields Never {
}
```

Since no `Never` value can exist, the iterator cannot normally produce an element.

Its `next()` result is:

```phalcom
Option<Never>
```

whose only normal value is `None`.

This follows directly from the ratified core type lattice.

---

## 6. Infinite generators

An infinite generator may be written conceptually as:

```phalcom
generator naturals() yields Int {
  var value = 0

  while true {
    yield value
    value++
  }
}
```

The generator declaration itself does not have result type `Never`.

Calling it returns an iterator normally:

```phalcom
const values = naturals()
```

What does not terminate is iteration to exhaustion.

Distinction:

```text
generator construction:
    completes and returns Iterator<Int>

generator sequence:
    may have no terminal exhaustion state
```

The initial type system need not encode finiteness.

---

## 7. Failure behavior

Generator failure is distinct from normal exhaustion.

A `next()` operation may:

- return `Some(value)`;
- return `None` on normal exhaustion;
- throw or propagate a generator failure.

Failure must not be represented as `None`, because `None` means normal absence of another element.

Exact failure storage, traceback preservation, cleanup, and resumption behavior remain subject to the existing generator and exception specifications.

---

## 8. Cleanup and finalization

The final generator design must define cleanup when:

- iteration reaches normal exhaustion;
- the generator throws;
- the consumer stops early;
- the iterator is discarded;
- cancellation or task termination occurs for async generators;
- a `finally` or `ensure` region surrounds a `yield`.

The recorded V1 type model does not answer these runtime-lifecycle questions.

---

## 9. Async generators

The recorded composition of async and iteration is:

```phalcom
@protocol
class AsyncIterator<out T> {
  next() -> Task<Option<T>>
}
```

An illustrative async generator:

```phalcom
async generator events() yields Event {
  while true {
    const event = await source.nextEvent()
    yield event
  }
}
```

Conceptual invocation type:

```phalcom
() -> AsyncIterator<Event>
```

Calling `next()` returns a task. Awaiting that task produces:

```phalcom
Some(event)
```

or:

```phalcom
None
```

on normal exhaustion.

This composition preserves distinct responsibilities:

```text
Task<Option<T>>
│    │      │
│    │      └─ yielded element type
│    └─ whether another element exists
└─ when the next-step result becomes available
```

All names and async interactions are provisional until reconciled with Phalcom's existing async and generator specifications.

---

## 10. Deferred generalized generator model

The following capabilities are deferred:

### 10.1 Observable completion payload

Possible future form:

```phalcom
generator parse() yields Token -> ParseSummary {
  ...
  return summary
}
```

Possible type:

```phalcom
Generator<Token, ParseSummary>
```

### 10.2 Values sent into a suspended generator

A fully generalized model might require:

```text
Generator<Yield, Send, Return>
```

This affects:

- the type of `yield` expressions;
- resumption APIs;
- delegation;
- exception injection;
- reflection;
- variance;
- static checking.

These features should not be introduced without a concrete use case and a dedicated design.

---

## 11. Reconciliation requirements

Before ratification, compare this record with the existing Phalcom generator specification and resolve at least:

1. Existing generator declaration syntax.
2. Existing iterator and generator object names.
3. Whether invocation returns an iterator, iterable, stream, or another object.
4. Exact `yield` expression semantics.
5. Existing support for sent values.
6. Existing support for return payloads.
7. Delegation such as `yield from` or equivalent.
8. Error propagation and cleanup.
9. Suspension across `ensure` or `finally`.
10. Reentrancy and concurrent iteration.
11. Reflection and callable-type representation.
12. Variance of iterator and generator types.
13. Async-generator composition.
14. VM frame, stack, GC, and bytecode obligations.
15. Interaction with pattern matching and comprehensions.

---

## 12. Recorded recommendations summary

The following decisions are preserved for later reconciliation:

- The initial generator model should be unidirectional.
- A generator should explicitly declare its yielded element type.
- Invocation should conceptually return `Iterator<T>`.
- V1 generator completion should be internal unit `()` with no exposed payload.
- `return`, `return ()`, and fallthrough should terminate normally.
- `return value` should be rejected in V1.
- `yield value` should have type `()` in the unidirectional model.
- `Iterator<out T>#next() -> Option<T>` should represent iteration.
- `None` should mean normal exhaustion.
- Generator failure must remain distinct from exhaustion.
- `Iterator<Never>` is meaningful and cannot produce an element.
- Infinite generators return iterator objects normally; only exhaustive iteration diverges.
- Async iteration should conceptually use `Task<Option<T>>`.
- Bidirectional send values and observable return payloads are deferred.

None of these provisions override existing Phalcom generator specifications until an explicit reconciliation document ratifies the final model.
