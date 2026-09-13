# System

Part of the Phalcom Language Specification. Status: Draft 0.1.

`System` is the runtime service surface through which Phalcom code reaches
console, clock, garbage collection, process environment, scheduler/executor
services, and reactor timers.

Design rule: effects are named, not ambient.

## 1. Structure

`System` is a stateless class-side service namespace:

- no user-facing instance constructor;
- services are class-side;
- returned values are ordinary Phalcom values;
- mutable runtime state stays behind VM-owned mechanisms.

## 2. Interface

### Console

| Signature | Meaning |
|---|---|
| `print(_)` | print plus newline; returns Unit |
| `write(_)` | write without newline |
| `printErr(_)` | stderr |
| `readLine` | `Option<String>` |

### Time

| Signature | Meaning |
|---|---|
| `clock` | the process-wide monotonic `Clock` |
| `now` | wall-clock epoch seconds as Float |

### Process and environment

The full process contract is owned by `stdlib/process.md` subject to its PDR status.

### Runtime

| Signature | Meaning |
|---|---|
| `gc` | request full GC; returns Unit |
| `version` | runtime version string |

### Scheduler / executor

| Signature | Meaning |
|---|---|
| `schedule(_)` | admit fresh work for a later executor turn and return a Fiber handle |
| `runScheduled` | synchronous library request to drive schedulable work according to executor semantics |
| `sleep(_ milliseconds: Int)` | `Future<Unit>` completing no earlier than a monotonic deadline; reactor-owned |

Current main has the ready queue/scheduling surface and the reactor timer
implementation. `Duration`-typed sleep migration remains part of the standard
library/concurrency follow-up.

CONC002.C2.P2 owns the implementation transition from scheduler-as-coroutine-resumer driving to a VM-owned executor. The selector-level scheduler surface remains.

`System.sleep` is governed by PDR-0004 and `stdlib/reactor.md`; implementation owner is CONC002.C3.P1.

## 3. Implementation ownership

Native System services live in:

```text
phalcom-core/src/primitive/system.rs
```

and execution/scheduler state in `phalcom-core/src/vm/`.

Current scheduler hardening/executor semantics are owned by CONC002 C1/C2.

Reactor/timer implementation is owned by:

```text
docs/implementation/CONC002-concurrency-control-and-failure-observability/
  C3-reactor-and-external-completion/
```

The exact internal native timer-registration selector is finalized by C3.P1 against the native-floor census.

The older proposal requiring source-visible `nextCompletion` / `parkForCompletion` guest pump seams is not normative.

Potentially blocking filesystem/network/process operations consume the reactor through their own host-surface specifications rather than being implemented as System methods.

See `concurrency.md` for Fiber/Future/executor semantics and `stdlib/reactor.md` for external completion machinery.

Wall-clock epoch time remains separate from the monotonic clock. `Timestamp`,
`Date`, `DateTime`, and `TimeZone` are deferred until a civil-time design is
ratified.
