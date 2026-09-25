---
id: LANG007.C3.P2.requirements-analysis
category: LANG
program: LANG007
checkpoint: LANG007.C3
plan: LANG007.C3.P2
kind: requirements-analysis
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
title: "Generalized Static Domains, Heterogeneous Application, and Associated Type Data"
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 60a27f79d69aa3e79c22e3d19df221466c7db920
prepared: 2026-09-22
depends_on:
  - LANG007.C3.P1
  - LANG007.C2
---

# LANG007.C3.P2 — Requirements Analysis

## Generalized Static Domains, Heterogeneous Application, and Associated Type Data

## 0. Executive conclusion

The pushed post-C2 repository contains a substantial but incomplete implementation of `LANG007.C3.P1` at:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   60a27f79d69aa3e79c22e3d19df221466c7db920
subject:    feat(semantic): type data foundation
parent:     62a6082ab463387c812d558e5561508267ecc63c
```

The implementation successfully establishes several important C3 foundations. The parser and AST now converge runtime products and sums onto one `Statement::Data(AlgebraicDef)` path; `AlgebraicPhase::{Runtime, TypeOnly}` and `AlgebraicShapeSyntax::{Product, Sum}` are explicit; canonical `data` sum syntax, record-payload cases, singleton shorthand, empty sums, and singleton-versus-explicit-zero-argument constructors exist; legacy `enum` normalizes into the canonical algebraic AST; one new `AlgebraicDeclarationInfo` facade exists over product/sum declaration semantics; runtime sum analysis continues to reuse mature `VariantId`, exact-case, GADT, associated-family, pattern, and lowering machinery; `StaticDomainId`, `KindData::StaticDomain`, `TypeData::{ConstInt, StaticData}`, and `StaticArgument` have been declared as the intended static-term representation seams; and compiler lowering explicitly skips `AlgebraicPhase::TypeOnly` declarations.

Those changes are useful and generally point in the right architectural direction. They are not, however, a safe P2 baseline yet.

The audit found correctness-critical P1 gaps that must be closed before P2 generalizes applications and inference:

1. **The current pushed revision appears compile-blocked by incomplete `TypeData` exhaustiveness migration.** `TypeData::ConstInt` and `TypeData::StaticData` were added, but visible exhaustive matches such as `types::instantiation::materialize_type_inner` and `metadata::export::MetadataExporter::export_type_form` contain no corresponding arms and no catch-all. No GitHub Actions workflow runs are recorded for the pushed commit.
2. **`type data` is still predeclared as an ordinary proper `Type`.** `session.rs` and Universe declaration bootstrap ignore `AlgebraicPhase` and create `KindId::TYPE` / `(...)->Type` forms plus `class_object_type` for every `Statement::Data`, including TypeOnly declarations. This violates the core C3 boundary between static domains and runtime value types.
3. **The unified algebraic semantic builder records `TypeOnly` but does not implement TypeOnly semantics.** It still constructs ordinary `DataInfo`/`EnumInfo`, proper-Type payload/result facts, runtime exact-case types, and runtime constructor products for both phases.
4. **The new static-domain/static-term structures are dead scaffolding.** `TypeStore` declares `StaticDomainData`, `AlgebraicStaticConstructorData`, `AlgebraicStaticTerm`, and `StaticArgument`, but there is no static-domain interner, static-constructor interner, or canonical formation path using `TypeData::ConstInt` / `TypeData::StaticData`.
5. **Metadata currently destroys static-domain identity.** `MetadataExporter::export_kind` maps every `KindData::StaticDomain(_)` to `KindNode::Type`; `phalcom-type-meta` remains schema v4 and has no static-domain/static-term/heterogeneous-argument nodes.
6. **Module and incremental declaration infrastructure also loses TypeOnly phase.** `DeclarationKind` still has only `Class`, `Trait`, `Adt`, `Data`, and `Alias`, and `semantic_shard` classifies every `Statement::Data` solely by product/sum shape.
7. **TypeOnly declarations currently reuse runtime associated-family categories.** Because they are built as `DataInfo`/`EnumInfo`, their cases/constructors can enter `Variant`/`DataConstructor` surfaces intended for runtime entities.
8. **The current static-constructor identity scaffold is too weak.** `case_name + arity` cannot distinguish `Dog` from `Dog()` and is insufficient for labeled/record selectors.
9. **P2 source grammar does not yet exist.** `KindSyntax` recognizes only the closed built-in set, and `GenericArgumentSyntax::Single` still contains only `TypeAnnotation`; therefore `S: Shape`, `N: Int`, `Integer<32, Endian::Big>`, and `Tensor<Float, Shape::Vector(3)>` do not have canonical generic-domain/argument syntax.
10. **P2's `TypeData::Applied` migration is repository-wide, not local.** More than twenty semantic consumers inspect applied types and assume `Box<[TypeId]>`; these consumers have different soundness requirements and cannot be mechanically changed as one group.
11. **The current checked substitution path contains a dangerous fallback.** `TypeSubstitution::try_apply` reconstructs `Applied` and uses `unwrap_or(ty)` if application fails, silently retaining the pre-substitution term instead of reporting invalidity.
12. **Associated `type data` needs new declaration/denotation/result categories.** The live `AssociatedSurface` knows only runtime `Variant` and `DataConstructor` families, while `AssociatedOwnerResolution` carries only `Vec<TypeId>`.
13. **There is a meaningful P2 performance risk in type-lambda application.** `TypeStore::apply_type_form` clones the entire `TypeLambdaArena` before beta reduction; more static application would amplify this avoidable arena-sized copy.

The principal architectural conclusion is therefore:

> **Do not start P2 by widening `TypeData::Applied` on top of the current commit. First finish P1's semantic foundation as corrective P1 work: restore build integrity, make TypeOnly declarations real static domains, implement canonical static-domain/static-constructor interning, enforce the runtime boundary, and stop metadata/module infrastructure from lying about those domains. Then P2 can safely generalize application, substitution, inference, aliases, type members, associated domains, persistence, and incremental analysis.**

This is not scope inflation. These defects belong to P1's existing acceptance objective. The lifecycle convention says corrective work stays with the checkpoint/plan objective whose truth it is required to establish.

Once that predecessor is truthful, the accepted P2 architecture remains sound:

```text
TypeId
    canonical handle for every non-row static term

ProperTypeId
    proof that a TypeId is specifically a proper Type

KindId / KindData
    classifier used by binders, application and inference
    including one open-ended StaticDomain(StaticDomainId)

StaticArgument
    Term(TypeId)
    RecordRow(RecordRowId)

RecordRowId
    remains the dedicated row-domain handle

InferVarId
    remains the non-row inference variable, classified by KindId

row inference
    remains the dedicated RecordRow solver authority
```

P2 should generalize that substrate coherently. It should not introduce a second static-term universe, a second generic-domain registry, fake proper types for compile-time constants/type-data terms, or a new RecordRow representation.

---

# Part I — Audit authority and repository state

## 1. Purpose

This document performs two jobs:

1. audit what the partial P1 commit actually implemented, including defects and incomplete obligations that materially affect the P2 starting point;
2. turn the accepted C3 P2 requirements into a repository-grounded requirements analysis for the later P2 patch-grade implementation plan.

It is not itself the P2 execution plan.

## 2. Normative P2 objective

The accepted P2 title and objective are:

**LANG007.C3.P2 — Generalized Static Domains, Heterogeneous Application, and Associated Type Data**

> Make user-defined static domains first-class throughout generic application, substitution, first-order inference, aliases/type members, imports/metadata, and associated `::` lookup.

The fixed P2 scope includes:

```text
heterogeneous generic application
static argument syntax
user-defined binder domains
first-order type-data inference
checked substitution
static aliases/definitions
type-member domain convergence
associated type data
unified ::
metadata/import persistence
incremental/source indexing
explicit trait boundary
```

P2 does not own arbitrary compile-time execution, theorem proving, general static pattern matching, trait requirements that bind whole type-data domains, pack partition search, Universe/corpus final migration, or VM static solving.

## 3. Audit method and evidence level

The audit inspected the pushed tree through the connected GitHub repository at revision `60a27f79...` and compared it to the completed C2 baseline `62a6082...` and the C3 requirements/specification already present in the repository.

No GitHub Actions workflow runs are recorded for the current commit. This document therefore distinguishes:

```text
Verified by source inspection
    direct facts visible in the pushed tree

Compile-blocking by Rust inspection
    enum-match/exhaustiveness inconsistencies that should fail compilation

Not execution-verified
    behavior for which no completed test/CI evidence exists
```

The C3 checkpoint itself remains `UNVERIFIED`, which is consistent with the absence of recorded verification evidence.

## 4. Lifecycle state is inconsistent

`LANG007.C3-CHECKPOINT.md` begins with an appropriate state:

```yaml
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
active_plan: LANG007.C3.P1
```

and the plan ledger marks P1 active. Later sections, however, still state effectively:

```text
active plan: null
next plan: LANG007.C3.P1
next action: create P1 plan
```

`PROGRAM.md` and `STATUS.md` are similarly only partially rebaselined: early C3 sections acknowledge the new checkpoint, while later current-state/next-action prose still refers to C2-era plans. `STATUS.md` also lacks a coherent C3 plan ledger.

Before P1 is accepted or P2 activated:

- `LANG007.C3-CHECKPOINT.md` must become the single truthful checkpoint state;
- `PROGRAM.md` and `STATUS.md` must agree with it;
- P1 must remain `IN_PROGRESS/PARTIAL` until the corrective gaps below are closed;
- no P1 walkthrough/handoff may claim completion prematurely;
- historical completed C1/C2 records should not be rewritten merely to look current.

---

# Part II — Partial P1 implementation audit

## 5. What P1 implemented correctly

### 5.1 Canonical algebraic AST convergence

The old semantic AST split has substantially converged. The live AST now exposes one structure equivalent to:

```rust
pub struct AlgebraicDef {
    pub name: String,
    pub name_range: SourceRange,
    pub phase: AlgebraicPhase,
    pub implementation: ImplementationKind,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub shape: AlgebraicShapeSyntax,
    pub attributes: Vec<Attribute>,
    pub range: SourceRange,
    pub legacy_enum_keyword_range: Option<SourceRange>,
}

pub enum AlgebraicPhase {
    Runtime,
    TypeOnly,
}

pub enum AlgebraicShapeSyntax {
    Product(ProductShapeSyntax),
    Sum(SumShapeSyntax),
}
```

`Statement` now uses `Statement::Data`; `EnumDef`/`VariantDecl` remain compatibility aliases rather than independent semantic statement categories. This is the correct direction.

### 5.2 Required product/sum syntax is present

The parser has support and tests for runtime tuple/record products, singleton sums, comma shorthand, tuple and record payload cases, explicit zero-argument constructor cases, empty sums, `type data`, intrinsic algebraic declarations, legacy `enum` normalization, and mixed-body rejection.

The accepted distinction is preserved:

```phalcom
data Example {
    Singleton
    Constructed()
}
```

### 5.3 Runtime ADT/GADT machinery is reused

The new algebraic builder retains the mature runtime identities/products:

```text
VariantId
VariantFamilyId
VariantFieldId
VariantConstructorId
CaseTypeEnvironment
ExactCase
EnumInfo
VariantInfo
```

Record-payload cases are projected into the same runtime sum machinery rather than creating a second ADT implementation. This is exactly what C3 requires.

### 5.4 Runtime representation remains specialized

Compiler dispatch now distinguishes the canonical semantic declaration by phase/shape and preserves the specialized product and ADT lowering paths. Retaining `Bytecode::Data` and `Bytecode::Enum` internally is not a defect; those remain runtime representation mechanisms beneath a unified language model.

### 5.5 Correct P1/P2 type seams were declared

The pushed source declares:

```rust
pub struct StaticDomainId(pub u32);

pub enum KindData {
    // existing domains...
    StaticDomain(StaticDomainId),
}

pub enum StaticArgument {
    Term(TypeId),
    RecordRow(RecordRowId),
}

pub enum TypeData {
    // existing terms...
    ConstInt(i64),
    StaticData(AlgebraicStaticTerm),
}
```

These are the correct representation *directions*. They are not yet implemented as working semantic infrastructure.

## 6. Audit finding register

| ID | Severity | Finding | Why it matters for P2 |
|---|---|---|---|
| P1-AUD-01 | BLOCKER | new `TypeData` variants are not integrated into exhaustive consumers | current revision likely does not compile cleanly |
| P1-AUD-02 | SOUNDNESS BLOCKER | `type data` predeclares as proper `Type` | destroys static-domain/proper-type boundary |
| P1-AUD-03 | ARCHITECTURE BLOCKER | TypeOnly phase is recorded but not semantically implemented | runtime/static semantics collapse |
| P1-AUD-04 | ARCHITECTURE BLOCKER | no static-domain/constructor interner or formation APIs | static terms are scaffolding only |
| P1-AUD-05 | PERSISTENCE BLOCKER | metadata maps `StaticDomain` to `Type` | imported identity and generic contracts become unsound |
| P1-AUD-06 | MODULE BLOCKER | declaration shells discard Runtime/TypeOnly phase | cross-module/incremental semantics cannot be correct |
| P1-AUD-07 | SEMANTIC BUG | TypeOnly declarations reuse runtime associated families | static constructors may acquire runtime invocation semantics |
| P1-AUD-08 | IDENTITY BUG | static constructor identity uses name+arity | accepted selector distinctions can collide |
| P1-AUD-09 | COVERAGE GAP | no semantic TypeOnly/static-term acceptance tests | parser success hides semantic absence |
| P1-AUD-10 | VERIFICATION GAP | no CI evidence; checkpoint remains UNVERIFIED | P1 cannot truthfully close |
| P1-AUD-11 | LIFECYCLE BUG | checkpoint/program/status disagree | next executor receives contradictory state |
| P1-AUD-12 | TOOLING GAP | source taxonomy still uses Enum/Data only | static-domain source identity cannot be canonical |


## 7. P1-AUD-01 — current revision is likely compile-blocked

`TypeData` gained:

```rust
TypeData::ConstInt(i64)
TypeData::StaticData(AlgebraicStaticTerm)
```

but multiple pre-existing exhaustive matches were not migrated.

A direct example is `phalcom-semantic/src/types/instantiation.rs::materialize_type_inner`. Its exhaustive match handles the previous variants and terminates with the previous leaf cases:

```rust
TypeData::Lambda(_)
| TypeData::SelfType(_)
| TypeData::Never
| TypeData::Unit
| TypeData::Nominal { .. }
| TypeData::ClassObject { .. } => Ok(ty),
```

There is no `ConstInt` arm, no `StaticData` arm, and no wildcard.

`phalcom-semantic/src/metadata/export.rs::MetadataExporter::export_type_form` is another visible exhaustive match over `TypeData` with no `StaticData` arm. Source inspection also found no `ConstInt`/`StaticData` handling in several other core transformation files (`types/substitution.rs`, `checker/inference.rs`, `types/rigid.rs`, `types/variance.rs`, `types/relation.rs`). Some of those have unrelated wildcard branches, so each must be audited rather than assumed compile-failing, but the two explicit exhaustive examples above are sufficient to make build integrity a prerequisite.

### Corrective requirement

Before P2 begins:

1. run the smallest compile/check that proves the semantic crate and metadata exporter are buildable;
2. enumerate every `match` over `TypeData`;
3. add explicit `ConstInt`/`StaticData` semantics based on consumer responsibility;
4. do not regain compilation through blanket `_ =>` arms that erase static semantics;
5. add focused regression coverage for canonical transforms over the new terms.

P2 will make `Applied` heterogeneous as well; leaving transformation exhaustiveness unresolved now would multiply the debt.

## 8. P1-AUD-02 — `type data` is incorrectly a proper runtime type

The strongest semantic defect is declaration predeclaration.

The live `session.rs` declaration scan handles every `Statement::Data(data_def)` without consulting `data_def.phase`. It creates a nominal form of kind `Type` (or an arrow ending in `Type`) and a `class_object_type` just as it does for runtime algebraic declarations.

The Universe bootstrap helper in `declarations.rs` follows the same model through `insert_source_nominal`.

Therefore:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}
```

is currently inserted into `DeclarationTypeTable` as though `Shape` were an ordinary runtime value type/type constructor.

This violates the central C3 law:

```text
stored/canonical static term
    !=
proper runtime Type
```

and effectively bypasses the intended `ProperTypeId` proof boundary at declaration origin.

### Consequences

A declaration-backed resolver can obtain an ordinary type form for a TypeOnly declaration. That enables several unsound or inconsistent downstream states:

- a runtime annotation may accept a type-data domain as a value type;
- a class-object form exists for a declaration that the compiler deliberately does not materialize;
- associated cases can be analyzed through runtime exact-case machinery;
- generic application can accept the domain through ordinary type-form APIs;
- runtime code may later reference a declaration global that compiler lowering intentionally skipped.

Even if one particular source path happens to fail later, the declaration identity is already wrong at the semantic owner.

### Corrective requirement

TypeOnly declarations must not be inserted into the runtime declaration table as `KindId::TYPE` forms with class-object identity.

The correct split is:

```text
Runtime algebraic declaration
    declaration-backed proper Type / type constructor
    runtime class/declaration surface as appropriate

TypeOnly algebraic declaration
    declaration-backed StaticDomainId
    KindData::StaticDomain(domain)
    static constructor/case declarations
    no runtime value type
    no class_object_type
```

A derived lookup map `DeclarationId -> StaticDomainId` is acceptable. It is not a second generic-domain registry if `TypeStore` remains the canonical domain interner and the declaration map is merely the semantic declaration index.

## 9. P1-AUD-03 — TypeOnly phase is metadata, not semantic behavior

`phalcom-semantic/src/checker/algebraic_declaration.rs::build_algebraic_semantics` constructs ordinary product/sum semantic products for both phases. `def.phase` is attached to the final `AlgebraicDeclarationInfo`, but the construction path still uses concepts such as:

```text
DataInfo
DataConstructorSignature
EnumInfo
VariantInfo
VariantConstructorSignature
ExactCase
CaseTypeEnvironment over proper-Type results
```

Generic GADT result checks still use ordinary proper-type result formation.

In other words, the new facade is presently a runtime data/enum builder with a phase field attached, not yet the phase-neutral algebraic authority required by C3.

### Corrective requirement

The common algebraic semantic authority must own common declaration facts once, and then project phase-specific products:

```text
common algebraic facts
    owner
    phase
    implementation provenance
    generic/static binder contract
    source
    shape
    component/case declaration identities

Runtime projection
    product constructor OR sum variants
    exact runtime case types
    GADT runtime evidence
    runtime associated constructor/value families

TypeOnly projection
    StaticDomainId
    static constructor/case declarations
    static payload-domain contracts
    canonical static terms
    NO exact runtime case type
    NO runtime InvocationTargetId
```

The same parser-level algebraic shape should feed both; runtime and static semantic products must not be conflated.

## 10. P1-AUD-04 — static-domain types are only scaffolding

The live `TypeStore` declares:

```rust
pub enum StaticDomainData {
    ConstInt,
    Nominal { declaration: DeclarationId },
}

pub struct AlgebraicStaticConstructorData { ... }
pub struct AlgebraicStaticTerm { ... }
```

but source inspection finds no canonical storage fields or formation methods corresponding to:

```text
static_domains
static_domain_interner
static_constructors
static_constructor_interner
```

and no live formation path using:

```text
TypeData::ConstInt
TypeData::StaticData
StaticArgument::...
```

### Corrective requirement

P1 closure needs canonical APIs conceptually equivalent to:

```rust
fn intern_static_domain(&mut self, data: StaticDomainData) -> StaticDomainId;
fn static_domain(&self, id: StaticDomainId) -> &StaticDomainData;
fn static_domain_kind(&mut self, id: StaticDomainId) -> KindId;

fn const_int(&mut self, value: i64) -> TypeId;

fn intern_static_constructor(
    &mut self,
    declaration: StaticConstructorDeclarationData,
) -> AlgebraicStaticConstructorId;

fn static_data(
    &mut self,
    domain: StaticDomainId,
    constructor: AlgebraicStaticConstructorId,
    arguments: Box<[StaticArgument]>,
) -> Result<TypeId, StaticDataFormationError>;
```

Formation must validate constructor/domain ownership, selector shape, arity/labels, argument static domains, and later associated owner specialization. Successful terms must be hash-consed through the existing canonical TypeStore graph.

## 11. P1-AUD-05 — metadata currently lies about static domains

The live exporter contains the mapping:

```rust
KindData::StaticDomain(_) => KindNode::Type
```

with a comment that static domains are effectively proper-type-kinded in metadata.

That is architecturally incompatible with C3. If `Shape` and `Type` serialize as the same domain classifier, metadata cannot validate generic arguments, recover imported inference-variable domains, or produce stable interface fingerprints for distinct static signatures.

`phalcom-type-meta/src/header.rs` still reports schema v4; `KindNode` has no static-domain form; `TypeNode::Applied` still contains `Box<[TypeNodeId]>`; `ScopedTypeNode::Applied` still contains `Box<[ScopedTypeNodeId]>`.

### Corrective requirement

The temporary `StaticDomain -> Type` mapping must be removed. Stable metadata needs explicit static-domain identity, conceptually:

```rust
enum StaticDomainRef {
    ConstInt,
    Nominal {
        declaration: StableDeclarationRef,
    },
    Associated {
        declaration: StableAssociatedStaticDomainDeclarationRef,
        owner_environment: Box<[StaticArgumentRef]>,
    },
}

enum KindNode {
    Type,
    RecordRow,
    TypePack,
    LabeledTypePack,
    CompleteTypePack,
    StaticDomain(StaticDomainRef),
    Arrow { ... },
}
```

The exact stable graph may use indexed domain nodes rather than embedding full records. The non-negotiable law is:

```text
metadata(static domain D) != metadata(Type)
```

## 12. P1-AUD-06 — module declaration shells lose algebraic phase

`phalcom-modules::DeclarationKind` currently contains:

```rust
Class
Trait
Adt
Data
Alias
```

while `phalcom-semantic/src/semantic_shard.rs` classifies every `Statement::Data` solely by shape:

```text
sum     -> DeclarationKind::Adt
product -> DeclarationKind::Data
```

`AlgebraicPhase` is discarded.

This matters directly to P2 because P2 owns imports, metadata, and incremental/static-domain behavior. A cross-module provider cannot export a valid static domain if its declaration shell has already recorded it as runtime ADT/data.

### Corrective requirement

Preserve phase and shape orthogonally in declaration-shell semantics. A conceptual model is:

```text
DeclarationCategory
    Class
    Trait
    Algebraic
    Alias

AlgebraicSummary
    phase: Runtime | TypeOnly
    shape: Product | Sum
```

A smaller mechanical encoding is acceptable, but do not replace the old `Enum`/`Data` split with four unrelated runtime/static product/sum categories.

## 13. P1-AUD-07 — TypeOnly associated surfaces are runtime surfaces

Current associated categories are:

```rust
pub enum AssociatedFamilyKind {
    Variant,
    DataConstructor,
}

pub enum AssociatedMemberId {
    Variant(VariantId),
    DataConstructor(DataConstructorId),
    DataComponent(DataComponentId),
}
```

Because TypeOnly declarations currently flow through ordinary `EnumInfo`/`DataInfo`, their cases/constructors can be published through these runtime-oriented associated surfaces.

A valid static term such as:

```phalcom
Shape::Vector(3)
```

must resolve to compiler-owned static constructor formation. It must not become `AssociatedMemberId::Variant` plus a runtime `InvocationTargetId::VariantConstructor` and exact runtime case type.

### Corrective requirement

Top-level TypeOnly case resolution must have a static result category with no runtime invocation target before P2 builds associated type-data on top of the namespace.

## 14. P1-AUD-08 — static constructor identity is too weak

The current scaffold records roughly:

```text
domain
declaration
case_name
arity
```

That cannot preserve accepted selector identity. These are distinct algebraic cases:

```phalcom
data Animal {
    Dog
    Dog()
    Dog(_ name: String)
    Dog(named age: Int)
}
```

`Dog` and `Dog()` share base name and arity zero. Labels also participate in selector identity.

### Corrective requirement

Static constructor/case declaration identity must use the same selector-shape principles as runtime cases, conceptually:

```rust
struct StaticConstructorDeclarationId {
    domain_declaration: StaticDomainDeclarationIdentity,
    selector: Selector,
    payload_kind: AlgebraicCasePayloadKind,
}
```

or an equivalent stable declaration-owned identity.

A concrete static term then contains:

```text
concrete static domain identity
constructor/case declaration identity
canonical static arguments
```

## 15. P1-AUD-09 — tests prove syntax, not TypeOnly semantics

The new parser tests are useful and the runtime semantic tests verify legacy `enum`/canonical `data` similarity plus record-payload sums. They do not prove:

```text
type data declaration -> StaticDomainId
type data kind != Type
no class object for type data
static term canonicalization
proper_type(static term) rejection
runtime-use rejection
static payload validation
metadata static-domain preservation
```

P1 closure must add these tests before P2 treats the substrate as established.

---

# Part III — Corrective predecessor gate

## 16. P2 must not silently absorb unfinished P1 semantics

The later P2 implementation plan should contain a hard predecessor condition:

```text
P2 MAY START ONLY WHEN:
    P1 source compiles
    AND TypeOnly declarations are not proper runtime Types
    AND TypeStore interns nominal static domains
    AND TypeStore interns static constructor/case terms
    AND TypeOnly cases do not publish runtime invocation semantics
    AND ProperTypeId boundary tests pass
    AND module/declaration phase is preserved
    AND checkpoint state records those facts truthfully
```

If workflow requires a new numbered corrective work package rather than continuing the still-open P1 work unit, it should remain under P1's acceptance objective. It should not be mislabeled ordinary P2 scope.

## 17. Metadata transition strategy

The original P1 requirements expected the first C3 metadata transition during P1. The partial implementation did not land one; schema remains v4 and the only new mapping is incorrect.

Two disciplined strategies remain:

### Strategy A — narrow P1 metadata closure

P1 introduces stable nodes for phase, shape, nominal static domain, and top-level static constructor terms. P2 later introduces a second schema change for heterogeneous `Applied` and associated-domain specialization if required.

### Strategy B — reserve the full C3 static-domain graph in the corrective closure

If no P1 metadata format has been externally shipped, the corrective closure can define the next stable graph with `StaticArgumentRef` and associated-domain-ready references while only implementing P1 behavior. P2 then fills in additional producers/consumers without another immediate incompatible format transition.

### Recommendation

Prefer Strategy B if repository compatibility policy confirms the partial P1 format has not become an external compatibility commitment. It avoids publishing a deliberately short-lived schema. Do not, however, claim P2 language features as implemented merely because the metadata graph can represent them.

The exact numeric schema version should be chosen from the live schema policy during implementation rather than hard-coded by this analysis.

---

# Part IV — P2 acceptance objective and fixed architecture

## 18. Acceptance objective

After the P1 foundation is truthful, P2 must make supported static domains first-class through the complete semantic pipeline:

```text
source syntax
    ↓
static-domain resolution
    ↓
generic binder contracts
    ↓
static argument elaboration
    ↓
canonical heterogeneous application
    ↓
substitution / scoped lambdas / inference
    ↓
static aliases and type members
    ↓
associated static-domain lookup
    ↓
stable metadata / imports
    ↓
incremental/source identity
```

The canonical P2 gate remains:

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

and the following must be canonical semantic identities:

```text
Parser::State
Parser::State::Done
Container<Int>::State
Container<String>::State
```

## 19. Fixed representation model

P2 must retain:

```text
TypeId
    canonical non-row static-term handle

ProperTypeId
    proof of actual Type domain

KindId
    one classifier architecture for Type, packs, rows, arrows and user static domains

StaticDomainId
    canonical store-relative user/primitive static-domain identity

StaticArgument
    heterogeneous application envelope

RecordRowId
    dedicated row identity

InferVarId
    non-row inference variable with KindId classifier
```

P2 must not introduce:

```text
StaticTermId as a parallel all-purpose canonical arena
GenericDomainId separate from KindId
PackVarId separate from existing InferVarId architecture
fake TypeId row encoding
runtime static-term solver
```

---

# Part V — Static-domain syntax and resolution

## 20. Current closed `KindSyntax` is insufficient

Current generic parameters contain:

```rust
pub struct GenericParameterSyntax {
    ...
    pub kind: Option<KindSyntax>,
}
```

and `parse_kind_expression` recognizes only:

```text
Type
RecordRow
TypePack
LabeledTypePack
arrow kinds
grouping
```

It cannot represent:

```phalcom
N: Int
S: Shape
```

### Required change

The right side of a generic binder is a static-domain expression, not merely one of a parser-closed set of primitive kinds.

Two implementations are acceptable:

```text
Option 1
    extend KindSyntax with a Named/StaticDomainRef form

Option 2
    introduce StaticDomainSyntax and migrate the binder AST
```

Option 1 is lower-risk for P2 because current C1/C2 code already treats `KindId` as the classifier. The important change is semantic resolution, not naming.

A suitable extension is conceptually:

```rust
enum KindSyntax {
    Type(...),
    RecordRow(...),
    TypePack(...),
    LabeledTypePack(...),
    CompleteTypePack(...),
    StaticDomainRef(StaticSymbolRef),
    Arrow { ... },
    Grouped { ... },
    Invalid { ... },
}
```

## 21. `resolve_kind_syntax(store, syntax)` is no longer sufficient

The current resolver can be store-only because every recognized domain is compiler-known. Resolving `Shape` requires declaration/import name resolution.

P2 needs a resolver with access to:

```text
current module
import/module symbol resolution
predeclared type-data domain facts
TypeStore domain registry
source range for diagnostics
```

Conceptually:

```text
resolve_static_domain_syntax(
    store,
    semantic declarations,
    current module/import resolver,
    syntax
) -> KindId
```

Do not resolve a user domain by name string inside `TypeStore`; TypeStore owns canonical term/domain identity, not module namespace lookup.

## 22. Forward-reference ordering

User-defined static domains create a declaration-order dependency. A binder may refer to a domain whose payload body has not yet been elaborated.

Required semantic ordering:

```text
Phase A
    discover all declaration identities
    preserve Runtime/TypeOnly phase
    allocate StaticDomainId for TypeOnly declarations

Phase B
    resolve generic binder domains

Phase C
    elaborate algebraic payloads/cases/constraints

Phase D
    publish final products
```

Nominal static-domain identity depends only on the declaration identity, so it can be predeclared before case/payload analysis.

## 23. Primitive `Int` domain

`N: Int` in static-domain position denotes the compile-time Int domain, not arbitrary runtime `Int` values.

P2 should represent the classifier through the same static-domain machinery:

```text
StaticDomainData::ConstInt
    -> StaticDomainId
    -> KindData::StaticDomain(id)
```

This lets ordinary binder/application/inference logic treat `Int` and user type-data domains uniformly while preserving the fact that runtime `Int` values are not automatically lifted into static identity.

## 24. `TupleEntries` bridge

P2 requirements list:

```phalcom
E: TupleEntries
```

while P3 owns final Universe exposure of `TupleEntries`.

The semantic identity is already fixed:

```text
TupleEntries public domain
    ==
C2 CompleteTypePack canonical domain
```

P2 should make the domain resolver capable of classifying this domain without introducing a second term representation. P3 can later expose the canonical source declaration/facade over the same `KindId::COMPLETE_TYPE_PACK` authority.

---

# Part VI — Heterogeneous generic application

## 25. Canonical `Applied` representation

Current:

```rust
TypeData::Applied {
    origin: TypeId,
    arguments: Box<[TypeId]>,
}
```

P2 target:

```rust
TypeData::Applied {
    origin: TypeId,
    arguments: Box<[StaticArgument]>,
}
```

with:

```rust
pub enum StaticArgument {
    Term(TypeId),
    RecordRow(RecordRowId),
}
```

Classifier law:

```text
Term(t)      -> kind_of(t)
RecordRow(r) -> KindId::RECORD_ROW
```

## 26. TypeStore generalized application API

Introduce one canonical API conceptually equivalent to:

```rust
pub fn apply_static_form(
    &mut self,
    origin: TypeId,
    arguments: &[StaticArgument],
) -> Result<TypeId, StaticApplicationError>;
```

Retain a type-only wrapper:

```rust
pub fn apply_type_form(
    &mut self,
    origin: TypeId,
    arguments: &[TypeId],
) -> Result<TypeId, TypeApplicationError>;
```

which wraps each argument as `StaticArgument::Term`.

`apply_static_form` must:

1. derive argument classifiers;
2. validate the origin arrow kind;
3. validate domain alignment;
4. beta-reduce lambdas where applicable;
5. flatten partial applications;
6. preserve heterogeneous argument categories;
7. intern the result canonically;
8. return structured failures rather than panic or silent fallback.

## 27. Preserve flat application nodes

Current application flattening should remain:

```text
apply(apply(F, [A]), [B])
    ->
Applied(F, [A, B])
```

After P2, `A/B` may be any supported `StaticArgument`. Flat nodes reduce recursive normalization depth and improve hashing, comparison, substitution, and metadata density.

## 28. Applied-type consumer migration must be classified

Repository search shows `TypeData::Applied` assumptions across many subsystems, including:

```text
advisory/formal
export
types/substitution
checker/associated
impls
traits
types/rigid
types/store
editor
types/variance
checker/expected
types/relation
types/specialization
checker/gadt_proof
checker/composition
types/environment
checker/call
types/instantiation
metadata/export
checker/context
types/annotation
checker/inference
types/projection
checker/expression
checker/row_inference
```

They should not be changed mechanically.

### Category A — general static-term consumers

These must preserve arbitrary `StaticArgument`:

```text
TypeStore decomposition
substitution/materialization
type lambdas
inference
formatting
metadata
generic environment construction
```

### Category B — proper-runtime-type consumers

Subtyping, runtime hierarchy/dispatch, runtime field/callable typing, and similar code must not silently treat a static argument as a runtime type merely because it uses a TypeId carrier.

These consumers should either accept `ProperTypeId` or explicitly prove the relevant term kind is `Type`.

### Category C — trait/conformance consumers

Do not widen these automatically. Trait generic static-domain support is explicitly bounded in C3. If coherence/evidence is not migrated, declaration/application must reject user static-domain trait parameters.

### Category D — presentation/advisory consumers

These should learn to render static arguments, but presentation strings must never become identity.

---

# Part VII — Static generic argument syntax

## 29. Current generic argument AST is type-form-only

Current:

```rust
pub enum GenericArgumentSyntax {
    Single(TypeAnnotation),
    Expansion(TypePackExpansionSyntax),
}
```

This cannot represent:

```phalcom
Integer<32, Endian::Big>
Tensor<Float, Shape::Vector(3)>
```

### Required AST direction

Conceptually:

```rust
pub enum GenericArgumentSyntax {
    Single(StaticArgumentSyntax),
    Expansion(TypePackExpansionSyntax),
}

pub enum StaticArgumentSyntax {
    TypeForm(TypeAnnotation),
    IntLiteral { value: i64, range: SourceRange },
    StaticTerm(StaticTermSyntax),
}
```

## 30. Static term grammar must be bounded

Do not use general runtime `Expr` as the generic static argument grammar.

The static grammar needs only deterministic language-authorized constructs such as:

```text
Int literal
proper type form
type-data singleton case
type-data tuple constructor term
type-data record constructor term
associated static-domain/case path
existing compiler static projections
```

It must not authorize I/O, ordinary runtime message dispatch, arbitrary closures, mutable state, randomness, or other runtime-dependent operations.

## 31. Domain-directed elaboration

Generic signatures already provide expected binder classifiers. Static argument elaboration should use those classifiers directly.

For:

```phalcom
data Integer<Bits: Int, Order: Endian>()
```

checking:

```phalcom
Integer<32, Endian::Big>
```

becomes deterministic:

```text
expected Bits domain = ConstInt
    32 -> canonical ConstInt(32)

expected Order domain = Endian
    Endian::Big -> canonical StaticData(Endian, Big, [])
```

This is safer and simpler than parsing arbitrary runtime expressions and trying to promote them afterward.

---

# Part VIII — Instantiation and checked substitution

## 32. `GenericInstantiation` generalization

Current:

```rust
pub struct GenericInstantiation {
    type_bindings: HashMap<TypeParameterId, TypeId>,
    row_bindings: HashMap<TypeParameterId, RecordRowId>,
}
```

The split correctly acknowledges the row boundary.

Two physical layouts are viable:

```text
Option A
    static_bindings: HashMap<TypeParameterId, StaticArgument>

Option B
    term_bindings: HashMap<TypeParameterId, TypeId>
    row_bindings: HashMap<TypeParameterId, RecordRowId>
```

### Recommendation

Prefer Option B internally if it avoids churn. Every non-row static term already has a TypeId carrier. Expose generalized `StaticArgument` accessors to consumers so they do not branch on private storage details.

## 33. Current checked substitution fallback is unacceptable for P2

Current `TypeSubstitution::try_apply` reconstructs `Applied` and effectively does:

```rust
store.apply_type_form(...).unwrap_or(ty)
```

If substitution makes an application invalid, the old term is retained silently.

P2 explicitly requires checked re-canonicalization, so the canonical path must return an error.

A suitable error family is conceptually:

```rust
enum StaticSubstitutionError {
    Application(StaticApplicationError),
    StaticData(StaticDataFormationError),
    Pack(PackFormationError),
    Row(RecordRowFormationError),
    AssociatedDomain(AssociatedDomainFormationError),
}
```

Every reconstructed node must pass through its checked canonical constructor.

## 34. Required recursive substitution coverage

P2 substitution must traverse:

```text
StaticArgument
TypeData::Applied
TypeData::StaticData
ConstInt
type-member projections
associated-domain specializations
packs
rows
scoped/lambda terms
```

Example:

```text
Shape::Vector(N)
[N -> 3]
    =>
canonical Shape::Vector(3)
```

No malformed noncanonical residual is acceptable.


# Part IX — Scoped type lambdas

## 35. Scoped application currently assumes every argument is a scoped TypeId

Current:

```rust
ScopedTypeData::Applied {
    origin: ScopedTypeId,
    arguments: Box<[ScopedTypeId]>,
}
```

and:

```rust
beta_reduce(..., args: &[TypeId], ...)
```

are too narrow once a generic application can contain RecordRow or other heterogeneous static categories.

### Required representation

Conceptually:

```rust
enum ScopedStaticArgument {
    Term(ScopedTypeId),
    RecordRow(ScopedRecordRowTerm),
}

ScopedTypeData::Applied {
    origin: ScopedTypeId,
    arguments: Box<[ScopedStaticArgument]>,
}
```

The exact row representation should reuse the existing scoped row/tail machinery. Do not manufacture fake scoped TypeIds for rows.

## 36. Scoped static-data terms are required

A static constructor term may contain bound variables:

```text
Shape::Vector(N)
```

where `N` is a generic/lambda parameter in the ConstInt domain.

A fully concrete term can remain a `ScopedTypeData::Free(TypeId)`. A term containing bound variables needs structural scoped representation, conceptually:

```rust
ScopedTypeData::StaticData {
    domain: ScopedStaticDomainReference,
    constructor: StaticConstructorDeclarationId,
    arguments: Box<[ScopedStaticArgument]>,
}
```

Alpha-normalization, free-term collection, substitution, partial beta reduction, and metadata export must all traverse it.

## 37. Beta-reduction performance problem

Current `TypeStore::apply_type_form` works around borrowing by cloning the complete `TypeLambdaArena`:

```rust
let mut arena = self.lambda_arena.clone();
let res = arena.beta_reduce(lambda_id, arguments, self);
self.lambda_arena = arena;
```

That is proportional to the accumulated arena size on every lambda application. P2 increases type-level/static application frequency and would magnify this cost.

### Required correction

Use an O(1) move-out/restore or equivalent borrow-safe architecture, for example conceptually:

```rust
let mut arena = std::mem::take(&mut self.lambda_arena);
let result = arena.beta_reduce(..., self);
self.lambda_arena = arena;
```

provided failure paths restore the arena correctly.

Performance invariant:

```text
one beta reduction must not clone work proportional to the entire lambda arena
```

---

# Part X — First-order static inference

## 38. Reuse `InferVarId`

The current solver already stores:

```rust
InferenceVariable {
    kind: KindId,
    ...
}
```

and solved non-row variables use canonical `TypeId` solutions. This is the correct foundation.

For:

```phalcom
S: Shape
```

instantiate:

```text
InferVarId(kind = KindId::StaticDomain(ShapeDomainId))
```

Do not introduce `StaticInferVarId`.

RecordRow remains the dedicated row-solver exception.

## 39. `InferenceTerm` needs structural static terms

Current `InferenceTerm::Applied` contains `Box<[InferenceTerm]>` and there is no structural static-data case.

P2 needs enough solver structure to represent symbolic terms, conceptually:

```rust
enum InferenceStaticArgument {
    Term(InferenceTerm),
    RecordRow(InferenceRecordTerm),
}

enum InferenceTerm {
    Canonical(TypeId),
    Var(InferVarId),
    ...
    Applied {
        origin: Box<InferenceTerm>,
        arguments: Box<[InferenceStaticArgument]>,
    },
    StaticData {
        domain: StaticDomainId,
        constructor: AlgebraicStaticConstructorId,
        arguments: Box<[InferenceStaticArgument]>,
    },
    ...
}
```

Concrete constants/static-data terms with no variables may remain `Canonical(TypeId)`.

## 40. Required unification laws

Same domain and constructor recurse structurally:

```text
Shape::Vector(N)
=
Shape::Vector(3)

=> N = 3
```

Different constructors are disjoint:

```text
Shape::Scalar
!=
Shape::Vector(N)
```

Different nominal domains are disjoint even if selectors/payloads match:

```text
Shape::Vector(3)
!=
OtherShape::Vector(3)
```

No user-defined `==`, hash, ordinary method, runtime reflection, constructor search, or backtracking participates.

## 41. Domain-safe solving

A variable classified by the `Shape` domain may only solve to a canonical term whose exact `KindId` is the same `KindData::StaticDomain(ShapeDomainId)`.

It must not solve to:

```text
proper Type
a different type-data domain
ConstInt
RecordRow
Dynamic
```

The existing exact `KindId` classifier is sufficient once P1's static-domain interner exists.

## 42. Static equality is not subtyping

The current inference engine supports lower/upper subtype constraints. P2 must not invent a subtype lattice for arbitrary type-data terms.

Initial law:

```text
Type-domain variable
    existing subtype/equivalence reasoning

nominal static-domain variable
    exact-domain structural equality/unification

RecordRow variable
    existing row solver
```

If a path attempts arbitrary subtype bounds over `Shape`, reject/mark unsupported rather than treating static constructors as runtime types.

## 43. Occurs check and materialization

Occurs-check traversal must descend through:

```text
heterogeneous Applied arguments
StaticData arguments
existing packs
row/static structures where relevant
```

Materialization must use canonical TypeStore formation APIs. Query-local `InferVarId` may never enter a durable `TypeId`, metadata node, fingerprint, or snapshot.

---

# Part XI — Static aliases and definitions

## 44. Current alias grammar is Type-only

Current type aliases contain a `TypeAnnotation` body and parse essentially:

```phalcom
type Name = TypeForm
```

P2 requires:

```phalcom
type DefaultShape: Shape = Shape::Scalar
type P: TypePack = T::Positionals
```

### Required AST direction

Conceptually:

```rust
struct TypeAliasDef {
    ...
    result_domain: Option<StaticDomainSyntax>,
    body: StaticDefinitionSyntax,
}
```

Unannotated aliases keep the existing default:

```text
no explicit result domain
    => result domain Type
```

## 45. Existing alias semantic storage is mostly reusable

`TypeAliasInfo` already contains:

```text
kind: KindId
form: TypeId
```

so non-row static aliases fit naturally once the resolver stops assuming `kind == Type`.

The alias subsystem must preserve symbolic forms and generic lambdas whose result kind is a static domain.

RecordRow-result aliases should continue to respect the row-specific canonical representation rather than forcing rows into TypeId.

---

# Part XII — Type-member convergence

## 46. Current type-member domain model is a bounded predecessor

The current semantic type-member subsystem has:

```rust
TypeMemberDomain::Kind(KindId)
TypeMemberDomain::ConstInt
TypeMemberDomain::Type(TypeId)
```

and:

```rust
TypeMemberArgument::Type(TypeId)
TypeMemberArgument::Const(ConstTerm)
TypeMemberArgument::RecordRow(RecordRowId)
```

This was an intentional C1 seam, not the final C3 model.

## 47. P2 must converge type-member domains with ordinary static domains

Compile-time Int must have one canonical semantic identity. P2 must not leave two independent representations such as:

```text
ConstTerm::Int(3)
TypeData::ConstInt(3)
```

that can diverge.

The preferred semantic endpoint is:

```text
parameter/result classifier
    KindId

argument/result term
    StaticArgument / TypeId(non-row)
```

Compatibility wrappers may remain internally while consumers migrate.

## 48. Type-member source syntax must use the same domain resolver

Current syntax separates `Kind(KindSyntax)` from `ConstType(TypeAnnotation)`. P2 must allow user domains such as:

```phalcom
type Initial: State
type Transform<S: Shape>: Shape
```

using the same static-domain resolution machinery as ordinary generic binders. Do not create a second “type-member static domain” namespace/resolver.

---

# Part XIII — Associated `type data`

## 49. Class AST currently has no associated type-data member

`ClassMember` currently includes ordinary behavior/field/delegation/index/type-member categories. When the parser sees `type` in a class body it routes to `parse_type_member_decl`.

Thus this accepted source cannot yet be represented:

```phalcom
class Parser {
    type data State {
        Start
        Done
    }
}
```

### Required AST category

Add a declaration-owned associated algebraic/static-domain member, conceptually:

```rust
ClassMember::TypeData(AssociatedTypeDataDef)
```

or an equivalent generalized associated-declaration category.

It must reuse the existing `AlgebraicDef`/case-shape machinery rather than implement a second type-data grammar.

## 50. Runtime-data owner grammar is a real design seam

The accepted P2 requirements also require associated `type data` on runtime data roots.

The current algebraic body is representation-bearing only: a braced root is either record components or sum cases. Simply allowing `type data` within those lists would break the clean product/sum grammar and blur representation with associated declarations.

### Requirement

Keep:

```text
representation-bearing algebra
    separate from
associated declarations
```

If the effective specification does not fully establish the exact source grammar for runtime-data-owned associated declarations, the later P2 implementation plan must mark that as a STOP/CONSULT item rather than letting the executor invent syntax.

A likely AST shape is an orthogonal associated-declarations collection on `AlgebraicDef`, but source spelling must follow the accepted specification.

## 51. Associated static-domain declaration identity

P2 needs a durable declaration identity conceptually equivalent to:

```rust
pub struct AssociatedStaticDomainDeclarationId {
    pub owner: DeclarationId,
    pub name: Box<str>,
}
```

or an equivalent stable owner/member identity.

For:

```phalcom
class Container<T: Type> {
    type data State {
        Empty
        Present(_ value: T)
    }
}
```

there is one declaration identity:

```text
Container::State
```

Concrete domain specializations are distinct:

```text
Container<Int>::State
Container<String>::State
```

## 52. Concrete associated-domain identity

A concrete associated domain needs:

```text
associated declaration identity
+
canonical owner environment
```

Conceptually:

```rust
StaticDomainData::Associated {
    declaration: AssociatedStaticDomainDeclarationId,
    owner_environment: StaticInstantiation,
}
```

The owner environment must be structural heterogeneous semantic data, not a rendered string such as `"Container<Int>"`.

## 53. Static constructor identity inside associated domains

The constructor/case declaration belongs to the associated domain declaration. The concrete term uses the concrete specialized domain.

Conceptually:

```text
declaration:
    Container::State::Present

concrete term:
    domain = Container<Int>::State
    constructor = Container::State::Present
    arguments = [...] 
```

Declarations must not be cloned per owner specialization.

---

# Part XIV — Unified `::` namespace

## 54. `AssociatedSurface` remains the authority

Current:

```rust
AssociatedFamilyKind::{Variant, DataConstructor}
```

must be generalized, conceptually:

```rust
enum AssociatedFamilyKind {
    Case,
    DataConstructor,
    StaticDomain,
    StaticMember,
    ...
}

enum AssociatedMemberId {
    Case(...),
    DataConstructor(...),
    DataComponent(...),
    StaticDomain(AssociatedStaticDomainDeclarationId),
    StaticMember(TypeMemberDeclarationId),
}
```

Exact factoring can differ. The invariant is one owner/base collision and lookup authority.

Do not build a separate `AssociatedTypeDataTable` with independent name rules.

## 55. Associated resolution needs static result categories

Current `AssociatedResolutionKind` is primarily runtime-value/call/family oriented:

```text
ExactValue
ExactCallable
Family
StaticInvoke
DynamicInvoke
```

P2 needs outcomes that carry no runtime invocation target, including conceptually:

```text
AssociatedStaticDomain
AssociatedStaticTerm
TypeMemberProjection
```

`Parser::State` should denote a static domain. `Parser::State::Done` should denote a term of that domain. Neither is a runtime message/call.

## 56. Owner specialization must become heterogeneous

Current:

```rust
AssociatedOwnerResolution {
    owner_form: TypeId,
    lookup_owner: DeclarationId,
    supplied_arguments: Vec<TypeId>,
    residual_kind: KindId,
}
```

cannot represent an owner such as:

```phalcom
class C<T: Type, N: Int, S: Shape> { ... }
```

The supplied owner environment must become `StaticArgument`/generalized `GenericInstantiation` data.

## 57. Static-domain denotation is distinct from a term

A useful semantic distinction is:

```text
Parser::State
    static-domain declaration/specialization

Parser::State::Done
    static term
```

Do not encode `Parser::State` as a fake proper Type simply to reuse `SemanticDenotation::TypeForm`.

The denotation system should gain a static-domain category (or an equivalent explicit associated-resolution result) so chained `::` lookup remains category-correct.

---

# Part XV — Trait boundary

## 58. Do not accidentally migrate trait coherence

C3 explicitly permits P2 to reject arbitrary user static-domain trait parameters unless the trait evidence/coherence stack is deliberately migrated.

Current trait structures commonly assume TypeId arguments with runtime/proper-type semantics. Storage compatibility is not semantic permission.

### Required behavior

If a trait declaration/application attempts a user type-data domain while trait support remains out of scope, emit a targeted diagnostic.

Do not:

```text
store static-data terms in trait arguments and hope existing evidence lookup works
hash them under ordinary type semantics
compare them through subtyping
let evidence matching silently fail
```

Current generic binder site classification does not make the trait boundary particularly explicit. P2 should add a dedicated trait binder site or an equally mechanical caller-side validation so the unsupported boundary cannot be bypassed accidentally.

---

# Part XVI — Stable metadata and imports

## 59. Stable metadata must become a static-term graph

Current stable nodes are type-centric but can retain their names. Their semantics must expand.

Required additions include conceptually:

```rust
KindNode::StaticDomain(StaticDomainRef)

enum StaticArgumentRef {
    Term(TypeNodeId),
    RecordRow(RecordRowRef),
}

TypeNode::ConstInt(i64)
TypeNode::StaticData { ... }

TypeNode::Applied {
    origin: TypeNodeId,
    arguments: Box<[StaticArgumentRef]>,
}
```

Scoped equivalents are also required.

## 60. Stable identity rules

Never serialize:

```text
raw StaticDomainId
raw TypeId
raw InferVarId
rendered owner environment string
source spelling as identity
```

Use stable declaration/module references and structural environments.

Associated static domains need a stable declaration reference plus owner static arguments.

## 61. Schema migration

Current baseline:

```text
TYPE_METADATA_SCHEMA_VERSION = 4
```

P2's structural changes require an explicit monotonic schema transition.

Compatibility rules should include:

```text
old Type-only Applied arguments
    decode as StaticArgumentRef::Term

new heterogeneous Applied
    rejected by readers without the new schema

old metadata
    must never map unknown static domains to Type silently
```

Missing static-domain information must be an incompatibility/validation issue, not a lossy fallback.

## 62. Cross-module static domain example

Provider:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}
```

Consumer:

```phalcom
import geometry { Shape }

data Tensor<S: Shape>()
```

The consumer must receive the provider's stable domain identity, not reconstruct `Shape` by spelling.

---

# Part XVII — Incremental analysis and source index

## 63. Required invalidation matrix

P2 must prove cold/incremental parity for edits to:

```text
type-data declaration identity
case add/remove/rename
static payload domain
generic binder domain
static generic argument
static alias result domain/body
associated type-data declaration
associated case
owner generic signature
associated owner specialization
associated namespace collision
imported provider static domain/term
```

Final semantic state after incremental recomputation must equal a cold build.

## 64. Source index taxonomy must stop treating Enum/Data as canonical C3 meaning

Current editor taxonomy still contains:

```rust
SourceDeclarationKind::Enum
SourceDeclarationKind::Data
```

P2 requires indexing static-domain declarations and associated paths. Source identity should reflect algebraic declaration plus phase/shape, not resurrect obsolete source categories.

P3 owns full LSP polish. P2 owns enough source identity to support dependency tracking, definition targeting, and later tooling without string reconstruction.

## 65. Legacy enum/data fingerprint rule

Source-input fingerprints may legitimately change when source spelling changes; they are allowed to trigger recomputation.

The semantic/interface product after recomputation must not acquire a new declaration/case identity solely because:

```text
enum X { A }
    ->
data X { A }
```

P1 currently has only partial semantic-equivalence tests, so P1/P2 incremental tests should explicitly distinguish input invalidation from semantic product identity.

---

# Part XVIII — Diagnostics

## 66. Domain diagnostics

P2 should distinguish:

```text
unknown static domain
runtime data used as static domain
unsupported trait static-domain binder
static argument domain mismatch
pack binder mode/domain mismatch
variance unsupported for non-Type static domain
```

## 67. Static term diagnostics

Required:

```text
invalid static constructor
wrong arity
wrong labels
wrong argument static domain
runtime expression in static argument
inaccessible static case
runtime use of TypeOnly term
```

## 68. Associated diagnostics

Required:

```text
associated static domain missing
associated base collision
unspecialized associated domain used as concrete domain
invalid owner specialization
static member/domain category mismatch
```

## 69. Diagnostic presentation quality

Current `KindData::StaticDomain(_)` formatting collapses to a generic `StaticDomain` label. P2 should retain enough domain identity in diagnostics to say, for example:

```text
expected Shape, got Endian
```

rather than:

```text
expected StaticDomain, got StaticDomain
```

Presentation may resolve names from canonical declaration identity; it must not become semantic identity itself.

---

# Part XIX — Performance and resource requirements

## 70. Static domain and term interning

Expected complexity:

```text
static domain lookup/intern
    amortized O(1)

static constructor lookup/intern
    amortized O(1)

static term intern
    O(immediate argument count) hashing
```

Do not scan all type-data declarations to form one term.

## 71. Avoid repeated namespace resolution

After source elaboration, `Shape` should be a canonical `StaticDomainId`/`KindId`. Inference, substitution, metadata export, fingerprinting, and associated specialization should not repeat module/name lookup.

## 72. First-order inference only

Unification cost should remain proportional to term structure:

```text
same constructor -> recurse pairwise
different constructor/domain -> fail immediately
```

No constructor search, arbitrary evaluation, theorem proving, or backtracking is required.

## 73. Intern associated-domain specializations

Repeated formation of:

```text
Container<Int>::State
```

should return the same canonical static-domain identity. The interner key should be declaration identity plus canonical owner environment.

## 74. Formatting remains presentation-only

Do not use `format_type`/rendered strings inside identity, metadata, specialization, or lookup. Formatting cost is acceptable for diagnostics; stringification/reparse is not an implementation strategy.

---

# Part XX — Soundness boundaries

## 75. No runtime-dependent static identity

P2 must preserve the prohibition against static identity depending on:

```text
I/O
wall-clock time
randomness
mutable globals
object address/identity
resource handles
ordinary closures
runtime method dispatch
runtime equality/hash
```

## 76. ProperType audit is mandatory

Changing `Applied.arguments` to heterogeneous static terms creates many new opportunities to confuse “stored in TypeStore” with “runtime type.”

Every API producing or consuming runtime value-type knowledge should use `ProperTypeId` or equivalent validation, especially:

```text
TypeKnowledge for runtime expressions
field types
callable runtime parameter/return types
subtyping operands
class hierarchy edges
runtime layout type facts
```

The P2 plan should make this an explicit audit task, not rely on compile errors to reveal it.

---

# Part XXI — Recommended implementation decomposition

## 77. Dependency order

After corrective P1 closure:

```text
P1 truthful static-domain foundation
    ↓
P2-A — domain syntax/resolution + heterogeneous application
    ↓
P2-B — scoped forms + instantiation + checked substitution
    ↓
P2-C — first-order static inference
    ↓
P2-D — static aliases + type-member convergence
    ↓
P2-E — associated type data + unified ::
    ↓
P2-F — metadata/imports/incremental/source index
    ↓
focused P2 stabilization
```

The later patch-grade plan may use different task numbering, but this dependency order should be preserved.

## 78. Why application comes before inference

Inference cannot be correctly designed while canonical application is still `Box<[TypeId]>`. Establish `StaticArgument`, scoped application, and checked materialization first; then make the solver mirror those canonical structures.

## 79. Why associated type data comes after static application

Generic-owner associated-domain identity depends on a canonical heterogeneous owner environment. Implementing associated domains first would either force string identity or duplicate specialization logic.

## 80. Why metadata/incremental closure is part of P2 acceptance

P2's semantics are not complete if they only work within one source module. The feature's identity contract explicitly includes imports, metadata, and cold/incremental parity. These are not documentation cleanup tasks.

---

# Part XXII — Test requirements

## 81. P1 corrective gate tests

Before P2:

```text
type-data declaration kind is StaticDomain
type-data declaration is not ProperType
type-data declaration has no runtime class-object identity
static singleton/payload term canonicalizes
singleton vs explicit-empty static constructor remains distinct
runtime use of TypeOnly declaration/term rejects
TypeOnly case has no runtime invocation target
metadata does not encode StaticDomain as Type
all TypeData canonical transforms handle ConstInt/StaticData
```

## 82. P2 binder/application syntax matrix

Positive:

```phalcom
data Tensor<T: Type, S: Shape>()
data Integer<Bits: Int, Order: Endian>()

type A: Type = Tensor<Float, Shape::Vector(3)>
type B: Type = Integer<32, Endian::Big>
```

Negative:

```text
runtime data used as binder domain
wrong static argument domain
runtime expression in static argument
row/static argument category mismatch
unsupported user-static-domain trait binder
```

Existing C2 type/pack application cases must remain green.

## 83. Canonical identity matrix

Prove:

```text
Shape::Vector(3) == repeated formation
Shape::Vector(3) != Shape::Vector(4)
Shape::Vector(3) != OtherShape::Vector(3)

Container<Int>::State
    shares associated declaration identity with
Container<String>::State

but concrete domains differ by owner environment
```

## 84. Inference matrix

Required:

```text
same static constructor solves payload variable
different constructor rejects
different nominal domain rejects
nested static constructor unifies recursively
static term inside Applied participates in generic inference
underconstrained static variable remains underconstrained
wrong-domain candidate reports domain mismatch
```

## 85. Substitution/materialization matrix

Required:

```text
static parameter -> concrete static term
nested StaticData substitution
heterogeneous Applied substitution
RecordRow argument stays row-owned
C2 pack duplicate exposed by substitution still errors
wrong-domain substitution errors
associated owner substitution re-specializes canonically
```

## 86. Alias/type-member matrix

Required:

```phalcom
type DefaultShape: Shape = Shape::Scalar
type P: TypePack = T::Positionals
```

and:

```text
unannotated alias defaults to Type
wrong alias result domain rejects
type member returning user domain works
ConstInt member uses canonical Int static term
```

## 87. Associated type-data matrix

Required class example:

```phalcom
class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

Verify:

```text
Parser::State
Parser::State::Start
Parser::Initial
```

Generic owner example:

```phalcom
class Container<T: Type> {
    type data State {
        Empty
        Present(_ value: T)
    }
}
```

Verify declaration-family identity and concrete specialization identity.

Also test namespace collisions among runtime cases/constructors, type members, associated static domains, associated static terms, and behavioral categories according to accepted collision rules.

## 88. Cross-module and incremental matrix

Provider/consumer tests should cover:

```text
exported type data
generic binder over imported static domain
imported static term in generic application
provider case edit
provider payload edit
provider domain rename
associated domain edit
metadata round trip
incremental final state == cold final state
```

## 89. Runtime-negative tests

Prove there is still:

```text
no TypeDataObject
no static constructor bytecode
no static term comparison bytecode
no runtime static solver
```

and that TypeOnly inhabitants cannot materialize as ordinary runtime bindings.

---

# Part XXIII — Expected source impact map

## 90. Parser/AST

Expected:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/error.rs
phalcom-ast/tests/**
```

Owns:

```text
static-domain references
static generic arguments
static alias result domains
associated type-data declaration syntax
```

## 91. Semantic type/store layer

Expected:

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/variance.rs
```

Not every file should be generalized indiscriminately; proper-Type-only consumers should stay narrow.

## 92. Checker/inference

Expected:

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/composition.rs
phalcom-semantic/src/checker/row_inference.rs
```

## 93. Declaration and associated authority

Expected:

```text
phalcom-semantic/src/algebraic_semantics.rs
phalcom-semantic/src/checker/algebraic_declaration.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/type_alias.rs
phalcom-semantic/src/associated.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/db/**
```

## 94. Module/import/incremental

Expected:

```text
phalcom-modules/src/declaration.rs
phalcom-modules/src/interface.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/**
```

## 95. Metadata

Expected:

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/declaration.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/tests/schema_compat.rs
phalcom-semantic/src/metadata/export.rs
```

## 96. Compiler/runtime

P2 should need little or no new VM architecture. Static normalization remains semantic/compiler-host work.

A requirement for new static runtime objects, opcodes, or VM inference is a STOP/CONSULT signal.

---

# Part XXIV — Risk register

## 97. Highest-risk P2 failures

| Risk | Failure mode | Prevention |
|---|---|---|
| Proper type collapse | static term accepted as runtime type | preserve `ProperTypeId`/kind checks |
| Duplicate domain authority | second generic-domain registry | one `KindId`/`StaticDomainId` architecture |
| Row collapse | RecordRow encoded as fake TypeId | `StaticArgument::RecordRow` |
| Partial Applied migration | hidden consumers still assume TypeId args | classified repository-wide audit |
| Inference domain leak | Shape var solved to Type/Int/other domain | exact KindId validation |
| Runtime equality leak | user `==`/hash defines static identity | compiler structural identity only |
| Associated string identity | owner specialization formatted to text | declaration + canonical environment |
| Trait accidental widening | evidence/coherence silently receives static terms | explicit unsupported boundary |
| Metadata collapse | static domain exported as Type | explicit stable static-domain nodes |
| Scoped loss | lambda drops row/static category | `ScopedStaticArgument` |
| Stale substitution | invalid rewrite returns old term | checked failure, no `unwrap_or(ty)` |
| Constructor collision | static case identity name+arity | selector-aware identity |
| Phase loss | module shell forgets TypeOnly | orthogonal phase metadata |
| Static/runtime associated mix | static case gets runtime target | static resolution category |
| Arena-copy regression | clone TypeLambdaArena per application | move-out/restore or borrow-safe API |
| P1/P2 scope confusion | P2 normalizes around broken P1 | hard corrective predecessor gate |

---

# Part XXV — P2 non-goals

## 98. Preserve the accepted boundary

P2 does not require:

```text
trait requirement binding a whole type-data domain
GAT redesign
theorem proving
arbitrary compile-time functions
runtime methods executed during static normalization
general static pattern matching
automatic lifting of runtime values into static terms
pack decomposition/backtracking
Universe structural-domain final exposure
corpus-wide enum removal
full LSP polish
VM static solver
runtime type-data objects
new Tuple/Record runtime representation
```

P3 owns Universe/corpus/tooling closure.

---

# Part XXVI — Final P2 acceptance gate

## 99. Canonical gate semantics

P2 is not accepted until the canonical program has complete source-to-persistence semantics:

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

Required facts:

```text
Shape
    nominal static domain, not Type

Shape::Vector(3)
    canonical static term

Tensor<Float, Shape::Vector(3)>
    canonical proper Type application carrying heterogeneous static arguments

V
    proper-Type alias

Parser::State
    associated static-domain declaration/specialization

Parser::State::Done
    static term, not runtime value

Parser::Initial
    static term in domain Parser::State
```

## 100. Gate dimensions

The final P2 gate must prove:

1. parser/AST support;
2. nominal static-domain resolution;
3. heterogeneous application;
4. ProperType boundary;
5. checked substitution/materialization;
6. scoped/type-lambda preservation;
7. first-order inference;
8. static aliases;
9. type-member convergence;
10. associated type-data identity;
11. unified namespace collision behavior;
12. trait boundary;
13. stable metadata round trip;
14. imported static-domain identity;
15. cold/incremental parity;
16. source-index identity;
17. C1/C2 pack/row regression;
18. zero runtime static solver.

---

# Part XXVII — STOP / CONSULT triggers for the later P2 plan

## 101. Required escalation conditions

The implementation plan should require STOP AND CONSULT if:

1. P1 cannot be made truthful without abandoning the accepted `TypeId`/`ProperTypeId` architecture;
2. `StaticArgument` cannot support a required consumer without collapsing RecordRow;
3. user static-domain binders appear to require a second classifier registry;
4. heterogeneous application requires weakening proper runtime type checks;
5. associated-domain identity appears to require rendered strings/source text;
6. trait coherence/evidence would require material redesign;
7. runtime-data-owned associated type-data grammar is not established by the effective specification;
8. TypeOnly terms appear to require VM/runtime objects;
9. inference appears to require search/backtracking instead of first-order structural unification;
10. metadata compatibility requires a material unplanned schema-policy decision;
11. the same nontrivial semantic failure persists after one serious correction;
12. two plausible fixes have materially different ownership/identity consequences;
13. passing tests requires weakening Runtime/TypeOnly or ProperType/static-term boundaries.

---

# Part XXVIII — Repository hygiene observation

## 102. Large `docs/implementation/plan.py` change

The P1 commit also includes a very large change under:

```text
docs/implementation/plan.py
```

It may be intentional repository tooling, but it is not obviously part of the semantic implementation. Before further feature work, verify whether it is canonical tooling or a one-off generated planning artifact. Do not delete it without checking history/ownership; simply avoid carrying unexplained generated-plan machinery as semantic feature scope.

---

# Part XXIX — Final requirements verdict

## 103. What P2 should inherit after P1 corrective closure

```text
one AlgebraicDef AST
phase/shape orthogonality
legacy enum normalization
record-payload case syntax
runtime ADT/GADT reuse
VariantId runtime case identity
specialized runtime lowering
StaticDomainId identity family
KindData::StaticDomain
StaticArgument envelope
TypeId non-row static-term carrier
ProperTypeId boundary
```

## 104. What P2 must not inherit as accepted architecture

The following are incomplete/incorrect current artifacts, not established invariants:

```text
TypeOnly declaration => KindId::TYPE
TypeOnly declaration => class_object_type
TypeOnly sum => runtime ExactCase as its static semantic identity
StaticDomain metadata => KindNode::Type
StaticDomainData without an interner
static constructor identity = name + arity
module DeclarationKind ignoring phase
TypeMemberDomain::ConstInt as permanent separate domain universe
GenericArgumentSyntax permanently limited to TypeAnnotation
Applied arguments permanently limited to TypeId
associated owner environment permanently limited to Vec<TypeId>
```

## 105. Recommended immediate next action

Before authoring/executing the P2 patch-grade plan:

```text
1. Finish/repair P1 under P1's acceptance objective.
2. Prove the semantic crate builds.
3. Add focused TypeOnly/static-term semantic regressions.
4. Make TypeOnly declaration identity genuinely static.
5. Implement static-domain/static-constructor interning.
6. Remove StaticDomain -> Type metadata collapse.
7. Correct module/declaration phase retention.
8. Update C3 checkpoint/PROGRAM/STATUS truth.
9. Produce P1 walkthrough/handoff only after focused verification.
10. Then plan P2 against that exact revision.
```

## 106. P2 implementation thesis

Once the corrective predecessor is complete, P2 should be implemented around one invariant:

> **Every supported compile-time argument has one canonical static-domain classifier and one canonical semantic term representation; generic application, substitution, inference, aliases, type members, associated lookup, metadata, imports, and incremental analysis all consume that same representation.**

That produces the intended Phalcom model:

```text
T: Type
N: Int
S: Shape
P: TypePack
L: LabeledTypePack
E: TupleEntries
R: RecordRow
```

as one coherent static-parameter system while retaining the specialized representations that are already architecturally useful:

```text
TypeId
RecordRowId
KindId
InferVarId
row inference
```

The challenge in P2 is not to invent a broader meta-language. It is to finish the static-domain unification that C1, C2, and the sound portions of P1 have already made possible without weakening the distinctions that keep the language and compiler sound.
