---
id: LANG007.C3
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: checkpoint-record
title: "Type Data and Unified Algebraic Static Domains"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
requires:
  - LANG007.C1
  - LANG007.C2
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
active_plan: LANG007.C3.P1
latest_completed_plan: null
next_plan: LANG007.C3.P2
---

# LANG007.C3 — Checkpoint Record

## Type Data and Unified Algebraic Static Domains

## 0. Checkpoint state

```yaml
program: LANG007
checkpoint: LANG007.C3
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG007.C3.P1
latest_completed_plan: null
next_plan: LANG007.C3.P2
blockers: none known at planning time
```

This checkpoint record is the single durable state authority for C3 once implementation begins.

The checkpoint must be changed to `IN_PROGRESS` when P1 starts, not merely because planning documents exist.

---

## 1. Objective

Establish one coherent algebraic declaration and static-domain system in which:

```text
data
    defines runtime closed algebraic domains
    including products, sums, ADTs, and GADTs

type data
    defines type-system-only closed algebraic/static domains

associated type data
    participates in the declaration-owned :: namespace

intrinsic data / intrinsic type data
    expose compiler-owned structural domains through ordinary language semantics
```

while preserving:

- exact-case/GADT behavior;
- C1/C2 pack and type-member identity;
- generic binder-mode semantics;
- row separation;
- metadata stability;
- incremental correctness;
- runtime representation freedom;
- the no-runtime-static-solver boundary.

---

## 2. Bounded acceptance statement

C3 is complete only when this statement is true:

> Runtime products and closed sums use one canonical `data` algebraic declaration model; `type data` provides nominal canonical static domains and terms; generic/static application, inference, substitution, aliases, associated lookup, metadata, imports, and incremental analysis preserve user static-domain identity; compiler intrinsic structural domains are exposed through that same model without duplicating C1/C2 terms; tooling and canonical source have migrated from `enum`; and no VM static-term execution authority has been introduced.

---

## 3. Ownership boundary

### 3.1 C3 owns

- unified runtime algebraic declaration syntax/semantic identity;
- `enum` source migration/removal;
- product/sum shape classification;
- runtime case syntax under `data`;
- `type data`;
- nominal static-domain identity;
- canonical type-data static terms;
- compile-time static Int term integration needed by C3;
- heterogeneous generic/static argument application;
- domain-directed non-row inference over user static domains;
- associated `type data`;
- associated static-domain lookup through `::`;
- stable metadata for C3 domains/terms/applications;
- cross-module/incremental identity;
- intrinsic structural domain source surface;
- Universe/corpus migration;
- C3 LSP/source-index closure;
- specification migration;
- checkpoint certification.

### 3.2 C3 does not own

- arbitrary compile-time execution;
- a macro system;
- a theorem prover;
- user-defined primitive kinds;
- user-defined static equality hooks;
- arbitrary type-level pattern functions;
- trait requirements that bind whole `type data` domains;
- GAT expansion beyond existing accepted semantics;
- a general trait static-domain generic redesign unless separately amended;
- automatic lifting of runtime values into static terms;
- new pack decomposition/backtracking;
- runtime ArgumentPack ABI redesign;
- runtime type-data values;
- VM static-domain solving;
- forced unification of Tuple/Record/data runtime storage;
- release-wide unrelated cleanup.

---

## 4. Plans ledger

| Plan | Scope | Status | Completion | Verification | Outcome |
|---|---|---|---|---|---|
| `LANG007.C3.P1` | Unified algebraic `data`; enum semantic migration; top-level `type data` foundation | IN_PROGRESS | PARTIAL | UNVERIFIED | active |
| `LANG007.C3.P2` | General static domains; heterogeneous application/inference; associated `type data`; metadata/import/incremental | PROPOSED | NOT_STARTED | UNVERIFIED | waits on P1 |
| `LANG007.C3.P3` | Intrinsic structural surface; Universe/corpus/tooling migration; certification and closure | PROPOSED | NOT_STARTED | UNVERIFIED | waits on P2 |

No additional P4 is planned.

If implementation reveals a bounded corrective closure needed for C3 acceptance, add a new plan in this checkpoint rather than falsely closing P3.

---

## 5. Predecessor state

### 5.1 LANG007.C1

C1 established the type-level domain/member substrate C3 relies on.

C3 must preserve:

- canonical `TypeId`/`KindId`;
- `ProperTypeId`;
- parameterized type-member identity;
- structural Tuple/Record operations;
- RecordRow separation;
- compiler-owned closed normalization.

### 5.2 LANG007.C2

C2 is:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

Inherited C2 facts:

```text
*   positional
**  labeled
*** complete

TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
CallableFromPacks

GenericParameterMode
GenericBinderHeader
checked substitution/materialization
whole-pack InferVarId(KindId) inference
metadata schema v4 scoped packs
cold/incremental parity
no runtime pack solver
```

Do not redesign these.

---

## 6. Repository baseline

Planning baseline:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   62a6082ab463387c812d558e5561508267ecc63c
```

Relevant post-C2 observation:

- C2 checkpoint records are complete.
- Future-facing `PROGRAM.md` / `STATUS.md` contain some stale C2 next-action wording.
- P1 start must rebaseline active program state to C3.

This planning document does not rewrite historical C2 records.

---

## 7. Inherited established invariants

These are already established before C3 begins.

### 7.1 Canonical type/static substrate

```text
C2-I01 — TypeId is a store-relative canonical type-level term handle.
C2-I02 — ProperTypeId proves the Type domain.
C2-I03 — KindId classifies canonical TypeId terms.
C2-I04 — RecordRow remains a separate row-domain representation/solver.
C2-I05 — LabeledTypePack is not RecordRow.
C2-I06 — CompleteTypePack is not Tuple.
```

### 7.2 Packs

```text
C2-I07 — * means positional lane.
C2-I08 — ** means labeled lane.
C2-I09 — *** means complete lane.
C2-I10 — ... is not pack syntax.
C2-I11 — GenericParameterMode is independent from KindId.
C2-I12 — Single(TypePack) != PositionalPack(TypePack).
C2-I13 — whole-pack inference uses ordinary InferVarId(KindId).
C2-I14 — no pack partition/decomposition solver exists.
C2-I15 — C2 materialization re-enters canonical TypeStore constructors.
```

### 7.3 Persistence/runtime

```text
C2-I16 — stable metadata is structural, not source-string identity.
C2-I17 — query-local inference IDs do not enter durable products.
C2-I18 — cold and incremental pack semantics converge.
C2-I19 — no runtime pack object/opcode/solver exists.
```

---

## 8. Required C3 acceptance invariants

These are requirements, not yet implementation facts.

They become "established" in this checkpoint only after the owning plan implements and verifies them.

```text
C3-I01 — `data` is the sole canonical runtime algebraic declaration category.
C3-I02 — `type data` is semantic phase TypeOnly with no runtime inhabitants.
C3-I03 — phase, shape, implementation provenance, and ownership are orthogonal.
C3-I04 — Product/Sum shape, not keyword category, triggers algebraic semantics.
C3-I05 — runtime sum exact-case and GADT semantics are preserved.
C3-I06 — one algebraic semantic authority owns product/case structure.
C3-I07 — TypeId remains canonical carrier for non-row kinded static terms.
C3-I08 — ProperTypeId remains the proper-Type proof boundary.
C3-I09 — user static domains use nominal StaticDomain identity, not per-domain Rust enum cases.
C3-I10 — StaticArgument is the heterogeneous application envelope.
C3-I11 — RecordRow remains row-owned under heterogeneous applications.
C3-I12 — GenericParameterMode stays orthogonal to static domain.
C3-I13 — non-row static inference reuses InferVarId classified by canonical domain.
C3-I14 — type-data unification is closed structural first-order unification.
C3-I15 — runtime equality/dispatch never defines static identity.
C3-I16 — valid unresolved static terms remain symbolic.
C3-I17 — associated type-data is declaration-owned, not trait-evidence-owned.
C3-I18 — one associated namespace authority owns collision/lookup semantics.
C3-I19 — associated domain specialization preserves owner environment.
C3-I20 — legacy enum compatibility never creates distinct semantic identity.
C3-I21 — TupleEntries is the public surface over C2 CompleteTypePack.
C3-I22 — intrinsic declarations do not replace existing C1/C2 canonical term owners.
C3-I23 — metadata stores stable declaration/domain/static-argument structure.
C3-I24 — incremental and cold results converge for C3 edits.
C3-I25 — type-data inhabitants require no VM value/object/opcode/solver.
C3-I26 — runtime algebraic storage remains representation-independent.
C3-I27 — LSP/source tools consume semantic products, not rendered-name reconstruction.
C3-I28 — current authoritative specs are coherent before checkpoint closure.
```

---

## 9. Stable takeover map

| Concept | Live owner | C3 rule |
|---|---|---|
| canonical type-level term | `TypeStore` / `TypeId` | extend; do not duplicate |
| proper type proof | `ProperTypeId` | preserve strictly |
| kind/domain classifier | `KindId` / `KindData` | extend with nominal static-domain form |
| row term | `RecordRowId` / row subsystem | preserve separate |
| generic binder mode | `GenericParameterMode` | preserve orthogonal |
| generic early contract | `GenericBinderHeader` | extend domain support |
| generic signature | `GenericSignature` | extend domain support |
| non-row inference var | `InferVarId(KindId)` | reuse |
| runtime product | `DataInfo` / `DataSemanticTable` | derive from unified algebraic authority |
| runtime sum | `EnumInfo` / `VariantInfo` | derive/reuse under unified algebraic authority |
| exact case | `VariantId` + `TypeData::ExactCase` | preserve identity; neutral terminology may wrap |
| GADT environment | `CaseTypeEnvironment` | preserve |
| associated lookup | `AssociatedSurface` / checker associated resolver | extend |
| type-member projection | type-member subsystem | integrate with common static domains |
| packs | C1/C2 TypeStore constructors | preserve |
| complete Tuple entries | `CompleteTypePack` | expose as `TupleEntries` |
| metadata | `phalcom-type-meta`, schema v4 baseline | version explicitly for C3 |
| Universe identity | canonical Universe modules | migrate source surface; keep identity |
| runtime data/sum lowering | `phalcom-core` specialized lowering | preserve representation freedom |

---

## 10. Architecture decisions adopted for planning

### A. Canonical term carrier

Use:

```text
TypeId + KindId
```

for all non-row canonical static terms, including new const/type-data terms.

Do not introduce a second general static-term arena.

### B. User static domain classifier

Use one dynamic nominal classifier equivalent to:

```rust
KindData::StaticDomain(StaticDomainId)
```

Do not add one compiler enum variant per source declaration.

### C. Heterogeneous application

Use a canonical envelope equivalent to:

```rust
StaticArgument::Term(TypeId)
StaticArgument::RecordRow(RecordRowId)
```

and migrate `Applied`, scoped application, substitution, metadata, and formatting.

### D. Inference

Reuse:

```text
InferVarId(kind)
```

for non-row user static domains.

Keep row inference separate.

### E. Algebraic declarations

Use one semantic declaration product/facade with:

```text
phase
shape
generic contract
product/case structure
```

DataInfo/EnumInfo may remain compatibility projections.

### F. Associated domains

Extend `AssociatedSurface`.

Do not create an independent static-domain namespace.

### G. Trait boundary

Whole associated `type data` requirements are out of scope.

User static-domain trait generics are explicitly unsupported until a plan proves safe conformance/coherence migration.

### H. Runtime boundary

Only Runtime algebraic data lower to runtime inhabitants.

TypeOnly terms never do.

---

## 11. P1 acceptance contract

`LANG007.C3.P1` must establish:

```text
source:
    canonical data Product/Sum
    type data Product/Sum
    legacy enum -> same algebraic AST during compatibility

semantics:
    one algebraic declaration authority
    preserved DataInfo/EnumInfo derived behavior
    top-level nominal StaticDomainId
    canonical type-data terms

soundness:
    ProperTypeId boundary
    static payload validity
    no runtime type-data inhabitants

runtime:
    exact-case/GADT/match/impl behavior preserved

persistence:
    first stable C3 algebraic/static-domain metadata
```

P1 must not complete heterogeneous generic application or associated type data.

---

## 12. P2 acceptance contract

`LANG007.C3.P2` must establish:

```text
StaticArgument
heterogeneous Applied
generic/static binder domains
type-data/const static arguments
domain-directed inference
checked substitution/materialization
static aliases/result domains
associated type data
unified :: lookup
metadata/import round trip
cold/incremental parity
source-index identity
explicit trait boundary
```

Representative source must work:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

data Tensor<Element: Type, S: Shape>()

type V: Type = Tensor<Float, Shape::Vector(3)>

class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

---

## 13. P3 acceptance contract

`LANG007.C3.P3` must establish and certify:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

and:

```text
TupleEntries == CompleteTypePack semantic domain

*T   == T::Positionals
**T  == T::Labeled
***T == T::Entries
```

where valid.

P3 also owns:

- Option/Result/Ordering migration to `data`;
- final enum compatibility removal/finalization;
- LSP tooling;
- corpus/examples/active docs migration;
- authoritative spec closure;
- metadata/interface fingerprint certification;
- cold/incremental parity;
- performance/resource evidence;
- runtime-negative audit;
- C3 checkpoint closure.

---

## 14. Verification ledger

No C3 implementation verification has run yet.

| Gate | Scope | Result | Meaning |
|---|---|---|---|
| predecessor C2.P3 G1–G5 | C2 substrate | PASS | C3 starts from verified packs/inference/metadata/runtime boundary |
| C3 gates | none | NOT RUN | checkpoint remains UNVERIFIED |

Do not copy predecessor PASS status into C3 verification.

---

## 15. Deferred / baseline issue ledger

### BASELINE-001

```text
Observed:
    semantic::capabilities::traits::
    trait_default_replays_external_nominal_callable_dependencies

Classification:
    inherited baseline / unrelated unless C3 proves causal interaction

Why nonblocking at C3 start:
    C2 explicitly deferred it; C3 architecture does not require trait-default repair.

When to revisit:
    only if a C3 focused trait-boundary test is blocked by the same cause.
```

### BASELINE-002

```text
Observed:
    semantic::capabilities::traits::
    trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities

Classification:
    inherited baseline / unrelated unless C3 proves causal interaction

Why nonblocking at C3 start:
    C2 explicitly deferred it.

When to revisit:
    only if a required C3 trait/conformance compatibility gate depends on it.
```

---

## 16. Known planning risks

| Risk | Severity | Owner plan |
|---|---|---|
| AST data/enum convergence loses GADT/case precision | critical | P1 |
| duplicate DataInfo/EnumInfo authority survives | high | P1 |
| type-data term accidentally considered ProperType | critical | P1/P2 |
| static-domain classifier becomes per-declaration compiler enum | high | P1 |
| heterogeneous `Applied` breaks row architecture | critical | P2 |
| trait/conformance assumes TypeId args are proper types | high | P2 |
| associated domain becomes second namespace | critical | P2 |
| associated specialization uses source strings | critical | P2 |
| metadata silently changes schema semantics | high | P1/P2 |
| TupleEntries duplicates CompleteTypePack | critical | P3 |
| intrinsic Tuple forces general data storage | high | P3 |
| enum keyword migration churns identity | high | P1/P3 |
| static syntax accidentally authorizes runtime expression execution | critical | P1/P2 |
| C3 grows into general compile-time evaluation | critical | all |

---

## 17. Consultation / amendment ledger

No C3 consultation incidents yet.

Use entries:

```text
INC-C3-<n>
Plan/task:
Trigger:
Observed:
Decision:
Architecture impact:
Plan amendment:
Verification required:
Status:
```

Material changes to:

- canonical term carrier;
- ProperTypeId boundary;
- row ownership;
- generic application representation;
- exact-case identity;
- associated namespace ownership;
- runtime/static execution boundary;

require a principal-architect amendment, not local implementer improvisation.

---

## 18. Mandatory checkpoint STOP conditions

Implementation must stop and consult if:

1. TypeId cannot represent new non-row static terms without weakening ProperTypeId.
2. RecordRow must be collapsed into TypeId to implement heterogeneous applications.
3. Data/Enum convergence breaks exact-case/GADT identity.
4. associated type-data requires trait evidence ownership.
5. one associated namespace cannot cover the required categories without duplicate lookup authority.
6. metadata requires source strings for identity.
7. CompleteTypePack cannot faithfully implement TupleEntries.
8. the VM must execute type-data constructors for semantic correctness.
9. enum/data compatibility produces different declaration or case identity.
10. C3 requires arbitrary static functions/evaluation to satisfy its accepted examples.
11. trait static-domain support becomes mandatory and requires coherence architecture changes.
12. any C2 pack/row invariant must be weakened.

---

## 19. Checkpoint verification strategy

### P1 evidence

Focus:

```text
parser/AST
algebraic semantic declaration
runtime ADT/GADT preservation
type-data canonical identity
metadata foundation
runtime-negative type-data boundary
```

### P2 evidence

Focus:

```text
generic/static application
inference
substitution
associated lookup
metadata/imports
incremental
source index
```

### P3 evidence

Focus:

```text
Universe/intrinsic structural domains
core algebraic data
TupleEntries/pack equivalence
LSP
corpus migration
spec consistency
cold/incremental certification
runtime-negative audit
```

Workspace/release-wide certification is not implied by `FOCUSED_TESTED`.

---

## 20. Active plan and next action

Current:

```text
active plan:
    null

next plan:
    LANG007.C3.P1

next action:
    create the patch-grade P1 implementation plan from:
      - LANG007.C3-requirements-analysis.md
      - LANG007.C3-CHECKPOINT.md
      - LANG007.C3-GUIDANCE.md
      - C2.P3 handoff/walkthrough
      - live repository
```

At P1 start:

1. verify local revision/worktree;
2. re-read live C3 target paths;
3. update `PROGRAM.md` / `STATUS.md` active lifecycle state;
4. mark checkpoint `IN_PROGRESS / PARTIAL / UNVERIFIED`;
5. mark P1 active;
6. do not begin P2 representation work opportunistically.

---

## 21. C3 completion truth table

C3 may be marked `COMPLETE` only when:

```text
[ ] P1 complete
[ ] P2 complete
[ ] P3 complete
[ ] unified algebraic source/semantic model implemented
[ ] type-data static domains implemented
[ ] heterogeneous static application implemented
[ ] associated type data implemented
[ ] intrinsic structural domains exposed
[ ] enum canonical source migrated/finalized
[ ] exact-case/GADT behavior preserved
[ ] metadata/import/incremental identity certified
[ ] LSP/source-index baseline certified
[ ] authoritative specifications coherent
[ ] runtime-negative type-data boundary certified
[ ] deferred failures classified
[ ] checkpoint state truthful
[ ] successor action explicit
```

Do not infer `RELEASE_COMPLETE` from this truth table.
