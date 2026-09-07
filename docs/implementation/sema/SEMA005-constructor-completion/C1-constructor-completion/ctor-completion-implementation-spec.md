# U-CTOR Completion Implementation Specification

**Status:** Pending explicit final-verification instruction
**Date:** 2026-07-22
**Scope:** Close U-CTOR planning, diagnostic, and documentation debt. Preserve implemented constructor semantics.

## 1. Goal

U-CTOR implementation largely matches live behavior. Close remaining work so the
unit can be accurately tracked and, when explicitly requested, independently
verified for completion.

This specification does **not** authorize constructor lowering, VM, parser, or
runtime semantic changes. Those changes are already implemented unless a required
diagnostic split below exposes a local implementation defect.

## 2. Locked decisions

1. Duplicate selectors and duplicate field declarations are separate error cases.
   Keep a selector-specific diagnostic for same-side canonical selector collisions
   and a field-specific diagnostic for duplicate field declarations. Do not use the
   broader `class.duplicate_member` / `ClassDuplicateMember` contract as a substitute
   for either case.
2. Final verification occurs only at end of this work and only after explicit user
   instruction. Do not start a throwaway-worktree verification, `cargo doc`, fixture
   red/green proof, `graphify update .`, or an independent `./scripts/verify.sh` run
   before that instruction.
3. Do not alter current constructor semantics while completing this unit. In
   particular preserve `@class`, `@construct`, `@constructor`, two-method lowering,
   `new_`, `native_repr`, inherited bare `new()`, matching super-constructor rewrite,
   class-side field storage, reserved names, and current broad fixture coverage.

## 3. Required changes

### 3.1 Plan status

Update `docs/work/pending/ctor/plan.md` from `Status: **READY**` to a status that
accurately records implementation completion with closure work pending. Name this
file as the closure authority and state that final verification is deferred pending
explicit instruction.

Do not mark U-CTOR fully complete until Section 5 is executed and recorded.

### 3.2 Split diagnostic contract

Replace U-CTOR plan references to `class.duplicate_selector` / `DuplicateSelector`
with the actual final selector-collision contract, or implement that exact contract
if code does not expose it. The end state must provide distinct diagnostics:

| Condition | Required contract |
|---|---|
| Two post-expansion members install the same canonical selector on the same side | `class.duplicate_selector` / `DuplicateSelector` |
| Two field declarations collide | dedicated duplicate-field diagnostic; not `DuplicateSelector` |

`class.duplicate_member` / `ClassDuplicateMember` may remain only if it is no longer
the user-visible diagnostic for either condition, or is a non-user-facing internal
abstraction with lossless mapping to the two contracts above.

Preserve same-side semantics: instance-side and class-side declarations with the same
selector do not collide. Run selector detection after expansion so generated members
participate; diagnostics for generated members must point to originating source.

Add or retain focused compile-error fixtures proving selector and field cases produce
their separate diagnostic contracts. Fixture execution belongs to Section 5 unless a
source edit requires focused validation.

### 3.3 Remove stale constructor diagnostic references

Remove or correct every live reference to `ConstructStaticCollision`, including:

- compiler comments and documentation;
- `docs/spec/current/traceback/output-catalog.md`; and
- U-CTOR planning text that presents it as current behavior.

Historical ADR/PDR text is provenance. Do not rewrite it unless it makes an
unqualified claim about current behavior; add a narrow status or supersession note
instead.

## 4. Out of scope

- Reopening constructor language design or changing lowering behavior.
- Broad cleanup of unrelated dirty worktree changes.
- Claiming a verifier pass based on earlier reports.
- Running final verification without explicit instruction.

## 5. Deferred completion verification

Execute this section only after explicit user instruction. Run from a clean
throwaway worktree at the candidate SHA, isolated from concurrent Cargo jobs.

1. Confirm fixture wiring: deliberately make each new selector and field diagnostic
   fixture disagree with its expected output, observe failure, restore expected
   output, and observe success.
2. Run `cargo doc` and require success with documentation for every new public item.
3. Run the repository completion gate: `./scripts/verify.sh`. Treat prior reported
   success as unconfirmed until this independent run finishes green.
4. Run `graphify update .` after source/documentation changes.
5. Record commands, SHA, worktree path, outcomes, and any baseline failures in the
   plan or completion log. Do not call U-CTOR complete if any required gate is absent
   or blocked.

## 6. Acceptance criteria

- U-CTOR plan no longer says `Status: READY`.
- Selector collisions and duplicate field declarations have distinct user-visible
  diagnostic contracts and focused fixture coverage.
- No active compiler comment or current documentation refers to
  `ConstructStaticCollision` as a live error variant.
- Constructor semantics listed in Section 2 remain unchanged.
- Final verification evidence is either recorded per Section 5 or clearly marked
  deferred pending explicit instruction; it is never inferred from prior reports.
