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
active_plan: LANG007.C3.P3
latest_completed_plan: LANG007.C3.P2
next_plan: LANG007.C3.P3
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
active_plan: LANG007.C3.P3
latest_completed_plan: LANG007.C3.P2
next_plan: LANG007.C3.P3.T2 / G2
blockers: none; documented clean-predecessor baselines and graph-build performance follow-up remain non-blocking
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
| `LANG007.C3.P2` | General static domains; heterogeneous application/inference; associated `type data`; metadata/import/incremental | COMPLETE | IMPLEMENTED | FOCUSED_TESTED | T1–T9 accepted; G-FINAL completed with documented clean-predecessor and formatter baselines; see P2 walkthrough/handoff |
| `LANG007.C3.P3` | Intrinsic structural surface; Universe/corpus/tooling migration; certification and closure | IN_PROGRESS | PARTIAL | UNVERIFIED | T1/G1, T2/G2, and T3/G3 passed; T4/G4 awaits INC-C3-12 runtime-facade binding decision |

No additional P4 is planned.

If implementation reveals a bounded corrective closure needed for C3 acceptance, add a new plan in this checkpoint rather than falsely closing P3.

### 4.1 P2 architecture amendment — static cases and static-domain kinds

On 2026-09-23 the user ratified the architecture recorded in `INC-C3-10.md`.
P2 therefore adds per-case checked constructor signatures to the existing
Alias/Header/Member graph, while retaining early structural case catalogs as
the declaration-identity authority. Singleton getters require no signature;
constructor-shaped cases, including `Unit()`, require a checked signature.
Cycles containing Member or StaticCaseSignature are rejected as one
`TypeSignatureCycle`; existing Alias-only and Header/Alias-only behavior stays
unchanged. Static-domain kind syntax carries an ordinary `TypeAnnotation` and
resolves through the existing type-form path so applied associated owners retain
their heterogeneous arguments. Details and implementation order are in
`INC-C3-10.md` and the P2 plan amendment.

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

```text
INC-C3-1
Plan/task: LANG007.C3.P2 / T5.C — type-member user-domain parameters/results
Trigger: T5 lacked source-declaration validation/publication; selecting its
  owner/index affects T7 namespace lookup.
Decision: declaration semantic products own checked signatures keyed by stable
  owner-plus-canonical-selector identity; AssociatedSurface owns name admission,
  collisions, and lookup selection and refers to that identity. DeclarationTypeTable
  remains nominal type-form metadata. No separately queried type-member registry.
Architecture impact: authoritative split recorded in the amended T5 contract.
Plan amendment: Level 1 clarification recorded in P2 T5 on 2026-09-22; T5 publishes
  declaration products, T7 routes selection through AssociatedSurface.
Verification required: user-domain parameter/result formation, wrong-domain rejection,
  projection identity, same-owner collision through AssociatedSurface, and incremental
  invalidation of changed signatures without invalidating unrelated members.
Status: ADOPTED; implementation resumed. T5 publication is now implemented and focused
  tested; source-member application/projection selection still belongs to T7.

INC-C3-2
Plan/task: LANG007.C3.P2 / T6.B — runtime-data-owned associated type data
Trigger: T6 explicitly requires STOP AND CONSULT if the effective specification does
  not define unambiguous runtime-data owner syntax.
Observed: docs/specs/type-members.md defines Inherent members on nominal declarations,
  trait-owned Associated members, and DomainNative members; docs/specs/objects/data-objects.md
  does not define nested associated type-data declarations or their source placement.
  No accepted runtime-data-owned `type data` syntax/ownership rule was found.
Accepted decision (user ratified 2026-09-23): classes may declare associated `type data` in the
  class body; a runtime `data` owner's representation remains closed, and its associated
  `type data` is declared in an unguarded inherent `impl` in the owner's module, not in
  a conformance impl. Both forms have the nominal owner as semantic owner; impl location
  is provenance only. Use stable owner-plus-name declaration identity, specialized by
  the canonical owner generic environment. Reject duplicate associated domain names
  across inherent impl blocks and let AssociatedSurface decide collisions with other
  associated categories. Guarded and partial-target impl declarations are excluded.
  Associated type data has no runtime inhabitants or invocation target; its static cases
  belong to the associated domain.
Architecture impact: accepted rule preserves the closed runtime-data representation,
  stable owner-plus-name declaration identity, and AssociatedSurface name authority.
  The impl fragment supplies provenance only; the owner generic environment supplies
  canonical specialization.
Plan amendment: applied to T6.B. The owning normative specifications define the domain,
  ownership/identity, representation boundary, and inherent-impl placement with
  cross-references rather than competing definitions.
Verification required: parse and publish class-body and inherent-impl
  forms; reject declarations in runtime-data representation bodies and conformance impls;
  reject duplicate names across inherent impls; verify stable declaration identity over
  two generic owner specializations, distinct canonical concrete domains, and no runtime
  value production.
Normative amendments: docs/specs/type-level-domains.md, docs/specs/type-members.md,
  docs/specs/objects/data-objects.md, and docs/specs/objects/impl.md.
Status: ADOPTED; T6 resumed and now has focused parser, declaration-publication,
  collision, generic-specialization, and no-runtime-invocation evidence recorded
  below.

INC-C3-3
Plan/task: LANG007.C3.P2 / T7 — source type-member projection selection
Trigger: mandatory trigger 14 (a second source of canonical resolution would be
  introduced by a local workaround); trigger 7 if the type-formation and surface
  query boundaries imply materially different integration designs.
Observed: source `TypeMemberDef` signatures are published as identity-keyed
  declaration products and `AssociatedSurface` stores their admitted identities.
  `resolve_type_member_projection` in `types/annotation.rs` receives only
  `&dyn TypeResolver`, resolves intrinsic operations locally, and otherwise builds
  a symbolic projection; it receives neither the selected surface entry nor a
  checked source declaration product. `CheckingContext` tracks associated-surface
  dependencies, but the type-formation resolver has no corresponding selected-member
  product/dependency seam.
Expected architecture: T7 must select by owner/base through AssociatedSurface,
  retrieve the checked signature by stable `TypeMemberDeclarationId`, and preserve
  that identity in the canonical projection. No parallel name lookup is allowed.
Scoped work already present: T6 publishes associated static-domain products and
  T7 resolves associated static domains and singleton cases as static semantic
  results; heterogeneous owner applications preserve `RecordRow` in the canonical
  `StaticArgument` envelope. Focused tests pass for those slices.
Implementer hypothesis: type formation needs an explicit session/query-provided
  associated-member resolution context that records dependencies and returns the
  selected declaration identity/product, while leaving AssociatedSurface as the
  sole selection authority.
Alternative: route type-member projection formation through a new canonical
  semantic query at the declaration/snapshot boundary rather than extending the
  resolver context. These differ in query ownership and invalidation wiring.
Decision: user confirmed the session/query-provided seam: AssociatedSurface
  selects the member; checked declaration products are retrieved by stable
  TypeMemberDeclarationId; consumed surface and signature products participate
  in invalidation dependencies. No independent name registry or precedence path.
Plan amendment: required and applied in T7 before implementation resumes.
Verification required: source type-member selection through surface, wrong-domain
  rejection, projection identity across equivalent spellings, collision through the
  surface, and changed-signature invalidation with an unrelated sibling reusable.
Status: RESOLVED; implementation now selects source members only through the
confirmed surface/query seam. Focused wrong-domain and selected-identity tests pass.
Source projection identity is now focused-tested in field, alias, and generic-header
annotations. The user-directed phase reorganization remains incomplete: even a
same-module alias application of a source generic data declaration fails because
nominal generic signatures are published later. INC-C3-6 records the ratified
alias/member cycle rule and the new decision needed before extending the dependency
graph to generic headers. Preserve AssociatedSurface as the only selector and do not
publish provisional or parallel lookup surfaces. Independent T4/T8 work may continue.

```

### INC-C3-4

Incident report: [INC-C3-4.md](INC-C3-4.md)

Plan/task: LANG007.C3.P2 / T8 — nominal static-case source-index identity
Trigger: mandatory trigger 11 (two plausible semantic authorities have different
  invalidation consequences) and trigger 14 (do not reconstruct canonical targets
  from source spelling).
Observed: the original defect was that nominal `Shape::Scalar` received
  `TypeOnlyRuntimeValue` before case selection, leaving no source target. The accepted
  fix now converges nominal and associated domains at canonical static-case selection.
Expected architecture: T8 targets derive from canonical semantic selection and stable
  domain/case identity, not token spelling or a second case-name registry.
Decision (advisor response adopted): resolve the nominal owner to
  `StaticDomainDeclarationId`, select its case through canonical static-domain
  semantics, publish the existing `StaticTerm` resolution with stable `StaticCaseId`,
  then retain `TypeOnlyRuntimeValue` for illegal runtime use. `CallableSourceAttachment`
  consumes this product and must not search declarations or source spelling. The
  semantic static-domain/member-selection dependency owns case-edit invalidation.
Plan amendment: T8's existing target and invalidation requirements are clarified by
  INC-C3-4; no second lookup authority or new resolution variant is introduced.
Verification: nominal target/runtime-diagnostic, missing-case, and rename/cold-parity
  tests plus semantic crate check passed as recorded in INC-C3-4. On the current HEAD,
  the same focused nominal-case set selected 3 and passed in a clean worktree; the
  shared dirty checkout currently fails earlier while parsing the unrelated local
  `fiber.ph` edit. T8 remains open for cross-module metadata/catalog validation and
  broader persistence/import parity.
Status: RESOLVED — nominal static-case selection is implemented and focused-tested.

Focused evidence: semantic crate check; `adts::declarations` (26 passed);
`adts::associated` (22 passed, 5 gated); `associated::invocation` (1 passed);
`incremental::type_members` (8 passed); `nominal_static_case_rename` (1 passed);
`integration::metadata` (23 passed); `incremental::adts` (13 passed, 3 gated);
core crate check; and `phalcom-type-meta` (9 passed). The semantic library-unit
filter is baseline-blocked by the previously recorded missing `generic_binder_header`
test initializer. Workspace formatter check remains baseline-failing on untouched
files; no broad formatting was applied.

### INC-C3-5 — T7 all-annotation phase ordering

Incident report: [INC-C3-5.md](INC-C3-5.md)

Plan/task: LANG007.C3.P2 / T7 — all declaration annotations
Observed: aliases lower before generic headers; callable/field signatures are
  materialized before data/enum and `AssociatedSurface` publication; the resolver
  receives the associated type-member context only after that publication.
Classification: known implementation gap from the accepted T7 amendment, not a
  newly reproduced semantic failure or a new architecture decision.
Decision: continue with the user-directed phase reorganization using one canonical
  surface and the INC-C3-3 identity/dependency seam. Preserve alias/generic-header
  dependency contracts. Consult only if live dependencies require changing those
  contracts or introducing another authority.
Verification: not run for the incident report; add annotation-phase, wrong-domain,
  canonical identity, and incremental-dependency tests with the implementation.
Status: OPEN — implementation and focused verification remain required.

### INC-C3-6 — T7 alias/member signature dependency ordering

Incident report: [INC-C3-6.md](INC-C3-6.md)

Plan/task: LANG007.C3.P2 / T7 — all declaration annotations
Trigger: mandatory trigger 7 (phase ordering may change the declaration-signature
  dependency contract); trigger 16 for cycle-policy changes.
Observed: the field-annotation regression initially selected a symbolic projection
  without its source declaration identity; the implementation now fixes this for
  field, alias, and generic-header cases. Alias lowering precedes members, while checked
  member signatures resolve domains through the alias-populated resolver. A same-module
  generic `Tensor` application in an alias still fails as unresolved because nominal
  generic signatures are built after alias lowering. The current graph models aliases
  and source members, not generic headers.
Decision: user accepted on 2026-09-23 that cross-kind declaration-signature cycles
  are rejected with a deterministic diagnostic; ordinary symbolic projection
  normalization remains valid. `docs/specs/type-members.md` and the T7 plan are
  amended; `TypeSignatureCycle` is implemented for alias/member SCCs. User has now
  accepted including nominal/trait generic headers in the unified graph: reject an
  SCC containing a source type-member signature under the same deterministic rule,
  while preserving alias-only and header-only cycle behavior.
Verification: `incremental::type_members` selects 12 and passes, including field,
  alias, generic-header identity, collision, wrong-domain, incremental-surface, and
  mixed-cycle tests. The same-module `Tensor` alias probe fails and is recorded in
  INC-C3-6; it is not a passing gate.
Status: PARTIALLY RESOLVED — cycle policy and focused implementation are in place;
  all-annotation scheduling, complete AssociatedSurface publication, and exact
  invalidation remain open. Generic-header participation and mixed-SCC policy are
  decided; implementation and focused cycle/application tests remain. See
  [advanced-agent handoff](LANG007.C3.P2-advanced-agent-handoff.md).

### INC-C3-7 — T7 graph invalidation and shell edit triggers

Incident report: [INC-C3-7.md](INC-C3-7.md)

Plan/task: LANG007.C3.P2 / T7 — incremental signature dependencies and shell
publication.
Trigger: mandatory trigger 15 (nonlocal incremental invalidation is uncertain) and
trigger 16 (the live retained-product lifecycle does not satisfy the accepted plan).
Finding: the Alias/Header/Member graph originally scheduled and classified SCCs but
was not retained or traversed for reverse invalidation. Existing alias/header maps did
not cover cross-kind member edges, and retained member products could bypass the
scheduler. The shell builder also failed to seed affected owners for behavior-only
inherent impl edits, leaving collision facts stale.
Decision: use the same stable-identity graph for scheduling, SCC handling, and reverse
invalidation. Traverse old reverse edges from changed declaration nodes before dropping
retained products; rebuild the exact consumer closure, refresh edges for recomputed
products, and apply SCC withholding before publication. Include old and new resolved
owners for every changed namespace-affecting inherent impl. Keep `AssociatedSurface`
as the sole selection/collision authority. No plan amendment is required; these
requirements are already in T7's exact invalidation contract. The current working diff
now persists the graph/reverse edges, seeds old-graph closure from source-level
declaration fingerprints before dropping products, and retains old/current inherent
impl targets for surface rebuilding.
Verification: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib`
and `git diff --check` pass. Focused semantic tests were not run against this diff; a
prior test attempt was blocked before the selected case by the canonical Universe
parser failure documented in the incident report.
Status: IMPLEMENTED / UNVERIFIED — compile checked; exact dependent reuse,
behavior-only collision refresh, and cold/incremental parity remain unverified.

Current focused update (2026-09-23): the semantic test target compiles after
updating T4/T7 test fixtures, and `incremental::type_members` selects and passes
12 existing tests. A new `Tensor<Int, Shape::Scalar>` alias/header regression
fails on its initial cold analysis: zero-payload `ExactEnumCase` formation
forwards `Shape::Scalar` to contextual trait-associated projection resolution
and emits `AnnotationUnresolved` for missing conformance evidence. The accepted
static term must instead be selected through canonical static-domain/case
authority, with exact dependency recording. The retained-graph edit and cold
parity portions of this regression remain unverified. The canonical Universe
`fiber.ph` parser defect was repaired separately; it had masked these tests.

### INC-C3-8 — T4 mixed inference solver ownership and frame scope

Incident report: [INC-C3-8.md](INC-C3-8.md)

Plan/task: LANG007.C3.P2 / T4 — heterogeneous applied inference with row arguments.
Trigger: mandatory triggers 3 and 5 (solver ownership and lifetime); T4 STOP/CONSULT.
Finding: the original `type_id_to_inference_with_rows` path made an entire `Applied` or
`StaticData` term opaque when one child was a `RecordRow`. Ordinary constraints lived in
a retained query-context `InferenceSession`, but row variables lived in a call-local
`GenericApplicationSession`.
Decision: preserve separate `InferVarId` and `RecordRowVarId` identities. Amend T4 so a
query-context-owned coordinator owns both existing solvers, routes mixed equations, and
uses explicit frame-scoped row roots. Row variables persist to query-context teardown;
unresolved cross-frame equations remain for the parent, and child results cannot
materialize unresolved outer rows. No row identity enters canonical products. The T4
plan is amended with this contract and required acceptance cases before implementation.
The row solver is generalized over field payloads: it solves labels/tails and emits
same-label payload pairs to ordinary inference using `InferenceTerm` payloads without
fake `TypeId`s. `InferenceCoordinator` owns the ordinary session and a separate row
solver, alternates both to a fixed point, materializes resolved field terms, and checks
cross-domain occurs through row fields and substitutions. Nested views inherit active row
parameter mappings; a child can retain a same-context symbolic result dependent on an
unresolved ancestor row instead of materializing a canonical generic instantiation.
Verification: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib` passed.
No T4 tests were added or run; mixed-argument behavior and nested resolution remain
unverified. Existing T7 working diff remains `IMPLEMENTED / UNVERIFIED` as recorded under
INC-C3-7.
Status: IMPLEMENTED / UNVERIFIED — implementation present; focused acceptance remains open.

### INC-C3-9 — T8 typed catalog and metadata validation boundary

Incident report: [INC-C3-9.md](INC-C3-9.md)

Plan/task: LANG007.C3.P2 / T8 — stable metadata catalog and validation.
Trigger: metadata could validate that a static constructor's owner matched its domain,
but the serialized artifact had no declaration/case catalog for membership, payload
kind, or associated owner-environment checks.
Decision: schema v8 adds a typed catalog keyed by stable nominal/associated declaration
identity. Each entry records ordered owner parameter kind/mode pairs and sorted cases
with ordered payload kinds. Associated owner specialization remains in
`StaticDomainRef::Associated`. Schemas v1-v7 keep their prior validation behavior; the
new catalog defaults empty when decoding older bundles, and older readers reject v8.
Snapshot-backed export consumes canonical current data/enum and associated declaration
products. It must not infer current declaration membership by enumerating TypeStore
interning history. v8 validation checks catalog membership, owner arity/kinds, case
membership, payload arity/kinds, and global/scoped Applied argument and result kinds.
The bounded implementation is present and `cargo check` passes for type-meta,
semantic, and core libraries. No tests were run. Cross-module catalog composition,
import reconstruction, provider invalidation, and cold/incremental parity remain open.
Status: IMPLEMENTED / UNVERIFIED — schema and exporter slice compiles; T8 remains partial.

Resolution update (2026-09-24): schema-v8 acceptance is now focused-tested. The
compatibility suite passes 14/14. The metadata integration module passes its new
round-trip/scoped cases; its one unrelated alpha-equivalence failure reproduces at the
clean `9c150747` predecessor. Cross-module provider/cold parity, source-index identity,
incremental ADT refresh, and TypeStore-history exclusion gates also pass. `INC-C3-9`
is RESOLVED / ACCEPTED with the baseline failure retained below as nonblocking evidence.

### INC-C3-10 — static cases in declaration annotations

Incident report: [INC-C3-10.md](INC-C3-10.md)

Plan/task: LANG007.C3.P2 / T5/T7 — static aliases and all-annotation availability.
Observed: the new `Tensor<Int, Shape::Scalar>` alias regression fails in cold
analysis before its incremental assertions. Zero-payload `ExactEnumCase`
formation routes to contextual trait-associated projection, while checked
nominal static-case products are built after aliases. The T5 static-alias
example and T7 all-annotation contract therefore remain unimplemented.
Decision: accepted architectural verdict in the attached INC-C3-10 consultation.
Publish a declaration-owned structural case catalog before signature checking;
its stable IDs use the existing structural `Selector`, which distinguishes a
singleton getter from an explicit-empty constructor. Checked payload signatures
are separate products in the Alias/Header/Member/StaticCaseSignature graph.
Only checked-signature SCCs containing a Member or StaticCaseSignature are
rejected with one deterministic `TypeSignatureCycle`; catalog/header recursion
is legal. Annotation and expression selection consume the same catalog; the
store only materializes selected terms. Catalog and selected signature reads
are exact dependencies, and associated namespace publication depends on the
domain header rather than payload signatures.
Plan amendment: apply this split in T7 before treating static alias, case
selection, or incremental gates as accepted. Preserve catalog identity when a
payload signature is invalid, and preserve cold/incremental parity.
Status: ARCHITECTURE RESOLVED / IMPLEMENTATION IN PROGRESS / UNVERIFIED.

### INC-C3-12 — runtime intrinsic facade binding prerequisite

Incident report: [INC-C3-12.md](INC-C3-12.md)

Plan/task: LANG007.C3.P3 / T4/G4 — Unit/Tuple/Record intrinsic runtime facade migration.
Trigger: mandatory triggers 1, 2, 7, and 12; T4's prerequisite and P3's ordered
  migration rule require a runtime facade binding that is not established by T2/G2.
Observed: the declaration table registers TypeOnly intrinsic kinds and existing
  Tuple/Record class-owner descriptors, but no runtime intrinsic Data declaration
  for Unit/Tuple/Record. Runtime product `Statement::Data` goes through
  `compile_data`, which emits `Bytecode::Data` / `FinalizeData`; native source
  presentations are indexed from Class and sum declarations. Existing structural
  TypeStore and runtime class authorities are present.
Decision: user ratified on 2026-09-24 that T4 first establishes and tests the
  declaration-identity binding and existing-owner compiler/runtime reuse path,
  then migrates source presentation. Keep within T4/G4 as a Level 1 sequencing
  clarification; no corrective plan.
Architecture impact: do not migrate Unit/Tuple/Record source or add speculative
  compiler/runtime handling until the binding ownership and reuse path are ratified.
Plan amendment: Level 1 clarification recorded in INC-C3-12.md and the active
  P3 plan. T4/G4 may resume.
Verification required: prove facade declarations resolve to the existing canonical
  Unit/Tuple/Record semantic/runtime identities, do not emit generic data lowering,
  retain native descriptor/source targets, and preserve exact runtime census.
Status: ADOPTED — T4/G4 implementation resumed.

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
    unifies only matching domain/constructor identities, infers a non-row
    static-domain generic application argument, and materializes
    through `TypeStore::static_data`. The `InferVarId(KindId)` identity and
    existing row solver remain unchanged. Six focused integration tests cover
    ConstInt payload solving, domain/case mismatch refutation, rejection of a
    proper-Type candidate, nested structural solving, preservation of an
    underconstrained static-domain variable, and inference through a static-domain
    generic application. `foundations::static_domain_inference`
    (5), `foundations::generic_application` (11), and `foundations::generics_core`
    (8) pass. The original library unit-test filter remains behind BASELINE-003.
    Heterogeneous row arguments in the general inference-term application
    representation remain unimplemented; alias/type-member convergence remains part
    of G3.

T5 partial
    type aliases accept an optional declaration-backed result kind such as
    `type DefaultShape: Shape = ...`; the alias lowering path validates the
    body kind and preserves the historical Type-valued default. Source static
    constructor paths still await T7 unified `::` lookup. Integer type-member
    arguments now canonicalize to `TypeData::ConstInt` at projection formation;
    normalization and intrinsic tuple indexing consume that canonical term.
    Focused type-member tests pass. Source declarations now publish checked
    signatures under stable owner-plus-canonical-selector identities. Snapshot
    products own signatures; AssociatedSurface entries reference identities, own
    collision/admission decisions, and depend on signature fingerprints. A source
    signature edit recomputes the entry while an unchanged sibling product is reused.
    A signature-only argument validator is available for a previously selected
    declaration product and rejects wrong-domain arguments; T7 owns routing source
    applications through AssociatedSurface and preserving selected-declaration
    projection identity across equivalent lookup spellings.

T6 focused-tested
    associated `type data` parses in class bodies and unguarded inherent
    implementations of runtime data owners. Stable declaration identity is
    owner-plus-name; owner generic arguments form canonical specialized domains.
    Checked declaration products are retained separately from AssociatedSurface,
    which admits the category and handles collisions. Constructor payload kinds
    are checked against the owner generic environment. Conformance and guarded/
    partial-target impl declarations are rejected; duplicate names across impl
    fragments are rejected. No runtime invocation target is created. Structural
    updates always republish a product-data owner's base constructor surface even
    when no associated static domain is contributed.

T7 partial
    Nested associated `::` paths parse, and kind atoms now wrap ordinary
    `TypeAnnotation` forms so `Container<Int>::State` resolves through the existing
    type-form path and `AssociatedSurface`. Static case catalogs publish stable
    identities for every case before payload checking; per-case constructor
    signatures (including zero-arity constructors) are query products in the
    Alias/Header/Member/StaticCaseSignature graph. Builders consume catalog and
    checked-signature products, and one shared intern/materialization path enforces
    singleton versus constructor requirements. The graph classifies any SCC with a
    Member or StaticCaseSignature as one `TypeSignatureCycle`, while recursive
    domain payloads remain legal. TypeOnly aggregate metadata now accepts static
    domain forms without running them through the proper-runtime-type checker.
    Class-body and inherent-impl associated signatures have per-case source
    fingerprints; an old duplicate impl builder path was redirected to the same
    catalog/signature products.

    Focused evidence on 2026-09-23: parser associated-domain kind-form test (1),
    associated generic-owner kind resolution (1), singleton versus `Empty()`
    signature/alias (1), recursive static-domain payload (1), alias/case-signature
    SCC (1), associated generic-owner SCC (1), nominal catalog rename and cold parity
    (1), class-owned signature edit with sibling reuse and cold parity (1),
    inherent-impl signature edit and cold parity (1), and runtime-negative TypeOnly
    value use (1) all pass; `cargo check -p phalcom-semantic` passes. These results
    close the ratified static-case/kind-form implementation slice, not all P2 gates.
    Heterogeneous row inference (T4), metadata/import reconstruction and provider
    parity (T8), remaining static type-member convergence, and final T9 stabilization
    remain open. Declaration fingerprint discovery is module-scoped, but dependency
    graph reconstruction and SCC traversal still scan the full graph; retain this as
    a performance follow-up rather than claiming exact-worklist completion.

T8 partial
    schema v8 adds a stable declaration/case catalog, validates associated owner
    argument kinds and static constructor membership/payload kinds, validates
    heterogeneous row argument fields/tails, and checks kinded arguments and result
    kinds for global/scoped Applied nodes. Snapshot-backed export
    consumes current nominal data/enum products and associated declaration products;
    the metadata contract does not enumerate retained TypeStore history. Schemas v1-v7
    retain their existing validation floors. The affected libraries compile, but no
    v8 behavior tests were run against this diff. Cross-module catalog composition and
    import reconstruction, provider edit invalidation, and cold/incremental parity
    remain open. The source index retains the prior focused nominal-case identity
    evidence; it does not close the broader T8 contract.

### Corrective closeout activation

`LANG007.C3.P2.R1-closeout-corrective-patch-grade-implementation-plan.md` is the
active execution companion for closing the remaining P2 acceptance gaps. Its
preflight started at `9c150747262275b34e52cb7b6642d7d3ceb4ad4d` with a clean
worktree. The two commits since the plan's `2ea4f3a1` repository anchor contain
the unrelated LANG004 setter-lane delivery and this corrective plan; neither
changes P2 semantic implementation. Live inspection confirms the three T4 gaps
remain: heterogeneous generic-parameter reachability, stable row-lacks traversal,
and transitive/frame-aware row-root completion.

### Corrective closeout execution progress

T2 implementation is applied. `type_contains_any_parameter` now traverses
heterogeneous `StaticArgument`s, record-row tails, associated static-domain owner
arguments, and scoped type-lambda bodies. Stable row-lacks collection follows the
same heterogeneous argument and owner paths. No `RecordRowVarId` is emitted into
stable facts. The direct RecordRow generic syntax attempted for a non-callable
declaration is rejected by the live checker (`RecordRow generic binders are
supported only on callable declarations`); no grammar was inferred or added. Tail
reachability is therefore covered by semantic construction beside the private
helper, rather than a fabricated source form.

T3 implementation is applied at the coordinator boundary. Requested roots are
normalized transitively before completion classification: local and solver-internal
residual tails remain underconstrained, while an unresolved ancestor-owned tail is
retained as a symbolic solution. The row solver remains frame-agnostic, roots remain
call-scoped, and ordinary/row variable identities remain separate. The focused
coordinator matrix passes in a disposable validation worktree: local and transitive
underconstraint, child-to-parent symbolic retention and later resolution, both
cross-domain occurs insertion orders, heterogeneous `Applied` and `StaticData`
children, inferred row-extension payload materialization, and semantic same-label
conflict reporting. The existing nested generic-closure row integration test also
passes.

The library unit-test target in the primary checkout remains blocked by stale,
unrelated test-only initializers (`DeclarationTypeInfo.generic_binder_header` and
legacy `InferenceStaticArgument` construction). For the isolated coordinator/helper
test run, only those test-harness compatibility sites were adjusted in a disposable
worktree; active implementation sources and new tests were copied unchanged. The
temporary worktrees were removed afterward.

The call-parameter helper matrix passes (4 tests), the stable row-lacks helper matrix
passes (3 tests), and the frame/coordinator matrix passes (9 tests). The focused
foundation gates pass: static-domain inference (6), row inference (6), generic
application (11), and generic core controls (8). `cargo check -p phalcom-semantic
--lib` passes. The focused record-row integration group selects 11 tests: 8 pass,
and these three
failures reproduce unchanged at the clean `9c150747` predecessor, so they are
classified as baseline: `repeated_row_parameter_conflict_is_rejected`,
`row_conflict_message_does_not_expose_solver_variables`, and
`return_record_prefix_contributes_lacks_constraint_to_row_inference`. T2/T3 are
implemented and focused-tested with the stated unit-harness limitation. CL-01's
row-tail-only case exercises declaration-generic selection with a constructed
canonical callable signature because the corresponding non-callable source binder
is rejected by the live language. T4's corrective matrix is accepted with those
three baseline failures explicitly classified. T5/T7 acceptance is now focused
tested: `incremental::type_members` selects 23 and passes, `adts::associated`
selects 27 (22 pass, 5 fixture-gated ignored), `associated::invocation` selects
1 and passes, and `foundations::kinds` selects 18 and passes. Evidence includes
callable parameter/return type-member projection, A/B/C associated-surface
collision refresh with cold parity, and a canonical mixed Term/RecordRow/constant
associated-owner specialization. T7 is accepted.
```

### Corrective closeout continuation — T8 accepted; T9 consultation open (2026-09-24)

T1–T8 are accepted for the P2 pre-T9 gate. This is implementation acceptance with
focused evidence, not release certification. T4's three record-row integration
failures remain classified as clean-predecessor baseline failures. T8 schema/import
coverage is complete: `phalcom-type-meta` passes 14/14; schema-v8 static alias,
associated-term, heterogeneous row application, scoped-bound, and stale-TypeStore
history regressions pass; provider payload edit/rename and associated owner
specialization tests pass; `integration::imported_resolution` passes 14/14,
`integration::source_index` 22/22, `incremental::adts` 13/13 selected non-gated tests,
and `incremental::type_store_revisions` 22/22.

The trait boundary rejects user-defined static-domain generic binders with a targeted
diagnostic, withholds the invalid header and conformance contribution, and retires a
previously published valid header after an incremental invalid edit. Both new focused
tests pass. `capabilities::traits` selects 39 tests: 37 pass and two dependency
assertions fail identically at clean predecessor `9c150747` because non-generic trait
defaults acquire a Universe `Bool` declaration-shell dependency. These are baseline
failures, not trait-boundary regressions.

T9 has started its audits. Runtime isolation found no P2 runtime static evaluator or
new VM static-domain execution contract. The P2 code does not weaken ProperType
admission; general heterogeneous consumers preserve `Term`/`RecordRow`, while
projection normalization and rigid/runtime-only consumers explicitly require
Type-valued arguments or return symbolic/reject. The different-origin supertype
projection path remains type-only and conservatively does not project heterogeneous
owner arguments; no required P2 acceptance case needs row inheritance through that
path. The resource audit found `self.lambda_arena.clone()` in
`TypeStore::apply_static_form`, which conflicts with canonical P2 `INV-P2-22` and
corrective `INV-24` / `CL-35`.

`INC-C3-11` is IMPLEMENTED / FOCUSED_TESTED. The architect's ruling is implemented:
one active `TypeLambdaArena` is threaded through recursive reduction and free-capture
specialization, and public TypeStore boundaries restore it on ordinary returns. Both
whole-arena clone sites are removed. No transaction, journal, or rollback behavior was
added.

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
semantic source type-member surface selection/domain validation    PASS (1)
semantic associated static-domain/case source-index targets         PASS (1)
semantic imported nominal static-case provider identity             PASS (1)
semantic integration/metadata                                      PASS (23)
semantic incremental/type_members                                  PASS (7)
semantic foundations/static_domain_inference                       PASS (6)
semantic incremental/adts                                          PASS (13; 3 ignored/gated)
semantic incremental type-data case edit cold parity               PASS (1)
semantic incremental associated static-domain edit/cold parity     PASS (1)
core crate check                                                   PASS
core language/algebraic_data                                      PASS (44; 19 ignored/gated)
core and semantic crate checks                                     PASS
semantic library static-data inference unit filter                 BASELINE_BLOCKED (BASELINE-003)
semantic foundations/type_members                                 PASS (7)
semantic integration/type_member_closure                           PASS (9)
semantic incremental/type_members                                  PASS (7)
AST parser associated type-data sites and nested path                PASS (2)
semantic associated static-domain publication/duplicates            PASS (focused filters)
semantic nested associated static domain/term lookup                 PASS (focused)
semantic generic-owner associated-domain specialization              PASS (focused)
core associated lowering boundary                                    PASS (cargo check only)
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic      PASS (post-amendment)
repo-wide cargo fmt --all -- --check                               FAIL (D; pre-existing formatting drift outside this slice)
semantic incremental/type_members                                  PASS (12; continuation)
semantic associated lookup                                         PASS (1; continuation)
semantic imported static-case provider identity                    PASS (1; continuation)
semantic type-data case cold parity                                PASS (1; continuation)
semantic mixed alias/member cycle                                  PASS (1; continuation)
semantic source-member alias projection                            PASS (1; continuation)
semantic source-member generic-header projection                   PASS (1; continuation)
semantic nominal_static_case at HEAD                               PASS (3; clean worktree)
type-meta schema_compat                                             PASS (9; clean worktree)
semantic integration/metadata                                       PASS (23; clean worktree)
semantic incremental/adts                                          PASS (13; 3 ignored; clean worktree)
semantic imported nominal static-case resolution                  PASS (1; clean worktree)
```

Final T9 / G-FINAL evidence acquired 2026-09-24:

```text
semantic foundations/type_annotations                               PASS (49; includes four INC-C3-11 regressions)
semantic foundations/record_row_inference                           PASS (6)
semantic foundations/static_domain_inference                        PASS (6)
semantic foundations/generic_application                            PASS (11)
semantic foundations/generics_core                                  PASS (8)
semantic foundations/substitution                                   PASS (7)
semantic foundations/kinds                                          PASS (18)
semantic adts/associated                                            PASS (22; 5 ignored/gated)
semantic incremental/type_members                                   PASS (23)
semantic integration/imported_resolution                           PASS (14)
semantic integration/source_index                                   PASS (22)
semantic incremental/adts                                           PASS (13; 3 ignored/gated)
type-meta schema_compat                                              PASS (14)
core crate check                                                    PASS
semantic integration/metadata                                       BASELINE_BLOCKED (24 pass, 1 clean-predecessor failure)
semantic integration/record_row_polymorphism                        BASELINE_BLOCKED (8 pass, 3 clean-predecessor failures)
workspace formatter                                                 BASELINE_BLOCKED (formatter drift already present in the four touched Rust files at HEAD; additional workspace drift)
```

The focused failures and formatter differences are clean-predecessor baselines and are
not attributed to the arena correction. The source audit finds no whole `TypeLambdaArena`
clone in TypeStore application, substitution, or beta-reduction paths. Runtime isolation,
ProperType, and Applied-consumer audits remain as previously recorded. P2 is
`COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; C3 remains partial because P3 work is active.

The type-member publication slice is committed and pushed as
`f2a6e3d8 lang007: publish checked type-member declarations`. The repository-wide
formatter check reports differences in untouched AST/core/checker files; no formatting
changes from that check were retained.

The paragraph above is a historical continuation snapshot and is superseded by the final
T9/G-FINAL state recorded here. The library unit-test filter remains blocked by
BASELINE-003. Full-graph edge reconstruction and SCC traversal remain a performance
follow-up, not a P2 correctness blocker.

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

## P3 takeover and migration manifest

T1/G1 completed at starting revision:

```text
repository: aureat/phalcom-lang
branch: main
revision: 518f66d19941eb24e379e60f64aa89df66643a8a
subject: fix(semantic): thread active type lambda arena
worktree: one unrelated untracked directory, phalcom-core/tests/integration/; preserved
metadata schema: 8
P1: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P2: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

The P2 hard predecessor contract is verified against the completed-P2 handoff,
checkpoint evidence, and live source symbols. Stable owners include `TypeStore`
and `TypeId`; `ProperTypeId`; `KindId` / `KindData`; `StaticDomainId`;
`StaticArgument::{Term, RecordRow}`; `RecordRowId` and the separate row solver;
`TypeStore::complete_type_pack` / `project_complete`;
`TypeData::CompleteTypePack`; `AssociatedSurface`; semantic editor
`target_at`/definition/reference products; metadata schema v8; the active
`TypeLambdaArena` move-out/restore boundary; and
`UniverseBootstrapMeasurement::{discovered_units, root_reachable_units,
executed_units}`. No missing predecessor authority was found.

Exact source/tooling anchors for follow-on tasks:

| Surface | Live anchor | T1 finding |
|---|---|---|
| Tuple and pack presentation | `phalcom-core/core/universe/src/collections/tuple.ph` | `native class Tuple`; exposes Positionals/Labeled/At/Map; no Entries |
| Record presentation | `phalcom-core/core/universe/src/collections/record.ph` | `native class Record` |
| Unit presentation | `phalcom-core/core/universe/src/option/unit.ph` | `native class Unit` |
| closed sums | `phalcom-core/core/universe/src/option/option.ph`, `errors/result.ph`, `object/ordering.ph` | `Option`, `Result`, `Ordering` remain `native enum` |
| other Universe sums | `time/calendar/date.ph`, `month.ph`, `weekday.ph`, `datetime.ph` | DateOverflow, Month, Weekday, LocalTimeDisambiguation, LocalTimeResolution |
| source index | `phalcom-semantic/src/source_index/scope.rs`, `symbol.rs`, `builder.rs`, `mod.rs` | SourceDeclarationKind and EditorSymbolKind still include Enum; builder has `visit_enum` |
| intrinsic operation dispatch | `phalcom-semantic/src/types/type_member.rs` | `lookup_intrinsic_type_operation` still dispatches by owner string |
| LSP navigation | `phalcom-lsp/src/backend.rs`; tests in `module_navigation.rs`, `workspace_indexing.rs` | definition/references substrate exists; no rename/prepareRename implementation or capability |
| editor syntax | `tools/vsphalcom/syntaxes/phalcom.tmLanguage.json`; tests in `src/test/suite/lsp.e2e.test.ts` | grammar retains enum/ADT declaration wording; no rename lane identified |
| bootstrap baseline | `phalcom-core/src/vm/mod.rs`; `tests/core/modules/universe.rs` | counters and measurement test exist; retain before/after counts at T4 certification |

Parser-backed census: a temporary external helper used `phalcom_ast::parse`
and `AlgebraicDef::legacy_enum_keyword_range` over repository `.ph` files,
excluding hidden directories and build output. It found these current declarations:

| Category | Parser-confirmed declarations / source | Treatment |
|---|---|---|
| A — canonical Universe | 8 declarations in `option/option.ph` (Option), `errors/result.ph` (Result), `object/ordering.ph` (Ordering), `time/calendar/date.ph` (DateOverflow), `month.ph` (Month), `weekday.ph` (Weekday), and `datetime.ph` (LocalTimeDisambiguation, LocalTimeResolution) | migrate in T5 |
| B — first-party examples and parser project sources | `examples/phalcom-parser/src/lex/token.ph` (Keyword, TokenKind); `syntax/ast.ph` (14 parsed sums); `syntax/diagnostic.ph` (DiagnosticKind); `examples/either.ph` has declaration-shaped source but current parser reports errors before yielding it | migrate active owned source; account for parser limitations explicitly |
| C — active positive test sources | `tests/core/typing_integration/sources/either.ph` (Either), `expression.ph` (Expression) | migrate with identity-preserving source changes |
| D — editor golden | `examples/ide-golden/phalcom-enum-highlighting-showcase.ph`; parser recovered Direction and reports errors on competing legacy forms; it contains seven showcase declaration spellings (four Direction variants plus Option, Result, Expr) | migrate/replace as editor migration fixture in T5/T6 |
| E — benchmarks/sample packages | no parser-confirmed declarations found | none at T1 |
| F — active authoritative specs/docs | prose and examples remain to be inventoried during T8; excluded from `.ph` AST census | update in T8 |
| G — embedded/generated/self-hosted source | `examples/phalcom_parser.ph` is a self-hosted embedded parser listing legacy syntax and has extensive current-parser errors; `examples/phalcom-parser/src/lex/lexer.ph` has enum-related text but no enum AST declaration | determine whether active/generated and migrate or document its generated source owner before T7 |
| H — intentional negative fixtures | `tests/fixtures/language/errors/annotation_variant_global_reference.ph` (Ordering), `annotation_variant_visitor_exhaustive.ph` (Shape) | retain as rejection/diagnostic coverage and update expectations at T7 |

The census also found parser limitations: `examples/either.ph` reports
`EnumBehaviorUnsupported`; `examples/phalcom-parser/src/lex/token.ph` reports
recovery errors after two parsed declarations; the IDE showcase reports errors
after its first recovered declaration; `examples/phalcom_parser.ph` and the
self-hosted lexer do not parse cleanly with the current parser. These are
path-classified source migration candidates, not evidence that the files are
historical or negative. T5/T7 must re-run the census and prove no positive
source route remains before parser retirement. The P3 plan’s §7.12 A–F census
categories are extended here with G (embedded/generated/self-hosted) and H
(intentional negative fixtures), resolving the supplied audit’s A–H wording.

Inherited P2 baseline ledger carried forward without new reproduction:

- one metadata alpha-equivalence failure;
- three record-row-polymorphism failures;
- two trait-suite failures;
- formatter drift already present at completed-P2 HEAD;
- full declaration signature graph/SCC traversal retained as a performance follow-up.

G1 result: PASS. The completed-P2 revision, required lifecycle records,
predecessor architecture, metadata schema, source identity substrate, and
bootstrap measurement hook were present.

T2/G2 result: PASS. Canonical Universe source now authors bodiless
`intrinsic type data` anchors for `TypePack`, `LabeledTypePack`, `TupleEntries`,
and `RecordRow` in the existing tuple/record modules. `DeclarationTypeTable`
holds declaration-identity bindings to `TYPE_PACK`, `LABELED_TYPE_PACK`,
`COMPLETE_TYPE_PACK`, and `RECORD_ROW`; the facades have no TypeId form and do
not allocate nominal `StaticDomainId`s. Session declaration/shell, nominal
hierarchy, static-case catalog, and fingerprint passes exclude registered
facades. Source indexing targets each authored declaration. The parser accepts
the no-shape form only for intrinsic data declarations. Core bootstrap-root
selection and runtime class census exclude TypeOnly facades.

T2/G2 focused evidence: AST `data_syntax` (27 tests); semantic intrinsic
facade authority, TupleEntries kind resolution, and source declaration target
tests; core TypeOnly-root unit test; and the core
`boot_01_bootstrap_measurement_separates_catalog_closure_and_execution` test.
That VM gate first exposed a session predeclaration path that rebuilt a
registered facade as a nominal type; after excluding intrinsic facades from
the nominal passes, the gate passed. The P2 starting-revision control passed
the same gate, confirming the first failure was introduced by P3 and was
corrected within T2.

T3/G3 result: PASS. `Tuple::Entries` is an intrinsic member whose public result
kind is `COMPLETE_TYPE_PACK` and whose normalization delegates to
`TypeStore::project_complete`; metadata still uses `CompleteTypePack`. Intrinsic
member selection consumes canonical `IntrinsicTypeOwner` descriptors populated
by declaration identity. `TypeMemberDeclarationId` is now a source-index target
for member definitions. Concrete and symbolic Entries projections pass, as do
the existing At/Map and symbolic projection cases. Focused evidence:
semantic `type_member_closure` (10 tests), the source-index type-member target
test, semantic crate check, and the canonical Universe bootstrap measurement
test with the new Tuple member compiled.

---

## 20. Active plan and next action

Current:

```text
active plan:
    LANG007.C3.P3

next task:
    LANG007.C3.P3.T4 / G4

next action:
    Implement the ratified T4/G4 binding/reuse gate from INC-C3-12, then migrate
    Unit/Tuple/Record source presentation. Preserve existing TypeStore/runtime
    class identities, native descriptor census, and P3's static/runtime
    separation. Keep the T7 graph-build/SCC full-traversal performance follow-up
    and documented predecessor baselines visible.

Advanced-agent continuation prompt: [LANG007.C3.P2-advanced-agent-handoff.md](LANG007.C3.P2-advanced-agent-handoff.md).

P2 implementation, focused acceptance, and lifecycle records are complete. P3
has passed T1/G1, T2/G2, and T3/G3 at the active P3 worktree. The P2→P3
handoff records accepted evidence, focused predecessor failures, formatter
baseline, and the deferred graph-build/SCC performance follow-up.
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
