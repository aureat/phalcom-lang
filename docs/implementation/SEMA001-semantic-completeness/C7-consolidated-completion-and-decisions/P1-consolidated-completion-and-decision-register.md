# 07 — Phalcom Typing Platform: Consolidated Implementation Plan and Decision Register

**Date:** 2026-08-23
**Revision:** repository-rebased execution map after Specs 01, 01.5, 02, 02.5, 03, 03.5, 04, 04.5, revised 05, and revised 06
**Status:** Ratified execution plan; implementation-state observations are rebased against `aureat/phalcom-lang@dd89c2f6f2021b0458e2a03e5bcb5ac5c0e7a3e2` (`main`, `docs(typing): reorganize advanced typing analysis`, 2026-08-23)
**Authority:** implementation sequencing, dependency gates, migration/compatibility policy, deletion criteria, repository ownership, cross-crate integration, conformance, performance validation, rollout, risk register, decision register, and completion criteria
**Does not own:** type-system semantics already ratified by Specs 01–06; runtime object-model semantics; selector identity; parser grammar beyond implementation sequencing; proof-backend semantics beyond the gates owned by Spec 05
**Primary owners:** `phalcom-semantic`, `phalcom-modules`, `phalcom-ast`, `phalcom-native-meta`, `phalcom-native-decl`, `phalcom-native-macros`, `phalcom-native-surface`, `phalcom-type-meta`, `phalcom-core`, `phalcom-lsp`
**Evidence policy:** repository source is authoritative for current implementation state. Attached Specs 01–06 are authoritative for ratified architecture. No build, test, benchmark, REPL, or CI success is claimed in this revision unless explicitly identified as executed evidence. At the inspected `main` commit, no fresh verification run was performed for this document.

---

# 0. Revision contract

This document replaces the previous **07 — Consolidated Implementation Plan and Decision Register** as the execution map for completing the Phalcom typing platform.

It does **not** replace or reinterpret the semantics owned by earlier normative specifications. Its job is narrower:

> Given the architecture ratified across Specs 01–06 and the implementation that exists on current `main`, identify exactly what remains to be implemented, migrated, deleted, validated, benchmarked, and stabilized, in dependency order.

The previous Spec 07 was useful when most of the platform existed only as a plan. That is no longer the repository state. Significant portions of Specs 01–03.5 and the Spec-04 syntax surface have landed. The old A–J phase plan therefore contains two kinds of stale assumption:

1. work described as future that now exists materially; and
2. early design choices superseded by later normative revisions.

This revision rebases the program around the live architecture rather than preserving old phase names for continuity.

## 0.1 Superseded assumptions from the previous Spec 07

| Previous assumption | Current decision |
|---|---|
| Specs 02/03 metadata and reflection are future phases | Core durable metadata, artifact carriage, runtime typing registry, lazy reification, reflection classes/capabilities, and much of the public API exist. Remaining work is correctness, semantic parity, profile hardening, and advanced-section integration. |
| Source type parsing is reference-only | Current `phalcom-ast` parses the broad Spec-04 type-form grammar, generic binders, kinds, `where`, records, type lambdas, aliases, and value-space type forms. Parser existence is not semantic-publication completion. |
| Generic substrate lacks variance, `Self`, type lambdas, record-row kind, or result-rich relations | These are materially present in the canonical semantic substrate. Their source publication and executable-checker integration remain incomplete. |
| Relation migration begins from a boolean-only kernel | `RelationOutcome`, budgets, cancellation, variance-aware applied relations, and generic supertype traversal exist. Compatibility booleans and incomplete consumers remain. |
| `SemanticDb` does not exist | A real `phalcom-semantic/src/db/` scaffold exists, including query keys, states, dependency recording, reverse invalidation, scheduler, metrics, budgets, and `CallableBody` key. It is not yet the active workspace/body-analysis engine. |
| Native semantic surface is handwritten | The generated `NATIVE_SURFACES` architecture and normalized `phalcom-native-decl` pipeline have landed. Legacy `NATIVE_MEMBERS`, dual runtime installation, and LSP compatibility paths still exist. |
| Prenex kind polymorphism is a ratified ordinary-generic requirement | Superseded by revised Spec 05 and Spec 06. Explicit arrow kinds and type lambdas are the ordinary HKT foundation. Public kind polymorphism is optional, deferred, and gated. |
| Finite exact-set generic constraints/default type arguments are part of the initial generic plan | Superseded. Initial canonical constraints are signature-owned `Subtype` and `Equivalent`; finite exact-set constraints and generic defaults remain deferred. |
| Effects/exits/termination/proofs may be staged as one advanced state | Superseded. Revised Spec 05 requires independent semantic products. |
| Runtime-cycle handling still sorts modules after failure | Current LSP static workspace code no longer sorts through a runtime cycle; it drops the static publication by returning `None`. That is still insufficient because the failure is not published as structured semantic state/diagnostic, but the old sorted fallback claim is no longer current. |
| Runtime reflection relation surface can be counted complete because selectors exist | Rejected. Current runtime `subtype`, `assignable`, `consistent`, and `conforms` delegate to equivalence. Surface existence is not semantic correctness. |

## 0.2 Evidence classifications used in this document

Every implementation-state claim should be read under one of these categories:

- **Repository-observed:** inspected in source at `dd89c2f6...`.
- **Observed test intent:** a current registered/visible test exists for an invariant; the test was not rerun for this document.
- **Spec-mandated:** required by Specs 01–06 or the handoff decisions ratified with them.
- **Implementation choice:** this document chooses an implementation organization that preserves the ratified semantics.
- **Open:** architecture or product decision still requires ratification.
- **Deferred:** intentionally not on the foundation critical path.
- **Not verified:** no build/test/benchmark/runtime execution claim is made.

## 0.3 Authority order on conflict

When the repository and specification differ:

```text
ratified semantic truth
        ↓
canonical compiler semantic representation
        ↓
runtime metadata/reflection projection
        ↓
current implementation accident
        ↓
presentation
```

A lower-layer defect does not redefine a higher-layer semantic rule.

Canonical example:

```text
semantic truth:
    Option :: Type -> Type

current runtime projection:
    Behavior#kind guesses arity and returns Type for Option

required action:
    repair runtime projection

forbidden action:
    weaken Option's semantic kind to Type
```

---

# 1. Repository archaeology baseline

## 1.1 Current `main`

The repository baseline for this revision is:

```text
repository: aureat/phalcom-lang
branch:     main
commit:     dd89c2f6f2021b0458e2a03e5bcb5ac5c0e7a3e2
message:    docs(typing): reorganize advanced typing analysis
date:       2026-08-23
```

Recent migration-relevant commits include:

```text
ba24a661... feat(native): canonicalize native universe surface
c97500ba... feat(lsp): continue canonical native core surface merge
84453c6f... fix(lsp): preserve source callable origins
9ae0191b... feat(typing): add canonical core surface metadata
edbced89... extend runtime typing metadata and reflection
78986b21... feat(typing): extend runtime metadata reflection
fbfef8dc... Align language semantics across parser, compiler, runtime, and LSP
a43f26e0... fix(typing): align never metadata and dispatch surfaces
59b3dce4... test(semantic): verify two-axis semantic tower determinism/invariants
```

The current analysis-series documents are present under:

```text
docs/work/analyses/typing/
```

The older `docs/spec/typing/STATUS.md` is historical and contains superseded concepts such as `Type.currentApplication`, `out`/`in`, finite-set bounds, and earlier document numbering. It is not used as the forward architectural authority for this plan.

## 1.2 Verification boundary

This revision inspected current source, tests, and recent commits through the repository connector. It did **not** execute a fresh local:

```text
cargo test
cargo check
cargo clippy
REPL run
benchmark suite
```

The current commit also had no useful combined CI status evidence surfaced during this review. Accordingly:

- existing tests are cited as **test intent/evidence**, not as freshly passing results;
- performance sections define measurement gates, not measured claims;
- runtime behavior is called “repository-observed” only where the source path establishes it directly.

---

# 2. Protected architectural invariants

Every workstream in this plan is blocked if it violates any of the following.

1. **Static type metadata never changes runtime selector identity.**
2. **Static type metadata never changes ordinary method dictionary keys, inline-cache identity, class/metaclass identity, instance layout, or allocation.**
3. **`List<Int>` is a semantic application, not a specialized runtime class.**
4. **`Unknown` is not `Dynamic`.** Missing evidence must not become an intentional dynamic boundary.
5. **Temporary solver state is not canonical semantic state.** In particular, `InferVarId != TypeId` in the final architecture.
6. **Runtime reflection observes semantic facts; it does not define them.**
7. **Source/native/generated/intrinsic methods share one ordinary callable-typing algorithm.** Implementation provenance does not define a second checker.
8. **Formal flow semantics live in `phalcom-semantic`.** LSP `ValueShape` remains advisory.
9. **Runtime contracts are runtime guards, not static proofs.**
10. **Proof trust is explicit.** Backend success alone does not imply trusted `Proven`.
11. **Return type, effects, exits, termination, contracts, and proof status remain orthogonal products.**
12. **Every recursive/query/solver path is bounded and cancellable where required by Specs 01/04.5/05.**
13. **No raw store-local or solver-local identity crosses durable metadata boundaries.**
14. **No current implementation bug is rationalized into language semantics.**

---

# 3. Current repository state by subsystem

Status vocabulary:

```text
IMPLEMENTED            target architecture substantially exists and is active
PARTIAL                important target pieces exist, but formal completion gate is not met
COMPATIBILITY          retained only as migration bridge/floor
NOT IMPLEMENTED        target product does not materially exist
BLOCKED                cannot correctly complete before another named gate
DEFERRED               intentionally outside the critical path
```

## 3.1 Consolidated status matrix

| Subsystem | Status | Repository evidence | Required conclusion |
|---|---|---|---|
| Store-local `TypeId`/`KindId`, `TypeStoreId`, `ProperTypeId` | **PARTIAL** | `phalcom-semantic/src/types/{id.rs,store.rs}` | Identity boundary exists; some proper-child construction is still protected only by `debug_assert!`, so release-safety hardening remains. |
| Kinds (`Type`, `RecordRow`, arrow) | **IMPLEMENTED substrate** | `types/kind.rs`, `TypeStore::new` | Canonical kind domain exists. Runtime projection is not yet metadata-driven. |
| Generic parameters owner/index, variance, constraints | **IMPLEMENTED substrate** | `types/parameter.rs` | Canonical data model is substantially aligned with 01.5. Source declaration publication is incomplete. |
| Type lambdas | **PARTIAL** | `types/type_lambda.rs`, parser AST, metadata exporter | Canonical scoped lambda machinery exists; source lowering does not yet bind lambda parameters capture-safely. |
| Partial type application | **IMPLEMENTED substrate** | `TypeStore::apply_type_form` | Supports residual kind and flattened application; must be exercised through source/runtime conformance. |
| `Self` semantic form | **PARTIAL** | `types/parameter.rs`, `TypeData::SelfType`, metadata | Representation exists; source owner/context publication and call specialization are incomplete. |
| Generic supertype relation support | **PARTIAL** | `types/relation.rs`, declaration `supertype_template` field | Relation engine can consume templates, but source workspace predeclaration currently publishes none. |
| `TypeData::Infer` | **COMPATIBILITY debt** | `types/store.rs`, checker/constraint | Still canonicalized in production paths; must be deleted after inference migration. |
| `LocalConstraintSolver` | **COMPATIBILITY debt** | `types/constraint.rs` | Store-coupled boolean solver remains in active checker paths. |
| Bounded relation result algebra | **PARTIAL** | `types/relation.rs` | Rich outcomes/budgets/cancellation exist; boolean wrappers and incomplete consumers remain. |
| Canonical callable signature table | **PARTIAL** | `phalcom-semantic/src/signature.rs` | Native surfaces use it; source callables are still chiefly represented by legacy concrete signatures. |
| Legacy concrete callable surfaces | **COMPATIBILITY debt** | `dispatch.rs`, `surface.rs`, `checker/declaration.rs` | Duplicated signature/return maps and cloned signatures remain. |
| Expression checker | **PARTIAL legacy** | `checker/{context,expression,call,statement}.rs` | Synthesis-heavy, monolithic, no formal bidirectional API, no canonical method-generic inference, no causal analysis graph. |
| Formal flow state | **NOT IMPLEMENTED** | compiler checker lacks persistent FlowState | Branch result unions exist, but not path-state semantics. |
| LSP advisory flow/inference | **IMPLEMENTED advisory** | `phalcom-lsp/src/semantic/{flow,infer,facts}.rs` | Useful algorithms; cannot remain formal authority. |
| Stable AST expression identity | **NOT IMPLEMENTED** | no `ExpressionId`/`LocalExpressionId` in current AST/semantic body products | 04.5 should introduce body-local deterministic expression IDs. |
| `for` element typing | **INCORRECT/COMPATIBILITY** | `checker/statement.rs` | Formal checker assigns `Dynamic(ExplicitEscape)`; must derive from `iterate(_)` / `iteratorValue(_)`. |
| Spec-04 parser/AST | **IMPLEMENTED broadly** | `phalcom-ast/src/{ast.rs,parser.rs}` | Syntax is ahead of semantic publication. |
| Explicit annotation outcome algebra | **PARTIAL** | `types/annotation.rs` | Invalid/kind/application failures still collapse to coarse `Unknown` states; `KindSyntax::Invalid` can recover as `Type`. |
| Source generic declaration publication | **NOT COMPLETE** | `workspace.rs` source predeclaration | Source classes are predeclared monomorphically with `generic_signature: None`, `supertype_template: None`. |
| Open record-row semantics | **PARTIAL substrate / BLOCKED** | `KindData::RecordRow`, type-meta `OpenRecord`; `TypeData::Record` closed only | Row kind/schema exist; canonical row term/solver/tail semantics from Spec 05 do not. |
| Transparent aliases semantic publication | **PARTIAL** | AST/compiler no-op exists | Runtime correctly emits no code, but semantic declaration/expansion/cycle ownership is incomplete. |
| Type-form values | **PARTIAL** | AST `Expr::TypeForm` | Checker currently falls through generic unchecked-expression path; formal semantic/runtime materialization integration incomplete. |
| `SemanticDb` substrate | **PARTIAL** | `phalcom-semantic/src/db/` | Query keys/states/dependencies/reverse invalidation/scheduler exist; one-shot workspace analyzer does not run through it. |
| Callable-body query | **SCAFFOLD** | `QueryKey::CallableBody(CallableId)` | No authoritative `CallableAnalysis` product yet. |
| Immutable semantic snapshot | **PARTIAL** | `snapshot.rs` | Snapshot exists; publication/module-state integration is not yet the final DB-owned model. |
| Module/source-owned diagnostics | **PARTIAL** | `diagnostic.rs` | Source span can carry module, but common checker helper defaults ownership to core and many diagnostics lack causes/notes/fixes. |
| Partial workspace failure publication | **NOT COMPLETE** | LSP `run_static_workspace_analysis` | Project/interface/import/link failures are often skipped; runtime-cycle failure drops publication rather than emitting structured failure. |
| Durable indexed metadata | **IMPLEMENTED core** | `phalcom-type-meta`, `semantic/metadata/export.rs` | Versioned graph, fingerprints, kinds, lambdas, signatures, `Self`, open-record schema exist; advanced sections and remaining semantic parity still gated. |
| Runtime typing registry/reification | **IMPLEMENTED core** | `phalcom-core/src/typing/*` | Architecture matches Specs 02/03; relation and kind-projection correctness gaps remain. |
| Runtime `Behavior#kind` projection | **INCORRECT** | `phalcom-core/src/primitive/typing.rs::behavior_kind` | Hard-coded arity recognizes List/Set/Map only; Option/Some and other generic declarations misproject. |
| Public runtime relation selectors | **INCORRECT placeholder** | `typing_context_subtype/assignable/consistent/conforms` | Currently collapse to equivalence; must project canonical relation semantics or report honest unavailability. |
| Public `applyKind` | **ABSENT by design** | primitive registration | Preserve absence; `FunctionKind` is observational. |
| Native canonical surface | **IMPLEMENTED formal core / PARTIAL migration** | `phalcom-native-decl`, `phalcom-native-surface/generated.rs`, `NATIVE_SURFACES` | Formal generated surface is real. Compatibility `NATIVE_MEMBERS` and dual runtime installation remain. |
| Native textual type syntax | **COMPATIBILITY front end** | `phalcom-type-syntax/src/lib.rs::TypeExpr` still includes `Unknown` and a smaller symbolic grammar | Keep separate native parser, but canonical lowering must translate opacity explicitly and converge below parsing; it must not make `TypeExpr::Unknown` a canonical semantic type. |
| Native normalized declaration IR | **IMPLEMENTED** | `phalcom-native-decl/src/normalized.rs::NormalizedPrimitiveDecl` | Correct VM-free authoring normalization seam shared by macro/generator; extend rather than fork when native syntax gains later canonical forms. |
| Native runtime descriptor installer | **PARTIAL migration** | `phalcom-core/src/native/*`, `VM::new_with_native_install_mode` | Descriptor path exists; default `Dual` and legacy floor remain until census equality. |
| Source/native shared call checking | **NOT COMPLETE** | native canonical signatures vs source legacy surfaces | Must converge on `CallableId` + canonical signature/view. |
| Compiler-owned effects | **NOT IMPLEMENTED** | native `EffectSpec` exists only as declaration input | Spec 05 workstream C remains. |
| Compiler-owned exits | **NOT IMPLEMENTED** | native `RaisesSpec`/`ReturnFlowSpec` inputs | Spec 05 workstream D remains. |
| Termination / `@total` analysis | **NOT IMPLEMENTED** | no semantic `TerminationKnowledge` implementation | Spec 05 workstream E remains. |
| Canonical static contract IR | **NOT IMPLEMENTED** | runtime guard weaving exists in core | Spec 05 workstream F remains. |
| Verification conditions | **NOT IMPLEMENTED** | no VC IR/generator | Spec 05 workstream G remains. |
| Proof backend/trust/artifacts | **NOT IMPLEMENTED** | metadata extension seams only | Spec 05 workstream H remains. |
| Shared VM-free diagnostics renderer | **NOT IMPLEMENTED** | no `phalcom-diagnostics` crate on current main | 04.5 D2 remains. |

---

# 4. Dependency graph: current implementation to completion

The implementation is no longer a simple 01 → 02 → 03 → 04 → 05 waterfall. Metadata/reflection/native work landed ahead of some source/checker integration. The remaining program should follow dependency *truth*, not historical document order.

```text
                     ┌─────────────────────────────────┐
                     │ R0 correctness / invariant gate │
                     │ proper types, outcomes, diag,   │
                     │ runtime kind/relation honesty   │
                     └───────────────┬─────────────────┘
                                     │
                 ┌───────────────────┴────────────────────┐
                 │                                        │
                 v                                        v
       ┌────────────────────┐                    ┌────────────────────┐
       │ S04 publication    │                    │ Q0 SemanticDb      │
       │ generics / where / │                    │ active inputs,     │
       │ lambda / Self /    │                    │ typed products,    │
       │ alias / type forms │                    │ partial failures   │
       └─────────┬──────────┘                    └─────────┬──────────┘
                 │                                        │
                 └───────────────────┬────────────────────┘
                                     v
                        ┌─────────────────────────┐
                        │ C0 callable convergence │
                        │ source/native ->        │
                        │ CallableId/signature    │
                        └───────────┬─────────────┘
                                    v
                        ┌─────────────────────────┐
                        │ E3 local inference      │
                        │ no canonical Infer      │
                        └───────────┬─────────────┘
                                    v
                     ┌──────────────────────────────┐
                     │ 04.5 expression/call engine │
                     │ IDs, bidirectional checking,│
                     │ generic calls, outcomes     │
                     └────────────┬─────────────────┘
                                  v
                     ┌──────────────────────────────┐
                     │ F formal flow + iteration   │
                     │ joins, loops, mutation      │
                     └────────────┬─────────────────┘
                                  v
                 ┌────────────────┴──────────────────┐
                 v                                   v
      ┌──────────────────────┐            ┌──────────────────────┐
      │ X/D explanations +   │            │ Q1 callable-body     │
      │ diagnostics renderer │            │ incremental products │
      └──────────┬───────────┘            └──────────┬───────────┘
                 └────────────────┬──────────────────┘
                                  v
                     ┌──────────────────────────────┐
                     │ L formal LSP convergence     │
                     │ advisory-only ValueShape    │
                     └────────────┬─────────────────┘
                                  │
             ┌────────────────────┴─────────────────────┐
             │                                          │
             v                                          v
   ┌──────────────────────┐                   ┌──────────────────────┐
   │ N/R native/runtime   │                   │ Spec 05 advanced     │
   │ parity + deletion    │                   │ independent products │
   └──────────┬───────────┘                   └──────────┬───────────┘
              │                                          │
              └───────────────────┬──────────────────────┘
                                  v
                     ┌──────────────────────────────┐
                     │ V stabilization/conformance │
                     │ perf, fuzz, rollout         │
                     └──────────────────────────────┘
```

## 4.1 Critical path

The critical path to ordinary typing-platform foundation completion is:

```text
R0
→ S04 semantic publication
→ canonical callable convergence
→ session-local inference
→ bidirectional expression/call checking
→ compiler-owned flow
→ ExplanationGraph/structured diagnostics
→ CallableBody SemanticDb integration
→ LSP formal migration
→ conformance/performance stabilization
```

Spec 05 proof-backend work is **not** on this critical path. Record-row semantics are partially coupled because Spec 04 already exposes row syntax, but effects/exits/termination/contracts/proofs can be implemented as independent products after the ordinary body-analysis substrate stabilizes.

---

# 5. Workstream R0 — immediate correctness and invariant repairs

This workstream should land before large checker migration because it removes false authority and makes later failures honest.

## R0.1 Enforce proper-type construction in release builds

**Goal.** Complete Spec 01's `ProperTypeId` boundary so a constructor-kinded form cannot silently enter value-typing positions in release builds.

**Current state.** `ProperTypeId` and `TypeStore::proper_type` exist. Several aggregate constructors still rely on `debug_assert!`-style assumptions for child properness.

**Primary files.**

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/evidence.rs
phalcom-semantic/src/checker/typed_expr.rs
phalcom-semantic/src/metadata/export.rs
```

**Implementation.**

- move public/semantic constructors for tuple/record/union/callable value types toward `ProperTypeId` children or checked `Result` constructors;
- leave raw `TypeId` constructors `pub(crate)` only where construction is proven by a trusted caller;
- ensure metadata/snapshot publication validates proper positions;
- preserve constructor-kinded forms in type-form positions.

**Tests first.**

- arrow-kind child rejected in tuple/callable/value annotation;
- partial generic application rejected in a proper-value position;
- `Never`, `Unit`, nominal, applied proper forms accepted;
- release-mode negative tests, not debug-only assertions.

**Deletion criterion.** No formal value-knowledge constructor can accept an unchecked arbitrary `TypeId` from public semantic code.

**Verification commands.**

```bash
cargo test -p phalcom-semantic
cargo test -p phalcom-semantic --release
```

## R0.2 Complete relation API migration

**Goal.** Make the rich relation algebra authoritative everywhere; compatibility booleans may not drive formal checker policy.

**Current state.** `RelationOutcome`, `Assignability`, budgets, cancellation, variance-aware applied relations, and supertype-template traversal exist. `is_subtype` and coarse helpers remain, and checker call sites often branch only on `Refuted`.

**Primary files.**

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/checker/{expression.rs,statement.rs,call.rs,context.rs}
phalcom-semantic/src/types/constraint.rs
```

**Implementation.**

- keep one internal compatibility boolean only if a non-formal caller genuinely needs it;
- formal checking must exhaustively handle:

```text
Proven / Assignable
Refuted
DynamicBoundary
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

- never turn cancellation/budget/internal failure into a mismatch;
- preserve relation operands/evidence so diagnostics do not fabricate `TypeId::DUMMY` placeholders.

**Deletion criterion.** Public/formal checker paths no longer call boolean `is_subtype` or interpret `DynamicBoundary` as proof.

## R0.3 Make semantic diagnostic ownership explicit

**Goal.** No checker diagnostic should accidentally claim `ModuleId::core()` merely because a convenience constructor omitted the source owner.

**Current state.** `SemanticSourceSpan` can own a module, but `SemanticDiagnostic::error(...)` defaults ownership to core and is widely used by the checker.

**Primary files.**

```text
phalcom-semantic/src/diagnostic.rs
phalcom-semantic/src/checker/*
phalcom-semantic/src/workspace.rs
phalcom-lsp/src/analysis_service.rs
```

**Implementation.**

- require `DiagnosticOwner` / `SemanticSourceSpan` at formal diagnostic construction boundaries;
- permit core-default helpers only in explicitly core/native tests or internal bootstrap code;
- carry project/interface/import/link/runtime-cycle failure ownership into semantic outcomes.

**Deletion criterion.** Repository search shows no production source-checker error constructed without an explicit source/module owner.

## R0.4 Repair runtime generic-kind projection

**Goal.** `Behavior#kind` must project canonical declaration metadata rather than re-author generic arity by class name.

**Current defect.** In `phalcom-core/src/primitive/typing.rs`, `behavior_kind` and `behavior_remaining_count` special-case:

```text
List -> 1
Set  -> 1
Map  -> 2
other -> 0
```

This makes `Option.kind` and `Some.kind` reflect `Type` despite semantic metadata requiring `Type -> Type`.

**Primary files.**

```text
phalcom-core/src/primitive/typing.rs
phalcom-core/src/typing/{registry.rs,inspect.rs,reify.rs}
phalcom-native-meta/src/universe.rs
phalcom-type-meta/src/{kind.rs,declaration.rs,generic.rs}
phalcom-core/tests/spec03_reflection.rs
```

**Implementation.**

1. resolve the class object's stable declaration/runtime metadata binding;
2. read its canonical declaration form/generic signature kind;
3. reify the `KindNode` through the existing `FunctionKind` mechanism;
4. compute `.remainingParameters` from the semantic form, not hard-coded names;
5. if metadata is unavailable in a profile where the answer cannot be established, return the API's honest unavailable/blocked result rather than guess.

**Required matrix.**

```text
Int.kind        == Type
List.kind       == FunctionKind(Type -> Type)
Set.kind        == FunctionKind(Type -> Type)
Map.kind        == FunctionKind(Type -> Type -> Type)
Option.kind     == FunctionKind(Type -> Type)
Some.kind       == FunctionKind(Type -> Type)
```

Also test every other generic declaration in `UNIVERSE_TYPE_FORMS` / canonical declaration metadata.

**Non-goal.** Do **not** add public `applyKind`.

## R0.5 Repair runtime relation honesty

**Goal.** The runtime reflection API must not claim canonical subtype/assignability/consistency/conformance while computing only equivalence.

**Current defect.** `typing_context_subtype` calls `inspect::equivalent`; `assignable`, `consistent`, and `conforms` delegate to subtype.

**Primary files.**

```text
phalcom-core/src/primitive/typing.rs
phalcom-core/src/typing/inspect.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/reflection/*           # if/when canonical query facade is materialized
```

**Implementation rule.** Runtime projection must consume the canonical semantic relation semantics. Acceptable implementation choices are:

- a VM-independent relation evaluator over validated metadata that is behaviorally proven against `phalcom-semantic`; or
- a shared lower-level relation crate/facade consumed by both compiler and runtime metadata reflection.

Unacceptable:

- copying another relation calculus into `phalcom-core`;
- returning `RelationRejected` when the operation is merely unavailable;
- continuing to label equivalence as subtype/assignability/consistency.

**Temporary compatibility policy.** If full semantics cannot land immediately, the public operation should return the appropriate `Blocked`/`Unavailable` variant for unsupported relation forms rather than a false semantic answer.

---

# 6. Workstream S04 — complete Spec-04 semantic publication

The parser is no longer the primary blocker. The blocker is getting parsed declarations into the canonical semantic declaration/signature model exactly once.

## 6.1 Updated Spec-04 S1–S9 gate matrix

| Gate | Current `main` status | Evidence | Remaining completion gate |
|---|---|---|---|
| **S1 core type-form parser** | **Implemented broadly** | `phalcom-ast/src/parser.rs::{parse_type_annotation,parse_type_form,parse_type_lambda,...}`; AST variants exist | Run/maintain registered parser corpus; remove any grammar compatibility behavior only through explicit migration decision. |
| **S2 explicit lowering outcomes** | **Partial** | `phalcom-semantic/src/types/annotation.rs::TypeFormResolution` is still Known/Dynamic/Unknown | Introduce distinct missing/unresolved/invalid/blocked/cancel/budget/internal outcomes; invalid syntax/kind/application must not become `UnannotatedDeclaration`; invalid kind syntax must not become `Type`. |
| **S3 generic binders/kinds** | **Syntax + substrate implemented; publication incomplete** | Class/method AST binders; `resolve_generic_signature`; canonical parameter model | `workspace.rs` must publish source declaration/method generic signatures and constructor kinds before body checking. |
| **S4 `where` constraints** | **Syntax + lowering helper present; publication incomplete** | AST `WhereClauseSyntax`, `GenericConstraint::{Subtype,Equivalent}`, `resolve_generic_signature` | Class/method/alias signatures must retain constraints in canonical tables; body/call checker consumes them, not reparses. |
| **S5 type lambdas** | **Partial** | parser + `type_lambda.rs` scoped arena + metadata schema | Source lowering must resolve lambda binders into bound scoped nodes, prove alpha/capture safety, and stop encoding the resolved body as an unbound `Free` term. |
| **S6 generic superclass / `Self`** | **Partial** | AST superclass is type form; canonical `Self`; relation template support | Source declaration publication sets `supertype_template`; `Self` resolves with owner/side; generic inherited lookup specializes correctly. |
| **S7 aliases / rows** | **Partial / row-blocked** | `Statement::TypeAlias`; row syntax; `KindData::RecordRow`; metadata OpenRecord | Alias declaration/expansion/cycle semantics must publish. Row tail cannot be enabled semantically until Spec-05 row domain/solver lands; current annotation lowering must stop silently dropping the tail. |
| **S8 type-form values** | **Partial** | AST `Expr::TypeForm` | Formal checker must synthesize value/denotation semantics; compiler artifact/runtime root bridge must materialize only when needed; checker wildcard must not return generic unchecked state. |
| **S9 native/source convergence** | **Advanced partial** | generated `NATIVE_SURFACES`, canonical native signature import, `phalcom-native-decl` | Source callables must enter the same `CallableSignatureTable`; legacy `NATIVE_MEMBERS`/dual installer/LSP native compatibility paths retire after parity. |

## 6.2 S04-A — publish source generic declaration signatures

**Primary files.**

```text
phalcom-semantic/src/workspace.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/parameter.rs
phalcom-modules/src/interface.rs
```

**Current defect.** Source class predeclaration creates a nominal form with `KindId::TYPE`, `generic_signature: None`, and `supertype_template: None` regardless of parsed class binders.

**Target algorithm.**

```text
predeclare DeclarationId
    ↓
allocate declaration generic binder IDs by owner/index
    ↓
lower binder kinds
    ↓
compute declaration constructor kind
    ↓
lower where constraints in binder environment
    ↓
publish GenericSignature
    ↓
lower generic supertype template in same environment
    ↓
publish DeclarationTypeInfo
    ↓
only then lower member signatures
```

No body query may be required to discover the declaration's generic interface.

**Tests first.**

- `class Box<T>` publishes `Box :: Type -> Type`;
- `class Functor<F: Type -> Type>` publishes correct higher-order kind;
- `+T`/`-T` variance published;
- class `where` round-trips;
- duplicate generic names fail with source-owned diagnostic;
- generic interface fingerprint changes when kind/variance/constraint changes and not when body-only code changes.

## 6.3 S04-B — publish source callable signatures once

**Target.** Every source method gets exactly one `CallableId` and one `CallableSemanticSignature` before body analysis.

**Primary files.**

```text
phalcom-semantic/src/signature.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/surface.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/workspace.rs
```

**Required data.**

```rust
CallableSemanticSignature {
    callable,
    owner,
    side,
    selector,
    generics,
    parameters,
    return_type,
    source,
    implementation,
    ...
}
```

Source declarations use the same canonical record shape as native imports. Source provenance differs; call semantics do not.

**Deletion gate.** Body checking no longer calls annotation lowering to reconstruct a signature already published by the declaration/interface phase.

## 6.4 S04-C — capture-safe type-lambda lowering

**Current defect.** Canonical scoped lambda machinery exists, but source lowering resolves the body without a lambda-binder scope and can encode it as a free term.

**Implementation.**

- enter a lambda-local binder environment distinct from declaration `TypeParameterId` ownership;
- lower references to lambda parameters as `ScopedTypeData::Bound { depth, index }`;
- retain declaration parameters as scoped `Free` terms;
- nest depths correctly;
- beta-reduce through the existing canonical lambda arena;
- test alpha equivalence and capture avoidance.

**Deletion criterion.** No source type lambda whose body references its own binder is represented as a free unresolved/declaration parameter.

## 6.5 S04-D — generic superclass and `Self`

**Implementation.**

- resolve superclass type form in declaration generic environment;
- require final superclass template to have kind `Type`;
- store unspecialized `supertype_template` in `DeclarationTypeInfo`;
- keep runtime erased superclass identity separately;
- thread owner/side context into `Self` lowering;
- ensure inherited member views substitute receiver environment before method-local inference.

**Runtime invariant test.** Adding generic metadata does not alter runtime superclass/class/metaclass edges.

## 6.6 S04-E — transparent aliases

**Implementation.**

- create semantic alias declaration identity for provenance/navigation;
- lower generic alias binders/constraints;
- normalize/expand transparently for semantic equivalence;
- maintain cycle detection/budget;
- fingerprint the public alias interface structurally;
- keep compiler runtime lowering as no-op.

**Deletion criterion.** `Statement::TypeAlias` is no longer merely “parsed then ignored by runtime”; it has an authoritative semantic declaration product.

## 6.7 S04-F — type-form values

**Implementation.**

- handle `Expr::TypeForm` explicitly in expression analysis;
- preserve two-axis fact:

```text
value type of reflected type-form value
semantic denotation of canonical form
```

- compile/materialize a runtime descriptor root only when the value is actually required at runtime;
- compile-time-elided semantic use allocates no descriptor;
- reject non-type-form origins through explicit source-lowering outcome.

**Tests.**

```phalcom
const t = List<Int>
const f = <T> =>> Result<T, Error>
```

plus `<`, `>`, `>>`, and adjacency regression cases.

## 6.8 S04-G — record-tail handoff

Until Spec-05 Workstream B lands, source row syntax may exist in the AST but must not be published as a false closed record.

**Immediate repair.** If `TypeAnnotationExpr::Record` contains a tail and the semantic row domain is not enabled, return an explicit blocked/feature-unavailable lowering outcome with a diagnostic. Do not drop `tail` and publish a closed record.

---

# 7. Workstream C0 — canonical source/native callable convergence

This is the ownership bridge between Spec 04 and 04.5.

## 7.1 Goal

Replace the current split:

```text
native method
    -> CallableSignatureTable / CallableSemanticSignature

source method
    -> MemberSurface.callable_signatures / dispatch::CallableSignature clone
```

with:

```text
source/native/generated/intrinsic declaration
        ↓
CallableId
        ↓
CallableSignatureTable[CallableId]
        ↓
selector surface maps owner/side/selector -> CallableId
        ↓
lookup returns SpecializedCallableView
        ↓
ordinary call checker
```

## 7.2 Primary files

```text
phalcom-semantic/src/signature.rs
phalcom-semantic/src/surface.rs
phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/checker/declaration.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
```

## 7.3 Migration stages

### Stage C0.1 — dual-write source signatures

Publish source signatures to `CallableSignatureTable` while retaining the legacy concrete surface for old checker callers.

Gate: differential tests prove the legacy materialized signature equals materializing the canonical signature/view.

### Stage C0.2 — selector surfaces return identity

Make normal dispatch resolution return:

```rust
CallableId
+ receiver specialization environment
+ lookup provenance
```

rather than a cloned concrete signature.

### Stage C0.3 — lazy specialized view

Use `TypeEnvironment` / `SpecializedCallableView` for receiver generic substitution. Eager `TypeSubstitution::apply` remains only as explicit materialization for serialization/debug/testing or cold compatibility boundaries.

### Stage C0.4 — retire duplication

Remove or collapse duplicated `MemberSurface` maps:

```text
callables return facts
callable_signatures by selector
callables_by_selector
```

into canonical ID mapping plus canonical tables.

## 7.4 Deletion criteria

Delete `dispatch::CallableSignature` as formal authority when:

- all source/native callables exist in `CallableSignatureTable`;
- every dispatch path can resolve `CallableId`;
- specialized signature materialization is behaviorally equivalent;
- checker, LSP formal consumers, metadata exporter, and reflection use canonical IDs/tables;
- no test directly requires old cloned signature identity.

---

# 8. Workstream Q0 — activate compiler-owned `SemanticDb`

The DB infrastructure is present; the missing work is making it the execution owner rather than a disconnected substrate.

## 8.1 Current state

`phalcom-semantic/src/db/` already provides:

```text
QueryKey
QueryState
QueryOutcome
DependencyEdge
reverse dependency index
scheduler
budget
metrics
revision management
```

and includes:

```rust
QueryKey::CallableBody(CallableId)
```

The active `analyze_workspace` still builds a fresh store and runs the whole workspace through phases in one function.

## 8.2 Target DB inputs

Add/finish typed input ownership for at least:

```text
workspace/project registry
source text/revision
parsed module product
unlinked interface
linked interface
project/import resolution result
semantic component graph
canonical declaration shell/signature
native catalog fingerprint
analysis configuration / budgets
```

Inputs must be fingerprinted structurally. LSP document revision IDs are adapters, not compiler semantic identity.

## 8.3 Typed query products

The current generic query-value storage is acceptable as scaffold; production APIs should expose typed products at the semantic boundary.

Recommended target:

```rust
pub enum SemanticProduct {
    ParsedModule(Arc<ParsedModuleUnit>),
    UnlinkedInterface(Arc<...>),
    LinkedInterface(Arc<...>),
    DeclarationShell(Arc<...>),
    SemanticComponent(Arc<...>),
    DeclarationSurface(Arc<...>),
    CallableBody(Arc<CallableAnalysis>),
    ModuleDiagnostics(Arc<[SemanticDiagnostic]>),
    ModuleMetadata(Arc<...>),
    // Spec 05 products remain separate query keys/products.
}
```

Exact Rust representation is an **implementation choice**; typed API behavior is required.

## 8.4 Workspace migration

Refactor `analyze_workspace` into either:

- a compatibility convenience wrapper that seeds a long-lived `SemanticDb` and requests products; or
- a cold one-shot driver built on exactly the same query functions as the long-lived service.

It must not remain a second semantic algorithm.

## 8.5 Partial failure publication

Replace LSP skip/`None` behavior with compiler-owned module states:

```text
Complete
Invalid { phase, diagnostics }
Blocked { phase, reason }
Cancelled        // not published as completed state
BudgetExceeded   // not published as successful state
InternalFailure  // not published as semantic success
```

Project load, interface build, import resolution, link failure, and runtime dependency cycle must produce source/project-owned diagnostics when possible.

A runtime dependency cycle remains a hard link/runtime-order error. No sorted fallback and no silent loss of the static snapshot.

## 8.6 Invalidation laws

- no-op update: no semantic product recomputed unless an explicitly volatile input changed;
- body-only edit with unchanged callable interface: invalidate changed `CallableBody` and its actual reverse body/advanced dependents, not unrelated interfaces;
- signature edit: invalidate declaration surface/interface fingerprint and exact reverse semantic closure;
- native catalog fingerprint change: invalidate native-dependent declaration surfaces/call bodies, not arbitrary source parsing;
- cancelled generation: never replaces the prior published snapshot.

## 8.7 Deletion criterion

The old whole-workspace phase driver ceases to be the only formal path. Compiler/CLI/REPL/LSP all reach semantic truth through DB-owned query products.

---

# 9. Workstream E — 04.5 executable-expression engine

This section consumes 04.5; it does not redesign it.

## 9.1 04.5 completion matrix at current `main`

| Capability | Status | Current evidence / gap |
|---|---|---|
| `InferVarId` separate Rust ID | **Partial** | Separate ID exists, but `TypeData::Infer` canonicalizes it. |
| Session-local inference | **Not implemented** | `LocalConstraintSolver` calls `TypeStore::infer`. |
| Bidirectional synthesis/check | **Not implemented** | formal API is synthesis-oriented. |
| Generic method call inference | **Not implemented** | `match_callable_arguments` consumes concrete signature. |
| Expected-result inference | **Not implemented** | no expected-result constraint collection. |
| Receiver specialization | **Partial** | applied receiver substitution exists, eagerly materialized. |
| `Self` call specialization | **Partial** | canonical term exists; source/call pipeline incomplete. |
| Union receiver member/call lookup | **Not implemented formally** | no all-reachable-arm algorithm in call checker. |
| Declared vs current binding type | **Not implemented** | local environment stores one changing fact. |
| Straight-line mutable reassignment semantics | **Incorrect current behavior** | assignment first checks against current inferred fact before overwriting. |
| Branch flow join | **Partial expression-only** | branch expression result union exists; no path-state join. |
| Loop fixed point | **Not implemented formally** | LSP advisory engine has precedent. |
| Protocol-derived `for` typing | **Not implemented / incorrect fallback** | loop binding gets explicit `Dynamic`. |
| Expression identity | **Not implemented** | no body-local `ExpressionId`. |
| ExplanationGraph | **Not implemented** | only small evidence/provenance structures. |
| Causal diagnostic suppression | **Not implemented** | flat diagnostics; no cause graph. |
| Structured fixes/notes/help | **Not implemented in semantic truth model** | renderer/diagnostic split incomplete. |
| `CallableBody` DB product | **Scaffold** | query key exists, product does not. |
| Formal LSP migration | **Partial** | static snapshot coexists with independent advisory flow engine and LSP-owned static workspace builder. |

## 9.2 E1 — expression identity and causal analysis model

**Create.**

```text
phalcom-semantic/src/checker/analysis.rs
```

**Target concepts.**

```rust
ExpressionId { owner: BodyId/CallableId, local: LocalExpressionId }
ExpressionAnalysis
AnalysisStatus
ExpressionAnalysisIndex
DiagnosticCauseId
```

**Identity policy.** Local expression IDs are deterministic within one callable semantic product. They are not promised to survive arbitrary source revisions and are not global query keys.

**Recommended numbering.** Deterministic pre-order over the canonical body AST after parser desugaring but before CFG block scheduling. The exact numbering algorithm is an implementation choice; stability within an unchanged callable is required.

**Tests.** Same unchanged body produces the same expression index; unrelated body edits do not change another callable's IDs.

## 9.3 E2 — binding identity and state split

Use `BindingId` for lexical bindings. Introduce:

```rust
BindingDeclaration {
    id: BindingId,
    declared: Option<ProperTypeId>,
    mutable: bool,
    ...
}

BindingState {
    current: TypeKnowledge,
    ...
}
```

Required semantics:

```phalcom
let x = 1
x = "hello"
```

After the assignment:

```text
x.current = String
```

No implicit permanent `Int` annotation exists.

At a branch join:

```phalcom
let x = 1
if condition { x = "hello" }
```

```text
x.current = Int | String
```

For an explicit annotation:

```phalcom
let x: Number = 1
x = 2.0
```

```text
declared constraint = Number
current fact: Int -> Float
```

## 9.4 E3 — session-local `InferenceSession`

**Create.**

```text
phalcom-semantic/src/checker/inference.rs
```

**Target state.**

```text
InferVarId -> kind
lower candidates
upper candidates
exact/equality candidates
solution
provenance
occurs-check state
budget/cancellation
```

New inference must never allocate `TypeData::Infer`.

Migrate in order:

1. empty list/set/map;
2. generic call inference;
3. contextual block/closure unknowns;
4. any residual local constraint tests/callers;
5. all `fresh_var` uses across the workspace.

Only a solved/materialized canonical type is interned back into `TypeStore`.

**Hard deletion gate.** Workspace search for:

```text
TypeData::Infer
TypeStore::infer
LocalConstraintSolver
fresh_var
```

must show no legitimate production semantic use before deleting the old variants/APIs.

## 9.5 E4 — bidirectional checking

**Create.**

```text
phalcom-semantic/src/checker/expected.rs
```

Target entry points:

```rust
analyze_expression(ctx, flow, expr, expected)
synthesize_expression(...)
check_expression(..., ExpectedType)
```

Expected types participate in:

- empty collections;
- closure parameters/results;
- return expressions;
- generic calls;
- expected-result generic inference.

Expected types never alter runtime selector identity.

## 9.6 E5 — canonical call resolution

Refactor `checker/call.rs` around:

```rust
CallResolutionOutcome
CallResolution
ReceiverSpecialization
MethodInferenceSolution
CallShapeFailure
```

Canonical sequence:

```text
type receiver
    ↓
resolve declaration/member by selector
    ↓
build receiver specialization environment
    ↓
retrieve CallableId + canonical signature
    ↓
instantiate method-local generic variables
    ↓
collect argument constraints
    ↓
collect expected-result constraints
    ↓
contextually check blocks/closures as constraints become available
    ↓
solve InferenceSession
    ↓
validate substituted where constraints
    ↓
check arguments exhaustively
    ↓
specialize result
```

Receiver generic bindings and method inference variables are separate environments and only compose through explicit substitution/viewing.

## 9.7 E6 — exhaustive relation-policy adapter

Create one shared checker policy function for relation outcomes so assignments, returns, call arguments, fields, and constraints do not each invent their own interpretation.

Only `Refuted` produces an ordinary type contradiction. Other terminal states propagate according to their category.

---

# 10. Workstream F — compiler-owned flow semantics

## 10.1 F1 — flow graph

Create compiler-owned flow graph/CFG products in `phalcom-semantic` from callable bodies. Preserve actual Phalcom control semantics, including:

- normal branch edges;
- short-circuit sends/control forms as represented after parser lowering;
- `return` and non-local block return;
- `break`/`continue`;
- `throw`/terminal raises;
- loop back edges;
- unreachable successors.

Do not copy the LSP `ValueShape` data model. Reuse algorithmic lessons only.

## 10.2 F2 — FlowState and direct predicates

Target separation:

```text
BodyAnalysisContext    stable body/services/declarations
FlowState              path-local binding types/facts/reachability
InferenceSession       local unknowns for one inference problem
```

Initial direct fact domain should include at least:

```text
x is T
x is not T
x == None
x != None
x == literal
x != literal
simple ordered predicates such as amount > 0
```

These are recorded direct facts. General implication proving remains Spec 05.

## 10.3 F3 — joins and loops

At a join:

- current binding types join conservatively;
- declared/base constraints remain unchanged;
- facts survive only when valid on every reachable predecessor;
- unreachable predecessors do not weaken the join.

Loops use bounded deterministic fixed-point iteration plus widening. Budget exhaustion yields `BudgetExceeded`/blocked analysis, never a fabricated stable type.

## 10.4 F4 — mutation invalidation

Writes kill directly dependent facts. Unknown/opaque calls conservatively kill mutable projection facts that could have been invalidated. Later Spec-05 effect summaries may preserve more facts but are not required for sound 04.5 behavior.

## 10.5 F5 — protocol-derived iteration typing

**Current defect.** `checker/statement.rs` binds `for` patterns as `Dynamic`.

**Target operation.**

```text
receiver R
    ↓
resolve R.iterate(_)
    ↓
derive cursor knowledge C
    ↓
resolve R.iteratorValue(C)
    ↓
derive element knowledge E
```

No hidden `Iterable<T>` primitive and no `.each` special rule.

If the formal surface cannot establish the element type, return `Unknown`/blocked reason. Use `Dynamic` only if the receiver or protocol crosses an explicit dynamic boundary.

**Conformance test.** Compare formal element inference with the compiler/runtime lowering protocol in `phalcom-core/src/compiler/lib/expr.rs` and `for` lowering code.

---

# 11. Workstream X/D — explanations, causal diagnostics, and rendering

## 11.1 X1 — ExplanationGraph

Create a compact arena/DAG of semantic derivation nodes. It should retain structured evidence for conclusions such as:

```text
expression synthesis
expected-type checking
member selection
receiver substitution
method generic solution
subtype/assignability relation
where constraint result
flow narrowing
join
```

Store IDs/edges/semantic operands, not formatted prose.

Recommended concepts:

```rust
ExplanationId
ExplanationNode
EvidenceRef
DerivationRule
RelationEvidenceRef
```

Canonical type identity is independent of explanation identity.

## 11.2 D1 — causal suppression

Model expression/body statuses so a root invalid expression can block dependents without generating one error per dependent send/field/return.

Required semantic categories include:

```text
Known/Ready
Unknown
Dynamic
Invalid { cause }
Blocked { cause }
Cancelled
BudgetExceeded
InternalFailure
```

Suppression must be causal, not “only one error per line”. Independent contradictions still emit independently.

## 11.3 D2 — structured diagnostics

Extend `SemanticDiagnostic` to carry:

```text
code
severity
primary label
secondary labels
notes
help
structured fixes
root cause / explanation reference
```

Rendering is a projection. The normal view uses the smallest useful slice; `--explain`/LSP tooling may walk deeper explanation edges.

## 11.4 D3 — extract VM-free shared renderer

Create, after behavior-preserving tests:

```text
phalcom-diagnostics/
  src/style.rs
  src/snippet.rs
  src/labels.rs
  src/report.rs
```

Move/adapt the mature core traceback visual primitives rather than introducing `miette` or a parallel visual language.

**Migration order.**

1. golden/structural tests around current runtime traceback/caret behavior;
2. extract VM-independent text/style/snippet substrate;
3. adapt `phalcom-core` runtime traceback through it;
4. add semantic report rendering;
5. route CLI static diagnostics through the shared substrate;
6. delete core-local duplicated style/caret implementation only after parity.

---

# 12. Workstream Q1 — callable-body products and incremental publication

## 12.1 Product shape

Recommended `CallableAnalysis`:

```rust
pub struct CallableAnalysis {
    pub callable: CallableId,
    pub expressions: ExpressionAnalysisIndex,
    pub bindings: BindingTable,
    pub flow_graph: Arc<FlowGraph>,
    pub entry_flow: FlowStateSummary,
    pub exits: BodyExitFacts,          // ordinary body facts; advanced ExitSummary remains Spec 05
    pub diagnostics: Arc<[SemanticDiagnostic]>,
    pub explanations: Arc<ExplanationArena>,
    pub dependency_fingerprint: ProductFingerprint,
}
```

Do not embed Spec-05 effects/termination/proof result into this object merely for convenience. Those are independent queries keyed by `CallableId`.

## 12.2 Query dependencies

`CallableBody(callable)` should depend on:

```text
callable canonical signature
owner declaration surface
referenced declaration surfaces
referenced callable signatures
relevant source/body fingerprint
native catalog fingerprint only when native members are referenced
analysis configuration / semantic model version
```

Flow facts are part of the body product. Advanced Spec-05 queries depend on the body product, not vice versa except where 04.5 explicitly consumes optional precision summaries.

## 12.3 Publication

A cancelled/budget-exhausted computation is never published as a ready callable analysis. Existing previous-generation products remain visible until an atomic new snapshot is valid.

## 12.4 Incremental acceptance

A body-only edit must not unconditionally rebuild:

```text
unrelated parsed modules
unrelated linked interfaces
unrelated declaration shells
unrelated callable analyses
metadata roots unrelated to the changed interface
```

Cold and incremental public semantic results must be structurally equivalent.

---

# 13. Workstream L — LSP formal migration

## 13.1 Current state

The LSP already wraps a compiler static snapshot while retaining its own:

```text
source catalog/project discovery
interface/linking path
flow.rs
infer.rs
facts.rs
module graph/invalidation machinery
ValueShape/callable summaries
```

This is a migration architecture, not the final formal ownership model.

## 13.2 Migration sequence

1. **Compiler DB owns source/project/module formal products.** LSP provides overlay input and requests a semantic snapshot.
2. **Compiler DB owns body formal products.** LSP reads `CallableAnalysis` for types, narrowing, formal call targets, and diagnostics.
3. **Parity harness.** For a representative corpus, compare compiler formal facts against existing LSP outputs where the concepts overlap.
4. **Move consumers.** Hover, signature help, completion filters, inlay hints, diagnostics, and navigation use compiler-owned formal products.
5. **Delete duplicate formal behavior.** Keep editor-only `ValueShape` heuristics explicitly advisory.

## 13.3 What may remain in LSP

- UI scheduling/debouncing;
- document overlays/URI mapping;
- presentation caches;
- editor-specific runtime-shape heuristics not claimed as formal types;
- speculative completion heuristics clearly separated from compiler rejection semantics.

## 13.4 Deletion criterion

No language-validity decision or formal type/narrowing rule exists only in `phalcom-lsp/src/semantic/flow.rs`, `infer.rs`, or `facts.rs`.

`run_static_workspace_analysis` no longer independently rebuilds the formal project/link/analyze pipeline. It becomes an adapter to compiler DB inputs or is deleted.

---

# 14. Workstream N — canonical native/source convergence cleanup

## 14.1 Current architecture

The following target pieces are real:

```text
#[primitive(...)] authoring metadata
      ↓
phalcom-native-decl normalized declarations
      ↓
phalcom-native-surface-gen
      ↓
checked-in/generated NATIVE_SURFACES
      ↓
phalcom-semantic native signature import
```

The runtime descriptor registry `PRIMITIVES` also exists.

The migration is not complete because:

- `phalcom-native-surface::NATIVE_MEMBERS` remains a compatibility inventory;
- `phalcom-lsp/src/semantic/core_source.rs` still references it;
- `VM::new()` defaults to `NativeInstallMode::Dual`;
- `Universe::install_primitives` remains the compatibility floor when descriptor census is incomplete;
- `spec03_5_census` reports set differences but does not yet require exact equality.

## 14.2 N1 — convert census to a deletion gate

Strengthen `phalcom-core/tests/spec03_5_census.rs` in stages:

1. generated surface keys are unique/canonical;
2. descriptor keys are unique/canonical;
3. all executable native rows required by the canonical surface have descriptors;
4. legacy-only set becomes empty;
5. descriptor-only/generated-only differences are either explicitly classified non-executable presentation rows or empty;
6. final exact executable census equality is asserted.

Do not delete the legacy installer before this test is an assertion rather than informational `eprintln!` output.

## 14.3 N2 — retire `NATIVE_MEMBERS`

Move all remaining consumers to `NATIVE_SURFACES` or a generated compatibility projection derived mechanically from it. The final repository must not require humans to maintain both tables.

## 14.4 N3 — descriptor-only runtime boot

After census equality and runtime conformance tests:

- make `DescriptorOnly` the default;
- run full runtime invariance/floor tests;
- remove `Universe::install_primitives` native method list;
- remove dual-install replacement behavior whose only purpose was migration.

## 14.5 N4 — LSP provenance cleanup

Replace native fake/source-sentinel AST identities with explicit member provenance. Native methods do not have a source AST node. Source wrappers and native implementations may share one semantic `CallableId` with distinct implementation provenance.

---

# 15. Runtime reflection correctness matrix

This matrix is a required release gate for the already-landed reflection platform.

| Capability | Current state | Required final behavior | Gate/tests |
|---|---|---|---|
| `Int.kind` | likely correct by zero-arity fallback | `Type` | direct runtime reflection test |
| `List.kind` | hard-coded correct | reflected `FunctionKind(Type -> Type)` from metadata | remove hardcode; result unchanged |
| `Set.kind` | hard-coded correct | reflected `FunctionKind(Type -> Type)` from metadata | remove hardcode; result unchanged |
| `Map.kind` | hard-coded correct | reflected `FunctionKind(Type -> Type -> Type)` from metadata | remove hardcode; result unchanged |
| `Option.kind` | **wrong** | `Type -> Type` | mandatory regression test |
| `Some.kind` | **wrong** | `Type -> Type` | mandatory regression test |
| other generic declarations | unreliable unless hard-coded | declaration metadata determines kind | enumerate canonical universe declarations |
| partial application | semantic support exists | residual kind + remaining parameters reflect canonical form | `Map<String>.kind == Type -> Type`-equivalent observation; identity/equivalence tests |
| type lambda `.kind` | schema/reifier support exists | arrow kind from lambda parameter kinds/result | alpha-equivalent lambda test |
| `FunctionKind` projection | implemented | observational descriptor only | inspect arguments/result; GC/identity tests |
| public `applyKind` | absent | **remain absent** | API census proves no selector |
| `equivalent` | implemented | canonical equivalence | structural cases |
| `subtype` | placeholder equivalence | canonical bounded subtype | differential compiler/runtime metadata relation corpus |
| `assignable` | placeholder subtype/equivalence | canonical assignability policy | dynamic-boundary cases |
| `consistent` | placeholder | canonical consistency | symmetry/dynamic cases |
| `conforms` | placeholder | canonical conformance or honest unavailable until protocol semantics | no false satisfied result |
| static member lookup | materially implemented | canonical `CallableId`/specialized signature view, visibility preserved | generic/native/source tests |
| `matches` | implemented shallow boundary | only cheap runtime evidence | erased generic must not falsely prove element args |
| `validate` | implemented explicit boundary | bounded deep validation only when evidence permits | no ambient validation hooks |

**Important:** no public standalone `applyKind` is implied by the existence of `FunctionKind` or the source kind syntax `Type -> Type`.

---

# 16. Spec-05 advanced semantic program

Spec 05 is implemented only after the ordinary body/query substrate it consumes is stable. The products remain independently queryable and independently invalidated.

## 16.1 Current Spec-05 completion matrix

| Domain | Current state | Existing substrate | Remaining work |
|---|---|---|---|
| Record-row kind | **Partial substrate** | `KindData::RecordRow`, kind metadata | canonical row term/tail, row variables, lacks, row relations/solver, source publication |
| Effects | **Native declarations only** | `EffectSpec` on native surfaces | compiler-owned atoms/knowledge, direct source inference, call/SCC propagation, opacity |
| Exits | **Native declarations only** | `RaisesSpec`, `ReturnFlowSpec` | canonical exit summary, CFG composition, handlers, call propagation |
| Termination | **Not implemented** | none authoritative | taxonomy, CFG/call graph/ranking evidence, `@total`, native termination metadata |
| Contracts | **Runtime guards only** | compiler weaving, `MethodObject.contracts` | stable `ContractId`, canonical contract IR, `old`, result/pre-state semantics, static eligibility |
| VCs | **Not implemented** | none | deterministic backend-free proof IR/generator |
| Proof outcomes | **Not implemented** | reflection/metadata seams only | result algebra + causal reasons |
| Proof trust | **Not implemented** | conceptual capability/profile | trust policy, backend identity, certificate policy |
| Proof artifacts | **Not implemented** | metadata extension envelope | exact fingerprints/cache/stale rejection/hostile decode |
| Public kind polymorphism | **Deferred** | explicit arrow kinds/type lambdas already exist | separate use-case/design gate only |

## 16.2 A — record rows

**Prerequisites.** S04 generic binder publication; canonical `RecordRow` kind already exists.

**Create/extend.**

```text
phalcom-semantic/src/rows/*        # exact module split may follow Spec 05
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/metadata/export.rs
phalcom-type-meta/*
```

Target concepts:

```text
RecordTail::{Closed, Parameter(TypeParameterId)}
RecordRowVarId            query-local only
row equation/constraint state
lacks constraints
row occurs check
read/write capability-aware record relation
```

Metadata already has a versioned `OpenRecord` representation; do not invent a second durable row graph.

**Gate.** `RecordRowVarId` never becomes `TypeId` or durable metadata.

## 16.3 B — effects

Independent query:

```text
callable_effects(CallableId)
```

Sequence:

1. define canonical effect atoms matching current native declarations;
2. distinguish known empty from unknown/opaque;
3. adapt native `EffectSpec` losslessly;
4. infer source direct effects from body/call facts;
5. propagate through call graph SCCs deterministically;
6. represent dynamic/reflection/FFI opacity explicitly;
7. add budgets/cancellation/dependencies;
8. add metadata/reflection extension.

Do not use `compiler::attributes::is_pure_expr` as formal purity proof.

## 16.4 C — exits

Independent query:

```text
callable_exits(CallableId)
```

Keep normal return possibility, raises, divergence/process exit/suspension as the Spec-05 model requires. Native `ReturnFlowSpec::Never` means no normal return, not termination or divergence.

## 16.5 D — termination and `@total`

Independent query:

```text
callable_termination(CallableId)
```

`@total` requires proven termination only. It does not imply purity or normal return.

Start with backend-free CFG/call-graph reasoning and simple ranking evidence. Unknown complex recursion remains unknown.

## 16.6 E — canonical contracts

Extract one source contract identity/IR before runtime-only closure lowering so runtime guards and static verification project the same source contract without sharing authority.

Required:

```text
ContractId
requires / ensures / invariant kind
canonical predicate IR
old(pre-state)
result reference
source spans
eligibility status
```

Runtime Debug/Release/Unchecked weaving behavior remains unchanged unless its own runtime spec changes.

## 16.7 F — VC generation

Implement backend-free deterministic VC generation first. Unsupported operations produce an explicit unsupported/unknown obligation state; they are never omitted from the formula to make it easier to prove.

## 16.8 G — backend, trust, and artifacts

Only after deterministic VCs:

```text
backend request/response
resource limits
raw verdict taxonomy
trust policy
counterexample validation
proof result variants
artifact fingerprints/cache
optional certificate checker
```

No backend verdict becomes `KernelChecked` without a local certificate checker.

## 16.9 H — ecosystem integration

Compiler/CLI/LSP/REPL/runtime reflection request advanced products on demand. Ordinary type lookup does not trigger proof generation.

---

# 17. Explicit migration and deletion ledger

Nothing in this table is deleted because the new architecture “looks cleaner.” Every deletion requires measurable parity.

| Transitional item | Final state | Compatibility period | Deletion gate |
|---|---|---|---|
| `TypeData::Infer` | no canonical inference node | until all inference callers migrate | workspace search has no production use; no metadata/snapshot/test requires it; inference does not increase store count |
| `TypeStore::infer` | removed | same as above | deleted with `TypeData::Infer` |
| `LocalConstraintSolver` | `InferenceSession` | until collection/call/block inference migrated | no production caller; deterministic/occurs/kind/budget tests pass |
| old `TypeConstraint` local solver vocabulary | inference-specific constraints/provenance | may survive test adapter briefly | no formal call checker consumes it |
| monolithic `CheckingContext` | `BodyAnalysisContext` + `FlowState` + local `InferenceSession` | wrapper during staged checker migration | no mutable path/inference state stored in long-lived body context |
| string-keyed `LocalEnv` | `BindingId` binding table + flow state | until all local lookup/assign paths migrate | formal body semantics use IDs exclusively |
| old `TypedExpression` shape | `ExpressionAnalysis` | wrapper may expose `.knowledge` | formal diagnostic/call/flow consumers use analysis IDs/status/explanations |
| `dispatch::CallableSignature` cloned authority | `CallableId` + canonical signature/view | dual-read while source signatures migrate | source/native differential parity + no formal consumer |
| duplicate `MemberSurface` signature/return maps | ID-indexed surfaces + canonical tables | dual-write during migration | all lookups resolve identity/table |
| `match_callable_arguments` | canonical `resolve_call` | until bidirectional/generic calls land | generic/native/source call corpus passes |
| repeated source annotation lowering in body checker | consume published signatures | until S04 publication complete | no body query re-resolves declaration signature syntax |
| eager `TypeSubstitution` hot-path specialization | `TypeEnvironment`/views | materialization helper may remain | no hot lookup/hover/call repeatedly rebuilds trees |
| coarse `TypeFormResolution::{Known,Dynamic,Unknown}` | explicit lowering outcome algebra | adapter during parser migration | invalid written forms never become unannotated unknown |
| native `phalcom-type-syntax::TypeExpr::Unknown` as if it were a semantic type | explicit native opacity/missing/invalid status at lowering boundary | parser compatibility may decode it | canonical/native surface output never interns or exports it as a proper type form |
| `KindSyntax::Invalid -> Type` recovery | explicit invalid result | none after migration | negative tests prove no successful publication |
| row tail ignored in annotation lowering | blocked or real row term | immediate repair | no open-row syntax silently becomes closed record |
| one-shot `analyze_workspace` as independent engine | DB-backed cold driver/wrapper | until compiler/CLI migrated | all formal entrypoints call same query functions |
| LSP `run_static_workspace_analysis` formal linking/checking | compiler DB adapter | parity window | no independent formal project/link/analyze pipeline |
| LSP formal flow/type logic | compiler `CallableAnalysis` | parity window | overlapping formal facts moved; remaining `ValueShape` marked advisory |
| LSP fake native AST sentinels | explicit member provenance | until canonical core merge consumers migrate | no `usize::MAX` native AST identity needed for formal/native members |
| `NATIVE_MEMBERS` handwritten compatibility table | generated `NATIVE_SURFACES` or generated compatibility projection | until runtime/LSP consumers migrate | zero human-maintained duplicate rows |
| `Universe::install_primitives` native list | descriptor-only installer | until exact executable census parity | census asserts equality + descriptor-only VM full regressions pass |
| `NativeInstallMode::Dual` default | descriptor-only default | after census gate | dual path unnecessary; remove when no migration consumer |
| hard-coded `Behavior#kind` arity | metadata-driven projection | immediate bug-fix window | generic declaration reflection matrix passes |
| runtime subtype/assignability/consistency/conformance = equivalence | canonical relation projection | temporary honest-unavailable allowed | differential semantic relation corpus passes |
| recursive `CompiledTypeRef` transitional export, if still consumed | indexed metadata graph only at durable boundary | until all consumers use `phalcom-type-meta` | no artifact/reflection consumer requires recursive adapter |
| core-local diagnostic style/caret implementation | shared VM-free substrate | until runtime rendering parity | runtime traceback goldens + semantic report tests pass |
| runtime contract closure identity as only contract representation | canonical `ContractId` + separate runtime closure projection | until Spec05 contract IR lands | runtime behavior unchanged; static proof identity stable |

---

# 18. Repository ownership map

## 18.1 `phalcom-ast`

Owns:

- source grammar/AST/recovery;
- generic/type-form/alias syntax nodes;
- source ranges;
- no canonical semantic identity.

Must not own:

- `TypeId`;
- inference variables;
- relation rules;
- formal flow facts.

## 18.2 `phalcom-modules`

Owns:

- project/module/source identity;
- source discovery;
- parsed-module products;
- interfaces/linking;
- reference/semantic/runtime graphs;
- runtime-cycle truth at graph level.

Must publish failures rather than let LSP reinterpret invalid projects as standalone modules.

## 18.3 `phalcom-semantic`

Owns:

- canonical type/kind store;
- declaration/callable/field signatures;
- relations;
- semantic query DB;
- executable expression/call/flow analysis;
- explanations/diagnostic truth;
- Spec-05 advanced semantic products;
- metadata export.

No VM objects or runtime heap handles enter this crate.

## 18.4 `phalcom-native-decl`, `phalcom-native-meta`, `phalcom-native-macros`, `phalcom-native-surface`

Own respectively:

- normalized authored native declaration representation/parsing;
- VM-free semantic native metadata vocabulary;
- Rust authoring macro validation/emission;
- generated VM-free tooling/compiler projection.

There must be one authored native declaration, not parallel hand-maintained semantic tables.

## 18.5 `phalcom-type-meta`

Owns durable, store-independent metadata and validation. It transports semantics; it does not solve/check them.

## 18.6 `phalcom-core`

Owns:

- compiled artifact carriage;
- runtime loading/registry/reification;
- explicit runtime validation boundary;
- VM/materialization;
- runtime contract execution;
- runtime diagnostic capture/adaptation;
- native descriptor installation.

It does not own a second static subtype checker by semantic authority.

## 18.7 `phalcom-lsp`

Owns:

- editor source overlays;
- scheduling/debounce/cancellation integration;
- URI/protocol adaptation;
- presentation;
- advisory runtime-shape heuristics.

It does not own formal language rejection/type semantics after migration.

---

# 19. Performance plan and benchmark contract

No performance number in this section is claimed as current performance. The first stabilization task is to establish a reproducible baseline on pinned corpora/hardware/build profile.

## 19.1 P0 — baseline harness before optimization claims

Record:

```text
commit
Rust toolchain
OS/CPU/RAM
build profile
corpus hash
workspace/project size
measurement repetitions
p50/p95/min/max where meaningful
RSS / allocator or process memory metric
TypeStore node counts
query counts/hits/recomputations
```

Corpora must include:

1. universe/std cold analysis;
2. representative small app;
3. representative medium multi-module app;
4. generic-heavy synthetic corpus;
5. deep-union corpus;
6. branch/loop-heavy flow corpus;
7. native-surface-heavy calls;
8. proof/VC corpus once Spec 05 proof work exists.

## 19.2 Cold analysis

Measure end-to-end:

```text
parse
interface/link
semantic declaration publication
body analysis
metadata export when requested
```

Optimization rule: do not improve warm behavior by making cold behavior pathologically expensive without an explicit tradeoff review.

## 19.3 Warm/no-op analysis

Invariant rather than arbitrary percentage target:

> Repeating analysis with identical semantic inputs must recompute zero non-volatile ready products.

Measure query hit/recompute counts to prove this.

## 19.4 Body-only edit

Required invalidation shape:

```text
changed source/body input
→ changed CallableBody
→ exact reverse semantic/advanced dependents
```

Unchanged linked interfaces/declaration surfaces must remain cache hits when their fingerprints are unchanged.

## 19.5 Signature edit

Measure exact reverse invalidation closure. The implementation must not default to total-workspace recomputation when declaration/interface fingerprints identify a smaller dependent set.

## 19.6 Generic-heavy programs

Track:

- inference variables/session;
- relation pairs visited;
- specialized views created;
- canonical types interned;
- call-resolution time;
- memory.

Key invariant after infer cleanup:

```text
temporary inference variable count
    does not directly increase TypeStore node count
```

## 19.7 Deep unions

Track bounded relation work and union-receiver call lookup. Budget exhaustion must terminate with an explicit result. Avoid Cartesian-product behavior without memoization/compression.

## 19.8 Loop fixed points

Track:

- iterations per loop SCC;
- widened facts;
- state size;
- budget exits.

No debug/release difference in fixed-point semantics.

## 19.9 Cancellation

Every long-running body/relation/advanced query checks cancellation at deterministic safe points. Cancelled work publishes no ready product and abandons external proof processes safely.

## 19.10 LSP latency

Do not freeze an arbitrary millisecond SLO before P0. Instead establish latency distributions for:

```text
hover
completion semantic filter
signature help
inlay hints
publish diagnostics
why-this-type / explain query
```

Then ratify product SLOs from measured baseline and editor UX requirements. The architectural gate now is that an unrelated body edit must not force workspace-wide formal recomputation before answering a local query.

## 19.11 Memory and TypeStore growth

Measure:

- live TypeStore entries by form;
- snapshot retention;
- query product bytes/counts;
- explanation arena size;
- flow-state sharing;
- repeated edit/revert cycles;
- runtime typing descriptor weak-cache behavior.

TypeStore compaction threshold remains data-driven. Do not freeze a constant before profiling.

## 19.12 Proof-cache behavior

Once proof artifacts exist, cache key must cover at least:

```text
canonical VC fingerprint
assumptions
referenced interface fingerprints
semantic-model version
proof backend/version
proof policy/trust inputs
proof-kernel/certificate version when applicable
```

A body/signature/dependency change invalidates only affected obligations. No cache hit may upgrade trust.

---

# 20. Test and conformance strategy

## 20.1 Focused unit tests

Add/maintain narrow tests for:

- type store/proper-kind invariants;
- generic binder/kind/constraint lowering;
- capture-safe type lambdas;
- relation terminal states;
- `InferenceSession`;
- call solving;
- FlowState joins/narrowing/widening;
- ExplanationGraph slicing;
- metadata validator;
- runtime kind/relation projection.

## 20.2 Semantic integration tests

Source-driven programs should cover declaration publication through body analysis, not only manually constructed `TypeStore` nodes.

Required files/targets may include new registered tests named along the 04.5 plan:

```text
spec04_5_expression_analysis
spec04_5_inference_session
spec04_5_bidirectional
spec04_5_generic_calls
spec04_5_flow
spec04_5_diagnostics
spec04_5_incremental
```

Use the repository's explicit Cargo test registration if a crate disables autotests.

## 20.3 Differential cold/incremental tests

For each mutation scenario:

1. analyze cold after edit;
2. analyze incrementally from prior revision;
3. compare store-independent public semantic products, diagnostics, call targets, flow summaries, and advanced summaries;
4. compare fingerprints/dependency closure as appropriate.

Raw `TypeId` equality across store epochs is not required.

## 20.4 Compiler/LSP convergence tests

For the same document snapshot, compare:

```text
diagnostic code/severity/primary span/related spans
formal expression type
generic call target + inferred arguments
flow narrowing
member/call resolution
```

LSP advisory `ValueShape` may add hints but may not contradict formal truth.

## 20.5 Runtime invariance tests

Typing features must not change:

```text
selector encoding
method lookup result
inline-cache key semantics
class/metaclass identity
instance layout
allocation path
ordinary value.class
```

Run existing object-model/native floor tests after each relevant migration.

## 20.6 Metadata/native parity tests

Required:

- semantic metadata fresh-store determinism;
- schema compatibility/hostile input;
- native generated catalog fingerprint determinism;
- generated/descriptor/legacy census during migration;
- final exact descriptor parity;
- source/native equivalent signature normalization;
- method implementation provenance side-table behavior.

Current useful test anchors include:

```text
phalcom-semantic/tests/metadata_export.rs
phalcom-semantic/tests/core_surface_conformance.rs
phalcom-semantic/tests/db.rs
phalcom-type-meta/tests/schema_compat.rs
phalcom-core/tests/spec03_reflection.rs
phalcom-core/tests/spec03_5_conformance.rs
phalcom-core/tests/spec03_5_census.rs
```

These are test-intent anchors, not freshly executed evidence in this document.

## 20.7 Diagnostic tests

Separate:

- structural semantic diagnostic tests: code, spans, notes, fixes, cause/explanation IDs;
- renderer golden tests: visual layout;
- LSP adaptation tests: URI/range/related-information correctness.

Do not bind semantic correctness to exact ANSI spacing.

## 20.8 Negative and mutation tests

Required because migration bugs often preserve happy paths:

- kind mismatch;
- invalid open row;
- underconstrained generic;
- contradictory constraints;
- dynamic boundary;
- budget/cancel/internal failure;
- stale snapshot;
- method-table mutation after reflected signature;
- corrupted metadata/proof artifacts;
- alias cycle;
- generic inheritance cycle;
- native catalog drift.

## 20.9 Fuzz/property tests

Targets:

- type parser/recovery;
- metadata decode/validation;
- canonicalization/equivalence laws;
- relation termination;
- type-lambda alpha/capture laws;
- row solver occurs/lacks behavior;
- diagnostic renderer hostile ranges;
- proof artifact decoder once implemented.

## 20.10 Verification command family

These are required commands to run at the relevant completion gates, not claims already executed:

```bash
cargo fmt --check
cargo check --workspace
cargo test -p phalcom-ast
cargo test -p phalcom-modules
cargo test -p phalcom-semantic
cargo test -p phalcom-type-meta
cargo test -p phalcom-native-decl
cargo test -p phalcom-native-meta
cargo test -p phalcom-native-surface
cargo test -p phalcom-core --test spec03_reflection
cargo test -p phalcom-core --test spec03_5_conformance
cargo test -p phalcom-core --test spec03_5_census
cargo test -p phalcom-core
cargo test -p phalcom-lsp
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

If the workspace's explicit test registrations differ at implementation time, use the current registered target names and record the commands in the phase report.

---

# 21. Rollout order and compatibility windows

## Wave 0 — truth repairs

Land R0.1–R0.5. These reduce false success and fix runtime projection bugs without depending on the new checker.

## Wave 1 — source declaration publication + active DB

S04-A through S04-F and Q0 may proceed in parallel with careful file ownership. Their integration gate is “canonical source signatures available before body checking through DB-owned declaration products.”

## Wave 2 — callable convergence + local inference

C0 and E3. Do not build the new generic call solver on legacy cloned signatures or canonical infer nodes.

## Wave 3 — bidirectional body/call semantics

E1/E2/E4/E5/E6. Keep compatibility wrappers so existing compiler integrations can migrate incrementally.

## Wave 4 — formal flow

F1–F5. Parity-mine LSP algorithms; compiler semantics are authoritative from the first production formal flow consumer.

## Wave 5 — explanations/diagnostics/incremental product publication

X/D + Q1. This should land before broad LSP consumer migration so editor behavior gets structured causes rather than another ad hoc diagnostic layer.

## Wave 6 — LSP formal migration

Move consumers, run parity, delete duplicate formal behavior.

## Wave 7 — native/runtime compatibility deletion

Strengthen census, retire `NATIVE_MEMBERS`, switch descriptor-only startup, fix all reflection parity.

Some R0 runtime fixes can land earlier; the *deletions* wait for parity.

## Wave 8 — Spec-05 products

Rows first where needed to make already-ratified syntax semantically usable; effects/exits can proceed once `CallableAnalysis` is stable; termination/contracts/VC/proof build after their prerequisites.

## Wave 9 — stabilization and platform-complete gate

Full differential tests, fuzzing, performance baseline/gates, deletion searches, workspace verification, documentation status update.

---

# 22. Decision register

Status vocabulary:

```text
Ratified
Implementation choice
Open
Deferred
Rejected
```

| ID | Decision | Status | Owner | Reversibility cost | Implementation gate |
|---|---|---|---|---|---|
| DEC-07-STATIC-RUNTIME | Static type metadata never changes selector/dispatch/class/layout/allocation identity | **Ratified** | 01/01.5/06 | Extreme | permanent invariant |
| DEC-07-INFER-ID | `InferVarId != TypeId`; no canonical inference nodes in final store | **Ratified** | 04.5 | High if delayed | E3 deletion gate |
| DEC-07-CONTEXT-SPLIT | Body context, FlowState, and InferenceSession are distinct lifetimes | **Ratified architecture** | 04.5 | Medium | E1/E2/E3/F |
| DEC-07-EXPR-ID | Expression IDs are body-local deterministic identities, not cross-revision durable IDs | **Ratified direction** | 04.5/06 | Low | E1 |
| DEC-07-EXPR-NUMBERING | Use deterministic body-local traversal numbering initially | **Implementation choice** | 07 | Low | E1 tests |
| DEC-07-BIDIRECTIONAL | Expected types participate in local inference | **Ratified** | 04.5 | High | E4/E5 |
| DEC-07-CALL-ORDER | receiver specialization precedes method-local inference; environments stay distinct | **Ratified** | 01.5/04.5 | High | E5 |
| DEC-07-MUTABLE-FLOW | unannotated mutable first value is current flow fact, not hidden annotation | **Ratified** | 04.5/06 | High | E2/F |
| DEC-07-FLOW-OWNER | formal flow lives in `phalcom-semantic`; LSP ValueShape advisory | **Ratified** | 01/04.5 | High | L |
| DEC-07-FOR-PROTOCOL | `for` element typing follows `iterate(_)` / `iteratorValue(_)` | **Ratified** | 04.5 | Medium | F5 |
| DEC-07-FLOW-PROOF-BOUNDARY | 04.5 records direct facts; Spec 05 proves general implications | **Ratified** | 04.5/05 | High | F/X/Spec05 |
| DEC-07-EXPLANATION | semantic derivations use structured explanation graph; rendering slices it | **Ratified** | 04.5 | Medium | X/D |
| DEC-07-CAUSAL-DIAG | dependent failures are causally suppressed | **Ratified** | 04.5 | Medium | D1 |
| DEC-07-DIAG-RENDERER | preserve in-house traceback visual language; extract VM-free substrate | **Ratified direction** | 04.5 | Medium | D3 |
| DEC-07-DB-GRANULARITY | callable body is initial body-analysis query granularity; no global per-expression query explosion | **Ratified** | 04.5 | Medium | Q1 |
| DEC-07-DB-PRODUCT-REP | exact Rust typed query-value representation | **Implementation choice** | 07 | Low/Medium | Q0 |
| DEC-07-TYPESTORE-COMPACTION | compaction/high-water threshold is measured, not frozen now | **Open implementation tuning** | 01/07 | Low | P0 data |
| DEC-07-NATIVE-AUTHORITY | generated canonical native surface is formal authority | **Ratified** | 03.5 | High | N |
| DEC-07-NATIVE-DUAL | dual runtime install is temporary compatibility only | **Ratified migration** | 03.5/07 | Medium | N1/N3 |
| DEC-07-REFLECT-KIND | runtime kind projection consumes metadata; hard-coded builtin arity is invalid | **Ratified consequence** | 03/06 | Medium | R0.4 |
| DEC-07-APPLYKIND | no public general `applyKind` in initial reflection API | **Ratified** | 03/06 | Low | API census |
| DEC-07-REFLECT-REL | runtime relation selectors must project canonical semantics, not equivalence placeholders | **Ratified consequence** | 03/01 | Medium | R0.5 |
| DEC-07-ROW-KIND | `RecordRow` is a distinct domain/kind | **Ratified** | 05 | High | Spec05 A |
| DEC-07-ADV-ORTHOGONAL | return/effects/exits/termination/contracts/proofs are separate query products | **Ratified** | 05 | High | all Spec05 |
| DEC-07-RUNTIME-GUARD | runtime guard success is not static proof | **Ratified** | 05/06 | High | Spec05 E–G |
| DEC-07-PROOF-TRUST | backend verdict does not automatically imply trusted proof | **Ratified** | 05 | High | Spec05 G |
| DEC-07-PROOF-BACKEND | default prover/backend | **Open** | 05/product | Medium/High | after deterministic VC gate |
| DEC-07-KIND-POLY | public prenex kind polymorphism | **Deferred** | 05 | High | separate use-case/design gate |
| DEC-07-GENERIC-DEFAULTS | generic default type arguments | **Deferred** | 01.5 | High after schema | separate design |
| DEC-07-FINITE-SET | finite exact-set generic constraints | **Deferred** | 01.5/04 | Medium | separate design |
| DEC-07-05-5 | create a new 05.5 semantic integration authority now | **Rejected for now** | 07 | Medium authority cost | revisit only if implementation finds a real cross-product contract gap |
| DEC-07-06-5 | create 06.5 merely to restate philosophy | **Rejected** | 07 | Low | 06 already owns rationale |

## 22.1 05.5 / 06.5 evaluation

A new **05.5 — Semantic Integration and Program Knowledge Model** is **not currently required as a prerequisite**.

Why:

- Spec 01 owns query products/snapshots/identity;
- 04.5 owns expression facts/evidence/derivations/explanations;
- Spec 05 explicitly defines independent advanced callable products and their dependencies;
- this Spec 07 can sequence those products without redefining their semantics.

Create 05.5 only if implementation exposes a concrete missing contract such as incompatible product provenance schemas, cross-product snapshot identity, or a required stable `DeclarationKnowledge` API that no existing owner can define without circular authority. If that happens, stop and write the missing integration contract rather than letting Spec 07 silently become a semantic spec.

A **06.5** is not needed. Revised Spec 06 already serves as the long-term rationale/constitutional audit. This plan may restate its invariants as implementation gates without creating another authority.

---

# 23. Risk register

| Risk | Why it matters | Mitigation / gate |
|---|---|---|
| Syntax-complete illusion | Parser/AST may look finished while declarations are not semantically published | S1–S9 matrix; tests must inspect canonical declaration/signature products |
| Canonical infer leak persists through “temporary” helpers | pollutes fingerprints/store/reflection and makes cancellation unsafe | E3 store-count tests + mandatory deletion search |
| Source/native checker divergence | native methods become special semantics path | C0 canonical ID/signature view; shared call corpus |
| Generic substitution allocation explosion | hover/call/member lookup repeatedly rebuilds trees | lazy `TypeEnvironment`/views; benchmark generic-heavy corpus |
| Flow engine duplication | compiler and LSP disagree on narrowing/rejection | parity migration; formal ownership only in semantic crate |
| Open-row syntax silently loses tail | unsoundly changes type meaning | immediate blocked outcome until row semantics; no tail dropping |
| Runtime reflection overclaims relation truth | user runtime libraries receive false semantic answers | R0.5 honest result gate |
| Runtime kind drift | generic class kinds differ between compiler and reflection | metadata-driven `Behavior#kind`; enumerate all generic declarations |
| Legacy native installer becomes permanent | four authority representations return over time | census equality as deletion gate; descriptor-only default |
| Diagnostics renderer fork | compiler/LSP/runtime visual language diverges | VM-free extraction before large semantic renderer buildout |
| Query DB remains unused scaffold | incremental architecture exists only on paper | Q0 makes all formal frontends consume DB functions |
| Over-granular queries | per-expression global cells create scheduling/memory overhead | callable-body granularity first; expression IDs local to product |
| Under-granular invalidation | body edit still recomputes whole workspace | instrumentation + body/signature differential tests |
| Explanation graph memory blowup | provenance can dwarf types | compact arena, sharing, bounded displayed slices, profiling |
| Proof platform contaminates ordinary checking | every hover/build invokes expensive theorem proving | independent demand-driven Spec05 queries |
| Solver/backend trust escalation | backend result treated as proof authority | explicit trust policy; certificate requirement for KernelChecked |
| Performance claims without baseline | optimization work becomes anecdotal | P0 reproducible benchmark/report requirement |
| Stale historical specs influence implementation | old `Type.currentApplication`, `out/in`, kind-poly assumptions return | authority order + decision register + repository searches |

---

# 24. Per-wave acceptance gates

## Gate V0 — truth and projection integrity

Must be true before broad checker work is called stable:

- proper-type release boundary hardened;
- relation terminal states consumed honestly;
- source diagnostics have real owners;
- `Option.kind` / `Some.kind` fixed metadata-first;
- runtime relation selectors no longer label equivalence as subtype/assignability/consistency.

## Gate V1 — Spec-04 semantic publication

- class/method generic signatures published canonically;
- `where` constraints published;
- constructor kinds correct;
- generic superclass templates published;
- `Self` owner context correct;
- type lambdas capture-safe;
- aliases semantically published;
- open-row syntax never silently closed;
- type-form values formally analyzed.

## Gate V2 — canonical callable model

- every source/native callable has `CallableId` + canonical signature;
- selector surfaces map to IDs;
- receiver specialization uses views;
- legacy materialized signature differential parity proven.

## Gate V3 — 04.5 inference/body semantics

- no new canonical infer variables;
- bidirectional checking active;
- expected-result generic inference works;
- underconstraint/ambiguity explicit;
- union receiver checking all arms;
- full relation outcomes propagated.

## Gate V4 — formal flow

- `BindingId` flow state;
- declared/current split;
- branch reachability joins;
- loop bounded fixed point;
- mutation invalidation;
- protocol-derived `for` typing;
- no formal LSP-only narrowing rule.

## Gate V5 — explanation/diagnostics/incremental

- ExplanationGraph retained;
- causal suppression;
- structured diagnostics/fixes;
- shared renderer substrate;
- `CallableBody` DB products;
- body-only invalidation proven;
- cold/incremental semantic equivalence.

## Gate V6 — consumer/native/runtime convergence

- compiler/CLI/REPL/LSP consume published formal snapshot/products;
- LSP formal duplicate logic deleted after parity;
- native descriptor census exact;
- descriptor-only VM default;
- `NATIVE_MEMBERS` human-maintained duplication removed;
- reflection correctness matrix green.

## Gate V7 — Spec-05 rows

Use revised Spec 05 §68 acceptance: canonical row identity/tail kind/lacks/occurs/relation capability/budget/metadata/no row vars published/cold-incremental equivalence.

## Gate V8 — Spec-05 effects/exits/termination

Effects/exits remain independent; `@total` passes false-positive-focused termination suite and never derives from `Never`, purity, timeout, or budget.

## Gate V9 — contracts/proof platform

Canonical contracts, deterministic VCs, honest unsupported states, backend resource limits, trust policy, exact artifacts, stale rejection, and no-false-`Proven` corpus.

## Gate V10 — stabilization

Full workspace verification, fuzz suites, performance baseline/comparison, deletion ledger searches, documentation/status synchronization, and no unclassified compatibility path left.

---

# 25. Final completion definition

The phrase “typing platform complete” should be split into two explicit milestones so advanced proving does not hold ordinary language development hostage and, conversely, ordinary checker completion is not overstated as proof-platform completion.

## 25.1 Foundation-complete: ready for ordinary feature development

Phalcom may move from foundation-building into ordinary language/type-feature development when **all** of the following are true:

1. canonical type/kind/generic forms are publishable and release-safe;
2. source generic declarations, methods, `where`, type lambdas, `Self`, generic supertypes, aliases, and type-form values lower into the canonical semantic model;
3. no solver metavariable is a canonical `TypeId`;
4. source/native/generated methods share `CallableId` + canonical signature/view and one call-typing algorithm;
5. expression checking is bidirectional;
6. generic call inference is local, bounded, expected-result-aware, and explicit about underconstraint/ambiguity;
7. formal flow is compiler-owned and models declared/current binding state, joins, loops, mutation invalidation, and protocol-derived iteration;
8. expression/call/flow conclusions retain structured derivation evidence;
9. diagnostics are causal, structured, source-owned, and rendered through the shared Phalcom visual substrate;
10. `SemanticDb` actively owns declaration/body products, dependency tracking, invalidation, cancellation, and atomic publication;
11. body-only edits do not unconditionally reanalyze the whole workspace;
12. compiler, `phalcom check`, REPL, and LSP consume the same formal semantic products;
13. LSP `ValueShape` is explicitly advisory and no longer owns duplicate formal type rules;
14. native surface has one formal authored/generated authority and the legacy runtime installer has been removed after exact parity;
15. runtime type reflection projects canonical kinds/relations correctly, including `Option.kind`, and no public `applyKind` has been invented;
16. metadata remains store-independent, versioned, validated, lazy, and free of solver IDs;
17. cold and incremental semantic outputs are structurally equivalent;
18. runtime selector/class/layout/allocation invariants remain unchanged;
19. the deletion ledger has no foundation-critical compatibility item left without an explicit deferred owner;
20. full registered workspace verification and the ratified performance/conformance gates have actually been executed and recorded.

At this point, adding a normal language feature should not require another rewrite of semantic identity, query ownership, generic inference, flow ownership, native/source representation, or diagnostic causality.

## 25.2 Full advanced semantic platform complete

Claim the full Specs 01–05 typing/verification platform only when, in addition:

- open record rows meet Spec-05 acceptance;
- effect summaries are compiler-owned and incremental;
- exit summaries are compiler-owned and independent of effects;
- termination knowledge and `@total` meet their acceptance suite;
- runtime/source contracts share canonical semantic identity without conflating guard execution with proof;
- deterministic VC generation exists;
- proof outcomes distinguish Proven/Disproven/Unknown/Blocked/Cancelled/BudgetExceeded/InternalFailure;
- trust policy is explicit;
- proof artifacts use exact dependency/fingerprint validity and hostile-input validation;
- proof generation remains demand-driven and does not impose ordinary runtime allocation/cost;
- advanced metadata/reflection projections preserve the same semantic/runtime authority boundary.

A particular default SMT/prover vendor does **not** have to become part of the language ABI to call the architecture complete. The backend-neutral protocol, result/trust semantics, and artifact-validity model do.

---

# 26. Final execution summary

The current repository does not need another type-kernel rewrite. It already contains much of the difficult canonical infrastructure: kinded type forms, generic parameter identity/variance/constraints, type lambdas, `Self`, partial application, result-rich relations, stable metadata, runtime reification, a query-DB substrate, and a generated native semantic surface.

The remaining risk is **integration debt**: several new canonical layers coexist with older production paths.

The highest-value sequence is therefore:

```text
repair false projections/invariants
    ↓
publish source declarations into canonical generic/signature tables
    ↓
make SemanticDb the active owner
    ↓
remove canonical inference variables
    ↓
replace legacy call checking with bidirectional generic call resolution
    ↓
move formal flow into phalcom-semantic
    ↓
attach explanations and causal diagnostics
    ↓
publish callable-body products incrementally
    ↓
migrate LSP formal consumers
    ↓
delete legacy native/runtime/checker compatibility paths
    ↓
implement independent Spec-05 advanced products
    ↓
measure, fuzz, verify, and stabilize
```

The architectural invariant at the end remains the same as at the beginning:

```text
more information
    ⇒ better checking
    ⇒ better inference
    ⇒ better diagnostics
    ⇒ better tooling
    ⇒ stronger proofs where possible
```

without ever implying:

```text
more static information
    ⇒ different selector
    ⇒ different runtime class
    ⇒ different dispatch semantics
```

That is the completion criterion for Phalcom's typing platform: one precise, incremental, explainable semantic intelligence layer over the existing message-oriented runtime—not a second execution model.
