# E003 · `System.schedule` pump resumes an arity-1 entry with zero args, failing the run

- **Status:** OPEN — confirmed 2026-07-19 (reproduced under `target/debug/phalcom`)
- **Severity:** minor — fails the run *after* the program's own code succeeded
- **Subsystem:** fibers / root-drive scheduler pump

## Defect

The root-drive pump (`phalcom-core/src/vm/dispatch.rs:261-264`) resumes a
scheduled fiber with `args = []`. A block entry that takes a parameter hits the
first-resume arity check (`phalcom-core/src/primitive/fiber.rs:278-284`) and the
`?` propagates an `Err` out of `run()` — after the program's own code already
succeeded. Remaining queued fibers never run.

## Reproduction

```phalcom
System.schedule { x => System.print(x) }
System.print("main done")   // prints "main done", then the run terminates with an arity error from the pump
```

## Fix direction (NOT verified)

The pump should deliver a `None`-padded / zero-arg resume, or skip-with-
diagnostic, rather than propagating `Err` out of `run`. Forward-looking risk for
U-FUTURE waiter resumption.
