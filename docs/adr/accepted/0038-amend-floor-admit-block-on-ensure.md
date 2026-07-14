# 38. Amend the frozen floor — admit `Block#on`/`Block#ensure` (error handling)

- Status: Accepted
- Date: 2026-07-12
- Related: [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (the frozen floor);
  [ADR-0031](0031-error-handling-surface-syntax.md) (the ratified error surface);
  [ADR-0008](0008-layered-exceptions-and-result.md) (error model — handling is a `Block` protocol);
  [ADR-0037](0037-amend-floor-admit-error-root.md) (the **raise** side — `Error#message`/`raise`);
  [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md) (amendment precedent);
  [`docs/forge/units/U-ERR/plan.md`](../forge/units/U-ERR/plan.md) (DEC-ERR-A)

## Context

[ADR-0031](0031-error-handling-surface-syntax.md) ratified `try`/`catch`/`on`/`ensure`
as **1:1 sugar over the `Block` protocol** `blk.on(ErrorClass){ e => … }` /
`blk.ensure{ … }` ([ADR-0008](0008-layered-exceptions-and-result.md)). But those two
`Block` primitives **do not exist**, and they **cannot be written in `.ph`**: a `.ph`
body cannot observe a Rust-level `Raise` unwind, nor the in-place non-local-return
frame-shrink it must also honor. They therefore fail the
[ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) §1 derivability test, so
admitting them is a **floor amendment**, not an ordinary commit.

[ADR-0037](0037-amend-floor-admit-error-root.md) admits the **raise** side
(`Error#message`/`raise`) for U-CORE-6's minimal reification. This ADR admits the
**catch** side for the **U-ERR** unit — the later error-surface unit
[decisions.md Q2](../spec/v0.2/core/decisions.md) reserved. `U-ERR/plan.md` DEC-ERR-A
flags it as the one gate that must clear before the primitives merge.

## Decision

**Amend [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) to admit exactly two
native primitives** (`floor-census.md` **+2**; drafted at 80→82, landed at **109 → 111**
after intervening floor bumps — U-COLLTYPES etc.):

- **`Block#on(_)(_)`** — run the receiver block; if a `Raise` whose `Error` `isA` the
  given class propagates, `close_upvalues_from` the pre-run mark **then** truncate the
  value/frame stacks to the pre-run snapshot, and run the handler block with the caught
  `Error`; otherwise return the receiver's value. Handlers chain — first matching class
  wins; a non-matching `Raise` keeps unwinding.
- **`Block#ensure(_)`** — run the receiver block, then run the cleanup block on **any**
  exit path — normal fallthrough, `throw`/`Raise`, non-local `return`, or fiber
  `abort` — and propagate the original outcome unchanged.

Everything else in the error surface is **`.ph` or parser sugar over these two**: the
`try`/`on`/`catch`/`ensure` statement (parser desugaring, ADR-0031), `{ … }.attempt()`
→ `Result`, and `Result`/`Ok`/`Err` + the `unwrap`/`okOr`/`ok` bridges (pure `.ph` —
U7 `construct`+fields have landed, so they need **zero** floor). Net floor delta = **+2**.

## Consequences

- **Unblocks U-ERR.** The catch machinery has a ratified floor once this is Accepted;
  the parser sugar, `attempt`, and the whole `Result` channel ride on top with no
  further floor.
- **Load-bearing borrow rule.** `on` must `close_upvalues_from` **before** truncating
  (a missed close is a use-after-free — the U-ERR risk); `ensure` must fire on the
  non-local-return case (which surfaces as `Ok` + shrunk frames), not only on `Raise`.
- **Fiber-safe by construction.** The snapshot/restore is **length-relative**, so it
  stays fiber-local when U-FIBER lands ([ADR-0030](0030-fibers-and-futures-cooperative-concurrency.md)
  D7 — the unwind floor can be a *fiber* floor).
- `floor-census.md` (+2; landed 109 → 111) must be updated in the same change that installs
  the primitives (R-INV-0.1).

## Alternatives considered

- **Implement `on`/`ensure` in `.ph`.** Impossible — a `.ph` body cannot see the VM
  `Raise` payload or the frame-shrink of a non-local return; this is the exact
  derivability failure ADR-0019 §1 draws the line at.
- **One combined `onEnsure` primitive.** Rejected — `on` (typed, first-match, catches)
  and `ensure` (untyped, always-fires, does not catch) are distinct protocols with
  different firing rules; fusing them muddies both.
- **A VM-global handler stack instead of a `Block` protocol.** Rejected — ADR-0008
  fixes handling as a message-based `Block` protocol; a parallel global mechanism would
  be the "second error channel" its hazards forbid.
