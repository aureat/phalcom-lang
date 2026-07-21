# PDR-0008 — Every failed cell reports, and reporting happens before unwinding

- Status: Accepted
- Date: 2026-07-20
- Related: [U-REPL impl/02-session-and-cells.md §51-52](../forge/units/U-REPL/impl/02-session-and-cells.md)
  (the `CellOutcome::Failed` contract this record makes true),
  [U-REPL surface.md §S4](../forge/units/U-REPL/surface.md) (value echo must degrade, never
  fail the cell), [ADR-0013](../adr/accepted/0013-closure-upvalues-and-frame-token-return.md)
  (open→closed upvalues — why cell unwinding closes them first),
  [PDR-0007](0007-bounded-call-depth-and-native-reentrancy.md) (the recursion case `catch_unwind`
  cannot catch)

## Context

U-REPL specifies, at `impl/02-session-and-cells.md:51-52`, that `CellOutcome::Failed` means
*"Compile or runtime failure; the diagnostic has already been printed."* The doc comment on the
variant in `phalcom-repl/src/repl.rs:16` repeats it.

It is false on two of the three paths, and degraded on the third. Verified 2026-07-20 at
`c346200`:

| Path | Site | Actual behaviour |
|---|---|---|
| Parse | `phalcom-repl/src/repl.rs:87` | `Err(_) => return CellOutcome::Failed` — diagnostic discarded |
| Compile | `phalcom-core/src/vm/dispatch.rs:156` | `pub fn compiler_error(&mut self, err: PhError) {}` — **empty body**, doc-labelled *"Placeholder for compiler-error reporting (currently a no-op)"* |
| Runtime | `phalcom-core/src/vm/api.rs:191-193` | prints, but with an **empty traceback** |

The caller compounds it: `phalcom-repl/src/main.rs:203` is
`CellOutcome::Unit | CellOutcome::Failed => {}`. Nothing downstream prints either. A syntax
error in the REPL produces a blank prompt.

The runtime case is the instructive one. `run_cell` is:

```rust
let res = self.run();
self.unwind_cell();
res
```

`unwind_cell` truncates frames to zero (correctly — ADR-0013's upvalues must close before the
stack is dropped). The caller then calls `runtime_error`, whose frame loop at
`vm/dispatch.rs:123` iterates `self.frames` — now empty. Every REPL traceback is empty by
construction. `vm/dispatch.rs:128-131` carries a careful comment about resolving REPL cell spans
against the right source, inside a loop that can never see a REPL frame.

Separately, value echo is specified to **send** `toString` and degrade to the class name if it
raises (`surface.md:78-81`). `repl.rs:38` calls `Value::to_string` — the native renderer, which
for a plain instance falls to `to_debug` with no dispatch — not `to_display_string`
(`value/render.rs:110`), the only method that sends. User `toString` overrides are therefore
dead in the REPL, and the `catch_unwind` wrapping the call guards a Rust panic on a path that
raises a `PhResult` instead.

## Decision

### 1. The `Failed` contract is honoured on all three paths

No `CellOutcome::Failed` may be returned without a diagnostic having been emitted. The parse
path prints its diagnostic; `compiler_error` is implemented rather than left a stub.

### 2. Reporting happens before unwinding, at the VM boundary

`run_cell` reports the runtime error itself, **before** calling `unwind_cell`, while the frames
that constitute the traceback still exist. Reporting does not move to the REPL, and
`unwind_cell` does not move after it.

This is the single change that makes REPL tracebacks non-empty, and it belongs in
`phalcom-core` because that is where the frames are.

### 3. `eval` prints what it owns; the loop prints nothing

`ReplSession::eval` emits parse and compile diagnostics — the failures it alone can see. The
input loop in `main.rs` prints only successful value echo. This matches the file-run path
(`interpret.rs`), so the two entry points report identically.

### 4. Value echo sends `toString`, and a failed echo unwinds

Echo calls `to_display_string`. On `Err`, it degrades to the class name and the cell **still
reports success**, per `surface.md:81`.

Echo stays in the input loop, *after* `eval` has returned `CellOutcome::Value`. This is what
structurally guarantees a raising `toString` cannot turn a successful cell into a failed one —
the outcome is already decided when the echo runs. Do not move echo inside `eval`.

After a failed echo send, `unwind_cell` runs again. It is idempotent, and skipping it leaves the
next cell executing on a dirty stack.

### 5. `catch_unwind` is deleted from the echo path

It guards the wrong failure mode. A raising `toString` returns `Err(PhError)`, which
`catch_unwind` never sees. An infinitely recursive `toString` overflows the Rust stack, which
Rust **aborts** rather than unwinds, so `catch_unwind` cannot catch that either — that case is
[PDR-0007](0007-bounded-call-depth-and-native-reentrancy.md) §1's native re-entrancy counter,
not a panic guard.

## Consequences

- Syntax errors, compile errors, and runtime errors all produce visible diagnostics.
- REPL tracebacks carry frames, which makes the span-resolution logic already written at
  `vm/dispatch.rs:128-131` reachable for the first time.
- User `toString` overrides take effect in the REPL, which is what `surface.md` always specified.
- `value_echo_survives_raising_tostring` stops being vacuous. As written it passes because
  `to_debug` emits the class name whether or not any guard exists — it would pass with the guard
  deleted. It must be rewritten to assert the override was *attempted*.
- §2 and §1's `compiler_error` touch `phalcom-core` (`vm/api.rs`, `vm/dispatch.rs`); the rest is
  confined to `phalcom-repl`. Sequence accordingly when other work holds those core files.

**The cost, named plainly:** §3 puts I/O in a library type. `ReplSession` writes to stdout/stderr
and cannot be embedded silently — an LSP or a test harness that wants to evaluate without
printing has no way to suppress it. That is a real limitation, accepted because consistency with
the file-run path is worth more today than embeddability nobody has asked for.

**What this precludes.** Silent embedding of `ReplSession`, until a reporter/sink is injected.
That change is additive and cheap when wanted — rustc's `DiagCtxt` is the shape — so this
forecloses nothing permanently. It does mean the first embedder pays for it.

That cost, the span-less compile diagnostic §1 settles for, and the two now-duplicated
reporting paths (`cmd_run` does not go through `interpret_source`) are recorded as items 6, 4
and 5 of
[`docs/deferred/repl-diagnostics-and-limits-followups.md`](../deferred/repl-diagnostics-and-limits-followups.md).

## Alternatives rejected

- **Return the diagnostic as data (`Failed(Diagnostic)`) and print in the loop.** Cleanest
  layering, trivially testable, and Roslyn's model. Rejected because it contradicts the
  `impl/02-session-and-cells.md:51` contract and diverges from the file-run path, leaving two
  reporting styles in one codebase for a benefit — embeddability — that has no consumer yet.
- **Inject a reporter/sink now.** rustc's `DiagCtxt`. Correct and more machinery than the
  current problem justifies. §3's cost paragraph records the door.
- **Report in the REPL after unwinding, reconstructing frames.** The frames are gone; there is
  nothing to reconstruct. This is what the code does today, and the empty traceback is the
  result.
- **Move value echo inside `eval` so it can be guarded centrally.** Rejected: it puts the echo
  *before* the outcome is decided, and a raising `toString` would then be able to fail the cell —
  precisely what `surface.md:81` forbids.
- **Keep `catch_unwind` as belt-and-braces.** It cannot catch either real failure mode (raise,
  or abort-on-overflow), so it buys nothing and implies a safety property that does not hold.
