---
id: LANG007.C4.algebraic-literals-follow-on
kind: implementation-plan
checkpoint: LANG007.C4
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
prepared_against_revision: 43a4f466
last_updated: 2026-10-11
---

# Contextual algebraic literals — execution plan and stop state

This tracked record preserves the task contract originally written in
`docs/superpowers/plans/2026-10-08-contextual-algebraic-literals.md`. That directory
is ignored by Git. The C4 checkpoint owns lifecycle state; this file owns the
remaining execution contract.

The follow-on has no assigned numbered plan ID. `LANG007.C4.P12` already belongs
to String–Char work and must not be reused. `LANG007.C4.P3` belongs to Pack and
call/rest convergence. Phase 3 of `cliteral.md` is contextual realization and
remains incomplete.

## Objective and fixed architecture

Implement the runtime algebraic-value and suffix-literal contract in
`cliteral.md`, including contextual Complex/Quaternion realization. Literals
remain required work. Static realization is deferred to the existing static
eligibility/materialization contract.

Keep mathematical behavior and nominal declarations source-owned. Preserve
canonical declaration identity, exact generic data types, capability evidence,
ordinary selector identity, single evaluation, and source order. Publish checked
realization facts in the semantic layer for compiler consumption. Do not select
units by source spelling or infer formal facts from advisory shapes.

## Architecture ruling and continuation — 2026-10-10

The user selected **Add class-side traits**. T3 now extends trait behavioral
requirements through ordinary `@class` dispatch while keeping conformance
evidence attached to the nominal target type. This does not add metatype
conformance. Enum class-side trait conformances and class-side index
requirements remain outside the supported boundary.

The trait spec, semantic requirement publication, default-body and witness
signature formation, data-component witness guard, and source declarations for
`Zero`, `One`, `RealEmbeddable`, and `BasisRealizable` have been updated.
`Complex<T>` and `Quaternion<T>` now declare direct class-side realization
operations. Rust compilation passed; source-language conformance, selector,
and runtime acceptance is still unverified. No tests were run in this
continuation.

The user also ruled that an assumed class-side capability on generic `T` gets
its nominal class receiver from the active runtime type environment. Keep the
conformance slot responsible for method and environment evidence; do not copy
the class receiver into that slot. Missing or non-nominal runtime bindings must
fail closed. This is a clarification within T5, not a new plan.

## Architect ruling and resolved blockers — 2026-10-11

The architect directed three semantic repairs: gate contextual realization on
classified source and algebraic context before capability lookup; summarize
arithmetic regions with one iterative post-order worklist; and carry the
receiver's class-side denotation through ownerless generic dispatch while
binding trait `Self` to the conforming subject type. These repairs and focused
semantic regressions are present. The 50,000-node summary test and focused
class-side tests pass.

The follow-up architect ruling was received: keep contextual realization in
`Option<Arc<ContextualRealization>>` on transient and published expressions and
separate ordinary binary checking from realization-specific locals. The
744-byte clean-main carrier baseline and 760-byte budget now pass. Debug frame
measurement additionally required moving binary dispatch outside the formal
and advisory analyzers' large variant matches; the unchanged 64-coefficient
Float Horner regression now passes on a 2 MiB semantic thread.

The intermediate bootstrap blocker (`AssociatedInvokeNotLoweredYet` in
`universe:callable.function`) has also been resolved and verified:
1. `CheckingContext::declaration_generic_signature` no longer records nominal
   `DeclarationShell` dependencies on trait requirements, resolving query
   suppression in `session.update_with_delta`.
2. `semantic_lowering.rs` now provides generic evidence for class-side
   conformance witnesses (`@class one`), resolving `MissingTraitRequirementSlot`.

The full Universe bootstrap regression (`canonical_bootstrap_stack.rs`) passes.
Next action: resume contextual realization tasks (T4/T5) and source package
acceptance. See the cross-cutting
[incident report](../../STDL001-numeric-library/stack-overflow-incident-2026-10-11.md), §16–17.

## Incremental dispatch invalidation consultation — 2026-10-11

P1-T's first add/remove conformance regressions reached a current-revision
`SuppressedDependency` while replaying cached body dependencies. The exact
blocked key was not exposed, but inspection confirmed that a retained
`TraitDispatchBucket` query with a prior fingerprint is rejected instead of
refreshed, while changed-bucket discovery runs later in the session update.
The read-only architect consultation confirmed this is a plausible direct
failure path and recommended keeping the bucket query as owner of its
fingerprint.

P1-T's implementation scope is amended to refresh changed bucket roots from
the current canonical `ConformanceSemanticView` immediately after constructing
the current trait-dispatch index and before cached body dependency replay.
Refresh the union of old and new bucket keys, including removed buckets at the
current empty-bucket fingerprint. Preserve incoming client edges and use the
existing dependency-fingerprint comparison to recompute only clients whose
observed bucket changed; do not invalidate the full reverse closure eagerly.
The original A/B/C tests remain the acceptance contract: add, remove, and
unrelated-bucket reuse with cold/incremental parity. This is a bounded
implementation correction; it does not change semantic architecture or require
an ADR.

## P2 literal-scalar contract clarification — 2026-10-11

The remaining-task acceptance example `2i` targeting `Strange<Tag, Float>`
conflicts with `cliteral.md` §10.4 and §19: the numeric literal grammar fixes
`2i` to Int and `2.0i` to Float, with no implicit Int-to-Float conversion.
The live contextual-realization checker enforces exact source/associated
scalar identity. The read-only architect consultation confirmed the existing
contract. P2 therefore uses `2.0i` for the Float realization and may retain a
negative check that `2i` does not realize against a Float coefficient. No
language-semantic amendment is made.

## P4-T6 accepted-witness lifecycle amendment — 2026-10-11

An incremental trait-side edit left a prior accepted Instance-side conformance
callable available to current-revision replay after the requirement changed to
Class side. The read-only architectural consultation confirmed that explicit
witness identity includes dispatch side. Do not republish the rejected source
method as an independent callable.

Reconcile the affected accepted witness set at session-level conformance
rebuild, before retained-definition replay. Preserve exact affected `ImplId`s
when a changed trait surface reaches a structurally reusable conformance
module. Retire rejected `CallableDefinition`, `CallableSignature`, and
`CallableBody` products and its `callable_analyses` entry; remove it from
signature/body refresh worklists; and capture reverse dependents before
retirement so their current semantic products and diagnostics are refreshed.
Keep the nominal `ConformanceContribution` and publish the new incomplete
dispatch plan. Acceptance requires cold/incremental equality for plan
fingerprints and diagnostics, absence of stale witness products, and refreshed
dependent clients. This is a bounded P4-T6 lifecycle/scheduling correction;
it does not alter language rules or require an ADR.

## Task ledger

| Task | State | Established result / remaining acceptance |
| --- | --- | --- |
| T1 — Failing syntax and runtime probes | PARTIAL | Syntax, basis identity, Quaternion laws/unavailability, typed realization, and projection probes exist. Add `2i` versus `2*i`, context propagation, shadowing, effect-order, and rejection coverage. |
| T2 — Lexer/parser and AST consumers | IMPLEMENTED / FOCUSED_TESTED | Dedicated coefficient/axis nodes, numeric grammar integration, atomic errors, and exhaustive consumers compile. Two focused syntax tests pass; all four new syntax tests passed in the larger AST run. |
| T3 — Source-owned basis/Quaternion algebra | PARTIAL / UNVERIFIED | The class-side trait decision is resolved. Side-aware requirements/witnesses and `Zero`, `One`, `RealEmbeddable`, and `BasisRealizable` source APIs are implemented. Rust crates compile; source-level conformance and runtime acceptance remain unverified. |
| T4 — Canonical semantic realization | PARTIAL / UNVERIFIED | Canonical products cover direct basis/suffix realization, `RealEmbeddable` scalar terms, and bounded arithmetic regions selected from canonical basis identities and known scalar types. Explicit targets win; completed bindings remain boundaries; unresolved scalar types fail closed. Source acceptance, inference boundaries, and incremental parity remain unverified. |
| T5 — Compiler projection and lowering | PARTIAL / UNVERIFIED | Direct and assumed generic class-side operations project and lower. Expression coefficients support both `scalar * basis` and `basis * scalar`; the latter stages the basis value while preserving source order. Real embeddings lower through the checked class-side requirement. Source acceptance and compiler/runtime coverage remain outstanding. |
| T6 — Acceptance and lifecycle | PARTIAL | Interim evidence and blocker records are delivered at the user-requested stop. Full feature acceptance is red. |

## Resume gates

1. Resolve the generic reification incident before changing the affected seam.
   Keep the exact projection regression and its identity oracle. **COMPLETE:**
   call-entry projection now closes covering-impl declaration and source-body
   coordinates bidirectionally from semantic `CoveringImplSubstitution`.
2. **Implemented, source acceptance pending:** establish scalar identities and
   explicit target-side realization capabilities in the numeric source modules.
   Validate coherence and exact callable selection, especially multiple
   Quaternion axes under one generic capability.
3. Add canonical semantic regressions for unconstrained Basis identity, typed
   uses, aliases/qualified access, same-spelled unrelated declarations, expected
   types, inference boundaries, and cold/incremental parity.
4. **Partially implemented:** select bounded arithmetic-region targets before
   nested operations: I-only defaults to Complex, J/K or mixed axes to
   Quaternion; known scalar domains must agree and completed bindings remain
   boundaries. Direct scalar terms use `RealEmbeddable`; scalar-by-basis uses
   direct component realization. Extend only where source evidence establishes
   the scalar type; unresolved types fail closed.
5. **Partially implemented:** publish checked realization evidence for a
   directly expected basis value, native suffix literal, or real scalar term.
   Per-expression products, fingerprints, and dependencies retain target,
   operation, capability selection, and evaluation order. Compiler projection
   consumes direct and assumed generic selections; assumed receivers derive
   from the active runtime type environment and fail closed for missing or
   non-nominal bindings. Add semantic and compiler acceptance coverage when
   authorized.
6. **Partially implemented:** lower checked realizations through selected
   class-side trait calls and the canonical basis data constructor. Preserve
   reads, side effects, single evaluation, exceptional Float components, large
   exact Int coefficients, and GC rooting. Fail closed without sufficient facts.
7. Complete focused AST, semantic, compiler, runtime, and incremental gates;
   review the scoped diff and update the checkpoint and completion records.

Run Cargo gates serially. Broader crate/workspace/release gates require an
explicit acceptance need. The current stop does not authorize a scope reduction.
