# Phalcom Async Semantics — Recorded Decisions Pending Reconciliation

**Status:** Provisional design record; not yet normative
**Important:** Phalcom already has async specifications whose semantics may differ from the assumptions recorded here. This document preserves the decisions accepted in the design discussion so they can later be compared with and reconciled into the authoritative async specification. It must not silently override existing async semantics.

---

## 1. Recorded conceptual model

An async callable has two related result types:

1. the logical value produced when asynchronous execution completes;
2. the immediate value returned when the callable is invoked.

The recorded preferred source form is:

```phalcom
async save() -> () {
  ...
}
```

Here, `()` denotes the logical awaited result.

Invocation conceptually returns:

```phalcom
Task<()>
```

General rule:

```text
async f(...) -> T

logical completion value: T
invocation result:         Task<T>
```

This model is subject to reconciliation with Phalcom's existing async syntax and runtime object model.

---

## 2. Recorded fallthrough behavior

An async callable whose body reaches the end completes successfully with unit:

```phalcom
async save() -> () {
  await database.flush()
}
```

Conceptually equivalent forms:

```phalcom
async save() -> () {
  await database.flush()
  return ()
}
```

```phalcom
async save() -> () {
  await database.flush()
  return
}
```

A declared non-unit async result must return that value on every normally completing path:

```phalcom
async load() -> Data {
  const bytes = await file.readAll()
  return Data.decode(bytes)
}
```

A path that falls through produces `()` and should receive a static diagnostic when `()` does not satisfy the declared result.

---

## 3. Recorded await behavior

```phalcom
const result = await save()
// result: ()
```

```phalcom
const data = await load()
// data: Data
```

`await` observes the task's logical completion result rather than the task object itself.

Failure and cancellation remain distinct from successful unit completion.

---

## 4. Expression-bodied async callables

The recorded design permits existing expression-body syntax to compose with async:

```phalcom
async fetchName(id: UserId) -> String =>
  (await fetchUser(id)).name
```

The logical result is `String`; invocation conceptually returns `Task<String>`.

This remains provisional until checked against existing parser, compiler, and async specifications.

---

## 5. `Task<T>` and `Promise<T>`

The recorded recommendation separates observation from mutable completion.

```phalcom
class Task<out T> {
  ...
}
```

`Task<T>` is covariant because consumers observe completed values but do not inject arbitrary values into an existing task.

Manual completion uses a separate mutable producer:

```phalcom
class Promise<T> {
  task -> Task<T>
  succeed(value: T)
  fail(error: Error)
  cancel()
}
```

This separation supports:

```text
Task<Dog> <: Task<Animal>
```

without permitting a consumer to complete a `Task<Dog>` with an arbitrary `Animal`.

The names `Task` and `Promise` are provisional and must be reconciled with existing Phalcom terminology.

---

## 6. Completion states

The recorded model keeps these outcomes distinct:

```text
()       successful completion without a payload
None     absence inside Option<T>
failure  exceptional asynchronous completion
cancel   asynchronous cancellation
```

Failure and cancellation must not be represented as `None` or `()`.

An illustrative task state API is:

```phalcom
task.state
// #pending, #succeeded, #failed, #cancelled
```

Awaiting a task may:

- return `T`;
- throw the task's failure;
- throw or propagate cancellation.

Exact effect and error semantics remain deferred to the authoritative async design.

---

## 7. `Task<Never>`

The recorded model allows:

```phalcom
async serveForever() -> Never {
  while true {
    await server.acceptAndHandle()
  }
}
```

Invocation still returns normally with a task object:

```phalcom
Task<Never>
```

Awaiting that task can never produce a normal value. It may:

- run indefinitely;
- fail;
- be cancelled.

This distinction should be retained during reconciliation because it follows directly from the core `Never` semantics.

---

## 8. Callable type representation

Two recorded representations were considered:

```phalcom
() -> Task<T>
```

and:

```phalcom
async () -> T
```

Recorded recommendation:

- preserve `async () -> T` as the source-level semantic form;
- normalize it to `() -> Task<T>` where ordinary callable comparison requires the invocation result;
- preserve async metadata reflectively.

Possible reflection:

```phalcom
method.isAsync
// true
```

```phalcom
method.declaredReturnType
// T
```

```phalcom
method.invocationReturnType
// Task<T>
```

This section is particularly likely to require adjustment to match Phalcom's existing callable-type and async specifications.

---

## 9. Interaction with unit

The recorded principles are:

- async fallthrough follows ordinary body semantics and produces logical unit;
- `async f() -> ()` represents asynchronous successful completion without payload;
- calling such a method produces an asynchronous handle, not the unit value immediately;
- awaiting it produces `()` on success;
- unit does not encode failure, cancellation, timeout, or absence.

---

## 10. Reconciliation requirements

Before ratification, compare this record with the existing Phalcom async specification and resolve at least:

1. Existing async declaration syntax.
2. Existing task/future/promise object names.
3. Whether declared return types denote logical or immediate results.
4. Exact lowering and invocation behavior.
5. Error propagation and typed failure semantics.
6. Cancellation representation.
7. Structured concurrency and task ownership.
8. Async method reflection.
9. Async callable subtyping and variance.
10. Interaction with actor, fiber, and scheduler semantics.
11. Whether async expression bodies are already permitted.
12. Whether `await` is an expression and how it is typed.
13. Constructor restrictions for asynchronous initialization.
14. ABI, bytecode, interpreter, and VM obligations.

---

## 11. Recorded recommendations summary

The following decisions are preserved for later reconciliation:

- `async f() -> T` should conceptually declare the awaited result `T`.
- Invocation should conceptually return `Task<T>`.
- Async fallthrough should complete successfully with `()`.
- A non-unit async result must return a value on every normal path.
- Awaiting `Task<T>` should produce `T`.
- `Task<out T>` should be covariant.
- Mutable completion should be separated into a producer such as `Promise<T>`.
- Failure and cancellation must remain distinct from `None` and `()`.
- `Task<Never>` is meaningful.
- Async metadata should be retained reflectively.
- Source-level async callable types may normalize to task-returning invocation types.

None of these provisions override existing Phalcom async specifications until an explicit reconciliation document ratifies the final model.
