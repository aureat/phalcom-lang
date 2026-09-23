---
id: LANG007.C3.P2
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
plan_revision: corrective-closeout-2026-09-23
depends_on:
  - LANG007.C3.P1
  - LANG007.C3.P2 current implementation
  - INC-C3-8
  - INC-C3-9
  - INC-C3-10
follows: LANG007.C3.P2-advanced-agent-handoff.md
supersedes: null
prepared_against_revision: 2ea4f3a171d8d6353497ad13882b4864bb322416
p2_predecessor_revision: 60a27f79d69aa3e79c22e3d19df221466c7db920
---

# LANG007.C3.P2 — Corrective Closeout Patch-Grade Implementation Plan

> **Execution status:** proposed corrective-closeout execution companion for the still-active `LANG007.C3.P2`.  
> **Repository anchor:** `main` at `2ea4f3a171d8d6353497ad13882b4864bb322416` (`fix semantic static domain signatures`).  
> **P2 predecessor anchor:** completed P1 at `60a27f79d69aa3e79c22e3d19df221466c7db920`.  
> **Intended executor:** GPT-5.6 Luna High/Extra High or an equivalent implementation agent operating under the existing Phalcom consultation protocol.  
> **Closeout target:** `LANG007.C3.P2` may finish `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; this plan does **not** close C3 and does **not** perform P3/release certification.

This document is a patch-grade execution plan for the remaining work inside the already-numbered P2. It does not create a new checkpoint, does not replace the canonical P2 specification, and does not reopen architecture already ratified under `INC-C3-8`, `INC-C3-9`, or `INC-C3-10`. Its purpose is to convert the current closeout report plus the repository audit at `2ea4f3a1` into a bounded sequence of implementation corrections, acceptance tests, incremental/import proofs, and lifecycle updates.

The decisive planning finding is that P2 is closer to closure than its historical handoff suggests, but T4 is not merely “implemented and unverified.” The query-context row coordinator exists and reflects the accepted architecture, yet three narrow correctness defects remain in production traversal/completion paths. Those defects must be corrected before T4 acceptance. T7 is architecturally advanced and has substantial focused evidence, but still requires a small set of explicit acceptance cases. T8 schema-v8 implementation exists and compiles, but the required validation, cross-module provider composition, exact invalidation, source identity, and cold/incremental equivalence evidence remains incomplete. T9 must remain gated on truthful T1–T8 acceptance.

---

## 0. Executor contract

This plan is designed for a capable implementation model working under constrained architectural authority.

The implementer:

- may adapt mechanical details to the live repository;
- must re-read the named live paths before editing;
- must preserve the architecture and invariants below;
- must not improvise material architecture;
- must not reopen accepted `AssociatedSurface`, static-case, row-identity, or cycle decisions;
- must follow the testing budget rather than testing reflexively;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when an escalation trigger fires;
- must keep `LANG007.C3-CHECKPOINT.md` current at coherent state transitions;
- must create the required P2 walkthrough and P2→P3 handoff only after P2 actually satisfies its acceptance conditions.

Testing is evidence gathering, not a ritual. Do not run tests merely because an edit occurred.

The executor should spend most implementation time in BUILD MODE. Broad verification is explicitly deferred until the named stabilization gates. A focused PASS proves the question that test was designed to answer; it is not an instruction to broaden testing immediately.

### 0.1 This plan's relationship to the canonical P2 plan

The canonical numbered plan remains:

```text
docs/implementation/LANG007/LANG007.C3/
  LANG007.C3.P2-generalized-static-domains-heterogeneous-application-and-associated-type-data.md
```

This closeout plan is subordinate to it. Where this document is more specific, it operationalizes verified live-repository facts or later ratified amendments. It does not supersede normative behavior or accepted decisions in the canonical plan.

If this document conflicts materially with:

1. an effective normative specification;
2. a later explicit user ruling;
3. a ratified consultation recorded after `2ea4f3a1`;

STOP AND CONSULT rather than silently choosing one.

---

## 1. Goal

Complete the remaining `LANG007.C3.P2` implementation and focused acceptance by correcting the three audited T4 row-inference defects, closing the remaining T7 acceptance gaps without redesign, proving schema-v8/cross-module/import/incremental T8 behavior, and then executing T9 stabilization and lifecycle closeout.

---

## 2. Checkpoint acceptance objective

This is a **corrective closure and stabilization plan for one active numbered plan**, not full checkpoint closure.

P2 is successful when all of the following are true:

```text
1. T4 heterogeneous first-order inference is correct and focused-tested,
   including mixed Term/RecordRow arguments and nested-frame row lifetime.

2. T5/T6 obligations are mapped to concrete focused evidence:
   static aliases, associated static-domain declaration/specialization,
   runtime-negative behavior, and no duplicate identity source.

3. T7's remaining all-annotation, static type-member convergence,
   heterogeneous associated-owner, collision, and exact invalidation
   obligations are accepted without reopening AssociatedSurface authority
   or the declaration-signature cycle rule.

4. T8 schema-v8 metadata survives round trip and validation; provider and
   consumer modules preserve stable static-domain/case identity; provider
   edits invalidate exactly the relevant consumers; source targets remain
   canonical; incremental results equal cold analysis.

5. T9 audits show no runtime static evaluator/invocation machinery,
   ProperType boundaries remain intact, all P2-introduced Applied consumers
   are intentionally heterogeneous-aware or explicitly type-only, and the
   type-lambda beta-reduction resource invariant remains satisfied.

6. P2 lifecycle state is truthful:
   status: COMPLETE
   completion: IMPLEMENTED
   verification: FOCUSED_TESTED

7. C3 remains IN_PROGRESS / PARTIAL because P3 still owns structural
   intrinsic exposure, migration, broad certification, and checkpoint closure.
```

This plan does **not** require workspace-wide release certification. P2 closure means focused semantic/type-meta/incremental evidence, not `RELEASE_COMPLETE`.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
live revision: 2ea4f3a171d8d6353497ad13882b4864bb322416
live commit: fix semantic static domain signatures
P2 predecessor / completed P1 revision:
  60a27f79d69aa3e79c22e3d19df221466c7db920
immediate predecessor to current static-case correction:
  5d8d66f91b978ac60e29eca9aac2569af9c7a071
```

The user reported the worktree clean at `2ea4f3a1`. That is not verifiable through the GitHub branch API. The implementer must run `git status --short` locally before editing and must treat the local result as authoritative.

### 3.1 Verified live repository facts

The following facts were re-established from the live repository during planning:

1. `main` is exactly `2ea4f3a171d8d6353497ad13882b4864bb322416`.
2. The static-case/kind-form slice missing from the older handoff is committed:
   - `StaticDomainCaseCatalogProduct` exists;
   - `StaticCaseId` is selector-shaped;
   - `StaticCaseSignatureProduct` exists;
   - semantic DB query/product/fingerprint integration exists;
   - checked case signatures participate in scheduling;
   - algebraic builders consume catalog/signature products;
   - singleton versus explicit-empty constructor identity is covered;
   - recursive static-domain payload behavior is covered;
   - mixed alias/static-case-signature SCC behavior is covered;
   - selected case signature edits have incremental/cold-parity tests.
3. `AssociatedSurface` remains the namespace selection/collision authority.
4. Static case membership is catalog-owned; checked constructor contract is signature-owned; TypeStore is an interner/materializer rather than source case authority.
5. `KindSyntax::StaticDomain { form: Box<TypeAnnotation>, ... }`-style behavior is represented by the current correction path; associated static-domain kind resolution now reuses type-form resolution and retains ordinary owner application semantics.
6. T4 query-context coordination exists:
   - `InferenceCoordinator` owns `InferenceSession` plus a distinct `RecordRowSolver<InferenceTerm>`;
   - row variables remain `RecordRowVarId`;
   - ordinary variables remain `InferVarId`;
   - `InferenceTerm::Applied` and `InferenceTerm::StaticData` use `InferenceStaticArgument::{Term, RecordRow}`;
   - row same-label payload pairs are fed back to ordinary inference;
   - cross-domain row payload occurs checks exist;
   - child inference can retain a same-context symbolic result.
7. T4 currently has three audited correctness gaps:
   - declaration-generic parameter reachability ignores row tails / heterogeneous row arguments / static data;
   - stable row-lacks discovery ignores `StaticArgument::RecordRow` in generalized applications and static data;
   - coordinated row root completion checks direct substitution presence rather than transitive unresolved tails and is not itself frame-aware.
8. Schema v8 exists:
   - `TYPE_METADATA_SCHEMA_VERSION = 8`;
   - a typed static-domain declaration/case catalog exists;
   - heterogeneous `StaticArgumentRef::RecordRow` exists;
   - validation checks owner arity/kinds, case membership/payload kinds, and row structure;
   - snapshot-backed `MetadataExporter` consumes canonical algebraic and associated declaration products.
9. Existing schema compatibility tests do not yet provide a dedicated v8 acceptance matrix.
10. Existing row-focused tests cover basic row/type identity, simple remainder solving, ordinary row-polymorphic calls, expected-result row inference, simple underconstraint, and one nested generic/closure case. They do not establish the full INC-C3-8 amended acceptance contract.
11. Existing `incremental::type_members` tests already establish significant T7/static-case evidence, including:
    - field/alias/generic-header type-member projection availability;
    - nominal header edit/cold parity;
    - singleton versus empty constructor;
    - recursive static payload;
    - alias/static-case-signature SCC;
    - associated generic static-case signature SCC;
    - static case rename parity;
    - associated static-case alias;
    - alias/type-member cycle;
    - collision rejection;
    - wrong user static-domain argument;
    - associated case edit parity;
    - associated/inherent-impl signature edit parity and sibling reuse.
12. Existing imported-resolution tests include provider identity for an imported nominal static case before runtime rejection, but not the full provider edit/cold parity matrix required by T8.
13. Production compiler metadata export already attaches:
    - project-universe stable identity;
    - algebraic semantics;
    - associated static-domain products;
    - aliases;
    before constructing the metadata bundle.
14. The P1 handoff records the P2 predecessor revision `60a27f79...`; use that revision as the final P2 diff/audit baseline rather than scanning unrelated historical runtime code.

> **Mandatory:** re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read these in order. Do not broadly re-audit the repository.

```text
1. AGENTS.md

2. docs/workflow/implementation-record-lifecycle-convention.md

3. docs/workflow/luna-patch-grade-plan-schema.md
   or the supplied luna-patch-grade-plan-schema.md

4. docs/workflow/sol-principal-architect-project-doctrine.md
   or the supplied sol-principal-architect-project-doctrine.md

5. docs/implementation/LANG007/LANG007.C3/
   LANG007.C3.P2-generalized-static-domains-heterogeneous-application-and-associated-type-data.md

6. docs/implementation/LANG007/LANG007.C3/
   LANG007.C3-CHECKPOINT.md

7. docs/implementation/LANG007/LANG007.C3/
   LANG007.C3.P2-advanced-agent-handoff.md

8. docs/implementation/LANG007/LANG007.C3/
   INC-C3-8.md

9. docs/implementation/LANG007/LANG007.C3/
   INC-C3-9.md

10. docs/implementation/LANG007/LANG007.C3/
    INC-C3-10.md

11. effective specifications explicitly governing the open work:
    docs/specs/type-level-domains.md
    docs/specs/type-members.md
    relevant generic/type-application and record-row specification
    relevant impl/data-object specification only if a T6 regression requires it

12. exact source paths named by the tasks below.

13. exact test files named by the tasks below.
```

Before each task, inspect only the small live seam named for that task. Avoid broad exploration unless a failing discriminator proves the seam is wider.

---

## 5. Normative authority

### 5.1 Desired language behavior

The effective specifications plus ratified project decisions own observable semantics. In particular:

- static domains are distinct kinds, not proper `Type`;
- `RecordRow` remains a distinct kind and identity space;
- static terms do not become runtime values;
- associated static domains and type members participate in one associated namespace authority;
- declaration-signature cycles follow the ratified T7 rule;
- static-case catalog identity is distinct from checked constructor contract;
- incremental semantics must agree with cold analysis.

### 5.2 Ratified implementation architecture

The following decisions are already settled and are normative for implementation:

#### INC-C3-8 — mixed row/type inference

```text
InferenceContext
    owns
InferenceCoordinator
    ├── InferenceSession / InferVarId
    └── RecordRowSolver<InferenceTerm> / RecordRowVarId
```

The solvers coordinate but their identities do not merge. Same-label row payloads become ordinary inference obligations. Query-local row variables may persist across nested inference frames but never escape into canonical products.

#### INC-C3-9 — schema v8

Schema v8 owns typed static-domain declaration/case catalog validation. Snapshot-backed export uses canonical declaration products. Older supported schema behavior remains versioned rather than silently upgraded by assuming v8 invariants.

#### INC-C3-10 — static case catalog/signature split

```text
StaticDomainCaseCatalogProduct
    owns source case membership + selector/shape identity

StaticCaseSignatureProduct
    owns checked constructor payload contract

AssociatedSurface
    owns associated namespace admission/selection

TypeStore
    owns canonical interning/materialization only
```

The catalog is not a signature SCC node. Constructor-shaped cases require checked signatures; singleton getters do not. `StaticCaseSignature` participates in the existing declaration-signature graph.

### 5.3 Current implementation

Live source at `2ea4f3a1` is implementation reality. It may contain defects. A defect does not change the architecture above.

### 5.4 Future/non-normative work

The following are not P2 architectural obligations:

- replacing full-graph signature edge/SCC reconstruction with a more incremental algorithm;
- P3 structural intrinsic exposure;
- Universe/corpus/tooling migration;
- release certification;
- broad runtime reflection polish;
- unrelated metadata-import architecture not already present in the P2 path.

---

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Current symbol/path | Owner | Durable invariant |
|---|---|---|---|
| static application argument | `types::store::StaticArgument::{Term, RecordRow}` | TypeStore/type semantics | row arguments never masquerade as `TypeId` |
| inference static argument | `checker::inference::InferenceStaticArgument::{Term, RecordRow}` | query inference | heterogeneous children remain structurally visible |
| ordinary inference variable | `InferVarId` / `InferenceVariable.kind` | `InferenceSession` | one kind-aware ordinary inference identity space |
| row inference variable | `RecordRowVarId` | row solver | separate identity; never durable |
| mixed coordinator | `InferenceCoordinator` | query inference context | owns both solvers for context lifetime |
| row payload solver | `RecordRowSolver<InferenceTerm>` | row shape solver | row solver owns labels/tails, not type equality |
| row stable lacks | `StableRecordRowLack` | semantic checker | stable parameter identity only; no row solver IDs |
| static case catalog | `StaticDomainCaseCatalogProduct` | declaration semantics | canonical membership/shape identity |
| static case contract | `StaticCaseSignatureProduct` | declaration-signature graph | checked constructor payload kinds |
| associated namespace | `AssociatedSurface` | semantic declaration surface | one selection/collision authority |
| static case ID | `StaticCaseId { domain, selector }` | semantic identity | singleton versus `Foo()` remains distinct |
| associated static domain | declaration + canonical owner environment | TypeStore/semantic declaration | no rendered-owner-string identity |
| schema version | `TYPE_METADATA_SCHEMA_VERSION = 8` | `phalcom-type-meta` | v8 catalog semantics are explicit |
| stable static domain metadata | `StaticDomainRef` | metadata | raw semantic IDs are not serialized |
| stable heterogeneous arg | `StaticArgumentRef::{Term, RecordRow}` | metadata | row category survives persistence |
| production metadata export | `MetadataExporter` plus canonical product attachments | semantic/compiler boundary | snapshot products, not TypeStore history, own declarations |
| source case resolution | `AssociatedResolutionKind::StaticTerm` + `StaticCaseId` | semantic resolution/source index | runtime illegality may coexist with canonical source target |

### 6.2 Existing focused evidence that should be inherited, not rediscovered

The executor should treat the following as established unless the live tree contradicts it:

- static-case signature query/product architecture exists;
- recursive static-domain payloads are legal;
- singleton and explicit-empty constructor identities differ;
- static case signature edits can rebuild selected products while retaining unrelated sibling signatures;
- associated and inherent-impl static case signature edits have cold parity;
- nominal static-case source selection preserves canonical target before runtime rejection;
- `AssociatedSurface` remains the only associated name collision/selection authority;
- metadata schema v8 structures compile.

Do not rewrite these mechanisms to solve T4/T8 acceptance.

---

## 7. Architecture

### 7.1 End-to-end direction of authority

```text
source syntax
    ↓
declaration shells / checked declaration products
    ↓
canonical TypeStore terms + stable declaration identities
    ↓
query-local inference coordinator
    ├── ordinary kind-aware inference
    └── distinct row-shape inference
    ↓
canonical solved semantic terms
    ↓
snapshot / stable dependency products
    ↓
metadata export + source-index products
    ↓
compiler/runtime consumers only where proper/runtime types are legal
```

No downstream consumer reconstructs static membership, case identity, type-member selection, or row inference authority.

### 7.2 T4 correction architecture

T4 fixes are traversal/completion corrections inside the accepted coordinator design.

#### Reachability

Generic declaration parameters are considered “used by this callable signature” if they occur anywhere in the canonical semantic structure, including:

```text
TypeData::Parameter
TypeData::Applied
    StaticArgument::Term
    StaticArgument::RecordRow
TypeData::StaticData
    associated owner environment
    constructor arguments
TypeData::Record
    field payloads
    Parameter tail
    Projection tail
nested callable/tuple/union/pack/projection forms
```

A row parameter appearing only as `RecordRowTail::Parameter(R)` is still a semantic parameter occurrence.

#### Stable row lacks

A canonical row:

```text
{ a: A, b: B | R }
```

publishes stable facts:

```text
R lacks "a"
R lacks "b"
```

regardless of whether the row appears directly, inside `Applied`, inside `StaticData`, or in an associated static-domain owner environment.

#### Frame-aware completion

A row root is not complete merely because `substitutions.contains_key(root)`.

Normalize the root:

```text
root R
  -> fields + tail
```

and classify its terminal tail:

```text
Closed / Parameter / Projection
    complete

Var(V) owned by ancestor frame
    not locally underconstrained;
    retain symbolic ancestor dependency

Var(V) owned by current frame
    underconstrained

Var(V) with no frame owner (solver-internal fresh residual)
    underconstrained
```

A child result that depends on the unresolved ancestor must remain symbolic and must not be materialized. Later parent evidence may solve the same shared row variable.

### 7.3 T7 closeout architecture

T7 work is acceptance closure around the existing graph/surface architecture.

```text
declaration shell
    ↓
AssociatedSurface admission
    ↓
selected stable declaration/member identity
    ↓
checked declaration/signature product
    ↓
annotation/expression consumer
```

The graph schedules products and propagates invalidation. It is not a lookup table.

### 7.4 T8 closeout architecture

Persistence and import identity flow must remain:

```text
canonical semantic declaration identity
    ↓
Stable*Ref metadata identity
    ↓
schema-v8 validation
    ↓
project/module consumer linkage by stable identity
```

Never:

```text
source spelling
    -> guessed case
    -> guessed provider
```

and never:

```text
TypeStore intern history
    -> current declaration membership
```

### 7.5 T9 audit architecture

Use the P2 predecessor revision as a bounded audit baseline:

```sh
git diff 60a27f79d69aa3e79c22e3d19df221466c7db920...HEAD
```

Audit P2-introduced changes rather than interpreting every historical `StaticDomain`/metadata reference in the repository as P2 work.

---

## 8. Ownership boundaries

### 8.1 Owns

This closeout plan owns:

- T4 traversal/completion correctness;
- T4 focused acceptance;
- remaining T7 focused acceptance and local corrections required by that evidence;
- T8 schema-v8 validation/round-trip evidence and local fixes;
- T8 cross-module provider identity, invalidation, source-index target, and cold parity;
- final P2 coverage reconciliation;
- T9 runtime/proper-type/Applied-consumer/resource audits;
- P2 checkpoint, walkthrough, and handoff truth.

### 8.2 Does not own

This plan does not own:

- a new generic inference algorithm;
- row identity redesign;
- a new associated lookup registry;
- static-case architecture redesign;
- a new metadata importer subsystem if one is not already part of the active P2 architecture;
- full declaration-graph performance redesign;
- P3 structural intrinsic exposure;
- Universe migration;
- broad standard-library repair;
- release certification;
- unrelated formatter cleanup;
- unrelated semantic library baseline initializer repair.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| generic parameter occurrence in canonical term | structural semantic term traversal | call generic-domain selection | type-only `.as_type_id()` filter |
| row prefix lacks facts | semantic row structure | row solver binding setup | inferred from diagnostics later |
| query-local row identity | `RecordRowSolver` + coordinator frame map | inference | TypeStore/metadata row variable |
| row completion status | coordinator + normalized row state + frame ownership | call result publication | shallow substitution-presence check |
| associated name collision | `AssociatedSurface` | annotation/expression lookup | per-category fallback search |
| static case membership | static case catalog | annotation/expression/builders | TypeStore constructor enumeration |
| static case checked payload | static case signature product | constructor materialization | catalog payload guess |
| stable static-domain declaration catalog | snapshot-backed metadata export | validator/runtime metadata | TypeStore history in snapshot mode |
| cross-module target identity | stable declaration/case identity | source index/import consumers | source spelling reconstruction |
| incremental dependency | semantic query/declaration graph | workspace update | broad workspace invalidation |
| runtime value legality | proper/runtime type boundary | compiler/runtime | treating static term as runtime object |

---

## 9. Global invariants

### T4 / inference

```text
INV-01 — InferVarId and RecordRowVarId remain distinct identity spaces.

INV-02 — No RecordRowVarId enters TypeId, metadata, snapshot-stable identity,
         or source-index identity.

INV-03 — InferenceStaticArgument preserves Term versus RecordRow category.

INV-04 — Row shape solving owns labels/tails only; same-label field payload
         semantics are ordinary inference obligations.

INV-05 — Generic parameter reachability is structural over all heterogeneous
         static arguments and row tails.

INV-06 — Stable row-lacks discovery is structural over all heterogeneous
         static arguments and associated owner environments.

INV-07 — A row root is complete only if its normalized terminal dependency is
         concrete/rigid or belongs to an unresolved ancestor explicitly retained
         symbolically.

INV-08 — Child inference never closes an unresolved ancestor row to an empty row.

INV-09 — Cross-domain occurs checking works independent of constraint insertion
         order.

INV-10 — No constructor search, backtracking, or static subtype lattice is added.
```

### T7

```text
INV-11 — AssociatedSurface remains the only associated namespace admission,
         collision, and selection authority.

INV-12 — Declaration-signature graph scheduling is not a name lookup authority.

INV-13 — StaticDomainCaseCatalogProduct owns case identity; checked signature
         owns constructor payload contract.

INV-14 — Cycles containing Member or StaticCaseSignature use the ratified single
         TypeSignatureCycle component rule.

INV-15 — Generic associated owner specialization retains canonical heterogeneous
         owner arguments without rendered strings.
```

### T8

```text
INV-16 — Schema-v8 static-domain metadata uses stable declaration identity,
         never raw StaticDomainId / TypeId / store constructor ID.

INV-17 — Snapshot-backed catalog export consumes current declaration products,
         not TypeStore historical interning.

INV-18 — Supported pre-v8 bundles retain their documented validation floor;
         v8 invariants are not silently imposed on old schemas.

INV-19 — Cross-module provider changes invalidate exact semantic consumers and
         incremental final state equals cold final state.

INV-20 — Source-index static-domain/case targets derive from canonical semantic
         selection, not spelling.
```

### T9

```text
INV-21 — Static domains and static terms do not create runtime evaluator,
         runtime constructor-object, runtime message-dispatch, or VM bytecode
         semantics.

INV-22 — ProperType remains required at runtime typing/layout/subtyping/dispatch
         boundaries.

INV-23 — Every P2-affected TypeData::Applied consumer is explicitly classified
         as heterogeneous-aware or intentionally proper-type-only.

INV-24 — Type-lambda beta reduction does not clone the complete type/static arena.

INV-25 — P2 closure truth is COMPLETE / IMPLEMENTED / FOCUSED_TESTED only after
         actual focused evidence; C3 remains open for P3.
```

---

## 10. Non-goals

Do not expand this plan into:

- replacing the retained declaration dependency graph because full edge/SCC traversal is currently global;
- changing accepted `AssociatedSurface` collision semantics;
- reworking `StaticCaseId`;
- restructuring the static case catalog/signature split;
- adding `StaticInferVarId`;
- collapsing `RecordRow` into Type;
- adding row/static unification via fake canonical TypeIds;
- adding a runtime representation for static terms;
- adding a generic “metadata importer” merely because the word “import reconstruction” appears in the plan; first verify the live canonical import seam;
- changing the source grammar for RecordRow static arguments without an effective specification;
- P3 migrations;
- broad formatting cleanup;
- broad workspace test/clippy repair;
- unrelated trait/default-body/Universe fixes.

---

## 11. Expected impact map

### 11.1 Expected source areas

#### T4 corrections

```text
phalcom-semantic/src/checker/call.rs
    generic parameter structural reachability

phalcom-semantic/src/checker/row_inference.rs
    stable row-lacks traversal over heterogeneous arguments

phalcom-semantic/src/checker/inference.rs
    coordinator row completion / frame-aware root classification

phalcom-semantic/src/types/row_solver.rs
    possibly a small normalized-root inspection helper;
    do not move frame ownership into the generic row solver unless mechanically
    unavoidable and architecturally neutral

phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/call.rs
    only if API plumbing is required for frame-aware row completion
```

#### T7 acceptance/local corrections

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/checker/type_member_declaration.rs
phalcom-semantic/src/associated.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/static_case.rs
```

Touch these only if a focused T7 discriminator exposes a live defect.

#### T8 acceptance/local corrections

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/declaration.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/bundle.rs
phalcom-type-meta/src/validate.rs

phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/source_index/*
phalcom-modules/src/declaration.rs
phalcom-modules/src/interface.rs
```

Do not modify all of these by default. The acceptance tests decide which seam is actually incomplete.

#### T9 audits/lifecycle

```text
phalcom-core/src/modules/compile.rs
phalcom-core/src/modules/semantic_lowering.rs
other P2-touched compiler/runtime files found in the baseline diff

docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
```

### 11.2 Expected tests

```text
phalcom-semantic/tests/semantic/foundations/record_row_inference.rs
phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs
phalcom-semantic/tests/semantic/foundations/static_domain_inference.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs

phalcom-semantic/tests/semantic/incremental/type_members.rs
phalcom-semantic/tests/semantic/adts/associated/*
phalcom-semantic/tests/semantic/associated/invocation.rs

phalcom-type-meta/tests/schema_compat.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/incremental/adts.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs
```

### 11.3 Expected documentation/state

```text
LANG007.C3-CHECKPOINT.md
LANG007.C3.P2-walkthrough.md
LANG007.C3.P2-handoff.md
possibly one concise incident/amendment record only if a STOP/CONSULT trigger fires
```

### 11.4 Unexpected-touch rule

Touching adjacent private helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem when that entry changes architecture, ownership, stable identity, lifetime, metadata compatibility policy, or observable semantics.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. separate `InferVarId` and `RecordRowVarId`;
2. query-context ownership of both solvers;
3. `InferenceStaticArgument::{Term, RecordRow}`;
4. `AssociatedSurface` as sole associated namespace authority;
5. static case catalog versus checked signature ownership split;
6. static-case cycle classification ratified in INC-C3-10;
7. schema-v8 catalog identity based on stable declarations;
8. static/runtime separation;
9. ProperType proof boundary;
10. exact/cold incremental equivalence requirement;
11. P2 final classification target (`FOCUSED_TESTED`, not release-certified);
12. T9 precondition that T1–T8 are accepted first.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- private helper names;
- whether a traversal is implemented as free functions or a small local walker struct;
- exact internal collection types;
- helper placement within the owning module;
- test function names;
- fixture decomposition;
- mechanical borrow/lifetime fixes;
- whether frame-aware root inspection returns an internal helper enum or a tuple, provided external semantic behavior and ownership remain unchanged;
- exact test filter spelling after using `-- --list`.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| HEAD is still `2ea4f3a1` or a mechanical descendant | `git rev-parse HEAD`, recent log | adapt to later committed corrections; do not reapply them |
| worktree is clean / protected paths are known | `git status --short` | preserve user work; scope diff |
| `type_contains_any_parameter` still ignores row tails/static row args | `checker/call.rs` | if already fixed, retain and test; do not duplicate |
| stable lacks traversal still filters via `as_type_id()` | `checker/row_inference.rs` | if fixed, retain and test |
| root completion still checks `substitutions.contains_key(root)` | `types/row_solver.rs`, coordinator | if fixed, inspect semantics before changing |
| `solve_rows_for_query` roots local row bindings | `checker/row_inference.rs` | adapt plumbing mechanically |
| schema version remains 8 | `phalcom-type-meta/src/header.rs` | if newer, inspect later migration before applying v8-specific edits |
| snapshot-backed exporter still attaches algebraic/associated products | `phalcom-core/src/modules/compile.rs`, exporter | if changed materially, STOP AND CONSULT |
| no general semantic MetadataImporter exists | targeted search | if one exists, use it for T8 import acceptance |
| all-annotation callable-signature projection coverage is still absent | T7 tests | if present, do not duplicate |
| heterogeneous associated-owner row coverage is absent | T7 tests | if present, reuse evidence |
| formatter drift remains unrelated baseline | checkpoint + `git diff --name-only` | if a changed file causes failure, it becomes active responsibility |
| semantic library initializer failure remains baseline | checkpoint/P1 handoff | if failure changes or touches modified initializer, reclassify |

---

## 13. Global STOP / CONSULT triggers

The implementer must stop editing and build a consultation packet if any of the following occurs:

1. the accepted coordinator architecture appears incapable of expressing the T4 fix;
2. row completion would require merging row identity into ordinary inference;
3. a canonical row solver must learn frame ownership globally rather than receiving/querying it at the coordinator boundary and the choice has architectural consequences;
4. parameter reachability cannot be made structurally exhaustive without changing generic ownership semantics;
5. a T7 test appears to require another associated lookup registry or precedence chain;
6. a T7 test would require changing the ratified SCC rule;
7. heterogeneous associated-owner row semantics require new source grammar not defined by an effective spec;
8. T8 needs stable identity that cannot be expressed by existing stable declaration/project/module references;
9. T8 correctness appears to require a whole-workspace scan instead of an exact dependency edge;
10. “import reconstruction” appears to require inventing a new metadata-to-semantic importer subsystem absent from the current architecture;
11. supported pre-v8 compatibility behavior is ambiguous after reading the existing compatibility code/tests;
12. a runtime static evaluator/object/invocation target appears necessary;
13. a ProperType boundary would need weakening;
14. incremental and cold results still disagree after one serious correction for the same root cause;
15. the same nontrivial semantic failure survives the allowed correction budget;
16. a test oracle conflicts with an effective specification;
17. two plausible fixes have materially different ownership/identity/lifetime implications;
18. an invariant in §9 would need weakening to pass tests;
19. patch scope becomes materially more cross-cutting than planned.

A triggered consultation cannot be self-waived.

### 13.1 Consultation packet format

Collect cheaply:

```text
Plan/task:
Live revision:
Observed:
Expected:
Smallest reproducer:
Relevant source path/symbol:
Relevant current data flow:
Hypothesis:
Evidence for hypothesis:
Attempted serious correction, if any:
Result:
Architectural question:
Invariants that must remain true:
Two plausible options, if applicable:
```

Ask the adviser for a decision, not a repository rediscovery.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress.

A cycle is:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

Examples:

- renamed helper;
- import mismatch;
- borrow error;
- test fixture syntax mismatch;
- wrong filter.

### 14.2 Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious corrective attempt for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

### 14.4 No unchanged reruns

Never rerun an unchanged failing test unless code, fixture, environment, or another fact relevant to the failure changed.

---

## 15. Testing surface analysis

The following closeout coverage ledger supplements the canonical `CV-P2-*` ledger. Every closeout item maps back to one or more P2 obligations.

| ID | Dimension | Invariants | Required case |
|---|---|---|---|
| CL-01 | structural reachability | INV-05 | declaration row generic referenced only by record tail is recognized |
| CL-02 | heterogeneous reachability | INV-05 | parameter inside `StaticArgument::RecordRow` under `Applied` is recognized |
| CL-03 | static-data reachability | INV-05, INV-15 | parameter inside static-data payload or associated owner environment is recognized |
| CL-04 | stable row lacks | INV-06 | nested row argument inside `Applied` produces prefix lacks |
| CL-05 | stable row lacks | INV-06 | nested row argument inside `StaticData` / associated owner specialization produces prefix lacks |
| CL-06 | shared-field inference | INV-04 | equal label emits ordinary inference equality and solves field type |
| CL-07 | tail extension | INV-04 | inferred payload carried by row extension materializes after ordinary inference |
| CL-08 | row conflict | INV-04 | conflicting same-label payloads reject semantically |
| CL-09 | direct underconstraint | INV-07 | unsolved local root remains underconstrained |
| CL-10 | transitive underconstraint | INV-07 | root with substitution ending in unsolved local/internal tail remains underconstrained |
| CL-11 | ancestor dependency | INV-07, INV-08 | child result depending on ancestor row remains symbolic, no child-local underconstraint |
| CL-12 | parent continuation | INV-08 | later parent evidence resolves retained symbolic child result |
| CL-13 | occurs order A | INV-09 | ordinary term reaches row before row bind; recursive bind rejects |
| CL-14 | occurs order B | INV-09 | row bind exists before ordinary term reaches row; recursive ordinary bind rejects |
| CL-15 | mixed Applied | INV-01–04 | `Term` + `RecordRow` applied arguments unify without category collapse |
| CL-16 | mixed StaticData | INV-01–04 | static constructor with heterogeneous payload/owner arguments unifies/refutes correctly |
| CL-17 | ordinary control | INV-10 | existing proper-Type and non-row static inference unchanged |
| CL-18 | all annotations | INV-11–14 | type-member/static-domain forms available in callable parameter + return annotations |
| CL-19 | type-member convergence | INV-11 | static-domain type-member parameter/result still use checked declaration product selected by surface |
| CL-20 | associated owner row | INV-15 | associated static-domain owner specialization preserves a RecordRow argument |
| CL-21 | collision invalidation | INV-11, INV-19 | behavior/inherent-impl edit introduces/removes associated collision with cold parity |
| CL-22 | exact T7 reuse | INV-12, INV-19 | selected dependent rebuilds; unrelated sibling retained |
| CL-23 | schema-v8 validation | INV-16–18 | current v8 catalog validates and malformed membership/kind cases reject |
| CL-24 | old schema floor | INV-18 | supported pre-v8 bundle retains its historical behavior without mandatory v8 catalog |
| CL-25 | metadata round trip | INV-16–18 | static domain, static term, heterogeneous Applied row, alias, associated specialization survive encode/decode/validate |
| CL-26 | scoped metadata | INV-16–18 | scoped bound static term / heterogeneous application survives schema-v8 graph |
| CL-27 | provider identity | INV-19–20 | consumer selection points at provider static declaration/case identity |
| CL-28 | provider payload edit | INV-19 | provider case payload edit recomputes exact dependent and matches cold |
| CL-29 | provider rename | INV-19–20 | provider case rename invalidates selection/source target and matches cold |
| CL-30 | associated provider edit | INV-19 | associated static case/owner environment edit propagates across module boundary |
| CL-31 | source index | INV-20 | canonical target survives legal runtime-negative context and provider identity |
| CL-32 | trait boundary | INV-21–22 | unsupported static-domain trait use yields targeted diagnostic, not evidence corruption |
| CL-33 | ProperType audit | INV-22 | runtime typing/layout/subtyping/dispatch boundaries reject static terms |
| CL-34 | Applied consumer audit | INV-23 | no P2-touched consumer retains accidental `Vec<TypeId>` assumption |
| CL-35 | resource | INV-24 | beta reduction has no whole-arena clone path |
| CL-36 | final focused gate | INV-25 | canonical P2 G5 + narrow added gates pass |

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Sparse discriminators only. No workspace ritual.

#### STABILIZE MODE

Run focused feature suites after a coherent workstream is complete.

#### CERTIFY MODE

Not part of ordinary P2 execution. Only enter if the user explicitly requests broader certification.

### 16.2 Verification ladder

```text
T0 — exact new regression / exact unit discriminator
T1 — directly affected focused module
T2 — owning semantic/type-meta subsystem filters
T3 — cross-module/import/source-index integration
T4 — broad crate/package verification, only near closeout if required
T5 — workspace/release verification, deferred
```

### 16.3 Mandatory during BUILD

For T4 corrections:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
```

Run this only after the coherent T4 code slice compiles enough for the result to be informative.

Then run exact new regressions individually.

For schema-v8 code corrections:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
```

Only if source code changed. If only tests were added, go directly to the exact test.

### 16.4 Mandatory at named gates

Defined in §19.

### 16.5 Mandatory during STABILIZE

At minimum:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_row_inference -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::record_row_polymorphism -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_members -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::imported_resolution -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::adts -- --nocapture
```

Adjust exact module path spelling only after `-- --list` confirms it.

### 16.6 Only if evidence demands

```text
- full phalcom-semantic test target
- broad phalcom-core tests
- phalcom-modules full suite
- LSP tests
```

### 16.7 Explicitly deferred / DO NOT RUN during BUILD

Do not run:

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Do not run full unrelated language corpora.

Do not repeatedly run `cargo fmt --all -- --check` during BUILD. It is a final T9 evidence item and is already known to expose unrelated historical drift.

Reason: these commands have low discriminatory value for the bounded remaining work and can consume implementation time on unrelated failures.

---

## 17. Baseline / unrelated failure policy

Classify failures:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: one bounded classification pass. If nonblocking and still unclear, record and continue.
- D: record and continue immediately.
- Do not weaken assertions to make a broad gate green.
- Do not consume adviser compute on D unless it blocks acceptance.

### 17.1 Known baseline/deferred items

At planning time:

1. repository-wide formatter check has recorded pre-existing drift outside the active P2 slice;
2. the semantic library unit-test initializer failure involving missing `generic_binder_header` in a legacy test initializer is a P1-recorded baseline;
3. full declaration-signature graph edge/SCC reconstruction remains a performance follow-up, not a correctness reason to replace the accepted dependency model.

Reclassify only if a P2-changed file is implicated by fresh evidence.

---

# 18. Tasks

## T1 — Preflight, state reconciliation, and closeout activation

### Purpose

Establish a truthful local starting state and prevent stale handoff metadata from causing duplicate implementation.

### Preconditions

- access to the live repository checkout;
- no code edits yet.

### Consumes

- live branch;
- P2 canonical plan;
- checkpoint;
- handoff;
- `INC-C3-8/9/10`;
- this closeout plan.

### Produces

- verified local HEAD/status;
- scoped list of P2-related modified/untracked paths, if any;
- checkpoint note that P2 closeout work is active, if the repository records active execution state at this point.

### Files and symbols

**Read:**

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
LANG007.C3.P2 canonical plan
LANG007.C3-CHECKPOINT.md
LANG007.C3.P2-advanced-agent-handoff.md
INC-C3-8.md
INC-C3-9.md
INC-C3-10.md
```

### Required implementation shape

Run:

```sh
git rev-parse HEAD
git status --short
git diff --stat
git diff --cached --stat
```

Expected baseline:

```text
HEAD == 2ea4f3a171d8d6353497ad13882b4864bb322416
```

If HEAD is later, inspect only the commits after `2ea4f3a1` and determine whether they already implement any task in this plan. Do not reapply completed work.

If the worktree is not clean, classify every unrelated path as protected user/concurrent work. Do not reset, restore, clean, broadly stage, or broadly format.

Inspect the three T4 blocker seams before editing:

```text
checker/call.rs::type_contains_any_parameter
checker/row_inference.rs::collect_stable_record_row_lacks_inner
types/row_solver.rs::solve_shapes_for_roots
checker/inference.rs::InferenceCoordinator::row_solution_for_roots
checker/row_inference.rs::GenericApplicationSession::solve_rows_for_query
```

### Forbidden approaches

- no implementation before status reconciliation;
- no broad repository scan;
- no reimplementation of static-case work already present;
- no staging unrelated paths.

### Tests to run now

None.

> **Do not run tests in T1. The purpose is repository-state truth, not behavior verification.**

### Acceptance

T1 passes when the implementer can state:

```text
live revision
protected worktree paths
which T4 blockers still exist
which later commits, if any, already cover planned work
```

### Local STOP / CONSULT triggers

- live tree contains material architecture beyond this plan that changes T4/T8 ownership;
- another active diff overlaps the exact T4 files in a semantically incompatible way.

### Checkpoint update

Only update if the repository convention expects active-plan start bookkeeping. Do not create a second state document.

---

## T2 — Correct T4 heterogeneous parameter reachability and stable row-lacks traversal

### Purpose

Fix the two structural traversal defects that currently allow row-valued generic occurrences and stable prefix constraints to disappear before the accepted inference coordinator can solve them.

### Preconditions

- T1 complete;
- audited defects still present or equivalent live defects confirmed.

### Consumes

```text
TypeData
StaticArgument
StaticDomainData
RecordRowData / RecordRowTail
TypeParameterId
StableRecordRowLack
```

### Produces

- exhaustive parameter-use traversal over heterogeneous applications/static data/rows;
- exhaustive stable row-lacks traversal over heterogeneous arguments;
- no new source of semantic truth.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/checker/call.rs
  type_contains_any_parameter
  callable_signature_mentions_parameters
  effective_declaration_generics

phalcom-semantic/src/checker/row_inference.rs
  collect_stable_record_row_lacks
  collect_stable_record_row_lacks_inner
```

**Read for representation only:**

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/parameter.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/record_row_inference.rs
phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs
new module-local unit tests only if production source syntax cannot isolate the helper
```

### Required implementation shape

#### A. Replace type-only generic parameter reachability

The live defect is conceptually:

```rust
TypeData::Applied { origin, arguments } => {
    type_contains_any_parameter(store, *origin, parameters)
        || arguments.iter().any(|argument| {
            argument.as_type_id().is_some_and(|argument| {
                type_contains_any_parameter(store, argument, parameters)
            })
        })
}

TypeData::Record(row_id) => {
    store.record_row(*row_id)
        .fields
        .iter()
        .any(|field| type_contains_any_parameter(store, field.ty, parameters))
}
```

This misses:

```text
StaticArgument::RecordRow
RecordRowTail::Parameter
RecordRowTail::Projection
TypeData::StaticData
associated static-domain owner environment
```

Implement a structurally exhaustive walker.

Recommended private decomposition:

```rust
fn type_contains_any_parameter(
    store: &TypeStore,
    ty: TypeId,
    parameters: &[TypeParameterId],
) -> bool {
    let mut walker = ParameterUseWalker::new(store, parameters);
    walker.visit_type(ty)
}

struct ParameterUseWalker<'a> {
    store: &'a TypeStore,
    parameters: &'a [TypeParameterId],
    seen_types: HashSet<TypeId>,
    seen_rows: HashSet<RecordRowId>,
    seen_domains: HashSet<StaticDomainId>,
}

impl ParameterUseWalker<'_> {
    fn visit_type(&mut self, ty: TypeId) -> bool { ... }

    fn visit_static_argument(&mut self, arg: StaticArgument) -> bool {
        match arg {
            StaticArgument::Term(ty) => self.visit_type(ty),
            StaticArgument::RecordRow(row) => self.visit_row(row),
        }
    }

    fn visit_row(&mut self, row: RecordRowId) -> bool {
        let row = self.store.record_row(row);

        row.fields.iter().any(|field| self.visit_type(field.ty))
            || match row.tail {
                RecordRowTail::Closed => false,
                RecordRowTail::Parameter(parameter) => {
                    self.parameters.contains(&parameter)
                }
                RecordRowTail::Projection(projection) => {
                    self.visit_type(projection)
                }
            }
    }

    fn visit_static_domain(&mut self, domain: StaticDomainId) -> bool {
        match self.store.static_domain(domain) {
            Some(StaticDomainData::Associated { owner_arguments, .. }) => {
                owner_arguments.iter()
                    .copied()
                    .any(|arg| self.visit_static_argument(arg))
            }
            Some(StaticDomainData::ConstInt)
            | Some(StaticDomainData::Nominal { .. })
            | None => false,
        }
    }
}
```

Exact names and visited-set factoring are mechanically flexible.

The important semantic rule is:

> If a declaration-owned generic parameter occurs anywhere in the canonical structure of a callable parameter/result/constraint—including row tails or heterogeneous owner arguments—the declaration generic domain must not be discarded before inference.

For `TypeData::StaticData(term)`, visit:

```text
term.domain if associated specialization carries owner arguments
term.arguments
```

For `TypeData::Applied`, visit every `StaticArgument`, not only type-valued ones.

Do not treat `RecordRow` as a TypeId.

#### B. Make stable row-lacks traversal heterogeneous

Refactor the existing row handling into a reusable helper:

```rust
fn collect_stable_record_row_lacks_from_row(
    store: &TypeStore,
    row_id: RecordRowId,
    visited_types: &mut HashSet<TypeId>,
    visited_rows: &mut HashSet<RecordRowId>,
    emitted: &mut HashSet<(TypeParameterId, Box<str>)>,
    lacks: &mut Vec<StableRecordRowLack>,
)
```

Semantics:

```text
for row { f1: T1, f2: T2 | R }:
    emit R lacks f1
    emit R lacks f2

visit each field payload type

if tail is Projection(P):
    visit P for nested stable facts

if tail is Closed:
    no tail fact
```

Then add:

```rust
fn collect_stable_record_row_lacks_from_static_argument(...) {
    match arg {
        StaticArgument::Term(ty) => collect_..._inner(..., ty, ...),
        StaticArgument::RecordRow(row) => collect_..._from_row(..., row, ...),
    }
}
```

Use it for:

```text
TypeData::Applied.arguments
TypeData::StaticData.arguments
associated static-domain owner arguments reachable from StaticData/domain forms
```

Do not emit any `RecordRowVarId`.

#### C. Preserve exact semantics for projection tails

A stable row projection tail is not automatically an inference variable and is not inherently underconstrained. Traverse the projection for nested parameter/lacks facts where the canonical representation exposes them, but do not invent a lack on the projection itself.

### Forbidden approaches

- no `RecordRow -> TypeId` conversion;
- no special-case string inspection of row syntax;
- no duplicate callable-generic selection path;
- no “assume all declaration generics are used” workaround;
- no turning every associated owner parameter into a dependency regardless of structural use merely to make tests pass;
- no changes to `AssociatedSurface`.

### Test changes required

#### CL-01 — row-tail-only generic reachability

Add a production call-path case in the smallest existing row polymorphism fixture where a declaration-owned row generic is referenced only by a row tail in a callable signature.

The exact source form depends on the live generic-owner syntax. Prefer an existing class/data generic pattern over inventing syntax.

The test must fail on the old reachability walker and pass after the fix.

#### CL-02 — `Applied` with `RecordRow`

At a foundation level, construct a canonical `TypeData::Applied` containing:

```text
StaticArgument::Term(...)
StaticArgument::RecordRow(row with Parameter(R) tail)
```

and prove parameter-use traversal reaches `R`.

If the private helper cannot be observed through a natural public call, add a small module-local `#[cfg(test)]` test adjacent to the helper rather than making the helper public API.

#### CL-03 — `StaticData` / associated owner environment

Construct or source-form a static term whose associated static-domain specialization contains a row-valued owner argument. Prove the declaration generic is recognized.

If source syntax for direct RecordRow generic arguments is not established by an effective specification, use a semantic construction test. Do not invent syntax.

#### CL-04 / CL-05 — nested lacks

Add a row with a known prefix and parameter tail inside:

1. `Applied`;
2. `StaticData` or an associated static-domain owner environment.

Prove the stable lack is collected and a duplicate field extension is rejected by normal row inference.

### Tests to run now

First exact new helper regressions.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_row_inference -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::record_row_polymorphism -- --nocapture
```

Do not run broader semantic tests yet.

### Tests explicitly deferred

```text
static-domain full inference suite
T7 suites
metadata/import suites
workspace tests
```

### Acceptance

T2 passes when:

```text
- a row-tail-only parameter activates declaration generic inference;
- Applied and StaticData heterogeneous row arguments participate in parameter
  reachability;
- stable row-lacks facts are preserved through those same structures;
- no row solver identity enters stable semantic state;
- existing row-polymorphic tests remain green.
```

### Local STOP / CONSULT triggers

- structurally correct reachability causes previously intentionally-unused declaration generics to become active in a way that changes accepted language semantics;
- associated-domain owner argument traversal requires a new stable identity model;
- a source test requires undefined RecordRow generic argument syntax.

### Checkpoint update

After T2 acceptance, record only the durable invariant:

```text
T4 generic reachability and stable row-lacks traversal are heterogeneous over
StaticArgument and row tails; no Type-only filter remains in these two owners.
```

---

## T3 — Correct T4 transitive frame-scoped row completion

### Purpose

Ensure a coordinated row result is not classified solved merely because its root has a direct substitution when the normalized row still ends in an unresolved local or solver-internal tail.

### Preconditions

- T2 accepted;
- existing coordinator/frame architecture intact.

### Consumes

```text
InferenceCoordinator
InferenceFrameId
row_variable_frames
RecordRowSolver<InferenceTerm>
RecordRowSolution
GenericApplicationSession
SymbolicInferenceResult
```

### Produces

- transitive row-root completion classification;
- explicit ancestor-dependency tolerance;
- local/internal unresolved tails remain underconstrained;
- unchanged symbolic child→parent behavior.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/checker/inference.rs
  InferenceCoordinator::row_solution_for_roots
  row_variable_is_owned_by_ancestor_frame
  term_has_unresolved_ancestor_row

phalcom-semantic/src/checker/row_inference.rs
  GenericApplicationSession::solve_rows_for_query
  has_pending_ancestor_row_dependency
  build_instantiation_from_active_terms_in_query

phalcom-semantic/src/types/row_solver.rs
  normalize_term
  solve_shapes_for_roots
```

**Possibly mechanical plumbing:**

```text
phalcom-semantic/src/checker/call.rs
```

**Tests:**

```text
module-local inference/coordinator unit tests if useful
phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs
```

### Required implementation shape

#### A. Do not move frame semantics into generic row solver authority

`RecordRowSolver<V>` is a reusable shape solver. It does not own Phalcom inference frames.

Prefer coordinator-level completion:

```rust
pub(crate) fn row_solution_for_frame_roots(
    &self,
    frame: InferenceFrameId,
    roots: &[RecordRowVarId],
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> RecordRowSolveResult<InferenceTerm>
```

Exact name is flexible.

#### B. Solve shape state without shallow completion first

The current generic solver's root-completion check is effectively:

```rust
roots
    .iter()
    .filter(|variable| !self.substitutions.contains_key(variable))
```

That is insufficient for:

```text
R -> { extra: T | S }
S unresolved
```

At the coordinator boundary, inspect the normalized root:

```rust
let normalized = solver.normalize_term(&RecordRowTerm {
    fields: Box::new([]),
    tail: RecordRowTermTail::Var(root),
});
```

Then classify terminal tail.

Recommended conceptual helper:

```rust
enum RowRootCompletion {
    Complete,
    PendingAncestor(RecordRowVarId),
    Underconstrained(RecordRowVarId),
}

fn classify_row_root_completion(
    &self,
    frame: InferenceFrameId,
    solver: &RecordRowSolver<InferenceTerm>,
    root: RecordRowVarId,
) -> RowRootCompletion {
    let normalized = solver.normalize_term(&row_var_term(root));

    match normalized.tail {
        RecordRowTermTail::Closed
        | RecordRowTermTail::Parameter(_)
        | RecordRowTermTail::Projection(_) => RowRootCompletion::Complete,

        RecordRowTermTail::Var(tail)
            if self.row_variable_is_owned_by_ancestor_frame(tail, frame) =>
        {
            RowRootCompletion::PendingAncestor(tail)
        }

        RecordRowTermTail::Var(tail) => {
            RowRootCompletion::Underconstrained(tail)
        }
    }
}
```

An unresolved row with no frame entry is solver-internal and therefore underconstrained for the current root.

Do not default it to closed.

#### C. Keep ancestor dependency as solved-shape-but-symbolic

A child local root may be constrained to:

```text
{ childField: T | ParentR }
```

where `ParentR` belongs to an ancestor frame.

That is **not** a child-local underconstraint error. The row shape may be retained in the solution, but publication must remain symbolic while the result depends on `ParentR`.

Preserve the existing mechanism:

```text
row solution
    +
term_has_unresolved_ancestor_row(...)
    =>
row_pending_ancestor = true
    =>
publish SymbolicInferenceResult
    =>
do not materialize GenericInstantiation
```

Improve it only as necessary to align with transitive completion. Do not invent a second symbolic-result channel.

#### D. Root lists must remain frame/call scoped

`GenericApplicationSession::solve_rows_for_query` currently derives roots from the call's row-parameter bindings. Preserve that scoped behavior unless a focused test proves a required result root is missing.

Do not use all allocated row variables as completion roots.

Solver-internal fresh residual variables are dependencies of a requested root, not independent roots. Their unresolved presence is discovered by normalized traversal.

#### E. Preserve existing materialization failure as a safety net

`materialize_query_row_term` should continue to reject unsolved or recursive row variables. It is a final safety boundary, not the primary completion classifier.

### Forbidden approaches

- no global “all allocated rows must solve” check;
- no treating any unresolved tail as child underconstraint regardless of frame owner;
- no auto-closing a residual tail;
- no canonicalizing a symbolic ancestor row;
- no making `RecordRowSolver` own `InferenceFrameId`;
- no second nested-call solver.

### Test changes required

#### CL-09 — direct local underconstraint

Construct a local row root with no evidence. Expected:

```text
RecordRowSolveResult::Underconstrained
```

not `Solved`.

#### CL-10 — transitive residual underconstraint

Create row equations that produce:

```text
R = { a: A | S }
```

with `S` solver-internal/unowned and unresolved.

Expected:

```text
R is Underconstrained
```

even though `R` itself has a substitution.

This is the exact regression for the audited shallow-completion defect.

#### CL-11 — child retains ancestor dependency

Build a parent inference frame with an unresolved row `ParentR`.

Build a child frame whose local row result normalizes to:

```text
{ mapped: Bool | ParentR }
```

Expected:

```text
child ordinary inference may solve;
row shape is retained;
no child-local RecordRowInferenceUnderconstrained diagnostic;
result is same-context SymbolicInferenceResult;
no canonical GenericInstantiation is created for the row-dependent result.
```

Prefer a module-local coordinator test for exact ownership classification plus a source-level nested-call test for production publication.

#### CL-12 — parent later resolves child

Continue the same context with parent evidence:

```text
ParentR = { cached: Bool, name: String | Closed }
```

Expected:

```text
shared coordinator resolves the retained dependency;
final outer expression materializes a closed row with all fields;
no query-local IDs escape.
```

#### CL-13 / CL-14 — cross-domain occurs, both orderings

Test both:

```text
A.
ordinary T references row R
then attempt R := { x: T }
=> RecordRowOccursCheck

B.
row R contains payload T
then attempt ordinary T := F<RecordRow R>
=> ordinary/row cross-domain occurs rejection
```

Use direct coordinator construction if source syntax makes the cycle impossible to write naturally. These are soundness tests, not syntax tests.

### Tests to run now

Run the exact new coordinator tests first.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::record_row_polymorphism -- --nocapture
```

If module-local tests live in the library target, run only their exact names, not all library tests, because of the known unrelated initializer baseline.

### Tests explicitly deferred

T4 broad gate until T4 task T4 below.

### Acceptance

T3 passes when:

```text
- direct unresolved local roots are underconstrained;
- roots with unresolved local/internal terminal dependencies are underconstrained;
- ancestor-row dependencies are retained symbolically rather than rejected or
  materialized;
- later parent evidence can resolve the shared dependency;
- both cross-domain occurs insertion orders reject recursion.
```

### Local STOP / CONSULT triggers

- correct classification appears to require frame ownership inside `RecordRowSolver`;
- a child needs to publish a canonical row containing an unresolved ancestor;
- existing symbolic result lifecycle cannot retain the dependency without a new durable identity;
- the same child→parent test fails after one serious coordinator correction.

### Checkpoint update

Record the frame completion rule as a durable T4 invariant.

---

## T4 — Complete and accept the full T4 heterogeneous inference matrix

### Purpose

Prove that the corrected coordinator satisfies the original T4 static-domain inference contract plus the INC-C3-8 row amendment.

### Preconditions

- T2 accepted;
- T3 accepted.

### Consumes

All T4 implementation.

### Produces

- complete focused T4 evidence;
- T4 status may be reconciled to accepted.

### Files and symbols

**Tests primarily:**

```text
phalcom-semantic/tests/semantic/foundations/static_domain_inference.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/foundations/record_row_inference.rs
phalcom-semantic/tests/semantic/integration/record_row_polymorphism.rs
```

Modify source only if a new test exposes a local correctness defect inside the accepted architecture.

### Required implementation/test shape

Ensure the combined T4 matrix proves:

#### Static-domain first-order behavior

```text
Vector(N) vs Vector(3) => N = 3
Vector(N) vs Scalar => mismatch
Shape::Vector(3) vs OtherShape::Vector(3) => mismatch
nested static constructor recursive unification
wrong-domain candidate rejected
underconstrained static variable remains underconstrained
static argument participates in generic application
```

Most of this exists in `foundations/static_domain_inference.rs`; verify rather than duplicate.

#### Heterogeneous Applied

Construct:

```text
Applied(
    origin,
    [
        Term(T),
        RecordRow({ x: U | R }),
        Term(StaticData(...))
    ]
)
```

and prove recursive unification preserves category and routes row equations to the row solver.

#### Heterogeneous StaticData

Use a constructor or associated domain with mixed payload/owner argument kinds if supported by the existing static constructor metadata. Prove:

```text
same domain + same constructor => recurse
different domain => mismatch
different constructor => mismatch
row child => row solver path
term child => ordinary inference path
```

#### Shared field inference

Prove:

```text
{ x: T | R }
unifies with
{ x: Int, y: String }

=> ordinary T solves to Int
=> row remainder accounts for y
```

The row solver must not decide `T == Int` using Rust equality.

#### Tail extension carries inferred payload

Prove a residual row containing a field whose payload is initially `InferenceTerm::Var(T)` materializes after T solves.

#### Conflict

Prove incompatible same-label payloads produce ordinary semantic inference conflict / row inference conflict as the established diagnostic path dictates, not an internal incident.

#### Existing controls

- ordinary Type generic inference remains unchanged;
- existing pack inference remains unchanged;
- existing row-polymorphic call behavior remains unchanged.

### Forbidden approaches

- no new inference identity;
- no constructor search;
- no backtracking;
- no arbitrary static subtyping;
- no test-only behavior branches.

### Tests to run now

Use G-T4 in §19.

### Acceptance

T4 is accepted only when the matrix in CL-01 through CL-17 has concrete passing evidence and the checkpoint reflects that evidence.

### Local STOP / CONSULT triggers

Any accepted T4 case requires a new inference identity, search, or merged row/type solver.

### Checkpoint update

Reconcile `INC-C3-8` and T4 status from `IMPLEMENTED / UNVERIFIED` or `PARTIAL` to accepted focused evidence. Do not mark P2 complete yet.

---

## T5 — Close remaining T7 acceptance without redesign

### Purpose

Convert the current strong T7/static-case implementation into a complete focused acceptance record by covering the remaining all-annotation, static type-member, heterogeneous owner, collision-refresh, and exact incremental cases.

### Preconditions

- T4 accepted;
- static-case signature architecture at `2ea4f3a1` remains intact.

### Consumes

```text
AssociatedSurface
TypeMemberDeclarationProduct
StaticDomainCaseCatalogProduct
StaticCaseSignatureProduct
Alias/Header/Member/StaticCaseSignature dependency graph
KindSyntax static-domain type-form path
canonical owner StaticArgument environments
```

### Produces

- all required annotation consumers proven;
- heterogeneous associated owner proven, including row form where semantically representable;
- static type-member domain convergence proven;
- behavior-only collision refresh and cold parity proven;
- T7 acceptance reconciled.

### Files and symbols

**Read/modify only if tests expose defects:**

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/checker/type_member_declaration.rs
phalcom-semantic/src/associated.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/static_case.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/incremental/type_members.rs
phalcom-semantic/tests/semantic/adts/associated/lookup.rs
phalcom-semantic/tests/semantic/adts/associated/scenarios.rs
phalcom-semantic/tests/semantic/associated/invocation.rs
phalcom-semantic/tests/semantic/foundations/kinds.rs
```

### Required implementation/test shape

#### A. Reuse existing evidence; do not duplicate

Existing tests already cover field/alias/generic-header projection availability. Add only the missing all-annotation consumer(s), especially callable parameter and return signatures if no equivalent test already exists.

Recommended source shape:

```phalcom
class Geometry {
    type Wrap<T: Type>: Type
}

class Consumer {
    transform(_ value: Geometry::Wrap<Int>) -> Geometry::Wrap<String> {
        ...
    }
}
```

The test should assert the checked callable signature retains canonical `TypeMemberProjection` with `source_declaration == Geometry::Wrap`.

If static-domain alias/case forms are the missing consumer instead, add the analogous callable annotation using a canonical static term/domain.

#### B. Static type-member domain convergence

The existing test:

```text
source_type_members_publish_checked_signatures_without_changing_namespace_selection
```

already proves a source type member parameter/result can use a user static domain and that wrong-domain arguments reject.

Reconcile this against the P2 T5/T7 obligation rather than writing another framework.

Add a projection/application test only if the current suite does not actually form a valid `Geometry::Transform<Shape::Scalar>`-style static member application.

The important observation is:

```text
AssociatedSurface selects TypeMemberDeclarationId
    ↓
checked TypeMemberDeclarationProduct validates canonical domains
    ↓
projection retains source declaration identity
```

#### C. Heterogeneous associated-owner argument including RecordRow

The canonical plan requires owner specialization to support `StaticArgument`, not `Vec<TypeId>`.

Test at the highest layer whose syntax is already specified.

If source syntax supports a row-valued generic owner argument, use it.

If not, construct the canonical owner specialization semantically:

```text
AssociatedStaticDomainDeclarationId(owner, "State")
+
owner_arguments [
    Term(Int),
    RecordRow(row),
    Term(ConstInt(3))
]
```

and prove:

```text
- associated domain specialization is canonical;
- kind/type-form resolution preserves the RecordRow category;
- case selection uses the same declaration identity;
- no rendered owner string is used.
```

Do **not** invent a new source spelling solely for this test.

#### D. Behavior-only collision refresh

The checkpoint/handoff records the owner-surface refresh architecture, but focused evidence remains a closeout obligation.

Add an incremental test with:

```text
Revision A:
    owner has associated type member/static domain `State`
    no colliding behavior

Revision B:
    an inherent impl / owner behavior contributes a colliding `State`

Expected:
    AssociatedSurface refreshes;
    collision diagnostic appears;
    exact selected declaration products respond appropriately;
    cold B has the same diagnostics/products.

Revision C:
    remove collision

Expected:
    collision clears;
    incremental C == cold C.
```

Use the accepted surface authority. Do not add invalidation by string name outside the existing owner/dependency seams.

#### E. Exact dependent versus sibling reuse

Reuse or extend existing pointer-reuse tests.

Required behavior:

```text
edit selected member/header/case signature
    => exact dependent recomputed

unrelated sibling member/case signature
    => retained where semantically unchanged

cold final state
    == incremental final state
```

Do not require pointer reuse for products whose current architecture does not promise object identity; assert it only where existing tests already use `Arc::ptr_eq` as the incremental reuse contract.

#### F. Full-graph traversal performance concern

Do not refactor it under T7 closeout.

Record:

```text
correctness accepted;
full graph edge/SCC reconstruction remains a performance follow-up.
```

### Forbidden approaches

- no second associated registry;
- no lookup priority chain;
- no string owner keys;
- no runtime wrapper for static domains/terms;
- no changing SCC classification to satisfy a test;
- no global invalidation workaround for behavior collision edits.

### Tests to run now

Exact new T7 tests first.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_members -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts::associated -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic associated::invocation -- --nocapture
```

Confirm nonzero selections.

### Tests explicitly deferred

Metadata/import and T9 gates.

### Acceptance

T7 is accepted when:

```text
- field, alias, generic-header, callable signature annotations all have the
  required source-member/static-domain availability;
- static type-member domains converge through checked products;
- heterogeneous associated owner environment preserves RecordRow if representable;
- collision introduction/removal refreshes one AssociatedSurface and matches cold;
- selected dependent invalidation and sibling reuse meet the retained graph contract;
- no new lookup authority exists.
```

### Local STOP / CONSULT triggers

- a test requires source grammar not specified;
- collision semantics conflict with the effective type-member spec;
- exact invalidation cannot be expressed with the retained dependency graph;
- a fix requires replacing `AssociatedSurface`.

### Checkpoint update

Reconcile the T7 incident ledger:

- mark `INC-C3-5` resolved only when all-annotation availability is proven;
- update `INC-C3-6`/`INC-C3-7` acceptance evidence;
- retain the full-graph traversal concern as performance follow-up;
- do not reopen resolved `INC-C3-10`.

---

## T6 — Prove and harden schema-v8 metadata behavior

### Purpose

Turn the existing schema-v8 structures/export/validator implementation into explicit compatibility and round-trip acceptance evidence.

### Preconditions

- T7 accepted;
- canonical semantic categories exist in memory.

### Consumes

```text
TYPE_METADATA_SCHEMA_VERSION = 8
StaticDomainDeclarationRecord
StaticDomainCaseRecord
StaticDomainRef
StaticArgumentRef
TypeNode::StaticData / Applied
ScopedTypeNode
MetadataExporter
validate_metadata_bundle
```

### Produces

- schema-v8 compatibility/validation matrix;
- lossless metadata round trips for P2 forms;
- local validator/export fixes if regressions expose them.

### Files and symbols

**Read/modify:**

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/declaration.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/bundle.rs
phalcom-type-meta/src/validate.rs

phalcom-semantic/src/metadata/export.rs
```

**Tests:**

```text
phalcom-type-meta/tests/schema_compat.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
```

### Required implementation shape

#### A. Add an explicit v8 schema-compatibility matrix

Current schema-compat tests have historical cases through earlier schema transitions but no dedicated schema-v8 acceptance matrix.

Add tests for:

1. **v8 catalog required for v8 static domain references**
   - a v8 bundle referencing a nominal/associated static domain without matching catalog entry fails with the existing targeted validation error.

2. **v8 duplicate catalog declaration**
   - fails deterministically.

3. **v8 duplicate/unsorted cases**
   - fails deterministically according to validator contract.

4. **v8 owner parameter kind/mode validity**
   - pack modes require corresponding pack kinds;
   - RecordRow/Type/static-domain owner arguments must match expected kind.

5. **v8 static-data case membership and payload arity/kinds**
   - missing case rejects;
   - wrong payload arity rejects;
   - wrong payload kind rejects.

6. **pre-v8 compatibility floor**
   - construct or reuse a supported schema-v7 bundle containing the static-domain features valid under v7;
   - prove the current decoder/validator does not impose the v8 catalog-membership requirement on v7.
   - do not infer expected compatibility; follow the existing version branches.

7. **newer unsupported version**
   - existing unsupported-version coverage remains green.

#### B. Round-trip canonical semantic forms

In semantic integration metadata tests, cover:

```text
ConstInt
nominal static domain kind
nominal StaticData
heterogeneous Applied with:
    Term
    RecordRow
static alias
associated static-domain specialization
associated static term
scoped bound static term / scoped heterogeneous application
```

Use the canonical pattern:

```rust
let bundle = MetadataExporter::new(
    &analysis.snapshot.store,
    Some(&analysis.snapshot.declarations),
    ...,
    MetadataProfile::RuntimePublic,
)
.with_project_universe(...)
.with_algebraic_semantics(
    &analysis.snapshot.data_semantics,
    &analysis.snapshot.enum_semantics,
)
.with_associated_static_domains(
    analysis.snapshot.associated_static_domains.as_ref(),
)
.with_aliases(
    analysis.snapshot.type_aliases.as_ref(),
)
.build_bundle(...)?;
```

Exact available constructor arguments may differ in the test harness. The critical requirement is to exercise the **snapshot-backed** path, not only the detached TypeStore fallback.

For each relevant bundle:

```text
export
encode JSON
decode JSON
validate
inspect stable declaration/case/argument identity
```

#### C. Snapshot-backed catalog membership

Write a regression that creates stale TypeStore history across incremental revisions:

```text
Revision A: Shape { Old }
Revision B: Shape { New }
```

Build metadata from revision B snapshot.

Expected:

```text
catalog contains New
catalog does not claim Old merely because TypeStore interned Old earlier
```

This directly proves INV-17.

Do not delete TypeStore history just to satisfy the test.

#### D. Static case stable representation

The current metadata catalog's case record may store a canonical encoded selector string in a field historically named `name`. Do not redesign the schema merely for naming aesthetics during closeout if stable identity remains lossless.

However, the test must prove:

```text
singleton Empty
!=
explicit-empty Empty()
```

if both are representable in the stable catalog.

If the current v8 schema actually collapses them in serialized identity, that is a correctness blocker and requires consultation because it changes stable metadata semantics.

### Forbidden approaches

- no raw semantic IDs in metadata;
- no snapshot-mode TypeStore enumeration as declaration authority;
- no silent StaticDomain→Type collapse;
- no weakening old-schema validator branches to pass new tests;
- no schema v9 unless a real correctness defect proves v8 structurally insufficient and a consultation authorizes migration.

### Tests to run now

Exact new schema tests first.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture
```

### Tests explicitly deferred

Cross-module provider/incremental tests to T7 below.

### Acceptance

T6 passes when v8 has explicit current-schema validation coverage, older supported schemas retain their documented floors, and all required P2 semantic forms round-trip losslessly through stable metadata.

### Local STOP / CONSULT triggers

- v8 cannot distinguish a semantically required case identity shape;
- old-schema compatibility policy is ambiguous;
- associated owner environment needs a new global identity scheme;
- passing validation would require serializing query-local IDs.

### Checkpoint update

Record:

```text
schema v8 accepted
compatibility floor
static-domain catalog authority
round-trip evidence
```

---

## T7 — Close T8 cross-module provider identity, import reconstruction, invalidation, source index, and cold parity

### Purpose

Prove P2 semantic identity survives module boundaries and provider edits without source-spelling reconstruction or broad invalidation.

### Preconditions

- T6 schema-v8 metadata gate accepted;
- T7 associated namespace accepted.

### Consumes

```text
SemanticWorkspaceSession
module/link interfaces
stable provider DeclarationId / StaticCaseId
semantic dependency graph
source index
metadata exporter
```

### Produces

- cross-module provider/consumer static-domain evidence;
- exact provider invalidation;
- source-index canonical target evidence;
- cold/incremental parity;
- clarification of “import reconstruction” against the live canonical import seam.

### Files and symbols

**Read/modify if tests expose defects:**

```text
phalcom-modules/src/declaration.rs
phalcom-modules/src/interface.rs

phalcom-semantic/src/session.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/source_index/*
phalcom-semantic/src/metadata/export.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs
phalcom-semantic/tests/semantic/incremental/adts.rs
possibly a focused new incremental static-domain import module
```

### Required implementation shape

#### A. Verify the live import seam before editing

Search narrowly for the mechanism by which semantic provider exports become consumer-visible declarations.

Determine whether T8 “import reconstruction” currently means:

```text
A. source/project semantic linking + stable metadata export
```

or whether a true:

```text
B. metadata bundle -> semantic declaration rehydration path
```

already exists.

At `2ea4f3a1`, no general `MetadataImporter` was identified during planning. Therefore:

- if the live architecture is A, test and fix A;
- if B exists, test B as well;
- if T8 appears to require inventing B from scratch, STOP AND CONSULT before adding a subsystem.

Do not interpret “import reconstruction” as permission to create a new persistence architecture.

#### B. Nominal provider/consumer identity

Provider:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

export Shape
```

Consumer, adapted to actual import syntax:

```phalcom
from provider import Shape

data Tensor<S: Shape>()
type V: Type = Tensor<Shape::Vector(3)>
```

Prove:

```text
- consumer binder kind points to provider Shape static domain identity;
- selected Vector case has provider StaticCaseId;
- no consumer-local synthetic declaration is created;
- if expression use is runtime-illegal, TypeOnlyRuntimeValue may coexist with
  the canonical provider source target.
```

Reuse the existing imported static-case target test where possible and extend it rather than duplicating setup.

#### C. Provider payload edit

Revision A:

```text
Shape::Vector(_ size: Int)
```

Revision B:

```text
Shape::Vector(_ size: OtherStaticDomain or changed accepted payload kind)
```

Choose a payload edit that produces a stable, testable semantic difference.

Expected:

```text
- provider StaticCaseSignature product changes;
- dependent consumer alias/header/application recomputes;
- unrelated provider/consumer product is retained where the architecture promises reuse;
- incremental B diagnostics/forms == cold B.
```

#### D. Provider case rename/add/remove

At least one rename case:

```text
Vector -> Sized
```

Expected:

```text
- old consumer selection invalidated;
- source-index old case target no longer resolves as current;
- diagnostics equal cold;
- no stale TypeStore-interned case is considered current declaration membership.
```

#### E. Associated provider specialization

Add a cross-module associated domain case if source export/import semantics permit:

```phalcom
class Container<T: Type> {
    type data State {
        Ready
        Value(_ item: T)
    }
}
```

Consumer selects:

```text
Container<Int>::State::Ready
```

and, where semantically available, a heterogeneous owner environment containing a row/static argument.

Prove declaration identity belongs to provider owner and specialization is declaration + canonical owner environment.

If associated static domains are not exportable through the current module surface, do not invent export semantics. Record the limitation and consult if it contradicts P2 acceptance.

#### F. Source index

Add a focused source-index assertion:

```text
consumer occurrence
    ->
provider StaticCaseId / static-domain declaration target
```

The target must derive from semantic resolution.

For a type-only static case used illegally at runtime:

```text
canonical source target exists
+
TypeOnlyRuntimeValue diagnostic exists
```

These are not mutually exclusive.

#### G. Metadata provider composition

Where practical in the semantic/compiler test harness, build a metadata bundle from a multi-module analyzed project and assert the provider static-domain catalog entry and consumer-referenced stable domain/case are both present and consistent.

The production compiler path already attaches canonical algebraic and associated products to `MetadataExporter`; use that architecture.

Do not fake composition by manually concatenating incompatible bundles unless a current API explicitly defines that behavior.

### Forbidden approaches

- no target reconstruction from import spelling;
- no consumer-local synthetic static declaration;
- no whole-workspace invalidation;
- no TypeStore historical membership fallback in snapshot-backed export;
- no new metadata importer without consultation;
- no runtime static target.

### Test changes required

CL-27 through CL-31.

At minimum:

```text
provider nominal static identity
provider payload edit cold parity
provider case rename cold parity
associated provider case if supported
source-index provider target
```

### Tests to run now

Exact new tests first.

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::imported_resolution -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::source_index -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::adts -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions -- --nocapture
```

If a whole existing filter is large, run exact new test names first and the full focused module only at G-T8.

### Acceptance

T8 cross-module/incremental acceptance passes when:

```text
- provider declaration/case identity survives consumer resolution;
- current metadata catalog uses current provider declaration products;
- provider edits invalidate exact consumers;
- stale cases do not survive as current membership;
- source-index targets remain canonical;
- incremental final result equals cold final result;
- no string reconstruction or broad workspace scan is introduced.
```

### Local STOP / CONSULT triggers

- import semantics require a new subsystem;
- exact invalidation lacks a stable dependency key;
- associated provider identity cannot be represented by existing stable refs;
- incremental/cold mismatch persists after one serious correction.

### Checkpoint update

Record T8 provider/import/incremental/source-index evidence and resolve `INC-C3-9` acceptance status.

---

## T8 — Reconcile T1–T8 acceptance and close remaining T5/T6/trait-boundary coverage

### Purpose

Before entering T9, prove the prerequisite “T1–T8 accepted” literally rather than assuming earlier focused slices cover every canonical plan obligation.

### Preconditions

- T4 accepted;
- T7 accepted;
- T8 metadata/import accepted.

### Consumes

Canonical P2 task acceptance sections and all current evidence.

### Produces

- a complete coverage map;
- only narrow missing regressions, if any;
- explicit declaration that T1–T8 are accepted.

### Files and symbols

Primarily documentation/tests. Source changes are allowed only for a newly exposed local defect.

### Required implementation shape

Build a table from the canonical P2 tasks and `CV-P2-01..22`.

At minimum reconcile:

#### T5

```text
static alias:
    declared result domain preserved
    canonical static body term
    wrong-domain body rejected

type-member convergence:
    Int/user static domains use canonical term/domain model
    no second constant/static term universe
```

#### T6

```text
class-body associated type data
runtime-data owner inherent-impl associated type data
stable owner+name declaration identity
canonical generic-owner specialization
singleton/constructor identity
runtime-negative behavior
```

#### T7

Use T5 task evidence from this closeout.

#### Trait boundary

Prove an unsupported arbitrary static-domain trait generic produces the intended targeted diagnostic rather than:

```text
dynamic fallback
missing evidence interpreted as conformance
internal inference failure
```

Do not expand the trait system.

#### Existing predecessor regressions

Reconcile C1/C2 pack/row behavior and ProperType negatives from existing evidence. Rerun only if the current P2 changes plausibly touch them or the canonical P2 final gate names them.

### Forbidden approaches

- no broad “run everything to see” strategy;
- no new features to fill a documentation checkbox;
- no reimplementing already-proven T5/T6 slices.

### Tests to run now

Only exact missing tests discovered by the reconciliation.

Then G-PRE-T9 in §19.

### Acceptance

T8 passes when the checkpoint can truthfully state:

```text
T1 accepted
T2 accepted
T3 accepted
T4 accepted
T5 accepted
T6 accepted
T7 accepted
T8 accepted
```

with each row pointing to concrete code/evidence.

Only then may T9 begin.

### Local STOP / CONSULT triggers

A canonical task acceptance condition has no implementation and requires material new architecture.

### Checkpoint update

Update the P2 coverage ledger and set next action to T9 stabilization.

---

## T9 — Stabilize P2, audit runtime/proper-type boundaries, and complete lifecycle records

### Purpose

Execute canonical P2 T9 only after T1–T8 acceptance.

### Preconditions

- T8 above accepted;
- no unresolved architecture consultation;
- all active semantic correctness blockers resolved.

### Consumes

All P2 interfaces and evidence.

### Produces

- final focused P2 gate;
- runtime isolation audit;
- ProperType audit;
- heterogeneous `Applied` consumer audit;
- trait boundary/resource evidence;
- truthful checkpoint state;
- P2 walkthrough;
- P2→P3 handoff.

### Files and symbols

**Audit baseline:**

```text
60a27f79d69aa3e79c22e3d19df221466c7db920...HEAD
```

**Lifecycle:**

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
```

**Create:**

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-walkthrough.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-handoff.md
```

### Required implementation shape

#### A. Runtime isolation audit

Start with the P2 diff:

```sh
git diff --name-only 60a27f79d69aa3e79c22e3d19df221466c7db920...HEAD
git diff 60a27f79d69aa3e79c22e3d19df221466c7db920...HEAD -- phalcom-core
```

Confirm P2 did not introduce:

```text
new static-domain VM opcodes
runtime static constructor objects
runtime static unifier
runtime message dispatch for static normalization
runtime invocation target for type-data static cases
```

Legitimate metadata inspection/display support is not a runtime static evaluator.

#### B. ProperType audit

Audit P2-touched paths that feed:

```text
call parameter/return runtime typing
field runtime layout
subtyping relation requiring proper Type
class hierarchy
dispatch receiver
runtime layout/lowering
```

Search for any new use of general `TypeId` where `ProperTypeId` proof used to be required.

The correct outcome is either:

```text
proper type proven
```

or a targeted static-domain/type-only diagnostic before runtime lowering.

#### C. `TypeData::Applied` consumer audit

Search the P2 diff and live tree for consumers of:

```rust
TypeData::Applied { origin, arguments }
```

Classify every P2-relevant consumer:

```text
H — heterogeneous-aware:
    matches StaticArgument::{Term, RecordRow}

T — intentionally type-only:
    uses as_type_id()/proper-type projection only after a proven boundary

B — bug:
    silently assumes all arguments are TypeId in a general semantic traversal
```

Fix all B cases required for P2 correctness.

Do not mechanically rewrite every type-only consumer. A consumer behind a proper-Type boundary may correctly reject/ignore row/static categories.

Special scrutiny:

```text
parameter occurrence walkers
dependency/fingerprint walkers
substitution/materialization
metadata export
source indexing
inheritance/supertype projection
diagnostic formatting
trait/conformance consumers
```

The previously audited different-origin Applied supertype projection path that currently only projects when all arguments are `Term` should be classified explicitly. If P2 acceptance does not require heterogeneous row inheritance and the behavior is a documented unsupported/follow-up seam, record it. If a required focused case fails because of it, fix within existing subtype architecture or consult if material redesign is required.

#### D. Trait boundary audit

Run the focused unsupported-domain diagnostic case.

Confirm no generalized static inference creates accidental conformance evidence.

#### E. Resource audit

Inspect beta-reduction implementation and P2 diff.

Acceptable evidence:

```text
- no clone of the entire type/static arena in ordinary beta reduction;
- a focused unit/regression proving the local substitution path;
- no benchmark required unless a measurable regression is suspected.
```

#### F. Formatter and core check

Run canonical T9 commands:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

If formatter fails only on untouched baseline files, classify D and record exact paths. If a P2-touched file fails formatting, fix that file.

#### G. Final focused P2 gate

Run G-FINAL in §19.

#### H. Lifecycle truth

Update P2 to:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only if every final required gate passes or any remaining failures are explicitly classified as nonblocking baseline failures under the plan.

Do not mark:

```text
LANG007.C3 complete
verification: RELEASE_COMPLETE
```

P3 remains.

### Forbidden approaches

- no P3 migration;
- no runtime static machinery;
- no broad unrelated refactor;
- no falsified verification state;
- no hiding formatter/baseline failures.

### Tests to run now

G-FINAL only, plus narrow affected core tests if the P2 diff contains non-mechanical compiler changes beyond metadata adapters.

### Acceptance

T9 passes when:

```text
P2 implementation coherent
focused semantic/type-meta/import/incremental evidence passes
runtime static machinery absent
ProperType boundary intact
Applied consumers classified/fixed
trait boundary targeted
beta resource invariant established
checkpoint truth current
walkthrough complete
handoff complete
```

### Local STOP / CONSULT triggers

- runtime machinery appears necessary;
- ProperType boundary has genuinely weakened;
- heterogeneous `Applied` requires a material subtype architecture change;
- final incremental/cold parity differs after one serious correction.

### Checkpoint update

Final P2 state and next action `LANG007.C3.P3`.

---

# 19. Verification gates

## G-T4A — T4 blocker regression gate

### Purpose

Prove the three audited T4 defects are specifically corrected before broader T4 testing.

### Run

Use exact new test names. The gate must include one test for each:

```text
row-tail-only generic reachability
nested Applied/StaticData stable lacks
transitive unresolved row root
ancestor symbolic dependency
both cross-domain occurs orderings
```

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
```

### Expected

```text
all exact regressions PASS
semantic library compiles
no new query-local identity appears in stable output
```

Do not broaden after PASS.

---

## G-T4 — Full T4 focused acceptance gate

### Purpose

Prove original T4 static inference plus amended row coordination.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_domain_inference -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_row_inference -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::record_row_polymorphism -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generic_application -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generics_core -- --nocapture
```

### Expected

```text
static constructor inference solves/refutes structurally
mixed Applied/StaticData row+term cases pass
shared field inference passes
row extension payload inference passes
unresolved local/internal row roots remain underconstrained
ancestor rows remain symbolic until parent resolution
cross-domain occurs checks reject
ordinary Type/pack inference controls pass
```

---

## G-T7 — T7 remaining acceptance gate

### Purpose

Prove all-annotation availability, static-member convergence, heterogeneous owner preservation, and exact surface invalidation.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_members -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts::associated -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic associated::invocation -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::kinds -- --nocapture
```

### Expected

```text
all annotation consumers resolve through one surface
static type-member domains use checked declaration products
heterogeneous owner specialization preserves category
collisions refresh incrementally and match cold
case/member/header selected dependencies rebuild exactly
sibling reuse remains intact where asserted
```

---

## G-T8A — Schema-v8 metadata gate

### Purpose

Prove current-schema and old-schema validation boundaries plus round trip.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture
```

### Expected

```text
schema v8 catalog validation covered
supported old schemas keep historical floor
heterogeneous Applied/RecordRow round trip
static term/alias/associated specialization round trip
snapshot-backed catalog excludes stale TypeStore history
```

---

## G-T8B — Cross-module and incremental parity gate

### Purpose

Prove provider identity, exact invalidation, canonical source targets, and cold parity.

### Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::imported_resolution -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::source_index -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::adts -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions -- --nocapture
```

### Expected

```text
provider domain/case identity retained
provider edits invalidate dependents
stale cases do not remain current
source targets point to provider canonical identities
incremental final state equals cold final state
```

---

## G-PRE-T9 — T1–T8 acceptance reconciliation

### Purpose

Authorize T9.

Do not run a broad suite. Review the coverage/evidence table and run only any exact test still marked missing.

Expected result:

```text
every original P2 task T1–T8 has accepted evidence
no unresolved architecture incident
T9 precondition satisfied
```

If any row is still “implemented but unverified,” T9 does not start.

---

## G-FINAL — Canonical P2 final focused gate

### Purpose

Prove P2 end-to-end without P3 or workspace release certification.

### Run

```sh
cargo fmt --all -- --check

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generic_application -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::generics_core -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::record_row_inference -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_domain_inference -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::record_row_polymorphism -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::type_members -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts::associated -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::imported_resolution -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::adts -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Also run the exact new source-index test(s) if not selected by the import suite.

### Expected

```text
all focused P2 obligations pass
known formatter drift, if still present, is explicitly classified by changed path
core checks
no runtime static evaluator exists
P2 may be COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

### Broad tests explicitly not part of G-FINAL

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

---

# 20. Final focused acceptance

At plan completion, fill this table with actual results. Do not pre-fill PASS.

| Coverage ID | Required evidence | Result |
|---|---|---|
| `CV-P2-01` | completed P1 gate/preflight inheritance | |
| `CV-P2-02` | binder/static-domain kind resolution | |
| `CV-P2-03` | heterogeneous generic application | |
| `CV-P2-04` | row argument application/substitution + CL-04..CL-17 | |
| `CV-P2-05` | ProperType negative | |
| `CV-P2-06` | repeated canonical formation | |
| `CV-P2-07` | static identity inequality | |
| `CV-P2-08` | checked substitution failure | |
| `CV-P2-09` | scoped/beta reduction | |
| `CV-P2-10` | same-constructor static inference | |
| `CV-P2-11` | constructor/domain refutation | |
| `CV-P2-12` | static alias | |
| `CV-P2-13` | type-member static-domain convergence | |
| `CV-P2-14` | associated declaration/specialization + heterogeneous owner | |
| `CV-P2-15` | unified `::` collision authority + incremental refresh | |
| `CV-P2-16` | trait boundary | |
| `CV-P2-17` | schema-v8 metadata round trip/validation | |
| `CV-P2-18` | provider edit incremental/cold parity | |
| `CV-P2-19` | runtime-negative static term | |
| `CV-P2-20` | C1/C2 inference/pack/row controls | |
| `CV-P2-21` | beta-reduction resource invariant | |
| `CV-P2-22` | domain-specific diagnostics | |

Add a second table for closeout-specific cases:

| Closeout ID | Test/evidence | Result |
|---|---|---|
| CL-01 | row-tail-only reachability | |
| CL-02 | Applied RecordRow reachability | |
| CL-03 | StaticData/associated owner reachability | |
| CL-04 | Applied nested stable lacks | |
| CL-05 | StaticData/owner nested stable lacks | |
| CL-06 | shared-field inference | |
| CL-07 | tail extension inferred payload | |
| CL-08 | field conflict | |
| CL-09 | direct row underconstraint | |
| CL-10 | transitive residual underconstraint | |
| CL-11 | ancestor symbolic dependency | |
| CL-12 | parent continuation | |
| CL-13 | occurs ordering A | |
| CL-14 | occurs ordering B | |
| CL-15 | mixed Applied | |
| CL-16 | mixed StaticData | |
| CL-17 | ordinary inference control | |
| CL-18 | callable all-annotation | |
| CL-19 | static type-member convergence | |
| CL-20 | associated owner RecordRow | |
| CL-21 | collision invalidation | |
| CL-22 | exact T7 reuse | |
| CL-23 | schema-v8 validation | |
| CL-24 | pre-v8 floor | |
| CL-25 | metadata round trip | |
| CL-26 | scoped metadata | |
| CL-27 | provider identity | |
| CL-28 | provider payload edit | |
| CL-29 | provider rename | |
| CL-30 | associated provider edit | |
| CL-31 | source index | |
| CL-32 | trait boundary | |
| CL-33 | ProperType audit | |
| CL-34 | Applied consumer audit | |
| CL-35 | resource audit | |
| CL-36 | final focused gate | |

P2 may be completed as:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only when the table truthfully supports it.

Do not claim `RELEASE_COMPLETE`.

---

## 21. Performance/resource evidence

This plan does not add a new performance project. It preserves existing resource invariants.

### 21.1 Required evidence

Record:

```text
row inference
    no search/backtracking
    explicit root completion
    no whole-graph row completion requirement

static inference
    no constructor enumeration/search

static domain/case lookup
    catalog/direct stable identity, not TypeStore scan for semantic membership

incremental
    exact dependency invalidation; no whole-workspace fallback

type-lambda beta reduction
    no whole-arena clone
```

### 21.2 Full declaration graph traversal

The current signature dependency implementation may reconstruct/traverse full graph edges/SCCs during analysis.

For P2 closeout:

```text
classification: performance follow-up
correctness blocker: no, unless focused evidence shows pathological failure
required redesign: no
```

Record it in walkthrough/handoff.

### 21.3 Performance STOP trigger

Consult if a required correctness fix appears to require:

```text
linear scan of all static domains per application
whole-workspace scan for local provider edits
whole type/static arena clone per normalization
exponential/backtracking inference
```

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

Keep:

```text
active plan: LANG007.C3.P2
P2 status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
live closeout starting revision: 2ea4f3a1...
```

The existing `FOCUSED_TESTED` label refers to implemented slices. Do not downgrade it merely because T4/T8 remain open; instead state precisely which tasks are accepted/open.

### 22.2 During plan

Update only for durable events:

```text
T4 blocker corrections accepted
T4 full acceptance
T7 acceptance
schema-v8 acceptance
cross-module/import/cold parity acceptance
new consultation/amendment
new baseline/deferred failure
T1–T8 prerequisite closure
final G-FINAL result
```

Do not turn checkpoint into an edit log.

### 22.3 Incident ledger reconciliation

At minimum:

```text
INC-C3-8
    move to RESOLVED / ACCEPTED only after T4 gate

INC-C3-9
    move to RESOLVED / ACCEPTED only after schema/import/cold parity gates

INC-C3-10
    retain resolved architecture/implemented evidence;
    do not reopen
```

Reconcile older T7 incidents against actual final evidence.

### 22.4 At plan completion

Record:

```text
P2 final revision
status COMPLETE
completion IMPLEMENTED
verification FOCUSED_TESTED
T4 coordinator invariants
AssociatedSurface/static-case graph invariants
schema v8 compatibility
cross-module provider identity
incremental/cold parity
runtime isolation
ProperType audit
Applied consumer audit
resource evidence
baseline/deferred failures
full-graph performance follow-up
next action LANG007.C3.P3
```

Do not mark C3 complete.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-walkthrough.md
```

It must contain:

1. final revision/state;
2. P2 predecessor revision and closeout starting revision;
3. final implementation summary;
4. generalized application representation;
5. T4 coordinator architecture;
6. the three T4 blockers and their exact corrections;
7. T4 acceptance matrix and results;
8. static alias/type-member evidence;
9. associated type-data identity/specialization;
10. T7 all-annotation/collision/invalidation evidence;
11. static-case catalog/signature architecture inherited;
12. schema-v8 metadata structure and compatibility;
13. cross-module provider/import identity;
14. incremental/cold parity evidence;
15. source-index behavior;
16. ProperType/runtime boundary audit;
17. `TypeData::Applied` consumer audit results;
18. trait-boundary evidence;
19. resource/performance evidence;
20. full-graph performance follow-up;
21. deviations from this plan;
22. consultations/amendments;
23. tests added;
24. tests actually run;
25. tests deliberately deferred;
26. baseline failures;
27. exact completion/verification classification.

Do not claim a test that was not run.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-handoff.md
```

The P2→P3 handoff must include:

```text
current revision
P2 verification state
stable TypeStore generalized application APIs
StaticArgument representation
static-domain/case identity model
InferenceCoordinator and row-frame completion invariant
static inference behavior
static aliases and type-member domains
associated static-domain declaration + owner environment
AssociatedSurface authority
schema v8 metadata compatibility
provider/import/source-index identity
incremental/cold equivalence evidence
ProperType/runtime isolation
resource invariant
known baseline failures
full-graph performance follow-up
first recommended P3 reads
things P3 must not redesign
```

P3 must inherit, not rediscover:

```text
- separate row identity;
- static case catalog/signature split;
- one AssociatedSurface;
- stable declaration-based metadata;
- static terms have no runtime value semantics.
```

---

## 25. Completion truth table

Do not conflate:

```text
source written
tests added
exact regression passed
focused task suite passed
T1–T8 accepted
P2 implemented
P2 focused-tested
C3 complete
release certified
```

Required truth at successful closeout:

| State | Required |
|---|---|
| source written | yes |
| required tests added | yes |
| exact regressions passed | yes |
| focused task suites passed | yes |
| T1–T8 accepted | yes |
| P2 completion | IMPLEMENTED |
| P2 verification | FOCUSED_TESTED |
| C3 complete | no |
| release certified | no |

---

## 26. Plan self-review

Before implementation begins, verify this plan against the following:

### Architecture

- [x] one owner for row shape, ordinary inference, namespace selection, static membership, checked payload, metadata identity;
- [x] no duplicate source of truth planned;
- [x] row and ordinary inference identities remain distinct;
- [x] static/runtime boundary explicit;
- [x] incremental/cold equivalence explicit;
- [x] import ambiguity is a verify-first seam, not an invitation to invent architecture.

### Luna executability

- [x] T4 defects have exact source seams and correction shapes;
- [x] frame-aware completion algorithm is specified;
- [x] T7 work is evidence-first, not redesign-first;
- [x] T8 schema/provider tests are concrete;
- [x] stop/consult triggers are objective;
- [x] debugging loops are bounded.

### Testing

- [x] correctness surface is broad by design;
- [x] execution breadth is narrow during BUILD;
- [x] exact regressions precede focused suites;
- [x] workspace tests/clippy are explicitly deferred;
- [x] baseline formatter/unit failure policy is explicit.

### Documentation lifecycle

- [x] checkpoint remains shared authority;
- [x] walkthrough required;
- [x] handoff required;
- [x] T9 cannot start before T1–T8 acceptance;
- [x] P2 completion does not claim C3 completion.

### Premium-compute discipline

- [x] accepted architecture is pre-decided;
- [x] likely consultation questions are identified;
- [x] mechanical implementation is delegated;
- [x] adviser use is reserved for material identity/ownership/compatibility conflicts.

---

# Appendix A — T4 implementation pseudocode in one place

This appendix is intentionally redundant with T2/T3 at the code-shape level so an implementer can work without reconstructing the design.

## A.1 Parameter reachability

```rust
struct ParameterUseWalker<'a> {
    store: &'a TypeStore,
    sought: &'a [TypeParameterId],
    seen_types: HashSet<TypeId>,
    seen_rows: HashSet<RecordRowId>,
    seen_domains: HashSet<StaticDomainId>,
}

impl ParameterUseWalker<'_> {
    fn visit_type(&mut self, ty: TypeId) -> bool {
        if !self.seen_types.insert(ty) {
            return false;
        }

        match self.store.get(ty) {
            TypeData::Parameter(parameter) => self.sought.contains(parameter),

            TypeData::Applied { origin, arguments } => {
                self.visit_type(*origin)
                    || arguments.iter().copied()
                        .any(|arg| self.visit_static_argument(arg))
            }

            TypeData::StaticData(term) => {
                self.visit_static_domain(term.domain)
                    || term.arguments.iter().copied()
                        .any(|arg| self.visit_static_argument(arg))
            }

            TypeData::Record(row) => self.visit_row(*row),

            // existing structural cases recurse as they already do.
            // Do not regress callable/tuple/union/projection/pack traversal.

            _ => false,
        }
    }

    fn visit_static_argument(&mut self, arg: StaticArgument) -> bool {
        match arg {
            StaticArgument::Term(ty) => self.visit_type(ty),
            StaticArgument::RecordRow(row) => self.visit_row(row),
        }
    }

    fn visit_row(&mut self, row_id: RecordRowId) -> bool {
        if !self.seen_rows.insert(row_id) {
            return false;
        }

        let row = self.store.record_row(row_id);

        row.fields.iter().any(|field| self.visit_type(field.ty))
            || match row.tail {
                RecordRowTail::Closed => false,
                RecordRowTail::Parameter(parameter) =>
                    self.sought.contains(&parameter),
                RecordRowTail::Projection(projection) =>
                    self.visit_type(projection),
            }
    }

    fn visit_static_domain(&mut self, domain: StaticDomainId) -> bool {
        if !self.seen_domains.insert(domain) {
            return false;
        }

        match self.store.static_domain(domain) {
            Some(StaticDomainData::Associated { owner_arguments, .. }) => {
                owner_arguments.iter().copied()
                    .any(|arg| self.visit_static_argument(arg))
            }
            _ => false,
        }
    }
}
```

Do not copy this mechanically if live types differ. Preserve the semantic traversal.

## A.2 Stable lacks traversal

```rust
fn collect_from_static_argument(
    store: &TypeStore,
    arg: StaticArgument,
    state: &mut LacksTraversalState,
) {
    match arg {
        StaticArgument::Term(ty) =>
            collect_from_type(store, ty, state),

        StaticArgument::RecordRow(row) =>
            collect_from_row(store, row, state),
    }
}

fn collect_from_row(
    store: &TypeStore,
    row_id: RecordRowId,
    state: &mut LacksTraversalState,
) {
    if !state.seen_rows.insert(row_id) {
        return;
    }

    let row = store.record_row(row_id);

    if let RecordRowTail::Parameter(parameter) = row.tail {
        for field in row.fields.iter() {
            state.emit_lack(parameter, field.name.clone());
        }
    }

    for field in row.fields.iter() {
        collect_from_type(store, field.ty, state);
    }

    if let RecordRowTail::Projection(projection) = row.tail {
        collect_from_type(store, projection, state);
    }
}
```

When visiting `TypeData::StaticData`, also visit its associated domain owner environment if that canonical domain carries static arguments.

## A.3 Frame-aware row completion

```rust
fn classify_required_row_root(
    coordinator: &InferenceCoordinator,
    solver: &RecordRowSolver<InferenceTerm>,
    frame: InferenceFrameId,
    root: RecordRowVarId,
) -> RowRootCompletion {
    let normalized = solver.normalize_term(&RecordRowTerm {
        fields: Box::new([]),
        tail: RecordRowTermTail::Var(root),
    });

    match normalized.tail {
        RecordRowTermTail::Closed
        | RecordRowTermTail::Parameter(_)
        | RecordRowTermTail::Projection(_) => RowRootCompletion::Complete,

        RecordRowTermTail::Var(tail)
            if coordinator.row_variable_is_owned_by_ancestor_frame(tail, frame) =>
        {
            RowRootCompletion::PendingAncestor(tail)
        }

        RecordRowTermTail::Var(tail) =>
            RowRootCompletion::Underconstrained(tail),
    }
}
```

Then:

```text
if any required root is Underconstrained:
    return RecordRowSolveResult::Underconstrained

else:
    return Solved(solution)
    and separately preserve PendingAncestor information through existing
    term_has_unresolved_ancestor_row / symbolic-result publication.
```

Do not treat `PendingAncestor` as materializable.

---

# Appendix B — Required closeout test scenarios

## B.1 T4 compact scenario set

### Mixed ordinary + row payload

```text
formal:
    Pair<T, { item: T | R }>

actual:
    Pair<Int, { item: Int, name: String }>

expect:
    T = Int
    R = { name: String }
```

### Shared field ordinary inference

```text
formal row:
    { item: T | R }

actual:
    { item: Int, extra: String }

expect:
    ordinary solver gets T ≡ Int
    row solver gets R = { extra: String }
```

### Transitive local residual

```text
R = { a: Int | S }
S unresolved, local/internal

expect:
    R underconstrained
```

### Ancestor residual

```text
parent owns P
child local C = { mapped: Bool | P }

expect:
    child symbolic
    no canonical row
    no child underconstraint
```

### Parent continuation

```text
later P = { cached: Bool | Closed }

expect final child/parent result:
    { mapped: Bool, cached: Bool }
```

### Cross-domain recursive cycle

```text
T contains row R
R payload contains T

expect:
    occurs rejection
```

Run both constraint orderings.

## B.2 T7 compact scenario set

```text
callable param type-member projection
callable return type-member projection
static-domain type-member parameter/result
associated owner specialization with heterogeneous row arg
behavior collision added then removed
incremental == cold
selected product rebuild / sibling retain
```

## B.3 T8 compact scenario set

```text
v8 missing catalog -> reject
v8 wrong owner kind -> reject
v8 wrong case/payload -> reject
v7 supported equivalent -> retain old floor

snapshot A:
    Shape { Old }
snapshot B:
    Shape { New }
metadata B:
    contains New
    does not publish Old as current membership

provider:
    Shape::Vector(Int)
consumer:
    Tensor<Shape::Vector(3)>

provider edit:
    payload/rename

expect:
    provider identity stable where appropriate
    exact consumer invalidation
    source target canonical
    incremental == cold
```

---

# Appendix C — Final implementation order

Use this exact order unless a STOP/CONSULT trigger changes it:

```text
T1  preflight

T2  fix parameter reachability + stable lacks
    ↓
G-T4A partial exact regressions

T3  fix frame-aware transitive row completion
    ↓
complete G-T4A

T4  full T4 acceptance
    ↓
G-T4

T5  T7 residual acceptance
    ↓
G-T7

T6  schema-v8 metadata acceptance
    ↓
G-T8A

T7  cross-module/import/source-index/incremental T8
    ↓
G-T8B

T8  T1–T8 acceptance reconciliation
    ↓
G-PRE-T9

T9  runtime/proper-type/Applied/resource audits + lifecycle
    ↓
G-FINAL

P2 walkthrough
P2 handoff
checkpoint final update
next: LANG007.C3.P3
```

Do not reorder T9 ahead of T8. Do not use T9 as a substitute for missing T4/T7/T8 acceptance.
