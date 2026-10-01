---
id: LANG007.C3.P4
category: LANG
program: LANG007
checkpoint: LANG007.C3
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - LANG007.C3.P3
follows: LANG007.C3.P3
supersedes: null
repository: aureat/phalcom-lang
prepared: 2026-09-25
prepared_against_revision: 71e77727f510fe1f693c084a833e2d5d5ea65745
---

# LANG007.C3.P4 — Intrinsic Structural Data, Structural Kinds, Product-Shape Expansion, and Tuple/Record Semantic Completion

> **Execution profile:** Luna patch-grade implementation plan.  
> **Checkpoint role:** accepted corrective/reopening plan for `LANG007.C3`; this plan completes the structural-data model that the original P1–P3 migration exposed but did not fully source-model.  
> **Primary architectural rule:** source declarations describe the logical structural family; canonical semantic products own type identity and specialization; the compiler projects those facts; the VM owns runtime representation and behavior-class identity. No consumer reconstructs structural semantics from syntax or runtime class names.

---

# 0. Executor contract

This plan is designed for a capable implementation model operating under constrained architectural authority.

The implementer:

- may adapt private helper names, imports, local decomposition, and mechanically drifted paths to the live tree;
- must preserve the semantic architecture, identities, runtime-family rules, and source-of-truth boundaries defined below;
- must not invent parallel pack/row representations, alternate declaration registries, per-specialization runtime classes, or a second associated-member authority;
- must not use `Dynamic` as a convenience escape from structural typing where this plan specifies a computable type;
- must not invent an existential `AnyTuple`, `AnyRecord`, generalized product protocol, or mixed Tuple/Record destructuring in this work;
- must preserve the existing `CompleteTypePack`, `TupleFromPacks`, `RecordRowId`, row solver, heterogeneous `StaticArgument`, runtime typing, and `AssociatedSurface` authorities unless this plan explicitly changes their source-facing name or projection contract;
- must follow the BUILD / STABILIZE / CERTIFY testing budget rather than running broad suites after every edit;
- must classify unrelated failures rather than repairing them opportunistically;
- must STOP AND CONSULT when any global or task-local escalation trigger fires;
- must keep `LANG007.C3-CHECKPOINT.md` current at meaningful lifecycle boundaries;
- must create the normal P4 walkthrough and handoff at completion.

Testing is evidence gathering, not a ritual. A focused PASS answers the question it was designed to answer; continue implementation unless a named gate requires broader evidence.

---

# 1. Goal

Complete Phalcom's Tuple/Record structural-data model by making structural kinds and shape expansion honest source-language concepts, making `Tuple` and `Record` canonical intrinsic data families with source-authored product shapes and independently projected runtime classes, migrating Record/Map/Set syntax to the ratified forms, replacing public `RecordRow` with `RecordFields`, defining `Unit = Tuple<>` and `EmptyRecord = Record<>` while preserving `Unit != EmptyRecord`, and closing the remaining declaration/runtime/type-member/incremental/tooling audit gaps without introducing duplicate semantic authorities.

The canonical source endpoint is:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields

intrinsic data Tuple<***Es>(***Es)

impl<***Es: CompleteTypePack> Tuple<***Es> {
    intrinsic type Entries: CompleteTypePack
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type Element: Type
    intrinsic type At<I: Int>: Type
    intrinsic type Map<F: Type -> Type>: Type

    // specialization-aware runtime behavior
}

intrinsic data Record<**Fs> {
    **Fs
}

impl<**Fs: RecordFields> Record<**Fs> {
    intrinsic type Fields: RecordFields
    intrinsic type Value: Type
    intrinsic type Map<F: Type -> Type>: Type

    // specialization-aware runtime behavior
}

intrinsic type Unit = Tuple<>
intrinsic type EmptyRecord = Record<>
```

The user-facing collection/product syntax at closure is:

```phalcom
()                       // Unit / empty Tuple
(1, "x")                 // Tuple
{}                       // EmptyRecord / empty Record
{name: "Altun"}          // Record
[1, 2, 3]                // List
#{1, 2, 3}               // Set
Map { name: "Altun" }    // Map
Map<Symbol, String> {
    name: "Altun"
}
```

---

# 2. Checkpoint acceptance objective

`LANG007.C3` was previously recorded complete after P3 and a bounded corrective closure. This plan intentionally reopens the checkpoint because the newly ratified structural-data semantics materially extend the same acceptance objective: source declarations, compiler structural domains, associated type computation, runtime structural owners, and tooling must agree on one model.

P4 is complete when all of the following are simultaneously true:

1. `TypePack`, `LabeledTypePack`, `CompleteTypePack`, and `RecordFields` are canonical source-visible intrinsic structural domains bound to the existing semantic authorities.
2. expansion mode and semantic kind are independently represented and checked. An omitted binder kind has a deterministic prefix-based default; there is no contextual kind inference. An explicit compatible kind may select another domain, so `**Ls: LabeledTypePack` remains valid while plain `**Fs` means `RecordFields`.
3. ordinary users can declare shape-generic nominal products such as:
   ```phalcom
   data MyTuple<***Es: CompleteTypePack>(***Es)
   data MyRecord<**Fs: RecordFields> { **Fs }
   ```
   and those declarations remain nominal while their product shapes specialize from the bound pack/field domain.
4. the intrinsic declarations:
   ```phalcom
   intrinsic data Tuple<***Es>(***Es)
   intrinsic data Record<**Fs> { **Fs }
   ```
   use the same source shape grammar but intrinsic family application canonicalizes to the existing structural Tuple/Record semantic types.
5. bare `Tuple` and `Record` no longer rely on the C3 workaround that rewrites their type forms to proper kind `Type`; their type-family projection retains its real higher kind while their runtime class projection remains independently available.
6. `Unit` is a canonical intrinsic type alias of `Tuple<>`; it no longer owns a distinct runtime class. The immediate unit value reports runtime class `Tuple`.
7. `EmptyRecord` is a canonical intrinsic type alias of `Record<>`; it is a distinct type/value from Unit and reports runtime class `Record`.
8. empty Tuple and empty Record have distinct runtime identity/representation even though both are zero-payload products.
9. Record literal/type/pattern syntax uses bare `{...}`; Map construction uses explicit `Map {...}`; Set literals use `#{...}`.
10. source-visible intrinsic associated type declarations are authoritative for existence, name, signature, navigation, and documentation; compiler intrinsic operation tables implement those source declarations rather than creating hidden public operations.
11. Tuple/Record public and internal source signatures are specialization-aware and do not erase known structural types to `Dynamic`.
12. metadata, cold/incremental analysis, source indexing, LSP/editor navigation, native surface import, and runtime lowering preserve the same canonical identities.
13. the P3 audit gaps are closed: declaration replacement/refresh semantics are explicit, runtime lowering consumes the explicit runtime owner, intrinsic application is consolidated, Universe identity is canonical, and broad post-P4 certification is performed once at closure.

This plan is therefore a **corrective semantic completion + source migration + checkpoint re-closure** plan.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch/default line inspected: main/default branch
revision: 71e77727f510fe1f693c084a833e2d5d5ea65745
recent implementation revision relevant to intrinsic projections:
    82c9dd0382ece4932de4a52e98fdd36452b122c8
predecessor lifecycle closure:
    LANG007.C3.P3
existing corrective record currently (incorrectly for the newly accepted plan) named:
    LANG007.C3.P4
```

Verified live facts at planning time:

- `phalcom-semantic/src/types/kind.rs` already has `KindData::{TypePack, LabeledTypePack, CompleteTypePack, RecordRow}`.
- `phalcom-semantic/src/types/store.rs` already has `TypeData::{TypePack, LabeledTypePack, CompleteTypePack, TupleFromPacks, Record}` and the heterogeneous `StaticArgument::{Term, RecordRow}` envelope.
- `RecordRowId`, `RecordRowData`, row tails, row inference, and row solving are separate from `TypeId`; this boundary is established and remains correct.
- `GenericParameterMode` is nominally separate from parameter kind, but `GenericSignature::validate_publishable` still hard-couples `LabeledPack -> LabeledTypePack`.
- Universe bootstrap currently resolves parameter kind from expansion mode *before* an explicit kind annotation; therefore `**Fs: RecordFields` cannot work correctly without changing the bootstrap/general binder path.
- `TupleEntries` currently names `KindId::COMPLETE_TYPE_PACK` through a source facade; `CompleteTypePack` itself is not the canonical public source declaration.
- `RecordRow` is the current public source spelling for `KindId::RECORD_ROW`.
- runtime intrinsic data currently use `AlgebraicShapeSyntax::IntrinsicFacade`; source-authored Tuple/Record product shapes are absent.
- `DataComponentSyntax`, `DataInfo`, `DataConstructorSignature`, and declaration lowering assume a finite fixed component list with one `DataComponentId` per component.
- `DataDeclarationLoweringSpec` currently publishes one fixed declaration-level product layout. `RuntimeDataRegistry` already supports exact specialization descriptors keyed by semantic owner and optional exact runtime type.
- current Tuple source contains `intrinsic data Tuple<***Es>` and multiple `Dynamic` or bare-`Tuple` signatures.
- current Record source contains `intrinsic data Record<R: RecordRow>` and `Dynamic` raw value access.
- current Unit source is `intrinsic data Unit()`.
- current `Value::unit()` is an immediate value whose `.class` path returns a distinct `unit_class`.
- the VM currently normalizes **both** empty Tuple and empty Record to `Value::unit()`.
- bare `{...}` currently parses Map associations; `#{...}` is a dedicated Record opener token; Set AST/checker/runtime machinery exists but source Set syntax is deferred.
- bare brace block literals are already removed; explicit closures use `|| { ... }`, so assigning bare `{...}` to Record does not steal the ordinary block-literal surface.
- Record patterns currently use `#{...}` while Map patterns use `{...}`.
- current `compile_intrinsic_runtime_data` obtains an `IntrinsicRuntimeFacadeLoweringSpec` but then re-derives the VM class from source module/name rather than consuming the projected owner directly.
- `DeclarationTypeTable::insert` currently preserves old intrinsic projections when a new shell carries defaults, making authoritative replacement and shell refresh semantically ambiguous.
- source-visible intrinsic type-member declarations and compiler intrinsic operation tables are not yet bidirectionally certified; `CompleteTypePack` has an intrinsic owner enum value but no operations.
- the current type-member projection representation still documents compiler intrinsic projections as possibly source-less.
- `../../../__unstructured__/spec/collections/03-records.md` already shows bare `{...}` Record syntax, but currently says Record spread is illegal; this conflicts with the ratified P4 model and must be amended.
- VM product documentation still records nullary Record as Unit; that becomes false under `Unit != EmptyRecord`.

**Implementation instruction:** re-read the named live paths immediately before editing. Adapt mechanical drift locally. Treat architectural drift—especially changes to row identity, runtime product representation, generic specialization, or declaration projection ownership—as an escalation condition.

---

# 4. Required reads before implementation

Read these before editing:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P3-handoff.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P3-walkthrough.md

the existing corrective records currently carrying LANG007.C3.P4
(the implementation-plan/corrective record, walkthrough, and handoff)

LANG007.C3.P4-requirements-analysis.md
(the requirements analysis supplied with this plan)

docs/specs/README.md
docs/specs/type-level-domains.md
docs/specs/structural-products.md
docs/specs/type-members.md
docs/specs/universe.md
docs/spec/collections/02-tuples.md
docs/spec/collections/03-records.md
docs/spec/collections/06-rest-spread-and-pack-operators.md
docs/specs/objects/values.md
docs/specs/vm/products.md
docs/decisions/accepted/0081-tuple-and-record-are-transparent-structural-value-products.md

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
```

Then inspect at minimum:

```text
phalcom-ast/src/{token.rs,lexer.rs,ast.rs,parser.rs}

phalcom-semantic/src/declarations.rs
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/checker/algebraic_declaration.rs
phalcom-semantic/src/checker/{expression.rs,pattern.rs,composition.rs}
phalcom-semantic/src/types/{kind.rs,parameter.rs,store.rs,row.rs,annotation.rs,instantiation.rs,type_member.rs,projection.rs}
phalcom-semantic/src/metadata/
phalcom-semantic/src/source_index/

phalcom-type-meta/src/{kind.rs,type_node.rs}

phalcom-core/core/universe/src/collections/{tuple.ph,record.ph,map.ph,set.ph}
phalcom-core/core/universe/src/option/unit.ph
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/{data_decl.rs,expr.rs,product_opt.rs}
phalcom-core/src/product/
phalcom-core/src/data.rs
phalcom-core/src/value/{mod.rs,repr.rs}
phalcom-core/src/primitive/{tuple.rs,record.rs}
phalcom-core/src/universe/{core_classes.rs,primitives.rs,invariants.rs}
phalcom-core/src/vm/{bootstrap.rs,data.rs}
phalcom-native-meta/src/universe.rs
phalcom-core/src/native/source.rs

phalcom-lsp/src/
```

Do not broadly rediscover unrelated subsystems once these seams are verified.

---

# 5. Normative authority

## 5.1 Ratified P4 behavior — FIXED

The following conversation decisions are accepted input to this plan and override contradictory current implementation or stale prose:

### Structural domain names

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields
```

`RecordFields` is the public language name for the existing row-polymorphic structural field domain. Internal Rust names may remain `RecordRow*`; public diagnostics, source declarations, reflection/presentation, and docs use `RecordFields`.

### Structural family declarations

```phalcom
intrinsic data Tuple<***Es>(***Es)

intrinsic data Record<**Fs> {
    **Fs
}
```

The declaration shape is semantically honest: `Es`/`Fs` are not phantom parameters. They determine the logical product components/fields of each specialization.

### User-defined product-shape expansion

Users may write:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)

data MyRecord<**Fs: RecordFields> {
    **Fs
}
```

These remain nominal families. Shape expansion does not make user `data` structural.

### Expansion mode versus kind

Expansion topology and semantic kind are independent facts.

Core valid combinations include:

```text
Single + explicit legal domain
*      + TypePack
**     + LabeledTypePack
**     + RecordFields
***    + CompleteTypePack
```

`LabeledTypePack != RecordFields`.

### Zero products

```phalcom
intrinsic type Unit = Tuple<>
intrinsic type EmptyRecord = Record<>
```

with:

```text
Tuple<>  == Unit == ()
Record<> == EmptyRecord == {}
Unit != EmptyRecord
```

### Runtime classes

```text
runtimeClass(Unit)        = Tuple
runtimeClass((Int, ...))  = Tuple
runtimeClass(EmptyRecord) = Record
runtimeClass({a: Int})    = Record
```

There is no language-level `EmptyTuple`, `NonEmptyTuple`, `EmptyRecord` runtime class, or `NonEmptyRecord` runtime class.

### Syntax

```text
Tuple       (...)
Record      {...}
List        [...]
Set         #{...}
Map         Map {...}
typed Map   Map<K,V> {...}
```

### Scope boundary

Mixed Tuple/Record destructuring, exposing a Tuple/Record product relationship, general product-coordinate protocols, user-defined structural shape protocols, and generalized Tuple↔Record conversions are explicitly deferred.

### Structural inherent-target ownership refinement — accepted 2026-09-25

Tuple and Record structural types are valid inherent-impl domains. Their behavior owners are the canonical Universe `Tuple` and `Record` declarations. This applies to direct structural spellings, family applications, generic covering heads, and specialized structural heads. The impl domain remains the canonical structural `TypeId`; it is not nominalized.

Equivalent spellings must canonicalize before applicability, overlap, and selector-collision analysis. Only the corresponding canonical Universe owner module may author these impls. Do not retain source-provenance checks whose only purpose is to distinguish `Tuple<...>` / `Record<...>` from equivalent anonymous structural types. Do not introduce structural target variants or a second behavior/member registry.

## 5.2 Existing normative architecture retained

The plan retains:

- `CompleteTypePack` as the canonical complete Tuple-entry static domain.
- `TupleFromPacks` as the symbolic Tuple formation authority.
- `RecordRowId` and the row solver as the internal Record-fields authority.
- `StaticArgument` as the heterogeneous generic/static argument envelope.
- `AssociatedSurface` as the sole `::` namespace/collision authority.
- canonical source declaration identities and stable source/member IDs.
- existing runtime typing/metadata identity rules.
- ordinary nominal identity for user `data`.
- one behavior/runtime class per nominal declaration unless already otherwise specified.
- structural Tuple/Record exact types as compiler semantic types, not one runtime class per shape.
- canonical Universe Tuple/Record declarations as the behavior owners for their structural families.

## 5.3 Current implementation is not normative where it conflicts

The following current behaviors are migration targets, not authorities:

- public `RecordRow`;
- `TupleEntries` as a pseudo-domain;
- `intrinsic data Unit()`;
- a distinct Unit runtime class;
- empty Record normalizing to `Value::unit()`;
- `#{...}` Record source syntax;
- bare `{...}` Map source syntax;
- intrinsic Tuple/Record declarations with no authored product shape;
- mode-to-kind one-to-one coupling;
- `Dynamic` source signatures caused only by generic VM value representation;
- bare `Tuple`/`Record` being forced to kind `Type` to make runtime native import work.

## 5.4 Generic pack application syntax and binder defaults — architect clarification accepted 2026-09-25

P4 supports only explicit expansion contributions to generic pack parameters.
Direct mixed variadic contributions such as `Foo<Int, String, x: Bool>` and
direct labeled Record contributions such as `Record<name: String>` are outside
this plan. Use explicit expansions:

```phalcom
Foo<***(Int, String, x: Bool)>
Foo<*(Int, String)>
Foo<**{ name: String }>
Record<**{ name: String }>
Tuple<***(Int, String, labeled: Bool)>

Foo<*Ts>
Foo<**Fs>
Foo<***Es>
```

Fixed generic parameters before a pack tail remain single arguments; every
contribution consumed by a variadic binder uses an explicit expansion. The
zero-argument forms `Tuple<>` and `Record<>` remain supported.

Binder mode and semantic kind are separate. With no explicit annotation, the
prefix supplies the default and context never infers another kind:

```text
T      = Type
*Ts    = TypePack
**Fs   = RecordFields
***Es  = CompleteTypePack
```

Explicit compatible kind annotations remain permitted, including
`**Ls: LabeledTypePack`. The canonical concise Universe declarations are
`intrinsic data Tuple<***Es>(***Es)` and
`intrinsic data Record<**Fs> { **Fs }`.

---

# 6. Takeover state

| Concept | Current symbol/path | Owner | Inherited invariant |
|---|---|---|---|
| proper/static type arena | `TypeStore`, `TypeData` | semantic | one canonical interning authority |
| complete Tuple shape | `TypeData::CompleteTypePack` | semantic | ordered positional + labeled lanes |
| symbolic Tuple formation | `TypeData::TupleFromPacks` | semantic | no nominal `Applied<Tuple,...>` replacement |
| Record row/fields | `RecordRowId`, `RecordRowData` | semantic row subsystem | remains outside `TypeId` |
| heterogeneous args | `StaticArgument::{Term,RecordRow}` | semantic | row terms are not fake types |
| generic row solving | row solver + inference coordinator | semantic checker | no second solver |
| declaration shell | `DeclarationTypeInfo` | semantic declarations | source declaration identity is canonical |
| intrinsic projections | `DeclarationTypeBinding` | semantic declarations | semantic application and runtime owner are siblings |
| type members | `TypeMemberDeclarationId`, `AssociatedSurface` | semantic | source identity/lookup authority is singular |
| runtime data specializations | `RuntimeDataRegistry` | VM | exact descriptors may share behavior class |
| anonymous products | `ProductShapeRegistry`, `RuntimeAnonymousProductDescriptorRegistry` | VM | shape/layout/exact runtime type are distinct |
| immediate Unit | `Value::unit()` | VM value representation | allocation-free canonical zero Tuple value remains desirable |
| Tuple class | `CoreClasses::tuple_class` | VM | single runtime behavior class for positive Tuples |
| Record class | `CoreClasses::record_class` | VM | single runtime behavior class for positive Records |
| native Tuple/Record methods | primitive modules + Universe `.ph` | runtime/source | source owns public contract; primitive owns execution |
| metadata | `phalcom-type-meta` + semantic exporter/importer | metadata | stable IDs, no arena integers as durable identity |
| source index/LSP | semantic source products | semantic/editor | no token-spelling reconstruction of canonical identity |

P4 changes the public shape and several projection contracts, but it must reuse these authorities rather than replacing them.

---

# 7. Architecture

## 7.1 High-level authority flow

```text
source syntax
    ↓
AST preserves:
  - declaration implementation provenance
  - product shape contributions
  - generic parameter expansion mode
  - explicit semantic kind annotation
    ↓
semantic declaration/header analysis
    ↓
canonical declaration product
  - declaration identity
  - type-family form/kind
  - runtime-owner projection when any
  - generic binder (mode + kind independently)
  - product-shape template
  - associated/static surface
    ↓
specialization / inference
  - StaticArgument terms
  - CompleteTypePack
  - RecordRowId (public RecordFields)
  - specialized nominal or intrinsic structural type
  - specialized product layout facts
    ↓
semantic lowering projection
    ↓
compiler
    ↓
bytecode / runtime descriptors
    ↓
VM
  - shared behavior class
  - specialization-specific layout/type descriptor
  - canonical empty Tuple/Record value representation
```

No lower layer may rediscover type shape from source spelling.

## 7.2 Declaration family/runtime projection

A named runtime type declaration is modeled as one declaration identity with independent projections.

For Tuple:

```text
DeclarationId(Tuple)
    static/type-family projection:
        CompleteTypePack -> Type
        reducer = intrinsic Tuple structural formation

    runtime projection:
        canonical VM Tuple class

    associated surface:
        Tuple's authored intrinsic type members and behavior

    source/reflection identity:
        canonical Universe Tuple declaration
```

For Record:

```text
DeclarationId(Record)
    static/type-family projection:
        RecordFields -> Type
        reducer = intrinsic Record structural formation

    runtime projection:
        canonical VM Record class
```

For ordinary user shape-generic data:

```text
DeclarationId(MyTuple)
    static/type-family projection:
        CompleteTypePack -> Type
        reducer = ordinary nominal application

    runtime projection:
        one MyTuple behavior class

    specialization:
        exact nominal type + specialization-specific product layout
```

This is the semantic completion of the C3 `semantic_form`/`runtime_owner` split. It does **not** require a new general-purpose declaration object runtime hierarchy.

## 7.3 Product-shape templates

A declaration with spread components cannot be represented as a fixed list of `DataComponentId`s.

Introduce one canonical semantic template capable of preserving fixed source components and symbolic expansion contributions. Exact names are mechanically flexible; the required shape is conceptually:

```rust
enum DataProductContribution {
    Fixed(DataComponentSemantic),
    Expansion(DataProductExpansionSemantic),
}

struct DataProductExpansionSemantic {
    site: StableExpansionSiteId,       // or equivalent stable source identity
    mode: ExpansionMode,
    parameter: TypeParameterId,        // or canonical static operand term
    domain: KindId,
    source: SemanticSourceSpan,
}

struct DataProductShapeTemplate {
    shape: DataShape,
    contributions: Box<[DataProductContribution]>,
}
```

Rules:

- fixed source components retain existing `DataComponentId`;
- one symbolic spread must **not** manufacture one `DataComponentId` per future specialized component;
- the expansion site may have its own stable identity for dependency/source provenance;
- specialization materializes a concrete logical component/field sequence from the template plus the generic/static environment;
- symbolic analysis may retain a symbolic template when a pack/row is not concrete;
- runtime layout is derived from the exact specialization, not from the declaration template itself.

## 7.4 Shared behavior class + specialization layout

For ordinary expanded nominal data, use:

```text
one declaration
    ↓
one behavior class
    ↓
many exact specialization descriptors/layouts
```

Do not create a VM class for each `MyTuple<***(Int,String)>` or `MyRecord<**{...}>`.

`RuntimeDataRegistry` already supports exact descriptors keyed by `(DeclarationId, Option<RuntimeTypeRef>)`; extend the same registry so declaration-to-behavior-class identity is available independently from a fixed base-layout descriptor. Then specialization-time `DataConstructionLoweringSpec` registers/looks up the exact descriptor with:

- shared behavior class;
- exact runtime type;
- exact layout;
- source-to-logical argument mapping.

Fixed-only `data` may retain its existing declaration-level fast path.

Expanded record fields cannot be installed as declaration-wide runtime getters because field sets vary by specialization. Exact expanded-field access must be resolved semantically against the specialized type and lowered to a logical slot. Fixed source fields keep stable `DataComponentId` ownership.

## 7.5 Unit and EmptyRecord runtime representation

Retain `Value::unit()` as the canonical allocation-free empty Tuple value, but change its runtime classifier:

```text
Value::unit().class == Tuple
```

Remove Unit as an independent VM class/runtime owner.

Introduce a distinct canonical empty Record value. Preferred implementation at the audited revision:

```rust
ValueTag::EmptyRecord
Value::empty_record()
Value::is_empty_record()
```

because the 16-byte tagged `Value` representation has unused tag space, this preserves allocation-free nullary product construction, naturally discriminates `===`/hash identity, and permits `.class == Record`.

If the live representation has materially changed or adding a tag conflicts with a frozen representation contract, STOP AND CONSULT rather than silently substituting a heap singleton or reusing Unit.

Runtime structural receiver helpers must treat:

```text
Unit immediate        as empty Tuple receiver
EmptyRecord immediate as empty Record receiver
```

so Tuple/Record methods work uniformly at arity zero.

## 7.6 Structural type-member authority

Public intrinsic type operations are split cleanly:

```text
source declaration
    owns:
      name
      member existence
      generic parameter domain
      result domain
      source identity
      docs/navigation

compiler intrinsic evaluator
    owns:
      normalization/reduction
```

Every public intrinsic operation must have a source `TypeMemberDeclarationId`; every source `intrinsic type` member in these structural domains must map to exactly one compiler evaluator.

No hidden public compiler-only member and no source intrinsic declaration without an evaluator.

## 7.7 Syntax migration architecture

After P4:

```text
`{` in value-product position       → Record literal
`{` in type-product position        → Record type
`{` in pattern position             → Record pattern
`#{`                                → Set literal opener
`Map {` / `Map<K,V> {`              → explicit Map construction
```

Do not inspect the contents of anonymous braces to decide Record versus Map.

`Map` construction may be recognized syntactically by the contextual builtin spelling, but semantic lowering must attach the canonical Universe Map declaration (`CoreIds`/`UniverseKey`/stable `DeclarationId`), not later infer identity from a string.

## 7.8 Typed runtime surface

VM primitives may physically return generic `Value`, but source signatures use the strongest sound semantic type.

Required examples:

```phalcom
impl<***Es: CompleteTypePack> Tuple<***Es> {
    intrinsic type Element: Type

    internal native rawAt(_ index: Int) -> Element
    first -> Option<Element>
    last -> Option<Element>
    get(_ key) -> Option<Element>
}

impl<**Fs: RecordFields> Record<**Fs> {
    intrinsic type Value: Type

    internal native rawValueAt(_ index: Int) -> Value
    get(_ key: Symbol) -> Option<Value>
}
```

Exact literal Tuple indexing remains `At<I>`.

A runtime-dependent operation whose result shape cannot be expressed as the current specialization—especially arbitrary Tuple slicing—must not use ill-kinded bare `Tuple` after Tuple becomes a higher-kinded family. Use an already-existing proper abstraction if one accurately describes the result, keep the operation internal with a sound proper return type, or defer the source surface. **Do not invent an existential Tuple type in P4.**

---

# 8. Ownership boundaries

## 8.1 Owns

P4 owns:

- public naming of structural static domains;
- kind syntax for `CompleteTypePack` and `RecordFields`;
- independent binder expansion mode/domain validation;
- data product-shape spread grammar and semantic templates;
- specialization of user-defined shape-generic nominal data;
- intrinsic Tuple/Record source shape declarations;
- Tuple/Record declaration family/runtime projection completion;
- Unit and EmptyRecord alias semantics;
- zero-product runtime class behavior;
- Record/Map/Set surface migration;
- Tuple/Record structural intrinsic associated types;
- specialization-aware Tuple/Record API typing;
- metadata/fingerprint/source-index/LSP migration required by those changes;
- closure of the prior C3 audit findings;
- focused plus broad checkpoint re-certification.

## 8.2 Does not own

P4 does not own:

- mixed Tuple/Record destructuring;
- a public generalized product kind/protocol;
- Tuple↔Record conversion relationships;
- arbitrary row concatenation algebra;
- first-class `RecordFields { ... }` terms;
- runtime pack objects;
- per-specialization runtime classes;
- generalized const evaluation;
- dependent types;
- a new type former hierarchy;
- generalized existential structural types;
- user-defined compiler intrinsic operations;
- a rewrite of all nominal class/data runtime representation;
- unrelated collection redesign;
- unrelated Set spread execution if it is not already complete and required for the basic `#{...}` literal surface.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| parameter expansion mode | generic parameter semantic product | inference, application, metadata, reflection | inferring mode from kind |
| parameter semantic kind | `KindId` on parameter | kinding, application, metadata | inferring kind solely from star count |
| complete Tuple shape | `CompleteTypePack` / Tuple pack machinery | Tuple family, type members, data expansion | `TupleEntries` second domain |
| Record field shape | `RecordRowId` internally / `RecordFields` source | Record family, data expansion, row solver | LabeledTypePack conversion |
| declaration type family | `DeclarationTypeInfo` static/type projection | type annotation/application | runtime class identity |
| runtime owner | declaration runtime projection | native import, codegen, VM | module/name re-derivation |
| product declaration shape | semantic shape template | constructor checking, lowering, reflection | compiler AST re-walk |
| exact nominal expanded layout | specialized semantic/lowering product | compiler/VM | declaration-wide guessed layout |
| associated member existence | `AssociatedSurface` + source type-member declaration | checker/LSP | intrinsic name switch as lookup authority |
| intrinsic normalization | intrinsic operation evaluator | type normalization | source interpreter |
| Unit type | canonical `TypeData::Unit` reached via `Tuple<>` alias | checker, metadata | nominal Unit data identity |
| Unit runtime class | Tuple runtime class | dispatch/reflection | separate Unit class |
| EmptyRecord type | canonical empty `TypeData::Record` | checker, metadata | Unit |
| EmptyRecord runtime class | Record runtime class | dispatch/reflection | Unit class or new EmptyRecord class |
| Map construction target | canonical Universe Map declaration | checker/compiler | raw `"Map"` semantic identity |
| incremental dependency | canonical declaration/member/template IDs | DB | source-string dependency keys |

---

# 9. Global invariants

`INV-01` — `TypePack`, `LabeledTypePack`, `CompleteTypePack`, and `RecordFields` are distinct public static domains.

`INV-02` — internal `RecordRowId` remains outside `TypeId`; renaming the source domain does not weaken the row/type boundary.

`INV-03` — expansion mode and semantic kind are stored and validated independently.

`INV-04` — `**` may bind `LabeledTypePack` or `RecordFields`; this never implies equality or implicit conversion between the domains.

`INV-05` — `***Es: CompleteTypePack` is the canonical complete Tuple-shape binder.

`INV-06` — `TupleEntries` does not survive as a second semantic domain.

`INV-07` — product-shape expansion is real declaration semantics, not a parser-only rewrite.

`INV-08` — fixed source components keep stable `DataComponentId`; specialization-generated components do not receive fabricated source component IDs.

`INV-09` — user-defined expanded `data` remains nominal.

`INV-10` — intrinsic Tuple/Record use the same product-shape syntax but compiler-owned family reduction yields structural types.

`INV-11` — `Tuple<> == Unit == ()`.

`INV-12` — `Record<> == EmptyRecord == {}`.

`INV-13` — `Unit != EmptyRecord`.

`INV-14` — every Tuple value, including Unit, has runtime class Tuple.

`INV-15` — every Record value, including EmptyRecord, has runtime class Record.

`INV-16` — Unit does not own a distinct runtime class after P4.

`INV-17` — EmptyRecord does not introduce a distinct runtime class.

`INV-18` — empty Tuple and empty Record use distinct runtime values/identities.

`INV-19` — one nominal behavior class is shared by all specializations of a user `data` declaration unless an already-established runtime rule says otherwise.

`INV-20` — exact specialization layout may vary without changing behavior-class identity.

`INV-21` — bare generic Tuple/Record static forms retain their real higher kinds; runtime class projection is not encoded by lying about static kind.

`INV-22` — runtime-native import consumes the runtime projection; semantic application consumes the static/type-family projection.

`INV-23` — authoritative declaration replacement and shell refresh are distinct APIs/operations.

`INV-24` — Record syntax is `{...}` in type/value/pattern contexts.

`INV-25` — Map construction is explicit `Map {...}` / `Map<K,V> {...}`.

`INV-26` — Set literal syntax is `#{...}`.

`INV-27` — Record `**` spread is structural row composition; Map `**` expansion is runtime association expansion.

`INV-28` — Record duplicate labels are statically rejected or represented as a row-lacks obligation; Map duplicate-key policy is independent.

`INV-29` — no structural product spread whose exact/symbolic static shape is unavailable silently degrades to `Dynamic`.

`INV-30` — public Tuple/Record intrinsic type-member declarations are source-authored and carry canonical source identity.

`INV-31` — every public intrinsic type member has exactly one compiler evaluator; every source intrinsic member has exactly one recognized evaluator.

`INV-32` — `Tuple::Entries : CompleteTypePack`.

`INV-33` — `Tuple::Element` is the canonical union/join of all payload types; empty is `Never`; unresolved symbolic subjects remain symbolic projections.

`INV-34` — `Record::Fields : RecordFields`.

`INV-35` — `Record::Value` is the canonical union/join of all field payload types; empty is `Never`; open/symbolic rows remain symbolic when the complete union is not known.

`INV-36` — VM generic `Value` representation does not justify `Dynamic` in source signatures.

`INV-37` — no P4 API uses ill-kinded bare `Tuple` or bare `Record` as a proper return/field type after the family kind is corrected.

`INV-38` — no existential AnyTuple/AnyRecord type is introduced by this plan.

`INV-39` — anonymous Tuple/Record syntax continues to lower to the canonical intrinsic families; it is not a parallel structural type system.

`INV-40` — metadata serializes semantic kind and expansion mode independently.

`INV-41` — cold and incremental analysis converge for all new declaration shapes and source-surface edits.

`INV-42` — source index/LSP navigation for intrinsic members targets the authored Universe declaration/member.

`INV-43` — runtime lowering consumes stable semantic projections and does not reconstruct canonical owner identity from source strings.

`INV-44` — no second pack, row, runtime-owner, type-member, or nominal-specialization registry is introduced.

`INV-45` — mixed Tuple/Record destructuring and generalized product relationship remain unimplemented and unexposed.

---

# 10. Non-goals

Do not implement any of the following as part of P4:

```text
Record <: Tuple
Tuple <: Record
Tuple/Record shared public Product protocol
tuple pattern on Record
record pattern on Tuple
integer-labeled Record/Tuple unification
RecordFields <-> CompleteTypePack conversion
first-class RecordFields literal syntax
arbitrary `{**A, **B}` row concatenation
dependent field lookup types
general static label kind
new existential Tuple/Record supertype
per-specialization VM classes
runtime TypePack/LabeledTypePack/CompleteTypePack/RecordFields values
open compile-time execution
new user-definable intrinsic evaluators
unrelated Set spread completion
unrelated Map duplicate-policy redesign
```

If implementation appears to require one of these to make P4 work, STOP AND CONSULT; do not expand scope silently.

---

# 11. Expected impact map

## 11.1 Expected source areas

| Area | Likely paths | Why |
|---|---|---|
| tokens/lexer | `phalcom-ast/src/token.rs`, `lexer.rs` | `#{` changes Record→Set role |
| AST | `phalcom-ast/src/ast.rs` | product expansion contributions, Map construction, Record remainder syntax |
| parser | `phalcom-ast/src/parser.rs` | declarations, literals, types, patterns, aliases |
| kinds | `phalcom-semantic/src/types/kind.rs`, AST kind syntax | source CompleteTypePack/RecordFields |
| generic binders | `types/parameter.rs`, declaration/header builders | mode/kind decoupling |
| type application | `types/annotation.rs`, `store.rs`, `instantiation.rs` | domain-aware variadic binding; intrinsic family application |
| rows | `types/row.rs`, row inference | public rename and product-shape specialization |
| algebraic data | `data_semantics.rs`, `checker/algebraic_declaration.rs` | shape templates |
| construction/inference | checker call/associated/data construction paths | variadic constructor specialization |
| declarations | `declarations.rs` | true family kind, runtime projection, replace/refresh |
| type members | `types/type_member.rs`, `projection.rs`, member declaration checker | source/evaluator closure |
| metadata | semantic metadata + `phalcom-type-meta` | RecordFields spelling, mode+kind, shape template as required |
| incremental | fingerprints/dependency graph | product shape and aliases invalidate correctly |
| lowering | `phalcom-core/src/modules/semantic_lowering.rs` | expanded nominal layout and runtime owner |
| compiler | `compiler/lib/data_decl.rs`, `expr.rs`, `product_opt.rs` | construction and syntax |
| VM data | `data.rs`, `vm/data.rs` | shared class / exact descriptors |
| value repr | `value/repr.rs`, `value/mod.rs` | EmptyRecord immediate; Unit class change |
| product runtime | `product/` | empty normalization and views |
| primitives | `primitive/tuple.rs`, `primitive/record.rs` | immediate empty receiver support |
| Universe | `.ph` sources, `core_classes.rs`, `primitives.rs`, native meta | declarations and Unit class retirement |
| source corpus | `.ph`, tests, examples | syntax migration |
| LSP/editor | `phalcom-lsp`, semantic source index | new tokens/source identity |
| docs/specs | structural/domain/collection/VM specs | normative convergence |

## 11.2 Expected test areas

```text
phalcom-ast/tests/
phalcom-semantic/tests/semantic/foundations/
phalcom-semantic/tests/semantic/capabilities/
phalcom-semantic/tests/semantic/integration/
phalcom-semantic/tests/semantic/incremental/
phalcom-core/tests/core/collections/
phalcom-core/tests/core/compiler/
phalcom-core/tests/core/object_model/
phalcom-core/tests/language-corpus/
phalcom-lsp tests
phalcom-type-meta tests
native source/surface tests
```

## 11.3 Unexpected-touch rule

Mechanical adjacent edits are allowed. STOP AND CONSULT before entering a materially different architecture area—GC design, generalized object metaclass redesign, global type-system replacement, new runtime generic specialization model, or generalized product protocols—because P4 should not need them.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

- public `RecordFields` spelling back to `RecordRow`;
- `RecordRowId`'s internal separation from `TypeId`;
- the mode/kind independence rule;
- intrinsic Tuple/Record declaration shapes;
- nominal identity of user-expanded `data`;
- structural identity of intrinsic Tuple/Record applications;
- `Unit = Tuple<>`;
- `EmptyRecord = Record<>`;
- `Unit != EmptyRecord`;
- one Tuple runtime class and one Record runtime class;
- removal of Unit as an independent runtime class;
- distinct runtime representation/value for EmptyRecord;
- bare Record / explicit Map / hash-Set syntax;
- source authority of public intrinsic type members;
- `AssociatedSurface` as the single `::` lookup authority;
- shared-class/specialization-layout architecture for expanded nominal data;
- no existential Tuple/Record invention;
- non-goal boundary on mixed destructuring/product unification.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact Rust enum/struct names for product-shape template nodes;
- whether expansion-site identity is a dedicated newtype or an existing stable source-site identity;
- private helper decomposition for materializing expanded shapes;
- exact internal naming of `RecordRowMap` versus `RecordFieldsMap`;
- whether `GenericParameterMode` itself is renamed, provided semantics and public presentation are correct;
- exact EmptyRecord tag numeric value, provided the representation remains valid and stable for the patch;
- local location of runtime Tuple/Record empty receiver view helpers;
- exact AST node name for explicit Map construction, provided the AST preserves explicit construction identity and semantic validation is canonical;
- exact test function names and file splits;
- formatter-required local changes in touched files.

## 12.3 VERIFY-FIRST

The planning audit resolved several former uncertainties and they are now FIXED inputs to execution:

- the 16-byte `Value` representation has spare tag values; a dedicated immediate EmptyRecord tag is feasible without widening `Value`;
- top-level type aliases already retain `ImplementationKind::Intrinsic` in the AST/parser; P4 needs semantic/runtime ownership handling, not new alias syntax;
- Set literal AST/checker/compiler/VM machinery exists; parser syntax is inactive and `SetLiteralEntry::Expansion` still deliberately fails in codegen, so P4 activates only the ordinary `#{...}` Set literal unless a separately ratified spread rule exists;
- `TYPE_METADATA_SCHEMA_VERSION` is currently `8`; the public `RecordRow -> RecordFields` rename alone does not require a schema revision because durable kind identity remains `KindNode::RecordRow`;
- Record patterns currently carry only required entries, so `{..., **rest}` requires an explicit AST/semantic/executable-pattern remainder addition;
- Unit has source-authored `toString`/`hash` but no Unit-specific native primitive floor; those behaviors can move to Tuple while the dedicated Unit runtime class is retired;
- empty Record compilation currently emits `Value::unit()`, so P4 must change runtime representation, not merely semantic aliases.

The implementer must still verify these live-tree assumptions immediately before editing:

| Assumption | Verify at | If false |
|---|---|---|
| Tuple and Record native classes remain preinstalled | core Universe bootstrap | adapt mechanically if names moved; consult if ownership changed |
| `RuntimeDataRegistry` still supports exact descriptors per semantic owner/type | `phalcom-core/src/data.rs` | consult if specialization storage authority moved |
| row solver accepts fixed fields + symbolic tail/lacks constraints | semantic row subsystem | consult if a new row solver would be required |
| bare brace blocks remain removed from ordinary expression grammar | parser | if ordinary `{...}` blocks are live again, STOP AND CONSULT due grammar conflict |
| explicit `Map<K,V> {...}` can be represented by extending the existing `MapLiteralExpr`/generic-argument path without conflicting with nominal record construction | parser + checker | use an equivalent explicit Map-construction AST; consult if name-resolution semantics must change materially |
| native type metadata still references `UniverseKey::Unit` | native meta/surface | preserve the key only as a semantic alias identity if necessary; never retain a Unit runtime class merely for metadata |
| old corrective P4 records can be lifecycle-normalized without violating repository workflow/tooling | checkpoint/workflow/crosslinks | STOP AND CONSULT rather than create two stable P4 identities |

---

# 13. Global STOP / CONSULT triggers

In addition to repository-wide triggers, stop immediately if:

1. the live tree requires `RecordFields` to become a `TypeId` to support `**Fs`;
2. implementing user-expanded data appears to require one VM class per specialization;
3. the only apparent route is to synthesize fake `DataComponentId`s for expansion-produced fields;
4. Unit cannot report Tuple runtime class without changing the object model beyond value classification/dispatch;
5. EmptyRecord cannot be represented distinctly from Unit without changing a frozen value-layout contract;
6. a runtime-dependent Tuple operation appears to require a new existential type to type-check;
7. `Map {...}` cannot be parsed without changing the accepted closure/block grammar;
8. current canonical Map shadowing/name-resolution policy conflicts with contextual builtin Map construction;
9. source intrinsic type-member declarations cannot be made authoritative without a second lookup table;
10. `Record<>` zero-argument variadic application cannot be expressed through the existing generic/static argument machinery;
11. expanded nominal Record specialization cannot reuse `StaticArgument::RecordRow`;
12. a metadata schema change cannot preserve older-bundle decoding/validation policy;
13. lifecycle normalization of the already-created P4 records would destroy historical evidence or conflict with workflow tooling;
14. any test oracle requires `Unit == EmptyRecord`, a separate Unit runtime class, `#{...}` Records, or bare `{...}` Maps after migration;
15. implementation scope expands into generalized Tuple/Record product relation or mixed destructuring.

A triggered consultation cannot be self-waived.

---

# 14. Debugging budget

## Mechanical failures

Allow up to three coherent correction cycles for imports, enum exhaustiveness, renamed AST variants, borrow cleanup, or test-selector mistakes.

Each cycle:

```text
inspect
identify one concrete cause
make one coherent correction
rerun the smallest discriminator
```

## Semantic failures

Before a nontrivial semantic correction, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same semantic cause. If materially the same failure persists, STOP AND CONSULT.

## Architectural failures

Zero speculative architecture patches. Escalate immediately.

Do not rerun unchanged failures when nothing causal changed.

---

# 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| CV-01 | lifecycle identity | INV-44 | only one canonical `LANG007.C3.P4` remains |
| CV-02 | kind surface | INV-01 | all four structural domains resolve with correct kind |
| CV-03 | domain distinction | INV-01/02/04 | LabeledTypePack is rejected where RecordFields required and vice versa |
| CV-04 | mode-kind independence | INV-03/04 | `**Ls: LabeledTypePack` and `**Fs: RecordFields` both publish |
| CV-05 | default binder compatibility | INV-03 | unannotated `**Fs` defaults to RecordFields; explicit `**Ls: LabeledTypePack` remains valid |
| CV-06 | CompleteTypePack source | INV-06 | no TupleEntries semantic domain |
| CV-07 | Tuple declaration parse | INV-07 | `data X<***Es: CompleteTypePack>(***Es)` |
| CV-08 | Record declaration parse | INV-07 | `data X<**Fs: RecordFields>{**Fs}` |
| CV-09 | product expansion placement | INV-07 | illegal complete/labeled placement rejected |
| CV-10 | user Tuple nominal identity | INV-09 | `MyTuple<***(Int,String)> != (Int,String)` |
| CV-11 | user Record nominal identity | INV-09 | `MyRecord<...> != {...}` |
| CV-12 | expanded constructor inference | INV-07/20 | value arguments infer complete pack / row |
| CV-13 | fixed+expanded Tuple | INV-07 | prefix + pack specializes layout correctly |
| CV-14 | fixed+expanded Record | INV-28 | fixed labels + row tail, duplicate/lacks enforcement |
| CV-15 | heterogeneous nominal args | INV-02/09 | RecordFields specialization uses `StaticArgument::RecordRow` |
| CV-16 | shared behavior class | INV-19/20 | two specializations have different exact type/layout, same class |
| CV-17 | expanded field lowering | INV-08/20 | access uses specialized logical slot, no fake source component |
| CV-18 | Tuple family kind | INV-21 | bare Tuple type form is CompleteTypePack -> Type |
| CV-19 | Record family kind | INV-21 | bare Record type form is RecordFields -> Type |
| CV-20 | runtime owner separation | INV-22/43 | native Tuple/Record import works while static family kind stays higher-kinded |
| CV-21 | shell refresh | INV-23 | refresh preserves projections; authoritative replace does not inherit stale projection |
| CV-22 | zero variadic application | INV-11/12 | `Tuple<>`, `Record<>` form correctly |
| CV-23 | Unit alias | INV-11/16 | Unit resolves exactly to canonical store Unit, no nominal Unit data |
| CV-24 | EmptyRecord alias | INV-12/13 | EmptyRecord resolves to empty Record type |
| CV-25 | runtime class Unit | INV-14/16 | `().class == Tuple` |
| CV-26 | runtime class EmptyRecord | INV-15/17 | `{}.class == Record` |
| CV-27 | zero value distinction | INV-13/18 | `() !== {}` and family equality/hash remain distinct |
| CV-28 | empty Tuple behavior | INV-14 | size/iteration/string/hash dispatch through Tuple |
| CV-29 | empty Record behavior | INV-15 | size/get/string/hash/equality dispatch through Record |
| CV-30 | Record literal syntax | INV-24 | `{name: value, **r}` parses/types/runs |
| CV-31 | Record type syntax | INV-24 | `{name: String, **Fs}` forms row type |
| CV-32 | Record pattern syntax | INV-24/45 | `{name, **rest}` works; no tuple/record mixing |
| CV-33 | Map syntax | INV-25 | bare old Map syntax no longer maps; explicit Map does |
| CV-34 | typed Map syntax | INV-25 | `Map<K,V>{...}` supplies expected generics |
| CV-35 | Set syntax | INV-26 | `#{}`, `#{1,2}` produce Set |
| CV-36 | old syntax diagnostics | INV-24/25/26 | old Record/Map forms receive migration-quality diagnostics where distinguishable |
| CV-37 | source intrinsic identity | INV-30/31 | go-to-definition/member declaration identity points to Universe source |
| CV-38 | Tuple projections | INV-32/33 | Entries/Positionals/Labeled/Element concrete + symbolic |
| CV-39 | Record projections | INV-34/35 | Fields/Value concrete + open symbolic |
| CV-40 | typed runtime API | INV-36/37 | no known-value access degrades to Dynamic; no ill-kinded bare family |
| CV-41 | metadata roundtrip | INV-40 | mode+kind, CompleteTypePack, RecordFields, aliases roundtrip |
| CV-42 | cold/incremental parity | INV-41 | edits to binder kind/shape/member/literal syntax converge |
| CV-43 | LSP/source index | INV-42 | hover/definition/rename reflect new source names and syntax |
| CV-44 | runtime owner lowering | INV-43 | projected runtime owner drives class binding |
| CV-45 | no scope leak | INV-45 | no mixed destructuring/product protocol added |
| CV-46 | compatibility import | INV-40/44 | older supported metadata schemas retain defined compatibility behavior |
| CV-47 | broad closure | all | workspace/check/test/lint/fmt classification recorded once |

---

# 16. Verification execution budget

## BUILD MODE

Use only T0/T1 discriminators. Compile the owning crate when structural Rust changes make that useful. Do not run workspace-wide tests during parser/semantic migration.

Typical BUILD commands:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic -- --list
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core -- --list
```

Do not mechanically run all three after every task.

## STABILIZE MODE

At coherent gates run directly affected suites:

- AST data/type/literal/pattern parser tests;
- semantic generic application, type annotations, type-member closure, row polymorphism, data declaration/construction, metadata, incremental tests;
- core product/value/object model/collections/native-surface tests;
- LSP source-index/navigation tests;
- selected language corpus groups migrated by P4.

## CERTIFY MODE

Because P4 re-closes C3 after a cross-layer semantic migration, final certification should attempt once:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Do not claim `RELEASE_COMPLETE` unless the full named gates are actually green. If inherited baseline failures remain, classify them and use `BASELINE_BLOCKED` or `FOCUSED_TESTED` truthfully.

## Explicitly deferred during BUILD

Do **not** run:

```text
cargo test --workspace
workspace clippy
full language corpus
full LSP extension-host suite
release packaging
```

until named stabilization/certification gates.

---

# 17. Baseline / unrelated failure policy

Classify unexpected failures:

```text
A — definitely caused by P4
B — probably caused by P4
C — unclear
D — clearly unrelated or predecessor baseline
```

- A/B are active responsibility.
- C receives one bounded causal-classification pass.
- D is recorded and immediately deferred.
- Do not weaken assertions, skip tests, or restore legacy syntax merely to make a broad suite green.
- Any predecessor failure that touches syntax or semantics P4 intentionally changes must be reclassified; it cannot be called baseline merely because it existed before.
- Preserve the prior checkpoint ledger's formatter/AST/impl-query/runtime baseline history, but update it with fresh evidence when final broad gates run.


# 18. Tasks

The task order is dependency-sensitive. Do not reorder semantic/runtime tasks merely to make an intermediate build green. Several intermediate states are intentionally inconsistent until the paired consumer task lands.

---

## T1 — Normalize the C3 lifecycle record and reopen the checkpoint

### Purpose

Make `LANG007.C3.P4` a unique stable implementation-plan identity before any code work begins. The repository currently contains a completed corrective closure already labeled P4. The newly ratified work is the actual accepted P4 scope and cannot coexist with a second stable P4 plan.

### Preconditions

- Read the lifecycle convention and current `LANG007.C3-CHECKPOINT.md`.
- Inspect every reference to the existing corrective P4 plan/walkthrough/handoff.
- Confirm whether repository automation assumes immutable plan IDs.

### Consumes

- completed P3 records;
- the existing corrective implementation at commit `82c9dd038...`;
- existing P4 corrective walkthrough/handoff;
- lifecycle convention.

### Produces

- exactly one canonical plan identity `LANG007.C3.P4` for this plan;
- preserved historical evidence for the old corrective closure, reclassified as a P3 corrective/addendum record rather than a numbered P4 plan;
- checkpoint status changed from closed to reopened/active for the accepted structural-completion objective;
- no loss of commit/evidence history.

### Files and symbols

**Read:**

```text
docs/implementation/LANG007/LANG007.C3/LANG007.C3-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P3-*.md
docs/implementation/LANG007/LANG007.C3/LANG007.C3.P4-*.md
docs/workflow/implementation-record-lifecycle-convention.md
```

**Modify/rename as appropriate:**

- existing corrective P4 records;
- checkpoint plan ledger;
- crosslinks inside C3 implementation records.

### Required implementation shape

The corrective work must remain historically visible, including:

- what defect it corrected;
- implementation revision;
- focused verification;
- stable `DeclarationTypeBinding` model it established.

It should become a **P3 corrective/addendum record** or equivalent non-numbered corrective record. Do not invent a second numbered plan identity merely to avoid renaming.

A suitable historical shape is conceptually:

```text
LANG007.C3.P3 corrective closure:
    intrinsic semantic-application/runtime-owner separation
    implementation revision: 82c9dd...
    verification: FOCUSED_TESTED
```

Then the plan ledger can contain:

```text
P1 COMPLETE
P2 COMPLETE
P3 COMPLETE (+ corrective closure recorded under P3)
P4 IN_PROGRESS — intrinsic structural data semantic completion
```

### Forbidden approaches

- no two `id: LANG007.C3.P4` records;
- no silent P5 substitution while telling the user work is P4;
- no deletion of old corrective evidence;
- no history rewrite of Git commits;
- no architecture edits in T1.

### Test changes required

None.

### Tests to run now

No compile/test commands. Use repository text/reference checks only.

```sh
rg -n 'LANG007\.C3\.P4' docs/implementation/LANG007/LANG007.C3
git diff --check
```

### Acceptance

- one stable P4 identity;
- old corrective evidence preserved and correctly linked;
- checkpoint explicitly records P4 as active;
- no dangling links inside C3 docs.

### Local STOP / CONSULT triggers

STOP if the lifecycle/tooling contract forbids reclassifying the existing P4 stable ID or if preserving links would require destructive historical rewriting. The architect must then choose a repository-level numbering amendment; Luna must not decide that alone.

### Checkpoint update

Mandatory. This is a durable lifecycle event.

---

## T2 — Complete structural kind syntax and decouple expansion mode from semantic domain

### Purpose

Make the type system capable of expressing the ratified binders honestly:

```phalcom
<***Es: CompleteTypePack>
<**Ls: LabeledTypePack>
<**Fs: RecordFields>
```

without encoding semantic kind in the number of stars.

### Preconditions

- T1 complete.
- Confirm the current AST `KindSyntax`, `ExpansionMode`, generic-parameter header structures, semantic `KindData`, and metadata `KindNode`.
- Confirm `KindId::RECORD_ROW` and `KindId::COMPLETE_TYPE_PACK` identities remain stable.

### Consumes

- existing `KindData::CompleteTypePack`;
- existing `KindId::RECORD_ROW`;
- existing `GenericParameterMode`;
- current `source_parameter_kind` bootstrap logic;
- metadata kind representation.

### Produces

- public kind spellings:
  ```text
  Type
  TypePack
  LabeledTypePack
  CompleteTypePack
  RecordFields
  ```
- explicit compatibility validation between `ExpansionMode` and kind;
- preserved defaults for omitted kind annotations;
- `RecordFields` presentation over the existing row kind;
- no `TupleEntries` kind facade.

### Files and symbols

**Read/modify:**

```text
phalcom-ast/src/ast.rs
  KindSyntax
  GenericParameterSyntax
  ExpansionMode

phalcom-ast/src/parser.rs
  parse_kind_expression
  parse_generic_parameters

phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/parameter.rs
  GenericParameterMode
  GenericBinderHeader
  GenericSignature::validate_publishable

phalcom-semantic/src/declarations.rs
  source_parameter_kind
  insert_source_nominal
  intrinsic_static_facade_kind

phalcom-semantic/src/metadata/
phalcom-type-meta/src/kind.rs

phalcom-core/core/future/type.telos
  if this remains the public reflection vocabulary source
```

### Required implementation shape

#### T2.1 — Add `CompleteTypePack` to source kind syntax

If built-in kinds are AST variants, add the matching variant. `resolve_kind_syntax` must return the existing `KindId::COMPLETE_TYPE_PACK`, not allocate a nominal static domain.

#### T2.2 — Rename public `RecordRow` to `RecordFields`

Source parser/presentation accepts `RecordFields`.

Internally:

```text
RecordFields source kind
    ↓
KindId::RECORD_ROW
    ↓
RecordRowId / RecordRowData
```

Do not rename every internal Rust type simply for cosmetic consistency. Rename public-facing diagnostics/display names and source anchors. Internal renaming is mechanically optional.

#### T2.3 — Remove TupleEntries as a domain

Canonical Universe source declares:

```phalcom
intrinsic type data CompleteTypePack
```

instead of:

```phalcom
intrinsic type data TupleEntries
```

`Tuple::Entries` later returns `CompleteTypePack`.

No second `StaticDomainId` or alias-kind wrapper is permitted.

#### T2.4 — Define mode/domain compatibility independently

Create one canonical validator, conceptually:

```rust
fn expansion_mode_accepts_kind(
    mode: GenericParameterMode,
    kind: KindId,
) -> bool
```

Required compatibility:

```text
Single          → any explicitly valid parameter domain
Positional/*    → TypePack
Labeled/**      → LabeledTypePack | RecordFields
Complete/***    → CompleteTypePack
```

Do not scatter this matrix across parser, header publication, inference, and metadata validation.

#### T2.5 — Explicit kind wins over star-derived default

Current bootstrap logic returns a kind immediately from `parameter.mode`. Reverse that relationship:

```text
if explicit kind annotation:
    resolve it
    validate compatibility with mode
else:
    apply the default domain for the mode
```

Defaults:

```text
T     => Type
*Ts   => TypePack
**Fs  => RecordFields
***Es => CompleteTypePack
```

Therefore:

```phalcom
**Fs: RecordFields
```

is valid and unambiguous.

#### T2.6 — Preserve Single(pack) distinction

These remain distinct:

```phalcom
<P: TypePack>    // one TypePack argument
<*P: TypePack>   // positional expansion binder
```

Metadata/fingerprints must preserve this distinction.

#### T2.7 — Public reflection vocabulary

Where the current reflection vocabulary publishes `GenericParameterMode` cases named `PositionalPack`, `LabeledPack`, `CompletePack`, migrate the public concept toward:

```phalcom
type data ExpansionMode {
    Single
    Positional
    Labeled
    Complete
}
```

The internal Rust enum name is mechanically flexible. Do not block the main implementation on renaming every private symbol.

### Forbidden approaches

- no `RecordFields` `TypeId`;
- no `LabeledTypePack == RecordFields`;
- no parser-only special case for `**Fs`;
- no string-based `"RecordFields"` checks after semantic kind resolution;
- no new kind identity for `TupleEntries`.

### Test changes required

Add AST/semantic tests covering CV-02..CV-06:

```phalcom
type A<***Es: CompleteTypePack> = ...
type B<**Ls: LabeledTypePack> = ...
type C<**Fs: RecordFields> = ...
type D<P: TypePack> = ...
```

Negative cases:

```phalcom
type Bad<*R: RecordFields> = ...
type Bad2<***R: RecordFields> = ...
type Bad3<**P: TypePack> = ...
```

Test that unannotated `**Fs` defaults to RecordFields and explicitly annotated
`**Ls: LabeledTypePack` remains valid.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast kind_expressions
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic generic_application
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic type_model
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

Use exact filters confirmed by `-- --list` if names drift.

### Tests explicitly deferred

- core runtime;
- full metadata;
- LSP;
- workspace.

### Acceptance

- both labeled domains work with `**`;
- no mode forces its kind when an explicit compatible kind exists;
- CompleteTypePack is source-nameable;
- RecordFields is source-nameable;
- TupleEntries domain is gone from canonical source model.

### Local STOP / CONSULT triggers

STOP if changing mode/kind independence requires weakening generic signature stable identity or if metadata cannot encode the pair independently without a schema decision.

### Checkpoint update

Record the mode/domain compatibility matrix as a durable invariant.

---

## T3 — Add typed product-shape expansion to `data` syntax and AST

### Purpose

Make product-shape expansion an ordinary source feature, so intrinsic Tuple/Record declarations do not require an opaque facade grammar exception.

### Preconditions

- T2 mode/kind model exists.
- Current data tuple/record parsing and `AlgebraicShapeSyntax` verified.

### Consumes

- `ProductShapeSyntax`;
- `DataComponentSyntax`;
- `ExpansionMode`;
- generic parameter syntax;
- current `parse_data_tuple_shape` / braced data body parser.

### Produces

- AST preserving fixed components and expansion contributions;
- tuple-shaped `data ...(*Ts, **Ls, ***Es)` grammar where legal;
- record-shaped `data ... { **Fs }` grammar;
- intrinsic provenance orthogonal to product shape;
- parser diagnostics for illegal placement/domain modes.

### Files and symbols

```text
phalcom-ast/src/ast.rs
  AlgebraicShapeSyntax
  ProductShapeSyntax
  DataComponentSyntax
  new contribution/expansion syntax

phalcom-ast/src/parser.rs
  parse_data_inner
  parse_data_tuple_shape
  parse_algebraic_braced_body
  data product diagnostics

phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/parser.rs
snapshot tests that encode AST shape
```

### Required implementation shape

Replace the fixed-component-only AST concept with an ordered contribution model. Conceptually:

```rust
enum DataProductContributionSyntax {
    Component(DataComponentSyntax),
    Expansion(DataProductExpansionSyntax),
}

struct DataProductExpansionSyntax {
    mode: ExpansionMode,
    operand: TypeAnnotation, // normally a bound static parameter reference
    range: SourceRange,
}
```

The exact operand representation may use an existing static/type-form node, but it must preserve source identity and kind-check later; do not reduce it in the parser.

`ProductShapeSyntax` becomes conceptually:

```rust
Tuple {
    contributions: Vec<DataProductContributionSyntax>,
    ...
}

Record {
    contributions: Vec<DataProductContributionSyntax>,
    ...
}
```

`components()` convenience methods that promise every entry is fixed must be retired or replaced by explicit fixed-component iteration.

#### T3.1 — Runtime intrinsic data may have normal product shape

The parser should produce:

```text
implementation = Intrinsic
shape = Product(...)
```

for:

```phalcom
intrinsic data Tuple<***Es>(***Es)
intrinsic data Record<**Fs> { **Fs }
```

Do **not** convert them into `IntrinsicFacade`.

Retain `IntrinsicFacade` only where it still correctly represents an opaque intrinsic static declaration with no authored shape, such as primitive static kind anchors if needed. Runtime intrinsic structural data must no longer depend on it.

#### T3.2 — Tuple expansion placement

Enforce structural placement independent of semantic kind resolution:

- `*` is positional contribution.
- `**` begins/occupies labeled contribution phase.
- `***` may contain both lanes; after a complete expansion, no later positional contribution is permitted.
- fixed positional entries cannot follow labeled entries.
- fixed labeled entries remain legal after positional contributions.
- duplicate fixed labels can be caught syntactically/semantically as today; duplicates involving symbolic expansions defer to specialization/lacks checking.

Examples:

```phalcom
data Good<*Ts: TypePack>(Int, *Ts, String)
data Good2<**Ls: LabeledTypePack>(Int, **Ls)
data Good3<***Es: CompleteTypePack>(Int, ***Es)

data Bad<***Es: CompleteTypePack>(***Es, Int) // reject
data Bad2<**Ls: LabeledTypePack>(**Ls, Int)   // reject
```

If existing generic-signature lane ownership prohibits multiple pack binders, preserve that restriction; this task does not redesign binder multiplicity.

#### T3.3 — Record expansion

Record-shaped data permits fixed fields plus P4's row expansion:

```phalcom
data Envelope<**Fs: RecordFields> {
    requestId: RequestId
    **Fs
}
```

P4 requires at most one structural RecordFields expansion tail. Do not implement arbitrary row concatenation.

Only labeled expansion mode is legal in a Record product body.

#### T3.4 — Type-only data guard

Do not accidentally make `type data` product expansion mean runtime product fields. Preserve the existing static-domain restrictions. If generic expanded `type data` is not already specified, reject it explicitly rather than guessing.

### Forbidden approaches

- no AST desugaring of `***Es` into fake components;
- no use of `IntrinsicFacade` to hide Tuple/Record shape after this task;
- no parser kind-solving;
- no arbitrary multiple Record row spreads;
- no generalized product protocol.

### Test changes required

Cover CV-07..CV-09.

Include source-range/AST snapshot checks proving:

- intrinsic + product shape coexist;
- ordinary + product expansion coexist;
- expansion mode/range retained;
- illegal tuple ordering fails at the right token;
- Record rejects `*`/`***` shape expansion.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast data_syntax
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast parser
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast
```

Do not run semantic/core tests until T4 consumes the new AST.

### Acceptance

The AST can faithfully represent all canonical P4 source declarations and user-expanded data declarations without manufacturing semantic components.

### Local STOP / CONSULT triggers

STOP if braced algebraic sum/product disambiguation cannot recognize `**Fs` without changing existing sum-case syntax semantics.

### Checkpoint update

Record the product-shape AST/template syntax interface if its shape is durable.

---

## T4 — Build semantic product-shape templates and domain-aware specialization

### Purpose

Replace the fixed-component semantic assumption with one canonical declaration template that can specialize fixed and expanded Tuple/Record products without losing nominal identity or row/pack domain identity.

### Preconditions

- T2 kind/mode semantics.
- T3 expansion-aware AST.
- existing C2/C3 pack/row application and inference machinery live.

### Consumes

- `AlgebraicDef` product contributions;
- `DataInfo`, `DataComponentSemantic`, `DataConstructorSignature`;
- `StaticArgument`;
- `GenericInstantiation`;
- `CompleteTypePack` projection helpers;
- row solver/lacks machinery.

### Produces

- declaration-owned product-shape template;
- stable expansion-site provenance;
- specialization/materialization helper;
- variadic constructor signature/template;
- ordinary nominal `MyTuple<***Es>` / `MyRecord<**Fs>` type formation;
- intrinsic Tuple/Record product semantics available for later T6;
- no fake `DataComponentId`s.

### Files and symbols

```text
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/checker/algebraic_declaration.rs
phalcom-semantic/src/types/{store.rs,instantiation.rs,annotation.rs}
phalcom-semantic/src/checker/{call.rs,composition.rs,row_inference.rs}
phalcom-semantic/src/identity.rs
semantic snapshot/db products and fingerprints
```

### Required implementation shape

#### T4.1 — Separate source-fixed components from shape contributions

Extend `DataInfo` or a sibling product so declaration semantics retain:

```rust
shape: DataShape
shape_template: DataProductShapeTemplate
fixed_components: ... // if useful as an index
constructor_template: DataConstructorTemplate
```

Do not force expanded contributions into `DataInfo.components`.

A stable template must preserve:

- contribution order;
- fixed component IDs;
- external labels;
- expansion source identity;
- expansion mode;
- operand parameter/static term;
- expected domain.

#### T4.2 — Domain-aware nominal root application

Current code that conceptually does:

```rust
let param_forms = generic_params.map(store.parameter_form);
store.apply_type_form(root_form, &param_forms)
```

is invalid for RecordFields binders.

Build canonical static arguments per parameter kind:

```text
kind Type / pack / static-domain term
    → StaticArgument::Term(parameter form)

kind RecordFields
    → parameter-only canonical RecordRowId
    → StaticArgument::RecordRow(row)
```

Use `apply_static_form`, not `apply_type_form`, for heterogeneous nominal specialization.

This is required for both ordinary `MyRecord` and intrinsic Record family environments.

#### T4.3 — Product shape materializer

Create one helper/service, conceptually:

```rust
materialize_data_product_shape(
    template,
    generic_instantiation/static arguments,
    mode
) -> ProductShapeMaterialization
```

It returns either:

```text
Concrete:
    ordered concrete components/fields + exact types + mappings

Symbolic:
    valid shape with unresolved pack/row contribution

Invalid:
    kind/duplicate/lacks/placement failure
```

Do not call symbolic shape `Dynamic`.

Tuple concrete expansion:

```text
CompleteTypePack
    → positionals + labeled
    → exact ordered component sequence
```

Record concrete expansion:

```text
RecordRowId
    → exact known fields (plus no unresolved tail for concrete constructor layout)
    → stable logical label order + source/presentation mapping
```

Open symbolic rows remain valid semantic shapes for type checking but cannot produce a concrete runtime layout until solved.

#### T4.4 — Constructor template

A shape-generic constructor cannot be represented only as fixed:

```rust
Box<[DataConstructorParameter]>
```

Add a constructor parameter template/contribution model parallel to product shape.

When checking a construction:

- match fixed components first according to existing declaration rules;
- collect pack/labeled/row residual contributions;
- solve generic pack/row binders through the existing inference coordinator;
- materialize the resulting exact constructor parameter list and result type;
- preserve source argument evaluation order separately from logical storage order.

The data constructor uses a narrow shape adapter at the shared callable
application boundary. The adapter consumes `DataConstructorTemplate`, maps
each source argument to a fixed formal or one expansion contribution, and
feeds constraints into the existing generic inference coordinator. The shared
application loop analyzes each source expression once, in source order. It
must not widen `bind_static_arguments` to generally accept runtime expansions,
or encode expansion contributions as fake callable parameters or
`DataComponentId`s.

For `CompleteTypePack`, the adapter aggregates positional and labeled
argument terms into `InferenceTerm::CompleteTypePack` and constrains the
existing binder variable. For `RecordFields`, it submits residual labeled
terms as a row equation through `GenericApplicationSession` and retains
`StaticArgument::RecordRow` through result materialization. Fixed Record fields
add row-lacks constraints before solving. This is an adapter over the shared
solver, not a constructor-local inference engine.

Do not create a second variadic constructor solver.

#### T4.5 — User MyTuple inference

Required behavior:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)

let x = MyTuple(1, "a")
```

infers:

```text
Es = CompleteTypePack(
    positionals = [Int, String],
    labeled = []
)

typeof(x) = MyTuple<***(Int, String)>
```

Labeled entries similarly infer the labeled lane.

The constructor adapter analyzes each argument once and uses the existing
inference session to bind `Es`; it does not synthesize one fake formal
parameter per expanded entry.

#### T4.6 — User MyRecord inference

Required behavior:

```phalcom
data MyRecord<**Fs: RecordFields> {
    **Fs
}

let r = MyRecord {
    name: "Altun",
    age: 42,
}
```

infers one RecordFields/row argument, yielding nominal:

```text
MyRecord<**{name:String, age:Int}>
```

The row remains a `RecordRowId`/`StaticArgument::RecordRow` from constraint
collection through nominal result materialization. Fixed fields are matched
first; only residual labels enter `Fs`, with fixed labels registered as row
lacks.

Internally this is `TypeData::Applied` with `StaticArgument::RecordRow`.

For mixed fixed+spread declaration:

```phalcom
data Envelope<**Fs: RecordFields> {
    id: Id
    **Fs
}
```

construction solves `Fs` from the residual fields and establishes `Fs lacks id`.

#### T4.7 — Duplicate/lacks behavior

Fixed-vs-expansion duplicate field/label conflicts are not source-order overwrites.

- concrete conflict: diagnostic;
- symbolic Record tail: row-lacks obligation;
- symbolic labeled Tuple pack: corresponding uniqueness proof/check if existing pack machinery supports it; otherwise reject a declaration whose uniqueness cannot be established rather than allow ambiguous identity.

Do not silently drop or overwrite.

### Forbidden approaches

- no one `DataComponentId` per specialization-generated field;
- no `RecordFields -> LabeledTypePack` conversion;
- no row-to-TypeId wrapper;
- no constructor-local second inference engine;
- no eager runtime layout allocation in semantic analysis;
- no approximation of unresolved expanded shape as fixed empty shape.

### Test changes required

Cover CV-10..CV-15.

Add semantic regressions for:

```text
MyTuple nominal identity
MyRecord nominal identity
zero/one/many expanded components
fixed prefix + pack
fixed record fields + row
duplicate field conflict
open symbolic shape retained
row StaticArgument identity
cold result stable
```

### Tests to run now

Use narrow semantic filters:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic algebraic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic generic_application
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record_row
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

Confirm filters select tests.

### Tests explicitly deferred

- runtime construction;
- syntax corpus migration;
- LSP;
- workspace.

### Acceptance

A shape-generic ordinary `data` declaration is semantically valid, produces nominal specialized types through existing static-argument machinery, and yields a canonical concrete/symbolic product shape without fake components.

### Local STOP / CONSULT triggers

STOP if a row-bound nominal `TypeData::Applied` cannot round-trip/materialize through existing `StaticArgument::RecordRow` without changing its canonical identity contract.

### Checkpoint update

Record the product-shape template and specialization service as stable takeover interfaces.

---

## T5 — Make expanded nominal data runtime-correct with shared behavior class and exact specialization layouts

### Purpose

Carry T4's specialized product shape into runtime data construction without introducing per-specialization classes or one fixed declaration layout.

### Preconditions

- T4 semantic product materialization.
- `RuntimeDataRegistry` and exact runtime type recipe path verified.

### Consumes

- specialized constructor/lowering facts from T4;
- `RuntimeDataRegistry`;
- `DataDeclarationLoweringSpec`;
- `DataConstructionLoweringSpec`;
- runtime type recipes and product layouts.

### Produces

- one behavior class per nominal shape-generic `data` declaration;
- exact per-specialization descriptors/layouts;
- construction mapping preserving source evaluation order;
- specialization-aware expanded-field access;
- fixed-only data behavior unchanged.

### Files and symbols

```text
phalcom-core/src/modules/semantic_lowering.rs
  DataDeclarationLoweringSpec
  DataConstructionLoweringSpec

phalcom-core/src/data.rs
  RuntimeDataRegistry
  RuntimeDataDescriptor

phalcom-core/src/vm/data.rs
phalcom-core/src/compiler/lib/data_decl.rs
compiler data construction lowering
runtime typing metadata helpers
data component access paths
```

### Required implementation shape

#### T5.1 — Separate behavior-class registration from base layout registration

Current `register_data_from_spec` couples:

```text
create behavior class
+
register one declaration descriptor/layout
+
install fixed getters
```

Generalize the existing registry—not a new registry—so it can answer:

```text
behavior_class_for_declaration(owner)
```

independently of whether a fixed base descriptor exists.

A shape-generic declaration may have:

```text
behavior class: yes
declaration-level fixed layout: no
exact descriptors: created per specialization
```

Fixed-only data may still register its base descriptor eagerly.

#### T5.2 — Exact specialization descriptor

At each concrete construction, semantic lowering provides:

```text
owner
exact type recipe
exact layout
argument-to-logical-component mapping
```

`get_or_bind_data_descriptor`:

1. obtains the shared behavior class for `owner`;
2. materializes exact runtime type;
3. interns/reuses `(owner, exact_type)` descriptor;
4. registers exact layout;
5. constructs value.

Do not derive layout from the class.

#### T5.3 — Expanded component access

Fixed declared components:

```text
have DataComponentId
may retain declaration-owned generated accessor
```

Expanded specialization-only components:

```text
have no fake DataComponentId
are resolved from exact specialized product shape
lower directly to logical slot/field access
```

The semantic checker/source index should distinguish these provenance classes. A generated field from `**Fs` may navigate to the expansion site/binder where useful, but must not claim a nonexistent source field declaration.

#### T5.4 — Runtime class identity regression

For example:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)

let a = MyTuple(1)
let b = MyTuple(1, "x")
```

must satisfy:

```text
a.type != b.type
a.class == b.class == MyTuple
layouts may differ
```

Likewise MyRecord.

#### T5.5 — No monomorphized classes

Exact specialization descriptors are allowed. Exact specialization classes are not.

### Forbidden approaches

- no `HashMap<ExactType, ClassId>` specialization class cache;
- no second RuntimeDataRegistry;
- no layout stored as an intrinsic property of behavior class for expanded data;
- no fake getters installed globally for specialization-only fields;
- no source AST re-walk in compiler to derive specialized layout.

### Test changes required

Cover CV-16/CV-17.

Add:

- runtime registry unit test: one class, multiple exact descriptors/layouts;
- source end-to-end MyTuple two arities;
- source end-to-end MyRecord two field sets;
- fixed field + expanded field access;
- repeated construction reuses exact descriptor where appropriate;
- generic construction through a generic callable retains class identity.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core data
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core product
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Use exact filters after listing.

### Acceptance

Shape-generic nominal data constructs and reflects correctly at runtime with shared behavior class and specialization-specific exact layout.

### Local STOP / CONSULT triggers

STOP if exact layout variation cannot be represented without changing the runtime type environment ownership model or introducing per-specialization classes.

### Checkpoint update

Record the shared-class/exact-layout invariant and registry interface.

---

## T6 — Complete declaration type-family/runtime projections and consolidate intrinsic application

### Purpose

Remove the remaining C3 workaround in which intrinsic Tuple/Record static forms are flattened to proper `Type` so runtime native ownership works.

### Preconditions

- T2 domain-aware binders.
- T4 product-shape semantic declaration.
- existing C3 corrective `DeclarationTypeBinding` split.

### Consumes

- `DeclarationTypeInfo`;
- `DeclarationTypeBinding`;
- `IntrinsicTypeApplication`;
- `IntrinsicRuntimeOwner`;
- type annotation lowering;
- native import/runtime facade lowering.

### Produces

- true higher-kinded static family forms for Tuple and Record;
- independent runtime owner projection;
- consolidated intrinsic application contract;
- explicit declaration replace versus refresh APIs;
- canonical Universe identity dispatch;
- removal of Unit intrinsic data application/owner.

### Files and symbols

```text
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/native.rs
phalcom-semantic/src/core_surface.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/data_decl.rs
phalcom-core/src/native/source.rs

incremental declaration fingerprints
```

### Required implementation shape

#### T6.1 — Preserve family kind

Bootstrap must no longer do the equivalent of:

```rust
info.form = store.nominal_type(declaration);
info.kind = KindId::TYPE;
info.generic_signature = None;
```

for intrinsic Tuple/Record.

Instead source binder publication establishes:

```text
Tuple form kind  = CompleteTypePack -> Type
Record form kind = RecordFields -> Type
```

and retains the generic application contract.

The runtime projection still points to the preinstalled VM class.

#### T6.2 — Intrinsic family application contract

Centralize special family application, conceptually:

```rust
struct IntrinsicApplicationContract {
    owner: IntrinsicTypeApplication,
    parameters: ... // modes + kinds
    result_kind: KindId,
}
```

Reducer behavior:

```text
Tuple:
    source variadic contributions
      → one CompleteTypePack
      → tuple_from_packs
      → Unit when empty

Record:
    source labeled RecordFields contributions
      → one RecordRowId
      → TypeData::Record(row)
      → empty Record when empty
```

Canonical and scoped type lowering must call the same semantic contract/helpers. Do not retain separate hand-written Tuple/Record application algorithms that can drift.

#### T6.3 — Zero variadic aggregation

For a variadic binder with no source contributions:

```text
***Es: CompleteTypePack → empty CompleteTypePack
**Fs: RecordFields      → empty RecordFields / closed empty row
```

This is required for:

```phalcom
Tuple<>
Record<>
```

and user-defined zero-specialization families.

The aggregation result is selected by **mode + expected domain**, not mode alone.

#### T6.4 — Remove Unit intrinsic application

Delete/retire:

```text
IntrinsicTypeApplication::Unit
IntrinsicRuntimeOwner::Unit
```

once Unit is an alias in T9/T10. If staging requires temporary compatibility, mark it explicitly and remove before G3.

Tuple empty reduction remains `store.unit()`.

#### T6.5 — Explicit replace versus refresh

Replace ambiguous:

```rust
DeclarationTypeTable::insert(...)
```

behavior that conditionally inherits old intrinsic projections with two explicit operations, conceptually:

```rust
replace_authoritative(info)
refresh_shell_preserving_registered_projections(info)
```

or equivalent.

Use:

- authoritative replacement when semantic identity/binding itself is replaced;
- refresh only for query/header rebuilds that are contractually preserving registered projections.

Add regression ensuring stale runtime/intrinsic binding cannot survive an authoritative replace.

#### T6.6 — Canonical Universe identity

Replace repeated module/name matching for Tuple/Record/static kind anchors where feasible with `UniverseKey`/canonical declaration identity.

String/source-path recognition may remain at the one bootstrap boundary that maps authored source to a canonical Universe key. It must not be repeated in semantic application, native import, or codegen.

##### Supplemental acceptance amendment — INC-C3-19 (2026-09-28)

The architectural ruling for the `Bytes.fromList` classifier failure is
adopted here without changing the numbered task structure. Keep declaration
identity, runtime class-object/value projection, and type-family projection
distinct. `FlowPredicate` currently represents static refinement, not the
runtime proposition itself. A runtime classifier condition remains a valid
Boolean when no sound static refinement can be represented.

The producer may publish a refinement only when the canonical runtime
classifier has a supported interpretation and the runtime observation entails
the proposed proper static value type. A proper type proof is necessary but
not sufficient for applied generic classifiers unless runtime generic
arguments are actually checked/reified. For this plan's current surface, bare
higher-kinded families and unsupported/applied generic classifier forms yield
no static predicate and preserve prior `TypeKnowledge`; do not invent erased,
existential, or placeholder types. Keep a defensive proper-type check at
publication. Do not change `TypeStore::union`, loop/`whileTrue` flow, or the
lexical-body implementation.

The shipped `Object#is` is subclass-inclusive and `Object#is!` is exact-class
equality; the normative reflection-pattern spec defers runtime `is` semantics.
Consequently, exact `is!` false must not reuse subtype-negative
`IsNotInstance` refinement; positive `is!` may only yield weaker membership
when that implication is sound. This documents current implementation
semantics, not a new normative language rule.

Focused discriminators: ordinary proper nominal `is` positive/negative
refinement remains intact; bare `List` remains a valid Bool with no improper
refinement; an existing proper fact is preserved; applied generic targets
without runtime reification fail closed; exact-class `is!` does not exclude
subclasses on false. Then run the targeted `Bytes.fromList`, `corpus::option`,
and `corpus::errors` checks, distinguishing semantic/bootstrap success from a
later fixture error. Finish the other T6.6 acceptance gates separately:
inline match/`if` lowering without Closure construction, Closure isolation,
natural braced `Result.transpose` and `Option.filter`, and canonical Universe
bootstrap through Result and Option. This amendment does not by itself accept
T6.6 or change P4 verification status.

#### T6.7 — Runtime facade lowering consumes owner

`compile_intrinsic_runtime_data` currently reads `facade.owner` but re-derives `ClassKey(module,name)`.

Project the actual runtime class authority or a canonical `UniverseKey`/runtime owner token that the compiler can resolve directly. Use `facade.owner` as the controlling branch. Source module/name may be asserted for consistency, never used as the primary runtime-owner derivation.

#### T6.8 — Canonical structural behavior owners for inherent impls

Centralize semantic target-owner classification so it maps:

```text
nominal / applied nominal -> declaration
exact closed-sum case     -> enum declaration
Tuple / TupleFromPacks     -> canonical Universe Tuple declaration
Record(row)                -> canonical Universe Record declaration
```

The resolved target retains its structural semantic head. The canonical declaration controls authorization and member ownership. Require `impl.module == canonical_owner.module`; return the ordinary foreign-target/coherence diagnostic when a foreign package attempts a structural Tuple/Record impl.

Treat a structural specialization as a conditional domain even when its head has no generic parameters or constraints. Treat `Tuple<***Es: CompleteTypePack>` and `Record<**Fs: RecordFields>` as covering domains only when the canonical family head and binder contract prove the whole-family correspondence. Derive the correspondence from the family binder kind/mode and target-domain contract; preserve RecordFields as a row binding through `GenericInstantiation` and checked row-aware materialization. Do not convert a RecordRow parameter to `TypeId`.

Tuple and Record direct structural spellings and family applications must arrive at the same canonical `TypeId` before domain indexing. Extend the existing matcher for exact Tuple/Record structures, including `TupleFromPacks` where required by the canonical representation. Reuse recursive field/element matching; do not add generalized open-row remainder solving.

Required target matrix:

```phalcom
// In the Universe Tuple owner module
impl<***Es: CompleteTypePack> Tuple<***Es> {}
impl (Int, String) {}
impl<T> (T, T) {}

// In the Universe Record owner module
impl<**Fs: RecordFields> Record<**Fs> {}
impl { name: String } {}
impl<T> { value: T } {}
```

Equivalent family applications, explicit structural spreads, and direct structural forms share one target identity and therefore participate in ordinary overlap/collision rules. Foreign-module versions are rejected as unauthorized targets. No `StructuralTuple`/`StructuralRecord` target variants, source-provenance owner side channel, or parallel registry is allowed.

### Forbidden approaches

- no runtime-form fallback used to type-check static application;
- no static family-kind flattening to `Type`;
- no duplicate canonical/scoped reducers;
- no string switch in application lowering;
- no second runtime-owner registry;
- no "preserve old binding if default" implicit merge in authoritative replacement.

### Test changes required

Cover CV-18..CV-22, CV-44.

Required semantic tests:

- Tuple/Record family kind;
- canonical structural behavior-owner classification, owner-module authorization, and foreign-module rejection;
- Tuple/Record covering and specialized impl domains, including equivalent-spelling canonicalization and overlap;
- intrinsic application with runtime owner removed;
- runtime owner with semantic form unchanged;
- zero application;
- scoped symbolic application;
- ordinary type lambda/application unaffected;
- authoritative replace drops stale projections;
- refresh preserves intended projections.

Core/native tests:

- runtime owner chosen through projected owner;
- source identity mismatch fails closed/asserts in tests.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic declarations
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic type_annotations
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::fingerprints
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core native
```

### Acceptance

Tuple/Record have correct static family kind and runtime class projection simultaneously; no consumer needs the old `kind=Type` workaround.

### Local STOP / CONSULT triggers

STOP if existing ordinary generic class/data application semantics rely on `DeclarationTypeInfo::form` being a proper Type even for generic declarations. That would contradict the broader type-family model and needs architectural review.

### Checkpoint update

Record the final declaration projection contract and replace/refresh distinction.


---

## T7 — Complete source-visible structural domains, intrinsic associated types, and specialization-aware Tuple/Record APIs

### Purpose

Make the Universe source accurately describe the public structural type calculus and eliminate avoidable `Dynamic` erasure from Tuple/Record behavior after T6 gives those declarations their real family kinds.

This task is intentionally split from T6. T6 establishes declaration-family mechanics; T7 consumes those mechanics to make the authored source surface truthful. Do not implement the source surface by adding compiler-only pseudo-members.

### Preconditions

- T2 has established `CompleteTypePack` and public `RecordFields` spelling.
- T6 has established true higher-kinded Tuple/Record static family projections.
- source-owned intrinsic type-member identity from C1 remains available.

### Consumes

```text
IntrinsicTypeOwner
IntrinsicTypeOperation
TypeLevelProjection
TypeMemberDeclarationId
projection normalizer
TypeStore::union
TupleFromPacks
RecordRowMap / row projection machinery
Universe Tuple/Record source
native Tuple/Record primitive surface
```

### Produces

- canonical source-visible declarations for every public primitive structural domain;
- source-authored declarations for every public intrinsic type operation;
- `TupleEntries` removal/convergence onto `CompleteTypePack`;
- `Tuple::Element` and `Record::Value` aggregate-payload projections;
- parameterized Tuple/Record impl environments;
- specialization-aware iterator and direct-access typing;
- explicit documented dynamic-shape boundaries for operations that cannot currently publish a proper exact result without an existential type.

### Files and symbols

**Read/modify, expected:**

```text
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/record.ph
phalcom-core/core/universe/src/collections/iterator_laws.ph

phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/checker/type_member_declaration.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/source_index/*

phalcom-native-meta / generated native surface only where source/native signatures change
phalcom-core/src/primitive/tuple.rs
phalcom-core/src/primitive/record.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/type_members/*
phalcom-semantic/tests/semantic/integration/source_index*
phalcom-core/tests/core/collections/*
Universe/native-surface conformance tests
```

### Required implementation shape

#### T7.1 — Canonical structural-domain source surface

The source declarations must converge on:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields
```

These declarations are **anchors for primitive semantic domains**. They do not create fresh nominal `StaticDomain(TypePack)`-style identities. Their canonical declaration identities bind to the existing built-in kinds:

```text
TypePack          -> KindId::TYPE_PACK
LabeledTypePack   -> KindId::LABELED_TYPE_PACK
CompleteTypePack  -> KindId::COMPLETE_TYPE_PACK
RecordFields      -> KindId::RECORD_ROW
```

Keep the internal Rust row vocabulary (`RecordRowId`, `RecordRowData`, row solver) unless a local rename is mechanically useful. Public source/presentation must say `RecordFields`.

#### T7.2 — Source-visible intrinsic operations

At minimum author these operations in source:

```phalcom
impl TypePack {
  intrinsic type Map<F: Type -> Type>: TypePack
}

impl LabeledTypePack {
  intrinsic type Map<F: Type -> Type>: LabeledTypePack
}

impl CompleteTypePack {
  intrinsic type Positionals: TypePack
  intrinsic type Labeled: LabeledTypePack
  intrinsic type Map<F: Type -> Type>: CompleteTypePack
}

impl RecordFields {
  intrinsic type Map<F: Type -> Type>: RecordFields
}
```

If concat operations are already exposed or required by effective specs, author them against the existing concat normalizers. Do **not** expand P4 solely to expose a new general concat API if it is not already public.

Tuple:

```phalcom
impl<***Es: CompleteTypePack> Tuple<***Es> {
  intrinsic type Entries: CompleteTypePack
  intrinsic type Positionals: TypePack
  intrinsic type Labeled: LabeledTypePack
  intrinsic type Element: Type
  intrinsic type At<I: Int>: Type
  intrinsic type Map<F: Type -> Type>: Type
}
```

Record:

```phalcom
impl<**Fs: RecordFields> Record<**Fs> {
  intrinsic type Fields: RecordFields
  intrinsic type Value: Type
  intrinsic type Map<F: Type -> Type>: Type
}
```

The public selector exists because the source declaration exists. `lookup_intrinsic_type_operation` maps that declaration/selector to its evaluator; it must not be an alternate publication source.

#### T7.3 — CompleteTypePack operations

Implement/route:

```text
CompleteTypePack::Positionals
    -> existing positional projection

CompleteTypePack::Labeled
    -> existing labeled projection

CompleteTypePack::Map<F>
    -> lane-wise map using existing pack-map authority
```

Concrete results normalize immediately. Symbolic results stay canonical residual projections/pack nodes. Do not manufacture Tuple-specific `TupleEntries` terms.

#### T7.4 — Tuple::Element

For a concrete Tuple type, collect every positional and labeled payload `TypeId` and call `TypeStore::union`:

```text
(Int, String)::Element == Int | String
()::Element            == Never
```

For `TupleFromPacks` or a subject whose payload set cannot yet be enumerated soundly, retain a `TypeMemberProjection` with source declaration identity. Do not guess `Object`, `Dynamic`, or `Never`.

#### T7.5 — Record::Value

For a closed concrete row, union all field payload types:

```text
{name: String, age: Int}::Value == String | Int
{}::Value                       == Never
```

For an open/symbolic row whose complete set of payloads is not statically known, keep the projection symbolic. Do not omit the tail from the union and pretend the partial answer is exact.

#### T7.6 — Bidirectional source/evaluator certification

Add a focused certification helper/test that checks both directions:

```text
for every public IntrinsicTypeOperation:
    canonical source member exists
    selector/result kind/parameter contract matches

for every source `intrinsic type` member on an intrinsic owner:
    exactly one compiler evaluator is recognized
```

Compiler-private operations may remain private, but they must be explicitly marked/private rather than silently source-selectable.

#### T7.7 — Parameterize Tuple/Record behavior impls

Move source behavior under family-specialized environments:

```phalcom
impl<***Es: CompleteTypePack> Tuple<***Es> { ... }
impl<**Fs: RecordFields> Record<**Fs> { ... }
```

Do not keep `impl Tuple` / `impl Record` if those names now denote unsaturated higher-kinded families in type position.

Class-side native behavior can remain declaration-owned/class-side where specialization does not participate, but instance-side member checking must know `Es`/`Fs`.

#### T7.8 — Type direct element/value observers

Use aggregate projections where the runtime operation returns one member of the exact subject shape:

```text
Tuple raw element observation -> Tuple<***Es>::Element
Record raw field-value observation -> Record<**Fs>::Value
```

However, current primitives return `None` on out-of-range internal reads. Choose one coherent contract:

**Preferred:** make `rawAt`/`rawLabelAt`/`rawValueAt` internal checked-by-caller primitives whose invalid index is a runtime/internal error and give them exact non-optional result types. Refactor public `get`/index operations to bounds-check before calling them.

If changing primitive invalid-index behavior would violate an accepted public contract, use `Option<Element>` / `Option<Value>` internally and unwrap/check in source. Do not declare a non-optional type while returning `None`.

#### T7.9 — Type Tuple lane observers from the shape itself

The existing VM primitive behavior already returns Tuple products for lane extraction. Prefer exact signatures:

```phalcom
rawPositionals -> Tuple<*Positionals>
rawLabeled -> Tuple<**Labeled>
```

or the mechanically equivalent accepted syntax after T6/T2 application aggregation.

Do not retain source signatures such as `List<Dynamic>` or `Map<Symbol, Dynamic>` when the primitive actually returns Tuple products.

#### T7.10 — Iterator typing

Parameterize iterator state:

```phalcom
class TupleIterator<***Es: CompleteTypePack> {
  _source: Tuple<***Es>
  ...
  next -> Option<Tuple<***Es>::Element>
}
```

and publish:

```phalcom
impl<***Es: CompleteTypePack> CanonicalIterable for Tuple<***Es> {
  type Item = Tuple<***Es>::Element
  type Iterator = TupleIterator<***Es>
}
```

Apply the same principle to directly affected TupleZip helpers only where their output type is already expressible with existing type features. Do not invent variadic zip type arithmetic in this plan.

#### T7.11 — Genuine runtime-shape boundaries

Some current operations return a Tuple whose exact shape depends on runtime values, especially `rawSlice(start,end)`. Once bare `Tuple` is correctly higher-kinded, `-> Tuple` is ill-kinded.

P4 must **not** create an accidental existential feature. For each such API:

1. determine whether the result can be expressed using an existing proper abstraction without changing semantics;
2. otherwise keep the primitive/internal method behind the narrowest explicit dynamic boundary and document why the shape is runtime-dependent;
3. ensure ordinary known-shape access/iteration no longer routes through that boundary unnecessarily.

A remaining `Dynamic` at a genuinely runtime-dependent internal shape boundary does not violate INV-36; using `Dynamic` for known `Element`/`Value` does.

### Forbidden approaches

- no reintroduction of `TupleEntries` as a primitive kind;
- no source-less public intrinsic operations;
- no unconditional normalization of symbolic `Element`/`Value` to `Dynamic`/`Object`;
- no ill-kinded bare Tuple/Record source annotations;
- no existential Tuple/Record type added to solve slicing;
- no signature that disagrees with actual primitive absence/out-of-range behavior.

### Test changes required

Cover CV-06, CV-37..CV-40.

Required focused cases:

- all four structural source domains resolve and navigate to source;
- CompleteTypePack lane projections concrete + symbolic;
- `Tuple::Entries == Es`;
- empty/concrete/symbolic `Tuple::Element`;
- empty/concrete/open `Record::Value`;
- `Record::Fields` exact identity;
- iterator Item follows Tuple specialization;
- native/source signatures agree with primitive results;
- no remaining `TupleEntries` source-visible identity;
- bidirectional source/evaluator certification.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic type_members
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic source_index
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core tuple
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core record
```

Use exact available filters after listing tests; do not invent filter names if the live suite differs.

### Acceptance

The authored Universe source is a complete, navigable declaration surface for the public structural calculus, and all known-shape Tuple/Record behavior is statically typed without relying on the VM's generic `Value` representation.

### Local STOP / CONSULT triggers

STOP if a public operation requires an existential structural-family instance type to be sound and no accepted proper abstraction already exists. Do not invent one locally.

### Checkpoint update

Record the source/evaluator authority rule and the exact intentional dynamic-shape boundaries, if any remain.

---

## T8 — Migrate Record/Map/Set syntax, row-tail spelling, and Record remainder patterns

### Purpose

Perform the user-visible syntax cutover only after the semantic families/domains are ready to consume it.

Target:

```phalcom
{}                              // Record
{name: value, **other}          // Record + structural spread
{name: String, **Fs}            // Record type + open fields
let {name, **rest} = value      // Record pattern + remainder

Map { name: value }             // Map
Map<String, Int> { [key]: 1 }   // typed Map
Map { **otherMap }              // Map association expansion

#{}                             // Set
#{1, 2, 3}                      // Set
```

### Preconditions

- T2/T6 structural domains and application contracts are active.
- T7 source families are correctly higher-kinded.
- the parser still has no ordinary bare-brace block literal in expression position.

### Consumes

```text
Token::RecordLBrace (`#{`) legacy token
Expr::{RecordLiteral,MapLiteral,SetLiteral,RecordConstruction}
TypeAnnotationExpr::Record
Pattern::{Record,Map}
Record row checker/solver
record/map/set literal bytecodes and builders
```

### Produces

- bare-brace Record value/type/pattern grammar;
- explicit Map construction grammar including optional type arguments;
- active ordinary Set literal grammar on `#{...}`;
- `**Fs` open Record type tail syntax;
- `**rest` Record pattern remainder;
- migration diagnostics and corpus/source migration;
- old ambiguous bare Map literal behavior removed.

### Files and symbols

```text
phalcom-lexer / token definitions if `RecordLBrace` naming lives there
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast tests

phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/advisory/*
phalcom-semantic/src/source_index/*

phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/patterns.rs
phalcom-core/src/compiler/lib/loops.rs
phalcom-core/src/modules/semantic_lowering.rs
bytecode/runtime product/map/set handlers

formatters/printers/LSP semantic tokens as applicable
language corpus fixtures containing old braces
```

### Required implementation shape

#### T8.1 — Bare braces are Record, not content-classified collection syntax

In ordinary expression position:

```text
`{` -> parse Record literal
```

Do not scan contents first to guess Record versus Map. The syntactic head decides the construct.

In type-consuming position:

```text
`{` -> Record type
```

In pattern position:

```text
`{` -> Record pattern
```

Existing nominal record-data construction `Person { ... }` remains a separate postfix construction grammar and must not be confused with anonymous Record literals.

#### T8.2 — Repurpose `#{` as Set opener

Rename the lexer/token/AST comments as appropriate so `#{` no longer means Record.

Parse:

```phalcom
#{}
#{1}
#{1, 2, 3}
```

into the existing `Expr::SetLiteral` machinery.

The current compiler explicitly rejects `SetLiteralEntry::Expansion`. P4 does not silently enable it. If parser support for `*expr` is already generated mechanically, reject with the existing targeted diagnostic unless a separate ratified Set-spread rule is in force.

#### T8.3 — Explicit Map head

Extend Map literal AST with optional explicit generic arguments, conceptually:

```rust
struct MapLiteralExpr {
    type_arguments: Vec<TypeAnnotation>,
    entries: Vec<MapLiteralEntry>,
    range: SourceRange,
}
```

Parse only explicit forms:

```phalcom
Map { ... }
Map<K, V> { ... }
```

The parser may recognize the syntactic `Map` head to establish the Map-entry grammar, but semantic checking must validate that the head denotes the canonical Universe `Map` family, not a same-spelled local value or unrelated declaration. Do not implement Map identity as a scattered string test in checker/compiler.

If the source language already forbids shadowing canonical prelude type names, reuse that rule. If shadowing is permitted, a shadowed/noncanonical `Map` construction must produce a deterministic semantic diagnostic rather than silently constructing the builtin Map.

#### T8.4 — Preserve Map entry semantics

Reuse existing:

```text
BareSymbol
Computed
Expansion
```

Map entries and current duplicate/expansion runtime policy. Record row-lacks rules must not leak into Map duplicate handling.

`Map<K,V> {...}` supplies an expected specialization to literal inference/checking. `Map {...}` infers `K,V` from entries and context using ordinary Map generic inference. `Map {}` without enough context remains ordinarily underconstrained; do not default K/V to `Dynamic` merely because the syntax is explicit.

#### T8.5 — Record value spread

Bare Record literal keeps the existing `RecordLiteralEntry::Expansion` runtime builder path:

```phalcom
{name: "Altun", **rest}
```

Semantic typing requires the source to be statically projectable to
`RecordFields`; compose rows through the canonical row authority and enforce
duplicate/lacks obligations. A Record carries its exact or symbolic row. A
labeled Tuple may contribute its labeled lane through the checked
`LabeledTypePack`-to-`RecordFields` projection; mixed Tuple positional elements
do not participate. These domains remain distinct, and a symbolic Tuple lane
that cannot be faithfully projected fails closed.

A Map is never a legal Record spread source, even when written as a literal
with statically visible keys: its type does not encode its exact key set.
Reject Map-to-Record spread with a direct semantic diagnostic explaining that
Record spread requires a statically known field shape. Do not special-case
literal provenance, fabricate a row, use `Dynamic`, or change the shared
runtime labeled-lane machinery. Map-to-Map remains unchanged; Record/labeled-
Tuple-to-Map remains valid through the existing entry-inference path.

Architectural ruling adopted 2026-09-28: Record spread is shape-preserving,
whereas Map spread is entry-producing. Runtime support for a broader labeled
lane does not grant every source construct the same source-level admission.

#### T8.6 — Replace `| R` record-tail source syntax

The canonical open record type spelling becomes:

```phalcom
{name: String, **Fs}
```

where `Fs : RecordFields`.

Normalize this to the existing row-tail semantic representation. Do not introduce a parallel public row-concat AST/solver. Accept at most the already-supported one symbolic tail unless a separate row-composition feature is explicitly ratified.

Retire or migration-diagnose the old `#{... | R}` / `| R` form according to repository compatibility policy.

#### T8.7 — Record pattern remainder

Change the AST conceptually to:

```rust
Pattern::Record {
    entries: Vec<RecordPatternEntry>,
    remainder: Option<Box<Pattern>>,
    range: SourceRange,
}
```

Parse:

```phalcom
let { name, **rest } = person
let { name, **_ } = person
```

Only one remainder is legal and it must be terminal.

Semantic typing:

```text
subject row R
required labels L
remainder row = R - L
remainder binding type = Record<**(R - L)>
```

Use existing row subtraction/lacks/field lookup authority. If the subject is open, preserve the residual open tail; do not close it accidentally.

Executable lowering must construct/bind a Record containing exactly the residual fields and class `Record`. Do not lower remainder by converting through Map.

#### T8.8 — Map patterns are not part of this syntax cutover unless required

Current `Pattern::Map` uses bare braces. Once braces become Record patterns, that syntax collides. P4 must resolve the collision deliberately:

- preferred: give Map patterns an explicit `Map { ... }` head if Map destructuring is still an accepted feature;
- otherwise, if Map patterns are provisional/unratified, disable/migrate them rather than stealing Record syntax.

Do not retain content-based pattern classification.

#### T8.9 — Metadata literal syntax

Audit parser-only metadata literals that currently use `RecordLBrace`. Align them only if they are user-facing Phalcom syntax. Internal serialized metadata structures may keep their own grammar if not part of the language surface. Do not accidentally turn internal metadata records into Sets.

#### T8.10 — Migration diagnostics

Add targeted messages where the old intent is knowable:

```text
old `#{name: value}` Record syntax
    -> structural Records use `{name: value}`; `#{...}` is Set syntax

context expects Map but receives `{name: value}` Record
    -> use `Map {name: value}` for runtime-keyed associations

old open record tail `| R`
    -> use `**R`
```

Do not create lexer heuristics that reinterpret valid new Set syntax as old Record syntax.

### Forbidden approaches

- no brace-content heuristics to choose Map versus Record;
- no `Map` identity by unchecked leaf-name string in semantic lowering;
- no collapsing Map spread and Record spread into one row operation;
- no allowing runtime Map keys to become RecordFields;
- no enabling Set expansion as collateral work;
- no mixed Tuple/Record pattern behavior.

### Test changes required

Cover CV-30..CV-36.

Parser tests:

- `{}` / `{name: x}` / `{name: x, **r}` -> Record;
- `Map {}` / typed Map / computed key / `**map`;
- `#{}` / `#{1,2}` -> Set;
- `{name: T, **Fs}` type;
- `{name, **rest}` pattern;
- old syntax diagnostics;
- nominal `Person { ... }` unaffected;
- trailing closure grammar unaffected.

Semantic/runtime tests:

- exact Record row composition;
- duplicate field/lacks failure;
- Map inference and typed expected K/V;
- Map expansion stays runtime association merge;
- Record remainder exact/open row;
- Set basic execution;
- no Set-spread promise.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic collections
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections
```

Use narrower filters first while BUILD is active.

### Acceptance

The language has an unambiguous structural/collection literal grammar, Record row spelling is consistent with `**`, and Record remainder patterns preserve exact residual row typing.

### Local STOP / CONSULT triggers

STOP if activating bare Record braces revives a conflict with ordinary block/closure grammar, or if Map pattern compatibility requires a new general pattern protocol.

### Checkpoint update

Record the syntax cutover and any intentionally disabled legacy Map-pattern syntax.

---

## T9 — Separate Unit from EmptyRecord at runtime and make zero-product construction obey the new family laws

### Purpose

Make the ratified semantic distinction observable and runtime-correct:

```text
Tuple<>  == Unit        == ()
Record<> == EmptyRecord == {}
Unit != EmptyRecord
().class == Tuple
{}.class == Record
```

Today both empty Tuple and empty Record collapse to `Value::unit()`. This task removes that contradiction.

### Preconditions

- T6 has zero-application family semantics.
- T8 has canonical `{}` Record syntax.
- the 16-byte `Value` tag representation still has spare tag capacity (verified during planning; recheck live).

### Consumes

```text
ValueTag::Unit / Value::unit()
Value::{PartialEq,Hash,type_name,class}
finish_tuple / finish_record
BuildTuple / BuildRecord / FinishRecordLiteral
anonymous product runtime views
expect_tuple / expect_record
Tuple/Record equality/hash/toString/iteration behavior
runtime type/reflection overlays
```

### Produces

- dedicated immediate EmptyRecord runtime value;
- empty Tuple remains Unit immediate;
- Unit classifies as Tuple;
- EmptyRecord classifies as Record;
- empty Tuple/Record primitive observers work without heap allocation;
- equality/hash/same-bits distinguish the families;
- empty Record lowering no longer emits Unit.

### Files and symbols

```text
phalcom-core/src/value/repr.rs
phalcom-core/src/value/mod.rs
phalcom-core/src/product/mod.rs
phalcom-core/src/primitive/mod.rs or expect_* helpers
phalcom-core/src/primitive/tuple.rs
phalcom-core/src/primitive/record.rs
phalcom-core/src/compiler/lib/expr.rs
VM product builder/bytecode handlers
runtime typing/inspection
GC/value tracing only if tag exhaustiveness requires it
core product/value tests
```

### Required implementation shape

#### T9.1 — Add one canonical immediate EmptyRecord value

Preferred representation:

```rust
ValueTag::EmptyRecord
Value::empty_record()
Value::is_empty_record()
```

with zero payload, no GC root, and equality/hash semantics analogous to other singleton immediates but a distinct tag from Unit.

Do not allocate an EmptyRecord heap object merely to make it distinguishable unless the Value representation premise has changed and consultation approves the alternative.

#### T9.2 — Value classification

Update:

```text
Value::class(unit)         -> tuple_class
Value::class(emptyRecord)  -> record_class
```

`Value::type_name` may continue to use diagnostic names such as `unit` and `record`, but user-visible reflection must use the semantic type metadata, not this coarse string.

Every exhaustive `ValueTag` match must gain EmptyRecord behavior intentionally: equality, hash, debug formatting, option wrapping, same-bits, GC tracing, serialization/debug-only representation where applicable.

#### T9.3 — Empty Tuple construction

`finish_tuple([],[])`, `BuildTuple{0,0}`, Tuple slicing to length zero, and any equivalent product path continue to return `Value::unit()`.

Tuple primitives must accept the immediate Unit as a zero-length Tuple receiver. If `expect_tuple` currently accepts only heap tuple objects, teach the canonical Tuple view layer to expose an empty borrowed/view representation for Unit rather than allocating.

Required observers:

```text
rawSize -> 0
positionals -> ()
labeled -> () / canonical empty labeled Tuple view according to T7 source contract
iteration -> empty
Tuple equality/hash/toString -> ordinary zero-arity laws
```

#### T9.4 — Empty Record construction

Change all empty Record formation paths:

```text
finish_record([],[]) / equivalent
BuildRecord{fields:0}
BuildStaticRecord(empty spec)
FinishRecordLiteral(empty builder)
compiler fast path for RecordLiteral entries.is_empty()
```

so they return `Value::empty_record()`.

`expect_record` and `record_view` must treat EmptyRecord as a valid zero-field Record receiver without allocating.

#### T9.5 — Distinct identity and equality

At the Value substrate:

```text
Value::unit() != Value::empty_record()
same_bits(unit, emptyRecord) == false
hash(Value::unit()) and hash(Value::empty_record()) need not share a digest
```

Language-level `==` remains dispatched through Tuple/Record behavior and must also return false cross-family.

#### T9.6 — Exact runtime type observation

Ensure runtime metadata/type-descriptor paths distinguish:

```text
unit value exact type        -> Unit / Tuple<>
empty record exact type      -> EmptyRecord / Record<>
```

If the runtime does not currently carry an exact descriptor for immediates, use the established immediate-type mapping authority rather than attaching ad hoc descriptor pointers to `Value`.

### Forbidden approaches

- no `Value::unit()` for empty Record after this task;
- no `EmptyRecord` runtime class;
- no heap allocation required solely for empty Record identity unless architecture is amended;
- no special-casing `.class` in surface source code;
- no equality rule that says all zero-products are equal.

### Test changes required

Cover CV-23..CV-29.

Required tests:

- `Tuple<> == Unit == ()` semantic identity;
- `Record<> == EmptyRecord == {}` semantic identity;
- `Unit != EmptyRecord`;
- immediate Value equality/hash/same-bits distinction;
- `().class == Tuple`;
- `{}.class == Record`;
- empty Tuple primitive behavior;
- empty Record primitive behavior;
- empty Record literal/compiler all paths return Record family;
- no allocation regression if allocation counting is readily testable; otherwise inspect representation/unit-test tags.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core value
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core product
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core tuple
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core record
```

### Acceptance

Zero Tuple and zero Record are allocation-free canonical values belonging to different runtime families and exact semantic types.

### Local STOP / CONSULT triggers

STOP if adding one Value tag would violate serialized ABI/frozen representation guarantees not visible in the current repository, or if immediate exact-type reflection requires a new runtime metadata ownership model.

### Checkpoint update

Record the two immediate zero-product laws and class mappings.

---

## T10 — Replace Unit runtime-class ownership with intrinsic aliases and direct Tuple/Record runtime owners

### Purpose

Complete the declaration/class projection model by removing Unit as a runtime class while keeping it as a canonical semantic type alias, adding `EmptyRecord` as the analogous Record alias, and making native/runtime owner registries describe only actual runtime classes.

### Preconditions

- T9 has distinct zero-product runtime values.
- T6 removed Unit from intrinsic runtime-data application semantics.
- type alias AST/parser already preserves `ImplementationKind::Intrinsic`.

### Consumes

```text
phalcom-core/core/universe/src/option/unit.ph
UniverseKey::Unit
UniverseBindingKind
UNIVERSE_CLASS_RELATIONS / UNIVERSE_BINDINGS
CoreClasses::unit_class
VM bootstrap class registration
object-model invariant census
native source index/conformance verification
Tuple runtime class
```

### Produces

- source aliases:
  ```phalcom
  intrinsic type Unit = Tuple<>
  intrinsic type EmptyRecord = Record<>
  ```
- no `unit_class` runtime class row;
- Unit methods (`toString`, `hash`) live on Tuple behavior or generic Tuple logic;
- native metadata either retains Unit only as a non-class semantic binding or removes its runtime-class binding cleanly;
- runtime owner tables contain Tuple and Record, not Unit.

### Files and symbols

```text
phalcom-core/core/universe/src/option/unit.ph (likely replace/remove)
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/record.ph

phalcom-native-meta/src/universe.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/src/vm/bootstrap.rs
phalcom-core/src/universe/invariants.rs
phalcom-core/src/universe/primitives.rs
phalcom-core/src/native/source.rs
phalcom-core/src/native/verify.rs

semantic type-alias publication/resolution
core-surface conformance
object-model census docs/tests
```

### Required implementation shape

#### T10.1 — Canonical intrinsic aliases

Use the already-supported intrinsic alias declaration syntax:

```phalcom
intrinsic type Unit = Tuple<>
intrinsic type EmptyRecord = Record<>
```

These aliases are transparent canonical semantic aliases. They do not create nominal types and do not own runtime classes.

If `TypeAliasInfo` currently ignores implementation kind semantically, retain/provide enough provenance for Universe conformance/tooling to know these are compiler/Universe-authored canonical aliases. Do not invent an `IntrinsicTypeApplication::Unit` again.

#### T10.2 — Move Unit behavior to Tuple

Current Unit source defines:

```phalcom
toString -> String { "()" }
hash -> Int { 0 }
```

Tuple already owns/generalizes these behaviors. Ensure zero-arity Tuple behavior produces the required Unit results. Delete redundant Unit behavior impl rather than forwarding through a phantom class.

#### T10.3 — Remove `unit_class`

Remove Unit from:

```text
CoreClasses runtime class fields
core class creation
bootstrap add_class
UNIVERSE_CLASS_RELATIONS runtime rows
primitive installation class arrays
object-model core-class census
class graph invariant tests
```

Update counts/census documentation based on the actual resulting floor. Do not leave a dead hidden Unit class.

#### T10.4 — UniverseKey policy

`UniverseKey::Unit` may remain only if the native/type metadata ecosystem needs a stable canonical **semantic** key for the alias. If retained, introduce or use a binding category that does not claim a runtime class. For example, an internal category conceptually equivalent to `TypeAlias`/`SemanticBinding` is acceptable.

It must not participate in `UNIVERSE_CLASS_RELATIONS` or resolve to `ClassId`.

If the canonical Universe registry already has a declaration-only key mechanism, reuse it. Do not overload `RuntimeSupportClass`—Unit is not a hidden runtime class after P4.

Add `UniverseKey::EmptyRecord` only if canonical alias identity infrastructure requires one; do not add it to the runtime class relation table.

#### T10.5 — Native/runtime owner direct consumption

Finalize T6's owner migration. Native Tuple/Record primitive registration and intrinsic data runtime facade lowering bind to:

```text
IntrinsicRuntimeOwner::Tuple -> tuple_class
IntrinsicRuntimeOwner::Record -> record_class
```

There is no Unit runtime owner.

#### T10.6 — Object model and reflection

Update class census expectations:

```text
Unit is no longer a class
Tuple is the class of ()
Record is the class of {}
```

If public reflection enumerates declarations and classes separately, Unit/EmptyRecord still appear as type aliases/declarations in declaration reflection while absent from class enumeration.

### Forbidden approaches

- no hidden Unit runtime class retained “for compatibility”;
- no `EmptyRecord` class;
- no alias implemented as nominal zero-field `data`;
- no `RuntimeSupportClass` misuse for a semantic alias;
- no native primitive copied from Unit to a new empty-tuple class.

### Test changes required

Cover CV-23..CV-29 and CV-44.

Add/adjust:

- parser/semantic alias tests for `intrinsic type`;
- Universe binding/conformance tests distinguish class vs alias;
- core class census/count;
- `.class` laws;
- class lookup no longer finds a Unit class;
- declaration lookup still finds Unit alias;
- Tuple zero behavior remains `()`/hash contract;
- native surface verifier accepts alias-owned Unit and class-owned Tuple/Record.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core object_model
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core native
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic aliases
```

Use actual suite filters.

### Acceptance

Unit and EmptyRecord are semantic aliases only; runtime class identity is exclusively Tuple/Record as ratified.

### Local STOP / CONSULT triggers

STOP if native schema consumers fundamentally require every `UniverseKey` to denote a runtime class and adding a semantic-binding category would ripple beyond the anticipated Universe/native metadata subsystem. Produce a compact schema/consumer census before consultation.

### Checkpoint update

Record Unit runtime-class retirement and final Universe-key treatment.

---

## T11 — Migrate the Universe/core source and repository corpus to the completed structural model

### Purpose

Move all first-party Phalcom source and directly owned fixtures to the final surface so no legacy source form remains as accidental authority.

### Preconditions

T2–T10 are semantically implemented enough that the final source syntax/type surface can compile.

### Consumes

- final parser grammar;
- final structural declarations;
- final alias model;
- final typed intrinsic members;
- compiler/runtime lowering.

### Produces

- canonical Tuple/Record Universe declarations;
- canonical Unit/EmptyRecord aliases;
- RecordFields spelling across source docs/tests;
- explicit Map literals;
- `#{}` Sets;
- migrated open Record types/patterns;
- no stale source examples that force compatibility hacks.

### Files and symbols

At minimum census/migrate:

```text
phalcom-core/core/universe/src/**/*.ph
phalcom-core/core/**/*.ph
examples/**/*.ph
benchmarks/**/*.ph where compiled by tests
phalcom-ast test fixtures
phalcom-semantic fixtures/corpora
phalcom-core tests/corpora
phalcom-lsp fixtures
```

Use repository search before and after migration for:

```text
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
intrinsic data Unit()
TupleEntries
RecordRow   # source/user-facing occurrences; internal Rust/docs theory may remain
#{          # classify Set vs old Record
bare map literals `{...}` in expression context
`| R` / record-tail syntax
impl Tuple
impl Record
: Tuple / -> Tuple where used as a proper unsaturated type
: Record / -> Record where used as a proper unsaturated type
Dynamic in Tuple/Record source surface
```

### Required implementation shape

#### T11.1 — Final Universe declarations

Converge on the accepted source, mechanically adapted to exact module layout:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields

intrinsic data Tuple<***Es>(***Es)

intrinsic data Record<**Fs> {
  **Fs
}

intrinsic type Unit = Tuple<>
intrinsic type EmptyRecord = Record<>
```

Do not keep legacy facade declarations alongside the new ones.

#### T11.2 — Specialization-aware helper classes

Update TupleIterator and directly coupled helper/view classes according to T7. Where a helper fundamentally stores “any runtime Tuple” and no proper type exists, either parameterize it from the originating `Es` or keep that helper private behind a documented dynamic boundary. Do not restore bare Tuple as a proper type just to make fields compile.

#### T11.3 — Record source usage

Migrate source Records:

```text
#{a: v}     -> {a: v}
#{a: T | R} -> {a: T, **R}
```

Preserve semantic intent. Do not blanket-replace `#{...}` because new Set syntax uses it.

#### T11.4 — Map source usage

Every dynamic Map literal becomes explicit:

```phalcom
Map { ... }
```

or typed:

```phalcom
Map<K,V> { ... }
```

Do not convert anonymous Records to Map merely because old code used braces; classify each call site by expected semantic type and operations.

#### T11.5 — Set activation

Migrate valid Set literals to `#{...}` only where there is actual Set intent. Keep `Set.new()` where no literal improves the code or where spread is required but still deferred.

#### T11.6 — Diagnostic fixtures

Update negative fixtures whose expected parse/semantic failure changes under the new syntax. Preserve what the test is intended to prove; do not mechanically change expected diagnostics if the test should now exercise a different legal construct.

### Forbidden approaches

- no global textual replace of `#{`;
- no compatibility parser accepting both complete old and new collection grammars indefinitely;
- no leaving Universe examples on obsolete syntax;
- no weakening fixture assertions to accommodate migration noise.

### Test changes required

Run source-corpus focused groups only after migration is coherent. Add a grep/census assertion script/test if the repository convention supports source-surface census tests.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test language_tests <affected-groups>
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration
```

Do not run the full workspace yet.

### Acceptance

First-party source consistently demonstrates the new model and no legacy syntax is required for core bootstrap.

### Local STOP / CONSULT triggers

STOP if a large corpus of old Map/Record syntax cannot be mechanically classified without changing accepted semantics. Produce the ambiguous call-site list rather than guessing.

### Checkpoint update

Record source migration completion and any consciously retained legacy spelling in non-normative historical docs only.

---

## T12 — Persist the new semantic surface through metadata, incremental fingerprints, source indexing, LSP, and specifications

### Purpose

Ensure P4 survives module boundaries, cold/incremental rebuilds, editor tooling, and durable documentation rather than existing only in one compiler session.

### Preconditions

- semantic/runtime/source migration is coherent through T11.
- current metadata schema remains v8 unless a structural wire change is proven necessary.

### Consumes

```text
phalcom-type-meta KindNode / generic parameter records / type nodes
semantic metadata exporter/importer/validator
source declaration/type-member identity
incremental fingerprints/dependency graph
source index/editor presentation/LSP
normative specs + checkpoint docs
```

### Produces

- durable mode+kind persistence for `**Fs: RecordFields`;
- CompleteTypePack source representation without TupleEntries surrogate;
- correct alias/declaration/member identities across metadata;
- cold/incremental final-state parity;
- source navigation for RecordFields and intrinsic members;
- updated normative docs consistent with P4;
- schema version changed only if the serialized structure actually changes.

### Files and symbols

```text
phalcom-type-meta/src/{header,kind,generic,type_node,scoped_type,validate,...}.rs
phalcom-type-meta/tests/schema_compat.rs
phalcom-semantic/src/metadata/*
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/incremental/*
phalcom-semantic/src/source_index/*
phalcom-semantic/src/editor.rs / presentation.rs
phalcom-lsp/src/* relevant navigation/completion/semantic tokens

docs/specs/type-level-domains.md
docs/specs/structural-products.md
docs/specs/type-members.md
docs/specs/universe.md
docs/spec/collections/02-tuples.md
docs/spec/collections/03-records.md
docs/spec/collections/06-rest-spread-and-pack-operators.md
docs/specs/vm/products.md
docs/spec/current/stdlib/map-and-set.md or its effective successor
accepted decision amendments where repository policy requires
LANG007.C3-CHECKPOINT.md
```

### Required implementation shape

#### T12.1 — RecordFields is source spelling, RecordRow remains durable semantic tag unless structure changes

Current schema v8 already has `KindNode::RecordRow` and row-tail nodes. Keep those wire identities for compatibility while presentation/source uses `RecordFields`.

Do **not** bump schema solely to rename an enum variant. A schema bump is required only if P4 adds durable fields/nodes that older v8 decoders cannot safely interpret.

#### T12.2 — Mode and kind remain independent in metadata

Persist, validate, import, and fingerprint both:

```text
parameter expansion mode
parameter semantic kind
```

Examples that must round-trip distinctly:

```text
mode=Labeled kind=LabeledTypePack
mode=Labeled kind=RecordRow(= public RecordFields)
mode=Single  kind=RecordRow
```

Remove validators that infer one from the other.

#### T12.3 — Product expansion declaration metadata

If public/cross-module nominal `data` declarations require their product-shape templates for downstream constructor/type checking, serialize the stable declaration-shape contribution necessary to reconstruct them. Prefer extending the existing declaration product metadata/catalog rather than storing raw AST.

Required durable facts may include:

```text
fixed source component references
expansion site mode/domain/parameter identity
source order / shape category
```

Do not serialize query-local solver IDs or specialization layouts.

If existing public declaration metadata already derives all necessary shape behavior from generic signature + authored data components, prove that with an import test before deciding no new node is required.

#### T12.4 — Type-member source identity

Ensure exported symbolic intrinsic projections retain `source_declaration` for all public source-authored members. Tighten comments/invariants that currently allow public intrinsic projections to be source-less.

Backward-compatible imported old projections may lack source identity; handle that as compatibility state without emitting new source-less public projections.

#### T12.5 — Alias metadata

`Unit` and `EmptyRecord` should import/export as transparent aliases to canonical Tuple/Record zero specializations. They must not serialize as runtime classes or nominal data roots.

#### T12.6 — Incremental fingerprints

Fingerprint at least:

- binder kind;
- binder expansion mode;
- product expansion contribution and its parameter identity;
- Tuple/Record intrinsic application binding;
- source intrinsic type-member declaration/signature;
- intrinsic alias body/provenance where relevant;
- syntax-driven semantic shape changes after reparse.

Reuse the existing declaration/dependency graph. No P4-specific invalidation registry.

Add cold/incremental differential tests for edits:

```text
LabeledTypePack -> RecordFields
* / ** / *** mode change
fixed component added/removed around expansion
Tuple/Record intrinsic member result kind change
Record literal field/spread change
Unit/EmptyRecord alias consumer
```

#### T12.7 — LSP/editor

Update completion/hover/go-to-definition/rename/presentation so:

- `RecordFields` is the public name;
- `RecordRow` is not suggested as source syntax;
- `CompleteTypePack` is source-visible;
- `Tuple::Entries` definition goes to authored source;
- `Record::Fields` and `Record::Value` go to authored source;
- product expansion parameter/source occurrence is indexed;
- Record pattern remainder binds/navigates like other pattern names;
- Map/Record/Set syntax is classified correctly for semantic tokens/completion.

#### T12.8 — Normative spec convergence

Update current effective specs. At minimum remove/replace stale claims:

```text
Record spread is illegal                 -> P4 permits structural ** Record spread
Record uses only old #{} syntax          -> Record uses {}
Map uses anonymous bare braces           -> Map uses explicit Map {}
Set literal reserved-inactive            -> ordinary #{} Set literal active
TupleEntries is public complete domain    -> CompleteTypePack
RecordRow is public spelling              -> RecordFields
Unit owns runtime class/data declaration  -> Unit = Tuple<>
empty Record is Unit                      -> EmptyRecord = Record<> distinct
bare Tuple/Record are proper Type         -> family projection has higher kind
```

Record the non-goals explicitly so future work does not infer mixed destructuring/product protocols from the common implementation substrate.

### Forbidden approaches

- no schema bump for cosmetic spelling only;
- no serialization of `RecordRowId`, `TypeId`, `ClassId`, or expansion-site local indices as durable identity;
- no duplicate incremental graph;
- no synthetic LSP-only intrinsic member table;
- no historical docs rewritten as if they were current normative sources unless repository policy asks for supersession links.

### Test changes required

Cover CV-41..CV-43, CV-46.

Required:

- v8/current metadata roundtrip for CompleteTypePack + RecordFields binder modes;
- older supported schema compatibility remains deterministic;
- cross-module user-defined expanded MyTuple/MyRecord declaration consumption;
- alias roundtrip;
- public intrinsic projection source identity;
- cold/incremental parity matrix;
- LSP definition/hover/completion fixtures;
- source presentation uses RecordFields.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp <focused-filters>
```

### Acceptance

A clean import/rebuild/editor session observes exactly the same structural-family model as the originating semantic session.

### Local STOP / CONSULT triggers

STOP if downstream public expanded-data checking requires a new metadata construct whose compatibility/versioning strategy is not obvious. Produce the exact missing facts and current v8 decoder contract before consultation.

### Checkpoint update

Record metadata version decision, incremental gates, and tooling/spec convergence.

---

## T13 — Stabilize, certify C3.P4, re-close LANG007.C3, and produce execution records

### Purpose

Run the bounded stabilization matrix, classify failures, perform the one required broad post-P4 certification pass, and leave durable repository state that truthfully describes what shipped.

### Adopted T13 amendment — guarded/derived conformance evidence

Guarded and derived conformances are valid evidence sources for ordinary trait
dispatch. Discover members by selector, then resolve applicability only through
the existing canonical conformance proof authority. Successful Exact,
Forwarded, or Derived proofs may produce the existing
`TraitDispatchSite::Evidenced` product and must preserve the selected rule,
substitution, nested proof provenance, and semantic dependencies. Do not add a
dispatch-local proof solver/index authority, change language meaning or the
compiler/VM ABI, install trait defaults on concrete classes, or alter Priority
3 classifications. Preserve existing ambiguity/coherence and inherent-override
behavior. Focused acceptance includes the guarded
`Iterator => Iterable` rule for concrete and symbolic receivers, guard
rejection, unrelated same-selector traits, nested derivation,
ambiguous/coherent overlap, and an actual source expression calling
`receiver.toList` on a canonically known Iterator receiver. The natural
`(0..).iter.take(10).toList` chain is a separate blocked integration
discriminator under INC-C3-20, not a passing T13 gate. Keep P4/C3 status
`IN_PROGRESS / PARTIAL / UNVERIFIED` until the named evidence is complete.

### Adopted INC-C3-20 disposition — Outcome B and known-receiver gate — 2026-09-29

The bounded callable-result census established Outcome B for unannotated
trait-default bodies: their return results are not currently published as a
canonical return-signature contract consumable by trait dispatch. Ordinary
class callable inferred-return publication exists and is outside this ruling.
`Iterator.take` is additionally blocked by `RecursiveFixpoint`. T13 must not
introduce trait-default return-signature inference/publication. That requires a
separate semantic design/remediation item covering publication, generic/Self
substitution, recursion, dependencies, metadata, and cold/incremental behavior.

The T13 actual-expression gate is amended to analyze `receiver.toList` where
the receiver's concrete Iterator type is already known from a canonical source
signature, and to include an ineligible known-receiver negative control. It
must assert canonical `Iterable::toList`, `TraitDispatchSite::Evidenced`, the
guarded Derived rule/provenance, and recorded proof dependencies. The natural
`(0..).iter.take(10).toList` discriminator remains in the suite but is ignored
as `BLOCKED by INC-C3-20`; it is not acceptance evidence. Priority 3 remains
unchanged, and P4/C3 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`.

### Preconditions

T1–T12 complete with their focused acceptance conditions.

### Consumes

- all P4 implementation and focused tests;
- checkpoint failure ledger;
- prior P3/P3.R1 baseline classifications;
- repository workflow conventions.

### Produces

- focused acceptance matrix mapped to CV-01..CV-47;
- one broad workspace certification attempt;
- classified residual failures;
- updated `LANG007.C3-CHECKPOINT.md`;
- P4 walkthrough;
- P4 handoff;
- truthful plan/checkpoint verification state.

### Files and symbols

```text
all directly affected tests
LANG007.C3-CHECKPOINT.md
LANG007.C3.P4-walkthrough.md
LANG007.C3.P4-handoff.md
this plan's lifecycle metadata if repository practice updates completed plans
```

### Required implementation shape

#### T13.1 — Search-based semantic census

Before broad tests, run targeted repository searches proving no live source/implementation leftovers:

```text
old TupleEntries public source identity
old RecordRow public source examples in effective specs/Universe
intrinsic data Unit()
unit_class runtime ownership
IntrinsicRuntimeOwner::Unit
IntrinsicTypeApplication::Unit
empty Record -> Value::unit fast path
legacy #{...} Record parser comments/tests
bare anonymous Map grammar
ill-kinded `-> Tuple` / `: Tuple` in migrated source surface
ill-kinded `-> Record` / `: Record`
known-shape Tuple/Record `Dynamic` signatures
source-less public intrinsic operation entries
```

Classify internal mathematical `RecordRow*` Rust symbols as expected, not migration failures.

#### T13.2 — Focused stabilization gates

Run G1–G5 below. Do not run the workspace gate until focused failures plausibly caused by P4 are green or explicitly blocked/classified.

#### T13.3 — Broad certification once

Run the CERTIFY commands from §16 once from a clean coherent tree. If a command fails:

1. classify A/B/C/D;
2. fix A/B;
3. give C one bounded classification pass;
4. record D and continue to the next gate if the failure does not prevent it;
5. rerun a broad command only after a relevant correction, not ritualistically.

#### T13.4 — Verification truth

Use:

```text
RELEASE_COMPLETE
```

only if every named broad gate is green.

If implementation/focused tests are green but inherited broad failures remain:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED or BASELINE_BLOCKED
```

according to the repository convention and failure effect.

#### T13.5 — Durable records

Walkthrough must distinguish:

```text
planned architecture
implemented architecture
deviations/mechanical drift
consultations
focused tests actually run
broad tests actually run
residual failures
metadata schema decision
runtime representation decision
lifecycle normalization result
```

Handoff must tell the next checkpoint/plan not to reopen:

- RecordFields versus RecordRow internal naming;
- Unit/EmptyRecord zero-product laws;
- one runtime Tuple/Record class;
- mode/domain independence;
- source intrinsic-member authority;
- product expansion template ownership.

### Forbidden approaches

- no declaration of C3 complete without updating checkpoint truth;
- no labeling broad predecessor failures as P4 failures without causal evidence;
- no skipping broad certification silently;
- no fixing unrelated architectural debt during final stabilization;
- no new feature additions after G3 except fixes required by acceptance.

### Tests to run now

Use the verification gates below.

### Acceptance

Every coverage obligation is either PASS or explicitly classified/nonblocking, checkpoint records are truthful, and the repository is ready for the next LANG007 objective without requiring conversation history.

### Local STOP / CONSULT triggers

STOP if focused acceptance can only pass by weakening a ratified invariant, or if broad failures reveal that the new semantic model breaks an unanticipated canonical subsystem rather than a mechanical migration site.

### Checkpoint update

Final authoritative checkpoint update.

---

# 19. Verification gates

## G1 — Structural kind, binder, and declaration-shape gate

### Purpose

Prove T2–T4 before runtime work is trusted.

Required coverage:

```text
CV-02..CV-15
```

Run the narrowest available equivalents of:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast data_syntax
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast parser
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic generics
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic data
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic record_row_polymorphism
```

Expected:

- domain/mode matrices are correct;
- user MyTuple/MyRecord shape templates publish;
- nominal identity remains nominal;
- no fake component identity is required.

Do not broaden after PASS.

---

## G2 — Specialization layout and declaration-projection gate

### Purpose

Prove T5/T6 architecture before source migration.

Required coverage:

```text
CV-16..CV-22
CV-44
```

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic declarations
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic type_annotations
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core data
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core native
```

Expected:

- one behavior class/multiple exact layouts;
- Tuple/Record family kinds are higher-kinded;
- runtime owner is independently projected;
- zero variadic family application works;
- authoritative replacement cannot retain stale intrinsic projections.

---

## G3 — Source surface, syntax, and zero-product gate

### Purpose

Prove T7–T10 as one coherent language/runtime surface.

Required coverage:

```text
CV-23..CV-40
```

Run focused AST/semantic/core groups for:

```text
Tuple/Record type members
Record/Map/Set parser
Record row type + spread
Record remainder pattern
Unit/EmptyRecord aliasing
Value zero-product representation
Tuple/Record runtime behavior
object model class identity
```

At this gate explicitly assert source examples:

```phalcom
let u = ()
let e = {}

assert(u.class == Tuple)
assert(e.class == Record)
assert(u != e)

let r = { name: "Altun" }
let m = Map { name: "Altun" }
let s = #{1, 2}
```

Use the repository's actual assertion/testing idiom rather than necessarily this pseudo-fixture.

No old runtime Unit class may remain after PASS.

---

## G4 — Persistence and tooling gate

### Purpose

Prove T12 cross-session correctness.

Required coverage:

```text
CV-37
CV-41..CV-43
CV-46
```

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::metadata
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-lsp <focused affected tests>
```

Expected:

- current metadata roundtrips;
- supported old schemas behave according to compatibility policy;
- cold/incremental parity holds;
- source navigation lands on authored members/domains;
- public spelling is RecordFields.

---

## G5 — Migrated source/corpus gate

### Purpose

Prove first-party source does not depend on legacy syntax or old class/kind workarounds.

Required coverage:

```text
CV-30..CV-40
CV-45
```

Run affected language corpus groups and source bootstrap. Include a search-based census from T13.1.

Expected:

- Universe boots from final declarations;
- Record/Map/Set syntax is unambiguous;
- no mixed destructuring/product-protocol scope leak;
- no legacy Unit runtime class dependency.

---

## G6 — Broad C3.P4 re-closure gate

### Purpose

Perform the one broad certification attempt omitted after the earlier bounded corrective closure and required now because P4 crosses AST, semantics, metadata, compiler, VM, Universe, LSP, and corpus.

Run from a clean coherent tree:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace
RUSTFLAGS='' RUSTC_WRAPPER='' cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Classify every failure before editing. Record exact commands/results in the walkthrough/checkpoint.

---

# 20. Final focused acceptance

Before considering P4 implemented, produce a coverage table in the walkthrough with every CV ID.

Minimum final mapping:

| Coverage family | Required evidence |
|---|---|
| CV-01 | lifecycle/checkpoint record has one P4 identity |
| CV-02..06 | structural kind/mode/domain tests |
| CV-07..15 | parser + user-defined shape-generic nominal data tests |
| CV-16..22 | runtime specialization + declaration projection tests |
| CV-23..29 | Unit/EmptyRecord semantic/runtime laws |
| CV-30..36 | Record/Map/Set syntax and diagnostics |
| CV-37..40 | source intrinsic members and typed Tuple/Record API |
| CV-41..43 | metadata/incremental/LSP |
| CV-44 | runtime owner direct-consumption test |
| CV-45 | explicit scope/non-feature assertion/census |
| CV-46 | compatibility test |
| CV-47 | broad closure command ledger |

A missing required test is not replaced by “code inspection” unless the plan explicitly marks that obligation as representation-only and the walkthrough explains why automated observation is unavailable.

---

# 21. Performance and resource evidence

P4 is primarily a semantic correctness migration, but it has two resource invariants worth defending without adding a benchmark program:

1. **Zero products remain allocation-free.** `Unit` and `EmptyRecord` should both be immediate values under the preferred representation. A unit test of Value tags/construction is sufficient evidence; do not add a microbenchmark.
2. **User-expanded nominal data does not create one behavior class per specialization.** Assert class identity/registry counts in tests. Exact specialization descriptors/layouts may be interned/cached as already supported.

No broader performance benchmark is required unless implementation evidence shows a material regression.

---

# 22. Checkpoint bookkeeping

## At plan start

T1 must:

- normalize the old corrective P4 identifier or consult if immutable;
- reopen `LANG007.C3-CHECKPOINT.md`;
- add this P4 as `IN_PROGRESS` / `PARTIAL` / `UNVERIFIED`;
- retain prior P1–P3/P3.R1 evidence rather than rewriting history.

## During implementation

Update checkpoint only at durable events:

```text
G1 structural domains/product template established
G2 specialization/runtime projection established
G3 final source/zero-product language laws established
G4 persistence/tooling established
consultation/amendment
new deferred baseline failure
```

Do not update checkpoint after every mechanical edit.

## At completion

Record:

- final plan status/completion/verification;
- exact metadata schema decision;
- final Value representation law;
- Unit runtime-class retirement;
- final syntax matrix;
- final family/kind laws;
- broad gate results;
- residual failures and classifications;
- next action.

---

# 23. Walkthrough deliverable

Create:

```text
LANG007.C3.P4-walkthrough.md
```

It must contain:

1. final result/status metadata;
2. revision/commit range;
3. lifecycle normalization result;
4. architecture actually implemented;
5. structural-domain/mode model;
6. product-shape expansion representation;
7. user nominal specialization runtime layout model;
8. Tuple/Record declaration projection model;
9. source intrinsic-member surface and evaluator mapping;
10. Record/Map/Set syntax migration;
11. Unit/EmptyRecord aliases and runtime representation;
12. Unit class removal / Universe changes;
13. metadata schema/import compatibility decision;
14. incremental/LSP changes;
15. tests added;
16. exact tests/gates actually run;
17. broad certification results;
18. deviations and consultations;
19. deferred/baseline failures;
20. residual risks.

Do not claim a test was run merely because it exists.

---

# 24. Handoff deliverable

Create:

```text
LANG007.C3.P4-handoff.md
```

Include:

```text
current revision
checkpoint status
stable interfaces
ratified invariants
source syntax matrix
where RecordFields maps to internal RecordRow machinery
where Tuple/Record runtime owners are projected
product shape-template/materialization owner
exact-specialization runtime descriptor owner
Unit/EmptyRecord Value representation
metadata schema/version compatibility
remaining baseline failures
next objective
first recommended commands
```

Explicitly say not to redesign/reopen:

```text
mode vs kind independence
RecordFields != LabeledTypePack
RecordRowId outside TypeId
Tuple<> == Unit
Record<> == EmptyRecord
Unit != EmptyRecord
one runtime Tuple class
one runtime Record class
no mixed destructuring
no general product protocol
source owns public intrinsic member identity
```

---

# 25. Completion truth table

Use repository lifecycle metadata truthfully:

| State | Meaning |
|---|---|
| source written | code/docs changed; no test implication |
| tests added | coverage exists; no PASS implication |
| focused tests passed | P4 feature behavior proved at named gates |
| affected suites passed | directly adjacent regressions green |
| implementation complete | every T1–T13 acceptance condition satisfied |
| checkpoint accepted | checkpoint record reflects the final implementation and residual failures |
| release certified | all G6 broad gates green |

Normal successful outcome may legitimately be:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

if broad inherited failures remain. Use `RELEASE_COMPLETE` only when G6 is fully green.

---

# 26. Patch sequencing and commit guidance

The implementer should preserve coherent rollback/review boundaries. Suggested commit slices are architectural, not mandatory one-task-per-commit rules:

```text
1. lifecycle/checkpoint normalization
2. structural kinds + mode/domain decoupling
3. product-shape AST/semantic templates
4. user nominal specialization layouts
5. declaration type-family/runtime projection closure
6. source intrinsic member surface + typed APIs
7. Record/Map/Set syntax migration
8. Unit/EmptyRecord runtime values + Unit class retirement
9. Universe/corpus migration
10. metadata/incremental/LSP/spec convergence
11. stabilization fixes + final records
```

Never intentionally commit a non-compiling tree unless the repository's normal workflow explicitly permits staged broken commits. A task may span multiple commits if each is coherent and does not encode temporary architecture that later commits undo.

---

# 27. Implementation anti-patterns

Reject patches exhibiting any of these patterns:

```text
if name == "Tuple" / "Record" / "Map" scattered across semantic consumers

RecordFields converted to a TypeId or LabeledTypePack because generic code expects TypeId

fake DataComponentId allocated for every expansion-produced specialization field

one VM class for MyTuple<Int>, another for MyTuple<String>

Unit runtime class retained while ().class is separately special-cased to Tuple

empty Record still represented as Value::unit()

public intrinsic type operation exists only in Rust lookup table

source intrinsic member has no compiler evaluator and silently remains opaque

bare Tuple/Record forced back to kind Type so old source annotations compile

rawAt/value observers remain Dynamic even though Element/Value is known

runtime-dependent slice result falsely typed as Tuple<***Es>

new existential AnyTuple/AnyRecord invented incidentally

brace-content scanning decides Record vs Map

Map spread implemented through RecordRow solver

Set spread silently enabled because SetLiteralEntry::Expansion exists

TupleEntries kept as a second CompleteTypePack identity

RecordRow source spelling left in public docs while code says RecordFields

metadata schema bumped only to rename RecordRow -> RecordFields

new P4-specific metadata/incremental/runtime registry introduced

mixed Tuple/Record destructuring added because their internal product machinery looks similar
```

---

# 28. Final architect self-review checklist

Before handing this plan to Luna, the planner has verified:

- [x] desired semantics are separated from current implementation accidents;
- [x] `RecordFields` remains backed by existing row machinery rather than becoming a type/pack;
- [x] expansion mode and kind have separate ownership;
- [x] user-expanded `data` has a nominal specialization model and a runtime layout strategy;
- [x] specialization-generated fields do not need fake source IDs;
- [x] Tuple/Record static family projection and runtime class projection are independent;
- [x] runtime owner lowering has one authority;
- [x] public intrinsic member identity is source-owned;
- [x] Unit/EmptyRecord have distinct semantic and runtime laws;
- [x] zero-product runtime representation is implementable in the current 16-byte Value without widening;
- [x] Unit runtime class removal is accounted for in bootstrap/census/native metadata;
- [x] explicit Map and Set syntax reuse existing runtime machinery rather than building parallel collections;
- [x] Record pattern remainder has an AST/semantic/runtime plan;
- [x] known-shape Tuple/Record Dynamic erasure has a replacement;
- [x] genuinely dynamic-shape Tuple APIs do not force an accidental existential feature;
- [x] metadata v8 compatibility is preserved unless implementation proves a structural schema extension necessary;
- [x] cold/incremental parity and LSP are included;
- [x] stale effective specifications are explicitly migrated;
- [x] lifecycle collision with the earlier corrective P4 is an implementation prerequisite, not ignored;
- [x] tests are tied to coverage obligations;
- [x] broad verification is delayed until one coherent certification gate;
- [x] STOP/CONSULT triggers cover every material architectural fork still possible;
- [x] mixed destructuring/general product relationship is explicitly excluded;
- [x] tasks form a dependency-ordered execution path from lifecycle normalization through checkpoint re-closure.

---

# 29. Executor summary

Implement P4 as a semantic convergence, not as a syntax patch.

The intended final architecture is:

```text
source declarations
    |
    |  bind expansion mode + static domain independently
    v
canonical declaration family
    |
    +--> product-shape template
    |       |
    |       +--> ordinary data: nominal specialization + exact product layout
    |       |
    |       +--> intrinsic Tuple/Record: structural canonical reduction
    |
    +--> associated source surface
    |       |
    |       +--> compiler intrinsic evaluator
    |
    +--> runtime-class projection
            |
            +--> one Tuple VM class
            +--> one Record VM class

Tuple<empty CompleteTypePack>
    = Unit
    = ()
    runtime value = Unit immediate
    runtime class = Tuple

Record<empty RecordFields>
    = EmptyRecord
    = {}
    runtime value = EmptyRecord immediate
    runtime class = Record

Unit != EmptyRecord
```

User syntax converges on:

```phalcom
intrinsic data Tuple<***Es>(***Es)
intrinsic data Record<**Fs> { **Fs }

intrinsic type Unit = Tuple<>
intrinsic type EmptyRecord = Record<>

{name: value, **record}
Map { name: value, **map }
#{a, b, c}
```

Do not reintroduce the old C3 workaround by making bare Tuple/Record proper `Type`s. Do not collapse RecordFields into labeled packs. Do not create per-specialization classes. Do not expose a generalized Tuple/Record relationship. Complete the already-established semantic authorities and make source, compiler, runtime, metadata, and tools agree on them.
