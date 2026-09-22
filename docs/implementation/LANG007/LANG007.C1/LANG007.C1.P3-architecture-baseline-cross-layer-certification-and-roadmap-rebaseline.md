---
id: LANG007.C1.P3
category: LANG
program: LANG007
checkpoint: LANG007.C1
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG007.C1.P1 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
  - LANG007.C1.P2 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
follows: LANG007.C1.P2
supersedes: null
repository: aureat/phalcom-lang
branch: main
planning_revision: 6551576035a6e55718497c457fac6073b322d599
planning_revision_subject: "feat(lang007): implement P2 authoritative specification and intrinsic keyword convergence"
prepared: 2026-09-22
executor: GPT-5.6 Luna High/Extra High
scope_note: checkpoint-closing architecture baseline, cross-layer integration certification, acceptance matrix, roadmap re-baselining, and LANG007.C1 closure; no new language semantics
---

# LANG007.C1.P3 — Architecture Baseline, Cross-Layer Certification, and Roadmap Re-baselining

## 0. Executor contract

This is the checkpoint-closing Luna patch-grade plan for `LANG007.C1`.

`LANG007.C1.P1` and `LANG007.C1.P2` are already complete and focused-tested at the planning revision. P3 therefore does **not** design or implement a new type-level feature. It closes C1 by proving that the already accepted and implemented type-level structural calculus is coherent across its architectural seams, by fixing only bounded defects that violate already-ratified C1 semantics, by producing the final invariant-to-evidence matrix, and by re-baselining future LANG007 work around what C1 actually owns.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve all P1/P2 semantic and ownership decisions;
- must not improvise new language semantics;
- must not reimplement capabilities already established by C1 under a new representation;
- must add the smallest cross-layer integration/incremental/stress evidence needed to make C1 closure truthful;
- may make bounded implementation corrections only when they implement an already accepted C1 invariant;
- must STOP AND CONSULT before changing language semantics, canonical identity, domain ownership, projection ownership, runtime responsibility, generic surface, or persistence authority;
- must preserve `intrinsic` as the source implementation-kind keyword and keep `NativeIntrinsicId` as distinct internal metadata;
- must preserve the completed source/specification authority from P2;
- must distinguish checkpoint certification from workspace/release certification;
- must classify and defer unrelated failures;
- must keep `LANG007.C1-CHECKPOINT.md` current;
- must produce the required P3 walkthrough and P3 handoff;
- must close `LANG007.C1` only if every P3 acceptance obligation is satisfied.

Testing is evidence gathering. P3 is broader than P1/P2 stabilization, but it is still a **focused checkpoint-certification plan**, not a release-wide test campaign.

### P3 start lock

Before substantive P3 work:

```text
LANG007.C1.P1:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

LANG007.C1.P2:
    COMPLETE / IMPLEMENTED / FOCUSED_TESTED

required records:
    LANG007.C1.P1-walkthrough.md
    LANG007.C1.P1-handoff.md
    LANG007.C1.P2-walkthrough.md
    LANG007.C1.P2-handoff.md

checkpoint:
    status IN_PROGRESS
    completion PARTIAL
    verification FOCUSED_TESTED
    active_plan null
    next action LANG007.C1.P3
```

If any prerequisite regresses before execution, STOP and restore truthful lifecycle state before proceeding.

## 1. Goal

Establish the final as-built `LANG007.C1` architecture baseline, prove the complete closed structural type-operation vertical across source formation, canonical identity, normalization, generic/inference preservation, metadata, incremental state, source identity, and the compiler/runtime boundary, produce a complete acceptance/evidence matrix, re-baseline the remaining LANG007 roadmap around the capabilities C1 already implemented, and close the checkpoint without introducing new language semantics.

## 2. Checkpoint acceptance objective

`LANG007.C1` owns the first complete closed structural type-operation vertical:

```text
TypePack
LabeledTypePack
RecordRow
typed compile-time type-member arguments
canonical type-member identity
symbolic type-member projections
Tuple Positionals / Labeled / At
TypePack / LabeledTypePack / Tuple / RecordRow / Record Map<F>
open-row mapped tails
generic substitution / materialization preservation
stable metadata
source identity
canonical Universe anchors
intrinsic compiler-side normalization
zero VM type-member execution
```

P1 implemented and focused-tested the semantic/compiler vertical. P2 converged decisions, specifications, source presentation, and the `intrinsic` keyword model. P3 is **full C1 checkpoint closure**.

P3 is accepted only when:

1. every accepted C1 invariant has one normative owner, one implementation owner, and adequate evidence;
2. the type-level structural calculus is proven across cross-layer seams, not only in isolated foundation tests;
3. source forms resolve to the same canonical member identities used by normalization and persistence;
4. symbolic projections survive substitution/materialization and durable persistence without query-local identity leakage;
5. at least one realistic metadata/module-boundary discriminator proves that C1 semantic meaning survives supported persistence;
6. at least one representative C1 edit has cold/incremental final-state parity;
7. source identity is proven for `T[I]` / `T::At<I>` and domain-distinct same-spelled members;
8. generalized type-member machinery is proven compatible with existing trait-associated projections;
9. no VM/runtime type-member execution or runtime pack/row object model exists;
10. zero-Tuple/Unit and empty-Record static/runtime semantics remain coherent;
11. resource behavior remains compatible with the accepted `At O(1)` / `Map O(n)` / bounded symbolic normalization model;
12. deferred features remain absent from the public language surface;
13. no duplicate pack/member/projection/normalization source of truth exists;
14. P3-owned failures are fixed or explicitly escalated;
15. unrelated baseline failures remain truthfully classified;
16. the remaining LANG007 roadmap is re-baselined so future work extends C1 instead of reimplementing it;
17. the C1 checkpoint record, guidance, program/status records, walkthrough, and handoff agree;
18. `LANG007.C1` is marked `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only after all required evidence passes.

P3 does **not** require workspace-wide `RELEASE_COMPLETE` certification.

## 3. Repository grounding

Prepared against:

```text
repository:        aureat/phalcom-lang
branch:            main
revision:          6551576035a6e55718497c457fac6073b322d599
revision subject:  feat(lang007): implement P2 authoritative specification and intrinsic keyword convergence
P1 implementation:
    479f8fb82fa58730fb2c40c2652b5e07016549ec
P2 completion:
    6551576035a6e55718497c457fac6073b322d599
```

### 3.1 Live lifecycle facts verified during planning

At `65515760…`:

- `LANG007.C1.P1` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C1.P2` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- `LANG007.C1` remains `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`.
- `LANG007.C1-CHECKPOINT.md` names `LANG007.C1.P3` as next action.
- `LANG007.C1-GUIDANCE.md` names P3 as next plan.
- P1 and P2 walkthroughs/handoffs exist.
- TDR-0083 through TDR-0089 are registered as accepted.
- `intrinsic` is the canonical fourth `ImplementationKind`.
- `@intrinsic` is retired from source/builtin authority.
- internal `NativeIntrinsicId` remains separate internal metadata.

### 3.2 Live architecture facts inherited from P1/P2

P1/P2 establish:

- distinct `TypePack` and `LabeledTypePack` kinds;
- distinct `RecordRow` semantics;
- canonical member identity and signatures;
- typed type-member arguments including `Const(Int)`;
- one canonical Subscript identity for `T[I]` and `T::At<I>`;
- symbolic type-member projection;
- closed intrinsic structural normalization;
- Tuple `Positionals`, `Labeled`, `At`, `Map`;
- Record/RecordRow Map including mapped open tails;
- substitution/inference/materialization integration;
- stable metadata nodes for new domains/terms/projections;
- source-index / Universe identity integration;
- no type-member VM dispatch;
- no runtime pack/row object model.

### 3.3 Evidence gaps found during P3 planning

1. `foundations/type_members.rs` is the principal focused structural-member suite, but there is no dedicated P3 closure fixture spanning the whole vertical.
2. `phalcom-semantic/tests/semantic/incremental/` currently has no dedicated LANG007/type-member incremental suite.
3. metadata/type-meta coverage exists generally, but planning search did not establish a C1-specific cross-module/persistence discriminator for type-member structural terms.
4. source-index infrastructure is rich, but planning search did not expose a dedicated `SemanticTargetId::TypeMember` symbol; P3 must verify the **actual** canonical target representation and prove alias/domain identity contracts instead of assuming an enum shape.
5. the original LANG007 roadmap assumed much of C1 implementation would happen later in C2–C6. Live `PROGRAM.md` now leaves future checkpoint allocation open. P3 must re-baseline from the as-built C1 state.
6. two trait dependency tests remain known unrelated baseline failures in the C1 checkpoint.

### 3.4 Drift instruction

> Re-read all primary paths before editing. Adapt mechanical drift locally. Treat changes to semantic ownership, canonical identity, result-domain authority, inference lifetime, metadata identity, source target authority, incremental ownership, runtime responsibility, or future-checkpoint semantics as STOP/CONSULT conditions.

## 4. Required reads before implementation

### 4.1 Workflow authority

```text
AGENTS.md
docs/implementation/README.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/shared-consultation-escalation-protocol.md
docs/workflow/commit-and-push-discipline.md
```

### 4.2 Current LANG007 lifecycle

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-GUIDANCE.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P1-walkthrough.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P1-handoff.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P2-walkthrough.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P2-handoff.md
```

### 4.3 Accepted decision authority

```text
docs/decisions/accepted/0081-tuple-and-record-are-transparent-structural-value-products.md
docs/decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md
docs/decisions/accepted/0083-type-level-domains-and-typed-compile-time-parameters.md
docs/decisions/accepted/0084-canonical-type-level-terms-and-inference-lifetimes.md
docs/decisions/accepted/0085-parameterized-type-members-and-symbolic-projections.md
docs/decisions/accepted/0086-type-level-subscript-is-tuple-at.md
docs/decisions/accepted/0087-structural-map-is-shape-preserving.md
docs/decisions/accepted/0088-native-type-operations-use-closed-compiler-normalization.md
docs/decisions/accepted/0089-intrinsic-is-a-first-class-implementation-keyword.md
```

### 4.4 Normative specifications

```text
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/structural-products.md
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/source-declarations.md
docs/specs/objects/impl.md
docs/specs/objects/traits.md
docs/specs/universe.md
docs/specs/objects/values.md
docs/specs/vm/products.md
docs/specs/vm/values-and-objects.md
docs/specs/callables/INDEX.md
docs/specs/callables/dispatch.md
```

P3 should not perform another broad specification rewrite. These are read authority unless an integration defect reveals a stale P2 statement.

### 4.5 Core C1 implementation takeover

Expected live owners:

```text
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/row_solver.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/metadata/export.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/validate.rs
phalcom-semantic/src/source_index/
phalcom-semantic/src/identity.rs
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/record.ph
phalcom-core/src/compiler/
phalcom-core/src/native/
```

### 4.6 Existing focused tests

```text
phalcom-semantic/tests/semantic/foundations/type_members.rs
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/foundations/substitution.rs
phalcom-semantic/tests/semantic/foundations/record_row_inference.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
phalcom-semantic/tests/semantic/integration/workspace.rs
phalcom-semantic/tests/semantic/integration/native_conformance.rs
phalcom-semantic/tests/semantic/incremental/mod.rs
phalcom-semantic/tests/semantic/incremental/associated_types.rs
phalcom-semantic/tests/semantic/incremental/product_stability.rs
phalcom-semantic/tests/semantic/incremental/recomputation_metrics.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
```

## 5. Normative authority

### 5.1 Semantic authority

P3 inherits and must not redesign:

- TDR-0083: distinct type-level domains and typed compile-time values;
- TDR-0084: stable canonical terms versus ephemeral inference variables;
- TDR-0085: parameterized domain-aware type members and symbolic projections;
- TDR-0086: one canonical type-level Subscript identity for `T[I]` / `T::At<I>`;
- TDR-0087: shape-preserving structural `Map<F>`;
- TDR-0088: closed compiler-side normalization and no VM execution;
- TDR-0089: `intrinsic` keyword / `ImplementationKind::Intrinsic`, with `@intrinsic` retired.

### 5.2 Product authority

TDR-0081 and structural-product specs own:

```text
Tuple:
    ordered positional lane + ordered labeled lane

Record:
    key-oriented structural row

Tuple([],[]) ≡ Unit

Record({}) remains a distinct static Record type
```

### 5.3 Associated-type compatibility

Trait-associated member evidence remains owned by `associated-types.md` and trait/conformance machinery. Intrinsic lookup must never capture or bypass associated identity/evidence.

### 5.4 Current implementation

P1 implementation is evidence of accepted architecture. P3 may correct a defect only when the correction is forced by already accepted semantics. It may not change accepted semantics because a seam is inconvenient.

### 5.5 Future/non-normative proposals

Remain outside C1:

```text
general nominal const generics
trait const generics
GATs
parameterized trait-associated members
open user type families
arbitrary compile-time Phalcom execution
const arithmetic
pack concatenation/split solving
variadic callable inference/lowering
Symbol/Range Tuple type indexing
union-distributed type members
new structural operators
runtime TypePack/LabeledTypePack/RecordRow values
VM type-member execution
anonymous structural inherent impls
```

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Expected current owner | P3 invariant |
|---|---|---|
| kind/domain identity | semantic type store/kind layer | one canonical domain system |
| pack canonical terms | semantic type store | no second pack representation |
| const member argument | type-member/annotation layer | typed value, not kind |
| member identity/signature | `types/type_member.rs` | declaration/domain identity |
| projection identity | type-member projection model | member + subject + canonical args/result domain |
| intrinsic normalization | projection normalizer | Reduced/Stuck/Invalid; no VM |
| source formation | annotation resolver | signature-directed argument elaboration |
| row mapped-tail term | row/projection model | stable term, not query-local row var |
| substitution/materialization | semantic/checker layers | preserve canonical structure |
| query-local inference | checker inference | never durable identity |
| stable metadata | semantic exporter + type-meta | no raw local/query IDs |
| source identity | source-index target infrastructure | aliases converge; domain members distinct |
| Universe anchor | Tuple/Record source | provenance only, not runtime member |
| implementation kind | `ImplementationKind` | `Intrinsic` distinct from `Native` |
| primitive intrinsic metadata | `NativeIntrinsicId` | separate from source implementation kind |
| runtime boundary | core/VM | zero type-member execution |
| lifecycle authority | C1 checkpoint | single durable state |

### 6.2 Known residual baselines

```text
BASELINE-001
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Both reproduce before LANG007 P1 and remain unrelated unless new evidence proves otherwise.

### 6.3 Roadmap state

The old conceptual C2–C7 split is no longer mechanically authoritative. C1 already implemented substrate previously imagined for later checkpoints. Live `PROGRAM.md` leaves future checkpoint allocation open. P3 must re-baseline future work from the final C1 state.

## 7. Architecture

P3 certifies this authority flow:

```text
source syntax / canonical Universe declaration
    ↓
semantic domain + canonical member/signature
    ↓
canonical projection / structural term
    ↓
intrinsic semantic normalization
    ↓
substitution / inference / materialization
    ↓
stable metadata + source identity + incremental products
    ↓
downstream compiler consumers

runtime VM
    receives no type-member solving responsibility
```

### 7.1 Baseline dimensions

P3 freezes:

```text
A. Source
   intrinsic type members
   type subscript
   At named presentation
   typed member-local arguments

B. Domains
   Type
   RecordRow
   TypePack
   LabeledTypePack
   Arrow
   Const(τ)

C. Canonical terms
   proper/higher-kinded forms
   packs/labeled packs
   rows
   Tuple shape
   const terms
   projections
   mapped row tails

D. Type-member system
   identity/signature
   parameter/result domains
   applicability
   projection
   normalization states
   intrinsic implementation
   associated compatibility

E. Structural algebra
   Positionals/Labeled/At/Map
   negative indexing
   open rows
   zero products
   union non-distribution

F. Generic/inference lifetime
   stable parameters
   ephemeral solver vars
   substitution/materialization
   no leakage

G. Persistence/tooling substrate
   metadata
   module boundary
   fingerprints
   source target
   incremental parity

H. Runtime boundary
   compile-time semantic normalization
   no VM bytecode/solver
   no runtime pack/row objects
```

### 7.2 Cross-layer certification principle

Prove transitions:

```text
syntax → canonical member identity
member identity → declared result domain
projection → reduction or stable residual
residual → substitution/materialization
canonical term → stable metadata
metadata/module boundary → same meaning
incremental edit → same final state as cold build
source alias → same semantic target
intrinsic type operation ↛ VM
```

### 7.3 No parallel architecture

P3 must find no second C1 canonical owner for:

```text
pack identity
labeled-pack identity
row identity
member identity
projection identity
result-domain derivation
Map/At normalization
intrinsic implementation selection
source target identity
stable metadata identity
```

## 8. Ownership boundaries

### 8.1 Owns

P3 owns:

- final as-built architecture inventory;
- invariant/spec/TDR/implementation/evidence matrix;
- dedicated C1 closure tests where evidence is missing;
- metadata/module-boundary proof;
- incremental/cold parity proof;
- source identity closure evidence;
- associated compatibility closure evidence;
- bounded stress/resource evidence;
- runtime-negative/deferred-feature audits;
- bounded corrections required by accepted C1 invariants;
- future-roadmap re-baselining;
- C1 checkpoint closure;
- P3 walkthrough and post-C1 handoff.

### 8.2 Does not own

P3 does not own:

```text
new syntax
new kinds/domains
new type members
general const generics
GATs
type families
pack split/backtracking inference
variadic callable syntax/lowering
Symbol/Range indexing
union distribution
runtime pack/row values
VM type-member execution
general LSP expansion
unrelated trait baseline repair
workspace release certification
benchmark infrastructure without observed regression
```

### 8.3 Source-of-truth table

| Fact | Canonical owner | Forbidden duplicate |
|---|---|---|
| domains | semantic kind/type store | alternate pack/row domain |
| member identity/signature | canonical member registry | selector-string authority |
| projection | canonical semantic projection | feature-local projection |
| Map law | structural spec + intrinsic normalizer | per-domain ad hoc semantics |
| At/subscript identity | one Subscript member | separate At member |
| mapped row tail | stable row projection | query-local row var in durable state |
| metadata | stable metadata graph | formatted-string/local-ID identity |
| source target | source-index semantic target | presentation spelling |
| intrinsic implementation | Intrinsic kind + semantic registry | NativeIntrinsicId inference |
| associated evidence | trait/conformance owner | intrinsic registry |
| runtime type-member behavior | none | VM solver/bytecode |
| lifecycle truth | C1 checkpoint | competing state record |

## 9. Global invariants

`INV-01 — P3 starts only after P1/P2 are COMPLETE / IMPLEMENTED / FOCUSED_TESTED.`

`INV-02 — C1 has one canonical domain system for Type, RecordRow, TypePack, LabeledTypePack, and arrow kinds.`

`INV-03 — Compile-time value arguments such as I: Int are Const(Int), not kinds.`

`INV-04 — TypePack, LabeledTypePack, Tuple, and RecordRow are not interchangeable representations.`

`INV-05 — Type-member identity is declaration/domain identity, not selector text.`

`INV-06 — Member result domain comes from canonical signature.`

`INV-07 — T[I] and T::At<I> denote one canonical Subscript identity.`

`INV-08 — Same-spelled Map members on different domains remain distinct identities.`

`INV-09 — Trait-associated members remain distinct and evidence-owned.`

`INV-10 — Valid abstract projections remain stable symbolic terms; concrete invalidity remains Invalid.`

`INV-11 — Query-local inference/row variables never enter canonical terms, metadata, snapshots, or cross-module identity.`

`INV-12 — Structural Map uses ordinary F<T> with F : Type -> Type.`

`INV-13 — Tuple Map preserves lanes, labels, and order.`

`INV-14 — Record/RecordRow Map preserves keys and maps open tails.`

`INV-15 — Tuple At indexes positionals then labeled payloads and supports accepted negative indexes.`

`INV-16 — Concrete At remains O(1) after shape access; concrete Map remains O(n).`

`INV-17 — Structural operations do not implicitly distribute over unions.`

`INV-18 — Tuple([],[]) ≡ Unit; Record({}) remains a distinct static Record type.`

`INV-19 — Stable metadata preserves semantic structure, not raw local/query IDs.`

`INV-20 — Cross-module consumers observe the same canonical meaning as producers.`

`INV-21 — Incremental final state equals cold final state for representative C1 edits.`

`INV-22 — C1 incremental handling does not use a whole-workspace reset escape hatch.`

`INV-23 — Source aliases/presentations converge on canonical semantic targets.`

`INV-24 — intrinsic remains the source implementation-kind keyword; @intrinsic stays retired.`

`INV-25 — NativeIntrinsicId remains distinct internal metadata.`

`INV-26 — Intrinsic type-member normalization never executes in VM/runtime dispatch.`

`INV-27 — C1 introduces no runtime TypePack/LabeledTypePack/RecordRow object model.`

`INV-28 — C1 introduces no general const-generic, GAT, open type-family, pack-split, or union-distribution surface.`

`INV-29 — Anonymous structural Tuple/Record types remain invalid ordinary inherent-impl targets.`

`INV-30 — Later LANG007 work extends C1 identities/representations rather than reimplementing them.`

`INV-31 — Final architecture/evidence matrix has no unowned invariant and no unsupported PASS.`

`INV-32 — P3 may correct implementation defects but may not decide new user-visible semantics.`

`INV-33 — P3 closure does not imply RELEASE_COMPLETE.`

## 10. Non-goals

Do not expand P3 into:

```text
new language design
new domains or intrinsic members
general const/trait const generics
GATs
open user type families
arbitrary compile-time source execution
const/pack arithmetic
pack split inference
variadic generic/callable lowering
Symbol/Range type indexing
union-distributed At/Map
runtime pack/row models
VM type-member bytecodes
general reflection/LSP redesign
trait baseline cleanup
workspace release certification
historical docs migration
```

## 11. Expected impact map

### 11.1 Preferred new focused tests

If live conventions fit:

```text
phalcom-semantic/tests/semantic/integration/type_member_closure.rs
phalcom-semantic/tests/semantic/incremental/type_members.rs
phalcom-semantic/tests/semantic/integration/mod.rs
phalcom-semantic/tests/semantic/incremental/mod.rs
```

Reuse existing files instead if clearer:

```text
foundations/type_members.rs
integration/metadata.rs
integration/source_index.rs
integration/workspace.rs
incremental/product_stability.rs
incremental/recomputation_metrics.rs
capabilities/traits.rs
```

### 11.2 Potential production corrections

Only if a P3 test exposes an accepted-invariant defect:

```text
phalcom-semantic/src/types/
phalcom-semantic/src/checker/
phalcom-semantic/src/metadata/
phalcom-semantic/src/source_index/
phalcom-semantic/src/identity.rs
phalcom-type-meta/src/
```

### 11.3 Lifecycle/program records

Modify:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-GUIDANCE.md
```

Create:

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P3-walkthrough.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P3-handoff.md
```

### 11.4 Normative specs

P2 already converged them. Modify only for a small unambiguous contradiction proven by P3. Material normative conflict is STOP/CONSULT.

### 11.5 Unexpected-touch rule

STOP before entering VM solving, new parser grammar, new generic syntax, trait-proof redesign, module-linker redesign, cross-program metadata redesign, LSP architecture, or pack-inference representation unless the active evidence proves that path is already the canonical owner and the correction is bounded.

## 12. Implementer decision authority

### 12.1 FIXED

Do not change:

1. P1/P2 domain taxonomy;
2. Const(Int) semantics;
3. member identity/signature rules;
4. one Subscript identity for []/At;
5. structural Map algebra;
6. open-row mapped-tail semantics;
7. Reduced/Stuck/Invalid distinction;
8. zero Tuple/Unit semantics;
9. empty Record static identity;
10. union non-distribution;
11. no-VM execution;
12. intrinsic keyword architecture;
13. NativeIntrinsicId separation;
14. associated-type evidence ownership;
15. no general const generics/GATs/type families;
16. no runtime pack/row model;
17. canonical C1 representations for test convenience;
18. truthful closure metadata.

### 12.2 MECHANICALLY FLEXIBLE

May adapt:

- test file/module placement;
- fixture helpers;
- actual source-index target representation;
- metadata round-trip helper;
- incremental fixture shape;
- dense cardinality;
- test filters;
- private bug-fix decomposition;
- roadmap section names;
- future checkpoint naming without inventing unaccepted scope.

### 12.3 VERIFY-FIRST

| Assumption | Verify | If false |
|---|---|---|
| P1/P2 complete | checkpoint/walkthrough/handoff | STOP |
| main contains P2 completion | git log/records | adapt only compatible later commit |
| type-member focused suite green | smallest P1 regression | classify/fix if C1-owned |
| no dedicated type-member incremental suite | incremental tree | reuse if added |
| metadata supports stable C1 forms | exporter/type-meta/tests | add minimal support if C1-owned |
| source index can express canonical member target | live identity/index | prove representation; STOP if redesign required |
| associated projections remain evidence-owned | trait tests/code | fix accepted regression only |
| supported module boundary preserves C1 meaning | workspace/meta paths | certify actual branch |
| incremental engine supports bounded dependency | incremental harness | STOP if only global reset works |
| At/Map complexity remains direct | normalizer/store | correct only accepted regression |
| future checkpoint allocation open | PROGRAM.md | re-baseline |
| known trait failures still baseline | checkpoint/discriminator | reclassify only with evidence |
| no policy requires workspace gates | lifecycle policy | obey if changed |

## 13. Global STOP / CONSULT triggers

STOP and build an incident packet if:

1. P1/P2 prerequisite truth is false;
2. a C1 invariant cannot be proven without semantic change;
3. member identity must change;
4. result-domain ownership must change;
5. source target cannot represent []/At single identity without redesign;
6. stable metadata needs local/query IDs;
7. cross-module support needs runtime type solving;
8. incremental correctness needs workspace-global reset without bounded hook;
9. associated compatibility needs moving proof authority into intrinsic lookup;
10. no-VM closure needs runtime support;
11. zero-product evidence conflicts with static Record semantics;
12. resource defect needs new canonical representation;
13. two plausible persistence/invalidation fixes have different architecture;
14. a deferred feature becomes necessary for C1;
15. same semantic failure survives one serious correction;
16. tests require weakening an invariant;
17. roadmap re-baselining requires a new language-design choice;
18. scope enters a materially unexpected subsystem;
19. accepted TDR/spec and live behavior materially conflict;
20. C1 closure would require claiming unexecuted release evidence.

## 14. Debugging budget

Mechanical failures: up to three coherent inspect/correct/retest cycles.

Semantic failure: record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same cause; then STOP.

Architectural failure: zero speculative architecture fixes.

Never rerun unchanged failures.

## 15. Testing surface analysis

| Coverage | Dimension | Invariants | Evidence |
|---|---|---|---|
| CV-01 | domain identity | 02–04 | pack/labeled-pack/row distinctions |
| CV-02 | const domain | 03 | Const(Int) |
| CV-03 | member identity | 05–08 | []/At same; Maps distinct |
| CV-04 | associated compatibility | 09 | trait projection evidence |
| CV-05 | symbolic normalization | 10 | generic P/L/R residuals |
| CV-06 | invalidity | 10,15 | OOB/bad mapper Invalid |
| CV-07 | substitution | 11 | projection survives |
| CV-08 | inference lifetime | 11 | no local-ID leakage |
| CV-09 | Tuple Map | 12–13 | lane/order/labels |
| CV-10 | Record Map | 12,14 | keys/open tail |
| CV-11 | At semantics | 15–16 | boundaries/labeled payload |
| CV-12 | Map complexity | 16 | linear traversal |
| CV-13 | zero product | 18 | Unit vs empty Record |
| CV-14 | stable metadata | 19 | stable round-trip/fingerprint |
| CV-15 | module boundary | 20 | downstream same meaning |
| CV-16 | cold/incremental | 21 | final-state parity |
| CV-17 | invalidation | 22 | no global reset |
| CV-18 | source alias | 23 | []/At same target |
| CV-19 | source distinction | 08,23 | domain Maps distinct |
| CV-20 | intrinsic boundary | 24–25 | keyword vs internal ID |
| CV-21 | no VM | 26–27 | negative architecture |
| CV-22 | union | 17 | no distribution |
| CV-23 | deferred surface | 28–29 | negative audit |
| CV-24 | duplicate authority | 02,05,30 | one canonical system |
| CV-25 | dense structure | 16 | bounded moderate density |
| CV-26 | roadmap | 30 | future work reclassified |
| CV-27 | checkpoint truth | 31–33 | complete matrix/lifecycle |
| CV-28 | baselines | 33 | unrelated residuals classified |

## 16. Verification execution budget

### 16.1 Modes

BUILD: smallest discriminator only.

STABILIZE: all P3-added tests plus directly affected P1/P2 suites and owning crate checks.

CERTIFY: complete bounded C1 focused matrix.

Do not automatically run release-wide gates.

### 16.2 Ladder

```text
T0 exact new P3 discriminator
T1 P3 module / affected foundation
T2 integration or incremental module
T3 adjacent C1 focused suites
T4 affected crate checks
T5 workspace/release certification — deferred unless policy requires
```

### 16.3 Expected final focused matrix

Adapt exact filters to live test names:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser intrinsic

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test   -p phalcom-semantic --test semantic foundations::type_members

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test   -p phalcom-semantic --test semantic foundations::type_annotations

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test   -p phalcom-semantic --test semantic foundations::record_row_inference

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test   -p phalcom-semantic --test semantic foundations::generic_application

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test   -p phalcom-semantic --test semantic integration::metadata

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test   -p phalcom-semantic --test semantic integration::native_conformance

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test   -p phalcom-semantic --test semantic integration::type_member_closure

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test   -p phalcom-semantic --test semantic incremental::type_members

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core

cargo fmt --all -- --check
git diff --check
```

If P3 extends existing files rather than creating named modules, substitute exact focused filters.

### 16.4 Associated compatibility

Run the smallest trait/associated suite that proves existing associated projection semantics. If `capabilities::traits` is the only bounded owner, run it and retain only the two known baseline failures if they reproduce exactly.

### 16.5 Only if evidence demands

Broader tests only when P3 changes shared metadata, incremental dependency primitives, source-target representation, or a focused failure shows wider fallout.

### 16.6 Explicitly deferred

Unless policy changed:

```text
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
full language corpus
full VM/runtime corpus
all LSP tests
extensions
broad performance benchmarks
release certification
```

## 17. Baseline / unrelated failure policy

Classify A/B/C/D:

```text
A definitely P3/C1
B probably P3/C1
C unclear
D unrelated/baseline
```

A/B are active. C gets one bounded classification pass. D is recorded and deferred.

For BASELINE-001/002, reuse prior classification unless P3 changes relevant trait dependency/evidence code. If a final associated gate encounters them, record the exact expected baseline result.



# 18. Tasks

## T0 — Activate P3 and Establish the Live Post-P2 Takeover Baseline

### Purpose

Verify prerequisites, establish actual repository state, consume P1/P2 handoffs, activate P3 in the checkpoint, and build the smallest live architecture/evidence inventory needed by later tasks.

### Preconditions

- P1 and P2 completion lock satisfied.
- C1 checkpoint still identifies P3 as next action.
- Working tree can be inspected without discarding unrelated state.

### Consumes

```text
current main
P1 walkthrough/handoff
P2 walkthrough/handoff
C1 checkpoint/guidance
accepted TDRs/specs
```

### Produces

```text
actual P3 start revision
active P3 checkpoint entry
verified stable-owner map
verified current evidence gaps
verified future-roadmap state
```

### Files and symbols

**Read:** all §4 required reads.

**Modify:**

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
docs/implementation/LANG007/STATUS.md
```

Modify `PROGRAM.md` only if current state is stale.

### Required implementation shape

1. Run:

   ```sh
   git status --short
   git branch --show-current
   git rev-parse HEAD
   git log -5 --oneline
   ```

2. Confirm final P1/P2 records.

3. Read P1/P2 handoffs; do not reconstruct their histories.

4. Inspect live C1 owners sufficient to verify:
   - member registry;
   - projection owner;
   - metadata owner;
   - source target owner;
   - incremental session owner;
   - runtime-negative boundary.

5. Confirm whether dedicated closure/incremental tests have appeared since planning.

6. Update plan ledger:

   ```yaml
   P3 status: IN_PROGRESS
   P3 completion: PARTIAL
   P3 verification: UNVERIFIED
   active_plan: LANG007.C1.P3
   ```

7. Record actual starting revision.

### Forbidden approaches

- no production edits;
- no broad rediscovery;
- no rerunning all P1/P2 tests;
- no accepted-design reopening;
- no future-roadmap semantic decisions yet.

### Test changes required

None.

### Tests to run now

If there is no drift signal, no Cargo test is required yet.

If a smoke is useful to validate the live target/filter:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic foundations::type_members
```

### Tests explicitly deferred

All integration/incremental certification until relevant tasks.

### Acceptance

T0 passes when P3 has a trustworthy live takeover and no prerequisite contradiction.

### Local STOP / CONSULT triggers

- P1/P2 lifecycle state is untruthful;
- accepted decision/spec authority changed materially;
- projection/member/source owner no longer matches inherited architecture.

### Checkpoint update

Mandatory: P3 activation and start revision.

---

## T1 — Build the Final C1 Architecture Baseline and Requirement-to-Evidence Matrix

### Purpose

Produce the authoritative as-built C1 architecture inventory and identify only genuine evidence gaps before adding tests or code.

### Preconditions

T0 PASS.

### Consumes

- inherited owner map;
- checkpoint invariants;
- accepted TDR/spec chain;
- existing P1/P2 test evidence.

### Produces

A working matrix with columns:

```text
Invariant ID
semantic statement
normative spec owner
TDR owner
implementation owner
positive evidence
negative evidence
cross-layer evidence
status
future consumer
```

### Files and symbols

**Modify:**

The active checkpoint may carry the compact matrix summary. Full detailed evidence may be recorded in the P3 walkthrough.

Do not create a competing `implementation-state.md`.

### Required implementation shape

Map every P3 `INV-*` and every durable C1 checkpoint invariant.

Classify evidence as:

```text
DIRECT_TEST
CROSS_LAYER_TEST
STATIC_ARCHITECTURE_REVIEW
NEGATIVE_SEARCH
INHERITED_P1/P2_EVIDENCE
MISSING
```

Do not add a new test where existing evidence is already sufficient.

Explicitly identify gaps for:

```text
source alias target identity
domain-distinct Map target identity
cross-module persistence
incremental/cold parity
resource/density
associated projection compatibility
no-VM final audit
roadmap ownership
```

### Forbidden approaches

- no production code;
- no vague PASS from “covered somewhere”;
- no requirement without one canonical owner;
- no broad test-count proxy for proof.

### Test changes required

None.

### Tests to run now

No new broad tests. Use source/test searches.

### Acceptance

Every C1 invariant has adequate evidence or a named P3 task that will produce it.

### Local STOP / CONSULT triggers

- an accepted invariant has no implementation owner;
- two implementations both claim canonical ownership;
- current specs/TDRs disagree materially.

### Checkpoint update

Record the architecture baseline skeleton and genuine evidence gaps.

---

## G1 — Architecture Baseline Gate

### Purpose

Prove P3 is certifying one coherent architecture rather than adding tests around unresolved ownership.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic foundations::type_members
```

Then inspect canonical-owner searches:

```sh
rg -n 'TypeMemberId|TypeMemberSignature|lookup_intrinsic_type_operation' \
  phalcom-semantic/src

rg -n 'TypePack|LabeledTypePack|TypeMemberProjection' \
  phalcom-semantic/src/types phalcom-type-meta/src
```

Expected:

```text
P1 focused member suite passes;
one member/signature authority;
one domain system;
one stable projection/metadata model;
missing evidence can be closed without semantic redesign.
```

Do not broaden after PASS.

---

## T2 — Certify Canonical Member Identity, Source Target Identity, and Associated-Type Compatibility

### Purpose

Close identity/composability evidence that local normalization tests cannot prove.

### Preconditions

G1 PASS.

### Consumes

- canonical member registry;
- source-index target representation;
- trait-associated projection machinery;
- P1 type-member tests.

### Produces

Focused evidence proving:

```text
T[I] and T::At<I>
    same canonical member identity
    same source/semantic target

Tuple::Map
Record::Map
TypePack::Map
LabeledTypePack::Map
RecordRow::Map
    distinct identities despite spelling

trait-associated same-spelled member
    remains trait-owned/evidence-resolved
```

### Files and symbols

Preferred new test module:

```text
phalcom-semantic/tests/semantic/integration/type_member_closure.rs
phalcom-semantic/tests/semantic/integration/mod.rs
```

Reuse `integration/source_index.rs` if clearer.

Potential bounded production correction only if a real accepted-invariant defect is exposed:

```text
phalcom-semantic/src/source_index/
phalcom-semantic/src/identity.rs
phalcom-semantic/src/types/type_member.rs
```

### Required implementation shape

#### A. Alias identity

Construct both accepted presentations if live source support permits:

```phalcom
T[I]
T::At<I>
```

Assert semantic member/target identity is the same. Do not merely compare reduced result types.

#### B. Domain identity

Construct canonical same-spelled structural Map members and assert different identities by owner/domain.

#### C. Associated compatibility

Use an existing or minimal trait fixture to prove contextual/qualified associated projection still resolves via trait evidence.

A same-spelled associated `Map` fixture is desirable if legal and compact.

### Forbidden approaches

- no string-based identity;
- no source-range identity;
- no “same result means same member” proxy;
- no new target enum solely for test aesthetics;
- no associated-type redesign.

### Test changes required

Covers:

```text
CV-03
CV-04
CV-18
CV-19
```

### Tests to run now

Run exact new tests only, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic foundations::type_members
```

### Tests explicitly deferred

Metadata/incremental/stress until later tasks.

### Acceptance

Canonical member identity, source target identity, and trait-associated separation are directly proven.

### Local STOP / CONSULT triggers

- source-target infrastructure cannot represent canonical member identity;
- alias identity requires new semantics;
- structural and associated lookup cannot coexist without ownership redesign.

### Checkpoint update

Record the final identity contract after G2.

---

## G2 — Identity and Compatibility Gate

Run the exact T2 tests and smallest associated-type compatibility suite.

Expected:

```text
[] / At same canonical identity;
domain Map identities distinct;
associated projection remains trait-owned;
no baseline trait issue becomes P3-owned without new evidence.
```

---

## T3 — Certify Stable Metadata and Cross-Module Semantic Preservation

### Purpose

Prove canonical C1 domains/projections survive the supported persistence/module boundary without identity loss or query-local leakage.

### Preconditions

G2 PASS.

### Consumes

- semantic metadata exporter;
- type-meta schema/validator;
- metadata/workspace integration harness;
- canonical C1 terms.

### Produces

One proven persistence branch:

```text
Branch A:
    supported durable/module boundary receives normalized/stable forms;
    raw symbolic projection need not cross

or

Branch B:
    stable symbolic C1 projection crosses metadata and round-trips exactly
```

P3 discovers/certifies the architecture P1 established; it does not choose by preference.

### Files and symbols

Likely tests:

```text
phalcom-semantic/tests/semantic/integration/type_member_closure.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/workspace.rs
```

Potential bounded production correction:

```text
phalcom-semantic/src/metadata/export.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/validate.rs
```

### Required implementation shape

#### A. Stable term discriminator

Use a representative nontrivial C1 term, preferably involving a projection, mapped structural term, or labeled/open structure.

#### B. Round-trip/downstream observation

Assert:

```text
producer semantic meaning
    ==
durable/exported/imported/downstream semantic meaning
```

at canonical semantic level.

#### C. Local-ID negative

Prove durable identity does not depend on:

```text
raw TypeId store index
InferVarId
RecordRowVarId
source span
formatted type string
```

Use validator/fingerprint structure and static review where direct serialization inspection is stronger.

#### D. Cross-module

Use the existing workspace/module integration harness. Do not create a bespoke module framework.

### Forbidden approaches

- no runtime solving on import;
- no reparsing formatted type strings;
- no metadata-owned inference;
- no serializing local/query IDs;
- no schema expansion unless the supported boundary actually needs it.

### Test changes required

Covers:

```text
CV-07
CV-08
CV-14
CV-15
```

### Tests to run now

Exact new metadata/cross-module test, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic integration::metadata
```

Run `cargo test -p phalcom-type-meta` only if type-meta production code changed or there is a directly relevant focused target.

### Acceptance

The module/persistence boundary has a proven stable architecture preserving C1 identity.

### Local STOP / CONSULT triggers

- public/module boundary needs symbolic forms but metadata cannot represent them without redesign;
- imported meaning requires runtime solving;
- stable metadata requires local IDs.

### Checkpoint update

Record Branch A/B and durable metadata contract.

---

## G3 — Persistence Gate

Run exact T3 discriminator plus existing metadata integration suite.

Expected:

```text
stable identity survives;
no local-ID leakage;
no runtime solver;
metadata branch recorded.
```

---

## T4 — Add Dedicated C1 Incremental / Cold-Parity Certification

### Purpose

Close the current evidence gap for C1 incremental correctness and prove representative structural type-member edits converge to the same final state as a cold rebuild.

### Preconditions

G3 PASS.

### Consumes

- semantic incremental session/harness;
- C1 canonical terms;
- relevant source/metadata products.

### Produces

A dedicated incremental module or equivalent focused coverage.

### Files and symbols

Preferred:

```text
phalcom-semantic/tests/semantic/incremental/type_members.rs
phalcom-semantic/tests/semantic/incremental/mod.rs
```

Reuse:

```text
incremental/support.rs
incremental/product_stability.rs
incremental/recomputation_metrics.rs
incremental/type_store_revisions.rs
```

Potential correction only if a C1-owned invalidation bug is exposed:

```text
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/
semantic query/dependency code
```

### Required implementation shape

#### A. Result-changing structural edit

Use a supported fixture equivalent to:

```text
before:
    structural At result == String

after:
    structural At result == Bool
```

or a Map/row edit if that is more natural through current source APIs.

Assert:

```text
incremental final canonical result
    ==
cold final canonical result
```

#### B. Symbolic/open-structure edit

If inexpensive, add an edit that preserves symbolic structure while changing a pack/row/mapper dependency.

#### C. Product parity

Where normal APIs expose them, compare:

```text
diagnostics
semantic type
source target
metadata/fingerprint
```

Do not compare unstable local IDs.

#### D. Invalidation scope

Use existing stats/reuse counters to prove no prohibited whole-workspace reset.

Do not require arbitrary exact reuse counts unless the existing incremental contract already does.

### Forbidden approaches

- no rebuilding a new session in the incremental branch;
- no global invalidation escape hatch;
- no new incremental engine;
- no raw local-ID equality across sessions.

### Test changes required

Covers:

```text
CV-16
CV-17
CV-20
CV-21
```

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::type_members
```

If shared incremental primitives changed, run the smallest owning adjacent incremental module.

### Acceptance

Representative C1 edits converge cold/incrementally with dependency-scoped recomputation.

### Local STOP / CONSULT triggers

- only a whole-workspace reset yields correctness;
- cold/incremental semantic identity differs after one serious correction;
- dependency ownership requires a broad module-architecture change.

### Checkpoint update

Record cold/incremental parity and invalidation contract.

---

## G4 — Incremental Gate

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic incremental::type_members
```

plus any adjacent incremental tests changed.

Expected:

```text
cold/incremental parity;
no local-ID comparison;
no global-reset workaround.
```

---

## T5 — Certify Structural Algebra Composition, Zero Products, Resource Bounds, and Runtime-Negative Boundary

### Purpose

Add/consolidate high-density and edge evidence proving the C1 calculus is mathematically complete within scope and stays bounded/no-VM.

### Preconditions

G4 PASS.

### Consumes

- P1 foundation tests;
- structural normalizer;
- zero-product model;
- runtime boundary;
- accepted laws.

### Produces

Focused closure coverage for:

```text
At/Map composition
Record lookup/Map composition where available
negative/boundary At
empty Tuple
empty Record
open-row Map
dense pack/Tuple/Record Map
no union distribution
no VM/runtime type operation
```

### Files and symbols

Prefer extending:

```text
phalcom-semantic/tests/semantic/foundations/type_members.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
```

or use the P3 closure integration module if clearer.

### Required implementation shape

#### A. Tuple law

For valid `I`:

```text
(T::Map<F>)::At<I>
    ≡
F<T::At<I>>
```

Assert canonical semantic equality.

#### B. Record law

Where exact field lookup API exists:

```text
Lookup(Record(R)::Map<F>, K)
    ≡
F<Lookup(Record(R), K>>
```

If no direct semantic lookup API exists, prove key/shape/payload mapping instead; do not invent new API.

#### C. Index boundaries

Ensure evidence covers:

```text
0
last positive
-1
-most-negative-valid
first positive OOB
first negative OOB
labeled payload position
```

Reuse P1 cases rather than duplicating them.

#### D. Zero products

Prove:

```text
Tuple([],[]) -> Unit
Record({}) stays static Record
Map preserves each corresponding static meaning
```

#### E. Moderate density

Use a deterministic cardinality around 64–256 elements, adapted to existing test cost.

Prove by functional assertion + code inspection:

```text
At uses direct indexed shape access
Map performs one payload traversal
normalization has no branch search
```

No timing thresholds.

#### F. Runtime-negative audit

Search/review for:

```text
type-member bytecode
runtime projection solver
runtime TypePack object
runtime RecordRow object
message send for static At/Map
VM intrinsic type-member entry
```

Do not confuse ordinary runtime Tuple collection/indexing behavior with type-level operations.

### Forbidden approaches

- no timing-flaky benchmarks;
- no Option-specific Map behavior;
- no Map fusion;
- no union distribution;
- no runtime execution as type normalization;
- no empty Record -> Unit static collapse.

### Test changes required

Covers:

```text
CV-06
CV-09
CV-10
CV-11
CV-12
CV-13
CV-21
CV-22
CV-25
```

### Tests to run now

Exact new closure tests, then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic --test semantic foundations::type_members
```

Run `type_model` only if changed.

### Acceptance

All required structural laws/boundaries have evidence, dense cases remain bounded by architecture, and no-VM audit is clean.

### Local STOP / CONSULT triggers

- accepted algebra law fails;
- density exposes representation complexity needing architecture change;
- a real runtime type-member path exists;
- empty Record static identity is lost.

### Checkpoint update

Record final structural/resource/no-VM closure facts.

---

## G5 — Structural and Runtime-Negative Gate

Run P3 structural closure tests plus P1 type-member foundations and no-VM search/review.

Expected:

```text
C1 structural laws green;
zero-product distinction preserved;
density bounded;
no type-member VM/runtime path.
```

---

## T6 — Audit Deferred-Surface Boundaries and Duplicate Authority

### Purpose

Prove C1 did not accidentally expose future features or leave competing canonical implementations.

### Preconditions

G5 PASS.

### Consumes

Final P3 code/tests plus P2 specs and accepted TDRs.

### Produces

A classified negative-audit matrix.

### Required implementation shape

#### A. Deferred feature audit

Inspect for unintended support of:

```text
general nominal const generics
trait const generics
GATs
parameterized associated requirements
open user type families
arbitrary compile-time user execution
pack split/backtracking
variadic callable lowering
Symbol/Range type indexing
union-distributed members
runtime pack/row values
anonymous structural inherent impl
```

Internal representational generality is not automatically public feature support.

#### B. Duplicate authority audit

Audit for a second canonical owner of:

```text
pack identity
member identity
projection identity
result-domain derivation
Map/At normalization
source target identity
metadata identity
intrinsic implementation selection
```

Textual helper duplication is not itself architectural duplication.

#### C. Intrinsic boundary

Confirm:

```text
ImplementationKind::Intrinsic -> source implementation kind
NativeIntrinsicId -> internal metadata
@intrinsic -> retired
```

### Forbidden approaches

- no deleting useful internal abstractions due naming overlap;
- no historical-doc mass cleanup;
- no converting audit hits into feature work without proof.

### Tests to run now

No Cargo tests unless audit finds a concrete C1-owned defect.

### Acceptance

No deferred feature leaks into C1 public semantics and no duplicate canonical system remains.

### Local STOP / CONSULT triggers

- a deferred public feature is intentionally implemented/specified;
- two real canonical owners exist;
- resolving duplication needs a broad representation migration.

### Checkpoint update

Record residual deferred surface and future ownership classification.

---

## T7 — Re-baseline the Remaining LANG007 Roadmap

### Purpose

Replace stale pre-C1 assumptions with an as-built capability-gap map so future checkpoints extend C1 rather than reimplement it.

### Preconditions

T6 PASS.

### Consumes

- final architecture/evidence matrix;
- `PROGRAM.md`;
- old roadmap concepts;
- accepted non-goals.

### Produces

Classification:

```text
ALREADY OWNED BY C1
PARTIALLY ESTABLISHED / EXTENDABLE
NOT YET IMPLEMENTED
DEFERRED / NOT YET ACCEPTED
```

### Files and symbols

Modify:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
```

Potentially update `LANG007.C1-GUIDANCE.md` with post-C1 do-not-redesign guidance.

Do not create a new checkpoint plan unless separately requested.

### Required implementation shape

Classify at least:

```text
TypePack/LabeledTypePack domains
canonical whole-pack terms
Const(Int) member args
type-member identity/signatures
symbolic projections
At/Subscript
structural Map
open-row Map
substitution/materialization
stable metadata
source identity
incremental substrate
pack expansion syntax
pack decomposition
prefix/suffix matching
unknown-length pack inference
variadic generic declaration/application integration
broader const-generic admission
additional structural operators
reflection/tooling beyond source-index substrate
adoption/performance/release stabilization
```

Do not blindly restore old C2–C7 scope labels.

If repository convention expects `LANG007.C2` as the next ID, describe only a **candidate bounded objective based on remaining gaps**, not an implementation plan or accepted semantics.

### Forbidden approaches

- no reimplementation assignment for C1-owned work;
- no ratification of deferred semantics through roadmap prose;
- no detailed C2 implementation design;
- no new TDR unless an already-decided omission is discovered; that would require consultation.

### Tests to run now

None.

### Acceptance

Future implementers cannot mistake C1-owned capabilities for future implementation work; every remaining topic has clear status.

### Local STOP / CONSULT triggers

- choosing the next checkpoint requires a user-visible semantic decision;
- two roadmaps imply different accepted language scope;
- a newer accepted checkpoint allocation already exists.

### Checkpoint update

Record final C1-to-future ownership boundary and next action.

---

## G6 — Roadmap and Scope Gate

Review:

```text
PROGRAM.md
STATUS.md
C1 checkpoint
P3 architecture/evidence matrix
```

Expected:

```text
C1-owned substrate explicit;
future work does not duplicate it;
deferred semantics remain unaccepted;
next action explicit without over-planning.
```

---

## T8 — Final C1 Focused Certification, Checkpoint Closure, Walkthrough, and Handoff

### Purpose

Run the final bounded C1 evidence matrix, close P3-owned failures, mark C1 complete only if truthful, and create the durable post-C1 handoff.

### Preconditions

G1–G6 PASS.

### Consumes

- complete P3 matrix;
- P1/P2 evidence;
- P3 tests/audits;
- baseline ledger;
- roadmap re-baseline.

### Produces

```text
LANG007.C1.P3 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
LANG007.C1 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
updated program/status/guidance
P3 walkthrough
P3 handoff
explicit next action
```

### Files and symbols

Modify:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1-GUIDANCE.md
```

Create:

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P3-walkthrough.md
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P3-handoff.md
```

### Required implementation shape

#### A. Coverage audit

Map every `CV-01..CV-28` to:

```text
test
static architecture review
negative search
inherited P1/P2 evidence
```

No `MISSING` row may remain.

#### B. Final focused matrix

Run §16.3 commands serially plus the exact associated compatibility gate chosen in T2.

If P3 production changes touched shared metadata/source-index/incremental code, run the smallest additional owning suite justified by the diff.

#### C. Baselines

If trait baseline failures appear exactly as recorded, retain them. Any new failure is classified A/B/C/D.

#### D. Hygiene

Run:

```sh
cargo fmt --all -- --check
git diff --check
git status --short
```

Review scoped diff for forbidden future feature work and duplicate architecture.

#### E. Checkpoint closure

Only if all obligations pass:

```yaml
LANG007.C1.P3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
  active_plan: null
```

Do not set `RELEASE_COMPLETE`.

#### F. Walkthrough

Record:

```text
actual revisions
as-built architecture baseline
acceptance matrix
tests added
bounded corrections
source identity result
associated compatibility
metadata persistence branch
cross-module evidence
incremental/cold parity
resource/no-VM evidence
deferred-feature audit
roadmap re-baseline
commands/tests run
tests deferred
baseline failures
consultations
final lifecycle state
```

#### G. Handoff

Include:

```text
final revision/state
canonical decision/spec chain
stable domains
member/projection ownership
source identity contract
intrinsic/NativeIntrinsicId boundary
Map/At laws
metadata/module contract
incremental contract
runtime-negative contract
baseline residuals
C1-owned capabilities — DO NOT REIMPLEMENT
remaining capability gaps
candidate next objective if already accepted
must-read files
first commands
do-not-redesign list
```

### Forbidden approaches

- no C1 closure with missing evidence;
- no release-certification claim;
- no unrelated baseline repairs for aesthetics;
- no future feature implementation;
- no rewriting P1/P2 walkthrough history.

### Acceptance

P3 and C1 close only when every coverage obligation has evidence, all P3-owned failures are resolved, and records agree.

### Local STOP / CONSULT triggers

- final matrix exposes an unimplemented C1 invariant;
- closure test requires new semantics;
- mandatory focused gate is baseline-blocked in a way that changes lifecycle classification;
- next action cannot be stated without a new language-design decision.

### Checkpoint update

Mandatory final closure update.

# 19. Verification gates

| Gate | Purpose |
|---|---|
| G1 | architecture baseline / single authority |
| G2 | member/source identity + associated compatibility |
| G3 | metadata / cross-module persistence |
| G4 | cold/incremental parity |
| G5 | structural algebra / resource / no-VM |
| G6 | roadmap and deferred-scope correctness |
| G7 | final C1 focused certification and closure |

## G7 — Final C1 Closure Gate

Run the final §16.3 matrix, exact P3-added tests, exact associated compatibility gate, and hygiene commands.

Expected:

```text
all P3-owned tests PASS;
all inherited P1/P2 focused regressions PASS;
only explicitly accepted unrelated baselines may remain;
no deferred feature appears in P3 diff;
checkpoint/program/status records agree;
C1 may truthfully become COMPLETE / IMPLEMENTED / FOCUSED_TESTED.
```

Do not run workspace release certification unless repository policy or the user explicitly requires it.

# 20. Final focused acceptance

P3 may close C1 only when all conditions below hold.

## Architecture baseline

- [ ] one domain system;
- [ ] one member/signature authority;
- [ ] one projection authority;
- [ ] one structural intrinsic evaluator;
- [ ] one metadata identity model;
- [ ] one source-target authority;
- [ ] one incremental dependency model;
- [ ] no runtime type-member authority.

## Identity and composability

- [ ] `[]` and `At` same member/target;
- [ ] domain Maps distinct;
- [ ] associated members distinct/evidence-owned;
- [ ] symbolic projections survive substitution/materialization;
- [ ] mapper application uses ordinary type application;
- [ ] Map/At composition proven.

## Persistence

- [ ] metadata branch recorded;
- [ ] no raw local/query IDs in durable identity;
- [ ] supported cross-module boundary preserves meaning;
- [ ] fingerprints/validation coherent.

## Incremental

- [ ] representative edit has cold/incremental parity;
- [ ] source/semantic products converge;
- [ ] no global-reset workaround.

## Structural completeness

- [ ] positive/negative At boundaries;
- [ ] labeled payload indexing;
- [ ] Tuple Map shape;
- [ ] Record/open-row Map;
- [ ] zero Tuple/Unit;
- [ ] empty Record static identity;
- [ ] union non-distribution.

## Runtime/resource

- [ ] no VM/runtime type-member execution;
- [ ] no runtime pack/row objects;
- [ ] At remains direct/O(1);
- [ ] Map remains linear;
- [ ] moderate dense cases pass;
- [ ] no exponential symbolic search.

## Scope integrity

- [ ] no general const generics;
- [ ] no GATs;
- [ ] no open type families;
- [ ] no pack split/backtracking;
- [ ] no Symbol/Range type indexing;
- [ ] no structural impl expansion;
- [ ] no future feature implementation in P3.

## Roadmap

- [ ] old roadmap topics classified against C1;
- [ ] C1 capabilities not assigned for reimplementation;
- [ ] remaining gaps explicit;
- [ ] next action does not invent semantics.

## Verification/lifecycle

- [ ] CV-01..CV-28 all mapped;
- [ ] P1/P2 regressions pass;
- [ ] P3 integration/incremental tests pass;
- [ ] affected crates check;
- [ ] format/diff hygiene pass;
- [ ] baselines classified;
- [ ] P3 walkthrough exists;
- [ ] P3 handoff exists;
- [ ] C1 checkpoint complete truthfully;
- [ ] no `RELEASE_COMPLETE` claim without release gates.

# 21. Performance/resource evidence

P3 requires architectural + functional performance evidence, not benchmarks:

```text
At:
    direct component lookup, O(1) after shape access

Map:
    one linear payload traversal, O(n)

symbolic normalization:
    bounded canonical residual formation
    no branch-exponential search

metadata:
    stable graph/canonical references
    no formatted-string identity

incremental:
    dependency-scoped recomputation
    no C1-specific workspace-global invalidation
```

Use code inspection and moderate deterministic density tests. Add profiling only for an observed regression.

# 22. Checkpoint bookkeeping

## At P3 start

- record start revision;
- add P3 plan ledger row;
- mark P3 active;
- preserve P1/P2 historical outcomes;
- retain baseline ledger.

## During P3

Update only at durable events:

```text
architecture baseline accepted
source/member identity gate passed
metadata branch fixed
incremental parity established
resource/no-VM closure established
consultation/amendment
new deferred baseline
roadmap re-baseline
```

## At P3 completion

Record:

```text
P3 lifecycle state
C1 final lifecycle state
architecture owner map
acceptance matrix summary
metadata branch
incremental contract
source identity contract
runtime-negative contract
baseline failures
consultations
deferred release gates
future roadmap classification
next action
```

# 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P3-walkthrough.md
```

Required content:

1. plan identity/final state;
2. actual revisions;
3. objective/outcome;
4. final C1 architecture baseline;
5. invariant/evidence matrix;
6. tests added/changed;
7. bounded corrections;
8. source identity result;
9. associated compatibility;
10. metadata persistence branch;
11. cross-module result;
12. incremental parity;
13. resource/no-VM result;
14. deferred-feature/duplicate audit;
15. roadmap re-baseline;
16. consultations;
17. tests actually run;
18. tests deferred;
19. baseline failures;
20. final verification classification;
21. checkpoint closure result.

Never claim evidence not actually obtained.

# 24. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C1/LANG007.C1.P3-handoff.md
```

This is the durable post-C1 takeover.

Must contain:

```text
final revision
C1 lifecycle state
TDR/spec chain
stable domain owner map
member/projection owner map
source identity contract
intrinsic/NativeIntrinsicId boundary
Map/At laws
metadata/module contract
incremental contract
zero-product/runtime boundary
known baselines
C1-owned capabilities — DO NOT REIMPLEMENT
remaining capability gaps
future-roadmap classification
next accepted objective if one exists
must-read files
first commands
do-not-redesign list
```

Keep it compact.

# 25. Completion truth table

Do not conflate:

```text
P3 tests written
P3 tests pass
P1/P2 regressions pass
cross-module persistence proven
incremental parity proven
architecture matrix complete
C1 accepted
next program action defined
workspace release certified
```

Intended normal result:

```yaml
LANG007.C1.P3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

`RELEASE_COMPLETE` remains false unless separately evidenced.

# 26. Plan self-review

## Architecture

- certifies rather than redesigns P1/P2;
- source → semantic → persistence → incremental → runtime boundaries explicit;
- domain/member/projection owners singular;
- trait-associated and intrinsic authority distinct;
- runtime outside type-level normalization;
- roadmap re-baselining prevents duplicate future implementation.

## Coverage

Explicitly covers:

```text
domains
Const(Int)
member identity
[] / At
domain Map identities
associated compatibility
symbolic projections
invalidity
substitution
inference lifetime
Tuple/Record/open-row Map
At boundaries
zero products
metadata
module boundary
incremental parity
invalidation
source targets
intrinsic keyword boundary
no VM
union boundary
deferred surfaces
duplicate authority
density/resource behavior
roadmap ownership
checkpoint truth
baselines
```

## Testing

- BUILD uses discriminating tests;
- P3 adds missing cross-layer evidence;
- final C1 matrix is broader than BUILD but narrower than release certification;
- baseline failures remain classified;
- no benchmark ceremony without regression.

## Escalation

- identity/persistence/incremental/runtime ownership changes are RED;
- one serious semantic correction maximum;
- Luna cannot invent new semantics;
- roadmap semantic choices remain architect/user decisions.

## Lifecycle

- P3 activates in shared checkpoint;
- P3 closes C1 only if truthful;
- walkthrough/handoff mandatory;
- checkpoint stays state authority;
- future scope re-baselined, not implemented.

# 27. Compact implementer directive

```text
Start only from completed P1/P2.

Do not design new language features.

Freeze the as-built C1 architecture and map every invariant to its spec, TDR,
implementation owner, and evidence.

Close missing cross-layer evidence:
- [] == At canonical member/source identity;
- distinct domain Map identities;
- associated-type compatibility;
- stable metadata and supported module boundary;
- representative cold/incremental parity;
- structural algebra, zero products, moderate density, no-VM behavior.

Fix only defects that violate accepted C1 semantics. Escalate any identity,
persistence, incremental-ownership, runtime, or language-semantics fork.

Audit deferred-feature leakage and duplicate canonical systems.

Re-baseline future LANG007 work so later checkpoints extend C1 rather than
reimplement domains, members, projections, Map/At, metadata, or source identity.

Run the bounded final C1 matrix, preserve unrelated baseline classifications,
mark C1 COMPLETE / IMPLEMENTED / FOCUSED_TESTED only if every coverage row has
evidence, and create the P3 walkthrough plus post-C1 handoff.

Do not claim RELEASE_COMPLETE unless release-wide gates actually run.
```
