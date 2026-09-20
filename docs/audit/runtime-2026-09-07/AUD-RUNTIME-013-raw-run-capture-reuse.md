# AUD-RUNTIME-013 - Raw module rerun can retain stale open captures

## Classification

- Severity: Medium
- Category: VM execution entry / upvalue lifecycle
- Confidence: Source-confirmed hazard; runtime sequence pending.

## Observed

`VM::run_in_module` clears frames and stack directly, while open upvalues use
absolute stack indices and are closed safely only by `unwind_to` before stack
truncation. A failed raw run followed by a second run can therefore leave an
escaped capture associated with discarded stack storage.

## Reproducer and repair direction

Create an escaped closure during a failing raw run, execute a second raw run
with slot reuse, then invoke the closure and assert the original value. The
canonical repair is `run_in_module` entry cleanup via close-before-truncate
unwind, while preserving failed frames long enough for traceback reporting.
