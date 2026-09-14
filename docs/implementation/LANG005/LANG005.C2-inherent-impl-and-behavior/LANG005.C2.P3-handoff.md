# Handoff — LANG005.C2.P3 Constrained and Specialized Inherent `impl` Applicability

```yaml
program: LANG005
checkpoint: LANG005.C2
plan: LANG005.C2.P3
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
head: 1a2f78bd
worktree: dirty; preserve unrelated working-tree changes
```

## C2 completion state

C2 is complete at focused verification strength. The checkout contains the advisor’s landed P3 runtime/semantic implementation and the remaining scoped editor, fingerprint, test, and record corrections. Nothing was staged, committed, or pushed by this handoff.

## Stable interfaces

- `InherentImplTarget` and `InherentImplContribution` remain the canonical target/contribution identities from P1/P2.
- `ConditionalDispatchSelection` is the canonical per-expression result: `ImplId`, target `CallableId`, declaring `DeclarationId`, and `DispatchSide`.
- `ResolvedDispatch.conditional` and `ExpressionAnalysis.conditional_dispatch` carry selection evidence from checking to lowering.
- `BoundBehavioralMember.conditional` carries already-selected evidence into `MakeConditionalFamily`.
- `receiver_effective_conditional_members` is the semantic editor/query boundary for concrete receiver alternatives.

## Identity and surface laws

`ImplId` identifies an impl contribution; `CallableId` identifies the callable. Neither replaces the other. Conditional members stay in the target-indexed conditional product and are not inserted into the unconditional `DeclarationSurface` or shared runtime behavior class. Ordinary selector precedence is checked first; same-owner duplicate selectors are rejected independent of domain.

## Matching and P2 composition

Structural head matching binds impl parameters, enforces repeated bindings and existing generic constraints, and projects inherited receivers to the declaring owner before matching. Exact enum cases use canonical `VariantId`; GADT case facts are established before the conditional proof. No consumer may inspect erased generic arguments or invoke a second matcher.

## Runtime boundary

`MakeConditionalFamily` materializes exact declaring class identity and a rooted closure fallback. `VM::select_conditional_method` is the single strict-subclass override probe used by direct conditional invocation and family activation. Class-side calls use the metaclass of the exact declaring class and preserve applied receiver evidence; no per-application runtime classes are created.

## Tooling and incremental ownership

The editor retains formal applied `TypeId` and dispatch mode independently, delegates applicability to semantic receiver-effective lookup, and fails closed for insufficient class-side evidence. Source/LSP products retain canonical impl-origin identity. Callable-body fingerprints include conditional selection; incremental query tests prove body-only reuse, target isolation, exact-case handling, and cold/incremental agreement.

## Verification inherited

- Semantic impls: 40/40.
- Semantic impl queries: 13/13.
- Core inherent impl: 15/15.
- Semantic editor: 6/6.
- LSP navigation: 2/2.
- LSP semantic completion: 9/9.
- Exact bound-family and applied class-side regressions: both passed.
- Three-crate check and scoped diff validation passed.

Broad workspace/release certification was not run. The next plan is `LANG005.C3.P1`, First-Class Trait Declarations and Abstract Trait Surfaces.
