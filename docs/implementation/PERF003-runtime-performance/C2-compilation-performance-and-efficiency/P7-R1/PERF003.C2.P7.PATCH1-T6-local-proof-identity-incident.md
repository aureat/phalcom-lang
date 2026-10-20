# IMPLEMENTATION INCIDENT: T6 match/pattern local-proof identity

## Resolution — 2026-10-17

The user approved a callable-owned immutable proof graph, frozen at publication
from reachable query-local proof identities. Preserve scope ancestry and distinct
existential openings; alpha-normalize into owner-local ordinals and rewrite every
retained local term. Keep `RigidArena` query-local. Persist and relocate the new
semantic graph with exact allocation accounting and fail-closed closure/visibility
checks. Fingerprints use frozen authority. Do not re-run checking after load or
carry proof machinery into VM executable sections without a runtime requirement.

Decision resolved; implementation and verification remain open. The checkpoint and
PATCH1 T6 amendment own the active execution contract.

## 1. Identity

- Program: PERF003
- Checkpoint: PERF003.C2
- Plan: PERF003.C2.P7.PATCH1
- Task/Gate: T6 match/pattern transport; T8 complete owner reconstruction
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
- Branch: `perf/universe-characterization`
- Starting revision: `5c5da26166e09df7c7825f0343307cfab55922e0`
- Current revision: same; continuation edits are uncommitted.

## 2. Escalation trigger

Shared protocol RED triggers 3, 4, 6, and 11: the remaining local-proof payload
needs an explicit ownership/identity contract. Retaining a scope/rigid owner and
omitting query-local proof views have different semantic consequences. Neither
decision is authorized as a codec-only adaptation.

## 3. Required invariant

PATCH1 §§7.3, 7.10 and T6 require owner-qualified local identities, exact checked
resolution products, and no inference or re-resolution during import. Native
arena offsets cannot become portable authority. Callable completeness and
PATCH3 allocation guards remain active.

## 4. Expected architecture

One callable-local ordinal projection exports a closed owner graph. Import
relocates canonical types and reconstructs frozen native products before
publication and fingerprint recomputation. Match/pattern fields are classified
as Wire. The plan does not specify a persistent rigid/scope owner or an omission
contract for `CaseInstantiation` and rigid-bearing `LocalType` views.

## 5. Observed behavior

`ResolvedVariantCandidate` retains `CaseInstantiation`; branch proofs and pattern
fields/bindings retain local types. `CaseInstantiation` contains a `RigidScopeId`
and parameter-to-`RigidTypeVariableId` map. Their kind, origin, and scope-parent
metadata live in `CheckerContext.rigids`. `CallableAnalysis` retains the pattern
products but has no rigid arena. `CaseInstantiation` is documented as query-local.

This is a persistence-contract uncertainty, not a reproduced runtime defect.
The current end-to-end test stops earlier at the deliberate completeness guard.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core build_and_load_universe_telosc_compiles_source_free_inline_user_entry -- --test-threads=1 --nocapture
```

Observed: one test fails at callable export completeness. Its source census
contains 94 match resolutions, 14 pattern sites, and nonempty local branch
bindings/equalities. That census does not establish that the current Universe
contains rigid-bearing candidates. Generic match transport must still account
for the native products described above.

Expected after closure: complete callable transport without serializing raw
query-local IDs or discarding checked authority.

## 7. Relevant code path

```text
checker/pattern.rs -> coverage/domain.rs::open_variant_case
  -> CaseInstantiation::open(store, &mut ctx.rigids, ...)
  -> ResolvedVariantCandidate { proof, case_instantiation, ... }
  -> CallableAnalysis.match_resolutions / pattern_site_resolutions
```

Key owners:

- `types/rigid.rs`: `RigidArena`, variable kind/origin and scope tree; `LocalType`.
- `types/case_instantiation.rs`: query-local opening and rigid substitutions.
- `checker/context.rs`: `rigids` and callable finalization.
- `match_semantics.rs`: retained proofs, candidates, local field/binding types.
- `checker/analysis.rs`: published callable fields, with no rigid-arena owner.

Paths above are under `phalcom-semantic/src/`.

## 8. Scoped implementation diff

The continuation adds family-application and positional-expansion transport,
validation, frozen indexes, and regressions. It removes the zero-only unsupported
counter DTO. The inherited importer/formal-projection draft remains guarded and
incomplete. No match-proof identity changes have been made.

## 9. Evidence

- Live native definitions and producer path show the ownership gap above.
- Semantic artifact unit module: 37 passed.
- Callable wire target: 12 passed; four inherited decode failures at the global
  `Allocation(UnaccountedBoundary)` guard.
- Required core test: still fails at callable export completeness.
- No independent allocator observation or complete source/import match parity
  has passed in this continuation.

## 10. Implementer's causal hypothesis

Complete match persistence needs a defined lifetime and owner for the local
proof identities. Copying raw IDs would preserve session-local offsets without
their defining authority. Reopening a constructor during import would rerun
semantic work instead of restoring the checked product.

## 11. Alternative hypotheses

A callable-local frozen proof graph could retain scopes, rigid definitions and
their references through ordinal relocation. An explicitly approved omission
contract could instead exclude query-local views while retaining the checked
execution products. The latter would amend the current exact-product contract
and needs consumer/fingerprint justification.

## 12. Attempts already made

Inspected the producer, native owners, and plan contract. No speculative
representation change, raw-ID transport, or proof omission was attempted.

## 13. Why local authority is insufficient

Retaining a new proof owner changes published semantic ownership and identity
lifetime. Omitting fields changes the plan's semantic parity contract. Either
choice exceeds private helper or wire-tag mechanics.

## 14. Decision requested

Should PATCH1 retain a callable-owned immutable scope/rigid proof graph for
match/pattern products? If approved, define its scope and the required variable
kind/origin, scope-parent, and cross-pattern reference invariants. Otherwise,
specify which query-local fields are omitted and the consumer/fingerprint
contract that makes that omission valid.

Status: OPEN. No subagent consultation was started; the user prohibits subagents.
