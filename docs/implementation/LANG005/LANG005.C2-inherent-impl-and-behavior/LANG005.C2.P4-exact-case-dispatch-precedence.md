---
id: LANG005.C2.P4
category: LANG
program: LANG005
checkpoint: LANG005.C2
kind: corrective-implementation-plan
status: IN_PROGRESS
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG005.C2.P3
trigger: DEF-006 / AR-07
---

# LANG005.C2.P4 — Exact-Case Conditional Dispatch Precedence

## Objective

Restore the C2 predecessor invariant that an applicable exact-enum-case
conditional member wins over same-selector enum-root behavior for an exact-case
receiver. The semantic result must flow through the existing lowering/runtime
selection contract; this follow-up must not introduce runtime applicability
solving or a second dispatch source of truth.

## Scope

1. Move exact-case conditional lookup ahead of ordinary enum-root return paths
   in `CheckingContext::resolve_dispatch_target_with_specialization` while
   preserving declaration-target conditional precedence and `super` behavior.
2. Make `receiver_effective_conditional_members` apply the same exact-case
   overlay rule used by checker dispatch.
3. Make the editor facade suppress an inherited ordinary callable when the
   canonical query returns the more-specific exact-case callable.
4. Add the smallest semantic precedence regression and rerun the two failing
   core regressions plus the complete `language::inherent_impl` lane.

## Non-goals and invariants

- Do not alter exact-case target identity, conditional publication, lowering
  metadata, method installation, or VM fallback selection unless focused
  evidence proves the semantic result is not being consumed.
- Do not install every exact-case method unconditionally on shared case classes.
- Do not re-solve impl domains in compiler, runtime, or editor code.
- Bodyless root requirements remain non-executable; valid exact-case bodies must
  remain eligible to satisfy the runtime call.
- Ordinary declaration-target conditional members retain their existing
  ordinary-surface precedence.

## Verification gate

The work unit is accepted only when the new semantic precedence regression,
both DEF-006 core regressions, the full `language::inherent_impl` lane, and the
focused AR-06/AR-08 semantic lifecycle tests pass. Then rerun consolidated G1
AR-01..AR-09 evidence before unlocking `LANG005.C3.P1.T3`.
