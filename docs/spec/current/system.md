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
- the live values it returns (a `Float` clock reading, a `String` line of input)
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
| `clock` | monotonic seconds as a `Float`, for measuring durations |
| `now` | wall-clock epoch seconds as a `Float` |

### Process & environment

> **Specced 2026-07-20:** this group is promoted to
> [`stdlib/process.md`](stdlib/process.md) (normative upon
> [PDR-0019](../../pdr/0019-process-and-environment-surface.md) ratification),
> which keeps these spellings, adds the exact-bytes siblings
> (`envBytes(_)`/`argsBytes` — lossy/exact split, PDR-0013 ruling 4 pattern), rules
> the environment **read-only**, and binds `exit(_)` to run reactor shutdown +
> resource drain + leak report first. The rows below remain the quick reference.

| Signature | Meaning |
|-----------|---------|
| `args` | the program's argument vector as a `List<String>` (lossy display form; `argsBytes` is exact) |
| `env(_)` | an environment variable as `Option<String>` (lossy; `envBytes(_)` is exact) |
| `exit(_)` | terminate the process with an integer status `0..255`, after the shutdown obligations |

### Runtime

| Signature | Meaning |
|-----------|---------|
| `gc` | request a garbage collection; returns `None` ([Values & Absence](values-and-absence.md)) |
| `version` | the runtime version `String` |

### Scheduler (with [Futures](concurrency.md))

| Signature | Meaning |
|-----------|---------|
| `schedule(_)` | enqueue a `Function` to run on a fresh fiber at the next scheduler turn — **landed** (U-SCHED, floor-census.md amendment): wraps `args[0]` as a fresh `Fiber` via the same validation `Fiber.new(_)` uses and pushes it onto the native ready-queue (`VM::ready_queue`); returns the `Fiber` handle, does not run it |
| `nextScheduled` | **landed** (U-SCHED, not originally in this table — a floor amendment in the same vein as ADR-0037/0038/0039/0049): pops and returns the next queued fiber as `Option<Fiber>`, `None` once the queue is empty; the drain seam every pump (native root-drive, `.ph` `runScheduled`) bottoms out in |
| `runScheduled` | **landed** (U-SCHED, `.ph`, `core.ph`): `while (next.isSome) { … }` pump over `nextScheduled` — drains everything queued so far, in order, including work newly scheduled mid-drain, then returns; the mid-program counterpart to the native root-drive below |
| `sleep(_)` | return a `Future` that settles after the given integral **milliseconds** — **ruled** ([PDR-0004](../../pdr/0004-io-is-future-shaped-reactor-owned.md) §5 closed the question this row left open; normative contract [`stdlib/reactor.md`](stdlib/reactor.md) §6: monotonic, `>=`-bounded). Unbuilt — lands with U-REACTOR ([`impl/reactor.md`](impl/reactor.md)) |

`VM::run`'s **root-drive pump** (`vm/dispatch.rs`) is the belt-and-suspenders
counterpart to `runScheduled`: once the top-level program's own activation
ends, it drains `VM::ready_queue` to exhaustion — via `fiber_try` (capture,
not propagate) — even if `main` never explicitly calls `runScheduled`, so a
scheduled task's side effect is never silently dropped at program exit.

`print(_)` returning its argument makes `System.print(x)` usable as a
pass-through in an expression position, consistent with everything being an
expression ([Classes §4](classes.md)).

---

## 3. Implementation

Present in an embryonic form
([`primitive/system.rs`](../../../phalcom-core/src/primitive/system.rs)):
`system_class_print` and `system_class_new` exist; `System` is registered as a
class in the [universe](../../../phalcom-core/src/universe.rs) bootstrap.

Each service is a `PrimitiveFn`
([`method.rs`](../../../phalcom-core/src/method.rs)) installed **on the metaclass**
(`System class`), since the calls are class-side. A primitive receives
`(&mut VM, receiver, args)` and returns a `PhResult<Value>`, so it can touch VM
state (the scheduler queue, the interner) directly — this is why `schedule`
belongs here rather than in Phalcom code (`system_schedule`/
`system_next_scheduled`, [`primitive/system.rs`](../../../phalcom-core/src/primitive/system.rs);
`VM::ready_queue`, [`vm/mod.rs`](../../../phalcom-core/src/vm/mod.rs)).
`runScheduled` itself is pure `.ph` orchestration over `nextScheduled` — no VM
state touched directly, so it lives in `core.ph` rather than native
([U-SCHED](../../forge/units/U-SCHED-FIBER/U-SCHED/plan.md)).

To reach the specified surface from today's tree:

1. install `write`, `printErr`, `readLine`, `clock`, `now`, `args`, `env`,
   `exit`, `gc`, `version` as primitives alongside `print`;
2. give the VM a monotonic clock handle and (for `readLine`) buffered stdin;
3. `schedule`/`nextScheduled`/`runScheduled` are landed (U-SCHED); `sleep` is
   **ruled and specced** ([`stdlib/reactor.md`](stdlib/reactor.md) §6 — fairness has a
   proposed default there, Q-R1) and lands with U-REACTOR; the process/environment
   rows land under [`stdlib/process.md`](stdlib/process.md) once PDR-0019 ratifies.

Because `System` is the only sanctioned effect surface, a sandboxed or test
embedding swaps the `System class` method dictionary for stubs and leaves the rest
of the language untouched — no other class performs I/O.

---

See [Fibers & Futures](concurrency.md) for the scheduler `System` drives, and
[Implementation Status](implementation-status.md) for the current gap.
