# E010 · Scheduler pumps swallow captured task errors; `await`'s quiescence diagnostic masks the real cause

- **Status:** OPEN — confirmed 2026-07-20 (reproduced under `target/debug/phalcom`)
- **Severity:** **major** — a diagnostics defect, not a wrong value: the terminal error the user sees actively points away from the root cause
- **Subsystem:** core library (`Future#await` root pump, `System.runScheduled`) × error observability
- **Related:** [E008](E008-double-schedule-kills-run.md) (the two channels again — here the *captured* channel is the lossy one); capture-not-propagate contract at `core.ph:1467-1469`

## Defect

Both pumps resume scheduled fibers with `f.try()` and **discard the result**
(`phalcom-core/core/core.ph:1478-1485` `runScheduled`; `core.ph:1621-1628` `await`'s
root-drive branch; native pump `phalcom-core/src/vm/dispatch.rs:296-299`). Capture-not-
propagate is the design — one task's failure must not abort its siblings — but the
captured `Error` is then reachable *nowhere*: no hook, no log, no aggregation. A
fire-and-forget task that fails is indistinguishable from one that succeeded.

The sharp edge is `await`'s composition with it. If the task that was supposed to settle
the future fails, the pump swallows the failure, the queue drains, and `await` raises its
quiescence diagnostic:

```
await: the future is still pending and the scheduler is empty; nothing can settle it
```

— which is true, but names the *symptom* while the swallowed `Error` named the *cause*.

## Repro (observed 2026-07-20)

```phalcom
const fut = Future.new()
System.schedule { fut.complete(42) }   // typo: the selector is settleValue
System.print(fut.await)
```

Output: only the quiescence traceback above. The actual failure —
`Future does not understand 'complete(_)'` — appears nowhere. One misspelled selector in
a completer task and the reported error is about scheduler emptiness.

(Found the honest way: this audit's own first await probe used `complete` and burned
fifteen minutes on the decoy.)

## Fix direction (unverified)

Minimum: the quiescence raise carries the last captured task error ("… nothing can settle
it; 1 scheduled task failed: <Error>"). Better: pumps accumulate captured failures and
surface them — an unhandled-failure hook or an end-of-run report. Precedent is uniform:
JS `unhandledrejection`, Python asyncio "Task exception was never retrieved", BEAM crash
reports — every cooperative runtime grew one because silent capture is a debugging tax
everyone pays eventually.
