# Annotations — contract runtime semantics (re-entrancy, purity, stripping)

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-11
- Depends on: [annotations-contracts.md](annotations-contracts.md)
- Resolves: contract soundness gaps — invariant re-entrancy, predicate purity, release-mode stripping, multiple invariants
- Related: ADR-0008 (errors), concurrency §1 (cooperative single-thread), DEC-C (no flow analysis)

## Context

annotations-contracts.md specifies the *weave* but not the runtime discipline
that keeps it sound. Three unaddressed traps: invariant re-entrancy (infinite /
mid-mutation checks), impure predicates (checks that mutate), and release-mode
stripping.

## Decision

### Invariant re-entrancy — outermost-boundary only

`__check_invariant()` must fire **only at the outermost public call boundary**,
not on nested self-sends. Between two field writes an object is legally
inconsistent; a public method it calls internally must not trip the invariant.

Mechanism: a per-fiber **`in_public_call` depth counter** (Phalcom is cooperative
single-thread, concurrency §1, so a per-fiber flag is race-free). The woven
prologue increments it; the invariant check runs only when it transitions 0→1
(entry) and 1→0 (exit). Nested public sends see depth ≥ 1 and skip. This is
Eiffel's rule (invariants disabled during a qualified call).

Without this, `@invariant` + any invariant predicate that sends a public message
recurses infinitely, and any multi-field mutator false-positives mid-write.

### Predicate purity — pure, with a syntactic floor

Contract predicates (`@requires`/`@ensures`/`@invariant` exprs) **must be
side-effect-free**. Phalcom has no effect system (same limit as DEC-C's
truthiness ban), so enforcement is a floor, not a proof:

- **Reject at expansion time** the syntactically-obvious mutations: assignments
  (`_x = …`), and sends of known-mutator selectors on `self`/fields.
- **Accept the rest on trust.** A predicate that mutates through an opaque send
  is undefined behavior of the contract, documented as a user error.

`old(...)` operands obey the same rule and additionally the mutable-aliasing
restriction (annotations-contracts.md).

### Multiple `@invariant` — conjoined

A class may declare several `@invariant`s; they are **conjoined** in declaration
order into one `__check_invariant`. Order is observable only in *which* failure
raises first; all must hold.

### Release-mode stripping — compile-mode gated

Contract weaving is gated on a compile mode:

| Mode | `@requires` | `@ensures` | `@invariant` |
|------|-------------|------------|--------------|
| `debug` (default) | woven | woven | woven |
| `release` | woven¹ | stripped | stripped |
| `unchecked` | stripped | stripped | stripped |

¹ Preconditions stay in `release` by default (they guard the public boundary —
Meyer's "demand" contracts); overridable. Stripping happens **in the expander**
(the pass emits no guard), so stripped contracts cost zero bytecode. Reflectable
predicate metadata (annotations-contracts.md D-contract-1) is retained regardless
of mode, so property testing works even against a `release` build.

## Consequences

- The reflectable metadata table and the woven guards are now **independently**
  controlled: mode decides guards, D-contract-1 decides metadata. Property-testing
  tools read metadata; production runs skip guards.
- The `in_public_call` counter is one machine word per fiber — allocated in the
  fiber's frame state (ADR-0013 frame token infrastructure).

## What this precludes

Outermost-only invariants preclude *intra-method* invariant checkpoints without a
new explicit `checkInvariant` primitive — acceptable; Eiffel proves the boundary
discipline is enough.
