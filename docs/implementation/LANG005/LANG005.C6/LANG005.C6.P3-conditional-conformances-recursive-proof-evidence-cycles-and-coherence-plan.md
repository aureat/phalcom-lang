---
id: LANG005.C6.P3
category: LANG
program: LANG005
checkpoint: LANG005.C6
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG005.C6.P2
follows: LANG005.C6.P2
supersedes: null
prepared: 2026-09-16
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: fc6325ad6e86a8f4a52c1ce3747c52dce4dab9b0
---

# LANG005.C6.P3 — Conditional Conformances, Recursive Proof Evidence, Cycles/Memoization, and Conditional Coherence

> **For Luna / agentic implementers:** Execute this plan task-by-task under the repository's Luna patch-grade workflow. Re-read the live repository before editing, adapt mechanical drift locally, and STOP AND CONSULT rather than changing any FIXED architecture below. P3 begins with a deliberately bounded P2 corrective-closure gate because P2's implementation records contain several verification/record gaps that recursive proof must not inherit ambiguously. That prerequisite is not authorization to reopen P2's architecture.

**Goal:** Extend Phalcom's P1/P2 generic-evidence architecture so a source conformance `impl` may carry a canonical generic contract, exact and generic conformance goals may recursively discharge that contract through one proof context, recursive proof constructs complete executable evidence rather than a Boolean result, cycles/memoization/budgets/cancellation terminate soundly, current C4 head-based coherence remains unchanged, and compiler/runtime layers execute already-proven derived evidence without runtime conformance search.

**Architecture:** Preserve `GenericSignature` as the generic proposition authority, `GenericEvidenceEnvironment` as the lexical evidence authority, `ConformanceIndex` as the conformance candidate/head authority, `ConformanceEvidence` as the exact source-conformance basis, `TypeData::AssociatedProjection` plus `types::normalize_type` as the associated projection authority, and `RuntimeConformanceEnvironment` as the behavioral runtime carrier. P3 adds one recursive proof session/context that evaluates canonical `Conforms`, `Equivalent`, and `Subtype` obligations, shares recursion state with projection normalization, memoizes canonical goals, constructs recursively derived conformance evidence containing the nested evidence required by conditional witnesses/defaults, and lowers that finite proof DAG into existing type/conformance runtime environments.

**Tech stack:** Rust workspace (`phalcom-ast`, `phalcom-semantic`, `phalcom-type-meta`, `phalcom-core`, `phalcom-lsp`), semantic DB/incremental query products, generic evidence/proof, C4/C5 conformance and associated-type architecture, P2 projection/equality/inference architecture, stable metadata, compiler call-environment recipes, VM behavioral evidence execution, Phalcom method/conformance fixtures, Cargo integration tests.

**Primary specification:** `docs/specs/generic-constraints.md`

**Principal predecessor records:**
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md`
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md`
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-generic-associated-projections-refinements-equality-and-inference-plan.md`
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-walkthrough.md`
- `docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-handoff.md`

**Planning baseline commit:**

```text
fc6325ad6e86a8f4a52c1ce3747c52dce4dab9b0
feat(lang): implement LANG005.C6.P2 generic associated projections, refinements, equality, and inference
```

The P2 walkthrough/handoff themselves still record the predecessor P1 revision `af2d0aec...`; this is an implementation-record defect, not the live P3 planning baseline. T0 owns truthful correction/amendment.

---

# 0. Executor contract

This plan is designed for GPT-5.6 Luna High/Extra High or an equivalently capable implementation model working under constrained architectural authority.

The implementer:

- may adapt helper names, private decomposition, imports, equivalent local APIs, and mechanical path drift to the live repository;
- must preserve the semantic ownership, evidence identity, recursive proof, projection normalization, coherence, metadata, incremental, lowering, runtime, and lifecycle invariants below;
- must not improvise material architecture;
- must not add a second generic constraint/evidence authority beside `GenericSignature` / `GenericEvidenceEnvironment`;
- must not add a second conformance head matcher beside `ConformanceIndex`;
- must not add a second projection normalizer beside `types::normalize_type`;
- must not turn conditional proof into runtime lookup/search;
- must not treat a proof cycle as evidence;
- must not introduce condition-based overlap specialization or "most specific wins";
- must not introduce blanket generic-target conformances merely to support later Iterator/Iterable work;
- must not collapse recursive, blocked, dynamic, cancelled, budgeted, ambiguous, or internal proof states into `Missing` or success;
- must preserve Phalcom's method-based language model; there are no source-level free functions;
- must follow the BUILD / STABILIZE / CERTIFY test budget rather than testing reflexively;
- must classify and defer unrelated failures;
- must preserve unrelated modified/staged/untracked work;
- must keep `LANG005.C6-CHECKPOINT.md` current at durable gates only;
- must update persistent C6 guidance only where the current lifecycle/current-action text is stale or a genuinely persistent C6 rule changes;
- must produce the required P3 walkthrough and P3-to-P4 handoff;
- must STOP AND CONSULT whenever any global or task-local trigger fires;
- must not commit or push unless the surrounding workflow/user instruction explicitly authorizes it.

Testing is evidence gathering. Run a test because it discriminates an invariant, validates a durable interface, or satisfies a named gate. Do not run broad suites merely because an edit occurred.

---

# 1. Goal

Implement one complete conditional conformance proof-and-execution vertical:

```phalcom
trait Printable {
    print -> String
}

class Box<T> {
    // ordinary class body
}

impl<T> Printable for Box<T>
    where T |= Printable
{
    print -> String {
        // legal method/conformance-witness body
        // may use behavior justified by T |= Printable
        ...
    }
}
```

such that a goal:

```text
Box<User> |= Printable
```

is processed as:

```text
canonical conformance goal
    ↓
C4/P3 head match
    ↓
T = User
    ↓
instantiate canonical impl GenericSignature
    ↓
prove User |= Printable in one recursive proof session
    ↓
construct outer conditional conformance evidence
    containing the nested evidence required by the witness/default
    ↓
lower finite evidence DAG
    ↓
RuntimeTypeEnvironment + RuntimeConformanceEnvironment
    ↓
execute already-selected witness/default with no runtime proof/search
```

P3 must also handle associated projection/equality/subtype conditions, cycles, memoization, cancellation/budget terminal states, cross-module publication, incremental invalidation, and current global no-overlap coherence.

---

# 2. Checkpoint acceptance objective

`LANG005.C6` owns the generic proof-bearing constraint/evidence model required before Iterator/Iterable law design and production iteration migration.

P3 is **recursive conditional-proof closure**, not C6 closure.

The bounded P3 acceptance sentence is:

> A legal source conformance may carry a canonical `GenericSignature`; exact or generic conformance goals select the existing coherent source head, derive the impl-parameter substitution, instantiate every condition, and prove those nested `Conforms` / `Equivalent` / `Subtype` obligations through one recursive proof context that shares projection-normalization state, memoizes canonical goals, terminates cycles without turning them into proof, preserves cancellation/budget/blocked/dynamic/error states, and produces a recursively complete conformance evidence graph. Compiler lowering materializes that graph into existing runtime type/conformance environments so condition-dependent witnesses/defaults execute with the evidence they were checked under, while the VM performs zero conformance/projection proof. Cross-module and incremental consumers observe the same semantics, and existing head-based global coherence remains unchanged regardless of conditional `where` clauses.

P3 advances C6 as:

```text
C6.P1
    direct generic conformance/evidence
        ↓
C6.P2
    generic associated projection/equality/inference
        ↓
C6.P3
    conditional source conformances
    nested obligation proof
    recursive evidence
    cycles / memoization / budgets
    conditional coherence
        ↓
C6.P4
    Iterator / Iterable generic laws
    blanket/derived implication design
        ↓
C6.P5
    production iteration migration
        ↓
C6.P6
    remaining closure / stress / certification
```

P3 does not implement P4's generic trait implication mechanism.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: fc6325ad6e86a8f4a52c1ce3747c52dce4dab9b0
revision subject:
    feat(lang): implement LANG005.C6.P2 generic associated projections,
    refinements, equality, and inference

relevant predecessor: LANG005.C6.P2
predecessor checkpoint classification:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED
checkpoint:
    LANG005.C6 IN_PROGRESS / PARTIAL / FOCUSED_TESTED
```

## 3.1 Verified planning facts

The architect verified or inherited the following post-P2 facts:

1. `GenericConstraint` already has canonical `Subtype`, `Equivalent`, and `Conforms`.
2. `GenericEvidenceEnvironment` owns direct conformance assumptions, equivalence/refinement information, and deterministic behavioral evidence layout.
3. `GenericEvidenceEnvironment::{try_from_constraints, try_compose}` use checked layout construction.
4. `GenericConformanceEvidence` currently distinguishes exact `ConformanceEvidence` from lexical assumed evidence.
5. `query_generic_conformance_proof(...)` is currently a direct/stateless proof entry point: lexical assumption first, exact conformance resolution second.
6. `GenericConformanceProof` already preserves rich terminal states including `Missing`, `Incomplete`, `CoherenceConflict`, `Unknown`, `Blocked`, `Dynamic`, `Cancelled`, `BudgetExceeded`, and `InternalFailure`.
7. `GenericEquivalenceProof` already includes a recursive outcome and P2 equality/refinement behavior.
8. C4/C5 conformance products already retain `generic_signature` fields on conformance head/contribution/witness products; P3 should activate those rather than inventing a parallel conditional declaration product.
9. conformance `impl ... where ...` remains deliberately blocked by `ImplWhereClauseUnsupported` / equivalent live diagnostic until generic trait constraints are implemented.
10. C4 head matching already derives an `impl_bindings` substitution from source conformance head to exact goal.
11. P2 has canonical associated specialization as `Conforms + Equivalent`.
12. P2 projection normalization remains `types::normalize_type` and can consult generic conformance/equality evidence.
13. P2 inference structurally represents `AssociatedProjection` and uses a bounded 16-iteration fixed-point loop with one-way inference.
14. P2 stable metadata contains `TypeNode::AssociatedProjection` and stable associated requirement identity.
15. runtime associated projection lookup/proof remains absent.
16. runtime conformance evidence is still behavioral method-handle transport through `RuntimeConformanceEnvironment`.
17. call frames preserve separate type and conformance environment IDs.
18. P2 commit `fc6325ad...` contains the P2 implementation and records, but the generated P2 walkthrough/handoff front matter still cites the older P1 revision.
19. the recorded P2 core gate explicitly skipped exact enum-case coverage and reports two ignored owner/receiver evidence tests.
20. the P2 plan required a callable-reference audit outcome, CV-01..CV-64 map, cross-module associated execution, recomputation metrics, and terminal-state evidence that are not all present in the P2 walkthrough/handoff.
21. C6 checkpoint already marks P3 active/proposed; C6 guidance contains persistent P3 rules but still has stale lifecycle/current-action material from P1.
22. Phalcom has no source-level free functions. All executable fixtures must use instance/class-side methods, trait methods, or conformance witnesses.

> **Re-read every named live path before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.**

---

# 4. Required reads before implementation

## 4.1 Workflow authority

Read:

```text
AGENTS.md
docs/implementation/README.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/luna-implementer-prompt.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
docs/workflow/sol-principal-architect-project-doctrine.md
```

Use the repository-current names if mechanically relocated.

## 4.2 C6 state

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6-CHECKPOINT.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6-GUIDANCE.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-generic-associated-projections-refinements-equality-and-inference-plan.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-walkthrough.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P2-handoff.md
```

Read P1 walkthrough/handoff only if a P3 issue requires direct generic evidence mechanics not summarized by P2.

## 4.3 Predecessor trait/associated architecture

Read only relevant current records:

```text
LANG005.C4 checkpoint/walkthrough/handoff for exact conformance/coherence
LANG005.C5 checkpoint/walkthrough/handoff for associated projections/evidence
```

Do not broadly re-audit C4/C5.

## 4.4 Normative language authority

At minimum:

```text
docs/specs/README.md
docs/specs/generic-constraints.md
authoritative trait/conformance specification
authoritative associated-type specification
authoritative generic/type-formation specification
authoritative kind/subtyping/equivalence specifications
authoritative callable/dispatch specification
authoritative exact enum-case specification
```

If normative coherence language conflicts with the current checkpoint rule that overlapping conditional heads remain rejected, STOP AND CONSULT before production edits.

## 4.5 Production architecture

Re-read live equivalents of:

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/source_index/builder.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/compile.rs
phalcom-core/src/typing/environment.rs
phalcom-core/src/typing/conformance_environment.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/vm/send.rs
phalcom-core/src/frame.rs

phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/validate.rs
```

## 4.6 Test conventions

Rediscover current names with `-- --list` and `rg`. Likely relevant modules:

```text
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs
phalcom-semantic/tests/semantic/impls/queries.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs
phalcom-core/tests/core/language/generic_conformance.rs
phalcom-core/tests/core/language/associated_traits.rs
```

P3 should create dedicated modules rather than overloading predecessor files:

```text
phalcom-semantic/tests/semantic/capabilities/conditional_conformance.rs
phalcom-core/tests/core/language/conditional_conformance.rs
```

if the live test organization supports that mechanically.

---

# 5. Normative authority

## 5.1 Generic propositions

Canonical relations remain:

```text
:     kind classification
|=    trait conformance/evidence
<:    subtype
==    canonical semantic type equivalence
```

P3 conditions are ordinary `GenericSignature` propositions. There is no new "conditional impl relation."

## 5.2 Conditional conformance

Normative form:

```phalcom
impl<T> Printable for Box<T>
    where T |= Printable
{
    ...
}
```

means:

```text
the source conformance head is Box<T> |= Printable

applicability requires proving its GenericSignature after
head substitution
```

The `where` clause does not alter trait identity or conformance head identity.

## 5.3 Associated specialization

P2 remains authoritative:

```phalcom
where I |= Iterator<Item = T>
```

means:

```text
Conforms(I, Iterator)
Equivalent(<I as Iterator>::Item, T)
```

P3 proves those canonical obligations. It does not create a special associated conditional relation.

## 5.4 Conditional overlap

Unless an authoritative trait/coherence specification explicitly says otherwise:

> `where` conditions do not partition global coherence domains. Structurally overlapping conformance heads remain rejected. There is no condition-based specialization and no "most specific wins."

This is FIXED.

## 5.5 Recursive cycles

A recursive obligation cycle is not proof by itself.

```text
A → A
```

or:

```text
A → B → A
```

must terminate in a recursive/blocked terminal state, not success.

## 5.6 Method-based language model

There are no source-level free functions.

All executable tests use:

```text
instance methods
class-side methods
trait methods
conformance witness/default methods
```

Internal Rust abstractions named `Callable*` remain implementation-general and need not be renamed.

## 5.7 Runtime boundary

The compiler/VM executes already-selected evidence. It does not:

```text
interpret GenericConstraint
search source conformances
decide candidate coherence
solve nested obligations
normalize associated projections
run recursive proof
```

---

# 6. Takeover state

## 6.1 Stable interface map

| Concept | Current owner/symbol | P3 invariant |
|---|---|---|
| generic proposition | `GenericConstraint` | P3 conditions remain ordinary constraints |
| generic contract | `GenericSignature` | one generic contract authority |
| lexical generic evidence | `GenericEvidenceEnvironment` | one lexical evidence authority |
| direct conformance identity | `GenericConformanceKey` | canonical subject + TraitRef |
| direct proof result | `GenericConformanceProof` | extend/contextualize, do not replace |
| direct proof source | `GenericConformanceEvidence` | preserve assumed vs exact distinction |
| equality/refinement | `GenericEquivalenceProof`, evidence equivalence graph | reused for conditions |
| associated projection | `TypeData::AssociatedProjection` | no new P3 projection type |
| projection normalizer | `types::normalize_type` | singular recursive authority |
| projection inference | `InferenceTerm::AssociatedProjection` | one-way inference remains |
| conformance candidate/head | `ConformanceIndex`, `ConformanceHeadMatch` | reuse canonical head matcher |
| exact source proof | `ConformanceEvidence` | remains exact conformance basis |
| conformance source plan | `ConformanceWitnessPlan` / live equivalent | retains generic signature |
| semantic call specialization | `CallSpecialization` | receives derived evidence source |
| runtime type state | `RuntimeTypeEnvironment` | generic substitutions only |
| runtime evidence state | `RuntimeConformanceEnvironment` | behavioral handles only |
| abstract trait invocation | `InvokeTraitRequirement` | reused; no proof opcode |
| stable projection metadata | `TypeNode::AssociatedProjection` | reused |
| coherence | C4 conformance index/publication | no condition-based overload |

## 6.2 Freeze P1

```text
assumed evidence != exact evidence
structural method coincidence != conformance
Dynamic != static proof
source order != semantic identity
receiver specialization and evidence remain distinct
runtime type env != runtime conformance env
VM does not prove
```

## 6.3 Freeze P2

```text
specialization = Conforms + Equivalent
projection identity = subject + TraitRef + AssociatedTypeRequirementId
types::normalize_type is singular
one-way associated inference
no reverse open-world subject search
associated equality has no runtime slot
stable associated projection metadata is canonical
```

## 6.4 P2 audit remainders inherited by P3

P3 owns only bounded closure of these predecessor ambiguities:

```text
P2R-01  actual P2 landing revision / stale record revision
P2R-02  owner-generic runtime evidence test ignored
P2R-03  owner + method-local runtime evidence test ignored
P2R-04  exact enum-case generic conformance explicitly skipped
P2R-05  callable-reference audit outcome absent
P2R-06  required P2 CV mapping incomplete in walkthrough
P2R-07  cross-module associated contract/execution not clearly evidenced
P2R-08  recomputation-metrics acceptance not clearly evidenced
P2R-09  P2 terminal projection states not clearly evidenced
P2R-10  associated declaration bound semantics (`type Item |= Trait`) uncertain
P2R-11  C6 GUIDANCE lifecycle/current action stale
P2R-12  P2 handoff statement "all stabilization gates certified" is stronger than recorded evidence
```

T0 closes/classifies these without rerunning the entire P2 plan.

---

# 7. Architecture

## 7.1 Authority flow

```text
source conditional conformance
    ↓
canonical conformance head
+
canonical impl GenericSignature
    ↓
conformance index publication/coherence
    ↓
conformance goal
    ↓
RecursiveConformanceProofContext
    ├─ lexical assumed evidence
    ├─ memoized completed goals
    ├─ active/visiting goals
    ├─ C4 head matching
    ├─ impl substitution
    ├─ nested GenericConstraint proof
    │    ├─ Conforms → recursive proof
    │    ├─ Equivalent → P2 equality/normalization
    │    └─ Subtype → existing subtype relation
    ├─ projection normalization through SAME proof context
    ├─ budget/cancellation
    └─ nested proof provenance
    ↓
recursive/derived conformance evidence DAG
    ├─ source exact conformance identity
    ├─ exact target / TraitRef
    ├─ impl type substitution
    ├─ witness/default selections
    ├─ associated bindings
    └─ nested evidence dependencies
    ↓
CallSpecialization / semantic lowering
    ↓
derived runtime evidence recipe
    ├─ direct exact source
    ├─ direct caller forwarding
    └─ conditional construction from nested evidence inputs
    ↓
RuntimeTypeEnvironment
+
RuntimeConformanceEnvironment
    ↓
existing trait requirement invocation
```

## 7.2 Declaration formation

A conditional source conformance has two semantic parts:

```text
head:
    Target |= TraitRef

conditions:
    GenericSignature
```

The head is still the C4 conformance identity/coherence surface.

The conditions are not part of head identity.

The conformance body is checked under the impl `GenericSignature` as lexical assumed evidence.

Declaration validity and use-site applicability are separate judgments.

## 7.3 Recursive proof session

Introduce one explicit recursive proof/session context, conceptually:

```rust
struct ConformanceProofContext<'a> {
    lexical_evidence: Option<&'a GenericEvidenceEnvironment>,
    memo: BTreeMap<GenericConformanceKey, ProofMemoState>,
    stack: Vec<GenericConformanceKey>,
    budget: QueryBudget,
    cancellation: CancellationToken,
    // repository-native semantic dependencies / provenance
}
```

Exact name/container choices are flexible.

The semantic requirements are FIXED:

```text
one lexical environment per proof session
one memo table per session
one visiting state per canonical goal
one shared budget/cancellation context
nested projection proof shares this session
```

Do not make a global process-wide cache keyed only by subject + trait.

## 7.4 Proof memo state

Equivalent to:

```rust
enum ProofMemoState {
    Visiting,
    Completed(GenericConformanceProof),
}
```

or a richer internal state.

Rules:

```text
Completed success/failure → reuse
Visiting re-entry         → recursive/blocked terminal
Cancelled/budgeted        → not durable cross-session truth
snapshot-local IDs        → never persist across invalidation/generation
```

## 7.5 Generic obligation evaluator

One evaluator proves a substituted canonical `GenericConstraint`:

```text
Conforms
    → recursive conformance proof

Equivalent
    → identity/evidence equality + normalize_type through same proof context

Subtype
    → existing subtype relation after substitution/normalization
```

Kinds remain canonical parameter-kind facts and trait identity/well-formedness checks.

Do not create:

```text
ConditionalConstraint
ImplCondition
AssociatedCondition
```

as new semantic proposition classes.

## 7.6 Candidate proof

For goal:

```text
Box<User> |= Printable
```

and source:

```phalcom
impl<T> Printable for Box<T>
    where T |= Printable
```

the proof must:

```text
1. check lexical assumed evidence
2. memo lookup
3. mark canonical goal Visiting
4. query C4 ConformanceIndex
5. require coherent single source head
6. obtain impl binding T = User
7. instantiate impl GenericSignature
8. recursively prove each condition
9. if all succeed, construct recursive exact/derived evidence
10. memoize Completed(success)
11. return
```

If the condition is not proven, preserve its truthful terminal state/provenance.

## 7.7 Projection/proof mutual recursion

P2 introduced:

```text
projection normalization
    → conformance proof
```

P3 introduces:

```text
conditional proof
    → Equivalent condition
        → projection normalization
            → conformance proof
```

Therefore normalization invoked during P3 proof must use the active proof context.

A fresh stateless call to `query_generic_conformance_proof` from inside `normalize_type` is not sufficient if it loses the active recursion stack.

The implementation may thread:

```text
&mut ConformanceProofContext
```

through a normalization context, or use an equivalent repository-native callback/session object.

What is fixed:

> nested projection normalization and nested conformance proof participate in the same termination universe.

## 7.8 Recursive evidence

A successful conditional proof must retain enough information for runtime execution.

Conceptually:

```rust
struct DerivedConformanceEvidence {
    source: Arc<ConformanceEvidenceOrPlan>,
    exact_target: TypeId,
    exact_trait_ref: TraitRef,
    impl_bindings: TypeSubstitution,
    nested: Box<[NestedGenericEvidence]>,
}
```

Exact spelling is flexible.

P3 may either:

1. extend `ConformanceEvidence` with an immutable nested evidence/environment recipe; or
2. add a wrapper/derived evidence type consumed by generic call specialization/lowering.

FIXED requirement:

```text
outer proof retains nested behavioral evidence needed by the selected witness/default
```

Do not reduce a conditional proof to:

```text
bool
ImplId
plain ConformanceEvidence that discards required nested evidence
```

if its witness body uses the conditions.

## 7.9 Runtime construction

For a conditional witness body checked under:

```text
T |= Printable
```

with exact target substitution:

```text
T = User
```

the runtime activation must receive:

```text
RuntimeTypeEnvironment:
    T -> User

RuntimeConformanceEnvironment:
    executable slots satisfying T |= Printable
```

The semantic/lowering recipe may recursively compose evidence environments.

The VM executes those prepared handles.

It does not solve the condition.

## 7.10 Generic assumed evidence as conditional input

For a generic caller that knows:

```text
T |= Printable
```

and needs:

```text
Box<T> |= Printable
```

through the conditional source impl, P3 must construct **derived conditional evidence** from the caller's assumed slot(s).

This is not identical to P1's direct slot forwarding.

Conceptual recipe:

```text
DerivedConditional {
    source_impl,
    type_bindings,
    nested_sources: [
        ForwardedCallerEvidence(T |= Printable)
    ]
}
```

The resulting outer evidence provides `Box<T> |= Printable`.

## 7.11 Successful proof DAG

Because cycles are not proof, every proof reaching runtime lowering must be a finite DAG.

Repeated subproofs may be shared/interned.

No runtime recursive proof graph/search is needed.

## 7.12 Coherence

Candidate coherence is checked structurally at publication/head level.

Conditions are **not** candidate ranking.

Invalid:

```phalcom
impl<T> Trait for Box<T>
    where T |= A
{
}

impl<T> Trait for Box<T>
    where T |= B
{
}
```

even when the conditions happen to be disjoint for currently known types.

P3 does not prove domain disjointness.

---

# 8. Ownership boundaries

## 8.1 P3 owns

- bounded P2 corrective-closure items that recursive proof directly depends on;
- canonical conditional conformance `impl ... where ...` activation;
- impl-body lexical generic evidence;
- recursive generic conformance proof context/session;
- nested canonical constraint evaluation;
- proof memoization;
- cycle detection;
- proof budgets/cancellation propagation;
- nested failure provenance;
- conditional conformance exact/derived evidence construction;
- nested evidence retention for conformance-local witnesses/defaults;
- generic assumed evidence feeding conditional conformance proof;
- conditional associated binding normalization;
- conditional associated specialization/equality/subtype proof;
- recursive trait-application well-formedness where required by nested conditions;
- associated declaration constraint discharge if VERIFY-FIRST shows P2 only implemented syntax;
- preservation of current global no-overlap coherence;
- semantic-to-core derived evidence recipes;
- runtime construction/interning of nested behavioral evidence environments;
- cross-module conditional conformance contracts;
- incremental dependency/fingerprint integration;
- P3 diagnostics/tooling needed for nested conditions;
- P3 memoization/resource evidence;
- P3 walkthrough/handoff/checkpoint closure.

## 8.2 P3 does not own

- blanket conformance over arbitrary naked type parameters;
- `Iterator ⇒ Iterable` law/mechanism;
- supertraits;
- trait-object/existential evidence;
- negative conformances;
- specialization / "most specific wins";
- condition-based overlap partitioning;
- generic associated types;
- higher-kinded associated declarations;
- reverse open-world inference;
- production Iterator/Iterable migration;
- production `for` lowering replacement;
- Map/Range/lazy iterator migration;
- cursor/runtime Iterable-superclass retirement;
- broad unrelated exact-enum redesign;
- runtime associated dictionaries;
- runtime conformance search;
- recursive proof bytecodes;
- a new persistent callable-reference capture object unless separately approved.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| source impl generic contract | canonical generic signature formation | conformance body/proof/metadata | conformance-specific constraint parser |
| conformance head identity | C4 `ConformanceIndex` head | proof/coherence | P3 alternate head matcher |
| impl head substitution | `ConformanceHeadMatch` | nested obligation instantiation | solver-local rematching |
| lexical impl assumptions | `GenericEvidenceEnvironment` | witness/body checking | conditional evidence env |
| direct/nested propositions | `GenericConstraint` | recursive evaluator | ConditionalConstraint enum |
| projection meaning | `TypeData::AssociatedProjection` | equality/inference/proof | conditional projection type |
| projection normalization | `types::normalize_type` | P2/P3 | proof-local normalizer |
| recursive proof state | P3 proof context | conformance/equality normalization | fresh nested stateless proof |
| exact source conformance | `ConformanceEvidence` basis | derived evidence/lowering | fake ImplId proof |
| recursive derived evidence | P3 semantic evidence product | call specialization/lowering | runtime solver |
| runtime type substitution | `RuntimeTypeEnvironment` | witness execution | conformance env |
| runtime behavioral evidence | `RuntimeConformanceEnvironment` | witness execution | semantic proof graph |
| coherence | C4 conformance publication/index | declaration validation | condition solver |
| public generic constraints | stable metadata graph | importer/reflection | source reparsing |
| incremental dependencies | semantic DB/query products | replay/currentness | global invalidate-all |

---

# 9. Global invariants

`INV-01` — `GenericSignature` remains the single canonical generic contract authority.

`INV-02` — `GenericEvidenceEnvironment` remains the single lexical generic evidence authority.

`INV-03` — a conditional conformance's `where` clause is an ordinary canonical `GenericSignature`.

`INV-04` — conformance conditions are not part of `TraitRef` identity.

`INV-05` — conformance conditions are not part of structural conformance head identity.

`INV-06` — existing C4 head matching remains the only conformance head/substitution authority.

`INV-07` — declaration well-formedness and use-site applicability are distinct judgments.

`INV-08` — an impl body is checked under its own generic conditions as assumed evidence.

`INV-09` — generic assumed evidence is never fabricated as `ImplId` or exact source `ConformanceEvidence`.

`INV-10` — recursive proof evaluates canonical `Conforms`, `Equivalent`, and `Subtype`; no conditional-specific proposition model exists.

`INV-11` — associated specialization remains `Conforms + Equivalent`.

`INV-12` — `types::normalize_type` remains the singular associated projection normalizer.

`INV-13` — projection normalization and recursive conformance proof share one termination/proof context when mutually recursive.

`INV-14` — one proof session is scoped to one lexical evidence context.

`INV-15` — proof memo identity is canonical conformance goal within that proof session.

`INV-16` — `Visiting` re-entry is a cycle/recursive terminal, not success.

`INV-17` — productive recursion through strictly different canonical goals remains possible.

`INV-18` — repeated completed subgoals are memoized within one proof session.

`INV-19` — memoized assumed proof is never reused across an unrelated lexical environment.

`INV-20` — cancelled or budget-exhausted proof is not durable cross-session semantic truth.

`INV-21` — cycles, blocked, unknown, dynamic, cancelled, budgeted, ambiguous/coherence, incomplete, and internal failure remain distinguishable.

`INV-22` — `Dynamic` is not static conformance proof.

`INV-23` — nested `Equivalent` uses P2 equality/refinement and projection normalization.

`INV-24` — nested `Subtype` uses the existing subtype relation.

`INV-25` — no open-world reverse search is introduced by conditional proof.

`INV-26` — source constraint order does not affect proof result, coherence, or evidence identity.

`INV-27` — canonical equality order from P2 remains symmetric.

`INV-28` — successful conditional proof produces evidence, not merely `true`.

`INV-29` — derived evidence retains every nested behavioral evidence dependency required by the selected witness/default.

`INV-30` — associated binding evidence remains part of the exact/derived conformance result.

`INV-31` — conditional associated bindings normalize only after applicability is proven.

`INV-32` — associated declaration constraints, if active, are discharged by the same nested generic obligation engine.

`INV-33` — exact enum-case subject identity remains exact through conditional proof.

`INV-34` — receiver specialization remains the type-substitution authority for owner-generic methods; P3 evidence composes with it.

`INV-35` — owner and method evidence may satisfy conditional obligations without per-instance evidence arrays.

`INV-36` — direct P1 slot forwarding and P3 derived conditional evidence are distinct recipe shapes when semantics require construction.

`INV-37` — runtime type and conformance environments remain separate.

`INV-38` — the VM performs no conditional conformance proof/search.

`INV-39` — the VM performs no associated projection normalization/search.

`INV-40` — successful proof DAGs reaching runtime are finite.

`INV-41` — equal runtime conformance environments remain internable/reusable.

`INV-42` — nested behavioral handles remain rooted through existing runtime environment ownership.

`INV-43` — overlapping conformance heads remain rejected regardless of `where` conditions.

`INV-44` — there is no P3 "most specific wins" or condition-based specialization.

`INV-45` — candidate condition failure does not trigger search among structurally prohibited overlapping heads.

`INV-46` — conditional conformance does not authorize naked generic target heads if C4 currently forbids them.

`INV-47` — P3 does not implement Iterator ⇒ Iterable.

`INV-48` — public/cross-module conditional semantics are reconstructed from semantic products/metadata, not source reparsing.

`INV-49` — conditional GenericSignature fingerprints use canonical semantics, not raw source order.

`INV-50` — nested semantic dependencies participate in incremental invalidation without broad global invalidation.

`INV-51` — Phalcom remains method-based; no free functions are introduced in source fixtures or implementation.

`INV-52` — P2 one-way projection inference remains intact.

`INV-53` — trait application identity formation remains separate from recursive application well-formedness proof.

`INV-54` — recursive proof does not become a second inference engine.

`INV-55` — failed conditional proof does not publish partial executable call specialization/evidence.

`INV-56` — recursive proof budgets use the repository's canonical budget/cancellation mechanism.

`INV-57` — diagnostics preserve nested-condition provenance where known.

`INV-58` — P3's bounded P2 corrective closure does not reopen or redesign accepted P2 architecture.

---

# 10. Non-goals

Do not expand P3 into:

```text
blanket impl<I> Trait for I over arbitrary generic target
Iterator ⇒ Iterable theorem encoding
supertraits
trait object construction
existential evidence
negative impls
specialization / ranking
condition-domain disjointness proof
generic associated types
higher-kinded associated type declarations
reverse-search inference from associated output to unknown subject
GADT solver work
production collection iterator migration
production for-loop lowering replacement
Map/Range/lazy adapter migration
cursor retirement
runtime associated type dictionaries
runtime proof bytecodes
global proof cache keyed only by TypeId/TraitRef
broad callable-reference lifetime redesign
unrelated enum lowering redesign
```

---

# 11. Expected impact map

## 11.1 Expected semantic source areas

- `phalcom-semantic/src/impls.rs` — activate conditional conformance source contracts, head match integration, exact/derived evidence, coherence.
- `phalcom-semantic/src/generic_evidence.rs` — recursive proof context/session, memo state, nested canonical obligation evaluation, derived evidence source.
- `phalcom-semantic/src/types/projection.rs` — active proof-context integration for projection normalization.
- `phalcom-semantic/src/types/annotation.rs` — canonical impl where-clause formation and recursive trait-application well-formedness seam.
- `phalcom-semantic/src/traits.rs` — trait application identity/proof interface only if needed.
- `phalcom-semantic/src/checker/context.rs` — proof session/budget/cancellation ownership.
- `phalcom-semantic/src/checker/body.rs` / `declaration_signature.rs` — conditional impl lexical evidence and witness signature/body checking.
- `phalcom-semantic/src/checker/call.rs` — consume recursive/derived evidence in call specialization.
- `phalcom-semantic/src/checker/inference.rs` — only if active proof-context plumbing intersects P2 projection fixed point; do not redesign inference.
- `phalcom-semantic/src/db/fingerprint.rs` — conditional canonical contract fingerprints/dependencies.
- `phalcom-semantic/src/session.rs` — publication/replay/currentness dependencies.
- `phalcom-semantic/src/metadata/export.rs` — only if live conformance publication lacks durable conditional signature representation.
- `phalcom-semantic/src/source_index/builder.rs` — nested-condition occurrences if missing.

## 11.2 Expected core source areas

- `phalcom-core/src/modules/semantic_lowering.rs` — project recursive semantic evidence into runtime recipes.
- `phalcom-core/src/modules/compile.rs` — construct nested type/conformance environment recipes deterministically.
- `phalcom-core/src/typing/environment.rs` — type env materialization if current recipe cannot represent impl substitutions.
- `phalcom-core/src/typing/conformance_environment.rs` — intern/reuse nested behavioral environments; no semantic lookup.
- `phalcom-core/src/vm/dispatch.rs` / `send.rs` / frame plumbing — only if existing witness activation does not already attach prepared environments.

## 11.3 Expected metadata/tooling areas

- `phalcom-type-meta/*` — VERIFY-FIRST only; reuse current generic/projection metadata if sufficient.
- `phalcom-lsp/*` — focused diagnostics/navigation only if semantic source-index products require plumbing; no P3 text resolver.

## 11.4 Expected tests

Create if live conventions support:

```text
phalcom-semantic/tests/semantic/capabilities/conditional_conformance.rs
phalcom-core/tests/core/language/conditional_conformance.rs
```

Modify focused predecessor tests only for P2R closure/regression:

```text
generic_conformance.rs
generic_associated_constraints.rs
associated_traits.rs
incremental/fingerprints.rs
incremental/recomputation_metrics.rs
```

## 11.5 Expected documentation/state

```text
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md          # stale lifecycle/current-action correction only
LANG005.C6.P3-walkthrough.md
LANG005.C6.P3-handoff.md
```

## 11.6 Unexpected-touch rule

Adjacent helpers/tests may be touched when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem if doing so changes:

```text
semantic ownership
proof identity
GC/lifetime
runtime evidence representation
coherence rules
stable metadata model
incremental product identity
callable-reference capture contract
```

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

1. `GenericSignature` ownership of generic conditions.
2. `GenericEvidenceEnvironment` lexical evidence ownership.
3. `ConformanceIndex` head/candidate ownership.
4. C4 global no-overlap coherence.
5. `TraitRef` identity semantics.
6. P2 projection identity/normalization authority.
7. P2 one-way inference rule.
8. cycles-not-proof semantics.
9. rich terminal-state preservation.
10. proof-session scoping to lexical evidence.
11. requirement that conditional proof produce executable nested evidence.
12. separation of runtime type/conformance environments.
13. no-proof VM boundary.
14. no free functions.
15. no blanket generic-target conformance in P3.
16. no Iterator/Iterable special case.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- proof context type name;
- memo map container;
- internal `Visiting/Completed` representation;
- helper decomposition for canonical goal instantiation;
- whether derived evidence wraps or immutably extends current `ConformanceEvidence`, provided exact C4/C5 meaning remains intact;
- internal diagnostic provenance struct;
- exact budget charging granularity;
- private recursion callback/context threading through normalization;
- file-local helper placement;
- test fixture class/trait names;
- test filters;
- whether metadata changes are unnecessary because existing conformance publication already serializes the canonical signature;
- private runtime recipe decomposition, provided semantics remain exact.

## 12.3 VERIFY-FIRST

Before production edits, verify:

| Assumption | Where | If false |
|---|---|---|
| HEAD is still at/descends from `fc6325ad...` | git | adapt baseline; record new revision |
| P2 walkthrough/handoff revision is stale | docs + git | correct/amend records truthfully |
| two owner/receiver core tests remain ignored | core generic_conformance tests | run; repair only if active evidence substrate defect |
| exact enum-case remains skipped only in record, not broken | core/semantic exact-case tests | run; classify before P3 |
| callable-reference P2 audit has no durable outcome | tests/walkthrough/code | produce Outcome A or explicit B |
| `ImplWhereClauseUnsupported` still gates conformance impl where-clauses | `impls.rs` diagnostics | adapt live gate; do not duplicate formation |
| conformance products retain `generic_signature` | `impls.rs` | STOP if conditional conditions need new identity architecture |
| `ConformanceHeadMatch` retains impl substitution | `impls.rs` | STOP before adding alternate head matcher |
| direct proof remains stateless | `generic_evidence.rs` | extend live API rather than parallel proof entry |
| projection normalizer calls proof through a fresh/stateless path | `types/projection.rs` | thread active context; STOP if ownership conflict |
| `BlockReason::RecursiveFixpoint` or equivalent can represent conformance cycle | outcomes | reuse if semantically adequate |
| `QueryBudget`/`CancellationToken` are already available in semantic query paths | context/outcome | reuse canonical mechanism |
| `ConformanceEvidence` lacks nested evidence storage | `impls.rs` | choose wrapper/extension preserving C4 identity |
| runtime witness activation can receive type+conformance env | lowering/frame/vm | extend recipe mechanically; STOP if new lifetime architecture |
| bound/callable-reference objects already capture/reconstruct conformance env | core callable refs | add regression if yes; defer/consult if new persistent capture object needed |
| conditional conformance signatures are already included in semantic fingerprints | fingerprint/session | add canonical dependency if absent |
| conformance contributions are durable cross-module products | metadata/session | reuse; STOP before source-reparse workaround |
| `TraitRequirementSyntax.constraints` is semantic or syntax-only | trait surface formation | if syntax-only, T5 closes ordinary associated bound semantics |
| associated requirement constraints are persisted in trait surface | traits | if absent, add minimal P3 closure only |
| P2 cross-module associated contract actually works | focused provider/consumer test | repair before P3 cross-module conditional tests |
| P2 recomputation-metrics coverage exists/live | incremental tests | add focused discriminator, not full suite |

---

# 13. Global STOP / CONSULT triggers

STOP AND CONSULT immediately if:

1. conditional conformance requires replacing `ConformanceIndex`;
2. an impl `where` clause cannot use canonical `GenericSignature`;
3. correct proof requires a second generic proposition/evidence model;
4. recursive proof cannot share cycle state with projection normalization;
5. exact conformance identity must change to support nested evidence;
6. nested evidence requires runtime conformance search;
7. witness execution requires a new persistent GC/lifetime architecture;
8. callable references require a new heap evidence-capture object/lifetime contract;
9. correct coherence seems to require condition-based specialization;
10. overlapping heads must be tested/ranked at use-site;
11. a cycle must count as proof to satisfy a test;
12. the memo key would ignore lexical evidence/environment identity;
13. stable metadata would need snapshot-local semantic IDs;
14. cross-module proof would require reparsing provider source;
15. associated projection proof requires a second normalizer;
16. conditional proof introduces reverse open-world inference;
17. nested terminal states would need to collapse into `Missing`;
18. exact enum-case identity would need widening;
19. owner evidence would require per-instance generic evidence arrays;
20. a legal P3 case requires a naked generic target head currently forbidden by C4;
21. Iterator/Iterable-specific behavior begins appearing in generic proof/compiler/runtime;
22. `Dynamic` would have to count as static proof;
23. a global process-wide memo cache appears necessary for correctness;
24. proof budgets/cancellation cannot use existing semantic mechanisms;
25. P2R closure reveals a material P2 architecture defect rather than a missing regression/record;
26. expected semantic test behavior conflicts materially with normative specs/checkpoint authority;
27. the same nontrivial semantic failure survives one serious correction;
28. implementation enters an unplanned subsystem with a new ownership contract;
29. an invariant must be weakened to pass;
30. P3 scope becomes materially broader than conditional proof/evidence closure.

A triggered consultation cannot be self-waived.

---

# 14. Debugging budget

## 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress:

```text
inspect
→ identify concrete cause
→ make one coherent correction
→ rerun smallest discriminator
```

Examples:

```text
renamed helper
borrow checker cleanup
missing import
test registration
mechanical API signature drift
```

## 14.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same semantic failure.

If it persists: STOP AND CONSULT.

## 14.3 Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

Never rerun an unchanged failing test unless something relevant changed.

---

# 15. Testing surface analysis

## 15.1 P2 corrective-closure obligations

| ID | Dimension | Required evidence |
|---|---|---|
| P2R-01 | lifecycle | actual P2 landing revision recorded truthfully |
| P2R-02 | receiver/runtime | owner-generic receiver evidence executes |
| P2R-03 | composition/runtime | owner + method-local evidence executes |
| P2R-04 | exact identity | exact enum-case generic evidence passes without widening |
| P2R-05 | lifetime | callable-reference audit has explicit Outcome A or B |
| P2R-06 | verification truth | P2 audit remainder map recorded; no fake CV completion claim |
| P2R-07 | cross-module | projection-bearing imported generic contract works |
| P2R-08 | incremental | associated constraint recomputation/invalidation discriminator |
| P2R-09 | terminal state | blocked/recursive/cancel/budget states preserved where testable |
| P2R-10 | associated declaration bounds | semantic status verified; close if syntax-only |
| P2R-11 | guidance | current lifecycle/action corrected |
| P2R-12 | record truth | P2 gate overclaim amended/corrected |

## 15.2 P3 coverage obligations

| ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | source formation | INV-01,03 | `impl<T> Trait for Box<T> where T |= Bound` forms |
| CV-02 | source formation | INV-03,10 | multiple conditions form one canonical GenericSignature |
| CV-03 | source formation | INV-10,23 | conditional `Equivalent` forms |
| CV-04 | source formation | INV-10,24 | conditional `Subtype` forms |
| CV-05 | specialization | INV-11 | `where I |= Iterator<Item = T>` remains Conforms+Equivalent |
| CV-06 | body assumptions | INV-08 | witness body may use behavior justified by impl condition |
| CV-07 | negative body | INV-08 | behavior not justified by condition still rejects |
| CV-08 | declaration validity | INV-07 | future-dependent condition not required globally |
| CV-09 | formation negative | INV-07 | kind-invalid/malformed condition diagnoses |
| CV-10 | canonicalization | INV-26 | condition order/regrouping has same semantic identity |
| CV-11 | exact nested proof | INV-06,28 | one-level concrete conditional proof succeeds |
| CV-12 | exact nested failure | INV-21,57 | unsatisfied nested condition rejects with provenance |
| CV-13 | nested proof | INV-17 | two-level finite recursive success |
| CV-14 | nested proof | INV-17 | deep finite nested success |
| CV-15 | exact base | INV-17 | recursion terminates at unconditional exact base |
| CV-16 | default | INV-28,29 | conditional trait default selected |
| CV-17 | explicit witness | INV-28,29 | conditional explicit witness selected |
| CV-18 | associated binding | INV-30,31 | conditional associated binding normalizes |
| CV-19 | assumed nested proof | INV-09,28 | lexical assumed evidence satisfies conditional impl |
| CV-20 | derived evidence | INV-36 | resulting outer evidence satisfies generic callee |
| CV-21 | multiple nested | INV-29 | multiple nested evidence inputs retained independently |
| CV-22 | owner evidence | INV-34,35 | owner-generic assumption feeds conditional proof |
| CV-23 | owner+method | INV-34,35 | owner + method assumptions feed different conditions |
| CV-24 | equivalence success | INV-23 | conditional `A == B` succeeds after substitution |
| CV-25 | equivalence failure | INV-23 | conditional equality failure rejects |
| CV-26 | projection equality | INV-12,13,23 | projection-bearing equality condition succeeds |
| CV-27 | subtype success | INV-24 | conditional subtype succeeds |
| CV-28 | subtype failure | INV-24 | conditional subtype failure rejects |
| CV-29 | mixed conditions | INV-10 | Conforms+Equivalent+Subtype conjunction works |
| CV-30 | self cycle | INV-16,21 | direct recursive goal terminates non-success |
| CV-31 | mutual cycle | INV-16,21 | A→B→A terminates non-success |
| CV-32 | productive recursion | INV-17 | Box<Base> chain proves with base |
| CV-33 | memoization | INV-18 | diamond graph computes repeated subgoal once/session |
| CV-34 | lexical memo isolation | INV-19 | proof success in env A not reused in env B |
| CV-35 | budget | INV-20,21,56 | budget exhaustion preserved |
| CV-36 | cancellation | INV-20,21,56 | cancellation preserved |
| CV-37 | blocked | INV-21 | blocked nested proof remains blocked |
| CV-38 | Dynamic | INV-22 | Dynamic condition is not proof |
| CV-39 | internal failure | INV-21 | internal terminal propagated where test seam exists |
| CV-40 | coherence duplicate | INV-43 | identical conditional heads rejected |
| CV-41 | coherence conditions | INV-43,44 | same head/different where clauses still overlap |
| CV-42 | coherence mixed | INV-43 | unconditional + conditional overlapping head rejects |
| CV-43 | coherence generic/concrete | INV-43 | existing C4 overlap rule preserved |
| CV-44 | source order | INV-43,44 | declaration order never chooses |
| CV-45 | runtime concrete | INV-37,38 | concrete conditional witness executes |
| CV-46 | runtime default | INV-37,38 | conditional default executes |
| CV-47 | nested runtime evidence | INV-29,37 | witness body invokes nested trait behavior correctly |
| CV-48 | generic runtime evidence | INV-36 | assumed caller evidence constructs derived outer evidence |
| CV-49 | runtime type env | INV-37 | conditional witness receives impl type substitution |
| CV-50 | slot isolation | INV-29,36 | multiple conditional evidence values do not cross-wire |
| CV-51 | environment reuse | INV-41 | repeated same proof/evidence reuses equal runtime env |
| CV-52 | runtime anti-authority | INV-38,39 | zero runtime conditional/projection solver state |
| CV-53 | conditional projection | INV-13,31 | conditional conformance associated projection normalizes |
| CV-54 | specialization condition | INV-11,23 | associated specialization participates in conditional proof |
| CV-55 | imported projection | INV-48 | cross-module conditional associated binding normalizes |
| CV-56 | proof/normalization cycle | INV-13,16 | conformance↔projection recursion terminates |
| CV-57 | inference regression | INV-25,52 | P2 one-way inference remains |
| CV-58 | cross-module conformance | INV-48 | imported conditional source proves consumer goal |
| CV-59 | cross-module runtime | INV-38,48 | imported conditional witness executes |
| CV-60 | incremental base change | INV-50 | nested base conformance change invalidates outer consumers |
| CV-61 | incremental condition change | INV-49,50 | impl where-clause semantic change invalidates consumers |
| CV-62 | incremental no-op | INV-26,49 | canonical reorder/regroup preserves semantic fingerprint |
| CV-63 | cold/incremental parity | INV-50 | cold and incremental proof outcomes match |
| CV-64 | diagnostic | INV-57 | nested failing obligation identified |
| CV-65 | P1 regression | INV-58 | direct exact/assumed conformance suites remain green |
| CV-66 | P2 regression | INV-58 | associated projection/equality/inference suite remains green |
| CV-67 | C4/C5 regression | INV-06,12,33 | unconditional exact conformance/associated behavior unchanged |
| CV-68 | exact case | INV-33 | exact enum-case conditional/direct proof remains exact |
| CV-69 | method model | INV-51 | no free-function test/source form introduced |
| CV-70 | associated requirement bound | INV-32 | if live feature exists, binding constraint uses same proof engine |
| CV-71 | trait application WF | INV-53 | trait application nested generic contract uses recursive context |
| CV-72 | failed-call isolation | INV-55 | failed conditional call publishes no partial specialization |
| CV-73 | runtime rooting | INV-42 | nested evidence handles survive required GC/lifetime path |
| CV-74 | callable reference | P2R-05 | Outcome A regression or explicit accepted Outcome B deferral |
| CV-75 | memo resource | INV-18 | proof count tracks unique goals, not path count |

Not every coverage obligation requires a unique test. One discriminating integration test may prove multiple IDs. Final walkthrough must map every required ID to concrete evidence.

---

# 16. Verification execution budget

## 16.1 BUILD MODE

Most implementation time should be here.

Mandatory during BUILD:

```text
T0: exact P2 remainder discriminators only
T1: exact conditional source/body tests
T2/T3: exact recursive proof/cycle/memo tests
T4/T5: exact evidence/projection-condition tests
T6: exact runtime conditional evidence tests
T7: exact cross-module/incremental tests
```

Do not run broad crate suites after each task.

## 16.2 STABILIZE MODE

At named gates:

```text
conditional_conformance semantic module
conditional_conformance core module
generic_conformance semantic regression
generic_associated_constraints semantic regression
specific C4/C5 conformance/associated tests if shared source changed
incremental fingerprint/recomputation filters touched by P3
```

## 16.3 CERTIFY MODE

P3 is not C6 checkpoint closure.

Do not automatically run:

```text
cargo test --workspace
cargo clippy --workspace
full language corpus
full semantic integration suite
all phalcom-core tests
all LSP tests
release builds
```

Only run them if:

```text
a named repository checkpoint gate requires them
a directly affected subsystem cannot be meaningfully certified narrower
the user explicitly requests them
evidence reveals cross-cutting failure
```

## 16.4 Verification ladder

```text
T0 — exact reproducer/new regression
T1 — directly affected feature module
T2 — owning subsystem suite
T3 — adjacent cross-layer integration
T4 — broad crate/package only if justified
T5 — workspace/release only at explicit certification
```

---

# 17. Baseline / unrelated failure policy

Classify every failure:

```text
A — definitely caused by P3
B — probably caused by P3
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active responsibility.
- C: one bounded classification pass; if nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- do not consume adviser compute on D unless it blocks a required gate.
- do not weaken/skip assertions to make a broad gate green.

Inherited legacy iterator/Range baselines remain deferred unless the new conditional evidence path reproduces the same root defect.

P2R failures are not automatically P3 defects; classify whether they are:
- missing verification only,
- live predecessor bug P3 depends on,
- accepted deferral,
- unrelated baseline.

---

# 18. Tasks

# T0 — Re-ground live post-P2 state and close P2 prerequisite remainders

## Purpose

Establish truthful P3 baseline and remove ambiguity from the exact P1/P2 evidence surfaces recursive proof will consume. This task is bounded corrective closure, not P2 reimplementation.

## Preconditions

- P2 landed at/near `fc6325ad...`.
- no P3 recursive proof edits started.

## Consumes

```text
P2 walkthrough/handoff
P2 plan acceptance obligations
checkpoint consultation/completion records
live P1/P2 tests
live callable-reference path
current incremental/metadata tests
```

## Produces

1. actual P3 starting revision;
2. truthful P2 record amendment/correction;
3. owner/receiver evidence runtime disposition;
4. exact enum-case disposition;
5. callable-reference Outcome A or B;
6. cross-module associated contract evidence;
7. recomputation/terminal-state focused evidence;
8. associated declaration-bound semantic status;
9. updated C6 GUIDANCE current lifecycle/action;
10. an explicit `P2R-*` closure ledger in checkpoint/walkthrough notes.

## Files and symbols

**Read:**

```text
LANG005.C6-CHECKPOINT.md
LANG005.C6-GUIDANCE.md
P2 plan/walkthrough/handoff

phalcom-core/tests/core/language/generic_conformance.rs
phalcom-semantic/tests/semantic/capabilities/generic_conformance.rs
phalcom-semantic/tests/semantic/capabilities/generic_associated_constraints.rs
incremental/fingerprints.rs
incremental/recomputation_metrics.rs
callable-reference semantic/lowering/runtime paths
trait associated-requirement formation
```

**Modify only if needed:**

```text
focused predecessor tests
minimal active substrate defects
CHECKPOINT.md
GUIDANCE.md
P2 walkthrough/handoff metadata or explicit audit amendment
```

Do not rewrite historical records silently; preserve truth.

## Required implementation shape

### A. Live baseline

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -10 --oneline
```

Record the actual P3 planning/start revision.

If HEAD advanced past `fc6325ad...`, inspect intervening commits before continuing.

### B. Record correction

P2 completion records cite `af2d0aec...` even though the landing commit is `fc6325ad...`.

Use repository lifecycle conventions to either:

```text
correct front matter to actual final P2 revision
```

or:

```text
append an explicit checkpoint audit correction
```

Do not pretend the predecessor revision was the final implementation revision.

### C. Owner/receiver runtime evidence

Discover the two ignored tests.

Run them directly.

If they pass after P2's actual code:
- unignore if appropriate;
- record PASS.

If they expose a live generic evidence substrate defect required by P3:
- fix minimally;
- rerun only affected P1/P2 modules.

If they require a new architecture:
- STOP AND CONSULT.

### D. Exact enum-case

Run the exact enum-case generic conformance test without `--skip`.

Do not change exact case identity to make it pass.

### E. Callable-reference audit

Trace:

```text
method reference creation
bound method object/environment capture
later invocation
runtime type environment
runtime conformance environment
```

Use a method whose body depends on owner/method generic evidence.

Outcome A:
- existing architecture preserves/reconstructs evidence;
- add focused regression.

Outcome B:
- support requires new persistent heap evidence-capture/lifetime architecture;
- record explicit accepted deferral boundary;
- do not implement speculative architecture.

### F. Cross-module associated contract

Run/add the smallest provider/consumer test proving an imported projection-bearing generic signature reconstructs associated semantics.

### G. Incremental/terminal state

Run/add focused:

```text
associated semantic change recomputes dependent product
canonical source-order rewrite preserves semantic product
recursive/blocked projection terminal does not collapse
```

Do not rerun all P2 CVs.

### H. Associated declaration bound status

Determine whether:

```phalcom
trait Sequence {
    type Item |= Printable
}
```

is:
- syntax-only;
- published on trait surface;
- validated on conformance binding;
- serialized if needed.

Record the exact missing seam for T5 if incomplete.

### I. Guidance

Update stale lifecycle/current action to P3.

Do not rewrite stable guidance architecture unnecessarily.

## Forbidden approaches

- no broad P2 reimplementation;
- no claiming CV-01..64 all pass without evidence;
- no exact-case widening;
- no new callable-reference lifetime object;
- no workspace tests;
- no conditional proof edits yet.

## Test changes required

Covers:

```text
P2R-01..P2R-12
CV-68
CV-74
```

## Tests to run now

Use exact live filters discovered with `-- --list`.

At minimum:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints
```

For core, run exact owner/receiver/exact-case filters, not the whole module first.

Run exact incremental/cross-module/callable-reference filters only as needed.

## Tests explicitly deferred

- new conditional-conformance tests;
- broad semantic suite;
- broad core suite;
- workspace.

## Acceptance

Every P2R item is classified as:

```text
PASS / repaired
or
explicitly accepted deferred boundary
or
unrelated baseline
```

No unresolved P2 substrate defect remains that would invalidate recursive proof architecture.

## Local STOP / CONSULT triggers

- owner/receiver evidence requires per-instance evidence storage;
- exact enum-case requires identity widening;
- callable reference requires new heap capture/lifetime contract;
- cross-module associated semantics require new metadata architecture;
- P2 terminal-state semantics are materially unsound rather than under-tested.

## Checkpoint update

Record:
- actual start revision;
- P2 audit correction;
- any accepted deferral;
- T0 gate outcome.

---

# T1 — Activate canonical conditional conformance source contracts and impl-body lexical evidence

## Purpose

Make source conformance `where` clauses legal canonical generic contracts and check the conformance body under those assumptions without yet implementing recursive concrete applicability.

## Preconditions

- T0 accepted.
- P1/P2 `GenericSignature` formation live and green.
- conformance products retain `generic_signature` or equivalent.

## Consumes

```text
ImplDef syntax
canonical generic signature formation
TypeParameterOwner / impl generic parameters
GenericEvidenceEnvironment
ResolvedConformanceHead
ConformanceContribution
ConformanceWitnessPlan
existing C4 coherence/publication
```

## Produces

1. legal `impl<T> Trait for Target<T> where ...`;
2. canonical impl `GenericSignature`;
3. conformance-body lexical evidence environment;
4. conditional source contribution publication;
5. canonical fingerprints/dependencies;
6. removal/replacement of `ImplWhereClauseUnsupported` only after semantics exist.

## Files and symbols

Likely:

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
```

Tests:

```text
create/register capabilities/conditional_conformance.rs
```

## Required implementation shape

### A. Reuse canonical generic signature formation

Do not hand-parse impl constraints into a conformance-specific vector.

Use the same staged formation semantics as owner/method generic contracts:

```text
kinds
Subtype
Equivalent
Conforms
associated specialization → Conforms + Equivalent
```

### B. Separate head from conditions

Publish:

```text
head:
    target + TraitRef

generic_signature:
    conditions
```

Do not alter conformance head identity/fingerprint rules except to include canonical signature as declaration content/dependency.

### C. Body assumptions

When checking explicit witnesses/associated bindings/default requirements for the generic source conformance:

```text
impl GenericSignature
    ↓
GenericEvidenceEnvironment
    ↓
body/signature checking
```

The conditions are assumptions in the generic source body.

Do not attempt to prove them globally.

### D. Static invalidity

Use existing generic signature well-formedness/consistency.

A condition depending on generic substitution is legal even when not globally proven.

### E. Coherence

Run existing C4 coherence on the head independent of conditions.

A second overlapping head is rejected even if conditions differ.

## Forbidden approaches

- no `ConditionalConformanceConstraint` type;
- no use-site recursive proof yet;
- no condition-based overlap ranking;
- no exact proof requirement while checking generic impl body;
- no source-order semantic identity;
- no runtime changes.

## Test changes required

Covers:

```text
CV-01 CV-02 CV-03 CV-04 CV-05 CV-06 CV-07 CV-08 CV-09 CV-10
CV-40 CV-41 CV-42 CV-43 CV-44
```

## Tests to run now

Run only new semantic conditional source-formation/body/coherence tests.

Do not run core runtime tests.

## Tests explicitly deferred

- concrete nested applicability;
- recursion/cycles;
- runtime;
- cross-module;
- full P1/P2 regression.

## Acceptance

Conditional conformance source declarations form/publish canonical signatures, witness bodies see their assumptions, and existing head coherence rejects overlapping sources regardless of conditions.

## Local STOP / CONSULT triggers

- impl conditions cannot reuse canonical generic signature formation;
- conditions must enter head identity to publish;
- body assumptions require fake exact evidence;
- current coherence intentionally permits conditional overlap contrary to checkpoint/spec.

## Checkpoint update

Record the durable interface:

```text
source conformance may carry canonical GenericSignature
body checks under assumed impl evidence
head coherence remains condition-independent
```

---

# T2 — Introduce the recursive conformance proof context and generic obligation evaluator

## Purpose

Create the single recursive semantic proof session that owns memoization, cycle state, budget/cancellation, and nested canonical constraint evaluation.

## Preconditions

- T1 conditional generic signatures publish.
- current direct proof API identified.
- projection normalizer call path inspected.

## Consumes

```text
GenericConformanceKey
GenericConformanceProof
GenericConformanceEvidence
GenericEquivalenceProof
GenericEvidenceEnvironment
QueryBudget
CancellationToken
BlockReason
GenericConstraint
existing subtype relation
```

## Produces

1. recursive proof context/session;
2. canonical memo state;
3. cycle detection;
4. nested `GenericConstraint` evaluator;
5. nested failure provenance;
6. direct proof API routed through context without changing external semantics.

## Files and symbols

Primary:

```text
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/types/outcome.rs      # only if needed for existing recursive block reason
phalcom-semantic/src/checker/context.rs    # proof-session construction/ownership
```

Focused tests may be unit tests plus semantic integration.

## Required implementation shape

### A. Context

Equivalent to:

```rust
struct ConformanceProofContext<'a> {
    environment: Option<&'a GenericEvidenceEnvironment>,
    memo: BTreeMap<GenericConformanceKey, ProofMemoState>,
    stack: Vec<GenericConformanceKey>,
    budget: QueryBudget,
    cancellation: CancellationToken,
    // current semantic authorities
}
```

Do not persist this object into stable metadata or runtime.

### B. Memo semantics

Equivalent:

```text
not present   → begin proof, mark Visiting
Visiting      → recursive/blocked terminal
Completed(x)  → reuse x
```

### C. External API

Preserve a convenient direct proof entry point for existing callers.

It may construct a short-lived context internally.

Recursive callers must use a context-aware entry point and never create a fresh independent context for a nested goal.

### D. Constraint evaluator

Add one operation equivalent to:

```rust
prove_constraint(
    constraint: &GenericConstraint,
    substitution: &TypeSubstitution,
    context: &mut ConformanceProofContext,
) -> GenericObligationProof
```

`Conforms` recurses.

`Equivalent` initially delegates P2 equality/normalization and will be fully context-integrated in T5.

`Subtype` delegates existing subtype relation.

### E. Terminal states

Reuse existing proof variants/reasons where semantically adequate.

Prefer:

```text
Blocked(BlockReason::RecursiveFixpoint)
```

if live `BlockReason` already represents recursive fixpoint.

Do not add a cosmetic duplicate `Recursive` conformance proof variant unless required for semantic precision.

### F. Budget/cancel

Check cancellation and budget at new semantic work boundaries.

Do not charge memo hits as full new proof work.

## Forbidden approaches

- no global memo cache;
- no bool proof;
- no recursion depth alone as the semantic budget;
- no cycle = success;
- no `Missing` collapse;
- no runtime proof context;
- no candidate selection changes yet beyond direct existing path.

## Test changes required

Covers:

```text
CV-30 CV-31 CV-33 CV-34 CV-35 CV-36 CV-37 CV-38 CV-39 CV-75
```

Use synthetic/direct proof graphs where possible before conditional candidate integration.

## Tests to run now

Run only exact recursive context/memo/cycle unit/integration filters.

If the implementation cannot meaningfully test cycles until T3 candidate integration, test memo/context primitives now and defer source-level cycles.

## Tests explicitly deferred

- runtime;
- cross-module;
- broad P1/P2 suites.

## Acceptance

There is one reusable proof session with context-sensitive memoization, cycle detection, budget/cancellation, and rich terminal propagation.

## Local STOP / CONSULT triggers

- proof context must be global/static;
- lexical environment cannot be scoped to a proof session;
- existing query architecture prevents context reuse without a material ownership redesign;
- cycle state conflicts with existing semantic DB reentrancy in a nonlocal way.

## Checkpoint update

Record recursive proof-context interface if production-stable.

---

# T3 — Integrate conditional candidate applicability with C4 head matching, nested proof, and coherence

## Purpose

Make conditional source conformances prove exact/generic goals by reusing existing head matching and recursively proving the instantiated impl contract.

## Preconditions

- T1 conditional source contracts publish.
- T2 proof context exists.
- `ConformanceHeadMatch.impl_bindings` verified live.

## Consumes

```text
ConformanceIndex
ConformanceHeadMatch
ConformanceWitnessPlan
impl GenericSignature
recursive proof context
TypeSubstitution / instantiation helpers
```

## Produces

1. conditional conformance candidate proof;
2. substituted nested obligations;
3. one-level and multi-level nested conformance proof;
4. source-level self/mutual/productive recursion behavior;
5. nested failure provenance;
6. no condition-based overlap selection.

## Files and symbols

Primary:

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/generic_evidence.rs
```

Tests:

```text
conditional_conformance.rs
existing impl coherence/query tests
```

## Required implementation shape

### A. Candidate selection

For exact enough goals:

```text
query ConformanceIndex
→ obtain coherent head match
→ obtain impl bindings
```

Do not rematch target shapes in generic evidence.

### B. Unconditional candidate

Preserve current exact C4/C5 behavior.

Unconditional sources should continue through existing evidence resolution as directly as possible.

Do not force every old exact proof through expensive new nested machinery if no conditions exist.

### C. Conditional candidate

Instantiate the source `GenericSignature` under `impl_bindings`.

Prove every canonical condition in the same proof context.

### D. Failure semantics

For a unique coherent head whose conditions are ordinary false/missing:

```text
outer conformance is not proven
```

but diagnostic/proof detail should retain:

```text
source impl
failing nested obligation
nested terminal/provenance
```

### E. Recursive cycles

Source-level self/mutual recursion must hit the same canonical goal memo/Visiting state.

### F. Productive recursion

Goals such as:

```text
Box<Box<Base>> |= A
→ Box<Base> |= A
→ Base |= A exact
```

must succeed.

### G. Coherence

Do not collect multiple overlapping heads and test conditions to choose.

Publication/coherence should have rejected them.

If the live index can still expose a conflict, propagate `CoherenceConflict`.

## Forbidden approaches

- no alternate head matcher;
- no fallback to checking all source impl conditions;
- no source-order candidate ranking;
- no "first condition that passes";
- no widening exact enum case;
- no runtime effects.

## Test changes required

Covers:

```text
CV-11 CV-12 CV-13 CV-14 CV-15
CV-30 CV-31 CV-32
CV-40 CV-41 CV-42 CV-43 CV-44
CV-57
```

## Tests to run now

Run new semantic conditional proof/coherence module only.

Run focused existing `impls::queries` tests if shared coherence code changed.

## Tests explicitly deferred

- derived executable evidence runtime;
- cross-module;
- broad semantic.

## Acceptance

Conditional goals use existing head matching, instantiate canonical contracts, recursively prove conditions, terminate cycles, support productive finite recursion, and preserve C4 coherence.

## Local STOP / CONSULT triggers

- conditional applicability requires examining prohibited overlapping heads;
- `ConformanceHeadMatch` substitution is insufficient to instantiate the generic contract;
- exact/unconditional evidence semantics must change materially;
- source-level recursion bypasses T2 context because of semantic query layering.

## Checkpoint update

Record conditional candidate proof gate and coherence invariant.

---

# T4 — Construct recursively complete conditional conformance evidence

## Purpose

Turn successful conditional proof into a semantic evidence object/DAG that retains the nested evidence required to execute the selected conformance behavior.

## Preconditions

- T3 proves conditional applicability.
- exact C4/C5 evidence construction path inspected.
- runtime witness environment requirements traced.

## Consumes

```text
ConformanceEvidence
ConformanceWitnessPlan
ConformanceHeadMatch
GenericConformanceEvidence
nested GenericObligationProof results
associated binding selections
impl type substitution
```

## Produces

1. derived conditional conformance evidence;
2. nested evidence dependency representation;
3. explicit distinction between direct exact, direct assumed, and derived conditional sources;
4. failure to publish partial evidence on failed proof;
5. generic caller assumed evidence usable as an input to derived evidence.

## Required implementation shape

### A. Evidence information content

The successful product must retain equivalent information to:

```text
source impl
exact/materialized target
exact/materialized TraitRef
impl generic substitution
selected requirements/witness/defaults
associated binding selections
nested evidence dependencies
```

### B. Representation decision

Preferred design:

```text
preserve ConformanceEvidence as the exact source-selection object
+
wrap it in a derived generic proof/evidence product containing nested dependencies
```

Conceptually:

```rust
enum GenericConformanceEvidence {
    Exact(Arc<ConformanceEvidence>),
    Assumed(AssumedConformanceEvidence),
    Derived(Arc<DerivedConformanceEvidence>),
}
```

with:

```rust
struct DerivedConformanceEvidence {
    exact: Arc<ConformanceEvidence>,
    impl_bindings: TypeSubstitutionOrStableSemanticEquivalent,
    nested: Box<[GenericObligationEvidence]>,
}
```

However, if the live code naturally extends immutable `ConformanceEvidence` with nested evidence without changing its C4/C5 identity semantics, that is mechanically acceptable.

What is not acceptable:

```text
Derived = ImplId only
Derived = bool
Derived = exact witness selections without nested conditions
```

### C. Nested condition evidence

For `Conforms` condition:
- retain exact/assumed/derived generic conformance evidence.

For `Equivalent`/`Subtype`:
- no runtime evidence slot is required;
- retain semantic proof only as needed for provenance; do not create runtime equality/subtype dictionaries.

### D. Associated bindings

Successful derived evidence exposes the same associated binding map semantics as exact evidence.

### E. Generic assumed input

A source conditional impl may be proven under an abstract caller environment.

The derived evidence must be able to reference caller evidence requirements rather than pretending an exact source impl exists for the nested abstract subject.

### F. Failure isolation

If any nested obligation fails:
- do not cache/publish successful derived evidence;
- do not publish partial `CallSpecialization`.

## Forbidden approaches

- no fake exact impl for assumed nested evidence;
- no runtime equality/subtype slots;
- no mutable global evidence graph;
- no proof result without nested evidence;
- no partial evidence on failure.

## Test changes required

Covers:

```text
CV-16 CV-17 CV-18 CV-19 CV-20 CV-21 CV-22 CV-23
CV-28 CV-29 CV-72
```

## Tests to run now

Semantic evidence inspection tests only.

Do not require VM execution yet.

## Tests explicitly deferred

- compiler/core runtime;
- cross-module runtime;
- environment interning.

## Acceptance

A successful conditional proof returns a recursively complete immutable semantic evidence DAG sufficient for later lowering, including abstract/assumed nested evidence sources.

## Local STOP / CONSULT triggers

- extending/wrapping `ConformanceEvidence` changes C4/C5 exact identity semantics;
- nested evidence needs runtime semantic IDs/lifetimes;
- abstract derived evidence cannot represent caller-evidence dependencies without a new semantic authority.

## Checkpoint update

Record derived-evidence contract as a durable P3 interface.

---

# T5 — Integrate P2 projection/equality/subtype recursion, trait-application well-formedness, and associated declaration bounds

## Purpose

Make recursive conditional proof consume the complete P2 generic relation system under one proof context, including mutual recursion between conformance proof and projection normalization.

## Preconditions

- T2/T3 recursive context active.
- T4 derived evidence exists.
- P2 `types::normalize_type` and equivalence/refinement APIs green.
- T0 associated requirement bound status known.

## Consumes

```text
types::normalize_type
GenericEquivalenceProof
Subtype relation
TypeData::AssociatedProjection
TraitRef::form_identity
trait-application well-formedness seam
associated requirement constraints if live
```

## Produces

1. context-aware projection normalization;
2. conditional `Equivalent`;
3. conditional `Subtype`;
4. associated specialization in conditional signatures;
5. conditional associated binding normalization;
6. recursive trait-application well-formedness;
7. associated declaration bound discharge if required.

## Required implementation shape

### A. Shared proof context

When recursive proof evaluates:

```text
Equivalent(<T as Trait>::Assoc, U)
```

and normalization needs:

```text
T |= Trait
```

that nested conformance query must enter the same active proof context.

Do not call a context-free top-level proof function that loses `Visiting` state.

### B. Equivalent

Use:

```text
canonical identity
GenericEvidenceEnvironment::are_equivalent
types::normalize_type
```

under the active context.

P2 canonical symmetric orientation remains unchanged.

### C. Subtype

Instantiate/normalize both sides as appropriate, then use existing subtype relation.

Do not build a proof-local subtype solver.

### D. Associated binding normalization

For a conditional source conformance with:

```phalcom
type Item = T
```

after proof succeeds:

```text
<ExactTarget as Trait>::Item
```

normalizes through the resulting derived/exact evidence.

### E. Associated specialization in conditions

Use ordinary:

```text
Conforms
+
Equivalent(projection, target)
```

### F. Trait application well-formedness

Keep:

```text
TraitRef identity formation
!=
proof of TraitRef GenericSignature
```

Nested generic trait constraints may invoke P3 context.

### G. Associated declaration constraints

If T0 proves P2 only added syntax for:

```phalcom
type Item |= Printable
```

then minimally close:

```text
trait surface retains associated requirement GenericSignature/constraints
conformance associated binding instantiates subject/binding
same recursive evaluator proves the associated bound
```

No second associated-bound solver.

If already implemented, add regression only.

## Forbidden approaches

- no second projection normalizer;
- no context-reset during normalization;
- no reverse open-world subject search;
- no runtime associated dictionary;
- no special associated condition representation;
- no GAT/higher-kinded associated declaration work.

## Test changes required

Covers:

```text
CV-24 CV-25 CV-26 CV-27 CV-28 CV-29
CV-53 CV-54 CV-56 CV-57 CV-70 CV-71
```

## Tests to run now

Run:

```text
new conditional semantic relation tests
generic_associated_constraints focused module
```

Do not run core runtime yet unless shared code unexpectedly changes core compilation.

## Tests explicitly deferred

- cross-module;
- broad incremental;
- runtime derived evidence.

## Acceptance

P3 and P2 form one sound recursive semantic subsystem: conditional proof can consume projection/equality/subtype conditions and projection normalization cannot escape P3 cycle/memo/budget state.

## Local STOP / CONSULT triggers

- sharing context requires a second normalizer;
- projection inference needs reverse search;
- trait application proof must be folded back into TraitRef identity;
- associated declaration bounds require generic associated types.

## Checkpoint update

Record shared proof/normalization context and any associated-bound closure.

---

# T6 — Lower derived conditional evidence through existing runtime type/conformance environments

## Purpose

Project finite semantic derived-evidence DAGs into compiler/runtime recipes so conditional witnesses/defaults execute with required nested evidence while the VM performs zero proof.

## Preconditions

- T4 derived evidence stable.
- T5 relation integration stable.
- runtime call/witness environment paths traced.
- T0 callable-reference outcome known.

## Consumes

```text
derived conditional semantic evidence
CallSpecialization
RuntimeTypeEnvironment recipes
RuntimeConformanceEnvironment recipes
GenericEvidenceSlotKey/layout
exact witness/default requirement selections
caller forwarded evidence slots
RuntimeConformanceEnvironmentRegistry::intern
existing detached/bound method preparation
```

## Produces

1. derived conditional runtime recipe;
2. impl generic type environment;
3. nested behavioral conformance environment;
4. execution of conditional explicit witnesses;
5. execution of conditional defaults;
6. generic caller assumed evidence → derived outer evidence execution;
7. environment interning/rooting evidence;
8. no runtime proof/search.

## Required implementation shape

### A. Recipe source variants

Extend repository-native evidence recipe representation to express equivalent cases:

```text
DirectExact
DirectForwarded
DerivedConditional {
    source_conformance,
    type_bindings,
    nested_behavioral_sources
}
```

Do not encode semantic `GenericConstraint` at runtime.

### B. Type environment

Materialize impl generic bindings for the selected witness/default activation.

Example:

```text
T = User
```

must be visible through `RuntimeTypeEnvironment` when the conditional witness body executes.

### C. Conformance environment

For each behavioral evidence slot required by the generic witness body:

```text
nested proof source
    → exact prepared method handle
    or caller ambient handle
    or recursively derived evidence result
```

build/intern the callee/witness environment.

### D. Derived abstract evidence

For generic caller:

```text
T |= Printable
→ derived Box<T> |= Printable
```

the outer witness environment may depend on caller slot(s).

Do not perform exact conformance search at runtime.

### E. Runtime anti-authority

VM receives:

```text
prepared callable/method handles
type env IDs
conformance env IDs
slot indices
```

not:

```text
ImplId candidate search
GenericSignature
GenericConstraint
proof stack
TraitRef solver
```

### F. Environment reuse

Use existing interning.

Repeated identical conditional evidence must not create one permanent environment per invocation.

### G. Rooting

All nested handles remain reachable via existing registry/rooting paths.

### H. Callable references

If T0 Outcome A:
- add conditional-evidence method-reference regression if naturally affected.

If T0 Outcome B:
- do not extend capture architecture in P3 unless conditional runtime correctness makes the deferral impossible; if so STOP AND CONSULT.

## Forbidden approaches

- no runtime conformance search;
- no runtime proof graph interpreter;
- no runtime equality/subtype/projection slots;
- no new VM opcode solely for conditional proof;
- no per-object evidence arrays;
- no unrooted handle side table;
- no invocation-count environment growth.

## Test changes required

Covers:

```text
CV-45 CV-46 CV-47 CV-48 CV-49 CV-50 CV-51 CV-52 CV-59 CV-73 CV-74
```

## Tests to run now

Create/run exact new core conditional-conformance runtime module.

Representative cases:

```text
conditional explicit witness executes
conditional default executes
witness uses nested evidence
generic assumed nested evidence constructs derived outer evidence
multiple evidence inputs stay isolated
repeated invocation reuses environment
```

Then run focused `language::generic_conformance` if shared lowering code changed.

## Tests explicitly deferred

- full core suite;
- production iterator tests;
- workspace.

## Acceptance

Conditional evidence executes entirely through existing environment machinery, including abstract nested inputs, with correct type/evidence environments, rooting, interning, and zero runtime proof.

## Local STOP / CONSULT triggers

- existing witness activation cannot attach both environments without new lifetime architecture;
- derived evidence cannot be represented as deterministic recipes;
- VM must inspect semantic constraints;
- nested evidence requires per-instance storage;
- equal environments cannot be interned without ownership redesign.

## Checkpoint update

Record runtime evidence gate and no-proof boundary.

---

# T7 — Close cross-module, incremental, metadata, diagnostics, and tooling integration

## Purpose

Make conditional conformance a durable semantic contract across module boundaries and incremental recomputation, and close user-facing nested diagnostics/navigation without introducing new identity authorities.

## Preconditions

- T1–T6 semantic/runtime vertical works in one module.
- P2 cross-module associated prerequisite passed in T0.

## Consumes

```text
conformance contribution/publication products
canonical impl GenericSignature
stable generic constraint/projection metadata
semantic dependency graph
source index/editor targets
recursive proof diagnostics
```

## Produces

1. cross-module conditional proof;
2. cross-module conditional runtime execution;
3. imported conditional associated binding normalization;
4. canonical conditional signature fingerprints;
5. nested dependency invalidation;
6. cold/incremental parity;
7. nested-condition diagnostics;
8. source-index/LSP identity for condition references.

## Required implementation shape

### A. Metadata/publication

First verify whether the current conformance contribution/interface already serializes or publishes the impl `GenericSignature`.

If yes:
- reuse it;
- add validation/tests.

If no:
- add only the stable generic signature reference needed by the semantic conformance contribution.
- reuse existing `GenericConstraintRef` and `TypeNode::AssociatedProjection`.

Do not invent a conditional-specific constraint serialization.

### B. Cross-module proof

Provider/consumer must use semantic products.

No source reparsing.

Example topology:

```text
module A:
    Printable
    User
    exact User |= Printable

module B:
    Box<T>
    conditional Box<T> |= Printable where T |= Printable

module C:
    consumes Box<User> as Printable
```

### C. Conditional associated binding

Cross-module consumer must normalize associated bindings from a conditional conformance after nested proof succeeds.

### D. Fingerprints

Canonical semantic fingerprint includes:

```text
head identity
canonical GenericSignature
witness/associated contribution identity
```

Condition source ordering/regrouping/equality polarity must not create false semantic changes.

### E. Dependencies

Proof consumer depends on:

```text
conditional source contribution
nested exact/conditional conformance contributions
relevant trait surface
relevant associated/equality/subtype products
```

Do not `invalidate_all`.

### F. Incremental tests

Required:

```text
nested base conformance add/remove/change
conditional where-clause semantic change
canonical no-op reorder/regroup
cold vs incremental parity
```

### G. Diagnostics

When a conditional candidate fails, preserve nested obligation source/provenance.

Preferred concept:

```text
`Box<User>` does not satisfy `Printable`

candidate:
    impl<T> Printable for Box<T>
        where T |= Printable

unsatisfied nested obligation:
    User |= Printable
```

Use repository diagnostic conventions.

### H. Tooling

Condition trait/projection names must resolve through canonical source-index targets.

No LSP text matcher.

## Forbidden approaches

- no source reparsing;
- no snapshot-local metadata IDs;
- no raw source-order fingerprint;
- no global invalidation;
- no duplicate source-index identity;
- no fake binary representation for conformance/refinement.

## Test changes required

Covers:

```text
CV-55 CV-58 CV-59 CV-60 CV-61 CV-62 CV-63 CV-64
```

plus P2R-07/P2R-08 if not already closed.

## Tests to run now

Exact cross-module conditional tests.

Exact incremental fingerprint/recomputation filters.

Exact diagnostic/source-index tests.

Run full LSP package only if shared LSP code changes materially; semantic-only source-index changes do not automatically justify it.

## Tests explicitly deferred

- workspace;
- production iterator modules;
- unrelated metadata suites.

## Acceptance

Conditional conformance semantics survive publication/import, nested dependencies invalidate precisely, no-op canonical rewrites remain semantic no-ops, diagnostics identify nested failures, and tooling uses canonical identities.

## Local STOP / CONSULT triggers

- public conditional conformance requires new metadata identity architecture;
- incremental correctness needs broad invalidation;
- nested proof dependencies cannot be represented by semantic query products;
- cross-module proof requires source parsing.

## Checkpoint update

Record cross-module/incremental gate.

---

# T8 — Stabilize P3, prove resource bounds, synchronize records, and hand off to P4

## Purpose

Run bounded final gates, map every required coverage obligation to evidence, classify remaining baselines, prove memo/environment resource behavior, update checkpoint/guidance truthfully, and create P3 walkthrough/handoff.

## Preconditions

- T0–T7 accepted.
- no unresolved A/B failures.
- consultations/amendments recorded.

## Consumes

All P3 products/tests.

## Produces

```text
final P3 coverage ledger
performance/resource evidence
truthful checkpoint state
LANG005.C6.P3-walkthrough.md
LANG005.C6.P3-handoff.md
P4 takeover state
```

## Required implementation shape

### A. Coverage ledger

Map:

```text
P2R-01..P2R-12
CV-01..CV-75
```

to:

```text
test/code audit
exact command
PASS/DEFERRED/BASELINE
what it proves
```

Do not claim unexecuted tests passed.

### B. Resource evidence

Record:

```text
unique canonical proof goals
proof computations
memo hits
cycle termination result
budget termination result

runtime environment count before
distinct derived evidence environments
repeated invocation count
runtime environment count after
```

No microbenchmark required.

### C. Runtime anti-authority audit

Confirm no new:

```text
conditional proof VM opcode
runtime GenericConstraint interpreter
runtime conformance candidate table used for proof
runtime associated projection solver
```

### D. Regression gates

Run named gates only.

### E. Lifecycle truth

Normal P3 completion:

```yaml
plan:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

checkpoint:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: FOCUSED_TESTED
```

P3 completion does not close C6.

### F. Next action

P4 owns:

```text
Iterator/Iterable generic laws
first-class derived/blanket implication decision
canonical Universe trait/conformance migration
```

P4 must consume P3 proof/evidence rather than invent an Iterator special case.

## Forbidden approaches

- no broad "fix everything" stabilization;
- no weakened assertions;
- no unrecorded deferral;
- no workspace/release certification claim from focused tests;
- no P4 implementation during P3 closure.

## Test changes required

All remaining coverage.

## Tests to run now

Run gates in Section 19.

## Tests explicitly deferred

Workspace/release gates unless explicitly requested.

## Acceptance

All required P3 coverage is mapped to actual evidence or an explicit accepted deferral/baseline, resource behavior is proven, no active P3-owned blocker remains, and durable records accurately describe what happened.

## Local STOP / CONSULT triggers

- final coverage reveals an unplanned architectural gap;
- P3 can pass only by weakening a fixed invariant;
- a P4 feature is required for P3 correctness rather than merely later usage.

## Checkpoint update

Final P3 lifecycle/state update.

---

# 19. Verification gates

# G0 — P2 corrective-closure prerequisite gate

**Purpose:** prove P3 is not building recursive proof on ambiguous predecessor evidence.

Required outcomes:

```text
actual P2 commit/start revision recorded
owner receiver evidence classified/passing
owner + method evidence classified/passing
exact enum-case test run
callable-reference Outcome A/B recorded
cross-module associated contract evidenced
incremental recomputation discriminator evidenced
terminal P2 state preservation evidenced
associated requirement bound status known
guidance lifecycle corrected
P2 record overclaim/revision corrected or amended
```

Do not begin T1 if G0 reveals a material architecture defect.

---

# G1 — Conditional source contract and coherence gate

**Purpose:** prove source conditional conformances form canonical GenericSignatures, impl bodies see assumptions, and existing no-overlap coherence remains.

Run the new semantic conditional module filtered to formation/body/coherence.

Expected:

```text
impl where legal
canonical condition identity
body assumed evidence
future-dependent conditions legal
malformed/kind errors diagnose
same head different conditions still overlaps
no source-order choice
```

No runtime tests.

---

# G2 — Recursive proof, cycle, memoization, and terminal-state gate

**Purpose:** prove the recursive proof engine terminates and retains semantic distinctions.

Expected:

```text
one-level nested success/failure
two-level/deep finite success
self-cycle
mutual-cycle
productive recursion
diamond memoization
lexical env memo isolation
budget
cancellation
blocked
Dynamic not proof
nested failure provenance
```

Run only the new semantic conditional module plus exact proof unit tests.

---

# G3 — P2 relation/projection integration gate

**Purpose:** prove recursive conformance proof composes with P2 equality, subtype, associated specialization, projection normalization, and one-way inference.

Run:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::conditional_conformance
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints
```

Use exact filters if module is large.

Expected:

```text
conditional equality/subtype
projection-bearing condition
conditional associated binding
proof↔projection cycle termination
trait application nested WF
associated requirement bound closure if applicable
P2 one-way inference still green
```

---

# G4 — Derived evidence runtime execution gate

**Purpose:** prove recursive evidence is executable and the VM remains proof-free.

Run:

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core language::conditional_conformance
```

plus focused existing `language::generic_conformance` if shared lowering changed.

Expected:

```text
conditional explicit witness executes
conditional default executes
witness body uses nested evidence
generic assumed nested evidence constructs outer evidence
impl type env correct
multiple evidence inputs isolated
environment interning/reuse
handles rooted
zero runtime proof/search
```

---

# G5 — Cross-module, incremental, metadata, and tooling gate

**Purpose:** prove conditional semantics are durable semantic products.

Run exact tests for:

```text
cross-module conditional proof
cross-module conditional runtime
conditional associated projection
incremental nested dependency invalidation
condition canonical no-op
cold/incremental parity
nested diagnostic
source-index navigation
```

Only run full LSP if LSP code changed.

---

# G6 — Final focused P3 stabilization gate

Mandatory focused regressions:

```sh
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::conditional_conformance
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::generic_conformance
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic capabilities::generic_associated_constraints
RUSTFLAGS='' cargo test -p phalcom-core --test core language::conditional_conformance
```

Additionally run:

```text
exact C4/C5 conformance/associated regression filters if shared code changed
exact incremental/cross-module filters touched by P3
```

Do not automatically run workspace tests/clippy.

Expected:

```text
all P3-owned coverage mapped
no unresolved A/B failures
P1/P2 substrate remains green
runtime anti-authority preserved
resource behavior proven
```

---

# 20. Final focused acceptance

P3 may be marked `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only when:

1. G0–G6 required gates have truthful outcomes;
2. every P2R item is closed/classified;
3. every required CV item is mapped to evidence;
4. conditional source impls are canonical generic contracts;
5. recursive proof is cycle-safe, memoized, budgeted, cancellable;
6. successful proof contains executable nested evidence;
7. generic assumed evidence can feed derived conditional evidence;
8. P2 projection/equality/subtype relations participate in the same proof context;
9. conditional associated bindings normalize;
10. current C4 global no-overlap coherence is preserved;
11. cross-module/incremental semantics are proven;
12. runtime performs zero conditional proof/search;
13. environment reuse/rooting is proven structurally;
14. no P4 Iterator implication mechanism is introduced.

Final walkthrough must include a table:

| Coverage | Evidence | Result |
|---|---|---|
| P2R-* / CV-* | test or audit | PASS / approved deferral / baseline |

No silent omissions.

---

# 21. Performance/resource evidence

P3 is not a microbenchmark plan. It has structural performance requirements.

## 21.1 Proof memoization

Instrument or test one diamond graph:

```text
A
├─ B ─ D
└─ C ─ D
```

Expected:

```text
D computed once per proof session
later path is memo hit
```

A debug/test counter is sufficient.

## 21.2 Cycle termination

Self and mutual cycles must terminate in bounded semantic work.

No wall-clock benchmark needed.

## 21.3 Budget termination

A deliberately deep productive chain with a small budget must return `BudgetExceeded` rather than stack overflow/hang/missing.

## 21.4 Fixed-point interaction

P2's 16-step associated inference loop must not multiply into uncontrolled recursive proof.

Evidence should show:

```text
proof context reused
normalization reinvokes proof only when required
fixed-point repeats only when information changes
memo hits avoid duplicate nested proof
```

## 21.5 Conformance index reuse

Do not scan all conformances manually for every nested obligation.

Use the canonical index/query.

## 21.6 Runtime environment reuse

Record registry cardinality around repeated identical conditional calls.

Growth should correlate with distinct evidence environments, not invocation count.

## 21.7 GC/rooting

Conditional nested method handles must remain rooted through existing registry/environment ownership.

No new root family expected.

---

# 22. Checkpoint bookkeeping

## 22.1 At plan start

Run:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -10 --oneline
```

Update checkpoint only after implementation actually begins:

```text
active plan: LANG005.C6.P3
starting revision: actual live HEAD
P2 predecessor: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P2 audit: corrective closure in T0
```

## 22.2 During plan

Update only for durable events:

```text
T0 P2 audit closure
conditional impl canonical contract established
recursive proof-context interface established
coherence gate passes
derived evidence contract established
proof/normalization shared-context contract established
runtime derived evidence gate passes
cross-module/incremental gate passes
consultation/amendment
meaningful baseline classification
```

Do not log every command.

## 22.3 At completion

Record:

```text
P3 final lifecycle state
actual final revision
G0-G6 outcomes
P2R closure
final recursive proof/evidence interfaces
coherence rule
performance/resource evidence
deferred/baseline failures
consultations/amendments
next action: C6.P4
```

C6 remains partial.

---

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P3-walkthrough.md
```

It must contain:

1. actual starting and final revisions;
2. final P3 lifecycle state;
3. P2 corrective-closure outcomes `P2R-01..P2R-12`;
4. conditional impl canonical formation;
5. impl-body lexical evidence architecture;
6. recursive proof context/session design;
7. canonical memo key and memo states;
8. cycle semantics;
9. budget/cancellation semantics;
10. generic nested obligation evaluator;
11. C4 head matching integration;
12. conditional coherence behavior;
13. derived conditional evidence representation;
14. generic assumed → derived evidence behavior;
15. projection/equality/subtype integration;
16. shared proof/normalization recursion contract;
17. associated declaration-bound closure outcome;
18. conditional associated binding behavior;
19. runtime lowering recipe;
20. environment interning/rooting evidence;
21. callable-reference audit outcome;
22. cross-module publication/import;
23. incremental dependency/fingerprint behavior;
24. diagnostics/tooling;
25. deviations;
26. consultations;
27. tests added;
28. tests actually run;
29. tests explicitly deferred;
30. full P2R/CV result mapping;
31. performance/resource evidence;
32. remaining baselines/risks;
33. verification classification;
34. P4 prerequisites.

Do not claim workspace/release certification unless it actually occurred.

---

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P3-handoff.md
```

It must give P4 an immediately usable takeover state.

Include:

```text
current revision
P3 lifecycle state
conditional impl GenericSignature representation
recursive proof-context API
memo key/state semantics
cycle terminal representation
nested generic obligation evaluator
derived conditional evidence representation
runtime recipe representation
conditional associated binding behavior
trait application recursive WF API
coherence rule: head overlap remains global
cross-module/incremental state
performance/memo/environment evidence
callable-reference state
associated declaration-bound state
deferred/baseline failures
must-read files
first P4 commands
```

Explicitly tell P4:

```text
P3 does NOT establish Iterator ⇒ Iterable.
P4 must choose/implement a first-class language mechanism.
No compiler/VM Iterator special case is allowed.
P4 must consume P3 proof/evidence rather than redesign it.
```

---

# 25. Completion truth table

Do not conflate:

```text
P3 plan file written
implementation started
conditional syntax/formation written
recursive proof written
derived evidence written
runtime execution passes
cross-module/incremental passes
focused P3 gates pass
P3 walkthrough/handoff written
P3 accepted
C6 checkpoint complete
workspace/release certified
```

Expected normal P3 completion:

```yaml
plan:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

checkpoint:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: FOCUSED_TESTED
```

C6 still has P4–P6.

---

# 26. Plan self-review

## Architecture

- [x] one generic contract authority;
- [x] one lexical evidence authority;
- [x] one conformance head matcher;
- [x] one projection normalizer;
- [x] recursive proof context explicit;
- [x] lexical memo scoping explicit;
- [x] cycles are not proof;
- [x] productive recursion remains possible;
- [x] rich terminal states preserved;
- [x] P2 equality/subtype/projection reused;
- [x] successful proof produces recursive executable evidence;
- [x] assumed nested evidence supported;
- [x] runtime type/conformance environments separate;
- [x] no-proof VM boundary fixed;
- [x] C4 global coherence preserved;
- [x] no condition-based specialization;
- [x] no blanket generic target in P3;
- [x] no Iterator-specific semantics;
- [x] no free functions.

## P2 corrective closure

- [x] stale P2 revision recognized;
- [x] ignored owner/receiver tests owned;
- [x] exact-case skip owned;
- [x] callable-reference missing outcome owned;
- [x] cross-module associated coverage owned;
- [x] incremental recomputation coverage owned;
- [x] terminal-state coverage owned;
- [x] associated declaration bound uncertainty owned;
- [x] stale guidance owned;
- [x] verification overclaim correction owned.

## Luna executability

- [x] FIXED/FLEXIBLE/VERIFY-FIRST explicit;
- [x] tasks are coherent, not edit-sized;
- [x] forbidden shortcuts explicit;
- [x] semantic debugging bounded;
- [x] architecture failures immediately escalated;
- [x] broad testing explicitly deferred.

## Testing

- [x] success/failure/negative cases;
- [x] recursion/cycles/memoization;
- [x] exact/assumed/derived evidence;
- [x] owner/receiver composition;
- [x] equality/subtype/projection interaction;
- [x] coherence;
- [x] runtime execution/rooting/reuse;
- [x] cross-module;
- [x] incremental;
- [x] diagnostics/tooling;
- [x] exact-case and P1/P2 regressions;
- [x] performance/resource structural evidence.

## Lifecycle

- [x] checkpoint updates bounded;
- [x] guidance correction bounded;
- [x] walkthrough required;
- [x] P4 handoff required;
- [x] P3 does not claim C6 closure;
- [x] no release certification implied.

No implementation placeholders remain. VERIFY-FIRST items are explicit repository facts to inspect, each with a defined local-adapt or STOP/CONSULT response.

---

# Appendix A — Conceptual recursive proof types

Exact Rust spelling is not mandatory.

```rust
enum ProofMemoState {
    Visiting,
    Completed(GenericConformanceProof),
}
```

```rust
struct ConformanceProofContext<'a> {
    lexical_evidence: Option<&'a GenericEvidenceEnvironment>,
    memo: BTreeMap<GenericConformanceKey, ProofMemoState>,
    stack: Vec<GenericConformanceKey>,
    budget: QueryBudget,
    cancellation: CancellationToken,
}
```

```rust
enum GenericObligationProof {
    Conformance(GenericConformanceProof),
    Equivalent(GenericEquivalenceProof),
    Subtype(RelationOutcome),
}
```

Preferred evidence extension:

```rust
enum GenericConformanceEvidence {
    Exact(Arc<ConformanceEvidence>),
    Assumed(AssumedConformanceEvidence),
    Derived(Arc<DerivedConformanceEvidence>),
}
```

```rust
struct DerivedConformanceEvidence {
    exact_source: Arc<ConformanceEvidence>,
    // canonical impl parameter substitution / environment
    nested: Box<[NestedGenericEvidence]>,
}
```

```rust
enum NestedGenericEvidence {
    Conformance(GenericConformanceEvidence),
    Equivalent,
    Subtype,
}
```

The implementation may retain only runtime-relevant nested evidence in the durable executable product while preserving diagnostic semantic proof separately.

---

# Appendix B — Required proof examples

## B.1 Concrete one-level proof

```phalcom
impl<T> Printable for Box<T>
    where T |= Printable
{
    ...
}
```

Goal:

```text
Box<User> |= Printable
```

Flow:

```text
head match
T = User
prove User |= Printable
    → exact evidence
construct Box<User> derived evidence
```

## B.2 Generic assumed proof

Inside method:

```phalcom
class Runner {
    use<T>(_ value: Box<T>)
        where T |= Printable
    {
        ...
    }
}
```

Need:

```text
Box<T> |= Printable
```

Flow:

```text
conditional source
nested T |= Printable
    → assumed lexical evidence
construct derived Box<T> evidence
```

## B.3 Productive recursion

```text
Base |= A exact

impl<T> A for Box<T>
    where T |= A
```

Prove:

```text
Box<Box<Base>> |= A
→ Box<Base> |= A
→ Base |= A
```

Success.

## B.4 Self cycle

```text
impl<T> A for Box<T>
    where Box<T> |= A
```

Proof of `Box<Int> |= A` re-enters same canonical goal.

Result: recursive/blocked, not success.

## B.5 Mutual cycle

```text
Box<T> |= A if Box<T> |= B
Box<T> |= B if Box<T> |= A
```

Terminates non-success.

## B.6 Projection condition

```phalcom
impl<I, T> SomeTrait for Wrapper<I>
    where
        I |= Iterator,
        I::Item == T
{
}
```

Projection normalization must use the same active proof context.

---

# Appendix C — Conditional coherence matrix

| Source A | Source B | P3 result |
|---|---|---|
| same head, same conditions | duplicate/overlap reject |
| same head, different conditions | overlap reject |
| unconditional generic head + conditional same head | overlap reject |
| generic head + concrete specialization overlapping under C4 | preserve C4 rejection |
| distinct target heads | allowed if C4 says non-overlapping |
| distinct trait applications | follow C4 trait-ref coherence |
| source order reversed | same result |

Conditions do not define coherence partitions.

---

# Appendix D — Runtime evidence flow

```text
semantic proof:
    Box<User> |= Printable
        ↓
source impl<T>
        ↓
T = User
        ↓
nested User |= Printable
        ↓
exact nested evidence
        ↓
DerivedConformanceEvidence(Box<User>, Printable)
        ↓
compiler
        ├─ RuntimeTypeEnvironment recipe: T -> User
        └─ RuntimeConformanceEnvironment recipe:
             nested Printable requirement handles
        ↓
conditional witness activation
        ↓
InvokeTraitRequirement slots inside witness body
```

For generic caller:

```text
caller slot:
    T |= Printable
        ↓
DerivedConditional recipe
        ↓
outer Box<T> |= Printable evidence environment
        ↓
callee/witness execution
```

The VM never asks whether `T` conforms.

---

# Appendix E — P4 boundary

P3 stops after proving explicit conditional conformances under existing legal conformance heads.

P4 must decide the first-class mechanism for:

```text
Iterator ⇒ Iterable
```

Possible future mechanisms may include:

```text
blanket/derived conformance
supertrait implication
another explicit trait implication construct
```

P3 does not choose among them.

In particular, P3 must not silently make this legal if current C4 target-head rules forbid it:

```phalcom
impl<I> Iterable for I
    where I |= Iterator
{
    ...
}
```

The recursive proof engine should be general enough to support the eventual P4 decision without architectural replacement.
