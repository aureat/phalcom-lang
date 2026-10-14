---
id: LANG007.C4.P8
kind: implementation-plan
title: "Incremental Durability, Metadata, Cross-Module Static Products, and Materialization"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
program: LANG007
checkpoint: LANG007.C4
branch: main
base_revision: cb8a068cd4ed7db2b9d17c0a4e096839c2a90ac0
prepared: 2026-10-13
depends_on:
  - LANG007.C4.P2
  - LANG007.C4.P3
  - LANG007.C4.P4
  - LANG007.C4.P5
  - LANG007.C4.P6
  - LANG007.C4.P7
blocks:
  - LANG007.C4.P9
---

# LANG007.C4.P8 — Detailed Implementation Plan

## Incremental Durability, Metadata, Cross-Module Static Products, and Materialization

---

# 1. Plan purpose

`LANG007.C4.P8` turns the C4 static system from a correct **single-semantic-session** feature into a durable **artifact/module boundary** feature.

P2–P7 establish semantic authorities inside one compiler session:

```text
TypeStore
StaticValueStore
StaticResidualStore
StaticDefinitionProduct
StaticConstantProduct
CheckedStaticBody
StaticEvaluator
SemanticDb
SemanticSnapshot
```

P8 must project those authorities into a stable artifact representation that can be validated, persisted, imported into a fresh semantic session, incrementally compared, consumed by compiler lowering, and materialized at runtime **without rerunning static source computation**.

The target architecture is:

```text
source compilation
    ↓
canonical P2–P7 semantic products
    ↓
P8 durable projection
    ↓
versioned validated metadata/artifact
    ↓
fresh importer
    ↓
reconstructed canonical semantic products
    ↓
existing semantic specialization/evaluation authorities
    ↓
frozen ModuleLoweringSemantics
    ↓
compiler/linker
    ↓
MaterializationDescriptor
    ↓
VM::ensure_static_constant_materialized
    ↓
canonical owner-held runtime value
```

P8 is not a new evaluator project.

The central rule is:

> **P8 serializes and reconstructs canonical meaning. It does not create parallel static semantics.**

---

# 2. Repository baseline and plan corrections

This plan was prepared after a second pass over `main`.

## 2.1 Audited revision

Current `main` at plan preparation:

```text
cb8a068cd4ed7db2b9d17c0a4e096839c2a90ac0
```

Commit message:

```text
fix(semantic): validate static residual specialization
```

The two commits after the earlier P8 requirements analysis modify P7 evaluator/specialization implementation and tests. They do not change the P8 metadata/materialization architecture, but they reinforce the P7 freeze prerequisite described below.

## 2.2 P8 is still not started in the lifecycle ledger

`LANG007.C4-CHECKPOINT.md` currently records:

```text
LANG007.C4.P8
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
```

No canonical P8 implementation plan exists in the repository at the audited revision.

## 2.3 Live metadata schema is v12

Current source:

```text
phalcom-type-meta/src/header.rs
```

defines:

```rust
pub const TYPE_METADATA_SCHEMA_VERSION: u32 = 12;
```

Therefore the older C4 planning statement that P8 evolves schema v10 is stale.

P8 begins from **v12**.

The preferred final C4 static-product release is one coherent **v13** schema unless implementation evidence forces a different decision.

## 2.4 Static query roots already exist

P8 must not add duplicate roots.

Current `phalcom-semantic/src/db/key.rs` already contains:

```rust
QueryKey::StaticDefinition(DeclarationId)
QueryKey::StaticConstant(StaticConstantId)
QueryKey::ModuleStaticBindingCatalog(ModuleId)
QueryKey::ModuleStaticBindings(ModuleId)
QueryKey::CheckedStaticBody(StaticBodyId)
```

Current `SemanticProduct` already contains corresponding typed products.

Therefore P8's task is:

```text
harden existing products
    +
give them durable projections
    +
give imported products an explicit reconstruction/publication path
```

not:

```text
invent P8-only query roots
```

## 2.5 Current materialization seam is already correct

The live constant path is:

```text
StaticConstantProduct
    ↓
StaticConstantProduct::materialization_descriptor
    ↓
MaterializationDescriptor
    ↓
StaticConstantLoweringSpec
    ↓
ModuleLoweringSemantics.static_constants
    ↓
VM::ensure_static_constant_materialized
    ↓
RuntimeStaticConstantRef
    ↓
ModuleObject.static_constants
```

P8 extends this seam.

It must not replace it.

## 2.6 P4 intentionally left metadata fail-closed

The semantic model supports:

```text
FieldKey = Symbol | String | Int | Bool
```

but type metadata still exposes:

```rust
RecordFieldRef {
    name: Box<str>,
    ...
}
```

and `MetadataExporter` intentionally reports:

```text
NonExportableRecordKey(...)
NonExportableRecordKeyDomain
```

for non-legacy cases.

That is not accidental technical debt.

It is an explicit P8 handoff.

## 2.7 Metadata encoding is JSON-only

The live codec is:

```text
encode_metadata_json
decode_metadata_json
serde_json
validate_metadata_bundle
```

There is no current binary/bincode metadata codec.

This matters for the v12→v13 compatibility strategy:

- field-name aliases can be handled by serde;
- legacy `RecordFieldRef.name` can be decoded as a Symbol `FieldKeyRef`;
- P8 does not need to preserve binary struct-layout compatibility.

---

# 3. Scope

P8 owns all of the following.

## 3.1 Durable identities

P8 owns stable artifact identities for semantic objects that currently exist only as session-local IDs when they cross the metadata boundary.

This includes:

```text
static constants
data components
variant fields
FieldKey values
StaticValue roots/nodes
static definition parameters/body operands
P7 residual operands, if residual export is selected
```

## 3.2 Durable static values

P8 must support the complete live `StaticValueData` family:

```text
Unit
Bool
Char
Int
Float
String
Symbol
Type
StaticTerm
Tuple
Sequence
Record
Data
Variant
Pack
```

## 3.3 Cross-module static constants

An exported `const` must carry its reduced semantic meaning across artifact boundaries.

The provider initializer must not execute in the importing compilation or at runtime.

## 3.4 Cross-module static definitions

A downstream compiler must be able to specialize an exported/imported static definition without requiring provider source re-analysis.

## 3.5 Metadata schema evolution

P8 owns the final C4 metadata representation for:

```text
FieldKey
StaticValue
StaticValueDomain
static constants
static definitions
durable residuals if required
```

## 3.6 Incremental durability

P8 owns stable product fingerprints and dependency behavior across fresh sessions/artifacts.

## 3.7 Compiler materialization

P8 completes general runtime materialization for C4 static values and makes artifact-backed products enter the existing `ModuleLoweringSemantics` path.

---

# 4. Explicit non-goals

P8 must not implement any of the following.

## 4.1 No second static evaluator

Forbidden:

```text
MetadataStaticEvaluator
ArtifactStaticInterpreter
VM static evaluator
compiler-time fallback evaluator in phalcom-core
```

Imported definitions must reconstruct or feed the existing P7 static semantics.

## 4.2 No second type serializer

Reuse:

```text
TypeNodeId
ScopedTypeNodeId
KindNodeId
StableDeclarationRef
StableTypeParameterRef
```

from `phalcom-type-meta`.

Do not embed a new recursive Type representation inside `StaticValueNode`.

## 4.3 No raw AST contract

Do not serialize:

```text
phalcom_ast::Expr
phalcom_ast::LexicalBlock
StaticDefinitionBodyRepr::Expression raw AST
StaticDefinitionBodyRepr::Block raw AST
```

as the public durable semantic contract.

## 4.4 No runtime const initializer execution

`const` is not a runtime global initializer.

Do not restore one.

## 4.5 No user-forgeable Product/Sum evidence

Imported structural capabilities remain compiler-derived from canonical declaration facts.

## 4.6 No global application cache by default

P7 evaluator-local memoization remains authoritative.

Do not introduce:

```rust
QueryKey::StaticApplication(...)
```

without concrete profiling evidence and a separately reviewed lifecycle design.

## 4.7 No P9 ontology cleanup

P9 owns final public TypePack/RecordFields retirement, documentation, tooling polish, and checkpoint-wide cleanup.

---

# 5. Hard architectural invariants

These invariants are normative acceptance criteria.

## P8-INV-01 — No process-local artifact identity

The durable representation must not use any of the following as semantic identity:

```text
TypeId
KindId
StaticValueStoreId
StaticValueId
StaticStringId
StaticSymbolId
StaticTupleId
StaticSequenceId
StaticRecordId
StaticDataValueId
StaticVariantValueId
StaticPackBackingId
StaticPackId
StaticResidualStoreId
StaticResidualId
ObjRef
VM Symbol
heap address
query arena index
```

## P8-INV-02 — Existing semantic authorities remain canonical

Metadata does not become an alternate semantic source.

On import, metadata is validated and reconstructed into canonical semantic products.

## P8-INV-03 — Stable Type identity reuses type metadata

A durable static Type/StaticTerm references the same stable Type graph used elsewhere in metadata.

## P8-INV-04 — Static values are DAGs

Durable structural values are acyclic immutable graphs.

No tree duplication requirement is imposed where DAG sharing exists.

## P8-INV-05 — Pack serialization is logical

Pack backing/slice offsets are not serialized as meaning.

Serialize the logical positional/labeled lanes.

## P8-INV-06 — Record keys are lossless

`FieldKey::Symbol`, `String`, `Int`, and `Bool` round-trip distinctly.

## P8-INV-07 — Legacy Record names mean Symbol

For schema ≤12:

```text
RecordFieldRef.name = "x"
```

decodes as:

```text
FieldKeyRef::Symbol("x")
```

never String.

## P8-INV-08 — Unordered Record identity uses canonical FieldKey order

Producer HashMap order and source reorder do not affect semantic Record fingerprints.

## P8-INV-09 — Tuple/Pack labels remain Symbol

P8 must not generalize ordered labels to `FieldKey`.

## P8-INV-10 — Exact Int is reused

P8 uses the v12 exact integer representation.

It does not create another arbitrary-precision integer codec.

## P8-INV-11 — Float payload is exact

Durable Float uses exact binary64 bits.

## P8-INV-12 — Semantic equality and representation identity are distinct

Metadata fingerprinting must follow the intended semantic/product contract, not accidentally inherit allocator identity.

## P8-INV-13 — Exported const value is interface meaning

Changing only an exported const value must invalidate consumers even when its type is unchanged.

## P8-INV-14 — Equivalent value may retain value identity

Two different initializers that normalize to the same durable value should have the same **value** fingerprint.

A source/body fingerprint may still differ.

## P8-INV-15 — Symbolic fingerprint is semantic

A symbolic result may not hash `StaticResidualStoreId`/`StaticResidualId`.

## P8-INV-16 — P7 freeze precedes residual schema freeze

No final durable residual schema is ratified while P7 residual families/normalization remain incomplete.

## P8-INV-17 — Compiler consumes frozen semantics

`phalcom-core` does not invoke `StaticEvaluator` during lowering.

## P8-INV-18 — Runtime consumes descriptors

The VM does not interpret static-definition bodies.

## P8-INV-19 — Constructors do not rerun

Static Data/Variant materialization structurally instantiates an already-proven value.

It does not execute user constructor/method source.

## P8-INV-20 — Constant publication is atomic

A failed structural materialization never leaves a partial owner constant.

## P8-INV-21 — One canonical runtime owner value

A successful heap-backed constant is cached in its canonical owner module and reused.

## P8-INV-22 — Imported re-exports preserve provider identity

A facade does not silently manufacture a new static constant identity.

## P8-INV-23 — Product/Sum evidence is recomputed or cryptographically bound to canonical shape

Serialized user claims do not become compiler-derived capability authority.

## P8-INV-24 — Metadata decode is bounded

Static metadata validation is iterative/resource-bounded.

## P8-INV-25 — Cold and incremental results agree

Durable product meaning must not depend on edit history.

## P8-INV-26 — Cancellation never publishes success

Cancelled export/import/evaluation does not become reusable current state.

## P8-INV-27 — Negative-to-positive transitions wake dependents

A previously blocked consumer must become valid when the missing durable dependency appears.

## P8-INV-28 — Removal invalidates

Removing a durable provider product retires stale importer state.

## P8-INV-29 — Deterministic producer output

Equivalent clean builds produce stable static metadata sections/fingerprints independent of store allocation order.

## P8-INV-30 — No silent compatibility reinterpretation

Every v12→v13 semantic reinterpretation is explicit and tested.

---

# 6. Target durable data model

The exact Rust names may be refined during implementation, but the semantic shape is fixed by this plan.

## 6.1 Stable FieldKey wire type

Recommended:

```rust
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub enum FieldKeyRef {
    Symbol(Box<str>),
    String(Box<str>),
    Int(StaticInt),
    Bool(bool),
}
```

Use the existing `phalcom_type_meta::scalar::StaticInt`.

Do not add a numeric ID table solely for keys.

## 6.2 Stable static value ID

```rust
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct StaticValueNodeId(pub u32);
```

This is an **artifact-pool-local index**, not language identity.

## 6.3 Stable static value domain

```rust
pub enum StaticValueDomainRef {
    Kind(KindNodeId),
    ValueType(TypeNodeId),
}
```

If the live `StaticValueDomain` can contain additional forms by implementation time, extend this enum exhaustively.

## 6.4 Stable sequence family

```rust
pub enum StaticSequenceFamilyRef {
    Array,
    Vector,
}
```

## 6.5 Stable identity additions

`phalcom-type-meta::identity` already contains:

```text
StableProjectRef
StableModuleRef
StableDeclarationRef
StableVariantRef
StableCallableRef
StableFieldRef
StableTypeMemberRef
StableImplRef
```

P8 should add only missing wire identities.

Recommended:

```rust
pub struct StableStaticConstantRef {
    pub module: StableModuleRef,
    pub name: Box<str>,
}

pub struct StableDataComponentRef {
    pub owner: StableDeclarationRef,
    pub index: u32,
}

pub struct StableVariantFieldRef {
    pub variant: StableVariantRef,
    pub index: u32,
}
```

No dedicated `StableDataConstructorRef` is required unless implementation evidence says otherwise because live `DataConstructorId` is canonically identified by its owner declaration.

For Data constructor metadata:

```text
constructor identity = owner StableDeclarationRef
```

is sufficient if and only if the one-constructor-per-data invariant remains true.

If that invariant changes, STOP AND CONSULT rather than silently extending the identity.

## 6.6 Static value node

Recommended conceptual model:

```rust
pub enum StaticValueNode {
    Unit,
    Bool(bool),
    Char(char),
    Int(StaticInt),
    FloatBits(u64),
    String(Box<str>),
    Symbol(Box<str>),

    Type(TypeNodeId),
    StaticTerm(TypeNodeId),

    Tuple(Box<[StaticValueNodeId]>),

    Sequence {
        family: StaticSequenceFamilyRef,
        result_type: TypeNodeId,
        elements: Box<[StaticValueNodeId]>,
    },

    Record(Box<[(FieldKeyRef, StaticValueNodeId)]>),

    Pack {
        positionals: Box<[StaticValueNodeId]>,
        labeled: Box<[(Box<str>, StaticValueNodeId)]>,
    },

    Data {
        owner: StableDeclarationRef,
        result_type: TypeNodeId,
        components: Box<[(StableDataComponentRef, StaticValueNodeId)]>,
    },

    Variant {
        variant: StableVariantRef,
        result_type: TypeNodeId,
        fields: Box<[(StableVariantFieldRef, StaticValueNodeId)]>,
    },
}
```

### Important physical-to-semantic projections

Live semantic:

```text
StaticPackBackingId + StaticPackData offsets
```

durable:

```text
logical Pack lanes
```

Live semantic:

```text
StaticStringId / StaticSymbolId
```

durable:

```text
text content
```

Live semantic:

```text
TypeId
```

durable:

```text
TypeNodeId in the same metadata bundle
```

## 6.7 Static value entry

Recommended:

```rust
pub struct StaticValueNodeEntry {
    pub node: StaticValueNode,
    pub structural_fingerprint: Fingerprint128,
}
```

The fingerprint must be based on stable child fingerprints/stable references, not numeric child IDs alone.

Numeric IDs can participate in the encoded artifact structure after deterministic canonical indexing, but the semantic fingerprint must survive independent reconstruction.

## 6.8 Durable static constant

Recommended:

```rust
pub struct StaticConstantRecord {
    pub id: StableStaticConstantRef,
    pub domain: StaticValueDomainRef,
    pub value: StaticValueNodeId,
    pub value_fingerprint: Fingerprint128,
    pub dependency_fingerprint: Fingerprint128,
}
```

Optional source/tooling provenance belongs in profile-specific fields/sections, not the semantic identity.

## 6.9 Durable static definition

The header can be fixed now:

```rust
pub struct StaticDefinitionRecord {
    pub id: StableDeclarationRef,
    pub parameters: Box<[StaticParameterRecord]>,
    pub result_domain: StaticValueDomainRef,
    pub body: DurableStaticBodyRef,
    pub semantic_fingerprint: Fingerprint128,
    pub dependencies: Box<[StaticDependencyRef]>,
}
```

The final form of `DurableStaticBodyRef` is **P7-gated**.

It must not be defined as raw AST.

---

# 7. Durable static-definition body decision

This is P8's highest-risk design point.

## 7.1 Required properties

The imported body must preserve enough checked semantics to execute/specialize through P7 without provider source.

It must preserve canonical identities for:

```text
static applications
member projections
proof predicates
control flow
match/pattern meaning
structural construction
static iteration
result domains
relevant dependency facts
```

## 7.2 Preferred design

After P7 closes, derive a stable normalized static semantic IR from the P7 residual/term vocabulary.

Conceptually:

```text
checked static definition
    ↓
stable semantic template
    ↓
import
    ↓
session-local P7 terms
    ↓
StaticEvaluator / specialize_residual
```

The imported representation should use stable refs and artifact-pool node IDs.

## 7.3 Rejected design

Do not serialize the entire live `CheckedStaticBody` structure mechanically.

It contains:

```text
Expr
LexicalBlock
ExpressionId
BindingId
TypeId
source-range keyed evaluator layout
query-local maps
```

These are not appropriate durable semantics.

## 7.4 Residual export policy

At P8.T0/P8.T7, choose one policy explicitly.

### Policy A — definitions are durable, residual results remain session-local

Preferred if imported static definitions can reconstruct all required open specialization work.

### Policy B — normalized residual DAG is durable

Required only if:

- residual values themselves are public artifact products; or
- artifact-only specialization cannot reproduce necessary semantic work from the durable definition body.

If Policy B is required, serialize only the normalized P7 semantic residual DAG.

Never serialize:

```text
evaluator frames
memo maps
active recursion stack
budget state
cancellation state
source AST pointers
StaticResidualStoreId
StaticResidualId
```

---

# 8. Schema strategy

## 8.1 One coherent v13 release

Preferred sequence:

```text
v12 live baseline
    ↓
staging types implemented but not emitted
    ↓
all core static wire types ready
    ↓
single TYPE_METADATA_SCHEMA_VERSION = 13 cutover
```

Do not bump the version once for every P8 task.

## 8.2 New metadata features

Extend `MetadataFeatures` explicitly.

Recommended fields:

```rust
#[serde(default)]
pub field_keys: bool,

#[serde(default)]
pub static_values: bool,

#[serde(default)]
pub static_products: bool,

#[serde(default)]
pub static_residuals: bool,
```

Exact grouping may be reduced if a smaller coherent feature set is preferable.

The key rule is:

> a decoder can tell whether a bundle claims to contain each new semantic capability.

## 8.3 Bundle additions

Preferred typed fields:

```rust
#[serde(default)]
pub static_values: Box<[StaticValueNodeEntry]>,

#[serde(default)]
pub static_constants: Box<[StaticConstantRecord]>,

#[serde(default)]
pub static_definitions: Box<[StaticDefinitionRecord]>,

#[serde(default)]
pub static_residuals: Box<[StaticResidualNodeEntry]>, // only if Policy B
```

Typed bundle fields are preferred over opaque `MetadataExtensionSection` payloads because these products affect semantic compilation.

## 8.4 Module roots

Do not require scanning the entire bundle to discover one module's static products if avoidable.

Either:

1. extend `ModuleMetadataRoot` with typed static-product root arrays; or
2. keep globally sorted static records keyed by `StableModuleRef` and build one validated index at decode.

Preferred if schema churn remains manageable:

```rust
pub struct ModuleMetadataRoot {
    ...
    #[serde(default)]
    pub static_constants: Box<[StaticConstantRecordId]>,
    #[serde(default)]
    pub static_definitions: Box<[StaticDefinitionRecordId]>,
}
```

If this introduces needless indirection, globally sorted records keyed by stable owner are acceptable.

Choose one policy in T3 and test deterministic ordering.

---

# 9. Compatibility strategy

## 9.1 `RecordFieldRef.name` migration

Current:

```rust
pub struct RecordFieldRef {
    pub name: Box<str>,
    pub ty: TypeNodeId,
    ...
}
```

Target:

```rust
pub struct RecordFieldRef {
    pub key: FieldKeyRef,
    pub ty: TypeNodeId,
    ...
}
```

Because the codec is JSON-only, P8 can support legacy JSON using serde-compatible decoding.

Recommended strategy:

- serialize v13 as `"key": ...`;
- accept legacy `"name": "x"` during decode;
- map it to `FieldKeyRef::Symbol("x")`;
- validate schema-version/feature consistency after deserialization.

Do the equivalent for scoped record fields.

## 9.2 Old bundles

P8 must retain the current supported version floor unless an explicit architecture decision changes it.

Older bundles:

- have no static value pool;
- have no static product section;
- use legacy Symbol Record names.

Do not fabricate static products that the old bundle never encoded.

## 9.3 New bundles to old readers

v12 readers are allowed to reject v13.

That is the purpose of the version bump.

---

# 10. Task graph

The intended task dependency graph is:

```text
T0  takeover / freeze gates
 │
 ├───────────────┐
 │               │
 v               v
T1 identity     T2 static value wire/fingerprint
 │               │
 └───────┬───────┘
         v
T3 schema v13 + FieldKey cutover
         │
         v
T4 semantic value/constant export
         │
   ┌─────┴────────────┐
   v                  v
T5 import values     T6 durable const interface/incrementality
   │                  │
   └─────┬────────────┘
         │
         │          P7 COMPLETE
         │              │
         │              v
         │             T7 durable static definition/residual model
         │              │
         └───────┬──────┘
                 v
               T8 semantic importer + cross-module specialization
                 │
       ┌─────────┴──────────┐
       v                    v
T9 structural materialize  T10 compiler/linker artifact consumption
       └─────────┬──────────┘
                 v
               T11 incremental/cold parity
                 │
                 v
               T12 robustness/performance
                 │
                 v
               T13 certification/lifecycle closure
```

---

# 11. Parallelization rules

## Safe parallel work

After T0:

### Lane A — metadata primitives

```text
T1 stable identity
T2 stable value wire/fingerprint
```

can proceed largely in parallel.

### Lane B — runtime structural materialization research

The implementer may audit existing Product/Pack/Data/Variant runtime construction in parallel, but should not land the final P8 descriptor shape before T2 defines stable structural value semantics.

### Lane C — P7 completion

P7 can finish independently.

P8 T1–T6 should not require P7 residual freeze.

## Hard serialization barrier

Do not begin T7 final wire implementation until P7 is accepted complete enough that:

```text
residual families are fixed
normalization is fixed
specialization is fixed
trusted proof transport is fixed
static control/structural residual behavior is fixed
```

---

# 12. P8.T0 — Takeover, baseline, and P7 freeze contract

## Objective

Record the exact implementation baseline and prevent P8 from serializing unstable P7 internals.

## Read first

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md
docs/implementation/LANG007/LANG007.C4/P7/lang007.c4.p7-audit.md
docs/implementation/LANG007/LANG007.C4/P7/LANG007.C4.P7-luna-patch-grade-implementation-plan.md
LANG007.C4.P8-implementation-requirements-analysis.md
```

## Repository audit targets

```text
phalcom-type-meta/src/{header,bundle,identity,type_node,scoped_type,validate,encode,fingerprint}.rs

phalcom-semantic/src/
    metadata/{export,stable_identity}.rs
    db/{key,product,query,fingerprint}.rs
    static_value/store.rs
    static_constant.rs
    static_definition.rs
    static_eval/{residual,specialize,machine,application}.rs
    materialization.rs
    snapshot.rs

phalcom-modules/src/{interface,fingerprint,linker}.rs

phalcom-core/src/modules/
    semantic_lowering.rs
    static_constant.rs
    materialize.rs
    linkage.rs
```

## Required T0 output

Add/update P8 lifecycle planning record with:

- exact base revision;
- live schema version;
- exact P7 status;
- static value variant inventory;
- materialization capability matrix;
- known metadata fail-closed cases;
- chosen residual freeze gate.

## Gate G0

G0 passes when:

1. P8 tasks independent of P7 are explicitly separated from P7-gated tasks;
2. no existing stable identity or wire product has been overlooked;
3. no existing query root is duplicated by the plan;
4. the implementer can state whether residual export is currently blocked.

## STOP / CONSULT

Stop if P7 has changed the residual/result model after this plan such that §7 no longer describes the live system.

---

# 13. P8.T1 — Stable identity and `FieldKeyRef` foundation

## Objective

Introduce only the missing stable wire identities and `FieldKeyRef`, without yet flipping the bundle schema.

## Primary write set

```text
phalcom-type-meta/src/identity.rs
phalcom-type-meta/src/lib.rs
phalcom-type-meta/src/field_key.rs            # new, preferred
phalcom-semantic/src/metadata/stable_identity.rs
phalcom-semantic/src/metadata/mod.rs
```

## Required additions

### `StableStaticConstantRef`

```rust
pub struct StableStaticConstantRef {
    pub module: StableModuleRef,
    pub name: Box<str>,
}
```

Conversion authority:

```text
StaticConstantId(SymbolId)
    ->
StableIdentityContext / ProjectUniverse
    ->
StableStaticConstantRef
```

### `StableDataComponentRef`

```rust
pub struct StableDataComponentRef {
    pub owner: StableDeclarationRef,
    pub index: u32,
}
```

### `StableVariantFieldRef`

```rust
pub struct StableVariantFieldRef {
    pub variant: StableVariantRef,
    pub index: u32,
}
```

### `FieldKeyRef`

Implement:

```text
Symbol(Box<str>)
String(Box<str>)
Int(StaticInt)
Bool(bool)
```

with semantic ordering:

```text
Symbol < String < Int < Bool
```

and within-category ordering matching P4.

## Required conversion functions

Add explicit conversion functions rather than ad hoc pattern matches throughout exporter code.

Conceptually:

```rust
to_stable_static_constant(...)
to_stable_data_component(...)
to_stable_variant(...)
to_stable_variant_field(...)
to_field_key_ref(...)
```

and later import inverses.

## Required tests

Add focused tests for:

- variant fields with same index but different variant do not collide;
- data components with same index but different owner do not collide;
- same static constant name in two modules does not collide;
- `FieldKeyRef::Symbol("x") != FieldKeyRef::String("x")`;
- exact large/negative Int keys round-trip;
- ordering equals P4 ordering.

## Gate G1

G1 passes when all new stable identities:

- are content/stable-reference based;
- contain no current-session IDs;
- round-trip through serde;
- have collision tests.

No schema version bump yet.

---

# 14. P8.T2 — Durable `StaticValue` DAG and semantic fingerprint

## Objective

Define an artifact-safe value graph covering every live `StaticValueData` form.

## Primary write set

```text
phalcom-type-meta/src/static_value.rs          # new
phalcom-type-meta/src/lib.rs
phalcom-type-meta/src/fingerprint.rs           # if shared helpers/tags are added

phalcom-semantic/src/metadata/static_value.rs  # new
phalcom-semantic/src/metadata/mod.rs
phalcom-semantic/src/metadata/export.rs        # exporter hooks, not bundle publication yet
```

## Do not modify

Do not repurpose:

```rust
StaticValueStore::content_fingerprint
```

into a cross-session API by forcing metadata context into the core store.

Keep the in-session method.

Add a separate durable projection/fingerprint path.

## T2 algorithm

### Step 1 — root projection

Given:

```text
StaticValueRef
StaticValueStore
TypeStore
MetadataExporter
StableIdentityContext
```

project the root recursively/iteratively into a temporary logical node graph.

### Step 2 — Type references

For every `TypeId`:

```text
MetadataExporter::export_type_form
    ->
TypeNodeId
```

must remain the sole type wire authority.

### Step 3 — child projection

Project structural children.

### Step 4 — content interning

Deduplicate identical durable nodes.

### Step 5 — topological ordering

Assign node IDs deterministically so every child is either:

- already defined; or
- validation explicitly permits forward references.

Preferred: child-before-parent topological ordering.

### Step 6 — stable node fingerprint

Fingerprint:

```text
variant tag
stable scalar content
stable declaration refs
stable child fingerprints
stable type structural fingerprints
```

Do not define semantic fingerprint as a hash of `StaticValueNodeId`.

## Pack rule

For:

```text
StaticPackData { backing, offset, len }
```

read the logical slice and serialize only those logical items.

## Type/StaticTerm rule

Hash the exported Type node's structural fingerprint.

Do not hash raw `TypeId.0`.

## Sequence rule

Fingerprint includes:

```text
family
stable result type fingerprint
element fingerprints
```

## Data rule

Fingerprint includes:

```text
stable owner
stable component identities/order
stable result type
component value fingerprints
```

## Variant rule

Fingerprint includes:

```text
StableVariantRef
stable result type
StableVariantFieldRef order
field fingerprints
```

## Required focused tests

Create a new test module such as:

```text
phalcom-semantic/tests/semantic/integration/static_value_metadata.rs
```

Required cases:

1. primitives;
2. Char;
3. >64-bit Int;
4. -large Int;
5. `+0.0` and `-0.0` preserve exact bits;
6. NaN bit payload survives round-trip;
7. String/Symbol same text remain distinct;
8. tuple;
9. sequence Array;
10. sequence Vector;
11. Record all four key kinds;
12. Data;
13. Variant;
14. Pack root;
15. Pack slice serializes logical content;
16. repeated shared child is deduplicated;
17. equivalent values allocated in different store lineages yield equal durable fingerprints.

## Gate G2

G2 passes when every live `StaticValueData` form has a stable node representation and independent-store equivalence is proven.

---

# 15. P8.T3 — Atomic schema v13 and `FieldKey` metadata cutover

## Objective

Publish the P8 wire types in `SemanticMetadataBundle` and remove the legacy Symbol-only Record bottleneck.

## Primary write set

```text
phalcom-type-meta/src/header.rs
phalcom-type-meta/src/bundle.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/encode.rs                 # only if decode compatibility helper needed
phalcom-type-meta/src/fingerprint.rs
phalcom-type-meta/tests/schema_compat.rs

phalcom-semantic/src/metadata/export.rs
phalcom-semantic/tests/semantic/integration/metadata.rs

phalcom-core/src/typing/loader.rs
phalcom-core/src/typing/identity.rs
phalcom-core/src/typing/environment.rs

all compile failures from RecordFieldRef.name migration
```

## Schema version

Bump exactly once in this task:

```rust
TYPE_METADATA_SCHEMA_VERSION: 12 -> 13
```

Update the header comment with the exact v13 features.

## `RecordFieldRef`

Target:

```rust
pub struct RecordFieldRef {
    pub key: FieldKeyRef,
    pub ty: TypeNodeId,
    pub presence: StructuralMemberPresenceRef,
}
```

Make the equivalent scoped-type change.

## Legacy JSON decoding

Support v12 JSON where fields use:

```json
{"name":"x", ...}
```

and convert to:

```text
FieldKeyRef::Symbol("x")
```

Implementation may use:

- `#[serde(alias = "name")]` plus a compatible field-key deserializer; or
- a custom compatibility DTO/deserializer.

Choose the smallest approach that preserves explicit semantics.

## `SemanticMetadataBundle`

Add typed static sections with `#[serde(default)]`.

At minimum:

```text
static_values
static_constants
static_definitions
```

If T7 residual policy is not yet ready:

```text
static_residuals
```

may remain absent until T7 if the schema can later add it without another semantic break, or may be included as an empty feature-gated section whose actual node set is not yet emitted.

Do not freeze a wrong residual enum merely to keep the version bump singular.

If avoiding a later v14 requires prematurely encoding unstable P7 semantics, prefer correctness over version-count aesthetics and STOP AND CONSULT.

## `MetadataFeatures`

Add explicit feature bits.

Validation must reject a bundle that contains new sections while claiming the corresponding feature is absent, unless compatibility policy explicitly permits it.

## Validation limits

Extend:

```rust
ValidationLimits
```

with bounded dimensions such as:

```text
max_static_value_nodes
max_static_value_edges
max_static_constants
max_static_definitions
max_static_definition_nodes
max_static_residual_nodes       # if applicable
```

Avoid only using `max_total_bytes`; structural counts matter.

## Validation rules

### Record keys

- sorted by canonical `FieldKeyRef` order where unordered structure requires sorting;
- no duplicate semantic keys;
- tail/remainder domain remains valid.

### Static value nodes

- indices in bounds;
- topological order valid;
- no cycles;
- stable refs structurally valid;
- TypeNode refs in bounds;
- sequence result types in bounds;
- Data/Variant payload identities structurally coherent where enough metadata exists.

## Schema compatibility tests

Extend `schema_compat.rs` with:

1. v12 `name` Record field decodes as Symbol key;
2. v13 Symbol key round-trip;
3. v13 String key;
4. v13 Int key;
5. v13 Bool key;
6. invalid duplicate keys rejected;
7. invalid order rejected;
8. old bundles default new static sections empty;
9. v13 static value section round-trips;
10. unsupported future schema rejected.

## Gate G3

G3 passes when:

- schema v13 is coherent;
- v12 compatibility is explicit;
- `NonExportableRecordKey` / `NonExportableRecordKeyDomain` are no longer required for semantics representable in v13;
- runtime typing consumers compile against `FieldKeyRef`.

---

# 16. P8.T4 — Semantic static-value and constant exporter

## Objective

Project current semantic values/constants into the new bundle deterministically.

## Primary write set

```text
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/metadata/static_value.rs
phalcom-semantic/src/metadata/stable_identity.rs
phalcom-semantic/src/static_constant.rs          # only helper APIs if required
phalcom-semantic/src/snapshot.rs                 # read-only accessor additions if required
```

## Extend `MetadataExporter`

Add explicit optional authorities:

```text
StaticValueStore
StaticConstantTable
StaticDefinitionTable / products
```

Do not make `MetadataExporter` discover these globally.

Recommended builder methods:

```rust
with_static_values(...)
with_static_constants(...)
with_static_definitions(...)
```

or one snapshot-backed entry point if that is architecturally cleaner.

## Root selection

Only serialize static constants/definitions required by the selected metadata profile and exported/public artifact surface.

Do not dump all private session values by default.

## Static constant record

For every selected successful constant:

1. obtain canonical `StaticConstantId`;
2. map to `StableStaticConstantRef`;
3. resolve result domain;
4. export the value root;
5. compute stable value fingerprint;
6. compute dependency fingerprint;
7. emit `StaticConstantRecord`.

## Do not export LKG as current semantics

`lkg_value` is presentation/recovery state.

Do not serialize it as the current constant value.

If tooling wants LKG, that is a separate profile/tooling decision.

## Failed constants

A failed current constant should not produce a successful durable value record.

Artifact creation should either:

- fail for required exported constants; or
- explicitly omit the invalid product and retain diagnostics according to existing artifact failure policy.

Do not serialize a stale value as current.

## Deterministic ordering

Sort roots by stable identity.

Do not use current table insertion order as artifact meaning.

## Gate G4

G4 passes when two independent clean semantic sessions compiling equivalent source produce identical static value/constant sections and stable fingerprints.

---

# 17. P8.T5 — Semantic static-value and constant importer

## Objective

Create an explicit semantic import seam that reconstructs stable static values and constants into a fresh semantic session.

## Primary write set

```text
phalcom-semantic/src/metadata/import.rs            # new
phalcom-semantic/src/metadata/mod.rs
phalcom-semantic/src/static_value/store.rs         # only canonical import helpers if justified
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/session.rs                    # integration point
```

## Do not reuse VM typing loader as semantic importer

Current:

```text
phalcom-core::typing::loader::load_metadata_bundle
```

is a runtime typing consumer.

P8 requires a semantic importer in `phalcom-semantic`.

## Import phases

### Phase 1 — validate bundle

Use `validate_metadata_bundle`.

### Phase 2 — resolve stable project/module/declaration identities

Build explicit mappings.

### Phase 3 — reconstruct types

Use the existing type metadata import/reconstruction authority or add a narrowly scoped semantic type importer if no complete path exists.

Do not recreate Type formation ad hoc in the static importer.

### Phase 4 — reconstruct static values

Process topological `StaticValueNodeEntry` order.

For each node:

```text
stable child IDs
    ->
current StaticValueRef children
    ->
StaticValueStore canonical intern
```

### Phase 5 — publish static constants

Reconstruct:

```text
StableStaticConstantRef
    ->
StaticConstantId
```

and insert/import a product whose current reduced value is the new current-session `StaticValueRef`.

The imported product must retain:

- stable provider identity;
- durable value fingerprint;
- dependency information sufficient for invalidation.

## Imported product model

Do not pretend imported constants have a local source initializer.

If `StaticConstantProduct` structurally requires a `CheckedStaticConstantInitializer`, introduce an explicit imported/durable product representation or generalize the product so source-backed and imported-backed origins are distinct.

Preferred:

```rust
enum StaticConstantOrigin {
    Source(Arc<CheckedStaticConstantInitializer>),
    Imported(ImportedStaticConstantOrigin),
}
```

or equivalent.

Do not fabricate an AST initializer.

## Gate G5

G5 passes when:

```text
export bundle
    ->
destroy source/session
    ->
new semantic session
    ->
import bundle
    ->
lookup constant
```

produces semantically equal current-session static values.

---

# 18. P8.T6 — Exported const interface and incremental dependency semantics

## Objective

Make durable constant values participate in module/interface invalidation and imported dependency edges.

## Primary write set

```text
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/state.rs          # only if imported publication needs it
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/checker/analysis.rs  # only dependency enum if extension required

phalcom-modules/src/fingerprint.rs
phalcom-modules/src/interface.rs          # avoid widening target unless proven needed

phalcom-semantic/tests/semantic/incremental/*
```

## Preserve existing source interface category

Keep:

```rust
ModuleBindingSemantics::StaticConstant
```

## Do not widen `LinkedExportTarget` unless needed

Current canonical target:

```rust
LinkedExportTarget::Binding(SymbolId)
```

already preserves provider identity.

Prefer attaching durable value/product fingerprint through semantic product/interface metadata rather than turning target identity into a value payload.

## Product fingerprint correction

Current successful `StaticConstantProduct` uses `StaticValueStore::content_fingerprint`.

Current symbolic outcome hashes:

```text
residual store ID
residual ID
```

P8 must remove that artifact-invalid behavior from any durable/public fingerprint.

At minimum:

- successful imported/exported constants use durable value fingerprint;
- symbolic/public products use stable semantic residual fingerprint if they may cross the boundary;
- otherwise symbolic constants are not publishable durable constants.

## Required incremental tests

Add focused tests:

### T6-A

```text
provider: export const x: Int = 10
consumer: uses x
edit provider x -> 20
```

consumer invalidates.

### T6-B

Same type, same name, same export target must still invalidate on value change.

### T6-C

Change initializer but produce same canonical value.

Expected:

```text
value fingerprint unchanged
```

Any broader source product may change.

### T6-D

Change unrelated private const.

Unrelated importer is reused.

### T6-E

Facade re-export preserves original provider constant identity/fingerprint.

### T6-F

Delete exported const.

Importer invalidates and diagnoses missing binding/product.

### T6-G

Previously missing/invalid provider const becomes valid.

Importer recomputes successfully.

## Gate G6

G6 passes when public constant invalidation is value-sensitive and edit-history independent.

---

# 19. P8.T7 — Durable static-definition body and residual policy

## Hard dependency

Do not start final T7 implementation until P7 is sufficiently complete and recertified.

## Objective

Define the stable semantic representation needed to specialize imported static definitions.

## Primary write set

Likely:

```text
phalcom-type-meta/src/static_definition.rs       # new
phalcom-type-meta/src/static_residual.rs         # only if Policy B
phalcom-type-meta/src/bundle.rs
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/fingerprint.rs

phalcom-semantic/src/metadata/static_definition.rs  # new
phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/static_eval/residual.rs         # stable projection helpers only
phalcom-semantic/src/static_definition.rs
```

## P7 freeze checklist

Before wire design, confirm final live P7 support for:

```text
Parameter
Apply
Proof
MemberProjection
If
Match
Pack
Tuple
Record
Data
Variant
structural iteration / recursive structural semantics as applicable
```

and any new residual families added after this plan.

## Stable parameter record

Conceptually:

```rust
pub struct StaticParameterRecord {
    pub index: u32,
    pub name: Box<str>,          // presentation; not identity
    pub domain: StaticValueDomainRef,
    pub mode: GenericParameterModeRef,
}
```

If a kinded parameter needs stable `StableTypeParameterRef`, include it.

## Stable body operands

Every body node that references a Type/value/declaration must use:

```text
TypeNodeId
StaticValueNodeId
StableDeclarationRef
StableTypeMemberRef
StableVariantRef
StableDataComponentRef
StableVariantFieldRef
stable proof proposition
```

## Match/pattern semantics

P6 handoff requires P8 to preserve checked match/iteration/proof semantics.

Do not serialize parser patterns.

Serialize canonical semantic pattern identities/structures sufficient to invoke the same P6 pattern matcher after reconstruction.

## Proof environment

Do not serialize "trusted" proof merely because a public proposition exists.

Preserve the same trust boundary P7 uses between:

```text
public proposition
trusted branch proof context
```

## Gate G7

G7 passes when the stable definition body:

- contains no AST;
- contains no source-local `ExpressionId`/`BindingId` as cross-artifact identity;
- can be reconstructed into the existing P7 execution/specialization machinery;
- preserves canonical match/proof/member identity.

---

# 20. P8.T8 — Imported static definitions and artifact-only specialization

## Objective

Import durable static definitions and prove they can specialize without provider source.

## Primary write set

```text
phalcom-semantic/src/metadata/import.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/resolver.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/{query,fingerprint,product}.rs
phalcom-semantic/src/static_eval/*
```

## Imported product origin

As with constants, do not fabricate local source bodies.

If needed, generalize:

```rust
StaticDefinitionProduct
```

so a definition can have:

```text
Source checked origin
Imported durable origin
```

while exposing one canonical semantic definition interface to consumers.

## Publication

Imported definitions must publish under the existing:

```rust
QueryKey::StaticDefinition(DeclarationId)
```

after stable identity reconstruction.

Do not add an imported-specific query namespace.

## Dependency edges

When an imported definition depends on another durable static product, current DB dependencies must point at current reconstructed QueryKeys.

## Artifact-only test harness

Build a test that explicitly does:

```text
compile provider source
export metadata
drop provider source/session
create consumer session
load provider artifact only
compile/specialize consumer
```

This is mandatory.

A one-workspace test where provider products remain live in memory does not satisfy P8.

## Required specialization cases

At minimum:

1. generic type-valued definition;
2. ordinary value-domain definition;
3. recursive changing-argument definition;
4. proof predicate;
5. structural reflection projection;
6. conditional static computation;
7. match static computation;
8. structural Pack/Record/Data result;
9. imported static constant read inside definition.

## Gate G8

G8 passes when provider source is absent and imported specialization matches source-cold semantics.

---

# 21. P8.T9 — General structural materialization

## Objective

Complete the runtime side of C4 static constants.

## Current live capability

`MaterializationDescriptor` supports:

```text
Unit
Bool
Char
Int
Float
String
Symbol
Sequence
```

and defers:

```text
Tuple
Record
Data
Variant
Pack
```

## Primary write set

```text
phalcom-semantic/src/materialization.rs
phalcom-core/src/modules/static_constant.rs
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/product/*
phalcom-core/src/heap/pack.rs
phalcom-core/src/data.rs
phalcom-core/src/adt.rs
phalcom-core/src/vm/adt.rs
phalcom-core/src/compiler/lib/expr.rs

tests:
phalcom-core/tests/core/compiler/static_const.rs
plus focused module/heap tests
```

Only touch the runtime storage files actually required after audit.

## Descriptor design

Do not keep adding deeply nested recursive enum trees if P8 durable DAGs would be duplicated.

Preferred options:

### Option A — descriptor DAG

```text
MaterializationNodeId
MaterializationNode
```

projected from durable/static values.

### Option B — descriptor references durable static value DAG

If dependency layering permits core to consume a compact immutable semantic descriptor without retaining the entire semantic store.

Choose based on crate boundaries.

## Tuple

Materialize through canonical Product/Tuple representation.

Preserve layout optimization.

## Record

Descriptor must include:

```text
canonical FieldKey list / RecordProductShape
child descriptors
```

Use existing Record product shape authority.

## Pack

Use existing Pack runtime representation.

Preserve positional/labeled separation.

Do not build a list of pairs.

## Data

Use canonical Data layout/constructor identity.

Do not call source constructor logic.

## Variant

Use canonical runtime variant identity/descriptor.

Do not rerun case selection.

## Atomic publication

Preserve the current Sequence precedent:

1. construct complete child graph under temporary roots;
2. if any child fails, publish nothing;
3. restore temporary-root depth;
4. after success, insert one canonical owner value.

## Required tests

For each structural family:

- same-module constant read;
- imported constant read;
- exported constant read;
- nested structural values;
- repeated read returns same owner value where heap identity is observable;
- forced GC keeps root and children alive;
- failure leaves no owner cache entry.

## Gate G9

G9 passes when no `DeferredStructural` remains for C4-supported immutable structural value families.

---

# 22. P8.T10 — Compiler/linker artifact consumption

## Objective

Make source-backed and artifact-backed modules enter one compiler/runtime lowering path.

## Primary write set

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/materialize.rs
phalcom-core/src/modules/context.rs
phalcom-core/src/modules/linkage.rs
phalcom-core/src/compiler/lib/{mod,expr}.rs
phalcom-core/src/vm/bootstrap.rs       # only if Universe path requires adjustment

phalcom-modules/src/interface.rs
phalcom-modules/src/linker.rs
```

## Core rule

By codegen time:

```text
all required static semantic work is complete
```

Compiler input should be equivalent regardless of origin:

```text
source provider
    -> SemanticSnapshot
    -> ModuleLoweringSemantics

artifact provider
    -> semantic metadata import
    -> reconstructed snapshot/product view
    -> ModuleLoweringSemantics
```

## No fallback

If an expected imported static product is absent:

```text
error / unsupported metadata
```

not:

```text
evaluate source
dynamic global
Nil/None
best effort
```

## Re-export path

Current:

```text
materialize_linked_exports_for_module
    ->
LinkedExportTarget::Binding(SymbolId)
    ->
static_constant_spec(provider)
```

is the right identity path.

Artifact-backed provider lowering must populate the same `static_constant_spec` authority.

## Gate G10

G10 passes when the compiler can compile a consumer against provider artifact static semantics with provider source evaluation disabled/unavailable.

---

# 23. P8.T11 — Cold/incremental parity and durable dependency certification

## Objective

Prove the whole P8 product graph is incrementally sound.

## Primary write set

Mostly tests plus any defects discovered in:

```text
phalcom-semantic/src/db/*
phalcom-semantic/src/metadata/*
phalcom-modules/src/fingerprint.rs
```

## New focused suite

Preferred new module:

```text
phalcom-semantic/tests/semantic/incremental/static_artifacts.rs
```

## Required matrix

| Scenario | Expected |
|---|---|
| exported const value changes, type unchanged | importer invalidated |
| initializer changes, reduced value unchanged | value-only dependency reusable |
| private unused const changes | unrelated importer reused |
| static definition body changes | dependent specializations invalidate |
| unrelated static definition changes | unrelated consumers reused |
| Product field shape changes | reflection-dependent consumer invalidates |
| unordered Record fields reorder semantically | semantic value/shape fingerprint stable |
| variant declaration order changes | `Variants` dependents invalidate |
| variant payload type changes | dependents invalidate |
| provider artifact replaced with semantically identical build | consumer reusable |
| missing product becomes available | prior blocked consumer recomputes |
| exported product removed | stale consumer invalidated |
| cancellation during provider recompute | no success reuse |
| retained old snapshot after update | old view remains self-consistent |
| clean build after edit sequence | equals incremental result |

## Cold parity comparison

Compare:

```text
semantic static value
result domain
diagnostics
stable value fingerprint
static-definition fingerprint
dependency set
metadata static section
runtime behavior
```

Do not compare session-local IDs.

## Gate G11

G11 passes only if the complete matrix has explicit recorded evidence.

---

# 24. P8.T12 — Decoder robustness and performance

## Objective

Treat metadata as bounded structured input and prove the new system is not pathologically expensive.

## Primary write set

```text
phalcom-type-meta/src/validate.rs
phalcom-type-meta/src/encode.rs
phalcom-type-meta/tests/schema_compat.rs

phalcom-semantic/src/metadata/import.rs
benchmarks / test-timings infrastructure where appropriate
```

## Validation attacks

Add negative tests for:

```text
static value child out of range
forward/topology violation
explicit cycle if wire permits it
too many value nodes
too many edges
too many constants
too many definitions
duplicate Record keys
duplicate Pack labels
noncanonical Record order
invalid Char scalar
malformed StaticInt
invalid TypeNode ref
unknown declaration ref
Data component owner mismatch
Variant field owner mismatch
wrong field count
wrong result domain
residual result-domain mismatch
oversized JSON
deep adversarial DAG
```

## Performance workloads

Measure and record:

### Metadata

- baseline metadata size before P8;
- v13 size after P8 for representative project;
- 1,000 exported primitive constants;
- 1,000 shared structural constants;
- one very large Pack;
- one very large Record;
- one shared nested DAG.

### Decode

- validation time;
- import/reintern time;
- memory peak if instrumentation exists.

### Incremental

- one exported const change with many consumers;
- one private static edit;
- repeated artifact reload with identical fingerprint.

### Static definition

- repeated imported specialization of same definition;
- changing arguments;
- memoized subproblem workload.

### Runtime

- startup with unused large constant;
- first materialization cost;
- repeated read cost;
- GC after materialization.

## Required asymptotic properties

Target:

```text
static value export/import: O(nodes + edges)
validation:                 O(nodes + edges)
materialization:            O(materialized nodes + edges)
repeated owner read:        O(1) map/cache lookup expected
```

Avoid:

```text
O(n²) repeated prefix copying
recursive host stack proportional to value depth
one full descriptor tree copied per use
```

## Gate G12

G12 passes when malformed metadata fails boundedly and the named workloads have recorded measurements with no known pathological scaling.

---

# 25. P8.T13 — Final certification and lifecycle closure

## Objective

Close P8 as an auditable compiler architecture phase.

## Required documents

Create:

```text
docs/implementation/LANG007/LANG007.C4/P8/
    LANG007.C4.P8-detailed-implementation-plan.md
    LANG007.C4.P8-walkthrough.md
    LANG007.C4.P8-handoff.md
```

The plan file may live in the established C4 naming convention if the repository chooses a sibling layout; do not duplicate canonical plans in two locations.

## Checkpoint update

Update:

```text
LANG007.C4-CHECKPOINT.md
```

with:

```text
status
completion
verification
commits
schema version
test evidence
known residuals
P9 handoff
```

## Final regression gates

At minimum:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta --test schema_compat

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic \
  --test semantic \
  integration::metadata \
  -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic \
  --test semantic \
  foundations::static_constants \
  -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic \
  --test semantic \
  integration::static_definition_bodies \
  -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-semantic \
  --test semantic \
  incremental \
  -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test \
  -p phalcom-core \
  --test core \
  static_const \
  -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

If filters select zero tests, the gate is invalid.

Record selected/pass/fail/ignored counts.

## Gate G13

P8 may be marked:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

only after all G0–G12 requirements are either passed or explicitly superseded by a recorded architecture decision.

---

# 26. Source-file implementation map

This section is intended as a practical patch guide.

## 26.1 `phalcom-type-meta`

### `src/header.rs`

Modify:

```text
TYPE_METADATA_SCHEMA_VERSION
MetadataFeatures
schema comments
```

### `src/identity.rs`

Add only missing stable wire identities.

Reuse existing:

```text
StableVariantRef
StableCallableRef
StableDeclarationRef
```

### `src/field_key.rs` — new

Own:

```text
FieldKeyRef
legacy compatibility decode helper if appropriate
```

### `src/static_value.rs` — new

Own:

```text
StaticValueNodeId
StaticValueNode
StaticValueNodeEntry
StaticSequenceFamilyRef
StaticValueDomainRef
StaticConstantRecord / IDs if kept here
```

If static constant records fit better in a separate `static_product.rs`, split deliberately.

### `src/static_definition.rs` — P7-gated new file

Own stable static-definition/header/body wire forms.

### `src/static_residual.rs` — optional/P7-gated

Only if residual export Policy B is ratified.

### `src/type_node.rs`

Migrate:

```text
RecordFieldRef.name
    ->
RecordFieldRef.key
```

### `src/scoped_type.rs`

Migrate scoped Record key representation equivalently.

### `src/bundle.rs`

Add typed static sections/root links.

### `src/validate.rs`

Add static graph budgets and semantic validation.

### `src/fingerprint.rs`

Add stable static tags/builders only if this is the shared correct layer.

### `src/encode.rs`

Keep JSON bounded decode.

Only add compatibility preprocessing/custom decode if serde alone is insufficient.

### `tests/schema_compat.rs`

Own version compatibility.

---

# 27. `phalcom-semantic` implementation map

## 27.1 `src/metadata/stable_identity.rs`

Add conversion authorities.

Do not duplicate project/declaration resolution logic elsewhere.

## 27.2 `src/metadata/static_value.rs` — new

Recommended responsibilities:

```text
StaticValueRef -> durable node
durable node fingerprint
durable node -> StaticValueRef import helper
```

Import may be split into `import.rs` if preferred.

## 27.3 `src/metadata/export.rs`

Extend `MetadataExporter`.

Remove legacy non-exportability only after v13 `FieldKeyRef` path exists.

## 27.4 `src/metadata/import.rs` — new

Own semantic reconstruction from validated metadata.

## 27.5 `src/db/fingerprint.rs`

Fix public/durable static product fingerprints.

Keep session-only fingerprints explicitly labeled where they remain.

## 27.6 `src/db/query.rs`

Imported products should publish through existing QueryKeys.

Preserve dependency recording rules.

## 27.7 `src/static_constant.rs`

Generalize source/imported origin if necessary.

Do not fabricate source initializer state.

## 27.8 `src/static_definition.rs`

Generalize source/imported durable body origin only if needed.

Preserve one consumer-facing semantic product.

## 27.9 `src/static_value/store.rs`

Only add narrowly scoped canonical intern/import helpers.

Do not teach the core store about metadata concepts.

## 27.10 `src/static_eval/*`

Only add stable reconstruction hooks required by T7/T8.

No duplicate evaluator.

---

# 28. `phalcom-modules` implementation map

## `src/interface.rs`

Preserve:

```text
ModuleBindingSemantics::StaticConstant
LinkedExportTarget::Binding(SymbolId)
```

unless direct evidence proves the interface cannot carry enough semantic identity.

## `src/fingerprint.rs`

Ensure provider module public fingerprint changes when exported durable static meaning changes through the appropriate semantic/interface product integration.

Do not simply hash a giant serialized blob if more precise product fingerprints exist.

## `src/linker.rs`

Preserve canonical provider `SymbolId` through re-exports.

---

# 29. `phalcom-core` implementation map

## 29.1 `src/modules/semantic_lowering.rs`

Current:

```rust
fn project_static_constants(...)
```

projects from:

```text
SemanticSnapshot.static_constants()
SemanticSnapshot.static_values()
```

P8 should make imported products appear through the same snapshot/product seam or provide one explicitly equivalent projection input.

Do not branch:

```text
if artifact { special artifact lowering }
else { source lowering }
```

when one canonical semantic projection can serve both.

## 29.2 `src/modules/static_constant.rs`

Extend `materialize_descriptor`.

Preserve:

```text
owner cache
temporary roots
no partial publication
GC tracing
```

## 29.3 `src/modules/materialize.rs`

Continue resolving export/import bindings through canonical provider identity.

## 29.4 `src/compiler/lib/expr.rs`

Remove `DeferredStructural` failure paths as each structural descriptor becomes supported.

Do not evaluate source.

## 29.5 Runtime product/ADT files

Reuse existing runtime layouts.

P8 is not authorization for a second Record/Data/Variant/Pack object family.

---

# 30. Fingerprint architecture

P8 must distinguish three concepts.

## 30.1 Query input fingerprint

Answers:

> Did the inputs to this current query change?

May legitimately contain current-session product fingerprints.

## 30.2 Query product fingerprint

Answers:

> Did this current semantic product change within this compiler's incremental graph?

Must be semantically meaningful enough for dependency reuse.

## 30.3 Durable artifact fingerprint

Answers:

> Is this semantic artifact meaning the same across independent sessions/builds?

Must contain no process-local identity.

Do not conflate all three merely because they are hashes.

Recommended naming if new types are needed:

```text
ProductFingerprint            # current DB
Fingerprint128                # durable metadata
```

Use existing `phalcom_type_meta::Fingerprint128` for artifact-stable content rather than introducing a third arbitrary hash width.

---

# 31. Static constant publication law

The final provider flow should be:

```text
source const
    ↓
CheckedStaticConstantInitializer
    ↓
StaticEvaluator
    ↓
StaticEvalOutcome::Reduced
    ↓
StaticValueRef
    ↓
durable StaticValueNodeId
    ↓
StaticConstantRecord
    ↓
provider interface/artifact fingerprint
```

Importer:

```text
StaticConstantRecord
    ↓
validate
    ↓
reconstruct StaticValueRef
    ↓
publish existing StaticConstant query/product identity
    ↓
consumer dependency
```

Runtime:

```text
imported/source StaticConstantProduct
    ↓
MaterializationDescriptor
    ↓
same ModuleLoweringSemantics path
```

---

# 32. Static definition publication law

Target:

```text
source static definition
    ↓
StaticDefinitionProduct
    +
CheckedStaticBody
    ↓
P7 normalized checked semantics
    ↓
DurableStaticBodyRef
    ↓
StaticDefinitionRecord
```

Importer:

```text
StaticDefinitionRecord
    ↓
validate stable refs/types/values
    ↓
reconstruct current-session semantic body
    ↓
publish QueryKey::StaticDefinition
    ↓
same StaticEvaluator/specializer
```

Forbidden:

```text
StaticDefinitionRecord
    ↓
deserialize AST
    ↓
re-run checker from source-like AST
```

That would make the artifact a source cache rather than a durable semantic product.

---

# 33. Artifact profile policy

P8 must ratify profile behavior.

Recommended:

## `RuntimeMinimal`

Retain:

- runtime-used exported/static constants;
- materialization-required type/value records;
- runtime public type metadata needed by the VM.

Omit static-definition bodies not needed at runtime.

## `RuntimePublic`

Retain public constant values and public runtime-reflection semantics.

Static definitions needed only for downstream compilation belong to the compile/interface artifact profile or a required static-products section if `RuntimePublic` doubles as compiler interface metadata.

## `ToolingDebug`

May retain source provenance and richer definition metadata.

## `Proof`

Retain proof-relevant static dependencies/evidence according to existing Proof profile policy.

If the repository needs a distinct compile-interface profile to avoid overloading `RuntimePublic`, STOP AND CONSULT rather than stuffing all compiler semantics into runtime-minimal bundles.

---

# 34. Materialization representation choice gate

Before T9 lands, choose between:

## Model A — self-contained materialization DAG

Pros:

- core does not need semantic static-value codec;
- runtime descriptor is purpose-specific.

Cons:

- can duplicate static value DAG;
- requires its own stable node representation.

## Model B — durable/static value DAG directly drives materialization projection

Pros:

- one structural graph;
- less duplication;
- direct artifact-backed constants.

Cons:

- crate boundary/lifetime may be more complex.

### Recommendation

Prefer a compact materialization DAG **projected once per module** from the static value DAG if `phalcom-core` cannot safely retain metadata static nodes directly.

Do not recursively clone `MaterializationDescriptor` trees at every read.

---

# 35. Diagnostics

P8 should add explicit diagnostics/errors for:

```text
metadata schema unsupported
static product feature missing
static value node malformed
stable declaration unresolved
stable static constant unresolved
static definition dependency unresolved
artifact static domain mismatch
artifact FieldKey invalid
artifact Data component mismatch
artifact Variant field mismatch
artifact residual unsupported
artifact static product fingerprint mismatch
runtime static materialization unsupported
```

Do not collapse these into generic "internal error" where the artifact is external input.

Compiler/runtime invariant violations that can only arise from compiler-produced validated metadata may remain internal errors if that distinction is proven.

---

# 36. Security and robustness

P8 metadata is structured input.

Treat it accordingly.

## Required protections

- byte-size limit before JSON parse;
- node-count limits;
- edge-count limits;
- no recursive descent proportional to adversarial DAG depth if avoidable;
- all indices checked before indexing;
- no `unwrap` on artifact-controlled identity;
- duplicate-key rejection;
- topological validation;
- explicit feature/schema checks;
- no gigantic diagnostic string generation from malicious graph dumps.

---

# 37. Test organization recommendation

Add focused files rather than overloading old suites.

Recommended:

```text
phalcom-type-meta/tests/schema_compat.rs                  # extend

phalcom-semantic/tests/semantic/integration/
    metadata.rs                                           # existing compatibility/integration
    static_value_metadata.rs                              # new
    static_artifact_import.rs                             # new

phalcom-semantic/tests/semantic/incremental/
    static_artifacts.rs                                   # new

phalcom-core/tests/core/compiler/
    static_const.rs                                       # extend

phalcom-core/tests/core/modules/
    static_artifact.rs                                    # new if module organization supports it
```

Exact module placement should follow the repository's current test module registration.

---

# 38. Mandatory acceptance fixtures

## 38.1 Heterogeneous Record

Provider:

```telos
export const r = {
    #symbol: 1,
    "string": 2,
    42: 3,
    true: 4,
}
```

Acceptance:

- metadata round-trip;
- artifact import;
- static lookup/reflection;
- runtime materialization;
- identical semantic key order.

## 38.2 Pack slice

Build a Pack, slice it statically, export the slice.

Acceptance:

- metadata contains only logical sliced lanes;
- importing store has no dependency on producer backing identity.

## 38.3 Data value

Export a static immutable data value.

Acceptance:

- stable owner/components;
- no source constructor execution;
- runtime product layout correct.

## 38.4 Variant value

Export enum variant with payload.

Acceptance:

- StableVariantRef/field identity;
- correct payload;
- runtime materialization;
- source declaration order remains intact for `Variants`.

## 38.5 Recursive imported static definition

Provider:

```telos
type F<...> = ...
```

Consumer specializes it.

Acceptance:

- provider source absent;
- exact cycles remain cycles;
- changing arguments remain budgeted computation, not false cycle.

## 38.6 Proof-dependent imported definition

Acceptance:

- canonical proof dependencies reconstructed;
- public proposition cannot forge trusted branch proof.

## 38.7 Re-export

```text
provider -> facade -> consumer
```

Acceptance:

- provider constant/definition identity preserved;
- one provider value/materialization;
- invalidation propagates.

---

# 39. Incident triggers

The implementer must file an implementation incident and stop the affected task when any of these occur.

## Trigger 1 — P7 wire instability

A residual family needed by imported definitions is still changing semantically.

## Trigger 2 — raw TypeId required in metadata

The stable type graph cannot currently express a required static value/body type.

## Trigger 3 — raw AST appears necessary

Imported specialization seems to require AST/checker rerun.

## Trigger 4 — importer needs a second solver

Stable metadata cannot reconstruct canonical meaning without independent inference/proof logic.

## Trigger 5 — metadata `FieldKey` cannot remain lossless

Any proposal suggests stringification.

## Trigger 6 — compiler needs evaluator fallback

Artifact product is insufficient for lowering.

## Trigger 7 — runtime needs source constructor execution

Materialization path cannot structurally create Data/Variant value.

## Trigger 8 — cold/incremental mismatch

After one serious attempt to fix an inconsistency, cold and incremental results still disagree.

## Trigger 9 — schema compatibility ambiguity

A v12 field can map to more than one v13 semantic meaning.

## Trigger 10 — stable identity collision

Two semantically distinct declarations/components/fields/constants map to the same stable wire identity.

---

# 40. Commit discipline

Recommended commit boundaries:

```text
P8.T0   docs/lifecycle baseline
P8.T1   stable identity + FieldKeyRef
P8.T2   StaticValue DAG/fingerprint
P8.T3   schema v13 atomic cutover
P8.T4   semantic exporter
P8.T5   semantic importer values/constants
P8.T6   const interface/incrementality
P8.T7   durable static definitions/residual policy
P8.T8   imported static specialization
P8.T9   general materialization
P8.T10  compiler/linker artifact consumption
P8.T11  incremental parity
P8.T12  robustness/performance
P8.T13  lifecycle closeout
```

Avoid mixing P9 ontology migration into these commits.

---

# 41. Required evidence format per task

Every task completion entry should record:

```text
task ID
commit SHA
source files changed
tests run
selected test count
passed
failed
ignored
baseline failures if any
new diagnostics
performance data if applicable
remaining blockers
```

Do not write:

```text
tests passed
```

without the command and selected count.

---

# 42. Final completion matrix

| Gate | Requirement | Completion evidence |
|---|---|---|
| G0 | takeover/P7 freeze policy | lifecycle record |
| G1 | stable missing identities + FieldKeyRef | focused type-meta tests |
| G2 | complete StaticValue DAG | independent-store roundtrip/fingerprint tests |
| G3 | schema v13 / Record key cutover | schema compatibility + validator tests |
| G4 | deterministic exporter | clean-build equality |
| G5 | semantic value/const importer | fresh-session import |
| G6 | const interface incrementality | edit matrix |
| G7 | durable static definition body | P7-gated semantic IR tests |
| G8 | artifact-only specialization | provider-source-absent tests |
| G9 | general structural materialization | Tuple/Record/Pack/Data/Variant runtime tests |
| G10 | compiler/linker one-path consumption | artifact-only compile |
| G11 | cold/incremental parity | full durable dependency matrix |
| G12 | robustness/performance | malformed corpus + measurements |
| G13 | lifecycle closure | walkthrough/handoff/checkpoint |

---

# 43. Definition of done

`LANG007.C4.P8` is complete only when the following statements are true.

## Durable metadata

- `TYPE_METADATA_SCHEMA_VERSION` explicitly represents the completed C4 durable-static contract.
- all four `FieldKey` kinds round-trip losslessly;
- every live stable `StaticValueData` form round-trips through an independent session;
- no producer store/arena IDs survive as semantic identity;
- equivalent clean builds produce equal static metadata fingerprints.

## Constants

- exported const value content participates in downstream invalidation;
- provider initializers never execute in importers or runtime;
- re-exports preserve canonical provider identity;
- structural constants materialize through the canonical descriptor path.

## Static definitions

- imported definitions specialize without provider source;
- imported execution uses P7 semantic machinery;
- no raw AST is the public semantic contract;
- proof/match/reflection semantics remain canonical.

## Incrementality

- value changes invalidate;
- semantically unchanged values can reuse value identity;
- removal and appearance transitions are correct;
- cancellation does not publish;
- cold and incremental builds agree.

## Runtime

- primitive/sequence/Tuple/Record/Pack/Data/Variant constants are supported;
- one owner-held heap value is reused;
- GC rooting is correct;
- failed materialization publishes nothing;
- user constructors/methods are not rerun.

## Compiler

- compiler consumes frozen static products;
- there is no opportunistic static evaluator call in `phalcom-core`;
- artifact-only providers enter the same lowering semantics as source providers.

---

# 44. Handoff to P9

P8 should leave P9 a system where static durability is already correct.

P9 may then focus on:

```text
public ontology cutover
legacy TypePack/RecordFields retirement
tooling presentation
documentation
checkpoint-wide performance certification
final lifecycle closure
```

P9 must not inherit unresolved:

```text
FieldKey metadata
static value serialization
artifact static definitions
const invalidation
structural materialization
```

from P8.

---

# 45. Final implementation directive

The P8 implementation should be approached as a **serialization/reconstruction boundary around existing semantic authority**, not as another semantic feature layer.

The highest-value design discipline is:

```text
do semantic work once
        ↓
freeze stable meaning
        ↓
transport stable meaning
        ↓
reconstruct canonical meaning
        ↓
reuse existing semantic/compiler/runtime authorities
```

The most dangerous anti-pattern is:

```text
serialize partial compiler state
        ↓
discover it is insufficient
        ↓
re-run checker/evaluator in compiler/runtime
        ↓
create a second semantics
```

If the implementation begins drifting toward that second shape, stop and file an architecture incident.

The intended P8 end state is strict:

> A provider's static semantics can be compiled once, persisted, loaded in a new process, specialized correctly by downstream code, lowered without semantic rediscovery, and materialized without re-executing source initialization.

That is the completion standard for `LANG007.C4.P8`.
