# Abandoned-fiber `ensure` + resource limits (proposed — untracked gaps)

- Status: Proposed · no open-Q covers ensure-on-drop; overlay flags limits "unspecified"
- Axis: concurrency (dropped coroutine) + security (resource exhaustion)
- Related: ADR-0008 (`ensure`), concurrency-adr.md (GC roots)

## Problem

1. **Dropped fiber.** ADR-0008 fires `ensure` on unwind — but a fiber suspended
   forever and then collected never unwinds. Do its `ensure`/`finally` blocks run?
   Lua never runs them (silent); Python runs them at GC via `GeneratorExit`.
2. **No caps.** Recursion depth and allocation are unbounded → a DoS surface and
   the fuzz/miri lanes have no defined pass criterion.

## Decision

- **Abandoned fibers do NOT run `ensure`.** A suspended fiber that becomes
  unreachable is collected *silently* — no resurrection, no `ensure` at GC. Any
  block whose home frame dies without unwinding simply never resumes. Cleanup that
  must be guaranteed belongs in the *resumer* (`try`/`ensure` around `fiber.call`),
  not in the abandoned fiber. Rationale: running user `ensure` code during GC
  re-enters the interpreter at an arbitrary point — unsound against the handle
  heap and cooperative scheduler. Matches Lua, not Python.
- **`Fiber.finish` (opt-in):** an explicit `f.finish` resumes a suspended fiber
  once with an `abort`, letting its `ensure` blocks run deterministically **on the
  caller's turn** — the safe way to get Python's behavior without GC re-entry.
- **Caps (configurable, defined errors):** max frame-stack depth → `StackOverflow`
  (an `Error`, never a Rust `panic!`); allocation ceiling per turn → `MemoryError`.
  Both are diagnostics, satisfying the "every panic on input is a robustness bug"
  posture.

## Precludes

- Guaranteed-cleanup semantics for silently-dropped fibers — you must use
  `finish` or resumer-side `ensure`. Accepted: the alternative (GC re-entering the
  VM) is unsound here.
- Unbounded recursion as a *feature* (deep non-TCO recursion now has a hard,
  diagnosable ceiling). Interacts with the still-open tail-call decision.
