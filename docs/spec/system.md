# System

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

`System` is the runtime's service surface: the single, well-known object through
which Phalcom code reaches the outside world — the console, the clock, the garbage
collector, process environment, and the concurrency scheduler
([Fibers & Futures](concurrency.md)).

Design rule: **effects are named, not ambient.** There is no free-floating
`print`; you send `print(_)` to `System`. Confining side effects to one receiver
keeps the object model pure (everything else is value-in, value-out) and gives one
obvious place to stub or sandbox the environment.

---

## 1. Structure

`System` is a **stateless singleton namespace**, not a data type:

- it has **no instance fields** and no user-facing constructor — `System.new` is
  not part of the surface protocol;
- every service is a **class-side** method (`static`), so `System` is used purely
  as a receiver of class messages, exactly like a module of free functions;
- the live values it returns (a `Number` clock reading, a `String` line of input)
  are ordinary objects — `System` itself never appears inside them.

In the [Object Model](object-model.md) catalog `System` is a `U` class whose sole
instance, if any, is irrelevant: all protocol lives on `System class`.

---

## 2. Interface

All class-side. Grouped by service.

### Console

| Signature | Meaning |
|-----------|---------|
| `print(_)` | write `x.toString` followed by a newline to standard output; returns `x` |
| `write(_)` | write `x.toString` with no trailing newline |
| `printErr(_)` | write to standard error |
| `readLine` | read one line from standard input as `Option<String>` (`None` at EOF) |

### Time

| Signature | Meaning |
|-----------|---------|
| `clock` | monotonic seconds as a `Number`, for measuring durations |
| `now` | wall-clock epoch seconds as a `Number` |

### Process & environment

| Signature | Meaning |
|-----------|---------|
| `args` | the program's argument vector as a `List<String>` |
| `env(_)` | an environment variable as `Option<String>` |
| `exit(_)` | terminate the process with an integer status |

### Runtime

| Signature | Meaning |
|-----------|---------|
| `gc` | request a garbage collection; returns `nil`'s surface substitute — settle on the unit convention in [Values & Absence](values-and-absence.md) |
| `version` | the runtime version `String` |

### Scheduler (with [Futures](concurrency.md))

| Signature | Meaning |
|-----------|---------|
| `schedule(_)` | enqueue a `Function` to run on a fresh fiber at the next scheduler turn |
| `sleep(_)` | return a `Future` that settles after N seconds (a timer completion source) |

`print(_)` returning its argument makes `System.print(x)` usable as a
pass-through in an expression position, consistent with everything being an
expression ([Classes §4](classes.md)).

---

## 3. Implementation

Present in an embryonic form
([`primitive/system.rs`](../../phalcom-core/src/primitive/system.rs)):
`system_class_print` and `system_class_new` exist; `System` is registered as a
class in the [universe](../../phalcom-core/src/universe.rs) bootstrap.

Each service is a `PrimitiveFn`
([`method.rs`](../../phalcom-core/src/method.rs)) installed **on the metaclass**
(`System class`), since the calls are class-side. A primitive receives
`(&mut VM, receiver, args)` and returns a `PhResult<Value>`, so it can touch VM
state (the scheduler queue, the interner) directly — this is why `schedule`/`sleep`
belong here rather than in Phalcom code.

To reach the specified surface from today's tree:

1. install `write`, `printErr`, `readLine`, `clock`, `now`, `args`, `env`,
   `exit`, `gc`, `version` as primitives alongside `print`;
2. give the VM a monotonic clock handle and (for `readLine`) buffered stdin;
3. add `schedule`/`sleep` once the [scheduler](concurrency.md) exists — `sleep`
   registers a timer whose firing settles the returned `Future`.

Because `System` is the only sanctioned effect surface, a sandboxed or test
embedding swaps the `System class` method dictionary for stubs and leaves the rest
of the language untouched — no other class performs I/O.

---

See [Fibers & Futures](concurrency.md) for the scheduler `System` drives, and
[Implementation Status](implementation-status.md) for the current gap.
