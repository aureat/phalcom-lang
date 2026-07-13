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

Contract weaving is gated on a compile mode. Guards and reflectable metadata
are now **two independent axes**, not one: guard stripping (already specified)
removes the woven checks from the method body; metadata stripping (new)
removes the retained `Symbol → [Block]` side table itself, so a build that
never runs property tests doesn't carry the predicate `Block`s in memory
either. Metadata defaults to retained (matching D-contract-1's original
intent — most `release` builds still want it available), but every mode may
opt out with a separate `--strip-contract-metadata` flag:

| Mode | `@requires` guard | `@ensures` guard | `@invariant` guard | Metadata (default) |
|------|-------------|------------|--------------|---------------------|
| `debug` (default) | woven | woven | woven | retained |
| `release` | woven¹ | stripped | stripped | retained (opt out with the flag) |
| `unchecked` | stripped | stripped | stripped | **stripped by default** |

¹ Preconditions stay in `release` by default (they guard the public boundary —
Meyer's "demand" contracts); overridable. Guard stripping happens **in the
expander** (the pass emits no guard), so stripped guards cost zero bytecode.
Metadata stripping happens at the same point: when stripped, the expander
never emits the `Symbol → [Block]` side-table entry for that class, so the
predicate `Block` objects are never allocated in the first place — not
retained-then-freed, simply never built. `unchecked` is the one mode that
strips metadata by default, since it signals "no contract tooling of any
kind"; `release` keeps the D-contract-1 property-testing use case available
unless the flag is passed, matching the size-conscious deployments (embedded,
constrained targets) that opt in explicitly.

## Consequences

- The reflectable metadata table and the woven guards are **independently**
  controlled along two axes: mode decides guards, mode **and** the
  `--strip-contract-metadata` flag decide metadata. Property-testing tools
  read metadata when present; a size-constrained `release`/`unchecked` build
  can now shed both.
- The `in_public_call` counter, and its receiver-scoping fix
  ([ADR-0052](../../../adr/0052-invariant-reentrancy-scope-and-layout-confined-decorator-state.md)),
  apply only when `@invariant` guards are woven — a build that strips
  `@invariant` (release/unchecked) allocates no `checking` set entries for
  that class either, since the woven prologue/epilogue that would touch it is
  never emitted.

## What this precludes

Outermost-only invariants preclude *intra-method* invariant checkpoints without a
new explicit `checkInvariant` primitive — acceptable; Eiffel proves the boundary
discipline is enough.
