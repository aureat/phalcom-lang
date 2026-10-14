---
id: UNIV002.C2.P32
category: UNIV
program: UNIV002
checkpoint: UNIV002.C2
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - UNIV002.C2.P31
follows: UNIV002.C2.P31
supersedes: null
---

# UNIV002.C2.P32 — Float hypot documentation

## 0. Executor contract

Add one Category A documentation comment immediately above
`FloatMathKernel.hypot`'s native declaration. Preserve executable source,
declaration, signature, imports, identity, ownership, and behavior. Do not run
tests. Stop if the live source or accepted contract disagrees with this plan.

## 1. Goal

Document that `FloatMathKernel.hypot` computes `hypot(x, y)` with the Float
backend's binary64 semantics.

## 2. Checkpoint acceptance objective

This is one bounded Category A plan advancing UNIV002.C2; it does not close
the checkpoint.

## 3. Repository grounding

Prepared against:

repository: /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections
branch: feat/telos-collections
revision: 4645c926365758f1c3127e2b4ff755cf97dd8fe5
predecessor: UNIV002.C2.P31
source: packages/universe/src/math/float_kernel.ph
source SHA-256 at plan start: 80b2f309bbbb2619c8401a3ee017d5ccc1fb32a7c08c4f3ec9c2bdd49b57fce0

The live declaration is `@class native hypot(_ x: Float, _ y: Float) -> Float`.
The numeric specification lists `hypot(x, y)` as a binary Float function and
states that Float functions follow the Rust `f64` backend's IEEE binary64
behavior. No method-level comment currently documents this contract.

## 4. Required reads before implementation

- Repository `AGENTS.md` and implementation-record lifecycle convention.
- This plan, C2 checkpoint, and P31 walkthrough and handoff.
- `docs/specs/numeric-math.md` §2.
- `packages/universe/src/math/float_kernel.ph` and scoped status/diff.
- `packages/universe/docs/modernization/INVENTORY.md` and validation ledger.

## 5. Normative authority

`docs/specs/numeric-math.md` §2 is normative. It places `hypot(x, y)` in the
binary Float function set and specifies Rust `f64` IEEE binary64 behavior for
Float functions, without promising identical final bits across hosts. This
plan documents only that backend contract and does not amend the spec.

## 6. Takeover state

P31 documented `IntegerMath.risingFactorial` without changing executable code
or the `universe.math` owner. Shadow semantic linking and runtime acceptance
remain unverified. The inventory records 174 MOD1 paths and 20 changed source
hashes through P31's committed source tree. `math/float_kernel.ph` still
matches its MOD1 hash. Separate uncommitted C3 crypto additions and the root
`package.ph` edit remain outside this plan.

| Concept | Current symbol/path | Owner | Invariant |
|---|---|---|---|
| Float native math | `FloatMathKernel` in `math/float_kernel.ph` | Shadow `universe.math` | Native unary/binary operations take and return `Float`. |
| Hypotenuse | `FloatMathKernel.hypot` | Same module/native surface | Follow the specified Float backend binary64 semantics. |

## 7. Architecture

Add exactly this line immediately above the native declaration for `hypot`:

```phalcom
/// Computes hypot(x, y) with the Float backend's binary64 semantics.
```

The native declaration remains the only source-level binding for this kernel.
Do not change the native API or make runtime verification claims.

## 8. Ownership boundaries

### Owns

- One comment above `FloatMathKernel.hypot`'s native declaration.
- P32 lifecycle evidence and the `math/float_kernel.ph` inventory hash refresh.

### Does not own

- Native implementation, numeric API, Float representation, or runtime behavior.
- The numeric spec, C9 acceptance, Option/Result/Unit identity, iteration,
  reflection, compiler, semantic layer, production Universe, or VM.
- Concurrent MODL001 P4 and UNIV002 C3 crypto changes.

## 9. Global invariants

- INV-01: The comment states only hypot and the backend binary64 semantics.
- INV-02: No executable line, declaration, signature, import, owner, API,
  identity, or behavior changes.
- INV-03: No cross-host last-bit equivalence is promised.
- INV-04: `universe.math` remains the owner of this source declaration.
- INV-05: Shadow Universe remains non-production; no semantic/runtime claim
  follows from the comment.
- INV-06: Concurrent P4 and C3 work remains untouched and unstaged.

## 10. Non-goals

- No native implementation, kernel behavior, API, or spec change.
- No C9 status change or semantic/runtime claim.
- No broad inventory heuristic rescan.
- No comment expansion to the other math kernels in this plan.

## 11. Expected impact map

- `packages/universe/src/math/float_kernel.ph` — exactly one comment.
- `packages/universe/docs/modernization/INVENTORY.md` — current hash/revision.
- `packages/universe/docs/modernization/VALIDATION-LEDGER.md` — P32 evidence.
- C2 checkpoint, plan metadata, walkthrough, and handoff.

No tests are required or permitted by this plan. Any executable change stops
P32 and requires rescoping.

## 12. Implementer decision authority

### FIXED

- Preserve INV-01 through INV-06 and the binary64 contract.
- Keep this change comment-only.

### MECHANICALLY FLEXIBLE

- None; the wording and insertion point are specified.

### VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Declaration and spec agree on binary64 behavior | Live native declaration and spec §2 | Stop and report; do not change behavior. |
| Target source/spec have no competing edits | Scoped status and diff | Coordinate before editing. |

## 13. STOP / CONSULT triggers

Stop if accurate documentation would require changing Float semantics, native
ownership, representation, or the compiler/runtime contract. Follow the shared
escalation protocol for any architectural or specification conflict.

## 14. Debugging budget

This comments-only plan requires exact diff review, scoped `git diff --check`,
and committed-tree inventory reconciliation. Do not run tests. If executable
source changes, stop and rescope.

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant | Evidence |
|---|---|---|---|
| CV-01 | Comment-only scope | INV-01 through INV-04 | Exact diff contains one comment and preserves the declaration. |
| CV-02 | Whitespace hygiene | INV-02 | Scoped `git diff --check`. |
| CV-03 | Inventory reconciliation | INV-05 | Compare the committed source tree with MOD1 and verify the overlay. |
| CV-04 | Semantic/runtime status | INV-05 | No behavior claim; tests not run. |

## 16. Verification execution budget

Do not run tests. Refresh the `math/float_kernel.ph` hash, independently
compare the committed `.ph` path/hash set with the MOD1 baseline, and confirm
the overlay equals the changed-hash map at the source commit. Preserve
heuristic columns as baseline-only. Concurrent uncommitted C3 files are
excluded and must remain untouched.

## 17. Baseline / unrelated failure policy

The prior production-Universe Boolean corpus import-resolution failure remains
assigned to concurrent MODL001 P4. Do not rerun or diagnose it here. Preserve
active P4 and C3 crypto changes.

## 18. Tasks

### T1 — Document hypot — COMPLETE

Added the exact comment from section 7 above `FloatMathKernel.hypot`'s native
declaration. No executable source changed.

### G1 — Scope and whitespace review — PASS

The source diff contained only the planned comment. Scoped `git diff --check`
passed; no tests were run.

### G2 — Inventory reconciliation — PASS

Comparison against the MOD1 baseline at committed source revision
`a351b6a7a286b14d5ece32928153cccd4474bcc0` confirmed 174 matching paths,
no additions/removals, and 21 changed hashes; the inventory overlay matches
that committed tree. Concurrent uncommitted C3 crypto additions and the
`packages/universe/src/package.ph` edit were excluded from this plan's
committed-tree comparison.

## 19. Checkpoint bookkeeping and completion artifacts

At completion, update P32 status, inventory/validation ledger, and checkpoint
state. Create the P32 walkthrough and handoff with source revision, comment
diff, evidence, semantic/runtime limits, and the next safe bounded action.

## 20. Completion record

- Source commit: `a351b6a7a286b14d5ece32928153cccd4474bcc0`.
- Source SHA-256: `92f228dcd420a8178e679e3960a8667515473019649407739be085c7f58b7074`.
- The comment matches the native declaration and normative Float contract.
  No semantic or runtime claim follows; shadow linking remains unverified
  and runtime parity remains blocked.
