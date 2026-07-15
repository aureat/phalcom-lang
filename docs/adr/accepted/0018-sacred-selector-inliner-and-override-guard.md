# 18. Sacred-selector inliner with override-epoch deopt guard

- Status: Accepted
- Date: 2026-07-11
- Related: `docs/spec/v0.2/control-flow.md` §2–§3; [ADR-0004](0004-boolean-as-abstract-bool-with-true-false.md); [ADR-0006](0006-function-as-abstract-callable-root.md); [ADR-0012](0012-selector-signature-encoding-and-dispatch.md); [ADR-0013](0013-closure-upvalues-and-frame-token-return.md)

## Context

Control flow in Phalcom is not built into the grammar — `if`/`while` desugar to
ordinary message sends (`ifTrue(_:ifFalse:)`, `whileTrue(_:)`) over block literals,
and boolean `and`/`or` are lazy sends over a block argument
([control-flow.md §2](../../spec/v0.2/control-flow.md)). This keeps the object model
uniform (Bool and Block are real classes with overridable methods) but makes the
*hot path of every program* a closure allocation plus a call frame per branch and
per loop iteration. The spec is explicit that the inliner is **load-bearing**
([control-flow.md §3](../../spec/v0.2/control-flow.md), Invariant 5): "If blocks are slow,
users learn to avoid them and every other decision in the spec unravels."

This is the canonical **speculative inlining ⊗ late binding** hazard. Splicing the
block body inline assumes the sacred selector still resolves to the kernel method.
But any method — including `Bool>>ifTrue:` or `Block>>whileTrue:` — can be
redefined at runtime (class reopening). An unguarded inline is therefore *unsound*:
it would silently ignore a user override.

## Decision

The compiler recognizes a **sacred selector** whose block arguments are **literal
blocks at the call site** and emits jump opcodes instead of a send, guarded by a
runtime type check plus an override epoch:

- New opcodes `Jump`, `JumpIfFalse`, `Loop` (backward), `GuardBool`, `GuardBlock`
  ([bytecode.rs](../../phalcom-core/src/bytecode.rs)). The inliner splices the block
  bodies directly into the enclosing function's bytecode — **zero closure
  allocation, zero call frames** on the common path.
- **Receiver guard.** `GuardBool`/`GuardBlock` verify the receiver's type before the
  inline body runs. A wrong-typed receiver (e.g. `5.ifTrue(...)`) fails the guard and
  **deoptimizes to a real message send**, which then resolves (or cleanly fails to
  resolve) through normal dispatch.
- **Override epoch.** `Universe` carries `bool_sacred_pristine` / `block_sacred_pristine`
  flags ([universe.rs](../../phalcom-core/src/universe.rs)). Installing a method whose
  selector is a sacred one (hooked in `note_method_installed` on `Bytecode::Method`)
  flips the corresponding flag. Once flipped, the inliner still emits the guarded
  fast path, but a redefined sacred method is honored because the deopt path is a
  real send — the override wins.
- Non-local `return` inside an inlined block body unwinds to the home method frame
  exactly as a non-inlined block would ([ADR-0013](0013-closure-upvalues-and-frame-token-return.md)):
  inlining changes the allocation strategy, not the return semantics.

## Consequences

- Blocks-as-control-flow is cheap on the common path, satisfying Invariant 5, while
  remaining fully overridable — the two goals the spec insisted could not be traded
  off against each other.
- A guard failure (wrong receiver type) is *observably identical* to the slow path:
  it deopts to the real send and produces the same result or the same "not found"
  error. Verified by `runtime_inline_guard_wrong_type` and
  `control_flow_inline_override_honored`.
- The override epoch is coarse (one flag per class family, not per selector). A
  redefinition of *any* sacred `Bool` method conservatively marks the whole `Bool`
  family non-pristine. This trades a small amount of missed optimization after a rare
  override for a trivial, always-correct check — acceptable because sacred-selector
  overrides are exceptional.
- The inliner recognizes compiler-synthesized lazy blocks (from `a and b`) the same
  way it recognizes user-written blocks — both are literal blocks at the AST level —
  so `and`/`or` in operator form inline identically to their method-call form.

## Deviations from spec (recorded deliberately)

These are implementation choices that differ from the letter of the spec as written;
each is intentional and, where the spec is affected, the spec should be reconciled to
match (or an open question resolved) rather than the code changed back.

1. **Selector spelling `ifTrue(_:ifFalse:)` vs spec's `ifTrue(_)ifFalse(_)`.**
   [control-flow.md §3](../../spec/v0.2/control-flow.md) lists the paired conditional as the
   comma/positional form `ifTrue(_)ifFalse(_)` (two block-typed positional slots),
   flagged against an [open question](../../spec/v0.2/open-questions.md). The implementation
   uses the keyword-labelled selector `ifTrue(_:ifFalse:)` because that is what the
   surface desugaring of `if/else` and the existing selector encoder
   ([ADR-0012](0012-selector-signature-encoding-and-dispatch.md)) naturally produce.
   The inliner and the registered `Bool` primitive agree on this spelling, so dispatch
   is self-consistent; the open question should be closed in favour of the keyword
   form.

2. **Jump offset width `i32`, not `i16`.** The jump/guard opcodes carry a signed
   32-bit relative offset. A 16-bit offset (Wren/CLox style) would be tighter, but a
   method body large enough to overflow ±32 KB of bytecode is plausible under
   inlining (each inlined branch splices a full block body), and an offset that
   silently truncates is a correctness bug. `i32` removes the failure mode at the cost
   of a few bytes per jump; revisit only if bytecode density becomes a measured
   problem.

3. **Class reopening added.** `Statement::Class` now reopens an existing same-named
   global class (attaching methods to it) instead of shadowing it with a fresh class
   ([compiler/lib.rs](../../phalcom-core/src/compiler/lib.rs)). This was not previously
   specified but is a prerequisite for the override-epoch design to be *testable* from
   surface Phalcom at all — without reopening, `class Block { whileTrue(_) { … } }`
   would define an unrelated shadow class and never flip the epoch. It also matches
   Smalltalk-family expectations. `install_core` correspondingly registers the kernel
   `Function`/`Block` classes as globals so a reopen resolves to the real class.

4. **`CallContext::Immediate` added.** Invoking a closure-backed (non-primitive)
   method on an immediate receiver (`Bool`, `Number`) previously panicked in
   `Value::to_context`. A new `CallContext::Immediate` variant
   ([frame.rs](../../phalcom-core/src/frame.rs)) gives immediates a well-defined call
   context, which is exactly the path a user override of a sacred `Bool` method takes
   after deopt.

## Amendment (U-CORE-2): one-armed sacred conditionals are `Some`-lifting

`docs/spec/v0.2/core/catalog-delta.md` §4.2 flagged a confirmed divergence: `ifTrue(_)`/
`ifFalse(_)` returned a **half-`Option`** (the taken arm's raw block result, the
untaken arm `None`) rather than a well-formed `Option` (`Some(A) ∪ None`), even
though `object-model.md` §4 and `control-flow.md` §1's `if/else ===
c.ifTrue { A }.ifNone { B }` desugaring both require the latter — `ifNone` is an
`Option` method, so it can't be sent to a raw block result. This was latent only
because the `Option` combinators (`ifNone`/`orElse`/`isSome`/`isNone`, now added to
`core.ph`) didn't exist yet to expose it.

U-CORE-2 `Some`-lifts the *taken* arm of `bool_if_true`/`bool_if_false`
(`primitive/boolean.rs`) via a new shared `wrap_some` helper (`primitive/nil.rs`,
factored out of the `Some.new(_)` primitive); the untaken arm's `None` was already
correct and is unchanged. The paired `ifTrue(_)ifFalse(_)` and `and`/`or` are
unaffected — they were never specified to return `Option` and still don't.

Per this ADR's own parity requirement, the inliner's fast path must match in
lockstep: a new `Bytecode::WrapSome` opcode (pop, allocate a `Some` wrapping the
popped value, push) is emitted after the inlined taken-arm body in
`compile_if_true`/`compile_if_false` (`compiler/inliner.rs`), so the guarded fast
path and the primitive deopt path are observationally identical, as ADR-0018
requires.

**Allocation elision.** `Some`-lifting a value that is immediately discarded (a
bare-statement `cond.ifTrue { ... }` with no chained receiver) would waste an
allocation for no observable benefit — nothing can tell `Some(A)` from `A` once
both are popped unread. `compile_statement_with_pop_control` already knows
statically whether a statement's expression result will be popped; that bit is
threaded down through `compile_expr_want`/`compile_sacred_call_want` into
`compile_if_true`/`compile_if_false`, which skip emitting `WrapSome` when the value
is unwanted. The primitive fallback does not get this optimization — a deopt'd
call always allocates — which is fine for the same reason: a discarded result is
unobservable regardless of which path produced it.

## Alternatives considered

- **No guard (trust the kernel).** Inline unconditionally and forbid overriding sacred
  selectors. Rejected: it contradicts the object model's "everything is an overridable
  method" ([ADR-0004](0004-boolean-as-abstract-bool-with-true-false.md),
  [ADR-0006](0006-function-as-abstract-callable-root.md)) and would make Bool/Block
  second-class.
- **Per-selector assumption invalidation** (fine-grained inline-cache-style epochs).
  More precise than the per-family flag, but far more bookkeeping for a case
  (overriding `ifTrue:`) that essentially never happens in practice. Deferred; the
  coarse flag is a correct subset and can be refined later without changing observable
  behavior.
- **Grammar-level control flow** (compile `if`/`while` straight to jumps, no selector).
  Fastest, but it makes control flow non-overridable and splits "looks like a send,
  isn't a send," breaking the uniform object model the rest of the spec depends on.
  Rejected.
