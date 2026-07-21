# E009 · Explicit `return` in a fiber entry block always raises `DeadFrameError`

- **Status:** OPEN — confirmed 2026-07-20 (reproduced under `target/debug/phalcom`, isolated by control)
- **Severity:** **major** — the most natural early-exit form (`if (c) { return x }`) crashes the run on first `.call()`, with a message that never mentions fibers
- **Subsystem:** fibers × block non-local return (home-frame tokens)
- **Related:** ADR-0013 (non-local return via frame token), ADR-0030 §6 / `concurrency.md:280-282` (cross-fiber `return` rule — the *general* rule this is a defensible-but-hostile instance of)

## Defect

A fiber's entry block keeps the `home_frame_token` of its **creation site**:
`fiber_resume` copies the block literal's original lexical home into the fresh entry frame
(`phalcom-core/src/primitive/fiber.rs:301-337`, `frame.home_frame_token = home_frame_token`).
That home frame lives on the fiber that *evaluated* `Fiber.new { … }` — a different fiber
from the one running the entry. `return` compiles to `ReturnNonLocal`, whose generation
check therefore always fails across the boundary → `DeadFrameError`
(`heap/fiber.rs:69-72` documents exactly this as the D4 consequence).

Net effect: falling off the end of an entry completes the fiber; writing `return x` at the
entry's own top level never does. No nesting, no yield, no await required.

## Repro (observed 2026-07-20)

```phalcom
const f = Fiber.new {
  System.print("body")
  return "done"
}
System.print(f.call())
```

Output: `body`, then `Traceback … non-local return from a block whose home method frame is
no longer alive (DeadFrameError)` pointing at the `return` line.

## Control

Tail-expression completion works: `Fiber.new { "tail-done" }` → `.call()` returns
`tail-done`. Every positive fixture in `tests/lang/concurrency/` uses only this form; no
fixture pins the explicit-`return` behavior in either direction.

## Fix direction (unverified — and this one is a *decision*, not just a repair)

1. **Re-anchor**: give the entry frame a fresh home token pointing at itself, making
   top-level `return` an ordinary frame-local completion, symmetric with tail-expression.
   Wren behaves this way (a fiber body's `return` completes the fiber). Must not weaken
   the *nested*-block case — a block created inside the fiber and sent elsewhere still
   needs the cross-fiber `DeadFrameError`.
2. **Document + diagnose**: keep the semantics, add the restriction to `concurrency.md`'s
   interface table, and special-case the diagnostic ("`return` cannot cross a fiber
   boundary; use a tail expression") — plus a negative fixture.

Either way a fixture is owed; today a change to `home_frame_token` assignment would flip
this behavior with zero test noise.
