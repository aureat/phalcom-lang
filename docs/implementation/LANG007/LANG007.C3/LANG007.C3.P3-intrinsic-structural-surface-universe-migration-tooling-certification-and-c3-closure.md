---
id: LANG007.C3.P3
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-plan
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG007.C3.P2
  - LANG007.C3.P1
  - LANG007.C2
follows: LANG007.C3.P2
supersedes: null
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 60a27f79d69aa3e79c22e3d19df221466c7db920
prepared: 2026-09-22
title: "Intrinsic Structural Surface, Universe Migration, Tooling, Certification, and C3 Closure"
---

# LANG007.C3.P3 — Intrinsic Structural Surface, Universe Migration, Tooling, Certification, and C3 Closure

## 0. Executor contract

This is the Luna-oriented patch-grade implementation plan for the final planned work unit of `LANG007.C3 — Type Data and Unified Algebraic Static Domains`.

P3 is a **migration, integration, tooling, certification, and checkpoint-closure plan**. It is deliberately not a new type-system architecture plan. It must consume the semantic architecture completed by P1 and P2, expose the compiler's already-canonical structural domains through the accepted source language, migrate canonical source and tooling to that model, retire legacy `enum` declaration syntax, then certify P1+P2+P3 as one coherent checkpoint.

The implementer may adapt private helper names, local decomposition, imports, exact test filters, file-local representations, borrow-checker mechanics, and other mechanical details to the live repository. The implementer must not silently change canonical semantic ownership, identity, runtime representation boundaries, row ownership, static-term representation, associated-domain identity, source-index authority, or the final language decisions fixed by this plan.

Operating rules:

- verify the completed-P2 live revision before substantive P3 edits;
- consume the completed P2 walkthrough and P2→P3 handoff instead of rediscovering P2;
- preserve unrelated modified, staged, and untracked work;
- keep `TypeId` as the canonical non-row static-term handle and `ProperTypeId` as the proof boundary for proper runtime Types;
- keep `RecordRowId` and the row solver separate from `TypeId`;
- keep `StaticArgument` as the single heterogeneous generic/static argument envelope established by P2;
- keep `AssociatedSurface` as the one `::` namespace/collision authority;
- keep C1/C2 pack semantics and identities unchanged;
- treat `TupleEntries` as a public facade over `CompleteTypePack`, never as a new semantic domain;
- treat intrinsic structural declarations as source/reflection facades over existing compiler/runtime authorities;
- preserve specialized runtime representations for `Unit`, `Tuple`, `Record`, `Option`, `Result`, and `Ordering` where they exist;
- migrate repository-owned active source before retiring legacy `enum` syntax;
- make editor/LSP behavior consume semantic target/reference products rather than reconstruct names textually;
- use sparse discriminating tests during BUILD mode and broader C3-specific evidence only at deliberate certification gates;
- classify and defer unrelated/baseline failures rather than repairing the world;
- STOP AND CONSULT when a trigger in this plan fires;
- keep `LANG007.C3-CHECKPOINT.md` current at meaningful lifecycle events;
- produce the required P3 walkthrough and terminal C3 handoff.

Testing is evidence gathering, not a ritual. Do not run the entire workspace after each migration step.

### 0.1 Hard execution gate — P3 is not executable from the current planning state

This plan was prepared while the repository was still at a **partial P1** revision and before P2 implementation began.

Planning baseline:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   60a27f79d69aa3e79c22e3d19df221466c7db920
subject:    feat(semantic): type data foundation
state:      P1 partial; P2 not started; P3 future
```

**Do not execute P3 against this revision.**

P3 begins only after both predecessor plans truthfully report:

```yaml
LANG007.C3.P1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3.P2:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

and the completed-P2 handoff confirms the hard predecessor contract in §6.1 below.

If P1 or P2 is still partial, blocked, unverified in a way that affects the P3 contract, or missing its walkthrough/handoff, **stop before P3 implementation**. The missing work belongs to corrective predecessor closure, not opportunistic P3 absorption.

### 0.2 P3 responsibility order

P3 must preserve this dependency order:

```text
1. EXPOSE
   bind compiler-owned structural static/runtime domains to canonical source declarations

2. MIGRATE
   move Universe and active first-party source onto the final data/type-data model
   without changing semantic identity

3. TOOL
   make source index, reflection, diagnostics, LSP, and editor syntax consume the
   final semantic identities and terminology

4. RETIRE
   remove accepted legacy enum declaration syntax only after active source is migrated

5. CERTIFY
   validate the full C3 semantic/persistence/runtime/tooling matrix and close the checkpoint
```

Do not reorder parser retirement ahead of source migration. Do not migrate `Tuple`/`Record`/`Unit` before the intrinsic source-facade binding exists.

---

## 1. Goal

Expose the compiler's existing structural static and runtime domains through canonical `intrinsic type data` / `intrinsic data` source facades; add the `TupleEntries`/`Tuple::Entries` public bridge; migrate canonical Universe and active repository source from legacy `enum`/class presentation to the final C3 model without changing semantic or runtime identity; complete source-index/LSP/editor/specification migration; retire accepted `enum` syntax; and certify/close `LANG007.C3` without weakening the runtime/static/proper-type boundary.

---

## 2. Checkpoint acceptance objective

`LANG007.C3` owns one coherent algebraic/static-domain language model:

```text
data
    sole canonical runtime algebraic declaration form
    ├─ Product
    └─ Sum

type data
    type-system-only algebraic/static-domain declaration
    ├─ user nominal static domains
    └─ intrinsic compiler structural facades
```

P1 establishes unified runtime algebraic declarations plus top-level `type data` foundations. P2 generalizes static-domain application, inference, substitution, aliases/type members, associated `type data`, persistence, and source identity. P3 makes that model the canonical public platform surface and proves every affected layer agrees.

P3 is the terminal planned checkpoint work unit. It is successful only when the following statement is true:

> The canonical source language exposes compiler-owned structural domains through `intrinsic data` / `intrinsic type data` without duplicating C1/C2/P1/P2 semantic identity; TupleEntries is the public name for the existing CompleteTypePack domain; Unit/Tuple/Record and Option/Result/Ordering retain their existing semantic/runtime identities under the new source presentation; active first-party source no longer requires `enum`; accepted `enum` declaration syntax is retired; source index, LSP, reflection, diagnostics, editor grammar, metadata, incremental products, and authoritative specs all consume the same final C3 model; and no runtime static solver, runtime type-data object model, or weakened ProperType boundary has been introduced.

### 2.1 Required source-visible endpoint

The endpoint includes declarations equivalent to:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

and structural members equivalent to:

```phalcom
impl Tuple<***Es> {
    intrinsic type Entries: TupleEntries
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type At<I: Int>: Type
    intrinsic type Map<F: Type -> Type>: Type
}
```

with semantic equivalences:

```text
TupleEntries == existing KindId::COMPLETE_TYPE_PACK / TypeData::CompleteTypePack domain

*T   == T::Positionals
**T  == T::Labeled
***T == T::Entries
```

where those operations are defined.

### 2.2 Final enum rule

Final P3 language acceptance is:

```text
data       canonical runtime Product/Sum declaration keyword
type data  canonical TypeOnly static-domain form
enum       not an accepted Phalcom declaration form
```

Private Rust/runtime names such as `EnumInfo`, `VariantId`, `enum_decl.rs`, or ADT implementation terminology may remain when they are implementation details and do not preserve a second source-language category.

### 2.3 Closure strength

Successful P3 may close C3 as:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

when the required checkpoint-specific certification passes.

P3 must **not** infer `RELEASE_COMPLETE` from checkpoint closure. Release-strength verification is a separate claim requiring the repository's broader release evidence.

---

## 3. Repository grounding

### 3.1 Planning baseline

Prepared from the P3 requirements analysis produced against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   60a27f79d69aa3e79c22e3d19df221466c7db920
subject:    feat(semantic): type data foundation
parent:     62a6082ab463387c812d558e5561508267ecc63c
date:       2026-09-22
```

At that revision:

```text
LANG007.C3     IN_PROGRESS / PARTIAL / UNVERIFIED
P1             partially implemented
P2             not started / not landed
P3             future
```

### 3.2 Execution baseline

The actual P3 execution revision must be the completed-P2 revision.

At P3 start record:

```text
P1 completion revision:
P1 final verification:
P1 walkthrough:
P1 handoff:

P2 completion revision:
P2 final verification:
P2 walkthrough:
P2 handoff:

P3 starting revision:
working-tree status:
metadata schema version:
current baseline-failure ledger:
```

These are execution-time facts. Do not copy the planning baseline into the walkthrough as though P3 executed there.

### 3.3 Verified pre-P3 baseline observations

The requirements audit at `60a27f79...` found, among other things:

- canonical Universe still used `native enum` for `Option`, `Result`, and `Ordering`;
- additional active `.ph` source still used `enum`;
- `Unit`, `Tuple`, and `Record` were source-anchored as native classes rather than `intrinsic data`;
- `Tuple` exposed `Positionals`, `Labeled`, `At`, and `Map`, but not `Entries`;
- `TupleEntries` was not yet a live canonical Universe source declaration;
- `TypePack`, `LabeledTypePack`, and `RecordRow` lacked canonical Universe source declarations;
- intrinsic type-member lookup still contained source-name-sensitive seams such as `"Tuple"`, `"TypePack"`, and `"RecordRow"`;
- the native/Universe source census was class/enum-oriented (`UniverseClassRow`, `SourceEnumRecord`, `SourceDeclarationRecord::{Class, Enum}`-style concepts);
- `SourceDeclarationKind` still contained `Enum`;
- LSP semantic tokens still recognized `Token::Enum` as a keyword;
- the VS Code TextMate grammar still had enum-specific declaration/variant rules;
- the LSP had definition/references/hover/completion support but no completed rename/prepareRename flow;
- authoritative `docs/specs/` still described `enum` as the canonical sum form and `data` as product-only.

These are migration indicators, not permission to assume the completed-P2 tree still has exactly the same private symbols.

### 3.4 Expected repository areas

P3 is expected to touch some or all of:

```text
phalcom-ast/
phalcom-modules/
phalcom-semantic/
phalcom-type-meta/
phalcom-core/
phalcom-lsp/
tools/vsphalcom/
docs/specs/
docs/implementation/LANG007/
Universe/first-party .ph source and active test/example fixtures
```

Known likely front-end retirement paths include:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/error.rs

phalcom-ast/tests/lexer.rs
phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/parser.rs
```

Known likely semantic/tooling paths include:

```text
phalcom-semantic/src/source_index/**
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/declarations.rs

phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/src/backend.rs

tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
editor golden fixtures
```

Private runtime files such as `enum_decl.rs`, `enum_semantics.rs`, or identifiers such as `VariantId` are not automatic rename targets.

> Re-read all named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Read these before substantive P3 edits:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-GUIDANCE.md

docs/implementation/LANG007/LANG007.C3/LANG007.C3.P1-...-walkthrough.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P1-...-handoff.md

docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-...-walkthrough.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P2-...-handoff.md

LANG007.C3.P3-requirements-analysis.md
the accepted LANG007.C3 unified algebraic/type-data specification
all checkpoint amendments/consultations recorded since this plan was authored

docs/implementation/LANG007/LANG007.C2/LANG007.C2-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C2/LANG007.C2.P3-walkthrough.md

docs/specs/README.md
docs/specs/objects/data-objects.md
docs/specs/objects/enums.md or its live successor
docs/specs/objects/impl.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/associated-types.md
docs/specs/structural-products.md
docs/specs/universe.md
live GADT/pattern/exact-case owning specs

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
phalcom-lsp test conventions/readme if present
VS Code extension test conventions if present
```

Before each task, read only that task's named source areas plus directly adjacent helpers needed to understand the call path. Do not broadly re-audit P1/P2 when their completed handoffs and live tree agree.

---

## 5. Normative authority

### 5.1 Final algebraic/static source model

The accepted model is:

```text
data
    phase = Runtime
    shape = Product | Sum

type data
    phase = TypeOnly
    shape/domain = nominal static algebra or compiler intrinsic structural facade

implementation provenance
    Phalcom | Native | Intrinsic | ...

ownership
    module | associated nominal owner
```

Phase, shape, implementation provenance, and ownership remain orthogonal.

### 5.2 Canonical static-domain architecture inherited from P1/P2

P3 consumes, rather than redesigns:

```text
TypeId
    canonical handle for non-row static terms

ProperTypeId
    proof that a TypeId is specifically KindId::TYPE

KindId / KindData
    one static-domain classifier system

StaticDomainId
    canonical identity for user/nominal type-data domains

StaticArgument
    canonical heterogeneous application envelope

RecordRowId
    row-owned canonical identity outside TypeId

InferVarId(kind)
    non-row inference variable model

AssociatedSurface
    one declaration-owned :: namespace/collision authority
```

### 5.3 Intrinsic static-domain facades

Required mapping:

| Source facade | Canonical semantic authority | Forbidden duplicate |
|---|---|---|
| `TypePack` | `KindId::TYPE_PACK` / existing pack terms | new nominal `StaticDomainId(TypePack)` |
| `LabeledTypePack` | `KindId::LABELED_TYPE_PACK` / existing pack terms | new nominal static-data domain |
| `TupleEntries` | `KindId::COMPLETE_TYPE_PACK` / `TypeData::CompleteTypePack` | `TypeData::TupleEntries` / second solver |
| `RecordRow` | `KindId::RECORD_ROW` / `RecordRowId` / row solver | fake row `TypeId` or nominal row domain |

A source declaration is a canonical public anchor. It does not replace the existing term/domain authority.

### 5.4 Intrinsic runtime structural facades

Required mapping:

```text
intrinsic data Unit()
    -> existing canonical Unit TypeStore/runtime identity

intrinsic data Tuple<***Es>
    -> existing structural Tuple type formation
    -> existing Tuple runtime class / UniverseKey association
    -> existing specialized Tuple representation

intrinsic data Record<R: RecordRow>
    -> existing TypeData::Record(RecordRowId)
    -> existing Record runtime class / UniverseKey association
    -> existing specialized Record representation
```

The source declaration identity, structural type identity, and runtime dispatch class are related projections of one platform concept; they are not interchangeable IDs.

### 5.5 Bodiless intrinsic declaration rule

P3 must support declaration-only forms such as:

```phalcom
intrinsic type data TypePack
intrinsic data Tuple<***Es>
```

with the meaning:

> This canonical source declaration presents a domain whose full representation/algebra is supplied by registered compiler-owned intrinsic semantics.

It does **not** mean empty sum, zero-field product, unknown abstract domain, runtime native body, or opaque fallback.

Every bodiless intrinsic declaration must bind to one registered compiler authority. Ordinary user bodiless `data` / `type data` remains invalid unless a separately accepted rule exists.

### 5.6 TupleEntries public naming

`TupleEntries` is the public name for the complete two-lane tuple entry static domain.

Internal authority remains:

```text
KindId::COMPLETE_TYPE_PACK
TypeData::CompleteTypePack
project_complete
project_positionals
project_labeled
```

Public presentation may say `TupleEntries`; implementation-oriented debug output may retain `CompleteTypePack` where appropriate.

### 5.7 Final enum policy

After active-source migration:

```text
enum -> Runtime + Sum compatibility
```

must cease to be an accepted source route.

Preferred final lexical policy is true retirement of `Token::Enum` and lexer keyword recognition **unless** a completed authoritative lexical decision explicitly retains `enum` as a reserved obsolete token solely for a hard diagnostic. Either final implementation must produce no semantic declaration from `enum`.

### 5.8 LSP rename architecture

Rename must be a thin projection over compiler-owned identity:

```text
cursor
    -> canonical SemanticTargetId (or live equivalent)
    -> declaration source site + semantic reference sites
    -> editability/name-category validation
    -> WorkspaceEdit
```

No textual all-matches fallback. If semantic target resolution fails, rename declines/fails safely.

### 5.9 Current implementation is not normative

Private names such as:

```text
SourceEnumRecord
UniverseClassRow
SourceDeclarationKind::Enum
EnumInfo
VariantInfo
VariantId
enum_decl.rs
```

are baseline implementation facts. They do not preserve an `enum` language category after P3.

### 5.10 Future work is not P3 authority

P3 does not authorize new static evaluators, new row solving, trait-owned type-data requirements, trait generic static-domain redesign, runtime type-data values, generic Tuple/Record storage redesign, general reflection architecture beyond the required facades, or unrelated release cleanup.

---

## 6. Takeover state

### 6.1 Hard P2 predecessor contract

P3 may begin only if the completed-P2 tree and handoff establish all of the following.

#### Canonical static term/application substrate

```text
[ ] TypeId remains canonical for non-row static terms
[ ] ProperTypeId still proves KindId::TYPE
[ ] StaticDomainId is canonical for user nominal static domains
[ ] StaticArgument is the canonical heterogeneous application envelope
[ ] TypeData::Applied preserves heterogeneous arguments
[ ] RecordRow remains row-owned
[ ] generic binder domains use one KindId classification system
```

#### Static inference/substitution

```text
[ ] non-row static inference reuses InferVarId(kind)
[ ] type-data unification is structural/first-order
[ ] checked substitution/materialization re-canonicalizes
[ ] static aliases/result domains work
[ ] type-member Int/user-domain semantics use canonical static terms
```

#### Associated domains

```text
[ ] associated type-data declarations exist
[ ] associated-domain declaration identity is stable
[ ] generic-owner specialization is declaration + canonical environment
[ ] AssociatedSurface owns :: collision/lookup semantics
[ ] source index can attach associated static identities
```

#### Persistence

```text
[ ] static domains/terms/applications round-trip through stable metadata
[ ] imported static domains preserve identity
[ ] incremental and cold P2 products converge
[ ] query-local IDs do not enter durable products
```

#### Runtime boundary

```text
[ ] type-data terms have no runtime inhabitants
[ ] no VM static solver exists
[ ] arbitrary runtime code is not executed for static identity
[ ] runtime proper-Type consumers still validate ProperType
```

If any item is false, P3 stops. Do not implement the missing semantic substrate locally in P3.

### 6.2 Stable interface/takeover map

| Concept | Expected completed-P2 owner/symbol | P3 invariant |
|---|---|---|
| non-row static term | `TypeStore` / `TypeId` | no replacement |
| proper-type proof | `ProperTypeId` | remains strict |
| static classifier | `KindId` / `KindData` | one classifier system |
| user nominal static domain | `StaticDomainId` | not used for structural builtin facades |
| heterogeneous argument | `StaticArgument` | one envelope |
| row identity | `RecordRowId` / row subsystem | remains separate |
| complete tuple-entry pack | `KindId::COMPLETE_TYPE_PACK` / `TypeData::CompleteTypePack` | public name `TupleEntries` only |
| complete projection | `project_complete` or live equivalent | powers `Tuple::Entries` |
| tuple structural types | `TypeData::Tuple` / `TupleFromPacks` family | not nominalized |
| record structural type | `TypeData::Record(RecordRowId)` | not replaced with nominal Applied |
| Unit | canonical Unit term/runtime identity | one identity |
| associated namespace | `AssociatedSurface` | remains sole collision/lookup authority |
| associated domain specialization | declaration + canonical owner environment | no formatted owner strings |
| source semantic identity | source index target/reference products | powers LSP/tooling |
| metadata | `phalcom-type-meta` stable graph | no raw local IDs |
| incremental | semantic DB/fingerprints | dependency-directed parity |
| runtime ADT/GADT | existing exact-case/variant machinery | preserve behavior/identity |
| native Universe census | source/runtime descriptor authority | generalized, not weakened |

### 6.3 Inherited baseline failures

P3 must read the **completed P2 handoff** for the actual baseline ledger.

Historically C2 recorded:

```text
BASELINE-001
semantic::capabilities::traits::
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

Do not assume these still exist. Do not repair inherited failures unless C3 causality is established or they block a required P3 gate.

---

## 7. Architecture

### 7.1 Direction of authority

P3 must establish this final direction of authority:

```text
canonical authored Universe/source declarations
        │
        ├─ ordinary data / type data
        │
        └─ intrinsic structural facade declarations
                    │
                    ▼
semantic declaration authority + intrinsic binding registry
                    │
        ┌───────────┼─────────────────────┐
        ▼           ▼                     ▼
 TypeStore/kinds   row authority      AssociatedSurface
        │           │                     │
        └───────────┴─────────┬───────────┘
                              ▼
                 canonical source semantic index
                    │                     │
                    ▼                     ▼
           stable metadata/imports     LSP/tooling
                    │
                    ▼
            runtime lowering only for Runtime declarations
                    │
                    ▼
             existing specialized runtime representations
```

No consumer should infer canonical structural-domain identity from strings, runtime object layout, or source spelling once semantic products exist.

### 7.2 Intrinsic binding registry

Use a semantic association conceptually equivalent to:

```rust
enum IntrinsicStaticDomain {
    TypePack,
    LabeledTypePack,
    TupleEntries,
    RecordRow,
}

struct IntrinsicStaticDomainBinding {
    declaration: DeclarationId,
    domain: IntrinsicStaticDomain,
}
```

and an equivalent registered mapping for intrinsic runtime structural declarations (`Unit`, `Tuple`, `Record`) to their existing semantic/runtime owners.

Exact private names are flexible. Required ownership is not:

```text
Universe source declaration
    owns public source/reflection identity

intrinsic binding registry
    owns source declaration -> existing compiler authority association

TypeStore / row subsystem / runtime Universe
    continue to own actual semantic/runtime identity
```

The registry is a binding table, not a second type store, kind registry, row store, or runtime class universe.

### 7.3 Structural static facades

Resolve source declarations:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow
```

to:

```text
KindId::TYPE_PACK
KindId::LABELED_TYPE_PACK
KindId::COMPLETE_TYPE_PACK
KindId::RECORD_ROW
```

respectively.

Do not allocate nominal `StaticDomainId`s for those four structural builtins merely because their public syntax is `type data`.

### 7.4 TupleEntries and structural type-member bridge

Add canonical source/member identity for:

```phalcom
Tuple::Entries
```

with normalization delegating to the existing complete-pack projection.

Preserve:

```text
Tuple::Positionals
Tuple::Labeled
Tuple::At
Tuple::Map
TypePack::Map
LabeledTypePack::Map
RecordRow::Map
Record::Map
```

Remove source spelling as the authority for intrinsic member recognition. Lookup may render names for diagnostics, but semantic dispatch must be keyed by canonical declaration/domain/member identity.

### 7.5 Unit bridge

`intrinsic data Unit()` must preserve:

```text
source canonical declaration identity
UniverseKey::Unit association
canonical TypeStore Unit identity
empty-tuple canonicalization
Unit -> Object runtime relation
existing runtime behavior/representation
```

It must not mint a second nominal Unit TypeId or use generic `DataObject` storage.

### 7.6 Tuple bridge

`intrinsic data Tuple<***Es>` must preserve three related but distinct identities:

```text
source declaration identity
    universe/...::Tuple (live canonical module)

structural type identity
    TypeData::Tuple(...)
    TypeData::TupleFromPacks(...)

runtime dispatch identity
    canonical Tuple runtime class / UniverseKey::Tuple
```

Preserve `Tuple -> Collection` (or the live equivalent runtime relation) even if ordinary `data` syntax does not expose a superclass clause.

Tuple values remain specialized Tuple objects. No generic `DataObject` migration.

### 7.7 Record bridge

`intrinsic data Record<R: RecordRow>` must preserve:

```text
TypeData::Record(RecordRowId)
RecordRow canonicalization
Record runtime class / UniverseKey association
Record -> Object relation
Record equality/hash semantics
Record::Map behavior
specialized Record runtime storage
```

Do not implement Record as `Applied(Nominal Record, [row])` when that would duplicate/replace the existing structural representation.

### 7.8 Behavior placement

Representation-bearing source declarations remain declaration syntax; behavior belongs in `impl` blocks under the C3 model.

Expected source organization:

```phalcom
intrinsic data Tuple<***Es>

impl Tuple<***Es> {
    intrinsic type Entries: TupleEntries
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type At<I: Int>: Type
    intrinsic type Map<F: Type -> Type>: Type

    // existing native/runtime and Phalcom behavior
}
```

A declaration being `intrinsic data` does not make every method compiler-intrinsic. Existing runtime operations such as `rawAt`, `rawSize`, `rawLabelAt`, or `rawValueAt` may remain `native` where that is their correct implementation origin.

Moving behavior must not change callable identity, native selector identity, source ownership, or runtime dispatch.

### 7.9 Native/Universe source census

Generalize the source/native census from class/enum assumptions to final semantic presentation categories. It must distinguish at least:

```text
runtime class-like declaration presentation
runtime algebraic data presentation
type-only static-domain presentation
behavior/member presentation
```

Public/exported terminology should prefer `Data`, `Case`, and `StaticDomain` where legacy `Enum`/`Variant` terminology would leak into the source model.

TypeOnly static-domain declarations must not enter:

```text
UNIVERSE_CLASS_RELATIONS
VM class tables
runtime descriptor census
```

Intrinsic runtime `data` anchors must still attach to canonical `UniverseKey` and runtime class relations.

### 7.10 Bootstrap isolation

Static facade discovery may affect semantic/source catalogs but must not by itself force runtime module execution.

Required rule:

```text
TypeOnly declaration discovery
    may affect semantic/source catalog

TypeOnly declaration discovery
    must not by itself require runtime initialization
```

Reuse existing Universe bootstrap/resource instrumentation. Record discovered units, root-reachable units, and executed units before/after the P3 facade migration where meaningful.

### 7.11 Core closed-sum migration

Migrate canonical Universe declarations:

```text
Option
Result
Ordering
```

from legacy `native enum` to canonical `native data` (or exact completed implementation-origin spelling) while preserving:

```text
DeclarationId
UniverseKey association
generic signature
case identity
selector identity
exact-case identity
associated-family identity
native runtime representation
reflection owner identity
module ownership
prelude/export visibility
semantic fingerprints where spelling-only migration is canonical-equivalent
```

Do not change `native` to `intrinsic` merely because runtime behavior is host-supplied.

### 7.12 Active-source migration

Perform a syntax-aware census of active repository-owned Phalcom source and migrate every active `enum` declaration to canonical `data` before parser retirement.

Migration categories:

```text
A. canonical Universe source
B. first-party examples
C. active test fixtures
D. editor golden fixtures
E. benchmarks/first-party sample packages
F. active authoritative specs/docs
```

Do not mass-rewrite:

```text
historical implementation records
historical ADRs/spec archives
Rust `enum` declarations
intentional negative fixtures
migration diagnostics/messages
private file names solely for cosmetic cleanup
```

Before T7 parser retirement, prove:

```text
no active repository-owned .ph source requires enum declaration syntax
```

except deliberately negative fixtures that prove rejection.

### 7.13 Source index final model

Final source identity must represent runtime sum data, type data, intrinsic static domains, and intrinsic runtime structural data without a current `Enum` language category.

A model equivalent to:

```text
SourceDeclarationKind::Data {
    phase: Runtime | TypeOnly,
    shape: Product | Sum | IntrinsicStructural,
}
```

is acceptable, but exact private representation is flexible.

The source index must attach canonical semantic targets to new static-domain/member categories. It must not reconstruct meaning from rendered source names.

### 7.14 LSP/tooling model

All of:

```text
hover
definition
references
completion
rename / prepareRename
diagnostics
semantic tokens
workspace symbols where relevant
```

must consume the compiler semantic snapshot/source index.

Rename rules:

- resolve one canonical semantic target;
- collect declaration/reference sites from compiler-owned reference data;
- verify rename category and editability;
- produce `WorkspaceEdit` only for semantic occurrences;
- decline safely on unresolved/unsupported targets;
- never text-search all matching names;
- if a new C3 target category lacks an identity, add it to the semantic identity/index model, not as an LSP-only ID.

### 7.15 Editor grammar

Update VS Code/TextMate classification for the final source language:

```text
intrinsic
data
type data
cases/constructors
```

If `Token::Enum` is retired, `enum` must not remain highlighted as a current Phalcom keyword.

User-visible grammar scope descriptions should prefer data/case terminology. Private legacy scope names may remain only when extension compatibility materially requires them.

### 7.16 Final enum retirement

Retire enum compatibility **after** active-source migration and a clean census.

Preferred endpoint:

```text
remove Token::Enum
remove lexer enum keyword recognition
remove parse_enum compatibility path
remove legacy enum-only AST compatibility field when no longer needed
convert enum syntax tests into rejection/non-keyword tests
```

If authoritative lexical policy retains an obsolete reserved token, it may exist only to produce a hard migration diagnostic and must never create a semantic declaration.

### 7.17 Metadata/interface identity

P3 may add stable source-facade/reflection association records when required, but it must not add a new semantic term encoding merely because a public facade has a new name.

In particular:

```text
TupleEntries
    must serialize through existing CompleteTypePack term/domain encoding

RecordRow
    remains row-owned stable structure

TypePack/LabeledTypePack
    retain existing stable pack-domain encodings
```

No raw:

```text
TypeId
StaticDomainId
RecordRowId
InferVarId
query/session-local source target IDs
```

may become portable identity.

If the metadata schema changes, version it explicitly and add compatibility fixtures. Do not silently reinterpret the previous schema.

### 7.18 Incremental model

Incremental invalidation must remain semantic/dependency-directed.

Spelling-only migration:

```text
enum X { ... } -> data X { ... }
```

must not cause semantic identity churn when both forms normalized to the same declaration before final parser retirement.

Final cold and incremental products must agree for:

```text
algebraic declaration edits
static-domain declaration edits
static arguments
associated domains
TupleEntries/member projections
source-index targets
metadata/import products
```

Do not restore parity with unconditional global invalidation.

### 7.19 Authoritative specification model

P3 must make `docs/specs/` internally coherent.

The final ownership model must describe:

```text
data
    Product + Sum runtime algebraic source

type data
    TypeOnly Product/Sum/static domains

intrinsic data / intrinsic type data
    canonical source facades over compiler/runtime-owned structural semantics
```

Current specs must not simultaneously say:

```text
objects/enums.md -> enum is canonical closed-sum syntax
objects/data-objects.md -> data is product-only
```

while a third document claims otherwise.

Update the `docs/specs/README.md` ownership map and all active dependent specs. Do not mass-rewrite historical implementation records merely to make them look current.

### 7.20 Runtime/resource boundary

The runtime should remain mostly unchanged.

P3 may change source/native presentation associations and reflection labels. It must not redesign:

```text
Tuple object layout
Record object layout
Option specialized representation
ADT runtime storage
GC model
VM bytecode model for unrelated reasons
```

No runtime value/class/opcode/solver exists solely for TypeOnly static domains.

---

## 8. Ownership boundaries

### 8.1 P3 owns

P3 owns:

- intrinsic source-facade registration for compiler-owned structural static domains;
- bodiless intrinsic facade validation where not already completed by P1;
- canonical source declarations for `TypePack`, `LabeledTypePack`, `TupleEntries`, and `RecordRow`;
- `Tuple::Entries` and public `TupleEntries` naming/presentation;
- migration of `Unit`, `Tuple`, and `Record` source presentation to intrinsic runtime data facades;
- preservation/adaptation of native source census and runtime-class associations under that migration;
- migration of `Option`, `Result`, `Ordering`, and remaining active first-party `enum` source to `data`;
- final active-source enum census;
- source-index category/target closure;
- LSP rename/prepareRename built on semantic identity;
- final hover/definition/references/completion/diagnostic/semantic-token consistency for C3 targets;
- VS Code grammar migration;
- final accepted `enum` parser/lexer retirement;
- authoritative `docs/specs/` closure;
- metadata/interface compatibility certification;
- incremental cold-parity certification;
- runtime-negative/static-isolation certification;
- Universe bootstrap/resource evidence;
- C3 checkpoint closure and final records.

### 8.2 P3 does not own

P3 does not own:

- new static-term carrier architecture;
- new `StaticArgument` architecture;
- new inference-variable family;
- new type-data unification architecture;
- new associated-domain identity/specialization model;
- moving `RecordRow` into `TypeId`;
- new pack decomposition or solver behavior;
- new runtime type-data object model;
- VM static evaluation/solving;
- trait-owned `type data` requirements;
- general trait static-domain generic redesign beyond the explicit P2 boundary;
- generic Tuple/Record `DataObject` migration;
- new Option runtime representation;
- release-wide unrelated Rust terminology cleanup;
- GC redesign;
- broad VM cleanup;
- formatter/linter redesign unless directly required by enum retirement;
- global historical documentation rewrite.

### 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| structural static-domain semantics | TypeStore/kind/row authorities | source facade, checker, LSP, metadata | new nominal builtin domains |
| facade declaration identity | canonical Universe source + semantic declaration table | reflection, source index, LSP | synthetic aggregate document |
| facade→builtin association | intrinsic semantic binding registry | semantic resolver, source index | string-name switch as authority |
| `TupleEntries` terms | CompleteTypePack authority | Tuple::Entries, packs, metadata | `TypeData::TupleEntries` |
| row identity | RecordRow subsystem | Record, generics, metadata | row-as-TypeId |
| Unit type identity | canonical Unit TypeStore authority | compiler/runtime/reflection | second nominal Unit type |
| Tuple structural identity | existing tuple TypeData family | checker/compiler/runtime | nominal Applied<Tuple,...> replacement |
| Record structural identity | `TypeData::Record(RecordRowId)` | checker/compiler/runtime | nominal Applied<Record,...> replacement |
| runtime class relations | canonical Universe/runtime tables | compiler/runtime/reflection | source-inferred replacement relation |
| runtime ADT/case identity | existing algebraic/variant semantic authority | compiler/runtime/LSP | keyword-derived second identity |
| source references | semantic source index | LSP rename/refs/definition | text-search resolver |
| stable metadata identity | `phalcom-type-meta` structural graph | imports/incremental | rendered strings/raw store IDs |
| enum final language status | lexer/parser/spec | diagnostics/editor | semantic compatibility backdoor |
| checkpoint state | `LANG007.C3-CHECKPOINT.md` | all plans/agents | plan-specific competing state file |

---

## 9. Global invariants

```text
INV-01 — P3 starts only from a completed, verified-enough P2 takeover satisfying §6.1.
INV-02 — TypeId remains the canonical handle for non-row static terms.
INV-03 — ProperTypeId remains the proof that a TypeId is KindId::TYPE.
INV-04 — RecordRow identity remains outside TypeId and keeps its dedicated solver.
INV-05 — StaticArgument remains the one heterogeneous application envelope.
INV-06 — AssociatedSurface remains the one :: namespace/collision authority.
INV-07 — TypePack facade maps to existing TYPE_PACK authority; no nominal duplicate.
INV-08 — LabeledTypePack facade maps to existing LABELED_TYPE_PACK authority; no duplicate.
INV-09 — TupleEntries is exactly the public facade over CompleteTypePack semantics.
INV-10 — RecordRow facade maps to the existing row domain; no fake TypeId or nominal static domain.
INV-11 — intrinsic static-domain facades have no runtime inhabitants/classes solely because of exposure.
INV-12 — Unit source migration preserves one canonical Unit type/runtime identity.
INV-13 — Tuple source migration preserves structural tuple types and specialized runtime representation.
INV-14 — Record source migration preserves TypeData::Record(RecordRowId) and specialized runtime representation.
INV-15 — intrinsic data presentation never implies generic DataObject storage.
INV-16 — moving behavior into impl blocks preserves callable/native selector identity.
INV-17 — Tuple::Entries normalizes through the existing complete-pack projection.
INV-18 — *T, **T, ***T remain semantically equivalent to Positionals/Labeled/Entries where valid.
INV-19 — Option/Result/Ordering source migration preserves declaration/case/native/runtime identity.
INV-20 — exact-case, GADT, exhaustiveness, refinement, and conformance behavior remain unchanged.
INV-21 — active first-party source is migrated before enum parser compatibility is removed.
INV-22 — final language acceptance has no semantic enum declaration route.
INV-23 — source-index identities are semantic products, not rendered-name reconstruction.
INV-24 — LSP rename edits only compiler-resolved semantic occurrences.
INV-25 — tooling presentation uses final data/type-data/case terminology where user-visible.
INV-26 — stable metadata never serializes raw query/store-local IDs as portable identity.
INV-27 — TupleEntries metadata reuses CompleteTypePack encoding.
INV-28 — cold and incremental final products converge without global invalidation.
INV-29 — TypeOnly declarations do not become runtime bootstrap roots solely by existing.
INV-30 — Universe source/native/runtime descriptor census remains exact.
INV-31 — no VM static value/object/opcode/solver is added.
INV-32 — authoritative docs/specs describe the same language model shipped by the implementation.
INV-33 — checkpoint verification truth never claims tests that were not run.
INV-34 — C3 FOCUSED_TESTED does not imply RELEASE_COMPLETE.
```

Every task and gate below must preserve these invariants.

---

## 10. Non-goals

P3 must not expand into:

```text
release-wide unrelated refactoring
renaming every Rust Enum/Variant identifier
runtime type-data reification
general compile-time evaluator
trait-owned type-data requirements
trait generic static-domain redesign beyond the completed P2 boundary
new pack features
new row solver
new Tuple/Record runtime storage
new Option runtime representation
GC redesign
VM bytecode cleanup unrelated to C3
global historical documentation rewrite
formatter/linter redesign unless required for enum removal
```

Do not opportunistically fold unrelated source-modernization work into the checkpoint closure patch.

---

## 11. Expected impact map

### 11.1 Front end

Likely:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/error.rs
phalcom-ast/tests/lexer.rs
phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/parser.rs
```

Reason: bodiless intrinsic facade syntax if still incomplete, final enum retirement, rejection diagnostics, and final keyword classification.

### 11.2 Semantic/type model

Likely:

```text
phalcom-semantic/src/types/**
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/source_index/**
phalcom-semantic/src/semantic_shard.rs
intrinsic/type-member normalization paths
associated target/reference paths where needed
```

Reason: intrinsic facade binding, TupleEntries/Entries, source identity, final declaration categories.

Expected semantic changes should be narrow. A large new `TypeStore` representation is a predecessor failure.

### 11.3 Metadata/modules/incremental

Likely:

```text
phalcom-type-meta/**
phalcom-modules/**
phalcom-semantic metadata/import/incremental paths
```

only if facade/source identity or stable source categories require explicit persistence changes.

### 11.4 Core/Universe/runtime association

Likely:

```text
phalcom-core/src/native/source.rs
Universe source/catalog modules
runtime class-relation tables
source/native descriptor generation/census
compiler declaration dispatch where source presentation changed
```

Runtime object-layout code should usually not need redesign.

### 11.5 LSP/editor

Likely:

```text
phalcom-lsp/src/backend.rs
phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
references/definition handlers or shared target resolver
new/updated rename tests

tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
editor golden fixtures
```

### 11.6 Source corpus

Migration categories:

```text
canonical Universe .ph source
active examples
active test fixture .ph files
editor golden examples
benchmarks/sample packages that are compatibility fixtures
generated/embedded source fixtures owned by repository
```

T1 must produce the exact completed-P2 path manifest before T5/T7.

### 11.7 Authoritative specs/state

Expected:

```text
docs/specs/README.md
docs/specs/objects/data-objects.md
docs/specs/objects/enums.md or replacement/renamed owner
docs/specs/objects/impl.md
docs/specs/objects/values.md
docs/specs/objects/traits.md where case examples exist
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/associated-types.md
docs/specs/generic-constraints.md
docs/specs/structural-products.md
docs/specs/universe.md
docs/specs/callables/families.md / references.md where associated constructors are discussed

LANG007.C3-CHECKPOINT.md
LANG007 PROGRAM.md / STATUS.md
P3 walkthrough
P3 handoff
```

### 11.8 Unexpected-touch rule

Adjacent helpers/tests may be edited when mechanically necessary.

STOP AND CONSULT before entering a materially different subsystem if doing so changes semantic ownership, identity, runtime representation, metadata architecture, inference architecture, or checkpoint acceptance scope.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

- the P2 predecessor contract;
- `TypeId` / `ProperTypeId` boundary;
- row ownership and `RecordRowId` separation;
- `StaticArgument` as the heterogeneous envelope;
- `AssociatedSurface` as the associated namespace owner;
- the mapping `TupleEntries == CompleteTypePack`;
- the rule that TypePack/LabeledTypePack/TupleEntries/RecordRow facades do not create nominal duplicate domains;
- the rule that intrinsic static facades do not create VM classes/values;
- existing Unit/Tuple/Record semantic and runtime identity;
- existing specialized Tuple/Record runtime representation;
- existing Option/Result/Ordering runtime identity/representation;
- migration-before-enum-retirement ordering;
- semantic-target-based LSP rename architecture;
- final absence of an accepted semantic `enum` declaration path;
- no runtime static solver/evaluator;
- no global-invalidation shortcut for incremental correctness;
- truthful checkpoint/verification state.

### 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- private helper/type names;
- whether intrinsic builtin registration uses enums, tables, static maps, or equivalent bounded structures;
- file-local decomposition;
- exact Universe module placement after T1 validates stable module/catalog constraints;
- exact representation of normalized source declaration categories;
- exact LSP helper decomposition and edit construction;
- exact grammar scope names when external extension compatibility requires retention;
- exact test fixture placement and filter syntax;
- whether legacy private `Enum`/`Variant` implementation names are renamed when doing so is purely mechanical and low risk;
- exact metadata schema number if a schema change is actually required.

Mechanical flexibility never authorizes a second semantic source of truth.

### 12.3 VERIFY-FIRST

| Assumption | Verify at P3 start | If false |
|---|---|---|
| P1/P2 are complete and their walkthroughs/handoffs exist | checkpoint + files + git revision | STOP; predecessor corrective work |
| P2 satisfies every §6.1 item | P2 handoff + live symbols/tests | STOP AND CONSULT |
| `CompleteTypePack` remains canonical complete-entry domain | TypeStore/kind/projectors | STOP AND CONSULT if redesigned |
| `project_complete` or equivalent exists | projection/type-member code | mechanically adapt name; consult if semantics absent |
| Unit/Tuple/Record canonical identities still match planning assumptions | TypeStore + Universe/runtime | adapt mechanics; consult on identity change |
| native source census still exists | phalcom-core/native source index | adapt mechanics; consult if ownership replaced |
| source index exposes canonical target/reference identities | source_index + P2 handoff | STOP if missing due to P2 gap |
| LSP can retrieve compiler targets/references | backend/handlers | adapt mechanics; consult if a second resolver seems necessary |
| active `enum` source remains finite/censusable | syntax-aware search | record exact manifest; consult if generated external source complicates acceptance |
| metadata can reuse CompleteTypePack encoding | type-meta | STOP if new TupleEntries term encoding seems necessary |
| bootstrap instrumentation exists | Universe/native tests | adapt exact metric helpers; add bounded instrumentation if absent |
| current authoritative lexical decision does not reserve `enum` | specs/accepted decisions | choose true retirement; consult on conflict |

---

## 13. Global STOP / CONSULT triggers

A triggered consultation cannot be self-waived.

Stop editing and prepare a compact incident packet if any of the following occurs:

1. Completed P2 does not satisfy the hard predecessor contract.
2. `TupleEntries` cannot map exactly onto CompleteTypePack semantics.
3. Exposing TypePack/LabeledTypePack/TupleEntries requires new canonical term variants.
4. Exposing RecordRow requires moving row identity into TypeId.
5. `intrinsic data Tuple` appears to require ordinary generic DataObject representation.
6. `intrinsic data Record` appears to require replacing `TypeData::Record`.
7. `intrinsic data Unit` would create a second Unit type identity.
8. Intrinsic static-domain source anchors appear to require VM classes or runtime values.
9. TypeOnly source declarations force runtime initialization and no local bootstrap fix preserves the accepted module architecture.
10. Moving Tuple/Record/Unit behavior to impl blocks changes runtime/native member identity.
11. Option/Result/Ordering migration changes declaration/case/native identity.
12. Removing enum compatibility breaks accepted data/GADT behavior after one serious correction.
13. Final enum lexical status conflicts with an authoritative accepted lexical decision.
14. LSP rename cannot be implemented from canonical source/reference targets without a second semantic resolver.
15. Source-index identity for an associated/static target is missing because P2 failed to publish one.
16. Stable metadata needs a new static term representation solely for TupleEntries.
17. Runtime representation must be redesigned to satisfy source facade semantics.
18. A VM static operation becomes necessary.
19. Cold/incremental equality cannot be restored without global invalidation.
20. A C1/C2/C3 invariant must be weakened for tests to pass.
21. Two plausible intrinsic-domain registration architectures have materially different semantic ownership consequences.
22. The same nontrivial semantic failure survives one serious corrective attempt.
23. Authoritative specs and accepted C3 decisions materially conflict.
24. Checkpoint closure would require claiming unrun verification as passed.
25. The patch becomes dominated by new type-system architecture instead of integration/migration/closure.
26. Active source cannot be migrated away from `enum` without changing accepted semantics.
27. Native/runtime source census can only pass by excluding declarations it previously certified.
28. A schema change would encode source spelling as stable semantic identity.

Incident packet should contain:

```text
Plan/task/gate:
Starting revision:
Observed:
Expected invariant:
Minimal reproducer:
Relevant source symbols:
Relevant diff:
Tests run + exact result:
Hypothesis:
Two plausible fixes, if applicable:
Decision required:
```

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

Examples: imports, renamed helpers, pattern exhaustiveness, test filter drift, borrow cleanup, stale path assumptions.

### 14.2 Semantic failures

Before editing, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure. If it persists, STOP AND CONSULT.

### 14.3 Architectural failures

Zero speculative architecture-fix attempts. Consult immediately.

Never rerun an unchanged failing test unless something relevant to the failure changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required evidence |
|---|---|---|---|
| CV-01 | predecessor takeover | INV-01–06 | P1/P2 completion + hard contract audit |
| CV-02 | static facade identity | INV-07–11 | source anchors map to existing kinds/row authority |
| CV-03 | bodiless intrinsic validation | INV-07–11 | registered succeeds; unregistered/user misuse rejects |
| CV-04 | TupleEntries canonicalization | INV-09,17,18,27 | Entries uses CompleteTypePack/project_complete |
| CV-05 | structural member preservation | INV-17,18 | Positionals/Labeled/At/Map unchanged |
| CV-06 | Unit identity | INV-12,15 | empty tuple, Unit type/runtime/source association unchanged |
| CV-07 | Tuple identity/runtime | INV-13,15,16 | structural TypeData and runtime class/storage preserved |
| CV-08 | Record identity/runtime | INV-14,15,16 | row-backed TypeData and runtime class/storage preserved |
| CV-09 | native source census | INV-16,30 | source selectors/descriptors/runtime descriptors exact |
| CV-10 | bootstrap isolation | INV-11,29 | static facade alone does not execute extra Universe units |
| CV-11 | Option migration | INV-19,20 | identity/runtime exactness under `data` |
| CV-12 | Result/Ordering migration | INV-19,20 | identity/runtime exactness under `data` |
| CV-13 | active enum census | INV-21,22 | zero active positive `.ph` enum declarations before retirement |
| CV-14 | ADT/GADT preservation | INV-19,20 | exact-case/refinement/exhaustiveness/conformance suites |
| CV-15 | final enum rejection | INV-21,22 | parser/lexer final behavior + negative fixtures |
| CV-16 | P2 static application | INV-02–06 | representative heterogeneous/static inference regressions |
| CV-17 | associated type data | INV-06,23 | owner specialization/collision/trait distinction preserved |
| CV-18 | metadata stability | INV-26,27 | round trip, schema, no raw IDs, facade mapping |
| CV-19 | import/source identity | INV-23,26 | cross-module static/associated/intrinsic references |
| CV-20 | incremental parity | INV-28 | cold vs incremental equality for C3 matrix |
| CV-21 | LSP semantic identity | INV-23–25 | hover/def/refs/completion consume semantic targets |
| CV-22 | rename | INV-23,24 | declaration+refs edits, unsupported target decline, no text fallback |
| CV-23 | editor syntax | INV-22,25 | data/type data/intrinsic/case highlighting; enum not current keyword |
| CV-24 | runtime-negative boundary | INV-03,11,31 | no static inhabitants/classes/opcodes/solver |
| CV-25 | runtime representation | INV-12–15,19 | Tuple/Record/Option representations unchanged |
| CV-26 | specification coherence | INV-32 | docs/specs ownership map and references consistent |
| CV-27 | performance/complexity | INV-27–30 | no algorithmic regression; bootstrap measurements |
| CV-28 | workspace integration | all | final whole-workspace build/check once |
| CV-29 | lifecycle truth | INV-33,34 | checkpoint/walkthrough/handoff accurately record evidence |

Do not mechanically add GC/concurrency tests unless a P3 change actually touches those mechanisms.

---

## 16. Verification execution budget

### 16.1 BUILD MODE

Use only discriminating feedback for the task being implemented.

Typical T0/T1 commands:

```sh
cargo test -p phalcom-ast <specific filter>
cargo test -p phalcom-semantic --test semantic -- <specific filter>
cargo test -p phalcom-core <specific filter>
cargo test -p phalcom-lsp <specific filter>
```

Adapt exact test binary/filter syntax to the completed-P2 tree.

During BUILD mode do **not** reflexively run:

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

and do not run broad unrelated corpora.

### 16.2 STABILIZE MODE

At coherent task/gate boundaries run owning subsystem suites required by the gate:

```text
AST/front-end focused suite
semantic C3/static/type-member suites
core Universe/native/ADT/structural runtime suites
LSP crate + C3 request suites
metadata/incremental suites
editor grammar lane if maintained
```

### 16.3 CERTIFY MODE

P3 is checkpoint closure, so one deliberate broader matrix is required.

Mandatory final compile breadth:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
```

A repository-preferred equivalent `cargo check --workspace --all-targets` may replace the build only if it exercises all required generated/LSP/extension integration targets; otherwise run explicit builds for those targets.

Mandatory checkpoint-specific breadth includes at least:

```text
phalcom-ast full crate tests
phalcom-type-meta full crate tests

phalcom-semantic:
    algebraic/data suites
    type model/type members/static application
    inference/substitution
    associated resolution
    metadata
    source index
    incremental

phalcom-core:
    Universe source/native census
    algebraic-data runtime
    native ADT runtime
    Option/Result/Ordering
    Tuple/Record/Unit
    relevant language corpus

phalcom-lsp:
    full crate tests + C3-specific requests

VS Code extension:
    grammar/unit/e2e lane when maintained and available
```

### 16.4 Optional release-strength evidence

These are useful but not automatically required for `C3 COMPLETE / IMPLEMENTED / FOCUSED_TESTED`:

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

If run, classify unrelated failures. Passing them does not by itself authorize `RELEASE_COMPLETE`; failing unrelated baselines does not automatically block C3 if all required C3 gates are satisfied and the failures are demonstrably outside acceptance.

---

## 17. Baseline / unrelated failure policy

Classify unexpected failures:

```text
A — definitely caused by P3/C3
B — probably caused by P3/C3
C — unclear
D — clearly unrelated/baseline
```

Policy:

- A/B: active implementation responsibility; C3 cannot close until fixed or architecturally escalated.
- C: one bounded causal classification pass. If it overlaps C3 acceptance and remains unclear, closure remains blocked. If demonstrably outside acceptance, record/defer.
- D: record in checkpoint and continue immediately.
- never weaken a C3 assertion, exclude a required target, or rewrite the expected behavior merely to make a broad suite green.
- do not consume adviser compute on D unless it blocks a required acceptance gate.

---

# 18. Tasks

## T1 — Certify the completed-P2 takeover and build the exact migration manifest

### Purpose

Convert the future completed-P2 repository into a verified P3 execution baseline, prove the hard predecessor contract, activate P3 lifecycle state, and produce an exact syntax-aware migration/certification manifest before source edits begin.

### Preconditions

- P1 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- P2 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- P1 and P2 walkthroughs/handoffs exist.
- working tree can be inspected without destroying unrelated work.

### Consumes

- completed P2 handoff/walkthrough;
- C3 checkpoint/guidance;
- this plan and P3 requirements analysis;
- live `git status`, HEAD, metadata version, baseline ledger;
- live Universe/source catalogs and test conventions.

### Produces

A verified execution baseline containing:

```text
starting revision
worktree status
P1/P2 final states
verified §6.1 predecessor contract
live stable-interface symbol map
current metadata schema
actual baseline failures
exact Universe paths for TypePack/LabeledTypePack/TupleEntries/RecordRow
exact Unit/Tuple/Record source anchors
exact Option/Result/Ordering source anchors
exact active `.ph` enum declaration census
exact source-index declaration/target categories
exact LSP request registration/test locations
exact VS Code grammar/test lane
bootstrap measurement baseline
```

Do not create a competing `implementation-state.md`. Put durable lifecycle facts in `LANG007.C3-CHECKPOINT.md`; retain the detailed migration manifest in plan execution notes/walkthrough inputs or a bounded migration manifest only if repository workflow already has a canonical place for such manifests.

### Files and symbols

**Read:**

```text
AGENTS.md
P1/P2 walkthroughs/handoffs
LANG007.C3-CHECKPOINT.md
LANG007.C3-GUIDANCE.md
TypeStore/kind/static application symbols named by P2 handoff
source_index target/reference model
phalcom-core native/Universe source census
Universe package/source catalog
phalcom-lsp backend/request registrations/tests
tools/vsphalcom grammar/tests
docs/specs/README.md
```

**Modify:**

```text
LANG007.C3-CHECKPOINT.md
PROGRAM.md / STATUS.md only as needed to mark P3 active and P2 completed truthfully
```

### Required implementation shape

1. Run `git status --short` and record HEAD.
2. Read P2 handoff first; verify live symbols instead of re-deriving P2 architecture.
3. Check every §6.1 item against live code and focused predecessor evidence.
4. Perform syntax-aware census for active Phalcom `enum` declarations. Partition results into A–H categories from §7.12.
5. Identify canonical Universe module placement candidates for structural static facades. Prefer existing structural modules; do not invent synthetic aggregate source documents.
6. Locate source/native/runtime census and bootstrap measurement hooks.
7. Confirm current LSP semantic target/reference APIs and whether rename methods are already partially present.
8. Record metadata schema and any P2 compatibility requirements.
9. Update checkpoint ledger: P2 completed, P3 active, starting revision, inherited baseline ledger.

### Forbidden approaches

- no feature edits before predecessor certification passes;
- no assumption that planning-baseline private names still exist;
- no global textual `enum` replacement;
- no broad workspace test solely to discover the repository;
- no creation of parallel checkpoint-state files.

### Test changes required

None unless a missing bounded census/measurement helper must be added to make later gates reproducible. If such helper changes semantic ownership, STOP AND CONSULT.

### Tests to run now

Run only predecessor smoke checks required to prove the handoff still describes the live revision. Prefer already-named focused tests from the P2 handoff.

Do **not** rerun all P1/P2 suites unless the handoff is stale or the live revision differs materially.

### Tests explicitly deferred

All P3 feature/runtime/LSP/corpus tests.

### Acceptance

T1 passes when:

```text
[ ] P1/P2 lifecycle states are confirmed
[ ] every §6.1 predecessor contract item is true
[ ] completed-P2 revision is recorded
[ ] migration census/manifest is exact enough to execute T2–T8
[ ] actual baseline failures are recorded
[ ] bootstrap/resource baseline is available
[ ] checkpoint says P3 active
```

### Local STOP / CONSULT triggers

- any §6.1 item is false;
- P2 semantic identity is missing from source index;
- CompleteTypePack was replaced by an incompatible architecture;
- active-source migration cannot be distinguished from historical/negative material;
- metadata state is materially inconsistent with P2 handoff.

### Checkpoint update

Mandatory. Record P3 start revision, predecessor verification, actual baseline ledger, and any consultation.

---

## T2 — Implement intrinsic structural static-domain facade registration and canonical source anchors

### Purpose

Expose `TypePack`, `LabeledTypePack`, `TupleEntries`, and `RecordRow` through canonical bodiless `intrinsic type data` declarations bound directly to existing compiler authorities, with no new semantic term universe and no runtime footprint.

### Preconditions

- T1/G1 passed.
- P2 domain/application/source identity is stable.
- canonical Universe module placement is known.

### Consumes

- existing algebraic declaration/provenance model;
- existing builtin `KindId`s and row authority;
- completed-P2 source-index/declaration identity;
- Universe source/catalog mechanism.

### Produces

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow
```

plus a semantic binding registry that maps those declaration identities to:

```text
TYPE_PACK
LABELED_TYPE_PACK
COMPLETE_TYPE_PACK
RECORD_ROW
```

without allocating nominal structural-builtin `StaticDomainId`s.

### Files and symbols

**Read/modify as live paths require:**

```text
phalcom-ast parser/AST only if bodiless intrinsic declarations are still incomplete
phalcom-semantic algebraic declaration/intrinsic registration
phalcom-semantic kind/domain resolver
phalcom-semantic source index
Universe source/catalog files selected by T1
phalcom-core native source census/bootstrap-root classification when declaration phase affects it
relevant semantic/parser/core tests
```

### Required implementation shape

1. If parser/AST already supports bodiless intrinsic declarations from completed P1, reuse it; do not add a parallel syntax node.
2. If still mechanically incomplete but semantically accepted, complete the declaration-only form for `intrinsic data` / `intrinsic type data` without permitting ordinary user bodiless `data`/`type data`.
3. Add bounded compiler-known binding descriptors keyed by canonical declaration identity.
4. Domain resolution for the four source declarations returns existing builtin kinds/row authority.
5. Source index records the declarations as semantic static-domain targets with canonical module/source identity.
6. TypeOnly declarations do not become runtime class relations, VM classes, or runtime bootstrap roots solely because they are present.
7. Public reflection/presentation may show `TupleEntries` for the complete-pack domain without changing the internal term identity.
8. Remove source spelling as the semantic key where the new declaration identity can replace it.

### Forbidden approaches

```text
new StaticDomainId(TypePack)
new StaticDomainId(LabeledTypePack)
new StaticDomainId(TupleEntries)
new StaticDomainId(RecordRow)
new TypeData::TupleEntries
row-as-TypeId
VM class for static facade
runtime object for static facade
synthetic generated aggregate source module
name-string registry as canonical identity
```

### Test changes required

Add focused tests for CV-02, CV-03, CV-10, CV-19:

- each source facade resolves to exact existing domain identity;
- `TupleEntries` reports complete-pack kind;
- `RecordRow` stays row-owned;
- ordinary unregistered/user bodiless `type data` fails as specified;
- no runtime class/census entry for TypeOnly facades;
- source definition target points to canonical authored declaration;
- adding static facades alone does not increase executed Universe units unexpectedly.

### Tests to run now

Run parser tests only if parser changed; otherwise semantic/domain/source-index focused tests plus the smallest Universe bootstrap/census test covering TypeOnly roots.

### Tests explicitly deferred

Full workspace, runtime ADT suites, LSP full crate, enum migration corpus.

### Acceptance

```text
[ ] four canonical source declarations exist
[ ] each maps one-to-one to existing authority
[ ] no duplicate term/domain representation exists
[ ] source index can target each declaration
[ ] no runtime class/value/bootstrap side effect is introduced
```

### Local STOP / CONSULT triggers

- registration appears to require a new kind/term representation;
- RecordRow cannot be exposed without fake TypeId representation;
- static facade source identity cannot coexist with builtin domain identity;
- bootstrap semantics require runtime initialization solely for TypeOnly declaration presence.

### Checkpoint update

Record the durable intrinsic-static-facade mapping after G2 passes.

---

## T3 — Add TupleEntries and canonical structural type-member anchors

### Purpose

Expose `Tuple::Entries` as the complete tuple-entry projection, prove pack/member equivalence, and converge structural intrinsic member lookup on canonical declaration/member identities rather than source strings.

### Preconditions

- T2/G2 passed.
- `TupleEntries` declaration identity resolves to `COMPLETE_TYPE_PACK`.

### Consumes

```text
existing CompleteTypePack canonical term
project_complete / project_positionals / project_labeled
existing TuplePositionals/TupleLabeled/TupleAt/TupleMap operations
completed-P2 type-member domain/argument architecture
AssociatedSurface/source-index identity
```

### Produces

A canonical intrinsic member equivalent to:

```phalcom
intrinsic type Entries: TupleEntries
```

and final source anchors for:

```text
Tuple::Entries
Tuple::Positionals
Tuple::Labeled
Tuple::At
Tuple::Map
TypePack::Map
LabeledTypePack::Map
RecordRow::Map
Record::Map
```

where owned by the completed language surface.

### Files and symbols

**Read/modify:**

```text
phalcom-semantic type-member definitions/normalization
Tuple structural projection helpers
AssociatedSurface/intrinsic member registration
source index member targets
Universe Tuple/structural source declarations selected by T1
metadata only if member source identity needs stable association
focused type-member/pack tests
```

### Required implementation shape

1. Add one intrinsic member identity for Tuple Entries.
2. Normalize it by calling/reusing the existing complete-pack projection.
3. Ensure public result domain is `TupleEntries`/`COMPLETE_TYPE_PACK`.
4. Preserve all existing Positionals/Labeled/At/Map behavior.
5. Replace intrinsic semantic dispatch keyed on strings such as `"Tuple"` or `"TypePack"` with canonical declaration/domain/member identity where the live code still uses strings as authority.
6. Preserve symbolic behavior: valid unresolved/symbolic tuple forms must project symbolically using existing C2/P2 machinery rather than being rejected or eagerly materialized.
7. Keep metadata term encoding as CompleteTypePack.

### Forbidden approaches

- `TypeData::TupleEntries`;
- second projection algorithm;
- second pack solver;
- formatting-equality tests in place of semantic identity tests;
- source-name string comparison as canonical member dispatch;
- changing pack mode semantics (`*`, `**`, `***`).

### Test changes required

Add CV-04/CV-05 cases for:

```phalcom
(Int, String, name: Symbol)
```

and symbolic forms proving:

```text
*T   == T::Positionals
**T  == T::Labeled
***T == T::Entries
```

plus existing `At`/`Map` regressions and member source definition identity.

### Tests to run now

Focused semantic type-member/pack projection tests only.

### Tests explicitly deferred

Runtime Tuple migration, full metadata suite, LSP, workspace tests.

### Acceptance

```text
[ ] Tuple::Entries exists as canonical source/member identity
[ ] normalization delegates to CompleteTypePack projection
[ ] concrete and symbolic equivalences pass
[ ] existing structural members remain unchanged
[ ] metadata uses existing complete-pack representation
```

### Local STOP / CONSULT triggers

- `Tuple::Entries` requires a new semantic term kind;
- symbolic equivalence cannot be preserved with existing C2/P2 projection architecture;
- canonical member identity cannot replace a name-string seam without redesigning AssociatedSurface.

### Checkpoint update

Record `TupleEntries == CompleteTypePack` and `Tuple::Entries` as established after G3.

---

## T4 — Migrate Unit, Tuple, and Record to intrinsic runtime data facades without representation change

### Purpose

Replace class-style source presentation of compiler-owned structural runtime concepts with canonical `intrinsic data` declarations and `impl` behavior while preserving TypeStore identity, runtime class relations, native members, source reflection, and specialized storage.

### Preconditions

- T2/G2 and T3/G3 passed.
- intrinsic runtime facade registration path exists and is proven not to route through ordinary generic data storage.

### Consumes

```text
canonical Unit TypeStore/runtime identity
Tuple structural type family + UniverseKey/runtime class
Record structural row-backed type + UniverseKey/runtime class
native source census
runtime class relation tables
existing native/member implementations
```

### Produces

Source surfaces equivalent to:

```phalcom
intrinsic data Unit()

intrinsic data Tuple<***Es>
impl Tuple<***Es> { ... }

intrinsic data Record<R: RecordRow>
impl Record<R> { ... }
```

with existing runtime/member behavior preserved.

### Files and symbols

**Read/modify according to T1 manifest:**

```text
canonical Universe Unit/Tuple/Record .ph source
Universe package catalog/exposure if module placement changes
phalcom-core native/source census
runtime UniverseKey/class relation tables
compiler declaration dispatch for intrinsic runtime data anchors
phalcom-semantic structural type/declaration association
source index / reflection descriptor code
Tuple/Record/Unit runtime tests
```

### Required implementation shape

1. Migrate source representation only after semantic intrinsic binding is available.
2. Bind `Unit` source declaration to existing canonical Unit type/runtime authority.
3. Bind `Tuple` source declaration to existing structural tuple type family and runtime Tuple class.
4. Bind `Record` source declaration to existing `TypeData::Record(RecordRowId)` and runtime Record class.
5. Move behavior from class bodies to `impl` blocks when required by final grammar, retaining native/Phalcom implementation provenance per member.
6. Preserve runtime hierarchy (`Tuple -> Collection`, `Unit -> Object`, `Record -> Object`, or live equivalents) through canonical platform metadata rather than fabricating source superclass syntax.
7. Adapt source/native census so moved native members remain associated with their canonical source owner.
8. Ensure TypeOnly structural domains from T2 do not leak into runtime census.
9. Preserve package/catalog/prelude exposure and canonical source URIs.

### Forbidden approaches

```text
new nominal Unit type
Applied(Nominal Tuple, [CompleteTypePack]) replacing structural Tuple
Applied(Nominal Record, [RecordRow]) replacing structural Record
generic DataObject Tuple/Record/Unit storage
runtime superclass inference from ordinary data rules
re-minting member/callable identities after moving to impl
weakening source/native descriptor census
```

### Test changes required

CV-06–CV-10/CV-25:

- Unit == empty tuple canonicalization;
- Unit UniverseKey/source/runtime identity;
- Tuple structural type equality/formation before/after source migration;
- Tuple runtime class/supertype and storage behavior;
- Record structural row identity and runtime behavior;
- native members still discoverable/callable and source-indexed;
- exact native descriptor census;
- TypeOnly static facades excluded from runtime census;
- bootstrap counts remain behaviorally acceptable.

### Tests to run now

Focused semantic structural-type tests + core Unit/Tuple/Record/native-census tests.

### Tests explicitly deferred

Option/Result/Ordering migration, enum parser retirement, full LSP, full workspace.

### Acceptance

```text
[ ] canonical source uses intrinsic data for Unit/Tuple/Record
[ ] semantic/runtime identities are unchanged
[ ] behavior moved to impl where required without identity churn
[ ] specialized runtime representations are unchanged
[ ] runtime class relations/census remain exact
```

### Local STOP / CONSULT triggers

- ordinary data lowering becomes required for Tuple/Record/Unit;
- member identity changes merely because behavior moved to impl;
- runtime class relation cannot be preserved with intrinsic metadata;
- source/native census can pass only by ignoring formerly certified members.

### Checkpoint update

Record intrinsic runtime structural facade invariants after G4.

---

## T5 — Migrate Option/Result/Ordering and all active first-party enum source to data

### Purpose

Move canonical Universe closed sums and all active repository-owned positive Phalcom source off legacy `enum` spelling while preserving the mature runtime ADT/GADT/exact-case semantic identity and leaving compatibility parsing temporarily available until the census is clean.

### Preconditions

- T4/G4 passed.
- active enum migration manifest from T1 is current.
- parser still accepts legacy enum compatibility for migration-only purposes.

### Consumes

- unified P1 algebraic declaration authority;
- exact-case/GADT/case identity machinery;
- native Option/Result/Ordering runtime paths;
- active source census.

### Produces

```text
canonical Universe Option/Result/Ordering declared with data
all active positive first-party .ph enum declarations migrated to data
explicit negative fixtures separated from positive corpus
zero positive active enum declarations before T7
```

### Files and symbols

**Modify:**

All exact source files in T1 categories A–E/F that positively use `enum`.

Expected core semantic/runtime areas should change only if migration exposes a source-category assumption:

```text
Universe native source catalog/census
compiler declaration routing from canonical algebraic facts
source index declaration presentation
ADT/GADT tests
```

### Required implementation shape

1. Migrate `Option`, `Result`, `Ordering` first and run focused identity/runtime tests.
2. Preserve `native` implementation origin where runtime/native behavior remains host-supplied.
3. Preserve declaration/module/case/selector/exact-case/associated-family identities.
4. Migrate remaining active positive first-party `.ph` source category by category.
5. Do not alter case payload semantics, GADT result clauses, visibility, requirements/defaults, or impl/conformance behavior merely because the keyword changes.
6. Convert fixtures intentionally testing old syntax into clearly negative/migration fixtures; do not silently delete coverage.
7. Re-run a syntax-aware active-source census and record zero remaining positive enum declarations.

### Forbidden approaches

- global text replacement across repository;
- renaming Rust enums;
- rewriting historical records;
- changing `native` to `intrinsic` for Option/Result/Ordering without semantic reason;
- changing Option specialized runtime representation;
- re-minting case IDs due to source spelling;
- removing parser compatibility before the source census is clean.

### Test changes required

CV-11–CV-14:

```text
Option/Result/Ordering declaration identity
case/selector/exact-case identity
native representation behavior
construction/match/exhaustiveness
GADT result specialization
branch refinement
exact-case impl/conformance
root requirements/defaults
case visibility
active corpus compile/parse
semantic fingerprint stability where canonical-equivalent
```

### Tests to run now

After the three core Universe migrations, run focused ADT/native runtime tests. Then run the smallest active-source corpus lane that covers migrated fixtures.

Do not run full workspace after every source batch.

### Tests explicitly deferred

Final enum rejection tests (T7), full LSP/editor, final workspace build.

### Acceptance

```text
[ ] Option/Result/Ordering use canonical data source syntax
[ ] their semantic/runtime identities are preserved
[ ] all active positive first-party .ph enum declarations are migrated
[ ] mature ADT/GADT behavior remains green
[ ] only explicit negative/historical/non-Phalcom uses of "enum" remain
```

### Local STOP / CONSULT triggers

- any source-only migration changes semantic fingerprint/DeclarationId/case identity unexpectedly;
- runtime Option/Result/Ordering representation changes;
- GADT/exact-case behavior breaks after one serious correction;
- an active source form cannot be represented by accepted `data` syntax without semantic loss.

### Checkpoint update

Record core sum migration and clean positive-source census after G5.

---

## T6 — Close source index, reflection, LSP rename, semantic tooling, and editor terminology

### Purpose

Make user-facing source tooling consume final C3 semantic identities/categories, implement semantic rename/prepareRename, remove active `Enum` source-category leakage, and update semantic tokens/editor grammar to the final language model before lexer/parser retirement.

### Preconditions

- T2–T5 gates passed.
- canonical source identities exist for structural static/runtime facades and migrated data sums.
- active positive source already uses final syntax.

### Consumes

```text
semantic source target/reference index
final algebraic declaration phase/shape/provenance
intrinsic facade declaration identities
associated/member target identities
compiler semantic snapshot consumed by LSP
TextMate grammar and editor fixtures
```

### Produces

Final source/tooling behavior for:

```text
runtime data roots/cases
type data roots/cases
associated type data
intrinsic static domains
intrinsic runtime structural data
TupleEntries
Tuple::Entries
```

and working semantic:

```text
hover
definition
references
completion
prepareRename
rename
diagnostics
semantic tokens
workspace symbols where supported
```

### Files and symbols

**Read/modify:**

```text
phalcom-semantic/src/source_index/**
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/declarations.rs

phalcom-lsp/src/backend.rs
phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
shared definition/reference target helpers
LSP test modules

tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
editor golden fixtures/tests
```

### Required implementation shape

#### Source index

1. Remove `Enum` as a current source-language declaration category or normalize it to final data/sum categories.
2. Represent runtime sum data, type data, intrinsic static domains, and intrinsic runtime data with canonical semantic targets.
3. Ensure cases/constructors and structural static members have canonical declaration/reference identities.
4. Preserve source URI/range/module identity through migrations.

#### Reflection/presentation

1. Public source/reflection terminology reports `data`/`case`/`type data` semantics.
2. Internal ADT/runtime terms may remain legacy-named when not user-visible.
3. `TupleEntries` is the public name for complete tuple entry domain where appropriate.

#### LSP rename

1. Implement/complete `prepareRename` by resolving the canonical semantic target and validating renameability.
2. Implement `rename` by retrieving declaration + semantic reference locations for that target.
3. Generate one `WorkspaceEdit` over compiler-resolved source ranges.
4. Reject/decline unsupported synthetic/native/non-editable targets explicitly.
5. Deduplicate overlapping edits deterministically.
6. Never fall back to textual same-name search.

#### Semantic tokens/editor grammar

1. Final `data`, `type data`, `intrinsic` forms classify correctly.
2. Cases/constructors use final user-facing terminology.
3. `enum` may temporarily still be recognized by the parser until T7, but the editor should not advertise it as canonical syntax; after T7 it must not remain a current keyword.

### Forbidden approaches

- LSP-side parser/name resolver;
- text search rename;
- LSP-only static-domain identity;
- rendered-name reconstruction of semantic targets;
- preserving `SourceDeclarationKind::Enum` as the canonical public category;
- altering semantic identity solely to satisfy editor rendering.

### Test changes required

CV-21–CV-23:

For each applicable target type:

```text
data root
data case
type data
type-data case
associated type data
TupleEntries
Tuple::Entries
```

cover:

```text
hover
definition
references
completion
prepareRename success/rejection
rename declaration+references
rename unsupported/non-editable target
semantic tokens
diagnostics
editor grammar/golden highlighting
```

Include same-spelling-unrelated-symbol cases proving rename is semantic rather than textual.

### Tests to run now

Focused source-index tests, targeted LSP request tests, then LSP crate tests if focused cases pass. Run the maintained grammar/unit lane if available.

### Tests explicitly deferred

Final enum lexer/parser retirement and full checkpoint certification.

### Acceptance

```text
[ ] no user-facing current Enum declaration category remains
[ ] all final C3 target categories have canonical source identities
[ ] rename/prepareRename use semantic target/reference products only
[ ] hover/def/refs/completion agree on targets
[ ] semantic tokens/grammar reflect final C3 terminology
```

### Local STOP / CONSULT triggers

- rename needs a second resolver;
- semantic target/reference identity is missing due to P2 gap;
- native/generated source editability cannot be decided without changing semantic ownership;
- tooling can only represent a C3 target by parsing rendered type strings.

### Checkpoint update

Record source-index final category model and rename architecture after G6.

---

## T7 — Retire accepted enum syntax after the active-source census is clean

### Purpose

Remove the temporary legacy Phalcom `enum` declaration compatibility route, convert former positive compatibility tests into rejection/non-keyword tests, and align diagnostics/tokens/tooling with the final language rule.

### Preconditions

- T5/G5 clean active-source census passed.
- T6/G6 tooling recognizes final language categories.
- no active positive first-party source depends on enum syntax.
- authoritative accepted lexical decision has been checked.

### Consumes

- P1 temporary enum normalization path;
- current token/lexer/parser/AST compatibility fields;
- migrated source corpus;
- final diagnostics/editor terminology.

### Produces

Preferred endpoint:

```text
no Token::Enum
no lexer enum keyword recognition
no parse_enum compatibility route
no enum->Runtime+Sum semantic normalization path
no active AST legacy enum spelling field unless required solely for a hard obsolete-token diagnostic
negative tests proving enum declaration syntax is rejected
```

If the authoritative lexical spec explicitly retains `enum` as a reserved obsolete token, keep only the token/diagnostic path and still remove all semantic declaration construction.

### Files and symbols

**Modify likely:**

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/error.rs
phalcom-ast/tests/lexer.rs
phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/parser.rs
semantic token classification
VS Code grammar if any final enum keyword rule remains
active negative fixtures
```

### Required implementation shape

1. Re-run active positive-source enum census immediately before removal.
2. Apply lexical policy fixed by authoritative spec/decision.
3. Remove semantic compatibility path completely.
4. Preserve final `data` Product/Sum parser behavior.
5. Convert old enum compatibility tests to explicit rejection/non-keyword behavior.
6. Ensure migration diagnostic text, if retained, does not imply support.
7. Run full AST crate tests and directly affected semantic parsing tests.
8. Re-run source census proving no accidental positive fixture remains.

### Forbidden approaches

- removing enum parser before clean census;
- keeping a hidden semantic enum route for old tests;
- weakening data/GADT parser behavior;
- deleting legacy tests without replacement negative coverage;
- globally renaming private Rust enums.

### Test changes required

CV-13/CV-15 plus regression cases from CV-14:

```text
enum declaration rejected
data sum accepted
GADT data sum accepted
data empty sum/product distinctions preserved
singleton vs zero-arg constructor preserved
mixed body rejection preserved
semantic tokens/grammar no longer advertise enum keyword
```

### Tests to run now

```sh
cargo test -p phalcom-ast
```

plus focused semantic parser/algebraic suites affected by the token/AST removal and the smallest corpus parse gate.

### Tests explicitly deferred

Full metadata/incremental/runtime/resource certification and workspace build until T9.

### Acceptance

```text
[ ] active positive source census remains clean
[ ] no accepted semantic enum declaration path exists
[ ] final data syntax covers all accepted runtime sums/GADTs
[ ] old enum tests now prove rejection/retirement
[ ] editor/token classifiers no longer present enum as current syntax
```

### Local STOP / CONSULT triggers

- authoritative lexical decision conflicts with preferred true retirement;
- accepted data/GADT behavior regresses after one serious correction;
- parser removal exposes a hidden active positive source dependency.

### Checkpoint update

Record final enum compatibility state after G7.

---

## T8 — Close authoritative specifications and current documentation ownership

### Purpose

Make `docs/specs/` describe the actual final C3 language/platform model with one coherent ownership map, while preserving historical implementation records as history.

### Preconditions

- T2–T7 feature/tooling language behavior is stable.
- final enum lexical status is known.
- final source module placement and public names are fixed.

### Consumes

- final implementation behavior;
- accepted C3 requirements/guidance;
- current `docs/specs/README.md` ownership map.

### Produces

A coherent active specification corpus in which:

```text
data      owns runtime Product/Sum source semantics
type data owns TypeOnly static algebra/domains
intrinsic data/type data explain compiler/runtime structural facades
TupleEntries is public CompleteTypePack naming
associated type data is distinguished from trait associated types
enum is not described as a current canonical declaration form
```

### Files and symbols

Audit/update active references in:

```text
docs/specs/README.md
docs/specs/objects/data-objects.md
docs/specs/objects/enums.md or replacement
docs/specs/objects/impl.md
docs/specs/objects/values.md
docs/specs/objects/traits.md where case examples exist
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/associated-types.md
docs/specs/generic-constraints.md
docs/specs/structural-products.md
docs/specs/universe.md
docs/specs/callables/families.md / references.md where associated constructors are discussed
VM specs only where they incorrectly state source semantics
```

### Required implementation shape

1. Choose one owning spec for runtime closed-sum data semantics: a new/renamed algebraic-data owner or normalized former enums spec.
2. Make `objects/data-objects.md` and the sum owner cross-reference rather than contradict.
3. Update `type-level-domains.md` for user and intrinsic `type data`.
4. Update `type-members.md` / structural-products for TupleEntries and structural member equivalences.
5. Update `associated-types.md` to distinguish inherent associated `type data` from evidence-bound trait associated types.
6. Update `universe.md` for canonical structural facade declarations and runtime/source identity.
7. Update `README.md` ownership map and all active links.
8. Do not rewrite `docs/spec/`, historical implementation records, walkthroughs, or old ADRs merely to change terminology.

### Forbidden approaches

- adding a third C3 note while leaving two contradictory current owners;
- rewriting historical records as if they were current specs;
- documenting architecture that did not actually ship;
- claiming release certification.

### Test changes required

No code tests solely for prose. Add/adjust documentation link/spec-consistency tests if the repository maintains them.

### Tests to run now

Run repository documentation/spec link checks if maintained. Use targeted searches for contradictory current `enum`-as-canonical statements in `docs/specs/`.

### Tests explicitly deferred

Final full C3 certification gate in T9.

### Acceptance

```text
[ ] docs/specs ownership map is coherent
[ ] no active authoritative spec describes enum as canonical sum declaration
[ ] data Product/Sum semantics are current
[ ] type data / intrinsic facades / TupleEntries / associated type data are documented
[ ] historical trees remain historical
```

### Local STOP / CONSULT triggers

- two accepted authoritative specs materially disagree about final language behavior;
- implementation behavior differs from ratified C3 semantics;
- spec closure would require inventing an unimplemented public behavior.

### Checkpoint update

Record authoritative spec closure after G8.

---

## T9 — Execute C3 persistence, incremental, runtime-negative, resource, performance, and integration certification

### Purpose

Prove P1+P2+P3 together across the full C3 acceptance matrix and gather the evidence required for truthful checkpoint closure.

### Preconditions

- T2–T8 complete and their focused gates pass.
- no unresolved A/B failure remains.
- active-source enum census is clean and parser retirement is complete.
- authoritative specs are coherent.

### Consumes

All final C3 implementation and tests.

### Produces

A completed certification ledger for:

```text
syntax/algebraic shape
identity
ADT/GADT behavior
P2 static application/inference
associated domains
structural domains/members
metadata/imports
incremental parity
source index/LSP/editor
runtime representation
runtime-negative/static isolation
Universe bootstrap/resource behavior
complexity/performance expectations
workspace build integration
baseline classification
```

### Required certification matrix

#### Syntax/algebraic shape

Cover:

```text
data tuple product
data record product
data singleton sum
data comma singleton shorthand
data tuple-payload sum
data record-payload sum
data explicit zero-arg constructor
data empty product
data empty sum
GADT data sum
type data equivalents
intrinsic data facades
intrinsic type data facades
mixed-body rejection
enum declaration rejection
```

#### Identity

Prove:

```text
structurally equal runtime data declarations remain nominally distinct
two structurally equal type-data declarations remain distinct domains
equal static terms canonicalize
unequal domains/constructors do not
associated domain specialization includes owner environment
Option/Result/Ordering identities survive source migration
TupleEntries maps to CompleteTypePack
```

#### ADT/GADT

Re-run focused mature behavior:

```text
exact-case construction
exact-case impl
case-specific conformance
GADT result specialization
branch refinement
exhaustive matching
non-exhaustive diagnostics
redundant branch diagnostics
root requirement/default completeness
case visibility
```

#### P2 static application/inference

Use representative source such as:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

data Tensor<Element: Type, S: Shape>()

type V: Type = Tensor<Float, Shape::Vector(3)>
```

Cover:

```text
wrong-domain rejection
ConstInt argument
associated-domain argument
RecordRow argument
pack arguments
partial application
static aliases
first-order inference
constructor/domain refutation
symbolic preservation
```

#### Associated domains

Use representative:

```phalcom
class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

Cover `Parser::State`, cases, aliases, generic-owner specialization, collision behavior, and distinction from trait-associated types.

#### Structural domains

Cover:

```text
TypePack source anchor
LabeledTypePack source anchor
TupleEntries source anchor
RecordRow source anchor
Tuple::Entries
Tuple::Positionals
Tuple::Labeled
Tuple::At
Tuple::Map
TypePack::Map
LabeledTypePack::Map
RecordRow::Map
Record::Map
```

and `*`/`**`/`***` equivalences.

#### Persistence/imports

Cover:

```text
metadata round trip
schema validation
cross-module type-data import
cross-module associated domain
intrinsic static-domain import/reference
stable source declaration references
no raw IDs
```

#### Incremental

Run cold-vs-incremental parity over the final C3 edit matrix. At minimum include declaration phase/shape, type-data cases/payload, static binder/argument, associated owner/environment, TupleEntries/member source, and enum→data canonical migration fixtures where relevant to preserved fingerprints.

#### Tooling

Run final LSP/editor cases over all target categories, including final removed-enum diagnostic/rejection behavior.

#### Runtime

Prove:

```text
runtime product behavior unchanged
runtime sum behavior unchanged
special Option representation unchanged
Tuple/Record specialized runtime representation unchanged
Unit identity unchanged
no static runtime objects
no static VM solver
```

#### Performance/resource

Confirm:

```text
case duplicate detection remains expected O(n)
static term interning remains hash-consed
StaticArgument application remains proportional to argument graph
TupleEntries projection has CompleteTypePack projection complexity
associated lookup remains indexed by owner/base
metadata remains graph encoding without stringify/reparse
incremental invalidation remains dependency-directed
C2 pack flatness remains intact
```

Record Universe bootstrap:

```text
discovered units
root-reachable units
executed units
```

before/after P3 where meaningful. Explain any increase caused by source reorganization; a major increase solely due to TypeOnly facade presence is unacceptable.

### Commands / execution breadth

At certification run:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
cargo test -p phalcom-ast
cargo test -p phalcom-type-meta
```

Run the completed-P2 repository's exact C3-relevant semantic/core/LSP test binaries/suites established by T1 and prior tasks. Prefer explicit suite/filter commands recorded in the walkthrough.

Run VS Code grammar/unit/e2e lane if the repository maintains one and it is part of current supported verification.

Do not require `cargo test --workspace --all-targets` or workspace clippy unless the plan's affected-suite evidence is insufficient, repository policy requires it, or stronger release evidence is intentionally requested.

### Forbidden approaches

- marking PASS based on tests not run;
- skipping required C3 suites because unrelated workspace baselines exist;
- broad `#[ignore]`/filter exclusions to hide C3 failures;
- global invalidation to force incremental parity;
- replacing resource evidence with intuition;
- inferring `RELEASE_COMPLETE` from a successful checkpoint matrix.

### Acceptance

T9 passes only when every required coverage obligation CV-01–CV-28 has explicit evidence or an explicitly justified non-applicability, no unresolved A/B failure remains, and all C3-critical C-class failures are either resolved or proven outside acceptance.

### Local STOP / CONSULT triggers

- same semantic failure persists after one serious correction;
- cold/incremental parity requires global invalidation;
- runtime/static boundary is violated;
- performance/resource regression indicates TypeOnly facade execution cost;
- checkpoint can close only by weakening a required invariant.

### Checkpoint update

Mandatory after each coherent certification gate. Record commands, outcomes, what they prove, and baseline classifications—not every individual test invocation.

---

## T10 — Close P3 and LANG007.C3 lifecycle records truthfully

### Purpose

Convert successful implementation/certification into durable repository state: final checkpoint ledger, walkthrough, terminal handoff, program/status updates, and explicit next LANG007 action.

### Preconditions

- all required G1–G11 gates pass;
- no unresolved checkpoint blocker remains;
- final revision/worktree state is known.

### Consumes

- final diff;
- verification ledger;
- consultation/amendment ledger;
- baseline/deferred failures;
- bootstrap/performance/runtime-negative evidence;
- final active-source enum census;
- final metadata schema and tooling state.

### Produces

```text
LANG007.C3.P3-...-walkthrough.md
LANG007.C3.P3-handoff.md
updated LANG007.C3-CHECKPOINT.md
updated PROGRAM.md / STATUS.md
explicit next LANG007 action
```

### Required implementation shape

#### Checkpoint

Update plan ledger to:

```yaml
LANG007.C3.P1: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
LANG007.C3.P2: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
LANG007.C3.P3: COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

then, and only if all C3 acceptance conditions are true:

```yaml
LANG007.C3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

Record established invariants, final stable interface map, verification gates, deferred failures, consultations, and next action.

#### Walkthrough

Must record:

```text
final revision
starting revision
intrinsic static-domain declarations added
intrinsic runtime structural declarations migrated
TupleEntries mapping
structural member registrations
Universe closed-sum migrations
remaining active enum census
parser/lexer compatibility removal
source-index/LSP changes
rename implementation
VS Code grammar changes
specification migration
metadata/schema changes, if any
incremental evidence
runtime-negative evidence
Universe bootstrap/resource evidence
performance/complexity evidence
commands actually run
commands deliberately not run
baseline failures
consultations/amendments
deviations from plan
final P3 classification
final C3 classification
```

#### Handoff

Because P3 closes C3, the handoff freezes the completed C3 substrate for the next accepted LANG007 objective. It must state:

```text
data/type-data final syntax
canonical term/domain architecture
TupleEntries mapping
StaticArgument contract
associated-domain contract
final enum compatibility state
metadata schema state
runtime representation boundaries
tooling/source-index state
known deferred baselines
next accepted LANG007 objective
must-not-redesign invariants
```

### Forbidden approaches

- claiming `RELEASE_COMPLETE` without release evidence;
- writing a second checkpoint state file;
- claiming tests passed based on design intent;
- leaving `active_plan: P3` after completion;
- inventing P4 when no corrective work is needed;
- marking C3 complete while any truth-table item below is false.

### Tests to run now

No new tests merely because documentation is being written. Re-run only a gate invalidated by a final code/spec change.

### Acceptance

Lifecycle records are internally consistent, reflect the actual final revision/evidence, and leave the next agent able to continue without conversation history.

### Local STOP / CONSULT triggers

- a required gate was not actually run;
- residual failure classification is ambiguous and overlaps C3 acceptance;
- a late documentation review reveals implementation/spec contradiction;
- closure would require overclaiming verification.

### Checkpoint update

This task is the final checkpoint update.

---

# 19. Verification gates

## G1 — Completed-P2 takeover gate

**Purpose:** prove P3 is starting from the architecture it is allowed to consume.

Required evidence:

```text
P1 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P2 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
all §6.1 predecessor items verified
actual baseline ledger recorded
migration manifest recorded
```

Use only focused predecessor smoke commands from the P2 handoff where necessary.

**PASS means:** P3 may begin feature edits.

**FAIL means:** stop; predecessor corrective closure.

---

## G2 — Intrinsic static-domain facade gate

**Purpose:** prove the four structural static source declarations are facades over existing authorities with no runtime footprint.

Required:

```text
TypePack -> TYPE_PACK
LabeledTypePack -> LABELED_TYPE_PACK
TupleEntries -> COMPLETE_TYPE_PACK
RecordRow -> RECORD_ROW / row authority
no duplicate StaticDomainId/TypeData representation
no runtime class/value/bootstrap execution caused solely by TypeOnly facades
canonical source definition targets
```

Run focused semantic/domain/source-index/bootstrap tests.

---

## G3 — TupleEntries/member equivalence gate

**Purpose:** prove public naming/member surface is semantically identical to C2 complete-pack machinery.

Required:

```text
Tuple::Entries exists
Tuple::Entries uses complete projection
*T == T::Positionals
**T == T::Labeled
***T == T::Entries
existing At/Map behavior unchanged
symbolic cases remain symbolic
```

Run focused type-member/pack suites.

---

## G4 — Structural runtime facade/runtime census gate

**Purpose:** prove Unit/Tuple/Record source migration changes presentation, not canonical type/runtime identity.

Required:

```text
Unit identity + empty tuple canonicalization
Tuple structural TypeData + runtime class/storage/supertype
Record row-backed TypeData + runtime class/storage
native member identity after impl migration
source native selectors == generated descriptors == runtime descriptors
TypeOnly facades absent from runtime descriptor census
bootstrap counts acceptable
```

Run focused semantic + phalcom-core structural/native census tests.

---

## G5 — Canonical closed-sum + active source migration gate

**Purpose:** prove all positive first-party source has moved to `data` and mature ADT/GADT behavior/identity is preserved before parser retirement.

Required:

```text
Option/Result/Ordering migrated
all active positive .ph enum declarations migrated
DeclarationId/case/selector/exact-case/native identity preserved
ADT/GADT/exhaustiveness/refinement/conformance focused suites pass
positive active-source enum census == 0
```

Do not remove enum parser compatibility until this gate passes.

---

## G6 — Source-index/LSP/editor gate

**Purpose:** prove tooling consumes final semantic identities/categories.

Required:

```text
no current public Enum declaration category
hover/definition/references/completion agree
prepareRename/rename operate on semantic target/reference index
same-spelling unrelated symbols are untouched
unsupported/non-editable targets decline cleanly
semantic tokens and TextMate grammar reflect final syntax
TupleEntries/Tuple::Entries are navigable/toolable
```

Run focused source-index + full relevant phalcom-lsp tests + maintained grammar lane.

---

## G7 — Final enum retirement gate

**Purpose:** prove legacy declaration syntax is no longer accepted and final data syntax remains complete.

Run:

```sh
cargo test -p phalcom-ast
```

plus focused semantic algebraic/parser tests and active-source census.

Required:

```text
no semantic enum declaration route
lexical policy matches accepted spec
old compatibility tests converted to rejection/non-keyword tests
data Product/Sum/GADT coverage remains green
active positive enum census remains zero
```

---

## G8 — Authoritative specification coherence gate

**Purpose:** prove active specs describe the implementation and have one coherent ownership map.

Required:

```text
README ownership map updated
runtime sums owned by data/algebraic-data spec
no current spec calls enum canonical
TypeOnly/intrinsic static domains documented
TupleEntries/member equivalences documented
associated type data distinguished from trait associated types
Universe structural facades documented
```

Run documentation/link checks if maintained and targeted contradiction searches over `docs/specs/`.

---

## G9 — Metadata/import/incremental gate

**Purpose:** prove stable persistence and cold/incremental identity after final source/tooling migration.

Required:

```text
phalcom-type-meta full crate tests pass
metadata round-trip for C3 terms/domains/sources
TupleEntries uses CompleteTypePack encoding
cross-module static/associated/intrinsic source references preserve identity
no raw local IDs serialized
cold/incremental C3 matrix parity
no global invalidation workaround
```

---

## G10 — Runtime-negative/resource/performance gate

**Purpose:** prove P3 exposure did not move TypeOnly semantics into runtime or regress platform resources.

Required:

```text
no runtime TypeOnly objects/classes
no VM static opcodes/solver
ProperTypeId rejects non-Type static terms
Tuple/Record/Option representations unchanged
Unit identity unchanged
bootstrap discovered/reachable/executed counts recorded
no major TypeOnly-only execution increase
TupleEntries projection complexity unchanged
static interning/application/associated lookup/incremental complexity expectations preserved
```

---

## G11 — Final workspace integration and C3 closure gate

**Purpose:** prove the completed patch compiles as one repository and all C3-owned affected suites are green or truthfully classified.

Run:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
cargo test -p phalcom-ast
cargo test -p phalcom-type-meta
```

plus the exact semantic/core/LSP/editor suites accumulated in G2–G10.

`cargo test --workspace --all-targets` and workspace clippy are optional release-strength evidence unless repository policy or new evidence makes them necessary.

**PASS means:** T10 may record P3 and C3 as `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.

**FAIL means:** classify; do not close C3 while any A/B or acceptance-overlapping unresolved C remains.

---

## 20. Final focused acceptance mapping

At completion the walkthrough must provide actual results for this table.

| Coverage ID | Owning task/gate | Required outcome |
|---|---|---|
| CV-01 | T1 / G1 | PASS |
| CV-02 | T2 / G2 | PASS |
| CV-03 | T2 / G2 | PASS |
| CV-04 | T3 / G3 | PASS |
| CV-05 | T3 / G3 | PASS |
| CV-06 | T4 / G4 | PASS |
| CV-07 | T4 / G4 | PASS |
| CV-08 | T4 / G4 | PASS |
| CV-09 | T4 / G4 | PASS |
| CV-10 | T2+T4 / G2+G4+G10 | PASS |
| CV-11 | T5 / G5 | PASS |
| CV-12 | T5 / G5 | PASS |
| CV-13 | T5+T7 / G5+G7 | PASS |
| CV-14 | T5+T9 / G5+G11 | PASS |
| CV-15 | T7 / G7 | PASS |
| CV-16 | T9 / G9+G11 | PASS |
| CV-17 | T9 / G9+G11 | PASS |
| CV-18 | T9 / G9 | PASS |
| CV-19 | T2+T9 / G2+G9 | PASS |
| CV-20 | T9 / G9 | PASS |
| CV-21 | T6 / G6 | PASS |
| CV-22 | T6 / G6 | PASS |
| CV-23 | T6+T7 / G6+G7 | PASS |
| CV-24 | T9 / G10 | PASS |
| CV-25 | T4+T5+T9 / G4+G5+G10 | PASS |
| CV-26 | T8 / G8 | PASS |
| CV-27 | T9 / G10 | PASS |
| CV-28 | T9 / G11 | PASS |
| CV-29 | T10 | PASS |

A coverage item may be marked N/A only if the live completed-P2 architecture makes it genuinely inapplicable and the walkthrough explains why without weakening checkpoint acceptance.

---

## 21. Performance/resource evidence

P3 has real performance/resource obligations because it changes Universe catalog/source discovery and structural facade presentation.

Required evidence:

### 21.1 Complexity

Confirm no asymptotic regression in:

```text
case duplicate detection            expected O(n)
static term interning               hash-consed
StaticArgument application          proportional to argument graph
TupleEntries projection             same order as CompleteTypePack projection
associated lookup                   indexed by owner/base
metadata                            graph traversal; no stringify/reparse
incremental invalidation            dependency-directed
pack terms                          preserve C2 flatness
```

Source code review + existing targeted tests are sufficient where no benchmark infrastructure exists. Add microbenchmarks only if the patch changes a performance-sensitive algorithm and existing evidence is inadequate.

### 21.2 Universe bootstrap

Record:

```text
discovered units before / after
root-reachable units before / after
executed units before / after
```

Interpretation rule:

- changes caused by ordinary runtime source reorganization may be acceptable if explained and behaviorally equivalent;
- a material increase caused solely by TypeOnly structural facade declarations is a regression/blocker.

### 21.3 Runtime allocations/representation

Use existing representation tests/introspection to prove no migration to generic data storage for Tuple/Record/Option and no new runtime allocation path for static domains.

---

## 22. Security/soundness boundary audit

Before closure explicitly audit:

### 22.1 Proper-type boundary

These must not enter APIs requiring `ProperTypeId` unless an operation yields `KindId::TYPE`:

```text
TypePack term
LabeledTypePack term
TupleEntries/CompleteTypePack term
RecordRow
type-data term
associated type-data term
ConstInt/static terms
```

### 22.2 Static identity purity

No final static identity may depend on:

```text
runtime object address
runtime hash/equality
mutable runtime state
environment variables
filesystem
clock
randomness
network
native runtime call
```

### 22.3 Stable identity

No portable metadata/interface identity may serialize raw:

```text
InferVarId
TypeId
StaticDomainId
RecordRowId
query-local source target ID
```

### 22.4 Runtime-negative audit

Search/inspect for any new:

```text
TypeData runtime object/class
static constructor VM opcode
static unification VM path
runtime module execution solely for TypeOnly source discovery
```

There should be none.

---

## 23. Checkpoint bookkeeping

### 23.1 At plan start

- verify/record completed P1/P2 state;
- mark P3 active;
- record P3 starting revision;
- refresh baseline/deferred issue ledger;
- do not mark C3 more verified than predecessor evidence supports.

### 23.2 During plan

Update checkpoint only at durable events:

- G2 establishes intrinsic static-domain facade mapping;
- G3 establishes TupleEntries/member equivalence;
- G4 establishes structural runtime facade mapping;
- G5 establishes canonical sum migration/clean source census;
- G6 establishes final tooling/source-index model;
- G7 establishes final enum state;
- consultation/amendment occurs;
- significant baseline/deferred failure is classified;
- coherent certification gates complete.

Do not turn the checkpoint into a chronological test log.

### 23.3 At plan completion

Record:

```text
P3 final revision/state
all gate outcomes
final invariants/stable interface map
metadata schema state
final enum state
runtime representation boundaries
source index/LSP state
spec closure state
bootstrap/performance evidence
deferred failures
consultations/amendments
next LANG007 action
```

---

## 24. Walkthrough deliverable

Create the final P3 walkthrough using the repository's naming convention, e.g.:

```text
LANG007.C3.P3-walkthrough.md
```

or the established descriptive-plan basename equivalent.

It must contain:

1. plan identity and final state;
2. starting and final revisions;
3. objective/outcome;
4. intrinsic static-domain declarations and exact canonical mappings;
5. intrinsic Unit/Tuple/Record source facade migration;
6. TupleEntries and structural member registrations;
7. Option/Result/Ordering migration;
8. final active enum census and lexical/parser retirement;
9. source-index/reflection/LSP/rename/editor changes;
10. authoritative spec migration;
11. metadata/schema changes or explicit statement that none were needed;
12. incremental/cold parity evidence;
13. runtime-negative/ProperType boundary evidence;
14. Universe bootstrap/resource evidence;
15. performance/complexity evidence;
16. important files/components changed;
17. deliberate deviations from this plan;
18. consultations/adopted decisions;
19. tests added;
20. tests actually executed with commands/results;
21. tests deliberately deferred/not run;
22. residual/baseline failures;
23. final P3 verification classification;
24. final C3 classification;
25. follow-up work, if any.

Never claim a gate passed without completed evidence.

---

## 25. Handoff deliverable

Create:

```text
LANG007.C3.P3-handoff.md
```

or the established descriptive-plan basename equivalent.

Because P3 closes C3, this is a C3-substrate handoff to the next accepted LANG007 work, not P3→P4 by default.

It must contain:

```text
current final revision
C3 lifecycle state
data/type-data final syntax
canonical term/domain architecture
ProperTypeId boundary
StaticArgument contract
RecordRow separation
TupleEntries == CompleteTypePack mapping
Tuple/Record/Unit facade/runtime boundary
associated-domain identity/AssociatedSurface contract
final enum lexical/parser state
metadata schema/compatibility state
source-index semantic target model
LSP rename model
tooling/editor state
known deferred baselines
must-read files
must-not-redesign invariants
next accepted LANG007 objective
first recommended verification commands for successor
```

Keep it compact enough to prevent rediscovery but complete enough that the next agent does not need this conversation.

---

## 26. Completion truth table

P3 may be marked complete only when all are true:

### Predecessors

```text
[ ] P1 complete / implemented / focused-tested
[ ] P2 complete / implemented / focused-tested
[ ] completed-P2 hard predecessor contract verified
```

### Language surface

```text
[ ] data is canonical runtime Product/Sum keyword
[ ] type data is canonical TypeOnly static-domain form
[ ] enum declaration syntax is no longer accepted
[ ] intrinsic data/type-data structural facades are source-visible
```

### Structural static domains

```text
[ ] TypePack facade maps to TypePack authority
[ ] LabeledTypePack facade maps to LabeledTypePack authority
[ ] TupleEntries facade maps to CompleteTypePack
[ ] RecordRow facade maps to row authority
[ ] no duplicate structural static domains exist
```

### Structural runtime data

```text
[ ] Unit facade preserves Unit identity/runtime
[ ] Tuple facade preserves structural Tuple identity/runtime
[ ] Record facade preserves row-backed Record identity/runtime
[ ] no generic DataObject migration occurred
```

### Structural members

```text
[ ] Tuple::Entries exists
[ ] Entries == complete pack projection
[ ] Positionals/Labeled/At/Map preserved
[ ] structural domain Map members preserved
[ ] source anchors use canonical member identity
[ ] * / ** / *** equivalences certified
```

### Runtime sums/corpus

```text
[ ] Option migrated to data with identity preserved
[ ] Result migrated to data with identity preserved
[ ] Ordering migrated to data with identity preserved
[ ] all active positive Universe enums migrated
[ ] all active first-party positive .ph enums migrated
[ ] exact-case/GADT behavior preserved
```

### P2 semantics

```text
[ ] heterogeneous application still passes
[ ] static inference still passes
[ ] aliases/type members still pass
[ ] associated type data still passes
[ ] trait unsupported boundary still explicit where applicable
```

### Persistence/incremental

```text
[ ] metadata round trip passes
[ ] stable declaration identity passes
[ ] interface fingerprints are intentional
[ ] no raw local IDs are serialized
[ ] cold/incremental parity passes
[ ] no global invalidation workaround exists
```

### Tooling

```text
[ ] source-index final categories/targets exist
[ ] hover passes
[ ] definition passes
[ ] references pass
[ ] completion passes
[ ] prepareRename/rename pass
[ ] diagnostics pass
[ ] semantic tokens pass
[ ] VS Code grammar is current
```

### Specs

```text
[ ] docs/specs ownership map is coherent
[ ] no active authoritative enum-as-canonical specification remains
[ ] type-level/type-member/associated/structural/Universe specs reflect final C3
```

### Runtime/resource

```text
[ ] no runtime TypeOnly inhabitants/classes
[ ] no static VM solver/opcodes
[ ] Tuple/Record/Option representations preserved
[ ] Unit identity preserved
[ ] Universe bootstrap/resource evidence acceptable
[ ] ordinary static operations remain within accepted complexity
```

### Verification/lifecycle

```text
[ ] all mandatory P3 gates passed
[ ] A/B failures resolved or architecturally escalated
[ ] acceptance-overlapping C failures resolved
[ ] deferred D failures recorded
[ ] P3 walkthrough complete
[ ] P3 handoff complete
[ ] C3 checkpoint record truthful
[ ] PROGRAM.md / STATUS.md truthful
[ ] next LANG007 action explicit
```

Only then may:

```yaml
LANG007.C3.P3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

be recorded.

`RELEASE_COMPLETE` remains a separate stronger claim.

---

## 27. Residual-risk reporting

If C3 closes with any deferred issue, the walkthrough/checkpoint must state:

```text
ID
observed failure/risk
classification
why it is outside C3 acceptance
reproduction
why required C3 evidence remains valid
when to revisit
```

Do not use a deferred ledger to hide an unresolved acceptance blocker.

Expected high-risk seams to report explicitly even when closed successfully:

```text
intrinsic Tuple/Record representation preservation
TupleEntries/CompleteTypePack non-duplication
TypeOnly bootstrap isolation
Option/Result/Ordering identity preservation
final enum parser retirement
semantic rename identity
metadata source-facade stability
cold/incremental parity
```

---

## 28. Plan self-review

Before implementation handoff, verify this plan remains true against the completed-P2 tree:

### Architecture

- one canonical owner exists for every structural/static/runtime fact;
- intrinsic source facades are bindings, not semantic replacements;
- TupleEntries is naming, not a new term domain;
- runtime structural and ADT representations remain specialized;
- source index is the tooling identity owner;
- no consumer reconstructs canonical semantics from strings.

### Luna executability

- T1 creates the exact future-path manifest that cannot be known while P2 is unimplemented;
- every later task consumes a concrete predecessor product;
- fixed/flexible/verify-first distinctions are explicit;
- stop triggers catch predecessor incompleteness and architecture drift;
- semantic debugging is bounded to one serious correction.

### Testing

- BUILD mode stays narrow;
- migration is proven before parser retirement;
- P3 deliberately broadens only at certification;
- metadata/incremental/runtime/resource/tooling evidence is explicit;
- optional release-strength workspace test/clippy is not confused with checkpoint closure.

### Lifecycle

- checkpoint updates happen at durable gates;
- walkthrough records actual implementation/evidence;
- terminal handoff freezes C3 substrate;
- no P4 is invented unless a bounded corrective closure is actually required.

### Final architecture sanity check

After successful C3 closure the platform should read coherently as:

```text
KindId
├─ Type
├─ RecordRow
├─ TypePack
├─ LabeledTypePack
├─ CompleteTypePack   <- public facade: TupleEntries
├─ Arrow(...)
└─ StaticDomain(id)   <- user/nominal type-data domains
```

```text
TypeId
├─ proper Types
├─ packs
├─ complete packs
├─ static constants
├─ type-data terms
├─ symbolic projections
└─ heterogeneous Applied terms

RecordRowId
└─ dedicated row identity
```

```text
data
├─ Product
└─ Sum
    ├─ singleton cases
    ├─ tuple-payload cases
    ├─ record-payload cases
    ├─ zero-arg constructor cases
    └─ GADT cases

type data
├─ nominal static algebra
└─ intrinsic compiler structural facade
```

```text
intrinsic type data TypePack
    -> KindId::TYPE_PACK

intrinsic type data LabeledTypePack
    -> KindId::LABELED_TYPE_PACK

intrinsic type data TupleEntries
    -> KindId::COMPLETE_TYPE_PACK

intrinsic type data RecordRow
    -> KindId::RECORD_ROW
```

```text
intrinsic data Unit()
    -> canonical Unit

intrinsic data Tuple<***Es>
    -> structural Tuple type family + specialized Tuple runtime class

intrinsic data Record<R: RecordRow>
    -> TypeData::Record(RecordRowId) + specialized Record runtime class
```

and every layer:

```text
parser
AST
semantic identity
TypeStore/row authority
inference/substitution
associated lookup
metadata/imports
incremental DB
Universe source
compiler/runtime
source index
LSP/editor
active specs
```

must describe that same model without independently reconstructing it.
