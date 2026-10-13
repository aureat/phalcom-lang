---
id: UNIV002.C2.P31
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P30
follows: UNIV002.C2.P30
supersedes: null
---

# UNIV002.C2.P31 — Rising factorial documentation

## 0. Executor contract

Add one Category A documentation comment immediately above
`IntegerMath.risingFactorial`'s `@class` annotation. Preserve executable
source, declaration, signature, imports, identity, ownership, and behavior.
Do not run tests. Stop if the live source or accepted contract disagrees with
this plan.

## 1. Goal

Document that `IntegerMath.risingFactorial` returns the exact rising product,
raises `ArgumentError` for negative arguments, and returns one when `k == 0`.

## 2. Checkpoint acceptance objective

This is one bounded Category A plan advancing UNIV002.C2; it does not close
the checkpoint.

## 3. Repository grounding

Prepared against:

repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: a64f4725f5e9ca54cac13502e157f03d5f910389
predecessor: UNIV002.C2.P30
source: packages/universe/src/math/integer.ph
source SHA-256 at plan start: 5d22da47e388f8dbb9be814e7772588e1e1ab7dbd1d94947b9bd59d5bd5424b1

The live method raises `ArgumentError` when either argument is negative,
returns one when `k == 0`, returns zero when `n == 0` and `k > 0`, and
otherwise multiplies the ascending factors starting from `n`. The normative
numeric spec specifies the exact rising product, invalid negative arguments,
and one for `k == 0`. No comment currently documents this method.

## 4. Required reads before implementation

- Repository `AGENTS.md` and implementation-record lifecycle convention.
- This plan, C2 checkpoint, and P30 walkthrough and handoff.
- `docs/specs/numeric-math.md`, “Integer algorithms”.
- `packages/universe/src/math/integer.ph` and scoped status/diff.
- `packages/universe/docs/modernization/INVENTORY.md` and validation ledger.

## 5. Normative authority

`docs/specs/numeric-math.md` is normative for `universe.math`. Its integer
algorithm table specifies the exact rising product, invalid negative
arguments, and one when `k == 0`. The live method agrees and raises
`ArgumentError` for negative arguments. This plan documents those facts; it
does not amend the spec.

## 6. Takeover state

P30 documented `IntegerMath.fallingFactorial` without changing executable code
or the `universe.math` owner. Shadow semantic linking and runtime acceptance
remain unverified. The inventory records 174 MOD1 paths and 20 changed source
hashes through P30's committed source tree. Separate uncommitted C3 crypto
additions and the root `package.ph` edit remain outside this plan.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Exact integer math | `IntegerMath` in `math/integer.ph` | Shadow `universe.math` | Keep exact `Int` algorithms in this module. |
| Rising factorial | `IntegerMath.risingFactorial` | Same module | Exact rising product; negative arguments error; `k == 0` returns one. |

## 7. Architecture

Add exactly this line immediately before `@class` for `risingFactorial`:

```phalcom
/// Returns the exact rising product; negative arguments raise ArgumentError, and k == 0 returns one.
```

The existing method remains the only executable behavior source. Do not
alter the numeric API or its accepted contract.

## 8. Ownership boundaries

### Owns

- One comment above `IntegerMath.risingFactorial`'s `@class` annotation.
- P31 lifecycle evidence and the `math/integer.ph` inventory hash refresh.

### Does not own

- The rising-factorial algorithm, numeric API, integer identity, or runtime behavior.
- The numeric spec, C9 acceptance, Option/Result/Unit identity, iteration,
  reflection, compiler, semantic layer, production Universe, or VM.
- Concurrent MODL001 P4 and UNIV002 C3 crypto changes.

## 9. Global invariants

- INV-01: The comment states only the exact rising product and specified
  boundary behavior.
- INV-02: No executable line, declaration, signature, import, owner, API,
  identity, or behavior changes.
- INV-03: No performance, allocation, or complexity promise is added.
- INV-04: `universe.math` remains the owner of exact `Int` algorithms.
- INV-05: Shadow Universe remains non-production; no semantic/runtime claim
  follows from the comment.
- INV-06: Concurrent P4 and C3 work remains untouched and unstaged.

## 10. Non-goals

- No algorithm rewrite, executable modernization, API/spec change, or test.
- No C9 status change or semantic/runtime claim.
- No broad inventory heuristic rescan.

## 11. Expected impact map

- `packages/universe/src/math/integer.ph` — exactly one comment.
- `packages/universe/docs/modernization/INVENTORY.md` — current hash/revision.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P31 evidence.
- C2 checkpoint, plan metadata, walkthrough, and handoff.

No tests are required or permitted by this plan. Any executable change stops
P31 and requires rescoping.

## 12. Implementer decision authority

### FIXED

- Preserve INV-01 through INV-06 and the exact public contract.
- Keep this change comment-only.

### MECHANICALLY FLEXIBLE

- None; the line and insertion point are specified.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Source behavior matches the spec | Live `IntegerMath.risingFactorial` body | Stop and report; do not rewrite behavior. |
| Target source/spec have no competing edits | Scoped status and diff | Coordinate before editing. |

## 13. STOP / CONSULT triggers

Stop if source contradicts the accepted contract, or accurate documentation
would require changing integer semantics, identity, representation, or
ownership. Follow the shared escalation protocol for any architectural or
specification conflict.

## 14. Debugging budget

This comments-only plan requires exact diff review, scoped `git diff --check`,
and committed-tree inventory reconciliation. Do not run tests. If executable
source changes, stop and rescope.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Comment-only scope | INV-01 through INV-04 | Exact diff contains one comment and preserves the method. |
| CV-02 | Whitespace hygiene | INV-02 | Scoped `git diff --check`. |
| CV-03 | Inventory reconciliation | INV-05 | Compare the committed source tree with MOD1 and verify the overlay. |
| CV-04 | Semantic/runtime status | INV-05 | No behavior claim; tests not run. |

## 16. Verification execution budget

Do not run tests. Refresh the `math/integer.ph` hash, independently compare
the committed `.ph` path/hash set with the MOD1 baseline, and confirm the
overlay equals the changed-hash map at the source commit. Preserve heuristic
columns as baseline-only. Concurrent uncommitted C3 files are excluded and
must remain untouched.

## 17. Baseline / unrelated failure policy

The prior production-Universe Boolean corpus import-resolution failure remains
assigned to concurrent MODL001 P4. Do not rerun or diagnose it here. Preserve
active P4 and C3 crypto changes.

## 18. Tasks

### T1 — Document rising factorial — COMPLETE

Added the exact comment from section 7 above `IntegerMath.risingFactorial`'s
`@class` annotation. No executable source changed.

### G1 — Scope and whitespace review — PASS

The source diff contained only the planned comment. Scoped `git diff --check`
passed; no tests were run.

### G2 — Inventory reconciliation — PASS

Comparison against the MOD1 baseline at committed source revision
`4645c926365758f1c3127e2b4ff755cf97dd8fe5` confirmed 174 matching paths,
no additions/removals, and 20 changed hashes; the inventory overlay matches
that committed tree. Concurrent uncommitted C3 crypto additions and the
`packages/universe/src/package.ph` edit were excluded from this plan's
committed-tree comparison.

## 19. Checkpoint bookkeeping and completion artifacts

At completion, update P31 status, inventory/validation ledger, and checkpoint
state. Create the P31 walkthrough and handoff with source revision, comment
diff, evidence, semantic/runtime limits, and the next safe bounded action.

## 20. Completion record

- Source commit: `4645c926365758f1c3127e2b4ff755cf97dd8fe5`.
- Source SHA-256: `589db05514cdaf2f5cd4e1a7fc76d1586dae76da6d4e8064ddaabb10c41512d1`.
- The comment matches the live method and normative integer contract. No
  semantic or runtime claim follows; shadow linking remains unverified and
  runtime parity remains blocked.
