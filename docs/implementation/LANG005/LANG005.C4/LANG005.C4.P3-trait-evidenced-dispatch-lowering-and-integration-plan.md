---
id: LANG005.C4.P3
category: LANG
program: LANG005
checkpoint: LANG005.C4
kind: implementation-plan
status: PLANNED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG005.C4.P1
  - LANG005.C4.P2
  - LANG005.C3
  - LANG005.C2 applicability-proof-state closure
follows: LANG005.C4.P2
prepared: 2026-09-14
repository: aureat/phalcom-lang
repository_baseline: 1fc1e8449e6dd77fc533455bcb056fe9b00290b9
planning_mode: post-P2-audit
next_checkpoint: LANG005.C5
---

# LANG005.C4.P3 — Trait-Evidenced Dispatch, Lowering, and Integration

> **For agentic workers:** REQUIRED SUB-SKILL: use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Track executable steps with checkbox state, preserve gate order, and review each gate before proceeding.

**Goal:** Complete C4 by making proven trait conformances participate in ordinary instance dispatch, callable references, compiler lowering, and VM execution while preserving `ConformanceEvidence` as the sole semantic authority and never copying trait/conformance behavior into target-owned inherent surfaces.

**Architecture:** P3 begins with a mandatory post-P2 correctness closure, then adds a bounded trait-evidenced candidate index/query keyed from ordinary lookup inputs `(receiver, selector, side)`. Semantic analysis publishes exact per-expression trait dispatch selections carrying conformance evidence; `phalcom-core` projects those selections into internal executable conformance plans, detached witness/default methods, and compact runtime conformance environments. Trait defaults execute with an active conformance environment so abstract requirement calls dispatch through the already-proven requirement map rather than runtime trait search.

**Tech Stack:** Rust workspace; `phalcom-semantic`; `phalcom-core`; Phalcom AST/compiler/VM; semantic snapshots and incremental DB; runtime type-environment registry; existing callable/selector and associated-lowering infrastructure; Rust integration tests and executable Phalcom fixtures.

**Spec:** `docs/specs/objects/traits.md`, `docs/specs/objects/impl.md`, `docs/specs/objects/enums.md`, relevant callable/visibility specs, `docs/specs/vm/bytecodes.md`, `docs/specs/vm/vm-execution-model.md`, `docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md`, `docs/implementation/LANG005/LANG005.C4/LANG005.C4-GUIDANCE.md`, and `docs/implementation/LANG005/LANG005.C4/LANG005.C4.P2-handoff.md`.

## Luna Patch-Grade Implementation Plan

P3 is the closure plan for `LANG005.C4`. P1 established explicit source conformance identity, ownership, exact target/`TraitRef` matching, and global coherence. P2 established conformance-owned witness identity, witness/default selection, conformance completeness, and exact `ConformanceEvidence`. P3 must make that evidence executable through ordinary language syntax without weakening any of the semantic distinctions established by C1–C3 or by C4.P1/P2.

The complete checkpoint boundary is:

```text
C3
    TraitRef
    TraitSurface
    TraitRequirementId
    trait-owned defaults checked under abstract Self
        ↓
C4.P1
    explicit impl TraitRef for Target
    ImplId provenance
    ownership
    coherence
    exact ConformanceHeadMatch
        ↓
C4.P2
    ConformanceWitnessPlan
    witness/default selection
    ConformanceCompleteness
    exact ConformanceEvidence
        ↓
C4.P3
    receiver+selector trait-evidenced discovery
    convergence/ambiguity
    per-expression TraitDispatchSelection
    semantic→executable projection
    detached witness/default compilation
    active conformance runtime environment
    default→requirement execution
    bound references
    vertical runtime certification
        ↓
C5
    associated types/bindings/projection
```

P3 is complete only when realistic programs can define traits, establish legal conformances through mixed witness sources, call trait-provided behavior through ordinary syntax, execute defaults that call abstract requirements/defaults, capture trait-evidenced callables, and produce correct VM results while semantic inspection proves that no trait/default/witness was injected into the target's inherent surface or runtime method dictionary.

---

## 0. Executor Contract

This plan is written for a Luna-class implementer with constrained architectural authority.

The implementer:

- must begin from the landed post-P2 repository and re-ground all referenced symbols before editing;
- must preserve P1/P2 semantic products rather than reconstructing conformance or witness selection in P3;
- must treat the P2 audit findings in §4 as mandatory entry defects to close before ordinary trait-evidenced dispatch consumes P2 evidence;
- may adapt helper names and exact file factoring where the live repository makes a different mechanical placement clearly superior;
- must not merge trait behavior into `DeclarationSurface`, conditional inherent sets, runtime target class method dictionaries, or source target ownership;
- must not introduce runtime class scanning, runtime conformance registration, import-order precedence, or a dynamic global “implements trait” registry as semantic authority;
- must keep C5 associated types and C6 trait constraints/conditional conformance out of scope;
- must use test-first, smallest-distinguishing verification during BUILD mode and expand only at named gates;
- must preserve cold/incremental parity for every new semantic selection/index/dependency product;
- must update the C4 checkpoint at every durable gate;
- must produce a P3 walkthrough, a C5/C6 handoff, and final C4 checkpoint closure before declaring completion.

Do not broadly refactor unrelated compiler/runtime infrastructure. If an architectural change outside the expected impact map becomes necessary, classify it under the STOP/CONSULT rules in §19 before proceeding.

---

## 1. Goal

Implement the executable half of C4 trait conformance:

1. close the post-P2 correctness gaps that would make `ConformanceEvidence` unsafe as an execution authority;
2. generalize callable-owner consumers for `CallableOwnerId::Conformance(ImplId)` without reintroducing fake declaration ownership;
3. create a bounded semantic discovery path from ordinary member-lookup inputs `(receiver, selector, dispatch side)` to all relevant proven trait-evidenced candidates;
4. preserve exact target specialization, exact `TraitRef`, source `ImplId`, `TraitRequirementId`, and selected witness/default in every candidate;
5. define convergence rules for multiple trait candidates and deterministic ambiguity when they do not converge;
6. integrate trait evidence at the canonical ordinary dispatch boundary rather than syntax-by-syntax special cases;
7. publish exact per-expression trait dispatch evidence for lowering, fingerprints, source tooling, and incremental invalidation;
8. project semantic evidence into a compact internal executable conformance plan;
9. compile conformance-local witnesses and trait defaults as detached executable methods, never target-owned methods;
10. execute trait-selected calls without runtime semantic re-resolution;
11. give trait defaults an active conformance context so abstract requirement calls reach the selected concrete witness/data/default target;
12. preserve that context through default→default calls and recursion;
13. make bound callable references freeze the same trait selection and conformance environment as the corresponding call;
14. correctly compose exact conformance evidence with existing runtime generic type environments and exact enum-case semantics;
15. integrate source/LSP navigation and incremental dependency tracking around the canonical semantic products;
16. certify C4 with non-trivial generic, mixed-witness, `Iterable`, `Iterator`, ambiguity, bound-reference, exact-case, GC, and incremental programs.

---

## 2. P3 Acceptance Objective

The final semantic/executable pipeline must be:

```text
source ordinary call/reference
    receiver + selector + side
        ↓
canonical inherent lookup
    ├─ concrete inherent result → ordinary inherent semantics
    └─ miss
         ↓
TraitDispatchIndex
    bounded candidate sources
         ↓
exact P1 target/TraitRef matching
         ↓
P2 exact ConformanceEvidence
         ↓
trait requirement candidate(s)
         ↓
convergence / ambiguity
         ↓
TraitDispatchSelection
    exact TraitRef
    TraitRequirementId
    source ImplId
    evidence fingerprint
    requirement selection
         ↓
ExpressionAnalysis
         ↓
semantic→core lowering projection
         ↓
ExecutableConformancePlan
    deterministic requirement slots
         ↓
selected executable target
    ├─ inherent callable
    ├─ C2 conditional inherent callable
    ├─ conformance-local witness
    ├─ data component projection
    └─ trait default
         ↓
VM execution under RuntimeConformanceEnvironmentId
         ↓
trait default abstract requirement call
         ↓
InvokeTraitRequirement(slot)
         ↓
active executable conformance plan
         ↓
already-selected target
```

The semantic authority remains:

```text
ConformanceEvidence
```

The runtime structure merely realizes that authority.

P3 must never introduce this anti-pipeline:

```text
ordinary receiver
    ↓
runtime class scan
    ↓
find method with matching selector
    ↓
pretend that proves trait conformance
```

---

## 3. Repository Grounding

### 3.1 Planning baseline

This plan is prepared against the pushed C4.P2 completion commit:

```text
repository: aureat/phalcom-lang
revision: 1fc1e8449e6dd77fc533455bcb056fe9b00290b9
commit: lang005: complete C4.P2 witness evidence
prepared: 2026-09-14
```

T0 must record the implementation worktree's actual HEAD, branch, and status. The baseline above is planning authority only if the executor begins from that exact revision or a direct descendant whose C4.P2 semantics have not been replaced.

### 3.2 Landed P2 takeover products

P3 consumes these as stable semantic roles:

```text
CallableOwnerId::Conformance(ImplId)
ConformanceIndex
ConformanceWitnessPlan
ConformanceCompleteness
RequirementSelectionTemplate
InstantiatedTraitRequirement
ConformanceEvidence
ConformanceResolution
SemanticSnapshot::resolve_conformance_evidence
SemanticSnapshot::conformance_evidence_for
resolve_effective_inherent_witness
InherentImplSpecialization
TraitSurface
TraitRequirementId
```

P3 must not rebuild these from AST or source text.

### 3.3 Canonical live integration seams verified at planning time

The post-P2 tree contains the following architectural seams:

- `phalcom-semantic/src/checker/context.rs::resolve_dispatch_target_with_specialization` is the canonical ordinary semantic dispatch path. It already owns C3 abstract trait-`Self` lookup, normal/inherited dispatch, exact-case conditional dispatch, and declaration conditional dispatch.
- `phalcom-semantic/src/checker/analysis.rs::ExpressionAnalysis` currently publishes `callable`, `call_specialization`, and `conditional_dispatch`; P3 adds trait-dispatch evidence beside these rather than encoding it indirectly.
- `phalcom-core/src/modules/semantic_lowering.rs` is the semantic→executable projection boundary and already has dedicated lowering-site products for associated and C2 conditional invocations.
- `phalcom-core/src/compiler/lib/impl_decl.rs::compile_behavior_member` can compile a `BehaviorMember` into a `MethodObject` independently of target installation; P3 should factor/reuse this capability for detached trait/conformance methods.
- `phalcom-core/src/frame.rs::CallFrame` is compact/`Copy` and already carries `RuntimeTypeEnvironmentId`, establishing the precedent for a compact `RuntimeConformanceEnvironmentId` rather than embedding semantic evidence in frames.
- `phalcom-core/src/typing/environment.rs` already interns runtime type environments and should be treated as the design analogue, not duplicated structurally inside trait logic.
- the compiler currently treats `Statement::Trait(_)` as compile-time-only; P3 is the checkpoint that begins compiling trait defaults when semantic lowering demands them.
- existing `BoundMethodObject` is intentionally minimal. P3 should not silently enlarge all ordinary bound methods solely for trait evidence.

### 3.4 Entry compile warning

At the planning baseline, `CallableOwnerId` has three variants, but at least one `phalcom-core` match in semantic lowering still handles only `Declaration` and `Variant`. P2 focused verification did not certify the whole workspace. T0/T1 therefore treat whole-workspace owner exhaustiveness as a mandatory entry gate, not a hypothetical cleanup.

---

## 4. Mandatory Post-P2 Audit Findings

P3 begins by closing these findings. They are part of the plan, not deferred debt.

| ID | Severity | Finding | P3 closure |
|---|---|---|---|
| `C4P2-F01` | CRITICAL | Cross-crate consumers are not fully generalized for `CallableOwnerId::Conformance`; core lowering contains declaration/variant-only assumptions. | T0–T1 |
| `C4P2-F02` | HIGH | Incomplete conformances are represented but do not reliably produce source diagnostics. | T2 |
| `C4P2-F03` | HIGH | Exact evidence resolution collapses `Unknown`/`Blocked`/`Dynamic`/cancel/budget/internal states to `Incomplete`. | T2 |
| `C4P2-F04` | HIGH | Aggregate terminal uncertainty can mask a separately proven requirement failure. | T2 |
| `C4P2-F05` | HIGH | Generic source planning may freeze a trait default even when an exact application later proves a higher-precedence conditional inherent witness. | T4 |
| `C4P2-F06` | HIGH, verify-first | An explicit conformance witness must not hide an incompatible existing inherent selector. | T3 |
| `C4P2-F07` | HIGH, verify-first | Generic ordinary inherent witnesses must be specialized through the target/conformance environment before compatibility. | T3 |
| `C4P2-F08` | MEDIUM-HIGH | Witness visibility must use actual access coverage, not approximate token ordering. | T3 |
| `C4P2-F09` | MEDIUM | Data-component compatibility must not pretend the witness has a callable identity. | T3 |
| `C4P2-F10` | MEDIUM-HIGH | `CallableOwnerId::Deref<DeclarationId>` and panicking total declaration-owner APIs are unsafe once conformance callables flow into compiler/runtime paths. | T1 |
| `C4P2-F11` | MEDIUM | The P2 generic-constraint mini-reasoner must not grow into a parallel C6 solver. | Preserve boundary; T3 only factors existing relation use if necessary |
| `C4P2-F12` | P3 architecture | Exact evidence requires a known `TraitRef`; ordinary lookup begins with receiver+selector and needs bounded candidate discovery. | T5–T7 |

G1 does not pass until F01–F10 are either proven already-correct by focused regressions or repaired with tests.

---

## 5. P2 Takeover Contract

P3 must preserve these P2 invariants exactly:

1. conformance-local callables use `CallableOwnerId::Conformance(ImplId)` and have no declaration owner;
2. conformance-local callables never enter target-owned inherent surfaces or target-owned dispatch mutation APIs;
3. one source generic conformance owns one symbolic source plan; exact applications bind its environment rather than minting source witness identities;
4. trait defaults remain trait-owned and were checked once by C3 under abstract `Self`;
5. `DataComponentId` remains a data-component witness identity; no synthetic getter is created;
6. conditional inherent witness selections retain canonical C2 applicability evidence and P3 never re-runs a parallel C2 domain solver;
7. only successful exact conformance evidence authorizes trait behavior;
8. exact target generic arguments and exact `TraitRef` generic arguments remain part of the applied semantic relation;
9. parent class conformance does not implicitly propagate to child classes;
10. bodyless/duplicate/unmatched explicit conformance members cannot fall through to another witness/default.

Any P3 patch that violates one of these is architecturally invalid even if its runtime demo passes.

---

## 6. Fixed P3 Semantic Decisions

These decisions are fixed for implementation unless the user explicitly changes the language design.

### 6.1 Ordinary availability is layered, not merged

```text
ordinary receiver behavior
    =
inherent/effective behavior
    +
proven trait-evidenced behavior
```

The expression-level lookup result may combine these layers, but their source products remain separate.

### 6.2 Inherent behavior keeps ordinary selector authority

P3 integrates trait-evidenced discovery only after canonical ordinary inherent/C2 lookup misses. This is deliberate:

- a compatible inherent member should already have been selected by P2 as the concrete witness for every relevant conformance;
- an incompatible same-selector inherent member makes a conformance invalid under T3 and therefore cannot coexist as a valid trait-only alternative;
- adding a concrete inherent member resolves a competing-default ordinary-selector ambiguity because ordinary lookup succeeds before trait-only fallback.

P3 must not make a trait default shadow an inherent member.

### 6.3 Trait-only public call type is the requirement contract

When ordinary lookup succeeds only through trait evidence, the expression's callable contract is the exact instantiated `TraitRequirementId` signature, not a more-specific witness signature. The witness proves that it covers the requirement; user-level dispatch through the requirement observes the requirement contract.

### 6.4 Convergence is by executable semantic identity

Multiple trait requirements may contribute the same selector. They converge only when their proven selections denote the same concrete executable capability under equivalent proof context.

Canonical convergence cases:

```text
same inherent CallableId
same DataComponentId
same conditional inherent CallableId + same retained C2 specialization/evidence
```

Non-convergence cases:

```text
distinct conformance-local CallableIds
different trait-owned defaults
same default callable under semantically different conformance environments unless exact evidence equivalence is proven
```

No source order, import order, trait order, or hash iteration order decides ambiguity.

### 6.5 Exact evidence, not runtime scanning, authorizes execution

Every executable trait call/ref originates from an exact semantic `TraitDispatchSelection`. Runtime representations may cache or compact that selection but may not independently discover conformances.

### 6.6 Runtime implementation uses an internal executable conformance plan

P3 adopts an internal plan/environment design instead of public trait-object/vtable semantics.

This is implementation machinery only. It does not make traits runtime class values, existential values, or public reflection objects.

---

## 7. Chosen Runtime Execution Architecture

P3 should implement the following concrete runtime model unless T0 proves an existing equivalent mechanism already landed.

### 7.1 Executable conformance plan

Project one exact semantic evidence result into an immutable compiler/runtime description:

```rust
pub struct ExecutableConformancePlan {
    pub source_impl: ImplId,
    pub requirements: Box<[ExecutableRequirementTarget]>,
}
```

Requirement slots are deterministic and correspond to canonical `TraitRequirementId` ordering in the instantiated trait surface. Do not assign slots by source hash-map iteration.

Conceptual target variants:

```rust
pub enum ExecutableRequirementTarget {
    Inherent {
        callable: CallableId,
        conditional: Option<ExecutableConditionalSelection>,
    },
    ConformanceWitness {
        callable: CallableId,
        method: RuntimeMethodHandle,
    },
    DataComponent {
        component: DataComponentId,
        logical_index: u32,
    },
    TraitDefault {
        callable: CallableId,
        method: RuntimeMethodHandle,
    },
}
```

The exact Rust handle type is mechanically flexible; use existing core runtime method/`ObjRef` conventions. If the plan contains GC-managed method objects, the registry must root them.

### 7.2 Runtime plan identity

Introduce a compact interned ID, conceptually:

```rust
pub struct RuntimeConformancePlanId(pub u32);
```

The registry owns immutable executable plans. Equivalent executable plans should intern deterministically where existing runtime registry conventions support it; correctness must not rely on pointer identity.

### 7.3 Runtime conformance environment

Generic runtime type substitutions already flow through `RuntimeTypeEnvironmentId`. Trait execution needs a compact pair:

```rust
pub struct RuntimeConformanceEnvironment {
    pub plan: RuntimeConformancePlanId,
    pub type_environment: RuntimeTypeEnvironmentId,
}

pub struct RuntimeConformanceEnvironmentId(pub u32);
```

Use an interned registry analogous to the type environment registry. `EMPTY` means “no active trait conformance context.”

### 7.4 Call frame propagation

Extend `CallFrame` with one compact field:

```rust
pub conformance_environment: RuntimeConformanceEnvironmentId,
```

Ordinary method/closure calls use `EMPTY`. Trait-selected witness/default invocation stamps the callee frame with the exact environment. Default→default and default→requirement invocations retain it.

Do not embed semantic `TraitRef`, `ConformanceEvidence`, maps, or Rust heap allocations directly in every frame.

### 7.5 Detached executable methods

Conformance-local witnesses and trait defaults are compiled to ordinary executable `MethodObject`s/closures but are not installed in the target's class dictionary.

Use the existing `compile_behavior_member` machinery after factoring compilation from inherent installation as needed.

### 7.6 Bytecode roles

P3 should add explicit bytecode/lowering roles rather than disguising trait calls as generic sends:

```text
InvokeTraitSelected
    call a semantically selected requirement target from ordinary source syntax

InvokeTraitRequirement
    call a TraitRequirementId slot from within a trait default using the active conformance environment

MakeTraitBoundMethod
    capture receiver + exact selected method/requirement + conformance environment
```

Exact encoding/operand packing is mechanically flexible and must follow existing chunk/executable-semantics table conventions. The bytecode names above may be adapted only if equivalent roles remain explicit.

### 7.7 Why this design is chosen

This architecture:

- compiles each trait default once rather than cloning its bytecode for every conformance;
- naturally preserves one conformance context through default→default and recursive calls;
- allows abstract requirement sites inside C3-checked defaults to become explicit `InvokeTraitRequirement` operations;
- reuses compact interned runtime-environment architecture already present for generics;
- avoids runtime trait search;
- avoids target method-table mutation;
- leaves a clean extension seam for C5 associated bindings and C6 nested evidence without implementing either now.

---

## 8. Semantic Trait-Dispatch Discovery Architecture

Create a focused semantic owner, preferably:

```text
phalcom-semantic/src/trait_dispatch.rs
```

Do not continue growing `impls.rs` with ordinary dispatch/lowering concerns unless the live repository strongly favors a smaller extraction.

### 8.1 Source contribution

Each eligible conformance+requirement publishes a discovery contribution, conceptually:

```rust
pub struct TraitDispatchContribution {
    pub impl_id: ImplId,
    pub target_family: TraitDispatchTargetFamily,
    pub trait_declaration: DeclarationId,
    pub requirement: TraitRequirementId,
    pub selector: Selector,
    pub side: DispatchSide,
}
```

This is a discovery/index product, not satisfaction evidence. The exact evidence is still queried from P2 when resolving an exact receiver.

### 8.2 Target-family key

Use a bounded canonical family identity:

```rust
pub enum TraitDispatchTargetFamily {
    Declaration(DeclarationId),
    ExactEnumCase(VariantId),
}
```

Applied generic target forms bucket under their nominal declaration family; exact matching later preserves arguments. Exact enum cases remain exact and must not bucket as their enum root for conformance propagation.

### 8.3 Index key

The hot ordinary lookup key is:

```text
(target family, selector, dispatch side)
```

A coarse trait declaration may be retained in secondary metadata but must not be required as an input because ordinary syntax does not know it.

### 8.4 Exact candidate query

Expose a canonical query conceptually like:

```rust
pub fn resolve_trait_evidenced_candidates(
    receiver: TypeId,
    selector: &Selector,
    side: DispatchSide,
    ...canonical snapshot products...
) -> TraitDispatchResolution
```

For each bounded contribution:

1. exact-match the source conformance target template to the receiver using the same P1 target-specialization machinery;
2. derive the exact `TraitRef` through that same impl substitution;
3. call the P2 exact evidence resolver for `(receiver, exact TraitRef)`;
4. reject non-proven evidence as a usable candidate while preserving its proof-state result for diagnostics/analysis status;
5. locate the exact instantiated `TraitRequirementId`/signature in the evidence;
6. publish a `TraitEvidencedMemberCandidate` retaining the exact evidence identity and requirement selection.

P3 may factor an exact target-only matcher out of P1 internals to avoid duplicating generic unification, but must not rebuild conformance matching independently.

### 8.5 No implicit parent-conformance propagation

Candidate discovery for `Child` does not enumerate `Base` conformances merely because normal method lookup walks class inheritance. A `Child` conformance may select an inherited callable as its witness, but `impl Trait for Base` alone does not make `Child` a candidate.

---

## 9. Trait Dispatch Result and Convergence Model

Use structured resolution results, not `Option<CallableId>`.

Conceptually:

```rust
pub struct TraitEvidencedMemberCandidate {
    pub exact_trait_ref: TraitRef,
    pub requirement: TraitRequirementId,
    pub source_impl: ImplId,
    pub evidence_fingerprint: ProductFingerprint,
    pub selection: RequirementSelectionTemplate,
    pub signature: CallableSignature,
    pub convergence: TraitDispatchConvergenceKey,
}
```

Resolution should preserve:

```rust
pub enum TraitDispatchResolution {
    Found(TraitDispatchSelection),
    Ambiguous(Box<[TraitEvidencedMemberCandidate]>),
    Missing,
    Unknown(...),
    Blocked(...),
    Dynamic(...),
    Cancelled,
    BudgetExceeded(...),
    InternalFailure(...),
}
```

### 9.1 Convergence key

Conceptually:

```rust
pub enum TraitDispatchConvergenceKey {
    Inherent(CallableId),
    ConditionalInherent {
        callable: CallableId,
        impl_id: ImplId,
        applicability_fingerprint: ProductFingerprint,
    },
    DataComponent(DataComponentId),
    ConformanceCallable(CallableId),
    TraitDefault {
        callable: CallableId,
        evidence_fingerprint: ProductFingerprint,
    },
}
```

Multiple candidates converge only when their keys are semantically equal under this model.

### 9.2 Ambiguity diagnostic payload

Keep enough information to report:

```text
receiver type
selector
candidate trait refs
requirement source ranges
selected witness/default origins
```

Do not collapse this to generic class-method ambiguity.

---

## 10. Canonical Ordinary Dispatch Integration

Integrate at `CheckingContext::resolve_dispatch_target_with_specialization` or its live successor.

Required order:

```text
1. C3 abstract trait-Self lookup
2. ordinary/inherited inherent lookup
3. exact-case C2 conditional inherent lookup
4. declaration C2 conditional inherent lookup
5. if Found/Ambiguous/Dynamic from inherent layer, preserve current semantics
6. if Missing, query trait-evidenced candidates
7. map trait result to canonical ResolvedDispatchResult-compatible form or a generalized dispatch result
```

Do not individually patch every expression visitor.

Method calls, getters, setters, indexes, operators, iteration protocol calls, and bound callable-reference family discovery must all consume the same canonical resolver where their existing architecture already routes through it.

Class-side dispatch is out of scope for C4 conformance. Trait dispatch fallback is instance-side only unless a later ratified metatype rule says otherwise.

---

## 11. Per-Expression Semantic Publication

P3 needs an explicit semantic product that survives into lowering.

Add conceptually:

```rust
pub enum TraitDispatchSite {
    AbstractRequirement {
        trait_declaration: DeclarationId,
        requirement: TraitRequirementId,
    },
    Evidenced(TraitDispatchSelection),
}

pub struct TraitDispatchSelection {
    pub exact_trait_ref: TraitRef,
    pub requirement: TraitRequirementId,
    pub source_impl: ImplId,
    pub evidence_fingerprint: ProductFingerprint,
    pub selection: RequirementSelectionTemplate,
}
```

Then extend `ExpressionAnalysis`:

```rust
pub trait_dispatch: Option<TraitDispatchSite>,
```

### 11.1 Abstract C3 default sites

`resolve_trait_contract_target` currently recognizes abstract `Self` and trait members during C3 default analysis. P3 must publish `TraitDispatchSite::AbstractRequirement` for requirement call sites so compiler lowering never reconstructs abstract requirement identity from AST text/ranges.

Calls from a trait default to a bodyful trait default should also retain the trait requirement identity associated with that member; runtime uses the same conformance slot whether the selected target is another default or a concrete witness.

### 11.2 Evidenced ordinary sites

An ordinary `value.method` that resolved through a conformance publishes `Evidenced(selection)`.

Synchronize this field through all expression re-analysis/publication paths and include it in body/product fingerprints.

---

## 12. Incremental Dependency Architecture

Trait dispatch consumers must track exactly what made their call valid.

Add or reuse typed dependency edges for at least:

```text
TraitSurface(trait declaration)
conformance source/head identity
ConformanceWitnessPlan(ImplId) or equivalent plan fingerprint dependency
selected callable signature / data declaration
hierarchy edges if inherited witness selection participates
C2 conditional applicability products if the selected witness is conditional
```

Prefer explicit dependency variants such as:

```rust
SemanticDependency::ConformanceHead(ImplId)
SemanticDependency::ConformanceWitnessPlan(ImplId)
```

if the current DB does not already expose an equivalent stable query key.

Do not solve invalidation by recording the entire source module or entire workspace as a dependency.

Body-only witness edits should not change head/coherence/evidence selection identity when signature and selection remain stable, but code generation must still refresh the executable detached method body.

---

## 13. Corrective Exact-Dependent Conditional Witness Selection

P2 source planning must not permanently select a lower-precedence trait default when a conditional inherent candidate is unresolved at the generic source domain but becomes proven for an exact application.

Required source-plan extension conceptually:

```rust
pub enum RequirementSelectionTemplate {
    ConformanceCallable { ... },
    InherentCallable { ... },
    DataComponent { ... },
    TraitDefault { ... },
    ConditionalInherentOrFallback {
        candidate: EffectiveInherentWitness,
        fallback: Box<RequirementSelectionTemplate>,
    },
}
```

Use repository-appropriate factoring rather than recursive boxing if a flatter structure is clearer.

Rules:

```text
source planning discovers the conditional candidate and fallback once
exact evidence specializes the retained candidate through the canonical exact environment
exact evidence consumes retained C2 applicability machinery
Proven       → conditional concrete witness
Disproven    → retained fallback
Unknown/etc. → corresponding terminal conformance state unless an independently higher-priority proven source exists
```

Exact evidence must not rescan the target surface or rediscover witness candidates.

This is the only P2 selection-model expansion authorized in P3 because ordinary execution would otherwise violate the fixed “concrete compatible witness > default” rule.

---

## 14. Diagnostics Requirements

P3 must preserve or add distinct diagnostics for:

```text
incomplete conformance
missing witness
incompatible witness
witness access/visibility too narrow
mutable property mismatch
explicit conformance witness conflicting with incompatible inherent selector
ambiguous trait-evidenced selector
competing trait defaults
terminal/blocked conformance proof when the call requires proof
invalid trait runtime lowering state as internal compiler incident, not user method-missing
```

Do not report a known trait/conformance ambiguity as generic `method not found` or ordinary unrelated overload ambiguity.

An invalid source conformance is diagnosed even if no call site currently uses the trait.

---

## 15. Global P3 Invariants

### `P3-INV-01` — Semantic authority

Only proven P2 `ConformanceEvidence` authorizes trait-evidenced behavior.

### `P3-INV-02` — No inherent-surface injection

Trait defaults and conformance-local witnesses never enter the target's `DeclarationSurface` or conditional inherent-member sets.

### `P3-INV-03` — No runtime class-table injection

Detached trait/default/conformance methods are never installed as target-owned runtime class methods merely to make sends work.

### `P3-INV-04` — No runtime proof search

The VM never scans classes, trait declarations, or source conformances to decide whether a receiver conforms.

### `P3-INV-05` — Exact target identity

`Value<Int>` and `Value<String>` remain distinct through semantic selection and executable plans.

### `P3-INV-06` — Exact TraitRef identity

Distinct exact trait applications remain distinct through dispatch evidence.

### `P3-INV-07` — Requirement identity survives

`TraitRequirementId` is retained even when several requirements converge on one concrete callable.

### `P3-INV-08` — Concrete inherent authority

Existing valid inherent behavior is resolved before trait-only fallback.

### `P3-INV-09` — Shared witness convergence

Distinct trait requirements selecting the same concrete capability are unambiguous.

### `P3-INV-10` — Competing defaults are ambiguous

Different trait defaults do not acquire implicit precedence.

### `P3-INV-11` — Conformance-local identity stays distinct

Different conformance-owned callables do not collapse merely because selectors/signatures coincide.

### `P3-INV-12` — Data remains data

Data-component witnesses lower as component projections, not synthetic getter methods.

### `P3-INV-13` — C2 evidence reuse

Conditional inherent witness execution consumes the retained C2 selection/proof; P3 does not re-solve the C2 domain.

### `P3-INV-14` — Trait defaults stay trait-owned

Default method identity remains the C3 trait callable.

### `P3-INV-15` — Active evidence for defaults

Every executing trait default entered through conformance evidence has a non-empty runtime conformance environment.

### `P3-INV-16` — Abstract requirement calls are explicit

C3 abstract requirement call sites compile to requirement-slot invocation, never ordinary runtime sends.

### `P3-INV-17` — Default→default preserves context

Calling another selected trait default preserves the same active conformance environment.

### `P3-INV-18` — Bound reference freezes selection

A trait-evidenced bound reference retains the exact semantic selection/environment from creation and does not re-resolve later.

### `P3-INV-19` — Parent conformance does not propagate

Class inheritance never creates conformance evidence on its own.

### `P3-INV-20` — Exact enum-case conformance stays exact

Case conformance does not leak to root/sibling variants.

### `P3-INV-21` — Proof states remain distinct

Unknown/blocked/dynamic/cancelled/budget/internal outcomes never collapse into false, true, or generic incomplete state.

### `P3-INV-22` — Proven failure dominates uncertainty

A conformance with a known unsatisfied requirement is invalid even if a separate requirement is unresolved.

### `P3-INV-23` — Access coverage is semantic

Witness visibility is validated through canonical access coverage, not enum-token ordering.

### `P3-INV-24` — Generic target member specialization is canonical

Declaration-owned generic witness signatures are specialized through the target/conformance environment before compatibility and execution.

### `P3-INV-25` — Per-expression evidence is lowering authority

Compiler lowering consumes `TraitDispatchSite` and does not query semantic indexes again.

### `P3-INV-26` — Deterministic runtime slots

Requirement-slot numbering is canonical and independent of hash/source traversal order.

### `P3-INV-27` — Runtime environment composes with generic type environment

Trait execution never discards the existing `RuntimeTypeEnvironmentId`.

### `P3-INV-28` — GC safety

Every runtime registry/object retaining detached method handles or trait-bound references is traced/rooted correctly.

### `P3-INV-29` — Incremental/cold equivalence

For identical source, cold and incremental analysis publish equivalent trait-dispatch selection/evidence and lowering products.

### `P3-INV-30` — C5/C6 boundary

P3 introduces no associated binding/projection or trait-conditioned conformance proof semantics.

---

## 16. Non-Goals

Do not implement any of the following in P3:

```text
associated type declarations/bindings/projection normalization   → C5
generic trait bounds / `T: Trait` proof                          → C6
conditional trait conformance                                    → C6
nested conformance evidence                                      → C6
supertraits / trait inheritance                                  → unratified
trait objects / existential values                               → later
public vtable objects                                             → later
public conformance reflection descriptors                        → later/C7+
trait-qualified call syntax                                      → reserved
metatype/class-side conformance                                  → reserved
conformance specialization / most-specific rule                  → not defined
implicit or structural conformance                               → forbidden
runtime registration as semantic membership                      → forbidden
copying defaults into target runtime class dictionaries          → forbidden
```

An internal `ExecutableConformancePlan` is not a public trait object/vtable. Keep it compiler/VM-internal.

---

## 17. Expected Impact Map

Verify live paths during T0. The following are the expected production/test touch points.

### 17.1 Semantic identity/evidence

```text
phalcom-semantic/src/identity.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/diagnostic.rs
```

### 17.2 New/expanded trait dispatch

Prefer:

```text
phalcom-semantic/src/trait_dispatch.rs        # create
phalcom-semantic/src/lib.rs                   # export internal/public roles as appropriate
```

### 17.3 Checker/publication/dependencies

```text
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/statement.rs     # only where protocol calls already route through canonical dispatch
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/semantic_shard.rs
```

### 17.4 Editor/source projection

```text
phalcom-semantic/src/editor.rs
phalcom-semantic/src/source_index/*
phalcom-semantic/src/presentation.rs          # only if exact trait target presentation is needed
phalcom-lsp/*                                 # only adapters that consume the semantic query product
```

### 17.5 Semantic→core lowering

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/mod.rs
```

### 17.6 Compiler

```text
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/impl_decl.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/associated.rs
phalcom-core/src/compiler/lib/error.rs
```

A focused trait-specific compiler module may be created if it keeps execution code smaller, e.g.:

```text
phalcom-core/src/compiler/lib/trait_dispatch.rs
```

### 17.7 Bytecode/chunk/disassembly

```text
phalcom-core/src/bytecode.rs
phalcom-core/src/chunk.rs
phalcom-core/bin/phalcom/disasm.rs
```

### 17.8 Runtime conformance environment

Prefer a focused module:

```text
phalcom-core/src/typing/conformance_environment.rs      # create, or equivalent runtime module
phalcom-core/src/typing/mod.rs
phalcom-core/src/frame.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs                              # only shared invocation helper if needed
phalcom-core/src/heap/object.rs                          # trait-bound callable object, if chosen
phalcom-core/src/heap/* GC tracing paths
```

### 17.9 Tests

Expected additions/expansions:

```text
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/incremental/*
phalcom-semantic/tests/semantic/integration/editor.rs or relevant editor suites
phalcom-core/tests/core/language/traits.rs               # create focused C4 runtime lane
phalcom-core/tests/core/language/compiler/*
phalcom-core/tests/core/language/inherent_impl/* regressions where owner generalization touches them
```

### 17.10 Checkpoint records

```text
docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-walkthrough.md
docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-handoff.md
```

---

## 18. Implementer Decision Authority

### 18.1 FIXED

The implementer may not change:

- semantic authority remains P2 exact evidence;
- conformance/default behavior remains outside target inherent surfaces and runtime class dictionaries;
- ordinary inherent behavior is tried before trait-only fallback;
- competing defaults have no implicit precedence;
- concrete shared witnesses converge;
- parent conformance does not propagate;
- exact enum-case conformance remains exact;
- default abstract calls use the active exact conformance plan;
- runtime does not search for conformance;
- bound references retain exact trait selection;
- C5/C6 semantics remain deferred;
- runtime trait execution uses detached methods + compact conformance environment rather than public trait-object semantics.

### 18.2 MECHANICALLY FLEXIBLE

The implementer may choose:

- exact module/file split of `trait_dispatch.rs` and runtime conformance environment;
- map/index container types, provided deterministic visible ordering is preserved;
- exact `TraitDispatchResolution`/`Selection` naming;
- exact runtime ID names and operand widths;
- whether executable plans are interned globally per VM or per compiled module/chunk, provided calls/ref captures can share stable IDs safely;
- whether trait-bound callable is a new heap object or a generalized existing internal callable wrapper, provided ordinary `BoundMethodObject` ABI/size is not casually enlarged;
- exact bytecode names/packing if all three semantic roles in §7.6 remain explicit;
- whether detached default/witness compilation is cached in compiler state or chunk executable-semantics registries;
- exact source-index target representation for requirement→witness/default navigation.

### 18.3 VERIFY-FIRST

Before committing to a mechanical design, verify:

- whether whole-workspace compilation currently fails from `CallableOwnerId::Conformance` exhaustiveness;
- every remaining call to `declaration_owner()`, `owner.declaration()`, and `Deref<DeclarationId>` reachable by conformance callables;
- the exact source retrieval API for trait defaults and conformance witnesses;
- whether existing `ExecutableInvocationTarget` can safely represent detached method handles or needs a trait-specific target enum;
- current GC tracing/rooting architecture for compiler/runtime registries holding `ObjRef`;
- whether `RuntimeTypeEnvironmentRegistry` has a reusable generic interning helper;
- whether callable-reference lowering can carry a trait-specific exact target without perturbing ordinary live-dispatch families;
- the current editor definition/navigation target API for `CallableId` and `TraitRequirementId`;
- exact iteration/protocol semantic call paths so the `Iterable` fixture goes through canonical dispatch rather than a hidden special resolver.

---

## 19. STOP / CONSULT Triggers

STOP AND CONSULT if implementation proves any of these:

1. P2 exact evidence cannot be made proof-state-correct without changing a fixed P1/P2 identity rule.
2. Correct conditional-witness/default precedence would require independently re-running P2 witness discovery for every exact application rather than retaining a source decision template.
3. Ordinary trait-evidenced lookup can only be implemented by inserting members into `DeclarationSurface`.
4. Conformance-local witnesses can only execute by installing them into the target runtime class method table.
5. Trait defaults can only execute by runtime trait/class scanning.
6. Abstract requirement calls cannot retain stable `TraitRequirementId` identity from C3 default analysis.
7. Supporting requirement dispatch appears to require public trait objects/existentials.
8. Correct generic runtime execution requires implementing associated types or C6 trait constraints.
9. Parent-conformance propagation appears necessary for required tests; this would change a fixed language rule.
10. Exact enum-case conformance cannot remain case-specific in runtime lowering without changing ratified enum identity.
11. Multiple trait requirements with the same selector require a new precedence rule rather than the fixed convergence/ambiguity model.
12. The only way to preserve bound trait selection is to re-resolve it dynamically at invocation time.
13. Runtime conformance plans cannot be rooted/GC-safe under existing heap architecture without a broader memory-model redesign.
14. `CallFrame` cannot accept a compact conformance-environment ID without violating a fixed VM representation invariant.
15. An implementation choice would expose internal executable conformance tables as public trait reflection semantics.
16. A required C4 test can only pass by implementing C5/C6 semantics.
17. The live branch contains a materially different post-P2 trait execution architecture not represented by this plan and overwriting it would be destructive.

Mechanical compile errors, moved symbols, and exhaustive-match updates are not consultation triggers unless they expose one of the architectural contradictions above.

---

## 20. Failure Classification

Classify every unexpected failure before patching:

```text
P2_ENTRY_DEFECT
    one of §4 audit findings reproduced at P3 entry

NEW_P3_BUG
    new trait dispatch/lowering/runtime behavior is wrong

REGRESSION
    P3 broke previously valid inherent/C1–C3 behavior

PREEXISTING
    failure reproduces at the P3 entry revision outside C4 changes

BASELINE_INFRA
    unrelated toolchain/environment failure

SCOPE_CONFLICT
    correct fix would cross fixed P3/C5/C6 boundary

SPEC_CONFLICT
    normative specifications disagree about required semantics
```

Never patch a `PREEXISTING`/`BASELINE_INFRA` failure into C4 unless it blocks a required C4 path and the user explicitly authorizes the repair.

---

## 21. Coverage Matrix

Use these IDs in test names/comments or the P3 walkthrough evidence table.

### 21.1 Entry correctness — EC

- **EC-01** whole `phalcom-core`/workspace compiles with `CallableOwnerId::Conformance`.
- **EC-02** conformance callables never require a fake `DeclarationId`.
- **EC-03** no reachable generic owner deref panics on conformance callables.
- **EC-04** missing requirement diagnoses incomplete conformance before use.
- **EC-05** incompatible witness diagnoses conformance failure.
- **EC-06** exact resolver preserves `Unknown`.
- **EC-07** exact resolver preserves `Blocked`/`Dynamic`/cancel/budget/internal states.
- **EC-08** known failed requirement dominates unrelated unknown requirement.
- **EC-09** generic declaration-owned inherent witness specializes through target env.
- **EC-10** incompatible inherent selector + explicit conformance witness is rejected.
- **EC-11** public requirement cannot use narrower private witness.
- **EC-12** access coverage uses real owner/module semantics.
- **EC-13** data witness proof retains `DataComponentId`, not fake callable identity.
- **EC-14** exact conditional concrete witness wins over default when proven.
- **EC-15** disproven conditional concrete witness falls back to default.
- **EC-16** unresolved conditional concrete witness preserves terminal state rather than freezing default incorrectly.
- **EC-17** bodyless/duplicate/unmatched explicit member remains invalid with default available.
- **EC-18** P2 plan/evidence fingerprints change only for semantic selection/proof changes.

### 21.2 Discovery/index — DI

- **DI-01** ordinary lookup can discover trait behavior without a caller-supplied `TraitRef`.
- **DI-02** index key is bounded by target family + selector + side.
- **DI-03** `Value<Int>` and `Value<String>` discover different exact conformances.
- **DI-04** distinct exact `TraitRef`s on one target remain distinct candidates.
- **DI-05** exact enum case discovers only case conformance.
- **DI-06** root/sibling do not discover case conformance.
- **DI-07** child does not discover parent conformance automatically.
- **DI-08** generic source conformance derives exact TraitRef under one impl substitution.
- **DI-09** non-proven exact conformance cannot produce usable candidate.
- **DI-10** terminal proof states survive discovery.
- **DI-11** add/delete conformance updates buckets owner-completely.
- **DI-12** publication/discovery order is deterministic.

### 21.3 Ordinary dispatch — OD

- **OD-01** inherent member wins before trait-only fallback.
- **OD-02** one trait default becomes available through ordinary syntax when no inherent member exists.
- **OD-03** one conformance-local witness becomes available through ordinary syntax.
- **OD-04** data-component witness participates where getter semantics permit.
- **OD-05** shared inherent witness across two traits converges.
- **OD-06** shared data component across traits converges.
- **OD-07** same C2 conditional witness + same proof converges.
- **OD-08** competing defaults are ambiguous.
- **OD-09** distinct conformance-local witnesses are ambiguous.
- **OD-10** same default identity under different evidence does not silently collapse.
- **OD-11** class-side lookup does not acquire C4 instance conformance.
- **OD-12** ambiguity is independent of source/module order.

### 21.4 Per-expression publication — XP

- **XP-01** ordinary trait call publishes exact `TraitDispatchSelection`.
- **XP-02** trait default abstract requirement call publishes `AbstractRequirement` identity.
- **XP-03** exact TraitRef retained at call site.
- **XP-04** requirement ID retained independently of witness callable ID.
- **XP-05** evidence fingerprint retained.
- **XP-06** conformance `ImplId` retained.
- **XP-07** call re-analysis clears stale trait selection.
- **XP-08** body fingerprint changes when trait selection changes.
- **XP-09** body fingerprint does not change merely from source range movement.
- **XP-10** editor/source projection uses published product, not AST re-resolution.

### 21.5 Lowering — LW

- **LW-01** trait-only ordinary call projects an executable requirement target.
- **LW-02** conformance witness projects detached method target.
- **LW-03** trait default projects detached method target.
- **LW-04** data witness projects data component target.
- **LW-05** conditional inherent witness retains C2 executable selection.
- **LW-06** abstract requirement site projects requirement slot.
- **LW-07** semantic lowering never queries `ConformanceIndex`/witness discovery to reselect.
- **LW-08** deterministic requirement slots independent of source traversal.
- **LW-09** invalid/ambiguous semantic sites do not receive executable attachments.
- **LW-10** target class lowering contains no copied default/witness members.

### 21.6 Runtime — RT

- **RT-01** simple conformance-local witness call executes.
- **RT-02** simple trait default call executes.
- **RT-03** trait default calls conformance-local abstract witness.
- **RT-04** trait default calls inherent abstract witness.
- **RT-05** trait default calls conditional inherent witness.
- **RT-06** trait default reads data-component witness.
- **RT-07** trait default calls another trait default.
- **RT-08** nested default calls retain same conformance environment.
- **RT-09** recursive default path retains environment and does not clone evidence per frame.
- **RT-10** generic runtime type environment survives trait call.
- **RT-11** exact specialized conformance receives correct runtime type environment.
- **RT-12** exact enum-case trait call executes only on matching case path.
- **RT-13** normal non-trait call frame uses empty conformance env.
- **RT-14** trait-selected callee gets non-empty exact env.
- **RT-15** missing active env at `InvokeTraitRequirement` fails as compiler/runtime internal error, not user-level dNU.
- **RT-16** forced GC during/after trait calls retains detached methods/plans.
- **RT-17** no trait/conformance method is present in target class dictionary solely because of conformance.
- **RT-18** runtime execution never depends on import/module initialization order.

### 21.7 Bound references — BR

- **BR-01** conformance-local witness bound reference executes.
- **BR-02** trait-default bound reference executes.
- **BR-03** bound trait default can call abstract requirement after capture.
- **BR-04** captured exact generic specialization remains exact.
- **BR-05** reference retains original conformance selection; no live trait re-resolution.
- **BR-06** ordinary non-trait bound method/family behavior remains unchanged.
- **BR-07** trait-bound callable survives GC.

### 21.8 Incremental/editor — IE

- **IE-01** add conformance invalidates dependent missing call into valid trait call.
- **IE-02** delete conformance invalidates valid call into missing/diagnostic.
- **IE-03** add/remove default changes trait availability.
- **IE-04** witness signature edit revalidates evidence and call.
- **IE-05** witness body-only edit preserves selection identity but refreshes executable body.
- **IE-06** requirement signature edit revalidates dependent conformances/calls.
- **IE-07** inherited override edit changes selected witness.
- **IE-08** C2 conditional-domain edit changes selected trait witness where applicable.
- **IE-09** competing default add/remove toggles ambiguity deterministically.
- **IE-10** cold and incremental `TraitDispatchSelection` agree.
- **IE-11** definition/navigation lands on inherent witness, conformance witness, or trait default according to selection.
- **IE-12** completion/hover expose trait-evidenced behavior from semantic index.
- **IE-13** ambiguous trait selector surfaces trait-specific diagnostic/source candidates.
- **IE-14** delete module removes stale dispatch contributions/evidence/tooling links.
- **IE-15** runtime recompile path does not retain stale executable plan after semantic selection changes.

### 21.9 Vertical acceptance — VA

- **VA-01** `Value<String>`/`Value<Int>` return distinct specialized `Tagged.tag` results.
- **VA-02** generic source conformance executes for multiple exact targets without source identity duplication.
- **VA-03** `Scalable<Int>.twice` returns 12 through default→requirement.
- **VA-04** C4 generic `Iterable<Item, Cursor>` executes mixed witness origins.
- **VA-05** `Iterable.toList` exercises default→default→requirements.
- **VA-06** stateful `Iterator<Item>` preserves independent instance state.
- **VA-07** shared concrete witness two-trait program is unambiguous.
- **VA-08** competing defaults compile-fail with trait ambiguity.
- **VA-09** adding concrete inherent resolver removes ambiguity.
- **VA-10** exact enum-case conformance executes without root leakage.
- **VA-11** bound trait default/reference executes through preserved environment.
- **VA-12** complex valid C4 stress program executes deterministic results.
- **VA-13** complex invalid C4 corpus emits expected diagnostic families.
- **VA-14** forced-GC stress leaves trait execution/reference results correct.

---

## 22. Task and Gate Sequence

Execute strictly in this order:

```text
T0  post-P2 entry lock and whole-workspace audit
T1  callable-owner cross-crate closure
T2  P2 completeness/proof-state/diagnostic closure
T3  P2 witness-compatibility correctness closure
T4  exact-dependent conditional-witness/default decision template
G1  P2 evidence is safe for execution consumers

T5  trait-dispatch source contribution/index
T6  exact receiver→TraitRef/evidence candidate query
T7  convergence and ambiguity
G2  receiver+selector trait evidence authority

T8  checker context wiring and canonical ordinary dispatch integration
T9  per-expression abstract/evidenced trait dispatch publication
T10 semantic dependencies, fingerprints, incremental lifecycle
G3  semantic trait dispatch complete

T11 executable conformance lowering model
T12 compiler detached method compilation and source retrieval
T13 runtime conformance plan/environment registry and frame field
T14 trait-selected invocation bytecode/runtime
G4  simple trait witness/default calls execute

T15 abstract requirement bytecode and default→requirement execution
T16 data/C2 conditional/default→default target execution
G5  full requirement map executes

T17 trait-evidenced bound callable references
T18 generic/exact-case/runtime-environment integration
G6  all witness origins and references execute

T19 editor/source projection and incremental certification
T20 vertical Iterable/Iterator/generic/ambiguity corpus
T21 C4 stress, broad certification, walkthrough, handoff
G7  C4 complete
```

Every task below ends in a reviewer-meaningful, independently testable state. Do not merge multiple gates into one unreviewed patch.

---

## 23. T0 — Post-P2 Entry Lock and Whole-Workspace Audit

**Purpose:** establish the exact implementation baseline and reproduce the post-P2 audit findings before feature edits.

**Files:** read-only except checkpoint evidence update.

```text
Read:
  docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
  docs/implementation/LANG005/LANG005.C4/LANG005.C4-GUIDANCE.md
  docs/implementation/LANG005/LANG005.C4/LANG005.C4.P2-handoff.md
  phalcom-semantic/src/identity.rs
  phalcom-semantic/src/impls.rs
  phalcom-semantic/src/snapshot.rs
  phalcom-semantic/src/checker/context.rs
  phalcom-semantic/src/checker/analysis.rs
  phalcom-core/src/modules/semantic_lowering.rs
  phalcom-core/src/compiler/lib/impl_decl.rs
  phalcom-core/src/frame.rs
Modify:
  docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
```

### Interfaces

**Consumes:** landed P2 handoff.

**Produces:** a recorded P3 entry revision, failure classification table, and verified list of live symbols to be used by T1–T21.

### Steps

- [ ] **T0.1 Record repository state before edits.**

```bash
git status --short
git branch --show-current
git rev-parse HEAD
git log -1 --oneline
```

Record unrelated local changes; never overwrite them.

- [ ] **T0.2 Run compile gates that P2 did not certify.**

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace
```

Expected: either all PASS, or failures are classified as `P2_ENTRY_DEFECT`/preexisting evidence. A `CallableOwnerId::Conformance` exhaustive-match error is expected from the audit and goes to T1.

- [ ] **T0.3 Audit declaration-owner assumptions.**

```bash
rg -n 'declaration_owner\(|\.declaration\(\)|Deref.*DeclarationId|CallableOwnerId::Declaration|CallableOwnerId::Variant' \
  phalcom-semantic phalcom-core phalcom-lsp
```

Classify every reachable site as:

```text
requires lexical declaration owner
supports conformance owner
must explicitly reject conformance owner
must use source module/ImplId instead
```

- [ ] **T0.4 Reproduce P2 correctness findings with the smallest existing/new tests before repairs.**

At minimum reproduce:

```text
missing Iterable requirements without diagnostic
Unknown completeness collapsing to Incomplete at exact query
known-failure + unknown aggregation
conditional witness vs default exact specialization
explicit witness vs incompatible inherent selector
Valued<U> for Box<U> generic inherent witness specialization
```

Do not change production behavior in T0.

- [ ] **T0.5 Pin live symbol/path map in C4 checkpoint.**

Record actual names for:

```text
ConformanceWitnessPlan
ConformanceEvidence
RequirementSelectionTemplate
exact evidence resolver
callable-definition origin
canonical dispatch resolver
ExpressionAnalysis
semantic lowering module
compiler behavior-member compiler
runtime type environment registry
CallFrame
```

- [ ] **T0.6 Gate evidence.**

T0 exits when every §4 finding is either reproduced or proven already-correct by a precise test. Update checkpoint status to `P3 ACTIVE / T0 COMPLETE`.

**Commit:** documentation-only if checkpoint state changed materially.

```bash
git add docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
git commit -m "docs: lock C4.P3 post-P2 entry state"
```

---

## 24. T1 — Callable-Owner Cross-Crate Closure

**Purpose:** make conformance-owned callables a safe first-class semantic/compiler identity without fake declaration ownership or latent panics.

**Files:**

```text
Modify:
  phalcom-semantic/src/identity.rs
  phalcom-semantic/src/checker/body.rs
  phalcom-semantic/src/editor.rs
  phalcom-semantic/src/source_index/builder.rs
  phalcom-core/src/modules/semantic_lowering.rs
  phalcom-core/src/compiler/lib/impl_decl.rs
  any additional owner-match sites proven by T0
Test:
  phalcom-semantic/src/identity.rs unit tests
  phalcom-semantic/tests/semantic/impls/queries.rs
  phalcom-core/tests/core/language/compiler/*
```

### Interfaces

**Consumes:** `CallableOwnerId::Conformance(ImplId)`, `CallableDefinitionOrigin::{InherentImpl, ConformanceWitness,...}`.

**Produces:** total source-module ownership, optional declaration ownership, explicit core lowering behavior by definition origin, and no generic `Deref<DeclarationId>` dependency.

### Steps

- [ ] **T1.1 Write identity tests proving conformance owners have no declaration owner and all safe accessors are total.**

Add assertions for:

```rust
assert!(callable.try_declaration_owner().is_none());
assert_eq!(callable.conformance_owner(), Some(&impl_id));
assert_eq!(callable.module(), &impl_id.module);
```

Add a compile-time/usage regression that production helpers no longer depend on implicit `Deref<DeclarationId>` for generic callable-owner handling.

- [ ] **T1.2 Remove or quarantine `Deref<Target = DeclarationId>` from `CallableOwnerId`.**

Preferred result: delete the impl and repair compile sites with explicit `try_declaration()`/pattern matching. If a narrow legacy API absolutely requires it, move it behind a declaration-only wrapper type rather than retaining a panicking general deref.

- [ ] **T1.3 Replace unsafe `declaration_owner()` calls on paths reachable by conformance callables.**

Use:

```text
try_declaration_owner()
conformance_owner()
module()
explicit owner match
```

according to actual semantic need.

Do not convert a conformance `ImplId` into a fabricated `DeclarationId`.

- [ ] **T1.4 Fix core semantic-lowering exhaustive matches by semantic origin, not wildcard suppression.**

For inherent projection, filter `CallableDefinitionOrigin::InherentImpl` first, then require owner `Declaration|Variant`. A `ConformanceWitness` definition belongs to the later trait lowering lane and must not be treated as an inherent target.

- [ ] **T1.5 Factor `compile_behavior_member` usage so compiling a method and installing it are clearly separate operations.**

No trait method execution yet; establish the seam T12 will consume. Existing inherent installation behavior must remain byte-for-byte semantically equivalent.

- [ ] **T1.6 Focused verification.**

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic identity
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::inherent_impl
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Acceptance: EC-01, EC-02, EC-03.

- [ ] **T1.7 Commit.**

```bash
git add phalcom-semantic phalcom-core
git commit -m "lang005: generalize callable ownership for C4 dispatch"
```

---

## 25. T2 — P2 Completeness, Proof-State, and Diagnostic Closure

**Purpose:** make P2 evidence trustworthy before it becomes a dispatch/execution authority.

**Files:**

```text
Modify:
  phalcom-semantic/src/impls.rs
  phalcom-semantic/src/snapshot.rs
  phalcom-semantic/src/session.rs
  phalcom-semantic/src/diagnostic.rs
Test:
  phalcom-semantic/tests/semantic/impls/queries.rs
  phalcom-semantic/tests/semantic/capabilities/traits.rs
```

### Interfaces

**Consumes:** `ConformanceCompleteness`, `RequirementFailure`, `ConformanceResolution`.

**Produces:** exact total proof-state mapping; deterministic completeness-state join; source diagnostics for invalid conformance.

### Steps

- [ ] **T2.1 Add failing diagnostic tests for missing and incompatible witnesses.**

Required source examples:

```phalcom
trait Sized { size -> Int }
class Empty {}
impl Sized for Empty {}
```

and:

```phalcom
trait Tagged { tag -> String }
class User {}
impl Tagged for User { tag -> Int { 1 } }
```

Assert a precise trait/conformance diagnostic family, not merely `snapshot.has_errors()`.

- [ ] **T2.2 Update old P1 head-only fixtures that now cross the P2 boundary.**

If a test wants to test only P1 head matching, either provide complete witness bodies/defaults or assert diagnostics separately. Do not leave semantically invalid conformance source under `assert!(!snapshot.has_errors())`.

- [ ] **T2.3 Implement source diagnostic projection from `ConformanceWitnessPlan::completeness`.**

`Incomplete { failures }` must produce a primary conformance-incomplete diagnostic with requirement-specific labels/messages. Do not double-report explicit-member syntax diagnostics unnecessarily; deduplicate by source cause where the existing diagnostic architecture supports it.

- [ ] **T2.4 Add exact resolution proof-state tests.**

Construct plans that settle to each supported state and assert:

```text
Unknown        → ConformanceResolution::Unknown
Blocked        → Blocked
Dynamic        → Dynamic
Cancelled      → Cancelled
BudgetExceeded → BudgetExceeded
InternalFailure→ InternalFailure
Incomplete     → Incomplete
```

- [ ] **T2.5 Replace the non-Complete blanket collapse in `resolve_conformance_evidence`.**

Map every `ConformanceCompleteness` variant one-for-one into `ConformanceResolution` before exact materialization.

- [ ] **T2.6 Add a multi-requirement failure-vs-uncertainty regression.**

One requirement must be definitely incompatible/missing; a second must yield `Unknown`/`Blocked`. Expected conformance state: `Incomplete`, retaining the known failure. It must not become merely `Unknown`.

- [ ] **T2.7 Implement canonical aggregate precedence.**

Use this proof ordering:

```text
any definite RequirementFailure
    → Incomplete
otherwise terminal analysis states
    → preserve canonical query-status priority
otherwise all selected
    → Complete
```

Within terminal states, reuse repository query-status precedence rather than inventing a second ordering if one exists.

- [ ] **T2.8 Focused verification.**

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls::queries
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::traits
```

Acceptance: EC-04 through EC-08, EC-17.

- [ ] **T2.9 Commit.**

```bash
git add phalcom-semantic
git commit -m "lang005: close conformance completeness proof states"
```

---

## 26. T3 — P2 Witness-Compatibility Correctness Closure

**Purpose:** prove the exact witness relationships P3 will execute are type/access/selector correct.

**Files:**

```text
Modify:
  phalcom-semantic/src/impls.rs
  phalcom-semantic/src/checker/context.rs or canonical access helper owner
  phalcom-semantic/src/editor.rs only if sharing access-coverage helper
Test:
  phalcom-semantic/tests/semantic/impls/queries.rs
  phalcom-semantic/tests/semantic/impls/targets.rs
  existing visibility/access tests
```

### Interfaces

**Consumes:** `InstantiatedTraitRequirement`, effective inherent witness resolution, access semantics.

**Produces:** source-generic target-member specialization; explicit-witness conflict rejection; canonical access coverage; witness-kind-aware compatibility proof.

### Steps

- [ ] **T3.1 Add generic inherent witness specialization regression.**

```phalcom
trait Valued<T> { value -> T }
class Box<T> { value -> T { ... } }
impl<U> Valued<U> for Box<U> {}
```

Assert the source plan is complete and the selected inherent callable remains `Box.value` after specializing declaration parameter `T_Box := U_impl`.

- [ ] **T3.2 If failing, specialize ordinary direct/inherited candidate signatures through the conformance target environment before compatibility.**

Reuse `specialize_receiver_to_owner`, `TypeEnvironment`, or the P2 conformance environment. Do not rename declaration-owned generic parameter identity; materialize a view for comparison.

- [ ] **T3.3 Add explicit witness vs incompatible inherent selector regression.**

```phalcom
trait Renderable { render -> String }
class Item { render -> Int { 1 } }
impl Renderable for Item { render -> String { "x" } }
```

Expected: invalid conformance. The explicit witness cannot create a second ordinary selector distinguished only by return type.

- [ ] **T3.4 Implement selector-conflict validation before accepting an explicit conformance-local witness.**

If effective inherent behavior already owns the same selector, it must itself be compatible with the trait requirement for the conformance to be valid. A conformance-local body does not shadow the target's inherent selector.

- [ ] **T3.5 Add access-coverage matrix tests across owner/module contexts.**

Cover public/private/protected/internal with trait-owner and target-owner conformances. Assert actual promised access set coverage, not literal visibility-token equality.

- [ ] **T3.6 Factor/reuse canonical access coverage.**

Prefer one helper that answers:

```text
can witness_access cover requirement_access for all promised call contexts?
```

It must reason from owner/module/hierarchy context used by ordinary semantic access rules.

- [ ] **T3.7 Make compatibility proof witness-kind aware.**

Replace any data-component proof field that stores a fake callable identity with an enum or structure such as:

```rust
enum WitnessIdentity {
    Callable(CallableId),
    DataComponent(DataComponentId),
}
```

or remove witness identity from the proof if the selection already owns it. Final evidence must preserve the real `DataComponentId`.

- [ ] **T3.8 Do not extend P2 generic constraint implication beyond current callable constraints.**

If factoring is required, delegate to canonical type relation/constraint implication. Do not add `Conforms` constraints or nested trait proof.

- [ ] **T3.9 Focused verification.**

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic visibility
```

Use the repository's actual visibility test filter if different.

Acceptance: EC-09 through EC-13.

- [ ] **T3.10 Commit.**

```bash
git add phalcom-semantic
git commit -m "lang005: harden C4 witness compatibility"
```

---

## 27. T4 — Exact-Dependent Conditional Witness/Default Decision Template

**Purpose:** preserve “concrete compatible witness > trait default” for generic source conformances whose conditional inherent witness becomes decidable only at exact application.

**Files:**

```text
Modify:
  phalcom-semantic/src/impls.rs
  phalcom-semantic/src/snapshot.rs if exact materialization helper lives there
  phalcom-semantic/src/db/fingerprint.rs if plan/evidence fingerprints are centralized
Test:
  phalcom-semantic/tests/semantic/impls/queries.rs
  phalcom-semantic/tests/semantic/incremental/*
```

### Interfaces

**Consumes:** proof-aware `EffectiveInherentWitnessResolution`, retained `InherentImplSpecialization`, trait default selection.

**Produces:** source plan capable of retaining `conditional concrete candidate + fallback`; exact evidence materializes that decision without witness rediscovery.

### Steps

- [ ] **T4.1 Add the discriminating generic fixture.**

```phalcom
trait Tagged {
  tag -> String { "default" }
}

class Value<T> {}

impl<T> Value<T> where T <: Number {
  tag -> String { "number" }
}

impl<T> Tagged for Value<T> {}
```

Exact assertions:

```text
Value<Int>    → conditional inherent Value.tag
Value<String> → trait default
```

Use actual core numeric type names supported by the repository.

- [ ] **T4.2 Prove current failure before changing production code.**

Expected pre-fix failure: source plan freezes `TraitDefault` or exact result otherwise violates precedence.

- [ ] **T4.3 Extend `RequirementSelectionTemplate` with a retained exact-dependent decision.**

Retain:

```text
conditional candidate callable
conditional ImplId/domain specialization evidence template
fallback selection
```

The fallback may be data/default/missing according to the source selection pipeline. Explicit conformance witnesses remain higher priority and do not use this variant.

- [ ] **T4.4 Materialize the retained decision in exact evidence.**

Use the exact conformance environment and retained canonical C2 applicability product. Do not re-run `resolve_effective_inherent_witness` or rescan target members.

- [ ] **T4.5 Preserve terminal states.**

If exact conditional applicability remains Unknown/Blocked/Dynamic/etc., do not silently use the fallback unless P2's fixed proof rules establish that fallback is independently valid regardless of that candidate. The plan/evidence state must remain sound.

- [ ] **T4.6 Extend plan/evidence fingerprints.**

The retained candidate, fallback, and exact selected branch must enter semantic fingerprints. Body text/ranges must remain excluded where they were previously excluded.

- [ ] **T4.7 Add incremental tests.**

Edit the conditional domain so `Value<Int>` toggles between concrete/default selection and compare incremental vs cold evidence/selection fingerprints.

- [ ] **T4.8 G1 verification.**

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::traits
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace
```

G1 acceptance: EC-01 through EC-18 all proven; P2 evidence is safe for P3 execution consumers.

- [ ] **T4.9 Update checkpoint and commit.**

```bash
git add phalcom-semantic docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
git commit -m "lang005: finalize C4 conformance evidence for dispatch"
```

---

## 28. T5 — Trait-Dispatch Source Contribution and Index

**Purpose:** make ordinary receiver+selector lookup discover only plausible conformance requirements without workspace scans.

**Files:**

```text
Create:
  phalcom-semantic/src/trait_dispatch.rs
Modify:
  phalcom-semantic/src/lib.rs
  phalcom-semantic/src/snapshot.rs
  phalcom-semantic/src/session.rs
  phalcom-semantic/src/semantic_shard.rs if source contribution fingerprints are needed
Test:
  phalcom-semantic/tests/semantic/impls/queries.rs or new focused trait_dispatch.rs test module
  phalcom-semantic/tests/semantic/incremental/*
```

### Interfaces

**Consumes:** eligible P1 `ConformanceContribution`s, C3 `TraitSurface` requirements.

**Produces:** immutable `TraitDispatchIndex`, source contributions keyed by `(target family, selector, side)`.

### Steps

- [ ] **T5.1 Write index construction tests first.**

Use conformances for:

```text
Value<Int> + Tagged.tag
Value<String> + Tagged.tag
User + Named.name
exact Result::Ok + HasValue.value
```

Assert bucket membership without claiming exact conformance proof.

- [ ] **T5.2 Define `TraitDispatchTargetFamily`, `TraitDispatchContribution`, and deterministic bucket key.**

Use canonical IDs only; no source strings.

- [ ] **T5.3 Build index from eligible conformance sources + trait surface requirements.**

Unauthorized/invalid P1 heads do not publish. P2 incomplete status does not need to be baked into the coarse index; exact resolution will filter it.

- [ ] **T5.4 Publish index in `SemanticSnapshot`.**

Add a snapshot-owned immutable `Arc<TraitDispatchIndex>` and constructor/with-method wiring consistent with existing products.

- [ ] **T5.5 Implement owner-complete incremental replacement/removal.**

Changing/removing a source module removes every dispatch contribution from its `ImplId`s before replacement.

- [ ] **T5.6 Add deterministic fingerprint/reuse behavior.**

Source ranges/body bytes do not alter a contribution if target/trait requirement selector/side identities are unchanged.

- [ ] **T5.7 Focused verification.**

Acceptance: DI-02, DI-11, DI-12.

- [ ] **T5.8 Commit.**

```bash
git add phalcom-semantic
git commit -m "lang005: index trait-evidenced member candidates"
```

---

## 29. T6 — Exact Receiver → TraitRef/Evidence Candidate Query

**Purpose:** turn a bounded discovery bucket into exact proven trait candidates using P1/P2 authority.

**Files:**

```text
Modify:
  phalcom-semantic/src/trait_dispatch.rs
  phalcom-semantic/src/impls.rs only to expose/reuse exact target matcher if needed
  phalcom-semantic/src/snapshot.rs query facade
Test:
  semantic trait-dispatch tests
```

### Interfaces

**Consumes:** `TraitDispatchIndex`, P1 target matcher, `ConformanceIndex`, `ConformanceWitnessPlan`, `TraitSurface`, `resolve_conformance_evidence`.

**Produces:** `TraitEvidencedMemberCandidate` and `TraitDispatchResolution` with exact proof states.

### Steps

- [ ] **T6.1 Write exact specialized target tests.**

Query by `(Value<Int>, tag, instance)` and `(Value<String>, tag, instance)` and assert exact source `ImplId`/`TraitRef`/requirement selection differ where expected.

- [ ] **T6.2 Factor/reuse target-template matching from P1.**

Do not call `query_exact` with an unknown TraitRef and do not duplicate unification. The target match must bind one impl environment from the exact receiver.

- [ ] **T6.3 Materialize exact TraitRef from the same impl substitution.**

Then invoke the canonical exact P2 evidence resolver.

- [ ] **T6.4 Preserve terminal/non-proof resolution.**

If evidence is incomplete/unknown/blocked/etc., return the corresponding trait-dispatch proof state. Only `Proven` creates a usable member candidate.

- [ ] **T6.5 Instantiate the exact requirement signature.**

The candidate stores the exact requirement contract after trait/target/impl substitution and `Self := receiver`.

- [ ] **T6.6 Add exact enum-case tests.**

Case candidate found; root and sibling missing.

- [ ] **T6.7 Add non-propagation inheritance test.**

`impl Printable for Base` alone must not produce a `Child` trait-dispatch candidate.

- [ ] **T6.8 Focused verification and commit.**

Acceptance: DI-01, DI-03 through DI-10.

```bash
git add phalcom-semantic
git commit -m "lang005: resolve exact trait dispatch evidence"
```

---

## 30. T7 — Convergence and Ambiguity

**Purpose:** define deterministic resolution when multiple proven traits expose the same ordinary selector.

**Files:**

```text
Modify:
  phalcom-semantic/src/trait_dispatch.rs
  phalcom-semantic/src/diagnostic.rs
Test:
  semantic trait-dispatch tests
```

### Interfaces

**Consumes:** proven `TraitEvidencedMemberCandidate`s.

**Produces:** canonical convergence key, `TraitDispatchSelection`, trait-specific ambiguity product.

### Steps

- [ ] **T7.1 Add shared inherent witness test.**

`Named.name` and `DisplayNamed.name` both select one `Person.name`; resolution is one selection, while both requirement identities remain inspectable.

- [ ] **T7.2 Add shared data-component test.**

Two getter requirements selecting one `Person.name` component converge.

- [ ] **T7.3 Add shared C2 conditional witness test.**

Convergence requires same callable + same conditional impl/proof identity.

- [ ] **T7.4 Add competing-default test.**

Two trait defaults with same selector and no concrete inherent member return `Ambiguous`, independent of source order.

- [ ] **T7.5 Add distinct conformance-local witness ambiguity test.**

Two traits selecting different conformance-owned callables for the same ordinary selector remain ambiguous.

- [ ] **T7.6 Add “same default callable/different evidence” guard.**

Do not collapse merely on callable identity if the active conformance context differs.

- [ ] **T7.7 Implement `TraitDispatchConvergenceKey` and deterministic candidate sort.**

No hash iteration order in diagnostics or selection.

- [ ] **T7.8 Add trait-specific ambiguity diagnostic data.**

Keep trait refs/requirements/selection origins for later checker reporting.

- [ ] **T7.9 G2 verification.**

Run focused semantic trait/impl suites and reverse declaration/module order variants.

Acceptance: OD-05 through OD-10, OD-12, plus all DI coverage.

- [ ] **T7.10 Update checkpoint and commit.**

```bash
git add phalcom-semantic docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
git commit -m "lang005: define trait dispatch convergence and ambiguity"
```

---

## 31. T8 — Checker Wiring and Canonical Ordinary Dispatch Integration

**Purpose:** make ordinary semantic member lookup consume trait evidence from one canonical seam.

**Files:**

```text
Modify:
  phalcom-semantic/src/checker/context.rs
  phalcom-semantic/src/checker/body.rs or query input structure
  phalcom-semantic/src/session.rs
  phalcom-semantic/src/dispatch.rs only if result type needs generalization
Test:
  semantic expression/call/getter/setter/index tests
  trait dispatch integration tests
```

### Interfaces

**Consumes:** snapshot `TraitDispatchIndex`, P1/P2 products, `TraitDispatchResolution`.

**Produces:** checker-attached immutable conformance semantic view and trait fallback in canonical dispatch.

### Steps

- [ ] **T8.1 Introduce an immutable checker view rather than cloning whole snapshots.**

Conceptually:

```rust
pub struct ConformanceSemanticView<'a> {
    pub trait_dispatch: &'a TraitDispatchIndex,
    pub conformance_index: &'a ConformanceIndex,
    pub witness_plans: &'a BTreeMap<ImplId, Arc<ConformanceWitnessPlan>>,
    pub trait_surfaces: &'a TraitSurfaceTable,
}
```

Attach it to `CheckingContext` with the same borrowed-product pattern used by data/enum/associated tables.

- [ ] **T8.2 Write one trait-only getter/method lookup test that currently reports missing.**

Example:

```phalcom
trait Tagged { tag -> String { "default" } }
class User {}
impl Tagged for User {}
class Caller { read(_ user: User) -> String { user.tag } }
```

Expected after fix: call expression known as `String`, no method-missing diagnostic.

- [ ] **T8.3 Integrate trait fallback only after inherent lookup returns `Missing`.**

Do not run trait candidate resolution when inherent lookup already succeeded or produced inherent ambiguity/dynamic state.

- [ ] **T8.4 Convert a found trait candidate to the canonical call-application shape.**

Use exact instantiated requirement signature. Preserve witness selection separately for lowering.

- [ ] **T8.5 Map trait ambiguity to trait-specific diagnostic path.**

Do not flatten candidate details into ordinary `ResolvedDispatchResult::Ambiguous` if that would lose requirement/evidence provenance; generalize the result or retain a side product.

- [ ] **T8.6 Map terminal conformance states to analysis status without calling them missing.**

- [ ] **T8.7 Verify inherent winner and class-side exclusion.**

Existing inherent member remains selected; class-object lookup does not inspect C4 instance conformances.

- [ ] **T8.8 Verify syntax routes through the same resolver.**

At minimum method/getter plus existing index/operator protocol tests where canonical dispatch is shared. Do not add duplicate trait logic to syntax visitors.

- [ ] **T8.9 Commit.**

```bash
git add phalcom-semantic
git commit -m "lang005: integrate trait evidence into ordinary dispatch"
```

---

## 32. T9 — Per-Expression Abstract/Evidenced Trait Dispatch Publication

**Purpose:** create the exact semantic handoff that compiler lowering will consume.

**Files:**

```text
Modify:
  phalcom-semantic/src/checker/analysis.rs
  phalcom-semantic/src/checker/context.rs
  phalcom-semantic/src/checker/associated.rs
  phalcom-semantic/src/db/fingerprint.rs
Test:
  semantic expression-product tests
  semantic trait default analysis tests
```

### Interfaces

**Consumes:** trait resolution result; C3 abstract contract lookup.

**Produces:** `TraitDispatchSite::{AbstractRequirement,Evidenced}` attached to `ExpressionAnalysis`.

### Steps

- [ ] **T9.1 Define `TraitDispatchSelection` and `TraitDispatchSite` in a semantic owner visible to analysis/lowering.**

Do not put runtime handles here.

- [ ] **T9.2 Add `ExpressionAnalysis::trait_dispatch`.**

Initialize to `None` in every constructor/test fixture.

- [ ] **T9.3 Publish `Evidenced` on ordinary trait-resolved calls.**

Store exact trait ref, requirement, source impl, evidence fingerprint, and selected requirement target.

- [ ] **T9.4 Publish `AbstractRequirement` inside C3 trait default analysis.**

When `resolve_trait_contract_target` sees `Self` and a trait surface member, retain the exact `TraitRequirementId`; do not rely only on trait-owned callable ID.

- [ ] **T9.5 Synchronize re-analysis paths.**

A reanalyzed expression must clear/replace stale trait dispatch exactly as call specialization/conditional dispatch already do.

- [ ] **T9.6 Include trait dispatch in semantic body/product fingerprints.**

Hash semantic IDs/fingerprints, not source ranges/debug strings where canonical hash helpers exist.

- [ ] **T9.7 Inspect semantic products in tests.**

Executable output is not enough. Assert each site records the correct requirement and selection origin.

Acceptance: XP-01 through XP-09.

- [ ] **T9.8 Commit.**

```bash
git add phalcom-semantic
git commit -m "lang005: publish per-expression trait dispatch evidence"
```

---

## 33. T10 — Semantic Dependencies, Fingerprints, and Incremental Lifecycle

**Purpose:** make trait-dispatch consumers incrementally correct before compiler/runtime work builds on them.

**Files:**

```text
Modify:
  phalcom-semantic/src/checker/analysis.rs
  phalcom-semantic/src/checker/context.rs
  phalcom-semantic/src/db/query.rs
  phalcom-semantic/src/db/fingerprint.rs
  phalcom-semantic/src/session.rs
  phalcom-semantic/src/semantic_shard.rs as needed
Test:
  phalcom-semantic/tests/semantic/incremental/*
```

### Interfaces

**Consumes:** source/head/plan/trait-surface/callable/data dependencies.

**Produces:** exact reverse invalidation for trait-dispatch selection and cold/incremental parity.

### Steps

- [ ] **T10.1 Add dependency variants/query keys for conformance head and witness plan if no equivalent exists.**

Do not use aggregate module dependency as substitute.

- [ ] **T10.2 Record dependencies during exact trait candidate resolution.**

At minimum trait surface, selected source impl/head/plan, and selected concrete source signature/data/hierarchy/C2 proof path.

- [ ] **T10.3 Add incremental matrix tests:**

```text
add/delete conformance
add/delete trait default
witness signature edit
witness body-only edit
trait requirement signature edit
inherited override edit
C2 conditional domain edit
add/remove competing default
module deletion
```

- [ ] **T10.4 Assert body-only witness edit behavior.**

Semantic call selection/evidence fingerprint remains stable if signature/selection are unchanged; callable body product changes.

- [ ] **T10.5 Assert cold/incremental equality of `TraitDispatchSelection`.**

Compare exact semantic product/fingerprint, not only diagnostics.

- [ ] **T10.6 G3 verification.**

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::traits
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

G3 acceptance: semantic ordinary trait dispatch is complete and incrementally correct before lowering starts.

- [ ] **T10.7 Update checkpoint and commit.**

```bash
git add phalcom-semantic docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
git commit -m "lang005: complete semantic trait dispatch publication"
```

---

## 34. T11 — Executable Conformance Lowering Model

**Purpose:** project exact semantic trait selections/evidence into compiler-owned executable plans without semantic re-query.

**Files:**

```text
Modify:
  phalcom-core/src/modules/semantic_lowering.rs
  phalcom-core/src/modules/mod.rs
Test:
  phalcom-core/tests/core/language/compiler/trait_lowering.rs   # create or integrate into compiler lowering suite
```

### Interfaces

**Consumes:** `ExpressionAnalysis::trait_dispatch`, `ConformanceEvidence`, `RequirementSelectionTemplate`, callable definitions, data layout, conditional invocation products.

**Produces:** `ExecutableConformancePlan`, deterministic requirement slots, trait invocation lowering attachments.

### Steps

- [ ] **T11.1 Write projection tests before defining runtime bytecode.**

Compile semantic snapshots for one conformance-local witness, one default, one data witness, one conditional inherent witness. Inspect lowering products only.

- [ ] **T11.2 Define executable plan/requirement target types.**

Keep semantic IDs needed for debug/provenance, but runtime execution fields must be compact and deterministic.

- [ ] **T11.3 Assign deterministic requirement slots from canonical trait surface order.**

Create and test a stable `TraitRequirementId → slot` map. Reversing source/module traversal must produce the same slots.

- [ ] **T11.4 Project every P2 selection origin.**

```text
InherentCallable         → inherent executable target
conditional inherent     → retained conditional executable target
ConformanceCallable      → detached-method placeholder/source key
DataComponent            → component/index target
TraitDefault             → detached-method placeholder/source key
```

- [ ] **T11.5 Project ordinary evidenced sites and abstract requirement sites separately.**

Add lowering-site kinds/specs such as:

```text
TraitInvoke
TraitRequirementInvoke
TraitCallableReference
```

or equivalent explicit roles.

- [ ] **T11.6 Enforce no semantic re-query in `phalcom-core`.**

The projection function may read the already-published exact evidence/selection from snapshot products, but it must not call trait candidate discovery to decide another winner.

- [ ] **T11.7 Add no-injection lowering assertion.**

`inherent_impls` for the target do not contain conformance/default callables.

Acceptance: LW-01 through LW-10 at projection level where applicable.

- [ ] **T11.8 Commit.**

```bash
git add phalcom-core
git commit -m "lang005: project executable conformance plans"
```

---

## 35. T12 — Detached Witness/Default Compilation and Source Retrieval

**Purpose:** compile executable bodies selected by conformance evidence without target method installation.

**Files:**

```text
Modify:
  phalcom-core/src/compiler/lib/impl_decl.rs
  phalcom-core/src/compiler/lib/mod.rs
  phalcom-core/src/compiler/lib/error.rs
  phalcom-core/src/modules/semantic_lowering.rs as source metadata requires
Possible create:
  phalcom-core/src/compiler/lib/trait_dispatch.rs
Test:
  compiler trait lowering tests
```

### Interfaces

**Consumes:** executable plan placeholders, `CallableDefinitionOrigin::ConformanceWitness(ImplId)`, source member index, C3 trait default callable/source identity.

**Produces:** compiler cache mapping selected conformance/default `CallableId` + generic environment recipe to detached `MethodObject` handles.

### Steps

- [ ] **T12.1 Add a compiler test that requests a conformance witness method and proves it is compiled but not installed.**

Inspect target class method table after compilation/execution setup.

- [ ] **T12.2 Factor behavior-member compilation from installation if T1 did not fully isolate it.**

A helper must accept:

```text
BehaviorMember
expected CallableId
```

and return compiled method metadata without emitting `Bytecode::Method`/`VariantMethod` installation.

- [ ] **T12.3 Implement conformance witness source retrieval.**

Use semantic callable definition origin + `ImplId` + `source_member_index`. Do not scan by selector alone.

- [ ] **T12.4 Implement trait default source retrieval.**

Use C3 trait callable/requirement/source identity. A bodyless requirement has no compilable default and reaching it as `TraitDefault` is an internal invariant violation.

- [ ] **T12.5 Compile/cache detached methods.**

Cache by canonical callable + compile mode and any runtime generic recipe identity required by existing compiler architecture. Do not mutate target class dictionaries.

- [ ] **T12.6 Preserve debug/source metadata and contracts exactly as ordinary method compilation does.**

- [ ] **T12.7 Regression ordinary inherent installation.**

Run existing inherent compiler/runtime suite.

- [ ] **T12.8 Commit.**

```bash
git add phalcom-core
git commit -m "lang005: compile detached trait and conformance methods"
```

---

## 36. T13 — Runtime Conformance Plan/Environment Registry and Frame Field

**Purpose:** create the compact runtime carrier used by selected calls and trait-default requirement invocations.

**Files:**

```text
Create or modify:
  phalcom-core/src/typing/conformance_environment.rs
  phalcom-core/src/typing/mod.rs
Modify:
  phalcom-core/src/frame.rs
  phalcom-core/src/vm/mod.rs or VM owner of registries
  phalcom-core/src/heap/GC tracing/root enumeration where registry handles require it
Test:
  runtime registry/frame unit tests
  GC tests
```

### Interfaces

**Consumes:** compiled `ExecutableConformancePlan`, `RuntimeTypeEnvironmentId`.

**Produces:** `RuntimeConformancePlanId`, `RuntimeConformanceEnvironmentId`, registries, `CallFrame::conformance_environment`.

### Steps

- [ ] **T13.1 Mirror established runtime type-environment conventions.**

Define compact ID types with `EMPTY` where appropriate and deterministic/hashable environment keys.

- [ ] **T13.2 Add plan registry.**

Register immutable executable plans and return compact IDs. If plans contain `ObjRef` methods, ensure registry lifecycle is VM-owned.

- [ ] **T13.3 Add environment registry.**

Intern `(plan_id, type_environment_id)` to one compact conformance environment ID.

- [ ] **T13.4 Add `conformance_environment` to `CallFrame`.**

`CallFrame::new` initializes it to `EMPTY`; block/fiber/frame-copy paths inherit/copy according to normal frame semantics only when the call helper explicitly sets it.

- [ ] **T13.5 Audit every frame construction/copy/park/unpark path.**

Because `CallFrame` is `Copy`, compile errors should expose missed literals; still inspect fiber storage/restoration and stack-walk presentation.

- [ ] **T13.6 Root executable plan method handles.**

If registry stores `ObjRef`, extend GC tracing/root enumeration. Add forced-GC unit test that plan methods remain alive with no target class reference.

- [ ] **T13.7 Assert ordinary calls keep `EMPTY`.**

No ambient conformance context leaks between unrelated calls.

- [ ] **T13.8 Commit.**

```bash
git add phalcom-core
git commit -m "lang005: add runtime conformance environments"
```

---

## 37. T14 — Trait-Selected Invocation Bytecode and Runtime

**Purpose:** execute ordinary source calls that semantic analysis resolved through exact trait evidence.

**Files:**

```text
Modify:
  phalcom-core/src/bytecode.rs
  phalcom-core/src/chunk.rs
  phalcom-core/bin/phalcom/disasm.rs
  phalcom-core/src/compiler/lib/expr.rs
  phalcom-core/src/compiler/lib/trait_dispatch.rs or equivalent
  phalcom-core/src/vm/dispatch.rs
Test:
  phalcom-core/tests/core/language/traits.rs
  compiler/disassembly tests
```

### Interfaces

**Consumes:** trait invoke lowering attachment + runtime executable plan.

**Produces:** `InvokeTraitSelected` executable semantics and callee frame with exact conformance env.

### Steps

- [ ] **T14.1 Add bytecode encoding/decoding/disassembly tests.**

The opcode must identify an executable plan/requirement slot (directly or through chunk table) and arity.

- [ ] **T14.2 Compile trait-evidenced ordinary call sites to `InvokeTraitSelected`.**

Compile receiver/arguments exactly once according to the existing call ABI. Do not emit ordinary `Invoke` as a fallback for trait-only behavior.

- [ ] **T14.3 Implement VM target selection from the compiler-provided executable plan.**

Runtime does not inspect semantic indexes. The opcode already identifies the selected plan/slot.

- [ ] **T14.4 Invoke callable targets with exact receiver and runtime type environment.**

Intern/obtain the runtime conformance environment and stamp the callee frame.

- [ ] **T14.5 Implement direct data-component target behavior only if ordinary call syntax reaches it at this stage.**

Getter requirement returns component value without method synthesis.

- [ ] **T14.6 Add first executable conformance-witness fixture.**

```phalcom
trait Tagged { tag -> String }
class User {}
impl Tagged for User { tag -> String { "user" } }
Assert.equal(User.new().tag, "user")
```

- [ ] **T14.7 Add first executable default fixture.**

```phalcom
trait Identified { identity -> String { "anonymous" } }
class Anonymous {}
impl Identified for Anonymous {}
Assert.equal(Anonymous.new().identity, "anonymous")
```

- [ ] **T14.8 Assert no target runtime method installation.**

Use runtime introspection/test helper on class method dictionary: `tag`/`identity` are absent unless independently inherent.

- [ ] **T14.9 G4 verification.**

Run focused compiler + trait runtime + inherent regression suites. Acceptance: RT-01, RT-02, RT-13, RT-14, RT-17.

- [ ] **T14.10 Commit.**

```bash
git add phalcom-core
git commit -m "lang005: execute trait-selected calls"
```

---

## 38. T15 — Abstract Requirement Bytecode and Default→Requirement Execution

**Purpose:** solve the central C4 execution invariant: a C3 trait default's abstract call must use the active exact conformance evidence.

**Files:**

```text
Modify:
  phalcom-core/src/bytecode.rs
  phalcom-core/src/chunk.rs
  phalcom-core/bin/phalcom/disasm.rs
  phalcom-core/src/modules/semantic_lowering.rs
  phalcom-core/src/compiler/lib/trait_dispatch.rs or trait-default compile path
  phalcom-core/src/vm/dispatch.rs
Test:
  phalcom-core/tests/core/language/traits.rs
```

### Interfaces

**Consumes:** `TraitDispatchSite::AbstractRequirement`, requirement slot map, active `RuntimeConformanceEnvironmentId`.

**Produces:** `InvokeTraitRequirement` runtime instruction resolving only within the active preselected executable plan.

### Steps

- [ ] **T15.1 Add canonical failing `Scalable<T>` runtime test.**

```phalcom
trait Scalable<T> {
  scale(_ value: T) -> T
  twice(_ value: T) -> T {
    self.scale(self.scale(value))
  }
}

class Doubler {}

impl Scalable<Int> for Doubler {
  scale(_ value: Int) -> Int { value * 2 }
}

Assert.equal(Doubler.new().twice(3), 12)
```

Before production fix, semantic analysis should succeed but lowering/runtime cannot correctly route the abstract call.

- [ ] **T15.2 Project abstract requirement sites to exact requirement slots.**

No AST selector lookup in compiler.

- [ ] **T15.3 Compile to `InvokeTraitRequirement`.**

The instruction does not encode a TraitRef search; it encodes the canonical requirement slot/reference in the currently active plan.

- [ ] **T15.4 Implement runtime requirement invocation.**

Read current frame's `conformance_environment`; `EMPTY` is an internal invariant failure. Resolve plan+slot and invoke its already-selected target.

- [ ] **T15.5 Preserve receiver and environment on nested requirement calls.**

The abstract call uses the executing default's `self`, not a trait object.

- [ ] **T15.6 Add bytecode assertion.**

Disassembly/default chunk must contain `InvokeTraitRequirement`, not ordinary dynamic send for `self.scale`.

- [ ] **T15.7 Add internal failure regression.**

A synthetic invocation of a requirement opcode without active env must fail closed as compiler/runtime internal error, never invoke `doesNotUnderstand` or scan the class.

- [ ] **T15.8 Commit.**

```bash
git add phalcom-core
git commit -m "lang005: dispatch trait defaults through conformance evidence"
```

---

## 39. T16 — Data/C2 Conditional/Default→Default Requirement Targets

**Purpose:** make every P2 witness origin executable through one requirement-slot mechanism and prove conformance context preservation.

**Files:**

```text
Modify:
  phalcom-core/src/modules/semantic_lowering.rs
  phalcom-core/src/compiler/lib/trait_dispatch.rs or equivalent
  phalcom-core/src/vm/dispatch.rs
  GC/rooting paths if new method caches are retained
Test:
  phalcom-core/tests/core/language/traits.rs
```

### Interfaces

**Consumes:** all `ExecutableRequirementTarget` variants.

**Produces:** complete requirement target execution.

### Steps

- [ ] **T16.1 Default→inherent witness fixture.**

A default calls an abstract requirement selected to an ordinary/inherited inherent method. Assert correct value and that no duplicate method was created.

- [ ] **T16.2 Default→C2 conditional witness fixture.**

Use an exact generic target for which P2 retained a proven conditional inherent specialization. VM executes the compiler-selected conditional target; no C2 applicability re-solve occurs.

- [ ] **T16.3 Default→data component fixture.**

```phalcom
trait Named {
  name -> String
  decorated -> String { "<\(self.name)>" }
}
data Person(name: String)
impl Named for Person {}
Assert.equal(Person.new(name: "A").decorated, "<A>")
```

Use actual data construction syntax from the repository.

- [ ] **T16.4 Default→default fixture.**

One default calls another default, which calls an abstract requirement. Assert the same runtime conformance environment ID is retained through both frames using an internal test hook if necessary.

- [ ] **T16.5 Recursive/default cycle safety.**

Compilation caches detached default methods by callable; compiling a recursive default graph must terminate and not recursively compile forever. Runtime recursion follows normal call semantics.

- [ ] **T16.6 Forced-GC test.**

Trigger GC between plan creation and nested default execution; detached methods and environment registry entries remain valid.

- [ ] **T16.7 G5 verification.**

Acceptance: RT-03 through RT-09, RT-15, RT-16 plus LW origin coverage.

- [ ] **T16.8 Update checkpoint and commit.**

```bash
git add phalcom-core docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
git commit -m "lang005: execute all C4 requirement targets"
```

---

## 40. T17 — Trait-Evidenced Bound Callable References

**Purpose:** make `&object.method(_)` freeze the same exact trait selection/evidence as the equivalent direct call.

**Files:**

```text
Modify:
  phalcom-semantic/src/checker/associated.rs
  phalcom-semantic/src/checker/analysis.rs if reference product is extended
  phalcom-core/src/modules/semantic_lowering.rs
  phalcom-core/src/compiler/lib/associated.rs
  phalcom-core/src/bytecode.rs
  phalcom-core/src/heap/object.rs
  phalcom-core/src/vm/dispatch.rs / callable invocation path
  GC tracing
Test:
  semantic callable-reference tests
  phalcom-core/tests/core/language/traits.rs
  existing associated/bound-family regressions
```

### Interfaces

**Consumes:** exact `TraitDispatchSelection`, detached method, runtime conformance env.

**Produces:** exact trait-bound callable object/reference with frozen environment.

### Steps

- [ ] **T17.1 Add semantic bound-reference test.**

For a selector available only through trait evidence, assert the callable-reference semantic product retains `TraitDispatchSelection` rather than becoming a live ordinary family with no trait context.

- [ ] **T17.2 Add trait-specific lowering variant.**

Conceptually:

```rust
CallableReferenceLoweringSpec::MakeTraitBoundMethod {
    plan: ...,
    requirement: ...,
}
```

- [ ] **T17.3 Prefer a dedicated runtime object rather than enlarging every ordinary `BoundMethodObject`.**

Conceptually:

```rust
pub struct TraitBoundMethodObject {
    pub method: ObjRef,
    pub receiver: Value,
    pub conformance_environment: RuntimeConformanceEnvironmentId,
}
```

If a preexisting generalized bound-callable wrapper is clearly better, use it; ordinary bound-method semantics/size must remain unchanged unless explicitly justified.

- [ ] **T17.4 Implement creation/invocation.**

Capture exact selected method/requirement/env at reference creation. Invocation stamps the captured conformance env; it never repeats semantic trait selection.

- [ ] **T17.5 Test conformance witness reference.**

- [ ] **T17.6 Test trait default reference.**

- [ ] **T17.7 Test captured trait default that calls an abstract requirement.**

This proves the captured env survives first-class callable invocation.

- [ ] **T17.8 Forced-GC test and ordinary-bound-reference regression.**

Acceptance: BR-01 through BR-07.

- [ ] **T17.9 Commit.**

```bash
git add phalcom-semantic phalcom-core
git commit -m "lang005: preserve conformance evidence in bound references"
```

---

## 41. T18 — Generic, Exact-Case, and Runtime-Environment Integration

**Purpose:** prove trait execution composes with exact generic reification and exact enum-case runtime semantics.

**Files:**

```text
Modify as defects require:
  phalcom-core/src/modules/semantic_lowering.rs
  phalcom-core/src/typing/conformance_environment.rs
  phalcom-core/src/vm/dispatch.rs
  compiler trait-dispatch path
Test:
  phalcom-core/tests/core/language/traits.rs
  generic/runtime type environment regressions
  exact enum-case regressions
```

### Interfaces

**Consumes:** `RuntimeTypeEnvironmentId` + `RuntimeConformanceEnvironmentId` composition.

**Produces:** correct exact generic/exact-case execution for all trait paths.

### Steps

- [ ] **T18.1 Add mandatory specialized `Value` executable test.**

```phalcom
trait Tagged { tag -> String }
class Value<T> {}
impl Tagged for Value<String> { tag -> String { "string" } }
impl Tagged for Value<Int>    { tag -> String { "int" } }

Assert.equal(Value<String>.new().tag, "string")
Assert.equal(Value<Int>.new().tag, "int")
```

Adapt constructor syntax minimally to the actual class model.

- [ ] **T18.2 Add generic source conformance runtime test.**

One `ImplId` applies to multiple exact target forms; runtime conformance envs differ only where exact type substitution requires it.

- [ ] **T18.3 Verify type environment is not replaced by trait env.**

Trait-selected/default calls use the exact `RuntimeTypeEnvironmentId` needed by generic runtime recipes.

- [ ] **T18.4 Add exact enum-case conformance execution.**

Only the exact case refined/constructed at the call site gets the trait behavior; root/sibling remain non-conforming.

- [ ] **T18.5 Add exact generic bound-reference capture.**

Reference from `Value<Int>` retains the Int-specific environment even if another `Value<String>` conformance exists.

- [ ] **T18.6 G6 verification.**

Run focused trait, generic reification, enum exact-case, associated bound-reference, and inherent regression suites.

Acceptance: RT-10 through RT-12, BR-04, VA-01, VA-02, VA-10.

- [ ] **T18.7 Commit.**

```bash
git add phalcom-core
git commit -m "lang005: specialize trait execution across runtime type environments"
```

---

## 42. T19 — Editor/Source Projection and Incremental Certification

**Purpose:** make tooling and incremental analysis consume the same canonical trait selection as the compiler.

**Files:**

```text
Modify:
  phalcom-semantic/src/editor.rs
  phalcom-semantic/src/source_index/*
  phalcom-semantic/src/presentation.rs if needed
  phalcom-lsp adapters only where semantic products require projection
  phalcom-semantic/src/session.rs/db dependencies as defects require
Test:
  semantic editor/source-index integration tests
  incremental trait dispatch tests
```

### Interfaces

**Consumes:** `ExpressionAnalysis::trait_dispatch`, requirement/witness/default source IDs.

**Produces:** navigation/hover/completion/diagnostics consistent with compiler semantic selection.

### Steps

- [ ] **T19.1 Add go-to-definition/source target tests for each selection origin.**

```text
inherent witness      → inherent callable source
conformance witness   → conformance member source
trait default         → trait default source
data component        → component source
```

- [ ] **T19.2 Add ambiguous selector tooling test.**

Diagnostics/related targets enumerate competing trait requirements/defaults deterministically.

- [ ] **T19.3 Make editor queries read `TraitDispatchSelection`; do not reconstruct from syntax or scan conformances.**

- [ ] **T19.4 Add completion/hover trait-evidenced behavior if existing editor surface API supports ordinary member enumeration.**

Candidate discovery should reuse `TraitDispatchIndex`, not build another trait index in LSP.

- [ ] **T19.5 Run full IE incremental matrix.**

Compare cold vs incremental semantic call-site products and diagnostics after each edit.

- [ ] **T19.6 Verify executable refresh where harness supports incremental compilation.**

Selection-changing edits must not leave stale executable plans/method bodies.

- [ ] **T19.7 Commit.**

```bash
git add phalcom-semantic phalcom-lsp
git commit -m "lang005: project trait dispatch into tooling and incrementality"
```

---

## 43. T20 — Vertical `Iterable` / `Iterator` / Generic / Ambiguity Corpus

**Purpose:** prove the architecture works as a language feature, not merely as isolated semantic/runtime mechanisms.

**Files:**

```text
Create/modify:
  phalcom-core/tests/core/language/traits.rs
  test fixture modules/files according to existing test harness conventions
```

### Interfaces

**Consumes:** all P3 semantic/lowering/runtime APIs.

**Produces:** executable high-density C4 acceptance evidence.

### Steps

- [ ] **T20.1 Implement the C4-safe generic `Iterable<Item, Cursor>` trait fixture.**

Do not use associated types. Include defaults:

```text
each
count
contains
toList
```

and any minimal helpers required to express them with current core APIs.

- [ ] **T20.2 Implement `RangeView`/`Countdown` mixed conformance.**

Required evidence map:

```text
iterate        → existing inherent member
iteratorValue  → conformance-local witness
each           → trait default
count          → trait default
contains       → trait default
toList         → trait default
```

- [ ] **T20.3 Execute assertions for `count`, `contains`, `each`, and `toList`.**

`toList` must exercise default→default→abstract requirements.

- [ ] **T20.4 Inspect semantic evidence alongside runtime output.**

The mixed-origin selection map is asserted explicitly.

- [ ] **T20.5 Implement stateful `Iterator<Item>` + `CountdownIterator`.**

Defaults:

```text
nextOr
countRemaining
drain
```

Use two independent mutable iterator instances and assert interleaved consumption.

- [ ] **T20.6 Add shared-witness valid program.**

`Named` + `DisplayNamed` share `Person.name`; `displayName` default calls `self.name` and executes.

- [ ] **T20.7 Add competing-default invalid program and resolved companion.**

`Pretty.render` + `Debuggable.render` defaults are ambiguous for `Item`; adding concrete inherent `render` on `ResolvedItem` makes the call unambiguous.

- [ ] **T20.8 Add bound default reference inside the vertical corpus.**

Capture a default whose body reaches an abstract requirement and execute after unrelated calls/GC.

- [ ] **T20.9 Forced-GC vertical run.**

Trigger GC at meaningful boundaries if the test harness exposes it.

Acceptance: VA-03 through VA-11 and RT/BR coverage not already proven.

- [ ] **T20.10 Commit.**

```bash
git add phalcom-core/tests
git commit -m "test: certify C4 vertical trait execution"
```

---

## 44. T21 — C4 Stress, Broad Certification, Walkthrough, and Handoff

**Purpose:** certify the entire checkpoint and leave durable takeover documentation for C5/C6.

**Files:**

```text
Create/modify:
  phalcom-core/tests/core/language/traits.rs or stress fixture files
  docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-walkthrough.md
  docs/implementation/LANG005/LANG005.C4/LANG005.C4.P3-handoff.md
  docs/implementation/LANG005/LANG005.C4/LANG005.C4-CHECKPOINT.md
```

### Interfaces

**Consumes:** completed P3.

**Produces:** C4 COMPLETE checkpoint, evidence table, C5/C6 stable-extension handoff.

### Steps

- [ ] **T21.1 Add one substantial valid C4 stress program.**

Target shape:

```text
6–8 traits
8–12 target types
15–20 conformances
```

Must combine:

```text
explicit conformance witnesses
existing inherent witnesses
data-component witnesses
inherited witnesses
C2 specialized/conditional witnesses
trait defaults
default→requirement
default→default
shared concrete witnesses
distinct generic TraitRefs
specialized generic targets
exact enum-case conformance
bound references
```

Compute deterministic values and assert them so execution failures cannot hide behind “program ran.”

- [ ] **T21.2 Add/complete invalid stress corpus.**

Each isolated invalid case asserts its precise diagnostic family:

```text
third-party ownership
duplicate exact conformance
generic overlap
generic+exact overlap
missing witness
incompatible witness
access mismatch
property mutability mismatch
extra conformance-local member
duplicate explicit witness
incompatible inherent selector vs explicit witness
competing defaults
unsupported associated binding (C5)
unsupported trait constraint/conditional conformance (C6)
unsupported metatype conformance
```

- [ ] **T21.3 Add a negative runtime-authority proof.**

Inspect target runtime class dictionaries and assert trait defaults/conformance-local witnesses are absent. Then prove the ordinary trait call still executes through `InvokeTraitSelected`/plan machinery.

- [ ] **T21.4 Run C4 certification commands.**

Use actual test module filters if repository module paths differ, but certify at least:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration impl_syntax
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic capabilities::traits
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic impls
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::inherent_impl
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::traits
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace
cargo fmt --all -- --check
git diff --check
```

Do not waive workspace failures introduced by C4.

- [ ] **T21.5 Write `LANG005.C4.P3-walkthrough.md`.**

It must document landed symbols and demonstrate:

```text
ordinary trait-evidenced lookup
convergence/ambiguity
per-expression evidence
lowering projection
runtime executable plans/environments
conformance/default detached compilation
default→requirement dispatch
bound references
generic/exact-case behavior
GC rooting
complex executable fixtures
proof of no target method-table injection
incremental/editor behavior
```

- [ ] **T21.6 Write `LANG005.C4.P3-handoff.md`.**

Name stable extension points for C5/C6:

```text
how C5 may extend ConformanceEvidence with associated bindings
how executable conformance plans may later carry associated runtime metadata if needed
how C6 may add nested evidence/conditional applicability without changing C4 witness identity
what P3 APIs must not be reopened
```

- [ ] **T21.7 Close C4 checkpoint.**

Set:

```text
P1 COMPLETE / IMPLEMENTED / CERTIFIED as applicable
P2 COMPLETE / IMPLEMENTED / CERTIFIED as applicable
P3 COMPLETE / IMPLEMENTED / CERTIFIED
C4 COMPLETE
next checkpoint: C5 associated types
```

Record exact commit/test evidence and any bounded deferred debt that does not invalidate C4.

- [ ] **T21.8 Final self-review against §15, §21, §46.**

Every invariant/coverage/completion item must map to landed code + test evidence.

- [ ] **T21.9 Commit.**

```bash
git add docs/implementation/LANG005/LANG005.C4 phalcom-core/tests phalcom-semantic/tests
git commit -m "lang005: complete C4 trait-evidenced execution"
```

---

## 45. Canonical Test Fixtures

These fixtures are requirements, not illustrative decoration. Adapt only syntactic details that differ in the live language.

### 45.1 Exact specialized generic target

```phalcom
trait Tagged {
  tag -> String
}

class Value<T> {}

impl Tagged for Value<String> {
  tag -> String { "string" }
}

impl Tagged for Value<Int> {
  tag -> String { "int" }
}

Assert.equal(Value<String>.new().tag, "string")
Assert.equal(Value<Int>.new().tag, "int")
```

Semantic assertions:

```text
different exact target TypeIds
different exact ConformanceEvidence/source ImplIds
same nominal Value declaration
no target surface pollution
```

### 45.2 Default→conformance witness

```phalcom
trait Scalable<T> {
  scale(_ value: T) -> T

  twice(_ value: T) -> T {
    self.scale(self.scale(value))
  }
}

class Doubler {}

impl Scalable<Int> for Doubler {
  scale(_ value: Int) -> Int { value * 2 }
}

Assert.equal(Doubler.new().twice(3), 12)
```

Required execution trace:

```text
ordinary twice lookup
→ trait default selected
→ InvokeTraitSelected
→ default frame with conformance env
→ InvokeTraitRequirement(scale)
→ conformance-owned scale method
→ 12
```

### 45.3 Concrete witness beats default

```phalcom
trait Describable {
  describe -> String { "default" }
}

class User {
  describe -> String { "user" }
}

class Empty {}

impl Describable for User {}
impl Describable for Empty {}

Assert.equal(User.new().describe, "user")
Assert.equal(Empty.new().describe, "default")
```

### 45.4 Data component through default

```phalcom
trait Named {
  name -> String

  displayName -> String {
    "<\(self.name)>"
  }
}

data Person(name: String)
impl Named for Person {}
```

Assert evidence selects `DataComponentId` and `displayName` executes through the data target.

### 45.5 Shared concrete witness

```phalcom
trait Named {
  name -> String
}

trait DisplayNamed {
  name -> String
  displayName -> String { "<\(self.name)>" }
}

class Person {
  name -> String { "Ada" }
}

impl Named for Person {}
impl DisplayNamed for Person {}

Assert.equal(Person.new().name, "Ada")
Assert.equal(Person.new().displayName, "<Ada>")
```

Both requirement IDs remain distinct and select the same `Person.name` callable.

### 45.6 Competing defaults

```phalcom
trait Pretty {
  render -> String { "pretty" }
}

trait Debuggable {
  render -> String { "debug" }
}

class Item {}
impl Pretty for Item {}
impl Debuggable for Item {}

// Item.new().render → compile-time trait ambiguity
```

Resolved companion:

```phalcom
class ResolvedItem {
  render -> String { "item" }
}
impl Pretty for ResolvedItem {}
impl Debuggable for ResolvedItem {}
Assert.equal(ResolvedItem.new().render, "item")
```

### 45.7 Exact enum case

Use repository-valid exact case target syntax to prove:

```text
Result<Int>::Ok(...) + HasValue<Int> → executable
Result<Int>::Error(...)              → no evidence
Result<Int> root                     → no inherited case conformance
```

### 45.8 Bound trait default

```phalcom
trait Transformer {
  transform(_ value: Int) -> Int
  twice(_ value: Int) -> Int { self.transform(self.transform(value)) }
}

class PlusOne {}
impl Transformer for PlusOne {
  transform(_ value: Int) -> Int { value + 1 }
}

const f = &PlusOne.new().twice(_)
Assert.equal(f.call(10), 12)
```

The captured callable must retain exact conformance context without runtime re-resolution.

---

## 46. Completion Criteria

### 46.1 Entry/P2 correctness

P3 cannot complete unless:

```text
[ ] whole workspace compiles with CallableOwnerId::Conformance
[ ] no general conformance path depends on panicking declaration ownership
[ ] incomplete conformances diagnose statically
[ ] exact conformance resolution preserves every proof state
[ ] definite failure dominates unrelated uncertainty
[ ] generic inherent witnesses specialize correctly
[ ] explicit conformance witness cannot hide incompatible inherent selector
[ ] access coverage is semantically correct
[ ] data witnesses retain DataComponentId identity
[ ] exact conditional concrete-witness/default precedence is correct
```

### 46.2 Semantic ordinary lookup

```text
[ ] receiver+selector finds relevant trait conformances without caller TraitRef
[ ] candidate discovery is bounded/indexed
[ ] exact target and exact TraitRef identities are retained
[ ] only proven evidence yields usable behavior
[ ] shared concrete witnesses converge
[ ] competing defaults diagnose ambiguity
[ ] distinct conformance witnesses remain ambiguous
[ ] inherent behavior remains ordinary authority
[ ] parent conformance does not propagate
[ ] exact enum-case conformance does not leak
```

### 46.3 Per-expression authority

```text
[ ] ordinary trait calls publish TraitDispatchSelection
[ ] C3 abstract default requirement sites publish TraitRequirementId identity
[ ] fingerprints include semantic trait dispatch
[ ] stale re-analysis selection is cleared
[ ] compiler lowering consumes the published product
```

### 46.4 Lowering/runtime

```text
[ ] exact evidence projects to deterministic ExecutableConformancePlan
[ ] conformance-local witnesses compile detached
[ ] trait defaults compile detached
[ ] target runtime class dictionaries remain unmodified by conformance
[ ] trait-selected ordinary call executes
[ ] default→requirement executes selected target
[ ] default→default preserves exact conformance context
[ ] inherent/data/C2 conditional/conformance/default requirement targets all execute
[ ] runtime type environment composes with conformance environment
[ ] missing active requirement environment fails closed internally
[ ] GC roots all detached methods/plans/references
[ ] VM never searches classes/traits to prove conformance
```

### 46.5 Callable references

```text
[ ] conformance witness bound reference executes
[ ] trait default bound reference executes
[ ] captured default can call abstract requirements
[ ] exact generic environment remains frozen
[ ] ordinary bound method/family behavior regresses cleanly
```

### 46.6 Integration/certification

```text
[ ] Value<String>/Value<Int> specialization executes distinctly
[ ] generic source conformance executes for multiple exact targets
[ ] Scalable default→requirement fixture returns 12
[ ] mixed Iterable fixture passes and evidence origins are asserted
[ ] stateful Iterator fixture passes with independent instances
[ ] shared witness valid case passes
[ ] competing defaults invalid case diagnoses precisely
[ ] exact enum-case execution is case-specific
[ ] complex valid C4 stress program passes
[ ] complex invalid C4 corpus diagnoses expected families
[ ] cold/incremental trait dispatch agrees
[ ] editor/source navigation uses canonical semantic selection
[ ] full workspace tests/checks/format/diff checks pass
[ ] P3 walkthrough and C5/C6 handoff are complete
[ ] C4 checkpoint is marked COMPLETE only after all above evidence exists
```

---

## 47. Self-Review / Specification Coverage

Before implementation begins, this plan has been checked against the C4 checkpoint/guidance and post-P2 audit as follows.

| Requirement | Owning tasks |
|---|---|
| P2 proof-state correctness | T2, T4 |
| P2 invalid conformance diagnostics | T2 |
| callable-owner generalization | T1 |
| generic witness/access correctness | T3 |
| concrete witness > default for exact conditional applicability | T4 |
| ordinary trait-evidenced discovery | T5–T6 |
| convergence / competing default ambiguity | T7 |
| canonical ordinary dispatch integration | T8 |
| per-expression evidence | T9 |
| incremental dependency correctness | T10, T19 |
| executable evidence projection | T11 |
| detached witness/default compilation | T12 |
| internal runtime evidence carrier | T13 |
| ordinary trait call execution | T14 |
| default→abstract requirement dispatch | T15 |
| all witness origins/default→default | T16 |
| bound callable references | T17 |
| generic/exact-case/runtime environment | T18 |
| editor/source tooling | T19 |
| realistic Iterable/Iterator execution | T20 |
| stress/adversarial certification | T21 |
| no runtime semantic authority | T11–T16, T21 negative proof |
| C5/C6 boundary | global constraints, T21 handoff |

Type consistency check:

```text
semantic:
    TraitDispatchContribution
    TraitDispatchIndex
    TraitEvidencedMemberCandidate
    TraitDispatchSelection
    TraitDispatchSite

lowering:
    ExecutableConformancePlan
    ExecutableRequirementTarget

runtime:
    RuntimeConformancePlanId
    RuntimeConformanceEnvironmentId
    RuntimeConformanceEnvironment
```

Mechanical names may be adapted, but later task interfaces must be updated coherently if any name changes during implementation.

No implementation step may be replaced by a vague “handle edge cases” action. Every named semantic edge appears in §21 coverage and a concrete task/test.

---

## 48. Final C4 Acceptance Trace

The strongest final trace should be the mixed `Iterable<Item, Cursor>` program:

```text
RangeView
    ↓
exact Iterable<Int, Int> conformance
    ↓
ConformanceEvidence
    iterate
        → InherentCallable(RangeView.iterate)
    iteratorValue
        → ConformanceCallable(ImplId::...)
    each
        → TraitDefault(Iterable.each)
    count
        → TraitDefault(Iterable.count)
    contains
        → TraitDefault(Iterable.contains)
    toList
        → TraitDefault(Iterable.toList)

source:
    view.toList
        ↓
canonical ordinary inherent lookup: Missing
        ↓
TraitDispatchIndex(receiver family, toList, instance)
        ↓
exact target match
        ↓
exact TraitRef = Iterable<Int, Int>
        ↓
Proven ConformanceEvidence
        ↓
TraitDispatchSelection(toList requirement)
        ↓
ExpressionAnalysis.trait_dispatch
        ↓
ExecutableConformancePlan
        ↓
InvokeTraitSelected(toList slot)
        ↓
trait default toList frame
    conformance_environment = E
        ↓
InvokeTraitRequirement(each slot)
    same E
        ↓
trait default each
        ↓
InvokeTraitRequirement(iterate slot)
    → existing inherent RangeView.iterate
        ↓
InvokeTraitRequirement(iteratorValue slot)
    → detached conformance-local method
        ↓
List<Int> result
```

Then the architectural negative assertions must prove:

```text
RangeView DeclarationSurface does not contain Iterable.each/toList/defaults
RangeView runtime class method table does not contain copied Iterable defaults
RangeView runtime class method table does not contain the conformance-local iteratorValue witness unless independently inherent
no runtime conformance/class scan occurred
```

If the result is correct but any of those negative assertions fail, C4 is not architecturally complete.

---

## 49. C4 Closure Statement

When G7 passes, C4 has established the following stable layer for C5/C6:

```text
explicit nominal conformance source
+ exact target/TraitRef identity
+ global coherence
+ complete requirement witness/default proof
+ structured exact evidence
+ ordinary trait-evidenced dispatch
+ deterministic ambiguity/convergence
+ semantic call/reference selection
+ detached executable witness/default representation
+ active conformance runtime environment
+ default→requirement execution
+ exact generic/case integration
+ incremental/tooling projection
```

C5 may extend `ConformanceEvidence` with associated type bindings/projections without changing C4 witness identity or ordinary dispatch semantics.

C6 may extend conformance applicability with trait constraints/nested evidence without changing the rule that only proven exact evidence authorizes P3 trait behavior.

The implementation must preserve the core sentence throughout:

> A conformance is a structured semantic proof. P3 makes that proof executable; it does not replace the proof with runtime method discovery.
