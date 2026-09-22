---
id: LANG007.C3.P1
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-handoff
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
prepared_against_revision: 60a27f79d69aa3e79c22e3d19df221466c7db920
prepared: 2026-09-22
next_plan: LANG007.C3.P2
---

# LANG007.C3.P1 — Handoff to P2

## 1. Checkpoint state

- `LANG007.C3.P1`: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C3`: `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.
- Next plan: `LANG007.C3.P2`.
- P3 remains responsible for intrinsic structural exposure, Universe/corpus/tooling
  migration, legacy-source closure, and final C3 certification.

## 2. Stable interfaces P2 must consume

1. `AlgebraicDeclarationInfo` is the canonical declaration product. `DataInfo`
   and `EnumInfo` are projections, not competing authorities.
2. `AlgebraicPhase` and algebraic shape are semantic facts. `data` is the
   canonical runtime source category; `enum` is compatibility syntax.
3. `KindData::StaticDomain` owns nominal static-domain kind identity.
4. `TypeData::ConstInt` and `TypeData::StaticData` are canonical static terms.
   Formation must continue to validate domain, constructor, arity, and argument
   kinds through `TypeStore`.
5. `ProperTypeId` remains the proof boundary. Static terms must never be
   silently coerced to proper `Type` terms.
6. Metadata schema v5 carries structural static-domain/term identity and
   declaration algebraic phase/shape. Do not serialize `TypeId`, `StaticDomainId`,
   or source spellings.
7. TypeOnly declarations do not lower to runtime declarations or runtime values.
   The current expression boundary rejects runtime use with
   `DiagnosticCode::TypeOnlyRuntimeValue`.

## 3. P2 implementation scope

The next implementer should begin with the active P2 plan and requirements
analysis. The intended work is:

- heterogeneous static application without changing the row-domain authority;
- domain-directed inference using canonical kind/domain classification;
- checked substitution/materialization for static arguments;
- static aliases and result-domain publication;
- associated `type data`/static-domain lookup through the existing declaration-
  owned `::` namespace;
- metadata/import/source-index preservation and cold/incremental parity.

Do not bypass the runtime-negative expression check merely to make static source
examples compile. Refine the boundary so static contexts form canonical terms,
while runtime contexts continue to reject them.

## 4. Verification already available

- AST integration: 25 `data_syntax`, 12 `enum_syntax` tests passed.
- Semantic ADT suite: 281 passed, 28 explicitly gated/ignored.
- Static foundations: 11 `foundations::kinds`, 17 `foundations::type_model` passed.
- Metadata: 21 semantic metadata tests and 9 schema-compatibility tests passed.
- Fingerprints/incremental: 33 fingerprint tests, 13 active + 3 gated ADT
  incremental tests, and 20 type-store revision tests passed.
- Runtime: 44 core algebraic-data tests passed, 19 explicitly gated/ignored.
- The planned `native_adt_runtime` target is absent from the live package; use
  the core algebraic-data target as the current runtime replacement.

## 5. Known baseline and scope boundaries

- The semantic library `static_data` unit-test binary remains blocked by the
  pre-existing `DeclarationTypeInfo` initializer omission of
  `generic_binder_header`. This is recorded as BASELINE-003 in the checkpoint.
- Do not repair unrelated trait baselines, gated ADT fixtures, workspace-wide
  release checks, LSP closure, or Universe migration under P2 unless the active
  P2 plan explicitly expands scope.
- No commit or push was performed; the shared working tree contains unrelated
  concurrent changes listed by the checkpoint/worktree inspection.

## 6. Decision point for P2

Preserve one canonical user-static-domain identity and make all new static
application/inference consumers use it. If associated lookup, generic
reification, or metadata import requires a second identity source, stop and
escalate under the C3 consultation protocol before editing.
