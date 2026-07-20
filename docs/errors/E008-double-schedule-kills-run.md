# E008 · Scheduling the same fiber twice fails the whole run — the pre-flight refusal rides the failure channel

- **Status:** OPEN — confirmed 2026-07-20 (reproduced under `target/debug/phalcom`, isolated by control)
- **Severity:** **major** — program's own code succeeds, then the run dies with exit 1; contradicts the documented capture-not-propagate pump contract
- **Subsystem:** fibers / scheduler (`System.schedule`, root-drive pump)
- **Related:** [E003](E003-schedule-pump-arity.md) (same channel confusion: an error *about resuming* a scheduled fiber is indistinguishable from an error *raised by* it)

## Defect

`System.schedule(_)` (`phalcom-core/src/primitive/system.rs:56-63`) enqueues an
already-constructed `Fiber` with no dedup and no status check. After the first drain runs
the fiber to `Done`, the second drain pops the same `ObjRef` and calls `fiber_try`, whose
pre-flight guard (`phalcom-core/src/primitive/fiber.rs:283-291`) returns
`RuntimeError::NotAllowed("cannot resume a finished fiber")` **before** any entry code
runs. That `Err` never reaches the try-mode capture (which only captures failures of the
callee's own execution) — at the native root-drive pump it hits the `?` at
`phalcom-core/src/vm/dispatch.rs:298` and terminates `VM::run`; in the `.ph` pump
(`System.runScheduled`, `core.ph:1478-1485`) the bare `next.try()` raises uncaught into
the caller.

Both pumps promise the opposite: "capture-not-propagate, so one scheduled task's uncaught
raise cannot abort another" (`core.ph:1467-1469`,
`tests/lang/concurrency/concurrency_sched_raising_fiber_does_not_abort_host.ph`). The
promise holds for entry-body raises and breaks for resume refusals — two different error
channels sharing one `PhResult`.

## Repro (observed 2026-07-20)

```phalcom
const f = Fiber.new { System.print("ran") }
System.schedule(f)
System.schedule(f)
System.print("main-exits")
```

Output: `main-exits`, `ran`, then `Traceback … cannot resume a finished fiber`.

## Control

Two *distinct* fibers schedule and drain cleanly in FIFO order (`a`, `b` print after
`end`, no traceback).

## Fix direction (unverified)

Either end works; they differ in *where* the contract lives:

1. `System.schedule(_)` refuses (or no-ops on) a fiber that is already queued or
   `Done`/`Failed` — validation at the enqueue boundary, matching E003's lesson.
2. The pump treats a resume-refusal (`NotAllowed` from the pre-flight guard) as
   skip-and-continue, distinct from an entry failure.

(1) fails fast at the user's line; (2) makes the pump total. Doing only (2) leaves
double-schedule silently meaning "run once", which then needs a spec sentence.
