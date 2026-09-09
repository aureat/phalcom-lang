# E010 · Scheduler pumps swallow captured task errors; `await`'s quiescence diagnostic masks the real cause

- **Status:** FIXED — implemented and focused-tested 2026-09-09
- **Severity:** **major** — a diagnostics defect, not a wrong value: the terminal error the user sees actively points away from the root cause
- **Subsystem:** core library (`Future#await` root pump, `System.runScheduled`) × error observability
- **Related:** [E008](E008-double-schedule-kills-run.md) (the two channels again — here the *captured* channel is the lossy one); capture-not-propagate contract at `core.ph:1467-1469`

## Defect

Both pumps dequeue scheduled fibers and resume them through the internal
`Fiber._$resumeScheduled()` scheduler path (`phalcom-core/core/universe/src/concurrency/fiber.ph`
`System.runScheduled`; `phalcom-core/src/vm/dispatch.rs` root-drive pump). The
scheduler mode isolates one task's failure so it cannot abort sibling work, but
the captured `Error` is then reachable *nowhere*: no hook, no log, no aggregation.
A fire-and-forget task that fails was indistinguishable from one that succeeded.

The sharp edge is `await`'s composition with it. If the task that was supposed to settle
the future fails, the pump swallows the failure, the queue drains, and `await` raises its
quiescence diagnostic:

```
await: the future is still pending and the scheduler is empty; nothing can settle it
```

— which is true, but named the *symptom* while the swallowed `Error` named the
*cause*.

## Repro (observed 2026-07-20)

```phalcom
const fut = Future.new()
System.schedule { fut.complete(42) }   // typo: the selector is settleValue
System.print(fut.await)
```

Before the fix, only the quiescence traceback appeared. The actual failure —
`Future does not understand 'complete(_)'` — appeared nowhere. One misspelled selector
in a completer task and the reported error was about scheduler emptiness.

(Found the honest way: this audit's own first await probe used `complete` and burned
fifteen minutes on the decoy.)

## Resolution

The VM now records scheduler-owned terminal failures that have no durable completion
observer. Reporting is deferred until a safe scheduler boundary, so sibling work still
runs and the scheduler resume path remains capture-not-propagate:

- `System.runScheduled` reports each detached failure once after draining the queue.
- The native root-drive pump reports detached failures before returning the root value.
- Root `Future.await` consumes failures from its own drive window and adds them to the
  quiescence diagnostic without claiming causal ownership of the Future.
- Future-owned action and continuation failures remain owned by their Future and are not
  double-reported.
- The failure inbox roots both the terminal Fiber and captured Error through GC.

The permanent regressions are the E010 scheduler, multiple-failure, Future-owned,
await-window, observer-failure, and GC-retention tests under
`phalcom-core/tests/fixtures/language/concurrency/` and the VM unit tests.

The root-await regression is intentionally a top-level drive. Running it beneath the
language `try`/`on` native `block_on` frame still raises
`CannotYieldAcrossNativeFrame`; that existing safety rule remains unchanged.
