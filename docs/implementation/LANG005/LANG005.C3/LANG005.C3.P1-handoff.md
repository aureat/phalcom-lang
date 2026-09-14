---
program: LANG005
checkpoint: LANG005.C3
plan: LANG005.C3.P1
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
next_plan: LANG005.C4.P1
worktree: dirty; preserve unrelated working-tree changes
---

# Handoff — LANG005.C3.P1 to LANG005.C4.P1

## Entry condition

C3 is complete at `IMPLEMENTED` + `FOCUSED_TESTED` strength. Do not begin C4
conformance or witness production until `C2-F03` applicability proof-state
granularity is closed. The semantic result must distinguish at least
`Proven`, `Disproven`, `Unknown`, `Blocked`, and `Dynamic` (repository-equivalent
names are acceptable) so C4 cannot turn uncertainty into a false conformance.

## Stable C3 interfaces

| Contract | Canonical owner and boundary |
|---|---|
| Trait declaration | module declaration shell with `DeclarationKind::Trait` and canonical `DeclarationId` |
| Trait generic metadata | DB-owned `TraitHeader` using `TypeParameterOwner::Declaration` |
| Generic contract reference | `TraitRef { declaration, arguments }`; not a `TypeId`, class object, or proof |
| Requirement identity | `TraitRequirementId { owner, selector, side }` |
| Source/default callable | ordinary declaration-owned `CallableId`; no parallel member/default ID |
| Complete contract | DB-owned `TraitSurface`, separate from `DeclarationSurface`, enum behavior, and conditional inherent products |
| Default context | abstract owner-relative `SelfTypeTerm`, complete trait surface, semantic-only abstract application target |
| Trait dependency | `SemanticDependency::TraitSurface(declaration)` plus ordinary external dependencies |
| Source tooling | `SemanticTargetId::Declaration(DeclarationId)` and `SemanticTargetId::Callable(CallableId)` with trait presentation kind |
| Compiler/runtime | `Statement::Trait` is a no-op compile-time/type-level declaration; ordinary VM dispatch is trait-unaware |

## First actions for C4.P1

1. Read the C4 checkpoint, guidance, and `LANG005.C4.P1` plan against the live
   C3 tree.
2. Close `C2-F03` at the owning semantic applicability-result boundary and
   record cold/incremental evidence before accepting conformance declarations.
3. Extend the shared `impl` declaration machinery for explicit
   `impl TraitRef for Target` only after the proof-state prerequisite is
   satisfied.
4. Keep conformance heads and source `ImplId` provenance separate from
   inherent behavior products; publish exact `(target, TraitRef)` candidates
   without claiming requirement satisfaction.
5. Preserve C3 `TraitSurface` signatures/defaults as the input to later witness
   selection. Do not re-solve trait member lookup in C4.P1.

## Explicit non-goals for the next slice

C4.P1 must not implement witness selection, default selection, conformance
completeness, associated types, generic trait bounds, conditional conformance,
trait objects, supertraits, metatype/class-side conformance, runtime vtables,
or trait scanning in ordinary dispatch. Those belong to later C4 slices or
later checkpoints.

## Final C3 evidence

The final focused lanes passed: AST 3, modules 1, semantic capabilities 13,
incremental 3, source index 1, core trait boundary 2, inherent impl 15, and
algebraic data 44 with 19 pre-existing ignored cases. Formatting and negative
architecture checks passed. Full workspace/release certification remains
separate.
