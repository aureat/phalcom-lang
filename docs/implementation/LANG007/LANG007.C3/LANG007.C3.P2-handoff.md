---
id: LANG007.C3.P2
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# LANG007.C3.P2 — Handoff to P3

## State

P2 is complete and focused-tested at committed revision
`518f66d19941eb24e379e60f64aa89df66643a8a` (`fix(semantic): thread active type
lambda arena`). C3 remains partial and active; P3 is the active successor plan.

Read the canonical state and evidence first:

- `LANG007.C3-CHECKPOINT.md` — shared checkpoint state, accepted invariants, and
  baseline evidence.
- `LANG007.C3.P2-walkthrough.md` — final implementation, coverage map, test results,
  audits, and residual follow-ups.
- `INC-C3-11.md` — architect's resolved arena-ownership decision and implementation
  evidence.

## Stable P2 interfaces and invariants

- `TypeStore::apply_static_form` and `apply_type_form` remain the canonical application
  boundaries; `StaticArgument` retains the distinct `Term` and `RecordRow` lanes.
- `TypeStore::apply_static_form_with_arena` and `apply_type_form_with_arena` carry one
  caller-owned active `TypeLambdaArena` through recursive beta reduction.
- `TypeStore::with_active_lambda_arena` performs the public O(1) move-out/restore scope.
  Ordinary result/error returns restore the arena; interning remains monotonic on
  checked errors.
- `TypeSubstitution::{try_apply_with_arena, apply_with_arena}` and row substitution
  helpers thread that same arena recursively. Do not call a public boundary from inside
  an active-arena recursion.
- `TypeStore` owns canonical `TypeId` creation; `TypeLambdaArena` owns scoped and lambda
  interning. There is one active arena per recursive normalization operation.
- No transaction/journal rollback behavior is promised. Panic/unwind recovery was out of
  scope.
- Static-domain and static-case identities are canonical semantic identities; the
  `AssociatedSurface` remains namespace/collision authority.
- Schema v8 compatibility, canonical workspace/module import linking, provider identity,
  source indexing, and incremental/cold parity are accepted at focused-test level.
- There is no separate general semantic declaration-graph importer from a metadata
  bundle; do not describe one as implemented.
- ProperType/runtime boundaries and the runtime-negative static-domain contract remain
  intact. Do not add runtime static execution during P3.

## Verification and residuals

INC-C3-11's four regressions plus the existing type-lambda tests pass in the
49-test `foundations::type_annotations` module. The remaining exact focused evidence and
known failures are listed in the walkthrough/checkpoint. Specifically:

- The metadata alpha-equivalence failure and three record-row-polymorphism failures are
  clean-predecessor baselines, not P2 regressions.
- Current rustfmt reports workspace drift already present in the four touched Rust files
  at HEAD; no broad reformat was applied.
- Full declaration graph edge/SCC reconstruction remains a performance follow-up.
- Workspace-wide test, clippy, and release certification remain P3/checkpoint scope.

## Next action

Continue `LANG007.C3.P3` at T2/G2 for intrinsic static-domain facade registration
and canonical source anchors. T1/G1 passed at the completed-P2 revision. Preserve
the accepted P2 architecture and baseline classifications. Do not claim C3 closure
until P3 and its checkpoint gates are complete.
