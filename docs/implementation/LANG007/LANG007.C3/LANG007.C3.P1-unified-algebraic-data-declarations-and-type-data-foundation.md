---
id: LANG007.C3.P1
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG007.C2
follows: LANG007.C2.P3
supersedes: null
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 62a6082ab463387c812d558e5561508267ecc63c
prepared: 2026-09-22
title: "Unified Algebraic Data Declarations and Type-Data Foundation"
---

# LANG007.C3.P1 — Unified Algebraic Data Declarations and Type-Data Foundation

## 0. Executor contract

This is a Luna-oriented patch-grade implementation plan. It converts the accepted
`LANG007.C3` specification and requirements analysis into one bounded executable
work package.

The implementer may adapt private names, local helper decomposition, imports, and
other mechanical details to the live repository. The implementer must not silently
change the architecture, identity rules, semantic ownership, runtime/static
boundary, or accepted language behavior described here.

The operating rules are:

- verify live paths and signatures before editing;
- preserve unrelated modified, staged, and untracked work;
- keep one canonical semantic authority for each fact;
- preserve the mature runtime ADT/GADT/exact-case implementation rather than
  reimplementing it;
- keep `ProperTypeId` as the proof boundary for actual proper types;
- keep `RecordRow` in its accepted dedicated canonical/solver domain;
- use sparse, discriminating tests during BUILD mode;
- classify and defer unrelated/baseline failures;
- STOP AND CONSULT when an architectural trigger in this plan fires;
- keep the shared `LANG007.C3` checkpoint record current;
- produce the required P1 walkthrough and P1→P2 handoff.

Testing is evidence gathering, not a reflex. Do not run broad workspace tests merely
because an edit occurred.

---

## 1. Goal

Implement one canonical algebraic declaration path for Phalcom runtime `data`
products and sums, preserve all existing runtime ADT/GADT/exact-case behavior under
that path, and establish top-level `type data` nominal static domains with canonical
compiler-owned static terms and no runtime inhabitants.

---

## 2. Checkpoint acceptance objective

`LANG007.C3` ultimately owns the unified runtime/static algebraic declaration model:

```text
data
    runtime algebraic domain

type data
    type-system-only algebraic/static domain
```

P1 is the first of three plans serving that checkpoint. It is not checkpoint closure.

P1 must establish the foundation on which P2 can generalize heterogeneous generic
application, inference, static aliases/type members, and associated `type data`.
P3 will later expose intrinsic structural domains through the Universe, migrate the
canonical corpus/tooling surface, remove the legacy `enum` source form according to
the final compatibility policy, and certify the checkpoint.

P1 is successful when all of the following are true:

1. runtime products and runtime sums are represented by one canonical algebraic
   declaration model before lowering;
2. canonical runtime source uses `data` for both products and closed sums;
3. legacy `enum`, while temporarily accepted, lowers to the exact same semantic
   declaration/case identity as equivalent `data`;
4. existing exact-case typing, GADT equations, variant/case families, root
   requirements/defaults, exact-case impl behavior, conformance, pattern
   refinement/exhaustiveness, and runtime lowering remain intact;
5. top-level `type data` declarations have nominal static-domain and constructor/case
   identity;
6. type-data terms are canonical structural compiler terms and cannot materialize as
   ordinary runtime values;
7. `ProperTypeId` still rejects all non-`Type` static terms;
8. `intrinsic data` and `intrinsic type data` are represented as provenance plus
   phase, not as parallel declaration categories;
9. stable metadata/fingerprints can preserve P1 algebraic/static-domain identity
   without serializing source spellings or store-local numeric IDs;
10. no general static evaluator, runtime type-data object, or VM static-term
    execution path is introduced.

---

## 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   62a6082ab463387c812d558e5561508267ecc63c
subject:    feat(lang007): implement whole-pack inference, persistence, and close C2 (LANG007.C2.P3)
date:       2026-09-22
```

### 3.1 Verified live repository facts

The following facts were verified while preparing this plan.

#### Repository layout

The relevant top-level crates are:

```text
phalcom-ast/
phalcom-modules/
phalcom-semantic/
phalcom-type-meta/
phalcom-core/
phalcom-lsp/
docs/
```

Do not use stale `crates/...` prefixes from older planning material.

#### C2 baseline

`LANG007.C2` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

C2 established and P1 must preserve:

- `TypePack`, `LabeledTypePack`, and `CompleteTypePack`;
- flat positional/labeled concat;
- `TupleFromPacks` and `CallableFromPacks`;
- `GenericParameterMode`;
- binder-aligned generic applications;
- `GenericBinderHeader`;
- checked substitution/materialization;
- whole-pack `InferVarId(KindId)` inference;
- metadata schema v4;
- cold/incremental parity for the C2 substrate;
- zero runtime/VM pack solving.

#### Current AST split

`phalcom-ast/src/ast.rs` still contains:

```rust
Statement::Enum(EnumDef)
Statement::Data(DataDef)
```

`EnumDef` and `DataDef` remain distinct syntax models.

#### Current static term/kind substrate

`phalcom-semantic/src/types/kind.rs` currently contains:

```rust
KindData::Type
KindData::RecordRow
KindData::TypePack
KindData::LabeledTypePack
KindData::CompleteTypePack
KindData::Arrow { ... }
```

There is no nominal static-domain form yet.

`phalcom-semantic/src/types/id.rs` already documents `TypeId` as a canonical
store/snapshot-local identifier for type-level forms. `ProperTypeId` remains a
validated wrapper requiring `KindId::TYPE`.

`phalcom-semantic/src/types/store.rs` currently contains canonical non-proper pack
terms in `TypeData`, while:

```rust
TypeData::Applied {
    origin: TypeId,
    arguments: Box<[TypeId]>,
}
```

is still type-only at the argument-storage boundary. P1 must not generalize
`Applied`; that is P2 work.

#### Current heterogeneous bounded seam

`phalcom-semantic/src/types/type_member.rs` already has:

```rust
TypeMemberDomain::{Kind(KindId), ConstInt, Type(TypeId)}
TypeMemberArgument::{Type(TypeId), Const(ConstTerm), RecordRow(RecordRowId)}
ConstTerm::Int(i64)
```

P1 may bridge this bounded representation to the new canonical static-term
foundation, but must not perform P2's language-wide type-member/domain convergence.

#### Current declaration semantics

Runtime products are owned by:

```text
phalcom-semantic/src/data_semantics.rs
DataInfo
DataShape::{Tuple, Record}
DataSemanticTable
DataDeclarationProduct
```

Runtime sums/GADTs are owned by:

```text
phalcom-semantic/src/enum_semantics.rs
EnumInfo
VariantInfo
VariantShape
CaseTypeEnvironment
EnumSemanticTable
EnumDeclarationProduct
```

The enum path is mature and must be reused.

#### Current runtime lowering

`phalcom-core/src/compiler/lib/data_decl.rs` lowers runtime products using the data
runtime path.

`phalcom-core/src/compiler/lib/enum_decl.rs` lowers runtime sums using the mature
enum/variant runtime path, including exact-case behavior installation and specialized
Option representation.

P1 does **not** require these two physical runtime representations or bytecode paths
to become one. Source/semantic unification happens above lowering.

#### Cross-cutting current AST consumers

A live search found `Statement::Enum` / `Statement::Data` consumers in, among other
places:

```text
phalcom-modules/src/interface.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-core/src/native/source.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-lsp/src/semantic_tokens.rs
```

This is an expected migration surface. It is not permission to redesign all of those
subsystems.

#### Metadata baseline

`TYPE_METADATA_SCHEMA_VERSION` is currently 4.

P1 is the first C3 metadata transition. This plan deliberately does **not** preselect
the exact next schema number; the implementer must audit the live compatibility
contract before changing it.

#### Lifecycle drift

`docs/implementation/LANG007/` currently contains C1/C2 plus `PROGRAM.md` and
`STATUS.md`, but no C3 directory.

`PROGRAM.md` and `STATUS.md` contain stale future-facing C2 lifecycle text even though
C2 is complete. P1 must rebaseline active lifecycle state to C3 without rewriting
historical completed C1/C2 records merely to make old prose look current.

> Re-read every named live path before editing. Adapt mechanical drift locally.
> Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read these before substantive edits:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-walkthrough.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-handoff.md

the accepted LANG007.C3 requirements analysis supplied with this plan
the accepted LANG007.C3 type-data/unified-algebraic specification supplied with this plan

docs/specs/README.md
docs/specs/objects/data-objects.md
docs/specs/objects/enums.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/objects/impl.md
the effective pattern/GADT specifications named by docs/specs/README.md

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
```

Before each task, read only the exact source and tests named for that task plus directly
adjacent helpers required to understand the current call path.

Do not broadly rediscover C1/C2.

---

## 5. Normative authority

### 5.1 Accepted language behavior

The C3 specification fixes these semantics:

```text
data
    one canonical runtime algebraic declaration form

type data
    one type-system-only algebraic/static declaration form

phase
    Runtime | TypeOnly

shape
    Product | Sum

implementation provenance
    orthogonal to phase

ownership
    orthogonal to phase/shape/provenance
```

P1 must preserve these exact source distinctions:

```phalcom
data Token()
```

is a zero-component product with one value.

```phalcom
data Impossible {}
```

is an empty sum with no values.

Likewise:

```phalcom
data Example {
    Singleton
    Constructed()
}
```

contains two semantically distinct cases:

- `Singleton` is an associated singleton value;
- `Constructed()` is a zero-argument constructor family.

Braced bodies are syntactically classified, not resolved by semantic lookup:

```phalcom
data Person {
    name: String
    age: Int
}
```

is a record product, while:

```phalcom
data State {
    Ready
    Failed(_ error: Error)
}
```

is a sum. Mixing root fields and cases is a dedicated error.

Record-payload sum cases are valid:

```phalcom
data Message {
    Request {
        method: Method
        path: String
    }

    Closed
}
```

Runtime GADT result specialization keeps its existing semantics.

`type data` uses the same algebraic shape model but introduces no runtime inhabitants:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}
```

`Shape::Vector(3)` is a canonical static term, not a runtime object.

### 5.2 Implementation architecture accepted for C3

P1 inherits these planning decisions:

```text
TypeId
    canonical handle for non-row static terms

ProperTypeId
    proof that a TypeId is specifically a proper Type

KindData::StaticDomain(StaticDomainId)
    one dynamic classifier for nominal static domains

StaticDomainId
    store-relative identity for a static domain

TypeData
    extended with canonical compile-time constant/static-data terms

RecordRow
    remains in its dedicated canonical row domain
```

P1 may introduce the canonical heterogeneous static argument envelope required to
represent type-data constructor payloads:

```rust
enum StaticArgument {
    Term(TypeId),
    RecordRow(RecordRowId),
}
```

or an exact equivalent.

However, P1 must use it only where P1 needs it. Generalizing
`TypeData::Applied`, generic application, static inference, type lambdas, aliases,
and associated-domain specialization to that envelope is P2 scope.

### 5.3 Current implementation is not normative

Current `DataDef`/`EnumDef`, `DataInfo`/`EnumInfo`, and `compile_data`/`compile_enum`
splits describe the baseline implementation. They do not override the accepted C3
language model.

### 5.4 Future work is not P1 authority

Associated `type data`, language-wide heterogeneous generic application, static
aliases over arbitrary domains, general type-data inference, Universe structural
domain exposure, full LSP completion, and final `enum` removal belong to P2/P3.

---

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Current symbol/path | Owner | P1 invariant |
|---|---|---|---|
| Canonical type-level term handle | `TypeId` — `phalcom-semantic/src/types/id.rs` | TypeStore | remains canonical for non-row static terms |
| Proper-type proof | `ProperTypeId` | TypeStore/API boundaries | never weakened |
| Kinds | `KindId` / `KindData` | TypeStore | extended, not replaced |
| Record rows | `RecordRowId` / row arena | row subsystem | remains separate |
| Runtime products | `DataInfo`, `DataDeclarationProduct` | semantic data declaration analysis | becomes product projection of algebraic authority |
| Runtime sums | `EnumInfo`, `VariantInfo`, `EnumDeclarationProduct` | semantic ADT/GADT analysis | mature machinery reused |
| Case identity | `VariantId` | semantic identity | preserved; no second case ID |
| Exact-case term | `TypeData::ExactCase` | TypeStore / ADT semantics | preserved |
| Product lowering | `compile_data` / data lowering specs | compiler/runtime | may remain specialized |
| Sum lowering | `compile_enum` / enum lowering specs | compiler/runtime | may remain specialized |
| Type-member const Int | `ConstTerm::Int` | bounded type-member subsystem | compatibility facade; not a second language identity |
| Metadata schema | schema v4 | `phalcom-type-meta` | explicit monotonic C3 transition |
| C2 pack substrate | C2 types/metadata/inference | semantic/type-meta | frozen and reused |

### 6.2 Inherited baseline failures

The C2 checkpoint records two pre-existing trait-suite failures:

```text
BASELINE-001
semantic::capabilities::traits::trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Do not repair these during P1 unless evidence shows P1 causally changed them or they
block a required P1 acceptance gate.

---

## 7. Architecture

### 7.1 Authority flow

P1 must establish this direction of authority:

```text
canonical algebraic source AST
    ↓
one semantic algebraic declaration product
    ├── runtime product projection
    ├── runtime sum / exact-case / GADT projection
    └── type-only static-domain projection
            ↓
stable semantic facts
    ↓
lowering / metadata / source-index consumers
    ↓
runtime specialization only for Runtime declarations
```

The compiler/VM must never infer phase or algebraic shape from runtime object layout.

### 7.2 Canonical AST

The target conceptual syntax model is:

```rust
struct AlgebraicDef {
    name: String,
    name_range: SourceRange,
    phase: AlgebraicPhase,
    implementation: ImplementationKind,
    generic_parameters: Vec<GenericParameterSyntax>,
    where_clause: Option<WhereClauseSyntax>,
    shape: AlgebraicShapeSyntax,
    attributes: Vec<Attribute>,
    range: SourceRange,

    // compatibility/presentation only if retained:
    legacy_enum_keyword_range: Option<SourceRange>,
}

enum AlgebraicPhase {
    Runtime,
    TypeOnly,
}

enum AlgebraicShapeSyntax {
    Product(ProductShapeSyntax),
    Sum(SumShapeSyntax),
}

enum ProductShapeSyntax {
    Tuple { components: Vec<DataComponentSyntax>, ... },
    Record { components: Vec<DataComponentSyntax>, ... },
}

struct SumShapeSyntax {
    cases: Vec<AlgebraicCaseSyntax>,
    ...
}

enum CasePayloadSyntax {
    Singleton,
    Tuple {
        parameters: Vec<ParameterDef>,
        explicit_empty: bool,
    },
    Record {
        components: Vec<DataComponentSyntax>,
    },
}
```

Exact Rust names are mechanically flexible.

The required endpoint is semantically one statement category. Prefer:

```rust
Statement::Data(AlgebraicDef)
```

where `phase` differentiates `data` and `type data`.

A temporary compatibility AST alias or parser helper is acceptable during migration,
but the final P1 semantic consumer graph may not preserve:

```text
Statement::Enum -> one semantic world
Statement::Data -> another semantic world
```

Legacy `enum` must normalize immediately to `Runtime + Sum` and may retain only its
keyword source span for migration diagnostics/presentation.

### 7.3 One semantic declaration authority

Introduce one canonical semantic product/facade conceptually equivalent to:

```rust
struct AlgebraicDeclarationInfo {
    owner: DeclarationId,
    phase: AlgebraicPhase,
    implementation: ImplementationKind,
    generic_signature: Option<GenericSignature>,
    representation: AlgebraicRepresentation,
    source: Option<SemanticSourceSpan>,
}

enum AlgebraicRepresentation {
    Product(AlgebraicProductInfo),
    Sum(AlgebraicSumInfo),
}
```

The existing `DataInfo`, `EnumInfo`, `VariantInfo`, `DataSemanticTable`,
`EnumSemanticTable`, `DataDeclarationProduct`, and `EnumDeclarationProduct` may
survive as transitional projections/adapters.

They must be derived from the one canonical algebraic declaration analysis. They may
not independently re-read the AST and rebuild competing component/case facts.

### 7.4 Case identity

Do not mint an independent P1 case identity.

Preserve the existing declaration-owned `VariantId` identity. A neutral public/internal
alias/facade such as:

```rust
type AlgebraicCaseId = VariantId;
```

is acceptable.

New persistent/public terminology should converge toward `case`. Private internal
`Enum`/`Variant` names may survive where renaming is mechanical and does not expose a
second semantic category.

### 7.5 Runtime representation boundary

P1 unifies language semantics, not physical storage.

Allowed runtime implementation after P1 still includes:

```text
DataObject/product lowering
AdtCaseObject/enum lowering
native Option representation
singleton/immediate case representation
Tuple/Record specialized representation
other existing optimized forms
```

The existing `Bytecode::Data`/`FinalizeData` and `Bytecode::Enum`/`FinalizeEnum`
families may remain if they are still the correct runtime implementation detail.

The compiler should dispatch from canonical algebraic facts:

```text
Runtime + Product -> existing product lowering
Runtime + Sum     -> existing sum/ADT lowering
TypeOnly          -> no runtime declaration lowering
```

It must not dispatch from legacy source spelling.

### 7.6 Static-domain foundation

Add store-relative nominal static-domain identity:

```rust
struct StaticDomainId(u32);
```

and one open-ended kind form:

```rust
KindData::StaticDomain(StaticDomainId)
```

The store needs canonical domain interning/lookup. Conceptually:

```rust
enum StaticDomainData {
    ConstInt,
    Nominal {
        declaration: DeclarationId,
    },
}
```

The exact split between primitive and nominal domain registries is mechanically
flexible, provided there is one canonical semantic identity and stable metadata never
serializes the raw store-local integer.

Associated/specialized domain variants are P2.

### 7.7 Canonical static terms

Extend the canonical non-row static term graph with forms equivalent to:

```rust
TypeData::ConstInt(i64)

TypeData::StaticData(AlgebraicStaticTerm)
```

where:

```rust
struct AlgebraicStaticTerm {
    domain: StaticDomainId,
    constructor: AlgebraicStaticConstructorId,
    arguments: Box<[StaticArgument]>,
}
```

For P1:

- a top-level product-shaped `type data` has one primary static constructor identity;
- a top-level sum-shaped `type data` has one case constructor/singleton identity per
  declared case;
- canonical identity includes the nominal static domain, constructor/case identity,
  and canonical static arguments;
- runtime `==`, hashing, dispatch, object identity, and runtime reflection do not
  participate.

The `StaticArgument` envelope may be landed now as the shared representation needed
by static-data terms, but do not migrate general nominal `Applied` to it in P1.

### 7.8 Proper-type boundary

This is a hard soundness boundary:

```text
stored by TypeId
    does not imply
is a proper Type
```

`TypeStore::proper_type()` or its live equivalent must reject:

- `ConstInt` terms;
- `StaticData` terms;
- existing pack terms;
- every other non-`KindId::TYPE` term.

APIs publishing ordinary runtime value-type knowledge must continue to validate
`ProperTypeId` or equivalent proof.

### 7.9 Static payload validation

A `type data` payload component must belong to a supported static domain.

P1 must support enough static terms to establish the feature foundation, including
compile-time `Int` and existing TypeStore-owned static domains required by the
accepted P1 tests.

Do not treat arbitrary runtime `Expr` as a static constructor argument.

Do not authorize arbitrary runtime values merely because their runtime type has the
same spelling as a static domain.

### 7.10 Runtime-use rejection

A `type data` declaration has no runtime inhabitants.

The semantic checker must reject attempts to use the domain/term as an ordinary
runtime value type/value before runtime lowering.

The compiler must not emit:

```text
TypeDataObject
TypeDataCaseObject
static-constructor VM opcode
static-term comparison opcode
runtime static inference
```

### 7.11 Metadata/fingerprint foundation

P1 must publish stable structural records for at least:

```text
AlgebraicPhase
AlgebraicShape
runtime algebraic declaration identity
case identity and payload shape
top-level static-domain declaration identity
type-data static constructor/case identity
type-data term when exposed through a durable interface
```

Stable metadata must use declaration/module references and structural terms.

Never serialize:

```text
raw StaticDomainId
raw TypeId
raw InferVarId
source spelling as semantic identity
```

Fingerprint changes must include semantic changes to phase, shape, case set, payload,
singleton-vs-zero-arg distinction, GADT result specialization, static-domain identity,
and static constructor arguments.

Fingerprints must **not** change solely because canonical-equivalent source is written
with temporary legacy `enum` rather than `data`.

---

## 8. Ownership boundaries

### 8.1 Owns

P1 owns:

- canonical algebraic AST/source normalization;
- runtime product/sum shape discrimination;
- legacy `enum` compatibility normalization;
- authoritative algebraic semantic declaration facts;
- identity-preserving runtime sum migration;
- record-payload case surface required by C3;
- top-level `type data` declaration identity;
- nominal `StaticDomainId`/`KindData::StaticDomain` foundation;
- canonical static Int/type-data term representation needed by P1;
- static payload validity;
- runtime-use rejection for type-data;
- intrinsic provenance/phase legality;
- first C3 metadata/fingerprint foundation;
- minimum normative spec migration required to stop current specs contradicting
  shipped P1 syntax;
- lifecycle rebaseline for C3.

### 8.2 Does not own

P1 does not own:

- language-wide heterogeneous generic application;
- migration of `TypeData::Applied` to heterogeneous arguments;
- arbitrary static-domain inference;
- associated `type data`;
- associated-domain specialization;
- generalized static aliases/type members over user domains;
- trait generic migration;
- trait-owned type-data requirements;
- full LSP hover/completion/rename migration;
- Universe declarations for `TypePack`, `TupleEntries`, `RecordRow`, Tuple, Record,
  or Unit;
- corpus-wide `enum` removal;
- final lexer/token removal for `enum`;
- arbitrary compile-time functions/evaluation;
- general type-level pattern matching;
- runtime representation redesign for data/ADTs;
- VM static solving.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| runtime/type-only phase | algebraic semantic declaration product | checker, metadata, lowering | compiler inferring from syntax/VM representation |
| product/sum shape | algebraic semantic declaration product | checker, coverage, lowering | separate Data/Enum AST-derived truth |
| component/case set | algebraic semantic declaration product | exact-case, pattern, lowering, metadata | independent consumer reconstruction |
| case identity | existing declaration-owned `VariantId` identity | exact case, associated family, lowering | new P1 case-ID universe |
| static-domain identity | TypeStore static-domain registry | kind checking, static terms, metadata export | source name strings |
| static-term identity | TypeStore canonical term graph | checker, metadata, later P2 inference | runtime object identity or user equality |
| proper-type proof | `ProperTypeId` validation | runtime value-type APIs | treating every TypeId as Type |
| runtime representation | compiler/runtime lowering | VM | semantic layer encoding physical layout as language identity |
| stable identity | metadata declaration references + structural terms | imports/incremental | raw TypeId/StaticDomainId |
| checkpoint state | `LANG007.C3-CHECKPOINT.md` | subsequent implementers/planners | plan-specific competing state document |

---

## 9. Global invariants

These are fixed for P1.

```text
INV-01 — `data` is the sole canonical runtime algebraic declaration form.

INV-02 — `type data` is semantic phase TypeOnly and has no runtime inhabitants.

INV-03 — phase, algebraic shape, implementation provenance, and ownership are
         orthogonal facts.

INV-04 — product versus sum is determined by algebraic shape, not by a separate
         enum/data semantic category.

INV-05 — runtime sum case sets remain declaration-owned and closed.

INV-06 — existing exact-case identity remains canonical and nominal.

INV-07 — existing GADT result specialization, applicability, and branch evidence
         survive source/AST convergence.

INV-08 — singleton case and explicit zero-argument constructor remain distinct.

INV-09 — record-product versus sum-body classification is syntactic and deterministic;
         mixed root bodies are rejected explicitly.

INV-10 — one authoritative semantic algebraic product owns phase/shape/case/component
         facts; adapters may project it but not recompute it.

INV-11 — legacy `enum` and equivalent `data` produce the same declaration/case semantic
         identity and interface fingerprint.

INV-12 — source-level unification does not force runtime storage or bytecode unification.

INV-13 — `TypeId` remains the canonical non-row static-term handle.

INV-14 — `ProperTypeId` remains the proof that a canonical term is actually of domain
         `Type`.

INV-15 — user nominal static domains use one open-ended
         `KindData::StaticDomain(StaticDomainId)` class, not one Rust enum variant per
         declaration.

INV-16 — `RecordRow` remains in its dedicated canonical row representation.

INV-17 — type-data term equality is compiler structural identity; runtime equality,
         hashing, dispatch, and object identity never participate.

INV-18 — type-data terms never lower to runtime inhabitants or VM static operations.

INV-19 — existing C2 pack domains/terms/modes remain unchanged.

INV-20 — P1 does not generalize `TypeData::Applied`; heterogeneous nominal generic
         application belongs to P2.

INV-21 — stable metadata records declaration/domain identity structurally and never
         serializes raw store/query-local IDs.

INV-22 — changing only legacy `enum` spelling to canonical `data` does not create a new
         semantic identity or fingerprint.

INV-23 — compiler/runtime consume canonical semantic facts rather than reconstructing
         phase, shape, case identity, or GADT facts.

INV-24 — no second ADT/GADT implementation is introduced.

INV-25 — no general compile-time evaluator is introduced.
```

---

## 10. Non-goals

Do not expand P1 into any of the following:

```text
associated `type data`
general user-domain generic application
general type-data inference
trait static-domain migration
general static aliases
general static member-domain convergence
Universe structural-domain facade
final `enum` grammar removal
full LSP feature migration
corpus-wide migration
new runtime data representation
new ADT VM object model
static VM opcodes
static evaluator
pack decomposition/backtracking
unrelated C1/C2 redesign
unrelated trait failure repair
```

If an apparently required P1 implementation cannot be completed without one of these,
treat that as a scope/architecture escalation rather than quietly absorbing the work.

---

## 11. Expected impact map

### 11.1 Expected source areas

Parser/AST:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/token.rs             # only if compatibility/migration diagnostics require it
phalcom-ast/src/lexer.rs             # only if token handling requires it
phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/parser.rs
```

Module/declaration extraction:

```text
phalcom-modules/src/interface.rs
```

Semantic declaration authority:

```text
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/checker/data_declaration.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
```

Existing runtime-sum consumers that may require mechanical convergence:

```text
phalcom-semantic/src/impls.rs
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/checker/coverage/**
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/semantic_shard.rs
```

Static-domain/type store:

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_member.rs       # adapter only if needed
```

Metadata:

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/declaration.rs
phalcom-type-meta/src/validate.rs
phalcom-semantic/src/metadata/export.rs
```

Compiler/lowering:

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/data_decl.rs
phalcom-core/src/compiler/lib/enum_decl.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/native/source.rs
```

Tooling continuity only where AST convergence otherwise breaks existing consumers:

```text
phalcom-lsp/src/semantic_tokens.rs
```

P1 must not expand this into P3's full LSP feature work.

### 11.2 Expected tests

```text
phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/enum_syntax.rs

phalcom-semantic/tests/semantic/adts/data_declarations.rs
phalcom-semantic/tests/semantic/adts/declarations.rs
phalcom-semantic/tests/semantic/adts/constructors.rs
phalcom-semantic/tests/semantic/adts/variants.rs
phalcom-semantic/tests/semantic/adts/exact_cases.rs
phalcom-semantic/tests/semantic/adts/vertical_gadt.rs
phalcom-semantic/tests/semantic/adts/matching/**
phalcom-semantic/tests/semantic/foundations/kinds.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/incremental/adts.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs
phalcom-semantic/tests/semantic/integration/metadata.rs   # verify exact live path before use

phalcom-type-meta/tests/schema_compat.rs

phalcom-core/tests/core/**
phalcom-core/tests/native_adt_runtime.rs
```

Prefer extending existing responsibility-owned modules over creating a parallel C3-only
test hierarchy.

### 11.3 Expected documentation/state

Create/update:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md

docs/specs/objects/data-objects.md
docs/specs/objects/enums.md
docs/specs/type-level-domains.md

LANG007.C3.P1-walkthrough.md
LANG007.C3.P1-handoff.md
```

If the C3 requirements/guidance/checkpoint artifacts have already been persisted by
the time execution begins, preserve and update those canonical records instead of
creating duplicates.

### 11.4 Unexpected-touch rule

Touching adjacent helpers/tests is allowed when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem when doing so would
change identity, semantic authority, runtime representation, trait/conformance
architecture, inference architecture, or checkpoint scope.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. one semantic algebraic declaration authority;
2. `data` as canonical runtime product/sum source category;
3. `type data` as TypeOnly/no-runtime phase;
4. phase/shape/provenance/ownership orthogonality;
5. preservation of existing runtime ADT/GADT/exact-case semantics;
6. existing `VariantId` identity as the case-identity lineage;
7. `TypeId` as canonical non-row static-term handle;
8. `ProperTypeId` as actual-Type proof;
9. one dynamic nominal `StaticDomainId`-backed kind class;
10. dedicated RecordRow ownership;
11. structural compiler-defined static equality;
12. no VM/static-runtime execution for type-data;
13. no P1 generalization of nominal `TypeData::Applied`;
14. stable metadata identity by declaration/structure, not raw store IDs or strings;
15. legacy `enum` compatibility producing canonical-equivalent semantic identity;
16. C2 pack substrate remaining unchanged.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt without consultation:

- exact Rust names for algebraic AST/product structs;
- whether neutral case naming is an alias, wrapper, or gradual rename;
- private helper placement;
- private module decomposition;
- parser helper decomposition;
- compatibility adapter naming;
- exact metadata record factoring, provided identity semantics remain fixed;
- exact TypeStore arena layout for `StaticDomainId`;
- exact static constructor-ID wrapper type;
- local iterator/map/set choices;
- import cleanup;
- mechanical updates to AST pattern matches;
- test helper factoring.

### 12.3 VERIFY-FIRST

| Assumption | Verify in | If false |
|---|---|---|
| C3 directory is absent | `docs/implementation/LANG007/` | reuse existing C3 records rather than duplicating |
| schema baseline is v4 | `phalcom-type-meta/src/header.rs` | adapt monotonic transition mechanically; consult if compatibility model changed |
| `VariantId` remains declaration+selector identity | `phalcom-semantic/src/identity.rs` | STOP if identity semantics materially changed |
| exact-case type still uses variant identity + enum/root type | TypeStore / ADT checker | adapt naming only; STOP if model changed |
| current enum checker owns GADT case environment | `checker/enum_declaration.rs` | preserve live owner; STOP if authority moved |
| runtime sum lowering still consumes semantic lowering specs | `phalcom-core/src/modules/semantic_lowering.rs`, compiler | adapt paths; STOP if compiler reconstructs semantics by design |
| existing type-member `ConstInt` is bounded compatibility seam | `types/type_member.rs` | bridge locally; STOP if it is now canonical language-wide const identity |
| semantic test layout still follows current README | `phalcom-semantic/tests/semantic/README.md` | follow live convention mechanically |
| core focused target still includes `language::algebraic_data` | `phalcom-core/tests/README.md`, test list | resolve current exact target before G4 |

---

## 13. Global STOP / CONSULT triggers

A triggered consultation cannot be self-waived.

STOP AND CONSULT if any of the following occurs:

1. one canonical AST/semantic declaration path cannot represent current exact-case or
   GADT information without semantic loss;
2. compatibility `enum` and canonical `data` cannot share declaration/case identity;
3. preserving runtime behavior appears to require a second new ADT/GADT
   implementation rather than adapting the existing one;
4. product/sum convergence requires weakening existing case visibility, case-local
   generics, root requirements/defaults, exact-case impls, or conformance semantics;
5. `TypeId + KindData::StaticDomain` cannot represent P1 static terms without
   weakening `ProperTypeId`;
6. a proposed fix would treat a `StaticData` or `ConstInt` term as a proper runtime
   type;
7. supporting type-data payloads appears to require general P2 heterogeneous nominal
   application;
8. `RecordRow` would need to be collapsed into TypeId to make P1 work;
9. type-data implementation appears to require runtime objects, VM opcodes, runtime
   static lookup, or runtime static inference;
10. legacy `enum` spelling necessarily changes semantic fingerprints relative to
    equivalent `data`;
11. metadata would need raw `TypeId`, `StaticDomainId`, or rendered source strings as
    durable identity;
12. the same nontrivial semantic failure persists after one serious correction;
13. two plausible fixes have materially different ownership/identity architecture;
14. the test oracle conflicts with the accepted C3 specification;
15. P1 requires associated `type data`, trait static-domain migration, or broad
    heterogeneous generic application to pass;
16. an unexpected major subsystem must become a new semantic authority;
17. a workaround/special case would duplicate canonical declaration or static-term
    reasoning;
18. patch scope becomes materially more cross-cutting than this impact map;
19. passing tests appears to require weakening any `INV-*` invariant.

---

## 14. Debugging budget

### 14.1 Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminating command
```

Examples:

- missed AST match arm;
- import/rename fallout;
- borrow/type mismatch after neutralizing a struct name;
- test fixture still matching old AST;
- metadata serde exhaustiveness;
- stale helper path.

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

Never rerun an unchanged failing test unless relevant code/data/test setup changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariants | Required evidence |
|---|---|---|---|
| CV-01 | canonical syntax | INV-01–04 | runtime tuple/record products and sum cases parse through canonical algebraic AST |
| CV-02 | body discrimination | INV-08–09 | record product, sum, empty product, empty sum, mixed-body error |
| CV-03 | legacy compatibility identity | INV-10–11, INV-22 | equivalent `enum`/`data` yield equal semantic identity/fingerprint |
| CV-04 | runtime sum preservation | INV-05–07, INV-24 | existing constructors, singleton cases, exact cases and GADTs remain green |
| CV-05 | record-payload sum | INV-04–10 | record case fields preserve identity/labels and runtime behavior |
| CV-06 | semantic authority | INV-10, INV-23 | adapters derive from one algebraic declaration product |
| CV-07 | nominal static domain | INV-13–16 | two structurally identical `type data` declarations remain different domains |
| CV-08 | static term canonicalization | INV-13, INV-17 | repeated same constructor/args interns identically; different domain/case/arg differs |
| CV-09 | proper-type boundary | INV-14 | `proper_type` rejects static Int and type-data terms |
| CV-10 | static payload validity | INV-17–18 | supported static payload accepts; runtime-only payload rejects |
| CV-11 | runtime-negative boundary | INV-02, INV-18 | type-data cannot become runtime binding/value; no compiler/VM inhabitant path |
| CV-12 | intrinsic orthogonality | INV-03 | intrinsic runtime data and intrinsic type data preserve independent phase/provenance |
| CV-13 | metadata stability | INV-11, INV-21–22 | stable round-trip for phase/shape/domain/case/term identity |
| CV-14 | fingerprint sensitivity | INV-21–22 | semantic edits invalidate; enum→data spelling-only migration does not |
| CV-15 | cold/incremental parity | INV-21–23 | algebraic/static-domain edits converge with cold analysis |
| CV-16 | C2 regression | INV-19–20 | existing pack kinds/terms/modes stay green |
| CV-17 | runtime representation freedom | INV-12 | product and sum still use their established runtime lowering without semantic divergence |
| CV-18 | no static VM work | INV-18, INV-25 | no type-data opcode/object/solver is introduced |

---

## 16. Verification execution budget

### 16.1 Modes

#### BUILD MODE

Default while implementing. Use only exact/new regressions and the smallest owning
test target that answers the current question.

#### STABILIZE MODE

Enter at G1–G5. Run focused regressions and directly affected semantic/core suites.

#### CERTIFY MODE

Not part of ordinary P1 execution. Workspace/release certification belongs to a
later checkpoint/release gate unless separately requested.

### 16.2 Verification ladder

```text
T0 — exact new regression
T1 — directly affected feature tests
T2 — owning subsystem/module suite
T3 — adjacent cross-layer integration
T4 — broad crate/package verification
T5 — workspace/release verification
```

### 16.3 Mandatory during BUILD

- parser test(s) for the exact syntax being implemented;
- one semantic regression for the exact identity/term behavior being implemented;
- one compiler/core regression only when runtime lowering behavior changed.

### 16.4 Mandatory at named gates

- G1: parser/AST algebraic surface;
- G2: runtime algebraic semantic preservation;
- G3: type-data/static-domain foundation;
- G4: metadata/incremental/runtime-boundary integration;
- G5: final P1 focused stabilization.

### 16.5 Only if evidence demands

- full `phalcom-ast` crate tests;
- full semantic integration binary;
- full `phalcom-core --test core`;
- broader metadata consumers;
- LSP tests beyond compile/AST fallout.

### 16.6 Explicitly deferred / DO NOT RUN during BUILD

Do not run during routine BUILD work:

```text
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
full language corpus
full LSP suite
broad unrelated trait suites
broad performance suite
release certification
```

Reason: P1 has a large cross-layer migration surface; repeatedly running broad gates
would provide low-value duplicate evidence and increase the risk of chasing unrelated
baseline failures. P3/checkpoint certification owns broad closure.

Cargo validation commands should run serially.

---

## 17. Baseline / unrelated failure policy

Classify every newly observed failure:

```text
A — definitely caused by P1
B — probably caused by P1
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active implementation responsibility;
- C: one bounded classification pass; if nonblocking and still unclear, record and
  defer;
- D: record and continue immediately;
- do not consume adviser compute on D unless it blocks a required gate;
- never weaken or skip a semantic assertion to make a broad gate green;
- inherited `BASELINE-001` and `BASELINE-002` remain deferred unless causal evidence
  changes their classification.

Record deferred failures in `LANG007.C3-CHECKPOINT.md`.

---

# 18. Tasks

## T1 — Activate C3 and converge the algebraic parser/AST

### Purpose

Create/normalize the C3 lifecycle state and establish one canonical AST representation
for runtime `data`, temporary legacy `enum`, and top-level `type data`, including all
P1-required product/sum shapes.

### Preconditions

- C2 P3 is complete.
- Working tree status has been inspected.
- C3 specification and requirements are available.
- Existing parser/data/enum tests have been read.

### Consumes

- existing `DataDef` and `EnumDef` grammar;
- existing generic/where/modifier syntax;
- `ImplementationKind`;
- current variant payload/GADT syntax;
- lifecycle convention.

### Produces

- active `LANG007.C3` checkpoint record;
- current `PROGRAM.md`/`STATUS.md` state pointing to C3.P1;
- one canonical algebraic AST declaration structure;
- parser support for canonical `data` products/sums and `type data`;
- compatibility `enum` normalization to `Runtime + Sum`;
- deterministic body-shape discrimination;
- source ranges sufficient for migration and mixed-body diagnostics.

### Files and symbols

**Read:**

```text
AGENTS.md
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/token.rs
phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/parser.rs
```

**Modify/Create:**

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md

phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/token.rs            # only if required
phalcom-ast/src/lexer.rs            # only if required

phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/parser.rs
```

### Required implementation shape

1. At plan start, create or normalize the C3 checkpoint record and mark P1
   `IN_PROGRESS / PARTIAL / UNVERIFIED`.
2. Rebaseline program-level current state from stale C2 future-facing text to active
   C3.P1. Do not rewrite historical C1/C2 records.
3. Replace the AST's semantic split with one canonical algebraic declaration model.
4. Parse:
   - tuple product;
   - record product;
   - singleton sum;
   - comma-separated singleton shorthand;
   - tuple-payload case;
   - record-payload case;
   - explicit zero-arg constructor case;
   - case-local generic case;
   - GADT result specialization;
   - empty tuple product `()`;
   - empty braced sum `{}`;
   - `type data` equivalents;
   - `intrinsic data`;
   - `intrinsic type data`.
5. Preserve the distinction `Singleton` vs `Constructed()`.
6. Diagnose mixed record-component/sum-case root bodies explicitly.
7. Accept `enum` only as compatibility syntax and normalize it immediately to the
   same canonical AST semantic shape as `data ... { cases }`.
8. Any retained legacy keyword range must be presentation-only.

### Forbidden approaches

- keeping `Statement::Enum` as a second semantic category;
- choosing braced body meaning from the first element encountered;
- semantic name lookup to decide body shape;
- collapsing singleton and explicit `()`;
- treating `type data` as an ordinary type alias followed by `data`;
- making `intrinsic` imply TypeOnly;
- deleting `enum` compatibility before downstream source is ready;
- changing GADT syntax/meaning in this task.

### Test changes required

Cover CV-01, CV-02, CV-03 parser half, CV-05 syntax half, CV-12 syntax half.

Required parser fixtures/assertions include:

```text
data Point(_ x: Int, _ y: Int)
data Person { name: String; age: Int }
data State { Ready; Failed(_ error: Error) }
data Weekday { Monday, Tuesday, Wednesday }
data Message { Request { method: Method; path: String }; Closed }
data Example { Singleton; Constructed() }
data Impossible {}
type data Shape { Scalar; Vector(_ size: Int) }
intrinsic data Unit()
intrinsic type data ShapeKind { ... }
legacy enum equivalent canonicalizes to Runtime+Sum
mixed field/case body rejects
```

### Tests to run now

Run only parser-focused targets:

```sh
cargo test -p phalcom-ast --test data_syntax
cargo test -p phalcom-ast --test enum_syntax
```

If new cases are placed in `parser.rs`, run the exact new test filter after confirming
it selects tests.

Do not run semantic/core tests yet. Downstream AST consumers are intentionally
incomplete after T1.

### Acceptance

T1 passes when parser tests prove the complete P1 algebraic syntax matrix and all
canonical forms produce one AST model with correct phase/shape/payload distinctions.

### Local STOP / CONSULT triggers

- canonical AST cannot preserve current case-local generic/GADT result data;
- record-payload cases require inventing incompatible field identity semantics;
- existing parser grammar makes record-product vs sum discrimination fundamentally
  ambiguous under accepted syntax.

### Checkpoint update

Record the canonical AST phase/shape model as a durable interface after T1 and record
G1 evidence after the gate.

---

## T2 — Establish one semantic algebraic declaration authority

### Purpose

Build one authoritative semantic declaration product and migrate existing product and
sum declaration checking to project from it without losing runtime ADT/GADT identity.

### Preconditions

- T1 canonical AST is stable.
- G1 passes.
- Existing data/enum semantic builders and identity types have been read.

### Consumes

- canonical `AlgebraicDef`;
- existing `DataInfo`/`DataDeclarationProduct`;
- existing `EnumInfo`/`VariantInfo`/`EnumDeclarationProduct`;
- existing `VariantId`, `VariantFamilyId`, `VariantFieldId`;
- existing GADT `CaseTypeEnvironment`;
- current declaration identity/generic signature machinery.

### Produces

- `AlgebraicDeclarationInfo` / `AlgebraicDeclarationProduct` or exact equivalents;
- canonical product and sum representation branches;
- neutral case/payload semantic abstraction;
- one case identity lineage reusing `VariantId`;
- transitional Data/Enum adapters derived from the canonical product;
- runtime sum `data` semantic checking through the mature ADT/GADT machinery;
- record-payload case semantic representation.

### Files and symbols

**Read/Modify:**

```text
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/checker/data_declaration.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/mod.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/adts/data_declarations.rs
phalcom-semantic/tests/semantic/adts/declarations.rs
phalcom-semantic/tests/semantic/adts/constructors.rs
phalcom-semantic/tests/semantic/adts/variants.rs
phalcom-semantic/tests/semantic/adts/exact_cases.rs
phalcom-semantic/tests/semantic/adts/vertical_gadt.rs
```

### Required implementation shape

1. Build common facts exactly once:
   - declaration identity;
   - phase;
   - implementation provenance;
   - generic signature;
   - algebraic representation;
   - component/case set;
   - source provenance.
2. Product projection reuses current Data semantics.
3. Runtime sum projection reuses current enum/variant semantics, including:
   - selector-derived `VariantId`;
   - case family identity;
   - payload fields;
   - visibility axes;
   - result template;
   - exact-case template;
   - case environment;
   - case-local generics.
4. Introduce neutral payload shape sufficient for:
   - singleton;
   - tuple constructor;
   - record constructor;
   - explicit empty constructor.
5. Preserve legacy compatibility by feeding equivalent enum/data AST into the same
   semantic builder.
6. Existing `DataSemanticTable` and `EnumSemanticTable` may remain only as derived
   compatibility projections until later cleanup.

### Forbidden approaches

- independently running old data and old enum builders over the same declaration;
- building a new ADT/GADT type system beside `EnumInfo`;
- minting a second case identity;
- widening GADT results to the root type to simplify convergence;
- collapsing visibility axes;
- making product constructor visibility govern sum case construction;
- changing runtime representation here.

### Test changes required

Cover CV-03 semantic half, CV-04, CV-05 semantic half, CV-06.

Add direct semantic identity tests proving:

```text
enum Legacy { A; B(_ x: Int) }
data Legacy { A; B(_ x: Int) }
```

produce equivalent declaration/case semantic identities/facts when analyzed as
source alternatives in otherwise equivalent module identities.

Preserve existing vertical GADT and exact-case tests unchanged where possible; the
value of this gate is proving compatibility, not rewriting test oracles.

### Tests to run now

Use the semantic integration binary with the narrow existing responsibility modules:

```sh
cargo test -p phalcom-semantic --test semantic adts::data_declarations
cargo test -p phalcom-semantic --test semantic adts::declarations
cargo test -p phalcom-semantic --test semantic adts::exact_cases
cargo test -p phalcom-semantic --test semantic adts::vertical_gadt
```

Confirm each filter selects tests.

### Acceptance

T2 passes when runtime products and sums originate from one semantic algebraic
authority and current ADT/GADT exactness remains unchanged.

### Local STOP / CONSULT triggers

- canonical product cannot feed existing exact-case/GADT products without rebuilding
  them independently;
- `VariantId` semantics need to change rather than be neutralized by naming;
- equivalent enum/data cannot share identity/fingerprint inputs;
- record-payload cases require changing exact-case identity rules.

### Checkpoint update

Record the canonical semantic product, adapter rule, and preserved case identity after
T2. Record G2 evidence after the gate.

---

## T3 — Migrate downstream runtime consumers and preserve specialized lowering

### Purpose

Move downstream semantic/compiler consumers from obsolete AST-category reasoning to
canonical algebraic semantic facts while preserving existing product and ADT runtime
representations.

### Preconditions

- G2 passes.
- Runtime product and sum semantic projections are canonical.
- Current lowering specs and compiler dispatch have been read.

### Consumes

- canonical algebraic declaration facts;
- existing Data lowering;
- existing Enum/ADT lowering;
- exact-case/root impl machinery;
- match/exhaustiveness semantic products;
- core/native source integration.

### Produces

- canonical phase/shape-driven runtime dispatch;
- migrated AST consumers;
- preserved product lowering;
- preserved ADT/GADT lowering;
- source-index/tooling continuity sufficient for P1;
- no source-spelling-based compiler decision.

### Files and symbols

Expected affected files include:

```text
phalcom-modules/src/interface.rs

phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/enum_requirements.rs
phalcom-semantic/src/checker/coverage/**
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/mod.rs
phalcom-core/src/compiler/lib/data_decl.rs
phalcom-core/src/compiler/lib/enum_decl.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/native/source.rs

phalcom-lsp/src/semantic_tokens.rs
```

### Required implementation shape

1. Perform a targeted search for all remaining `Statement::Enum`, `EnumDef`,
   `Statement::Data`, and `DataDef` pattern matches after T1.
2. Classify each consumer:
   - syntax presentation;
   - semantic declaration;
   - runtime product;
   - runtime sum;
   - tooling continuity.
3. Migrate semantic consumers to phase/shape or canonical algebraic projections.
4. Preserve closed-sum matching/exhaustiveness triggers based on `Runtime + Sum`, not
   legacy enum syntax.
5. Preserve exact-case impl targets/root requirements/conformance behavior.
6. In compiler/lowering:
   - `Runtime + Product` routes to current product lowering;
   - `Runtime + Sum` routes to current ADT lowering;
   - no TypeOnly lowering is authorized yet.
7. Internal `EnumLoweringSpec`, `Bytecode::Enum`, `Variant*`, etc. may remain where
   they are still runtime representation terms rather than language-category truth.
8. Keep Option/Result specialized/native behavior unchanged.

### Forbidden approaches

- renaming every internal enum/variant helper merely for cosmetic consistency;
- creating a generic "one runtime object representation" for products and sums;
- reconstructing case sets from AST in compiler code;
- bypassing semantic lowering when linked semantic facts are available;
- broad LSP feature work;
- silently skipping unsupported TypeOnly declarations at runtime if that could allow
  runtime use to escape semantic rejection.

### Test changes required

Cover CV-04, CV-05 runtime half, CV-17, and the runtime part of CV-18.

Preserve/extend core algebraic data tests. Verify exact existing test names from:

```sh
cargo test -p phalcom-core --test core -- --list
```

before selecting filters.

### Tests to run now

Run the current focused algebraic-data compiler/runtime target documented by
`phalcom-core/tests/README.md` if it still exists:

```sh
cargo test -p phalcom-core --test core language::algebraic_data
```

Also run:

```sh
cargo test -p phalcom-core --test native_adt_runtime
```

If the first filter has mechanically drifted, resolve the exact current filter with
`-- --list`; do not broaden to all core tests just to compensate for a stale filter.

### Acceptance

T3 passes when all runtime declarations are dispatched from canonical algebraic facts,
existing products/sums execute through their established specialized representations,
and no runtime behavior depends on legacy `enum` spelling.

### Local STOP / CONSULT triggers

- compiler requires duplicating semantic case/GADT reasoning;
- preserving Option/native ADT behavior requires source keyword checks;
- a runtime representation redesign appears necessary;
- an AST consumer cannot migrate without becoming a competing semantic authority.

### Checkpoint update

Record runtime lowering as a representation projection beneath canonical semantic
phase/shape, not a unified storage requirement.

---

## T4 — Land top-level `type data`, nominal static domains, and canonical static terms

### Purpose

Establish the P1 static-domain foundation: nominal top-level `type data` declarations,
canonical static constructor/case terms, compile-time Int terms, static payload
validation, and hard runtime rejection.

### Preconditions

- canonical algebraic semantic product is stable;
- runtime product/sum migration is green;
- TypeStore kind/interner and `ProperTypeId` behavior have been read;
- existing `ConstTerm::Int` type-member path has been read.

### Consumes

- algebraic TypeOnly declarations from T1/T2;
- TypeStore kind/term interning;
- declaration identity;
- existing compile-time Int parsing/validation where reusable;
- existing RecordRow boundary.

### Produces

- `StaticDomainId`;
- nominal static-domain interning;
- `KindData::StaticDomain(StaticDomainId)`;
- stable top-level type-data domain registration;
- canonical `ConstInt` static term;
- canonical `StaticData` term;
- static constructor/case identity;
- static payload validation;
- runtime-use rejection;
- no VM lowering for type-data inhabitants.

### Files and symbols

**Read/Modify:**

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/identity.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/checker/**
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/kinds.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/adts/data_declarations.rs
```

Prefer adding type-data scenarios under the existing semantic responsibility tree
rather than making a second standalone semantic test binary.

### Required implementation shape

1. Add `StaticDomainId` as a store/snapshot-local identity.
2. Add canonical domain storage/interning keyed by semantic domain data.
3. Add exactly one dynamic nominal kind category:
   `KindData::StaticDomain(StaticDomainId)`.
4. Register one nominal static domain per top-level `type data` declaration.
5. Extend TypeData with compile-time Int and static-data constructor/case terms.
6. Introduce/reuse a `StaticArgument` envelope sufficient for static-data payload
   terms. It may include `Term(TypeId)` and `RecordRow(RecordRowId)`.
7. Do **not** change `TypeData::Applied` in this task.
8. Give each type-data constructor/case stable declaration-owned semantic identity.
9. Validate payload domains before term construction.
10. Canonicalize by structural semantic identity.
11. Keep symbolic well-formed terms symbolic where P1 encounters them; do not turn
    them into `Dynamic`, `Never`, or an unrelated proper type.
12. Add a direct proper-type boundary regression.
13. Reject runtime local/value/type materialization involving TypeOnly domains before
    compiler runtime lowering.
14. Existing `ConstTerm::Int` may remain a type-member compatibility surface, but it
    must not become a second durable language identity. Add explicit conversion or
    canonicalization through the TypeStore-owned Int term where needed.

### Forbidden approaches

- adding one `KindData` Rust variant per `type data` declaration;
- creating a second global StaticTerm arena duplicating TypeStore interning;
- weakening `ProperTypeId`;
- encoding static terms as fake nominal proper types;
- invoking runtime `==`, `hash`, method dispatch, or reflection for equality;
- evaluating arbitrary runtime expressions in static constructor arguments;
- collapsing RecordRow into TypeId;
- migrating general nominal generic application;
- adding runtime type-data objects/opcodes.

### Test changes required

Cover CV-07 through CV-12 and CV-18.

Minimum semantic tests:

```text
type data Width(_ value: Int)
type data Height(_ value: Int)
domain(Width) != domain(Height)

type data Shape {
    Scalar
    Vector(_ size: Int)
}

Shape::Vector(3) canonicalizes on repeated formation
Shape::Vector(3) != Shape::Vector(4)
Shape::Vector(3) != OtherShape::Vector(3)

proper_type(ConstInt(3)) rejects
proper_type(Shape::Vector(3)) rejects

runtime-only payload domain in type data rejects
runtime binding/materialization of Shape term rejects
intrinsic type data remains TypeOnly
intrinsic data remains Runtime
```

### Tests to run now

```sh
cargo test -p phalcom-semantic --test semantic foundations::kinds
cargo test -p phalcom-semantic --test semantic foundations::type_model
cargo test -p phalcom-semantic --test semantic adts::data_declarations
```

Run only exact new filters beyond these if needed.

Do not run generic-application/inference suites merely because new kinds exist; P2 owns
their generalized behavior unless a compile regression requires a bounded fix.

### Acceptance

T4 passes when top-level type-data domains and terms have canonical nominal/structural
identity, proper-type boundaries are intact, invalid static payloads are rejected,
and no runtime inhabitant path exists.

### Local STOP / CONSULT triggers

- TypeStore requires an independent competing canonical term universe;
- `ProperTypeId` must be weakened;
- P1 type-data formation requires general P2 generic application/inference;
- supported P1 payloads cannot be represented without collapsing RecordRow;
- runtime compiler/VM changes appear necessary for type-data term formation.

### Checkpoint update

Record the static-domain/term authority, proper-type boundary, and runtime-negative
boundary as durable P1 interfaces after T4.

---

## T5 — Persist algebraic/static-domain identity and certify incremental equivalence

### Purpose

Make P1's new semantic categories durable across metadata, imports, fingerprints, and
incremental analysis without committing P2's heterogeneous generic-application
encoding.

### Preconditions

- G3 passes.
- canonical phase/shape/static-domain/static-term representation is stable.
- current schema-v4 compatibility rules have been audited.

### Consumes

- algebraic declaration facts;
- stable declaration references;
- `StaticDomainId` canonical data;
- `TypeData::ConstInt` / `StaticData`;
- existing metadata/fingerprint/incremental machinery.

### Produces

- first C3 metadata schema transition;
- stable algebraic phase/shape records;
- stable nominal static-domain reference;
- stable static-data term encoding as required by P1 interfaces;
- stable case/constructor references;
- validation rules;
- enum/data spelling-equivalent fingerprint behavior;
- incremental invalidation/cold parity tests.

### Files and symbols

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/declaration.rs
phalcom-type-meta/src/validate.rs

phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs

phalcom-type-meta/tests/schema_compat.rs

phalcom-semantic/tests/semantic/incremental/adts.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
```

Verify the exact integration metadata path before editing.

### Required implementation shape

1. Audit the current v4 backward-compatibility contract.
2. Select the next monotonic schema version according to the live policy; record it in
   the checkpoint. The plan intentionally does not hardcode a number.
3. Extend stable kind/domain representation with a structural declaration-based
   nominal static-domain reference.
4. Encode algebraic phase and shape as stable semantic facts.
5. Encode case/constructor identity by stable declaration/member references, not
   source strings.
6. Encode static Int/static-data terms required by public P1 signatures.
7. Do **not** change `TypeNode::Applied` to heterogeneous arguments in P1.
8. Validate malformed combinations:
   - TypeOnly term with runtime-only payload domain;
   - unknown case/constructor;
   - wrong arity/labels;
   - inconsistent phase/shape;
   - raw/local identity leakage.
9. Update structural/interface fingerprints:
   - semantic phase change => change;
   - Product↔Sum => change;
   - case/payload/GADT changes => change;
   - nominal static domain/term changes => change;
   - legacy enum→equivalent data spelling only => no change.
10. Add cold/incremental parity for:
    - runtime product↔sum edit;
    - runtime↔type-data phase edit;
    - case add/remove/payload edit;
    - static constructor argument edit where P1 exposes one;
    - enum spelling→equivalent data spelling.

### Forbidden approaches

- serializing `StaticDomainId` or `TypeId`;
- identity by rendered names/source snippets;
- opportunistically implementing P2 heterogeneous `Applied`;
- silently mutating schema v4 in place;
- making legacy enum spelling a persistent declaration category;
- skipping validation because the semantic producer "should be correct."

### Test changes required

Cover CV-03, CV-13, CV-14, and CV-15.

### Tests to run now

```sh
cargo test -p phalcom-type-meta --test schema_compat
cargo test -p phalcom-semantic --test semantic incremental::adts
cargo test -p phalcom-semantic --test semantic incremental::fingerprints
cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions
cargo test -p phalcom-semantic --test semantic integration::metadata
```

Before the last command, confirm the live module/filter exists and selects tests.

### Acceptance

T5 passes when P1 identities round-trip stably, semantic changes invalidate correctly,
legacy enum→data spelling migration is identity/fingerprint-neutral, and incremental
final state matches cold analysis.

### Local STOP / CONSULT triggers

- stable metadata needs raw store-local IDs;
- schema compatibility cannot map legacy enum metadata to canonical runtime-sum facts
  without a material compatibility decision;
- P1 persistence requires general heterogeneous `Applied`;
- enum/data spelling-neutral fingerprints conflict with an established external
  compatibility contract.

### Checkpoint update

Record schema transition, stable identity encoding, and incremental gate results.

---

## T6 — Migrate authoritative specifications, stabilize P1, and close the plan

### Purpose

Remove normative contradictions introduced by P1, run the bounded focused acceptance
matrix, update lifecycle truth, and create the implementation walkthrough and P2
handoff.

### Preconditions

- G1–G4 pass.
- no unresolved A/B failures remain.
- C3 checkpoint ledger is current.

### Consumes

- implemented P1 behavior;
- accepted C3 specification;
- all gate evidence;
- deferred/baseline ledger.

### Produces

- effective P1-aligned normative docs;
- final focused verification evidence;
- truthful P1 lifecycle state;
- `LANG007.C3.P1-walkthrough.md`;
- `LANG007.C3.P1-handoff.md`;
- P2 takeover map.

### Files and symbols

Normative docs at minimum:

```text
docs/specs/objects/data-objects.md
docs/specs/objects/enums.md
docs/specs/type-level-domains.md
```

Also update directly contradictory effective sections in:

```text
docs/specs/objects/impl.md
the effective pattern/GADT owner
```

only where P1-shipped behavior would otherwise be contradicted.

Lifecycle:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P1-walkthrough.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P1-handoff.md
```

### Required implementation shape

1. Update the effective runtime data specification so `data` owns both Product and
   Sum forms.
2. Migrate enum/sum normative ownership so `enum` is compatibility/historical syntax,
   not a canonical semantic category.
3. Generalize the static-domain foundation to nominal user-defined TypeOnly domains.
4. Preserve explicit wording that:
   - type-data terms are not runtime values;
   - proper types are distinct from arbitrary static terms;
   - runtime sum exact-case/GADT semantics survive;
   - storage representation remains free.
5. Do not document P2/P3 capabilities as implemented.
6. Run G5 only after the coherent implementation is complete.
7. Update plan status:
   `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` only if all P1 acceptance obligations
   actually have evidence.
8. Keep `LANG007.C3` itself `IN_PROGRESS / PARTIAL` because P2/P3 remain.
9. Produce walkthrough and handoff.

### Forbidden approaches

- claiming associated type data, generalized static generics, Universe exposure, or
  full LSP support as implemented;
- marking C3 complete;
- claiming `RELEASE_COMPLETE`;
- rewriting historical implementation records;
- converting broad deferred testing into implied evidence.

### Tests to run now

Run G5 only; do not add workspace/release certification unless separately requested.

### Acceptance

T6 passes when normative docs no longer contradict P1, focused acceptance is green,
lifecycle records are truthful, and P2 can begin without rediscovering P1
architecture.

### Local STOP / CONSULT triggers

- an effective normative spec conflicts materially with the accepted C3 design;
- focused stabilization exposes an unresolved A/B semantic failure after the allowed
  correction budget;
- P2 takeover would require redesigning a P1 identity/authority decision.

### Checkpoint update

Final P1 update is mandatory. Set:

```yaml
LANG007.C3.P1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: FOCUSED_TESTED
```

only when supported by executed evidence.

---

# 19. Verification gates

## G1 — Canonical algebraic parser/AST gate

Purpose: prove syntax and shape distinctions before semantic migration.

Run:

```sh
cargo test -p phalcom-ast --test data_syntax
cargo test -p phalcom-ast --test enum_syntax
```

Required evidence:

- runtime Product tuple/record;
- runtime Sum singleton/tuple/record payload;
- singleton shorthand;
- singleton vs explicit empty constructor;
- GADT case syntax;
- TypeOnly equivalents;
- intrinsic provenance;
- empty product vs empty sum;
- mixed-body rejection;
- legacy enum canonical AST compatibility.

Do not broaden after PASS.

---

## G2 — Runtime algebraic semantic preservation gate

Purpose: prove canonical semantic authority without losing mature ADT/GADT behavior.

Run:

```sh
cargo test -p phalcom-semantic --test semantic adts::data_declarations
cargo test -p phalcom-semantic --test semantic adts::declarations
cargo test -p phalcom-semantic --test semantic adts::constructors
cargo test -p phalcom-semantic --test semantic adts::variants
cargo test -p phalcom-semantic --test semantic adts::exact_cases
cargo test -p phalcom-semantic --test semantic adts::vertical_gadt
```

Required evidence:

- Product and Sum share canonical algebraic authority;
- exact-case identity unchanged;
- GADT equalities/refinement unchanged;
- equivalent enum/data semantic identity;
- record-payload cases work;
- no second semantic builder.

Do not run all semantic tests after PASS.

---

## G3 — Static-domain/type-data foundation gate

Purpose: prove nominal domain identity, canonical terms, proper-type boundary, and
runtime-negative behavior.

Run:

```sh
cargo test -p phalcom-semantic --test semantic foundations::kinds
cargo test -p phalcom-semantic --test semantic foundations::type_model
cargo test -p phalcom-semantic --test semantic adts::data_declarations
```

Also run the exact new type-data regression filters created by T4.

Required evidence:

- nominal domain distinction;
- term interning/canonicalization;
- constructor/domain disjointness;
- proper-type rejection;
- invalid payload rejection;
- runtime-use rejection;
- no VM type-data path.

---

## G4 — Persistence, incremental, and runtime representation gate

Purpose: prove durable identity and cross-layer preservation without P2 scope creep.

Run:

```sh
cargo test -p phalcom-type-meta --test schema_compat
cargo test -p phalcom-semantic --test semantic incremental::adts
cargo test -p phalcom-semantic --test semantic incremental::fingerprints
cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions
cargo test -p phalcom-semantic --test semantic integration::metadata
cargo test -p phalcom-core --test core language::algebraic_data
cargo test -p phalcom-core --test native_adt_runtime
```

Verify test filters select tests. If `language::algebraic_data` or
`integration::metadata` drifted mechanically, resolve their exact live filter first.

Required evidence:

- stable metadata round-trip;
- enum→data spelling identity neutrality;
- semantic edit fingerprint sensitivity;
- cold/incremental parity;
- runtime product/sum representation behavior preserved;
- native ADT behavior preserved.

---

## G5 — Final P1 focused stabilization gate

Purpose: prove the complete P1 coverage surface without escalating to release
certification.

Run the following owning suites serially:

```sh
cargo test -p phalcom-ast --test data_syntax
cargo test -p phalcom-ast --test enum_syntax

cargo test -p phalcom-semantic --test semantic adts
cargo test -p phalcom-semantic --test semantic foundations::kinds
cargo test -p phalcom-semantic --test semantic foundations::type_model
cargo test -p phalcom-semantic --test semantic incremental::adts
cargo test -p phalcom-semantic --test semantic incremental::fingerprints
cargo test -p phalcom-semantic --test semantic integration::metadata

cargo test -p phalcom-type-meta --test schema_compat

cargo test -p phalcom-core --test core language::algebraic_data
cargo test -p phalcom-core --test native_adt_runtime
```

Additionally run the directly affected C2 kind/type-model regressions if the live test
layout does not already include them in the commands above. Do not run the whole
workspace merely to increase the verification label.

Expected P1 verification state after a clean G5:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

not `RELEASE_COMPLETE`.

---

## 20. Final focused acceptance

At P1 completion, the walkthrough must map every coverage obligation to executed
evidence.

Required final matrix:

| Coverage | Required result |
|---|---|
| CV-01 canonical syntax | PASS |
| CV-02 body discrimination | PASS |
| CV-03 enum/data compatibility identity | PASS |
| CV-04 runtime sum preservation | PASS |
| CV-05 record-payload sum | PASS |
| CV-06 one semantic authority | PASS |
| CV-07 nominal static domain | PASS |
| CV-08 static term canonicalization | PASS |
| CV-09 proper-type boundary | PASS |
| CV-10 static payload validity | PASS |
| CV-11 runtime-negative boundary | PASS |
| CV-12 intrinsic orthogonality | PASS |
| CV-13 metadata stability | PASS |
| CV-14 fingerprint sensitivity/neutrality | PASS |
| CV-15 cold/incremental parity | PASS |
| CV-16 C2 regression | PASS |
| CV-17 runtime representation freedom | PASS |
| CV-18 no static VM work | PASS |

P1 must not close with an unresolved A/B failure in this matrix.

---

## 21. Performance/resource evidence

P1 is not a performance-optimization plan, but it must avoid obvious semantic
complexity regressions.

Required design properties:

- case duplicate detection remains indexed/hash-based rather than avoidably quadratic;
- static domain/term canonicalization uses interning/hash-consing;
- no source-string parse/render loop is introduced for metadata;
- substitution/interning work remains proportional to reachable static-term size;
- no general evaluator or search/backtracking solver is introduced;
- runtime ADT representation remains specialized where it already is.

No standalone benchmark suite is mandatory unless implementation evidence shows a
material compile-time or memory regression.

---

## 22. Checkpoint bookkeeping

### 22.1 At plan start

The implementer must:

1. inspect `git status --short`;
2. create or normalize `LANG007.C3-CHECKPOINT.md`;
3. add P1 to the plans ledger and mark it active;
4. record starting revision;
5. rebaseline `PROGRAM.md` and `STATUS.md` to C3/P1 current truth;
6. record inherited C2 stable interfaces and baseline failures;
7. ensure accepted C3 requirements/specification sources are referenced.

### 22.2 During plan

Update the checkpoint only when one of these durable events occurs:

- canonical algebraic AST is established;
- canonical semantic algebraic authority is established;
- static-domain/term authority is established;
- a consultation/amendment changes architecture;
- a baseline/deferred failure is classified;
- G1–G4 completes.

Do not turn the checkpoint into a chronological edit log.

### 22.3 At plan completion

Record:

- final P1 state;
- final durable interfaces/invariants;
- G5 verification evidence;
- chosen metadata schema transition;
- deferred/baseline failures;
- consultations/amendments;
- P2 next action.

C3 remains active/incomplete.

---

## 23. Walkthrough deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P1-walkthrough.md
```

It must contain:

1. plan identity and final lifecycle state;
2. objective and actual outcome;
3. canonical algebraic AST implemented;
4. semantic authority implemented;
5. runtime product/sum migration;
6. exact-case/GADT preservation;
7. static-domain and static-term architecture;
8. metadata/schema changes;
9. important files/components changed;
10. durable interfaces/invariants established;
11. deliberate deviations from the plan;
12. consultations and adopted decisions;
13. tests added;
14. tests actually executed;
15. tests deliberately deferred;
16. residual/baseline failures;
17. verification classification;
18. follow-up work required by P2.

Never claim a test passed without completed evidence.

---

## 24. Handoff deliverable

Create:

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P1-handoff.md
```

The handoff must contain:

1. current repository revision/state anchor;
2. P1 final lifecycle state;
3. stable interface/takeover map;
4. canonical algebraic declaration architecture;
5. case identity rule;
6. runtime representation boundary;
7. `StaticDomainId` / `KindData::StaticDomain` architecture;
8. canonical static-term forms;
9. `ProperTypeId` boundary;
10. metadata/schema identity rules;
11. inherited C2 invariants;
12. deferred/baseline failures;
13. must-read files;
14. P2 objective:
    generalized static domains + heterogeneous application/inference +
    associated `type data`;
15. first recommended P2 commands/tests;
16. explicit guidance not to redesign P1/C2 substrates.

The handoff must state prominently:

```text
P2 must generalize generic/application/inference consumers onto the P1 static-domain
foundation. It must not replace TypeId, ProperTypeId, StaticDomainId, canonical
type-data term identity, RecordRow ownership, or the unified algebraic declaration
authority.
```

---

## 25. Completion truth table

Do not conflate:

```text
canonical AST written
semantic adapters compile
runtime ADT regressions pass
type-data terms exist
metadata round-trips
incremental parity passes
P1 focused acceptance passes
C3 checkpoint closes
release certification passes
```

The intended P1 terminal state is:

```yaml
LANG007.C3.P1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3:
  status: IN_PROGRESS
  completion: PARTIAL
  verification: FOCUSED_TESTED
```

`LANG007.C3` cannot become COMPLETE until P2 and P3 acceptance objectives also pass.

`RELEASE_COMPLETE` is not implied.

---

## 26. Plan self-review

Before implementation begins, verify this plan still satisfies all of the following:

### Architecture

- one canonical algebraic declaration authority exists in the plan;
- runtime Product/Sum storage is not unnecessarily unified;
- existing exact-case/GADT machinery is reused;
- case identity is not duplicated;
- `TypeId` remains non-row canonical static-term identity;
- `ProperTypeId` is not weakened;
- nominal static domains are open-ended through one dynamic classifier;
- RecordRow remains separate;
- no runtime/static VM execution path is planned;
- P2 heterogeneous generic application is not accidentally implemented in P1.

### Luna executability

- every task has a bounded purpose and concrete output;
- fixed/flexible/verify-first boundaries are explicit;
- high-risk semantic failure loops have objective STOP/CONSULT triggers;
- likely cross-layer migration surfaces are named;
- mechanical naming drift can be adapted without consultation.

### Testing

- parser, semantic identity, ADT/GADT preservation, static-domain identity,
  metadata, incremental behavior, and runtime boundary all have coverage;
- commands are focused and serial;
- filters must be confirmed to select tests;
- workspace/release gates are explicitly deferred;
- inherited unrelated trait failures are not active scope.

### Lifecycle

- C3 checkpoint creation/normalization occurs at plan start;
- stale program/status state is rebaselined;
- historical completed records are preserved;
- walkthrough and handoff are mandatory;
- plan completion does not imply checkpoint/release completion.

### Residual planning risks

The highest implementation risks remain:

1. losing exact-case/GADT information while neutralizing Enum/Data source categories;
2. accidentally retaining two semantic builders behind a common facade;
3. collapsing singleton and explicit zero-argument constructor cases;
4. allowing type-data terms into proper runtime type/value APIs;
5. over-expanding P1 into P2 heterogeneous generic application;
6. encoding store-local static-domain IDs in metadata;
7. producing different fingerprints for equivalent legacy enum/canonical data source;
8. accidentally making runtime representation convergence a requirement of source
   semantic convergence.

If implementation evidence activates any of these as an architectural problem, stop
and consult rather than compensating with a special case.

---

# 27. Implementation sequence summary

The intended execution order is:

```text
T1
    C3 lifecycle activation
    + canonical algebraic AST/parser
        ↓
G1
        ↓
T2
    one semantic algebraic authority
    + identity-preserving runtime sum migration
        ↓
G2
        ↓
T3
    downstream runtime consumer/lowering migration
    while retaining specialized representations
        ↓
T4
    top-level type data
    + StaticDomainId / KindData::StaticDomain
    + canonical static terms
    + proper-type/runtime boundary
        ↓
G3
        ↓
T5
    stable metadata/fingerprints
    + incremental/cold parity
        ↓
G4
        ↓
T6
    effective spec migration
    + focused stabilization
    + lifecycle records
        ↓
G5
        ↓
P1 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
        ↓
P2 handoff
```

This ordering is intentional. Do not start by generalizing generic application or
metadata before the canonical algebraic/static-domain identities exist.
