---
id: LANG007.C3
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: checkpoint-record
title: "Type Data and Unified Algebraic Static Domains"
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
requires:
  - LANG007.C1
  - LANG007.C2
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
active_plan: LANG007.C3.P2
latest_completed_plan: LANG007.C3.P1
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
verification: FOCUSED_TESTED
active_plan: LANG007.C3.P2
latest_completed_plan: LANG007.C3.P1
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
| `LANG007.C3.P1` | Unified algebraic `data`; enum semantic migration; top-level `type data` foundation | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | completed; walkthrough and handoff present |
| `LANG007.C3.P2` | General static domains; heterogeneous application/inference; associated `type data`; metadata/import/incremental | IN_PROGRESS | PARTIAL | UNVERIFIED | active; started from completed P1 working tree |
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

C3 remains IN_PROGRESS / PARTIAL / FOCUSED_TESTED because P2/P3 still own
generalized static application, associated type-data lookup, intrinsic surface
exposure, corpus/tooling migration, and checkpoint closure.

| Gate | Scope | Result | Meaning |
|---|---|---|---|
| predecessor C2.P3 G1–G5 | C2 substrate | PASS | C3 starts from verified packs/inference/metadata/runtime boundary |
| P1 AST syntax filters | phalcom-ast integration data_syntax, enum_syntax | PASS | 25 + 12 parser/compatibility tests passed |
| P1 semantic static-domain foundation | phalcom-semantic integration foundations::kinds static filters | PASS | canonical domains/ConstInt/StaticData, proper-type rejection, and source type-data kind publication passed |
| P1 semantic crate check | cargo check -p phalcom-semantic | PASS | all new TypeData consumers compile |
| P1 core compile boundary | cargo check -p phalcom-core | PASS | compiler/runtime consumers compile with static forms present |
| P1 runtime algebraic preservation | semantic `adts::declarations`, `constructors`, `variants`, `exact_cases`, `vertical_gadt` | PASS | 16 + 13 + 12 (9 active, 3 gated) + 5 active (1 gated) + 1 tests passed; legacy enum/data semantic identity remains equivalent |
| P1 metadata/static-term export | semantic `integration::metadata` and `foundations::kinds` | PASS | 21 metadata tests and 11 kind tests passed; schema-v5 static-domain, ConstInt, StaticData export/round-trip validation and runtime-negative coverage are covered |
| P1 metadata compatibility | phalcom-type-meta schema_compat | PASS | 9 tests passed; v1 fixture and v2–v4 compatibility remain valid, v5 feature/kind floors reject malformed static nodes |
| P1 ADT/static incremental parity | semantic `incremental::adts`, `incremental::fingerprints`, `incremental::type_store_revisions` | PASS | 13 active + 3 gated, 33, and 20 tests passed; runtime ADT and TypeOnly case-edit cold parity are green |
| P1 runtime representation | phalcom-core `core language::algebraic_data` | PASS | 44 passed, 19 explicitly ignored/gated; native_adt_runtime is not a live Cargo test target |
| P1 library unit-test binary | cargo test -p phalcom-semantic static_data | BASELINE_BLOCKED | pre-existing DeclarationTypeInfo test initializer omits generic_binder_header; outside the focused integration/G5 matrix |

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

### BASELINE-003

```text
Observed:
    phalcom-semantic library test compilation fails in
    checker/context.rs:3362 because a pre-existing DeclarationTypeInfo test
    initializer omits generic_binder_header.

Classification:
    unrelated baseline compile failure; the C3 integration test binary does
    not depend on this cfg(test) initializer and remains usable.

Why nonblocking at this stage:
    focused C3 integration tests and both affected crate checks pass.

When to revisit:
    before claiming a broader semantic crate unit-test gate; it does not block
    the focused P1 G5 matrix.
```

### P1-RESOLVED-001 — stable algebraic phase/shape declaration records

```text
Observed:
    schema v5 now persists stable static-domain kinds, ConstInt/StaticData
    nodes, and declaration-owned algebraic phase/shape records. Runtime
    Product versus Sum facts are projected from the canonical algebraic tables.

Classification:
    resolved P1 implementation and metadata gate.

Evidence:
    `metadata_publishes_stable_algebraic_phase_and_shape` and
    `metadata_keeps_legacy_enum_and_equivalent_data_identity_neutral` pass;
    the exporter records Runtime/TypeOnly and Product/Sum structurally and the
    interface fingerprint includes those records.
```

### P1-RESOLVED-002 — type-data runtime-negative and incremental parity evidence

```text
Observed:
    TypeStore proper-type rejection, TypeOnly compiler no-op behavior, and
    source-level runtime-use rejection are implemented. TypeOnly case edits are
    covered by a cold/incremental parity regression.

Classification:
    resolved P1 implementation and verification gate.

Evidence:
    `type_data_case_cannot_be_used_as_a_runtime_value` and
    `type_data_case_edit_matches_cold_static_domain_analysis` pass. The latter
    proves product-fingerprint invalidation, stable nominal domain kind, static
    constructor-registry convergence, and cold/incremental semantic equality.
```

### P1-RESOLVED-003 — final lifecycle/specification closure

```text
Observed:
    The effective data, enum, and type-level-domain specifications now describe
    the shipped unified/static foundation, and the P1 walkthrough/handoff are
    present.

Classification:
    resolved P1 lifecycle/specification closure.

Evidence:
    G5 focused suites are green; the plan is COMPLETE / IMPLEMENTED /
    FOCUSED_TESTED. The live P1→P2 handoff records the remaining generalized
    application, associated lookup, and P3 migration scope.
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

### P2 in-progress implementation evidence

The following bounded P2 slices are implemented in the live working tree:

```text
T1
    P1 prerequisite rechecked with `cargo check -p phalcom-semantic`.
    P2 activated from the P1-complete working tree at HEAD 60a27f79...;
    no commit or push was authorized.

T2
    named static-domain kind syntax (`S: Shape`) parses and resolves through
    declaration-backed `KindId::StaticDomain` classification.

T3
    `TypeData::Applied` uses heterogeneous `StaticArgument` entries;
    `TypeStore::apply_static_form` performs checked kinding, flat canonical
    application, and type-lambda compatibility reduction; type-only callers
    delegate through `apply_type_form`; substitution preserves the envelope
    and reports application failures instead of failing open.

T4 partial
    inference now lifts non-row `StaticData` constructor terms into a
    solver-local structural form, recursively tracks their variables/support,
    unifies only matching domain/constructor identities, and materializes
    through `TypeStore::static_data`. The `InferVarId(KindId)` identity and
    existing row solver remain unchanged. Library unit tests for the new
    solver cases remain behind BASELINE-003.

T5 partial
    type aliases accept an optional declaration-backed result kind such as
    `type DefaultShape: Shape = ...`; the alias lowering path validates the
    body kind and preserves the historical Type-valued default. Source static
    constructor paths still await T7 unified `::` lookup.

T8 partial
    durable applied metadata uses `StaticArgumentRef`, schema v6 is emitted,
    and v1-v5 compatibility validation remains covered.
```

Focused evidence acquired during this P2 continuation:

```text
AST parser named static-domain binder test                         PASS (1)
semantic foundations/kinds                                         PASS (15)
AST parser static-alias result-domain test                          PASS (1)
semantic foundations/generics_core                                 PASS (8)
semantic foundations/type_annotations                               PASS (45)
semantic adts                                                      PASS (281; 28 ignored/gated)
semantic metadata integration                                     PASS (21)
type-meta schema compatibility                                     PASS (9)
core language/algebraic_data                                      PASS (44; 19 ignored/gated)
core and semantic crate checks                                     PASS
semantic library static-data inference unit filter                 BASELINE_BLOCKED (BASELINE-003)
```

This is partial P2 evidence, not P2 acceptance. Remaining P2 implementation includes
full first-order inference coverage and row-aware solver terms, static type-member
domain convergence, associated `type data` declaration/specialization and unified `::`
lookup, lossless record-row application metadata/import handling, and
cold/incremental/source identity certification.

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
    LANG007.C3.P2

next plan:
    LANG007.C3.P2

next action:
    continue P2 after the generalized application slice:
      - finish first-order static-domain inference coverage and checked materialization;
      - converge static type members and then associated `type data` lookup;
      - complete metadata/import/incremental/source identity coverage;
      - leave P3 intrinsic-surface/corpus/tooling migration untouched until P2 is accepted.

P2 started from the live working tree at `60a27f79d69aa3e79c22e3d19df221466c7db920`.
The completed P1 implementation and its verification records are present as dirty
working-tree changes; no commit or push is authorized for this task. Unrelated
changes remain outside the P2 scope.
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
