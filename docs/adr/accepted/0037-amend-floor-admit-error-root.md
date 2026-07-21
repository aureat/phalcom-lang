# 37. Amend the frozen floor — admit `Error#message`/`Error#raise`

- Status: Accepted (code-confirmed 2026-07-14 — `Error.new().raise()` and
  `_message` wiring present throughout `core.ph`, per the U-CORE-6 comment)
- Date: 2026-07-12
- Related: [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) (frozen
  floor, amended here); [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md)
  (sibling omnibus amendment — pre-cleared `Error#message`/`raise` "in
  principle" for U-CORE-6 alongside `hash` (U-CORE-1), `Method` reflection
  (U-CORE-3), and `Number#toString` (U-CORE-4)); [ADR-0028](0028-amend-floor-admit-method-reflection.md)
  and [ADR-0036](0036-amend-floor-admit-number-tostring.md) (sibling per-unit
  landing records — same amendment pattern, 80 → 85 and 85 → 86 respectively);
  [ADR-0008](0008-layered-exceptions-and-result.md) (layered exceptions/
  `Result` design — the one-unwind-primitive model this amendment realizes
  the `Raise` half of); [ADR-0031](0031-error-handling-surface-syntax.md)
  (`throw`/`try`/`on`/`catch`/`ensure` surface spelling — confirms `throw expr
  === expr.raise()` targets this unit's mechanism); ADR-0012 (message-send
  dispatch / `doesNotUnderstand(_:)`, method-lookup.md §2 — the miss path this
  unit re-wires); `docs/forge/units/U-CORE-6/as-built.md` §2, §3, §6
  (drafted amendment text, concrete bodies, and the native/`.ph` split);
  `docs/spec/current/core/floor-census.md` §1.1, §2.15 (re-baselined in the same
  implementation change as this ADR)

## Context

[ADR-0019](0019-freeze-vm-blessed-primitive-floor.md) froze a permanent
VM-blessed primitive floor. [ADR-0023](0023-amend-floor-admit-hash-and-kernel-reflection.md)
admitted four capabilities to that floor **in principle** as a single omnibus
ratification — `hash` (U-CORE-1), `Method` reflection (U-CORE-3),
`Number#toString` (U-CORE-4), and `Error#message`/`raise` (U-CORE-6) — so none
of the four owning units would individually block on its own ADR round.
ADR-0023 was explicit that clearing the gate "in principle" is not the same as
landing: each unit's own floor-census update "applies its own delta *in
lockstep with its own primitive installs*."

U-CORE-1, U-CORE-3, and U-CORE-4 have since landed (73 → 80 → 85 → 86).
U-CORE-6 now lands the fourth and final slice of the ADR-0023 ceiling: the
minimal reification of the surface error hierarchy (object-model.md §4
"Errors", [ADR-0008](0008-layered-exceptions-and-result.md)). Per its as-built
specification (`docs/forge/units/U-CORE-6/as-built.md` §2), exactly **two**
capabilities in this unit's scope fail the ADR-0019 §1 derivability test:

| Capability | Why not `.ph` |
|---|---|
| `Error#message` | Reads a Rust-stamped fixed slot (`_message`, slot 0) never assigned in any `.ph` body — a `.ph` getter over an unassigned field trips the compiler's read-before-write check (`compiler/lib.rs`), the same shape `Message`'s accessors already solve natively (floor-census §2.14). |
| `Error#raise` | Initiates the VM's unified unwind (a new `RuntimeError::Raise` payload) — control-flow below the `.ph` boundary, not expressible as an ordinary send. |

Both were already the deliberate architect call in the as-built spec (its §6
D2): a `.ph` `message { return _message }` getter is rejected by the same
read-before-write check that rules out a naive `.ph` `Message` accessor; a
`.ph` `raise` cannot itself produce a Rust-level unwind payload. This trades a
strict-minimal floor (a hypothetical +1, `raise` alone, with `message` derived
some other way) for the robustness of mirroring the already-proven `Message`
pattern — both are ADR-0019 amendments regardless.

**No other error-hierarchy capability is admitted by this amendment.** Per the
as-built spec's explicit scope fence (§0 "Explicitly OUT of scope") and
U-CORE-3's hand-off (`docs/forge/units/U-CORE-3/as-built.md` §0.2), the native
`RuntimeError::Arity`/`Type`/`ZeroDivision`/`DeadFrameError`/etc. variants stay
**native** through this unit — reifying them into surface
`ArgumentError`/`TypeError`/`RangeError`/`DeadFrameError` classes is reserved
for a later error-reification unit, once `Error` exists as a root to hang them
off. `Result`/`Ok`/`Err` and the `on(_)`/`ensure` handling protocol (now
spelled by [ADR-0031](0031-error-handling-surface-syntax.md)) are likewise
reserved, not built here.

## Decision

Amend [ADR-0019](0019-freeze-vm-blessed-primitive-floor.md)'s floor list to
add **two** new floor bindings, both on the new kernel `Error` class:

1. **`Error#message`** — a getter reading the receiver's `_message` slot
   (slot 0), surfaced through the standard absence boundary (`None` if
   unset). Native slot accessor (`error_message`), mirroring
   `Message`'s native accessors (floor-census §2.14) rather than a `.ph`
   getter, for the read-before-write reason above.
2. **`Error#raise`** (`raise()`, zero-arity) — initiates the unified unwind's
   `Raise` payload (`RuntimeError::Raise { error, rendered }`, the sibling of
   U10's `Return`/`Bytecode::ReturnNonLocal` payload under
   [ADR-0008](0008-layered-exceptions-and-result.md)'s single-unwind-primitive
   model). `throw expr` desugars to exactly this send
   ([ADR-0031](0031-error-handling-surface-syntax.md) §1). Installed on
   `Error` only, so a non-`Error` receiver has no `raise` (a future `throw 42`
   misses → `doesNotUnderstand(_:)` → `MessageNotUnderstood`, the runtime half
   of R-INV-6.3; the compile-time rejection of `throw 42` is the error-syntax
   unit's job, not this amendment's).

**The dNU miss path is re-pointed, not newly bound.** The existing
`(Object, doesNotUnderstand)` binding (`object_does_not_understand`) is
unchanged as an installed binding — its *body* now builds a surface
`MessageNotUnderstood` instance and raises it via `RuntimeError::Raise`,
rather than constructing the retired native `RuntimeError::MessageNotUnderstood`
variant. This contributes no new binding (the `(Object, doesNotUnderstand)`
pair already existed in the ADR-0019 floor) — only a body substitution behind
it, the same shape ADR-0036 used for `Object#toString`'s re-home.

**Producing `RuntimeError::Raise` is plumbing, not a bound selector.** The new
enum variant is a Rust-internal unwind payload, not itself a `(class,
selector)` pair `install_primitives` binds — it does not count toward either
metric (ADR-0023 Decision §4 makes this explicit).

**Floor count.** This unit moves the census **86 → 88** (+2 bindings),
completing the ADR-0023 ceiling across all four amendment units — `hash` at
80, `Method` reflection at 85, `Number#toString` at 86, `Error#message`/
`raise` here at 88. Distinct native Rust functions move **71 → 73** (+2:
`error_message`, `error_raise` — both wholly new fns, no rehome subtlety).
Floor-carrying classes move **16 → 17**: `Error` is the only one of the two
new kernel classes to carry a primitive (`MessageNotUnderstood` inherits both
`message` and `raise` from `Error`, carrying none of its own).
`floor-census.md` §1.1 and the new §2.15 are re-baselined to 88 in the same
implementation change that lands this surface (R-INV-0.1/R-INV-6.5) — not by
this document alone.

## Consequences

- Clears the concrete, auditable amendment for U-CORE-6's slice of the
  ADR-0023 ceiling: the floor list in ADR-0019 now names
  `Error#message`/`Error#raise` explicitly, rather than resting on ADR-0023's
  in-principle admission alone. This closes the ADR-0023 ledger: all four
  named capabilities (`hash`, `Method` reflection, `Number#toString`,
  `Error#message`/`raise`) are now individually amended and landed.
- A genuine `doesNotUnderstand(_:)` miss now propagates a **catchable surface
  value** (`isA(Error)`) through the ordinary `PhResult`/`?` channel, instead
  of a native Rust error string — the load-bearing precondition a later
  `on(_)`/`ensure`/fiber-result-slot consumer needs (forward-compat §1/§2).
  Uncaught, the render and exit-code behavior are unchanged (the `Raise`
  variant's `#[error("{rendered}")]` reproduces the retired variant's display
  string byte-for-byte).
- The permanent native surface grows by 2 bindings (86 → 88), reaching exactly
  the "at most 88" ceiling ADR-0023 authorized; nothing here exceeds it.
- Unblocks the later error-reification unit (re-pointing native
  `RuntimeError::Arity`/`Type` at surface `ArgumentError`/`TypeError`) and the
  later error-syntax unit (`throw`/`try`/`on`/`catch`/`ensure`,
  [ADR-0031](0031-error-handling-surface-syntax.md)): both now have a real
  `Error` root and a real `Raise` unwind payload to build against, rather than
  a native-string placeholder.

## Alternatives considered

- **Treat ADR-0023's in-principle clearance as sufficient; land no further
  document.** Rejected, for the same reason ADR-0028/ADR-0036 rejected it: a
  per-unit landing record keeps the floor list in ADR-0019 an accurate,
  citable statement of what is *actually* native today.
- **A `.ph` `message` getter over `_message`, keeping only `raise` native
  (+1 instead of +2).** Rejected (as-built §6 D2): trips the compiler's
  read-before-write check, since the reopened `Error`/`MessageNotUnderstood`
  bodies never *assign* `_message` in `.ph` — only the Rust miss-path writes
  it. Mirrors why `Message`'s four accessors are already native.
- **A bespoke `PhError::Raise(Value)` variant at the top level, instead of
  `RuntimeError::Raise` nested inside `PhError::Runtime`.** Rejected for this
  landing: the nested form needs zero edits to `run_file`'s exit-code match or
  `runtime_error`'s render path (both already handle `PhError::Runtime(_)`
  generically), while a top-level variant would force edits to both for no
  behavioral gain at this unit's scope. The load-bearing requirement — `error`
  is the raw catchable `Value`, `rendered` is a display cache only — holds
  either way; revisiting the payload's placement is available to a later unit
  if `on`/fiber ergonomics need it.
