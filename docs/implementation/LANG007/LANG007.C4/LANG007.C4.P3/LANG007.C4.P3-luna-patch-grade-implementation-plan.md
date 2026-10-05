---
id: LANG007.C4.P3
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: implementation-plan
title: "First-class Pack and Call/Rest Convergence"
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
depends_on:
  - LANG007.C4.P2
follows: LANG007.C4.P2
supersedes: null
prepared_against_revision: 5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521
prepared: 2026-10-05
---

# LANG007.C4.P3 — First-class Pack and Call/Rest Convergence

## Luna patch-grade implementation plan

> **Planning status:** execution-ready implementation plan. This document does **not** claim that P3 source implementation or P3 verification has begun.
>
> **Repository anchor:** `aureat/phalcom-lang`, `main`, `5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521`.
>
> **Planning basis:** `LANG007.C4.P3-requirements-analysis.md`, the C4 checkpoint record, the P2 walkthrough/handoff, repository workflow contracts, and a fresh read-only audit of the live P3 implementation seams.

---

# 0. Executor contract

This plan is intended for GPT-5.6 Luna High/Extra High implementation under the repository's normal architectural-escalation discipline.

The implementer may adapt private helper names and mechanically drifted paths after re-reading the live tree. The implementer must **not** silently redesign Pack semantics, type identity, static-value ownership, GC lifetime, call/rest behavior, semantic authority, metadata identity, or the fixed-call fast path.

The implementer must:

- read the required records before editing;
- inspect `git status --short` and preserve unrelated user/agent work;
- execute tasks in dependency order unless a task explicitly permits overlap;
- keep semantic analysis as the canonical source of Pack/type/construction identity;
- preserve the C1–C3 TypePack algebra as optimized type-level pack authority;
- extend P2's `StaticValueStore` rather than create a second static Pack arena;
- keep `ArgumentPackBuilderObject` compiler/VM-private;
- keep ordinary statically shaped calls on the existing direct invoke path;
- run the smallest discriminating test at each task and broaden only at named gates;
- classify unrelated failures A/B/C/D under repository workflow;
- STOP AND CONSULT when a global or local trigger fires;
- update `LANG007.C4-CHECKPOINT.md` only at durable implementation events;
- create the required walkthrough and handoff only after actual P3 completion.

Testing is evidence gathering, not ritual. A focused PASS does not authorize unrelated cleanup or automatic workspace-wide testing.

---

# 1. Goal

Implement one coherent first-class `Pack` abstraction whose semantic meaning is shared by:

```text
proper Pack types
first-class runtime Pack values
P2 compile-time StaticPack values
* / ** / *** rest capture
* / ** / *** spreading and forwarding
C1–C3 optimized TypePack / LabeledTypePack / CompleteTypePack terms
```

while preserving:

```text
fixed statically shaped call
    -> existing direct argument window
    -> existing direct/specialized invoke
    -> zero required first-class Pack allocation
    -> zero required ArgumentPackBuilder allocation
```

P3 is successful when Pack is a real immutable structural transport value, all language rest-capture entry paths return Packs rather than Tuples/Lists, Pack spreading can feed calls and explicit Tuple construction without intermediate product conversion, compile-time Pack values use P2's semantic value system, and existing C1–C3 pack terms remain the optimized internal type-level machinery.

---

# 2. Checkpoint acceptance objective

`LANG007.C4` is the Structural Static Upgrade checkpoint. P3 is one bounded plan inside that checkpoint, not checkpoint closure.

P3 establishes the structural transport substrate required by later plans:

```text
P2 — semantic static values and evaluator
        ↓
P3 — first-class Pack type/value + call/rest convergence
        ├───────────────┐
        ↓               ↓
P4 — FieldKey       later static structural algorithms
        └──────┬────────┘
               ↓
P5 — Product / OrderedProduct / UnorderedProduct / Sum reflection
               ↓
P6+ — static structural iteration/matching/recursion/durability
```

P3 does **not** close C4. P4–P9 remain separate checkpoint work.

---

# 3. Repository grounding

Prepared against:

```text
repository: aureat/phalcom-lang
branch: main
revision: 5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521
latest completed plan: LANG007.C4.P2
active checkpoint plan candidate: LANG007.C4.P3
```

## 3.1 Live facts verified during planning

| Area | Live fact | P3 consequence |
|---|---|---|
| Lifecycle | P2 is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; P3 is `PROPOSED / NOT_STARTED / UNVERIFIED`. | Do not reopen P2. Mark P3 in progress only when implementation starts. |
| AST lanes | `ExpansionMode::{Positional,Labeled,Complete}`, `PackItem`, AST `RestMode`, and static parameter `mode` already exist. | Reuse the three-lane model; no second lane enum. |
| Typed remainders | `AnonymousRemainderSyntax` and `RemainderLaneMode` already model typed/unconstrained ordered tails. | Reuse this concept for `Pack<*: T>`, `Pack<**: T>`, `Pack<***: T>`. |
| Static parameters | `StaticParameterSyntax` retains mode, but semantic `StaticParameter` drops it for value-domain parameters. | P3 must preserve mode as canonical signature data. |
| C3 pack algebra | `TypeData::{TypePack,LabeledTypePack,CompleteTypePack,TypePackConcat,LabeledTypePackConcat,TupleFromPacks,CallableFromPacks}` and `GenericParameterMode` are live. | First-class Pack bridges to these; it does not replace them. |
| Ordinary relation | ordinary relation operates on `KindId::TYPE`; kinded pack terms are not proper Types. | First-class Pack needs a distinct proper-Type representation. |
| Type metadata | metadata already has TypePack variants, TupleFromPacks, open Tuple remainder forms, and scoped equivalents. | New proper Pack types require minimal stable metadata support in P3. |
| P2 static values | `StaticValueData` contains primitive values plus Tuple/Record/Data/Variant; Pack is absent by design. | Extend the same store. |
| Outgoing call builder | `ArgumentPackBuilderObject` owns mutable positional/label/value vectors and is compiler/VM-private. | Retain as staging infrastructure; add immutable Pack finalization. |
| Duplicate labels | builder uses `labels.contains`. | Large labeled assembly needs thresholded indexed membership. |
| Dynamic sends | compiler emits one builder then `InvokePack` / `SuperSendPack`; VM consumes it directly. | Do not create a surface Pack merely to perform a dynamic call. |
| Dynamic Tuple | expanded Tuple construction ends in `FinishTuplePack`. | Keep it; add a separate Pack finalizer. |
| Fixed-call guard | `outgoing_packs_completion.rs` already proves static calls/static Tuple avoid builder machinery. | Extend this guard; regressions are blocking. |
| Method rest | `VM::call_rest_method_as_with_environments` creates Tuple for positional/labeled/split/complete rest. | Replace all modes with one Pack capture authority. |
| Block rest | block rest currently materializes Tuple. | Converge to Pack. |
| Fiber rest | Fiber entry currently materializes List for excess/rest positionals. | Converge if this is the same language-rest contract. |
| Product substrate | Tuple/Record share `ProductStorage`, `ProductLayout`, shape registries, anonymous descriptors, and GC layout tracing. | Reuse storage/layout, not Tuple identity. |
| Value tags | tag space currently reaches `UninitializedBinding = 11`. | Immediate `EmptyPack` is available subject to exhaustive audit. |
| Runtime exact type | anonymous product descriptors may retain exact runtime type, with limited live consumers. | Pack roots may reuse descriptors; slices must not fabricate layout-invalid descriptors. |
| Test architecture | semantic tests own formal identities; core tests own compiler/runtime/GC/performance. | Place tests by architectural responsibility. |

> Re-read every named live path before editing. Adapt mechanical drift locally. Treat architectural drift as a STOP / CONSULT condition.

## 3.2 Planning audit verification strength

This repository audit was read-only. It did not execute P3 behavior tests and does not change P3 from `UNVERIFIED`. P2's prior verification is inherited evidence only.

---

# 4. Required reads before implementation

Read, in order:

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
docs/workflow/shared-consultation-escalation-protocol.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3-luna-patch-grade-implementation-plan.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3-requirements-analysis.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P2-walkthrough.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P2-handoff.md
```

Then read the live versions of the task-owned paths:

```text
phalcom-ast/src/{ast,parser}.rs

phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/types/{parameter,store,relation,substitution,instantiation,type_lambda}.rs
phalcom-semantic/src/metadata/{export,import}.rs

phalcom-type-meta/src/{type_node,scoped_type,validate}.rs

phalcom-core/src/heap/pack_builder.rs
phalcom-core/src/product/**
phalcom-core/src/parameters.rs
phalcom-core/src/method/mod.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/{block,fiber}.rs
phalcom-core/src/value/{repr,mod,render}.rs
phalcom-core/src/heap/{object,mod,accessors,trace}.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-native-meta/src/universe.rs

phalcom-semantic/tests/semantic/README.md
phalcom-core/tests/README.md
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
```

Follow `docs/specs/README.md` to any effective Pack/rest/type-application specifications named by the live index.

---

# 5. Normative authority

## 5.1 Fixed language behavior

Canonical model:

```telos
intrinsic data Pack<***Es>(***Es)

type PositionalPack<T: Type> = Pack<*: T>
type LabeledPack<T: Type> = Pack<**: T>
type CompletePack<T: Type> = Pack<***: T>

type PositionalTypePack = PositionalPack<Type>
type LabeledTypePack = LabeledPack<Type>
type CompleteTypePack = CompletePack<Type>
```

Semantic rules:

```text
Pack is immutable.
Pack has positional and labeled lanes.
Order is semantic within each lane.
Labeled encounter order is semantic.
Pack labels are Symbol in P3.
Duplicate labels are invalid.
Pack equality/hash is structural over visible lane contents.
Pack() is a distinct empty Pack value.
Pack != Tuple.
Pack != Record.
Pack() != Unit.
Pack() != EmptyRecord.
```

### Lane syntax

```text
*    positional lane
**   labeled lane
***  complete positional+labeled structure
```

`...` is **not** Pack rest/spread syntax.

### Rest result

```text
*rest     -> positional Pack
**rest    -> labeled Pack
***rest   -> complete Pack
split *rest + **rest -> two Packs
empty residual lane -> Pack()
```

No language rest path may continue returning Tuple or List.

### Tuple/Record boundary

Tuple and Pack are distinct ordered products; movement is explicit by construction/spread. Record is not implicitly a labeled Pack and does not become ordered in P3.

### Rest annotation meaning

`**args: T` constrains every captured labeled **value** to `T`; it does not mean the complete `args` Pack itself has type `T`. Exact captured shape uses a Pack-shape type form.

## 5.2 Fixed internal authority

The C1–C3 optimized pack terms remain canonical internals:

```text
TypeData::TypePack
TypeData::LabeledTypePack
TypeData::CompleteTypePack
TypeData::TypePackConcat
TypeData::LabeledTypePackConcat
GenericParameterMode::{Single,PositionalPack,LabeledPack,CompletePack}
```

They are kinded compile-time terms, not proper first-class Pack types. P3 adds an explicit bridge rather than merging categories.

P2's `StaticValueStore` remains the sole compile-time value store. The private runtime `ArgumentPackBuilderObject` remains staging infrastructure rather than a surface value.

## 5.3 Explicit later-plan scope

P3 must not implement or require:

- P4 heterogeneous `FieldKey` / unordered-product calculus;
- P5 Product/Sum reflection and structural capabilities;
- P6 static `for`, static Iterator, or general structural matching;
- P7 general recursive residualization/memoization;
- P8 durable StaticPack value persistence or full structural-const materialization;
- P9 final retirement of legacy public TypePack names.

---

# 6. Takeover state

## 6.1 Stable interface map

| Concept | Current authority | P3 use | Forbidden replacement |
|---|---|---|---|
| lane syntax | AST `ExpansionMode`, `PackItem`, `RestMode` | source rest/spread/Pack contributions | new lane enum |
| generic source mode | `GenericParameterMode` | kinded and value-domain variadics | encoding mode into kind |
| type-level positional pack | `TypeData::TypePack` | C3 Type lane | StaticPack array |
| type-level labeled pack | `TypeData::LabeledTypePack` | C3 labeled Type lane | Record row |
| complete type pack | `TypeData::CompleteTypePack` | paired C3 lanes | proper Pack type |
| symbolic composition | C3 concat nodes | substitution/normalization | new P3 concat hierarchy |
| Tuple symbolic composition | `TupleFromPacks` | implementation precedent | Pack-as-Tuple |
| static parameter identity | `StaticParameter`, TypeParameterId for Kind only | add canonical mode | fabricated type parameter |
| static values | `StaticValueStore` / `StaticValueRef` | add StaticPack | parallel store |
| static resources | `QueryBudget` + cancellation | Pack work charging | hidden Pack cap |
| outgoing assembly | `ArgumentPackBuilderObject` | calls + Pack finalization | public mutable Pack |
| call shape | `ParameterShape`, `RestLayout`, `ArgumentShape` | binding/rest | reparsed selectors |
| runtime storage | ProductStorage/layout registries | Pack roots | List/DataObject backing |
| dynamic call consume | `InvokePack`, `SuperSendPack` | unchanged | Pack materialization before send |
| dynamic Tuple finish | `FinishTuplePack` | unchanged Tuple construction | generic Pack finish alias |
| fixed calls | existing direct invoke family | unchanged | universal builder path |
| type metadata | type-meta + semantic export/import | proper Pack type persistence | raw TypeId persistence |

## 6.2 P2 invariants inherited unchanged

- Kind-domain terms and ordinary static values remain distinct.
- Only Kind-domain static parameters receive `TypeParameterId`.
- `StaticValueRef` is store-qualified and append-stable.
- StaticValue nodes contain no runtime `Value`, `ObjRef`, or VM Symbol identity.
- semantic equality/hash/fingerprint are content-based rather than arena-address based.
- checked static evaluation consumes ordinary semantic checker products rather than source-name resolution.
- budget/cancellation outcomes remain distinct and are not cached as successful reduction.
- P2's runtime materialization supports its named primitive vertical only; unsupported structural values fail closed.

---

# 7. Architecture

## 7.1 Authority flow

```text
source
  Pack(...), Pack<...>, * / ** / *** rest/spread
        ↓
AST lane/remainder nodes
        ↓
semantic authority
  canonical Pack declaration identity
  proper Pack type + C3 pack-term bridge
  checked Pack construction/spread facts
  StaticParameter mode
  StaticValue::Pack
        ↓ stable lowering facts
compiler
  fixed Pack construction -> direct Pack product build
  expanded Pack construction -> private builder + FinishPack
  dynamic call -> private builder + InvokePack (no surface Pack)
  fixed call -> existing direct Invoke
        ↓
VM/runtime
  EmptyPack immediate
  immutable Pack root/slice objects
  ProductStorage-backed root
  GC-safe root-referenced O(1) slices
  one shared rest-capture primitive
  direct Pack-lane append into outgoing builders
```

Compiler/runtime must consume canonical semantic identity/facts. No arbitrary `name == "Pack"` checks.

## 7.2 Proper semantic Pack type

### Selected representation

Add a proper `KindId::TYPE` Pack representation, conceptually:

```rust
pub struct PackTypeElement {
    pub label: Option<Box<str>>,
    pub ty: TypeId,
}

pub struct PackType {
    pub elements: Box<[PackTypeElement]>,
    pub positional_remainder: AnonymousTupleRemainder,
    pub labeled_remainder: AnonymousTupleRemainder,
}

pub enum TypeData {
    // existing...
    Pack(PackType),
    PackFromPacks {
        positionals: TypeId, // KindId::TYPE_PACK
        labeled: TypeId,    // KindId::LABELED_TYPE_PACK
    },
}
```

Private names are flexible. Required semantics are not:

1. Pack is a proper Type.
2. Concrete fixed C3 lanes normalize into the same canonical concrete Pack type as direct formation.
3. `Pack<*: T>`, `Pack<**: T>`, `Pack<***: T>` use typed anonymous lane remainders, not TypeParameters.
4. symbolic C3 lane terms remain `PackFromPacks` until substitution makes them concrete.
5. normalization re-checks labeled uniqueness after substitution.
6. empty Pack type is a proper Pack type, never Unit.
7. no Pack-specific inference-variable arena or second solver is added.

### Canonical formation/projection

Provide one formation path, conceptually:

```rust
TypeStore::pack(PackType) -> Result<TypeId, PackFormationError>
TypeStore::pack_from_packs(positionals: TypeId, labeled: TypeId) -> Result<TypeId, PackFormationError>
TypeStore::project_pack_positionals(pack: TypeId) -> Result<TypeId, ...>
TypeStore::project_pack_labeled(pack: TypeId) -> Result<TypeId, ...>
```

If existing generic pack projection APIs can be generalized without ambiguity, prefer reuse to near-duplicate public helpers.

### Relation policy

Do not implement `Pack -> Tuple -> relation` conversion.

Reuse/extract ordered-product relation mechanics:

```text
ordered structural relation helper
    common: fixed positional alignment
            fixed labeled alignment
            typed/open remainder constraints
            label encounter order
    Tuple policy
    Pack policy
```

Cross-identity Pack-vs-Tuple relation is refuted unless an explicit conversion is being type-checked.

For unresolved `PackFromPacks`, reuse C3 pack equivalence/substitution facts. If canonical relation cannot prove a result, preserve its normal unknown/blocked outcome; do not guess.

## 7.3 Pack type syntax and C3 bridge

`Pack<*: T>`, `Pack<**: T>`, and `Pack<***: T>` are anonymous typed remainders. They are distinct from expansion of an existing pack term such as `Pack<*P>`.

Preferred AST shape:

```rust
pub struct TypePackRemainderSyntax {
    pub mode: ExpansionMode,
    pub remainder: AnonymousRemainderSyntax,
    pub range: SourceRange,
}

pub enum GenericArgumentSyntax {
    // existing cases...
    Remainder(TypePackRemainderSyntax),
}
```

Reuse `AnonymousRemainderSyntax`; do not create a second homogeneous-tail model.

Only canonical structural Pack type formation should consume this syntax unless another already-ratified structural type explicitly supports it. Ordinary generic applications reject it.

Public `PositionalPack<Type>` / `LabeledPack<Type>` / `CompletePack<Type>` semantics bridge to existing C3 terms in type-level pack contexts by canonical declaration identity. C3 kinded binders remain C3 binders.

## 7.4 StaticPack representation

Add StaticPack to P2's store using immutable backing plus O(1) slice descriptors:

```rust
structural_id!(StaticPackBackingId);
structural_id!(StaticPackId);

pub struct StaticPackBacking {
    positionals: Box<[StaticValueRef]>,
    labels: Box<[StaticSymbolId]>,
    labeled_values: Box<[StaticValueRef]>,
}

pub struct StaticPackData {
    backing: StaticPackBackingId,
    positional_start: u32,
    positional_len: u32,
    labeled_start: u32,
    labeled_len: u32,
}

pub enum StaticValueData {
    // existing...
    Pack(StaticPackId),
}
```

Store-local backing/slice IDs are representation identities only and are never semantic content or durable metadata.

### Interning vs O(1) tails

Whole Pack finalization may content-intern a backing at O(n). Slice creation must remain O(1). A representation-keyed interner such as:

```text
(backing_id, positional range, labeled range) -> StaticPackId
```

is acceptable. Do not content-hash the entire visible suffix merely to create each rest slice.

`semantic_eq`, `semantic_hash`, and `content_fingerprint` traverse visible logical contents. Equal slices over different backing IDs compare/fingerprint equally.

### StaticPackBuilder

Add evaluator-private mutable staging:

```text
append positional
append labeled
append positional slice
append labeled slice
append complete slice
finalize immutable StaticPack
```

The builder is not a StaticValue and never escapes evaluation.

## 7.5 Value-domain variadic static parameters

Make parameter mode explicit in semantic signatures:

```rust
pub struct StaticParameter {
    pub name: String,
    pub domain: StaticValueDomain,
    pub mode: GenericParameterMode,
    pub index: u32,
    pub type_parameter_id: Option<TypeParameterId>,
    pub range: SourceRange,
}
```

Rules:

```text
Kind(...) + pack mode
    -> existing C3 TypeParameterId + pack-term binding

ValueType(T) + Single
    -> one StaticValue

ValueType(T) + PositionalPack
    -> one positional StaticPack; every value assignable to T

ValueType(T) + LabeledPack
    -> one labeled StaticPack; every value assignable to T

ValueType(T) + CompletePack
    -> one complete StaticPack; every value assignable to T
```

Zero capture produces `Pack()`.

Reuse/extract existing source-lane alignment rules. Do not fabricate TypeParameterId or a new argument-partition solver.

## 7.6 Runtime Pack representation

### Chosen ownership/lifetime pattern

Use a dedicated `Object::Pack` with root/slice representation. Root owns `ProductStorage`; every non-empty slice holds an `ObjRef` to the root. Slice-of-slice normalizes to the original root.

Conceptually:

```rust
pub enum PackStorage {
    Root {
        descriptor: RuntimeAnonymousProductDescriptorId,
        storage: ProductStorage,
    },
    Slice {
        root: ObjRef,
    },
}

pub struct PackObject {
    storage: PackStorage,
    positional_start: u32,
    positional_len: u32,
    labeled_start: u32,
    labeled_len: u32,
}
```

Exact private decomposition is flexible. The lifetime model is fixed.

### Product substrate

Add a distinct Pack kind/shape:

```rust
AnonymousProductKind::Pack
ProductShape::Pack(PackProductShape)
```

Pack shape may share layout helpers with Tuple but not semantic identity. Root physical component order is:

```text
[positionals...][labeled values...]
```

with ordered labels in the Pack shape.

### Slice semantics

A slice stores active positional/labeled ranges over the root. It does not own/copy ProductStorage.

GC:

```text
Pack root  -> trace ProductStorage Value slots
Pack slice -> trace root ObjRef only
```

A small live suffix may retain a large root backing. This is an accepted P3 memory/complexity tradeoff. Raw pointers into another heap object's storage are forbidden.

### Empty Pack

Preferred representation:

```rust
ValueTag::EmptyPack = next available tag
Value::empty_pack()
Value::is_empty_pack()
```

Map it to canonical Pack class. It is distinct from Unit/EmptyRecord.

If exhaustive tag/ABI audit exposes a material constraint not visible during planning, STOP AND CONSULT before switching to singleton-object representation.

### Exact runtime type

A root built from statically known construction may retain optional exact `RuntimeTypeRef` via product descriptor. A slice must not fabricate a descriptor whose active shape and root layout cardinalities disagree.

Verify runtime type-observation/reification paths before finalizing. If exact per-slice runtime structural type is required for sound dispatch/conformance, STOP AND CONSULT and design an explicit slice-type descriptor rather than abusing product metadata.

## 7.7 Builder convergence

Keep three representations conceptually distinct:

```text
language semantic argument structure -> Pack semantics
surface immutable value             -> Object::Pack / StaticValue::Pack
private mutable staging             -> ArgumentPackBuilderObject / StaticPackBuilder
```

Dynamic sends remain:

```text
NewArgumentPack
append contributions
InvokePack / SuperSendPack
```

No surface Pack is created solely for a dynamic call.

Expanded first-class Pack construction becomes:

```text
NewArgumentPack
append contributions
FinishPack
```

Expanded Tuple construction remains:

```text
NewArgumentPack
append contributions
FinishTuplePack
```

### Duplicate label scaling

Use a small-fast/large-indexed policy:

```rust
labels: Vec<Symbol>
label_index: Option<HashSet<Symbol>>
```

Linear scan below a small private threshold; populate index once when threshold is crossed. Threshold is mechanically flexible.

Label reservation/duplicate validation timing is fixed: validate before evaluating the corresponding labeled value exactly as current outgoing-pack tests require.

## 7.8 Rest capture convergence

Introduce one VM-owned canonical immutable Pack capture helper from already-evaluated lanes.

Conceptually:

```rust
fn capture_rest_pack(
    &mut self,
    positionals: &[Value],
    labeled: &[(Symbol, Value)],
) -> PhResult<Value>
```

or a range-based equivalent.

Every language rest-binding path uses the same finalizer:

```text
method Positional -> positional Pack
method Labeled    -> labeled Pack
method Split      -> two Packs
method Complete   -> one complete Pack
block positional rest -> positional Pack
Fiber positional rest -> positional Pack if same public rest contract
re-entrant/native language binding -> same normalized behavior
```

Do not broaden closure/block grammar beyond existing supported modes merely because runtime enums support richer modes.

## 7.9 Spreading and forwarding

Pack must not need `Iterator` conformance to support `*pack`.

Structural fast path directly reads visible lanes and appends to the current destination:

```text
Pack -> outgoing ArgumentPackBuilder
Pack -> dynamic Tuple builder
Pack -> List positional expansion target where existing `*` syntax applies
Pack -> first-class Pack builder
```

Existing generic Iterator fallback remains for arbitrary iterable operands. Tuple/Unit fast paths remain.

Labeled/complete expansion appends visible labels/values directly in order. Existing supported Map/Tuple expansion compatibility remains compatibility behavior and does not establish Pack identity/subtyping.

Multi-spread uses one mutable destination and is O(total elements).

## 7.10 First-class Pack construction lowering

`Pack(...)` binds to canonical intrinsic Pack declaration/constructor. Special representation lowering is selected by exact semantic identity.

Preferred fixed-shape path:

```text
values on stack in source order
BuildStaticPack { spec }
```

where the semantic lowering spec carries canonical shape and optional exact runtime type recipe.

Dynamic shape/expansion path:

```text
NewArgumentPack
compile dynamic contributions
FinishPack { optional exact-type/spec reference }
```

If an existing generic product-build opcode can produce identical Pack semantics/allocation behavior with less surface area, it may be reused. The fixed-call path remains unrelated.

## 7.11 Pack views

Canonical source view types:

```telos
intrinsic data PositionalPackView<T> {
    Empty
    Entry(_ value: T, _ rest: PositionalPack<T>)
}

intrinsic data LabeledPackView<T> {
    Empty
    Entry(_ label: Symbol, _ value: T, _ rest: LabeledPack<T>)
}

type PositionalTypePackView = PositionalPackView<Type>
type LabeledTypePackView = LabeledPackView<Type>
```

Views are one-step observations over flat/shared storage, never recursive physical storage.

Prefer a minimal native floor for lane length, first/at, label access, and O(1) rest slice. Define higher behavior in Telos source where practical. Do not require P5 reflection or P6 Iterator.

## 7.12 Stable metadata boundary

Proper Pack types can appear in ordinary exported signatures during P3. Add minimal stable logical nodes, conceptually:

```text
TypeNode::Pack(...)
TypeNode::PackFromPacks { positionals, labeled }
ScopedTypeNode equivalents
```

Encode logical element/remainder/type-pack structure, never session-local TypeId.

P3 does **not** serialize:

```text
StaticPackId
StaticPackBackingId
StaticValueStoreId
runtime ObjRef
runtime Pack root/backing ID
```

Durable StaticPack values remain P8 scope.

---
# 8. Ownership boundaries

## 8.1 P3 owns

- proper first-class Pack type formation/relation/normalization;
- anonymous typed Pack lane-remainder syntax;
- explicit bridge between public Pack-of-Type vocabulary and existing C3 pack terms;
- StaticValue Pack backing/slice representation;
- value-domain variadic static parameter capture;
- runtime Pack object, empty Pack, GC-safe slices, and minimal floor primitives;
- first-class Pack construction lowering;
- private builder Pack finalization and scalable duplicate detection;
- rest capture convergence across methods/blocks/Fibers;
- direct Pack spreading/forwarding;
- canonical one-step Pack views;
- minimal ordinary type metadata required to persist Pack types;
- fixed-call non-regression and Pack-specific performance/resource proof;
- P3 lifecycle records.

## 8.2 P3 does not own

- generalized Record `FieldKey`;
- Record becoming ordered or Pack-compatible by subtype;
- Product/Sum reflection protocols;
- public structural capability derivation;
- static `for`, static `match`, or static Iterator;
- general recursive static functions/residualization;
- durable StaticPack value metadata;
- full structural static-constant runtime materialization;
- replacing C3 TypePack internals;
- final P9 legacy public name removal;
- a new escape-analysis/scalar-replacement optimizer;
- expanding block/closure rest grammar beyond current supported forms.

## 8.3 Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| Pack declaration identity | semantic/Core Universe declaration identity | type formation, evaluator, compiler, runtime class binding | name/string checks |
| Pack proper type | TypeStore Pack nodes/formation | checker, inference, metadata, static value typing | compiler-only reconstruction |
| type-level pack algebra | existing C1–C3 TypePack forms | specialization, generic binders, bridge | StaticPack-as-type-array |
| static Pack value | P2 StaticValueStore extension | evaluator, fingerprints, future P8 | runtime `Value` |
| static parameter mode | StaticDefinition signature | checker, evaluator, fingerprints | deriving later from kind |
| runtime Pack storage | Object::Pack root/slice + product registries | VM/source methods | Tuple/List identity |
| mutable outgoing assembly | ArgumentPackBuilderObject | compiler bytecode/VM | public mutable Pack |
| rest construction | shared VM Pack capture helper | method/block/Fiber | per-path Tuple/List builders |
| fixed-call dispatch | current compiler invoke path | VM | forced builder/Pack path |
| Pack metadata | phalcom-type-meta logical nodes | import/export | raw session IDs |
| Pack views | source view declarations + O(1) slice primitive | runtime/static algorithms | physical cons list |

---

# 9. Global invariants

**P3-INV-01 — First-class value.** Pack is a first-class immutable structural value with positional and labeled lanes.

**P3-INV-02 — Distinct identity.** Pack is distinct from Tuple, Record, Unit, List, Map, and the private argument builder.

**P3-INV-03 — Empty identity.** `Pack()` is not Unit and not EmptyRecord.

**P3-INV-04 — Ordered lanes.** Positional order and labeled encounter order are semantic.

**P3-INV-05 — Symbol labels.** P3 Pack labels are Symbol; P4 FieldKey does not leak into Pack.

**P3-INV-06 — Unique labels.** Duplicate labels are rejected deterministically.

**P3-INV-07 — Canonical syntax.** `*`, `**`, `***` are lane syntax; `...` is not Pack syntax.

**P3-INV-08 — C3 continuity.** TypePack/LabeledTypePack/CompleteTypePack remain optimized canonical type-level representations.

**P3-INV-09 — Proper type separation.** First-class Pack type is `KindId::TYPE`; C3 TypePack terms remain kinded terms.

**P3-INV-10 — One bridge.** Pack-of-Type public semantics bridge to C3 terms through canonical APIs, not duplicate state/inference.

**P3-INV-11 — One StaticValue lineage.** StaticPack lives in P2's StaticValueStore.

**P3-INV-12 — Semantic-only static data.** StaticPack contains no runtime `Value`, `ObjRef`, VM Symbol, or ProductStorage pointer.

**P3-INV-13 — Logical fingerprints.** StaticPack equality/hash/fingerprint use visible logical contents, never backing/session identity.

**P3-INV-14 — O(1) static slice.** Static Pack rest/slice does not copy or content-hash the suffix at construction time.

**P3-INV-15 — O(1) runtime slice.** Runtime Pack rest/slice shares immutable root backing with O(1) structural work.

**P3-INV-16 — Flat backing.** Runtime/static Pack is not physically recursive `Cons` storage.

**P3-INV-17 — Fixed-call fast path.** Ordinary fixed calls require no surface Pack or argument-builder allocation.

**P3-INV-18 — Builder privacy.** `ArgumentPackBuilderObject` remains non-surface/unforgeable.

**P3-INV-19 — Fused multi-spread.** Multiple spreads append into one mutable destination and are O(total elements).

**P3-INV-20 — Rest means Pack.** Every language rest capture returns Pack(s), never Tuple/List.

**P3-INV-21 — Activation parity.** Equivalent calls have equivalent rest semantics across activation paths.

**P3-INV-22 — Explicit Tuple interoperability.** Tuple↔Pack movement is explicit construction/spread, not identity/subtyping.

**P3-INV-23 — Record isolation.** Record is not implicitly LabeledPack; P3 adds no ordered Record relation.

**P3-INV-24 — Value variadics bind one Pack.** Value-domain variadic static parameter receives one StaticPack containing captured lane(s).

**P3-INV-25 — Kinded variadics stay C3.** Kind-domain pack parameters retain TypeParameterId/C3 pack-term behavior.

**P3-INV-26 — Mode independent of domain.** Parameter mode is canonical independent of Kind vs ValueType.

**P3-INV-27 — One relation engine.** Pack assignability/subtyping uses existing relation budget/cancellation/outcomes; no Pack-only subtype engine.

**P3-INV-28 — Evaluation order.** Pack construction/spread preserves source evaluation order and current duplicate-label error timing.

**P3-INV-29 — Builder-only acceleration.** Duplicate-label hash index is private mutable staging, not immutable Pack semantics.

**P3-INV-30 — GC correctness.** Pack roots trace payload references; slices trace root; no raw borrowed pointer crosses GC.

**P3-INV-31 — Root normalization.** Slice-of-slice normalizes to original root rather than forming a view chain.

**P3-INV-32 — No P4 dependency.** Pack does not require heterogeneous FieldKey.

**P3-INV-33 — No P5 dependency.** Basic Pack does not require Product reflection/capability derivation.

**P3-INV-34 — No P6 Iterator dependency.** `*pack` uses structural fast paths; Pack need not implement Iterator for P3.

**P3-INV-35 — Static value durability deferred.** StaticPack value serialization remains P8; ordinary Pack type metadata is implemented in P3 as needed.

**P3-INV-36 — P9-compatible ontology.** Legacy public TypePack declarations may remain for compatibility, but new public design uses Pack vocabulary.

**P3-INV-37 — No descriptor fraud.** Pack slices do not use exact descriptors whose active shape disagrees with root layout.

**P3-INV-38 — Structural const boundary remains truthful.** StaticPack reduction does not imply full structural runtime materialization; unsupported use remains explicit deferred behavior.

**P3-INV-39 — No quadratic large-label path.** Unbounded large labeled assembly does not perform a full linear duplicate scan for every append.

**P3-INV-40 — One-step views.** `Entry(..., rest)` observes flat Pack storage; it does not define physical representation.

---

# 10. Non-goals and forbidden shortcuts

Do not implement:

```text
Pack as Tuple alias
Pack as List
Pack as recursive cons
Pack spread only through Iterator
public ArgumentPackBuilderObject
all calls through InvokePack
fixed calls materializing Pack
method rest Pack while block/Fiber remains Tuple/List
Record as ordered LabeledPack
C3 TypePack replaced by StaticPack
value variadic fabricated as TypeParameterId
Pack-specific subtype engine or inference-variable arena
independent P3 concat tree
`...` as Pack syntax
StaticPack backing IDs in metadata
raw pointers into GC-managed Pack backing
name-based Pack identity checks
broad structural static materialization added merely for StaticPack
```

---

# 11. Expected impact map

## 11.1 AST/parser

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast parser/unit tests
```

Purpose: typed Pack remainder contribution syntax using existing lane/remainder abstractions.

## 11.2 Semantic types

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/annotation.rs or current type-formation owner
phalcom-semantic/src/checker/inference.rs and exhaustive visitors as required
```

Purpose: proper Pack, PackFromPacks, relation, normalization, substitution, visitors.

## 11.3 Static semantics

```text
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/static_value/{mod.rs,store.rs}
phalcom-semantic/src/checker/{expression.rs,static_body.rs}
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/db/fingerprint.rs
```

Purpose: canonical mode, StaticPack, checked construction/application, evaluator support, fingerprints.

## 11.4 Type metadata

```text
phalcom-type-meta/src/{type_node.rs,scoped_type.rs,validate.rs}
phalcom-semantic/src/metadata/{export.rs,import.rs}
metadata schema/version owner if required
```

Purpose: stable ordinary Pack types and PackFromPacks; no StaticPack value persistence.

## 11.5 Universe/declaration identity

```text
phalcom-native-meta/src/universe.rs
canonical core declaration identity owner
phalcom-core/src/universe/core_classes.rs
phalcom-core/core/universe/src/collections/pack.ph   # expected new file
Universe module/catalog indexes as required
```

## 11.6 Runtime storage/product/value

```text
phalcom-core/src/heap/pack.rs                       # expected new file
phalcom-core/src/heap/{object.rs,mod.rs,accessors.rs,trace.rs}
phalcom-core/src/product/{shape.rs,anonymous.rs,view.rs,mod.rs}
phalcom-core/src/value/{repr.rs,mod.rs,render.rs}
```

## 11.7 Call/rest/compiler/VM

```text
phalcom-core/src/heap/pack_builder.rs
phalcom-core/src/parameters.rs
phalcom-core/src/method/mod.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/modules/semantic_lowering.rs or current executable construction-spec owner
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/{block.rs,fiber.rs}
```

## 11.8 Expected tests

Preferred organization:

```text
phalcom-ast focused parser tests
phalcom-semantic/tests/semantic/foundations/pack_types.rs
phalcom-semantic/tests/semantic/foundations/static_values_pack.rs
phalcom-semantic/tests/semantic/integration/static_pack_definitions.rs
phalcom-semantic/tests/semantic/incremental/pack_types.rs
phalcom-type-meta focused Pack round-trip tests
phalcom-core/tests/core/collections/pack.rs
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
phalcom-core/tests/core/calling/rest_pack.rs        # or nearest existing calling module
phalcom-core/tests/core/memory/pack.rs              # or nearest existing memory module
```

Exact test filenames may adapt to current module organization.

## 11.9 Expected documentation/state

```text
effective source language Pack/rest/type docs
docs/specs/vm/bytecodes.md when bytecodes change
LANG007.C4-CHECKPOINT.md
LANG007.C4.P3-walkthrough.md at completion
LANG007.C4.P3-handoff.md at completion
```

## 11.10 Unexpected-touch rule

Adjacent exhaustive-enum/generated metadata changes are allowed. STOP AND CONSULT before entering a materially different subsystem because Pack architecture/ownership would otherwise change.

---

# 12. Implementer decision authority

## 12.1 FIXED

The implementer must not change:

1. Pack/Tuple/Record identity separation.
2. proper Pack type vs C3 kinded pack-term separation.
3. C3 TypePack algebra as canonical optimized internal authority.
4. StaticPack ownership by P2 StaticValueStore.
5. direct StaticParameter mode retention for value-domain pack binders.
6. GC-safe root/slice ownership principle.
7. distinct empty Pack semantics.
8. private mutable builder vs immutable surface Pack separation.
9. dynamic calls consuming builder directly without surface Pack materialization.
10. fixed-call direct invoke path.
11. rest capture result semantics.
12. evaluation order and duplicate-label timing.
13. no Iterator requirement for Pack structural spread.
14. logical metadata for proper Pack types; no StaticPack value persistence.
15. no P4/P5/P6 dependency.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- private Rust helper/type names;
- whether root/slice are one enum or private pair of structs;
- small-label indexing threshold;
- generalized vs Pack-specific projection helper names;
- exact test filenames;
- use of an equivalent existing direct product-build opcode instead of `BuildStaticPack`;
- renaming/generalizing `PackTryExpandTuplePositionals` vs broadening its semantics in place;
- local safe data-movement strategy in rest capture;
- private native primitive names not ratified as public API.

## 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| Pack gets stable Universe/declaration identity without spelling checks. | native meta + semantic/core registration | STOP if identity cannot be carried canonically. |
| proper Pack nodes fit current metadata architecture. | type-meta + semantic import/export | extend minimally; STOP if raw IDs or StaticValue persistence would be required. |
| product descriptors can add Pack kind for roots. | product shape/anonymous registries | adapt mechanically; STOP if Tuple identity would be required. |
| exact per-slice runtime type is not required for sound dispatch/reification. | runtime type inspection paths | STOP if a new slice descriptor architecture is needed. |
| block surface has only its current rest modes. | AST/compiler/block primitive | do not broaden grammar without spec authority. |
| Fiber List capture represents language rest. | Fiber callers/tests | STOP if it is an intentionally different public API contract. |
| P2 structural static constants remain runtime-deferred. | materialization/compiler tests | preserve deferred boundary. |
| label reservation-before-value timing remains normative. | outgoing-pack tests/effective spec | preserve; consult on conflict. |

---

# 13. Global STOP / CONSULT triggers

Stop editing the affected issue and prepare a consultation packet if any repository-wide trigger fires, especially if:

1. first-class Pack appears to require replacing C3 TypePack terms;
2. proper Pack typing needs a new general pack-partition solver;
3. value-domain variadics need fabricated TypeParameterId or altered P2 domain rules;
4. O(1) runtime slices cannot be GC-safe without raw pointers or a new GC model;
5. fixed calls would need builder/Pack machinery merely because P3 exists;
6. `Object::Pack` cannot share product storage without pretending to be Tuple;
7. exact runtime typing requires a materially new slice-descriptor/reification architecture;
8. Pack metadata would serialize process-local identities;
9. Record must become ordered/LabeledPack to satisfy Pack work;
10. an optimization changes expression order or duplicate-error timing;
11. Pack requires exposing mutable builder state;
12. static Pack evaluation requires arbitrary runtime/native dispatch;
13. basic Pack semantics require P5 reflection or P6 Iterator/static-for;
14. compatibility failures show `Single(TypePack)` and variadic TypePack semantics collapsed;
15. the same semantic Pack failure persists after one serious architecture-consistent correction;
16. GC stress indicates a lifetime flaw;
17. the live repository materially changed one canonical ownership seam after the audited revision.

A triggered consultation cannot be self-waived.

---

# 14. Debugging budget

## Mechanical failures

Allow up to three coherent cycles:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminator
```

## Semantic failures

Before a nontrivial corrective edit, record:

```text
Observed:
Expected invariant:
Causal hypothesis:
Evidence:
Predicted effect:
Smallest discriminating test:
```

Allow one serious correction for the same underlying semantic issue. If it persists, STOP AND CONSULT.

## GC/lifetime/type-identity failures

Treat as architectural unless the cause is an obvious exhaustive-match/mechanical omission. Do not trial multiple representation redesigns.

Never rerun an unchanged failing test when nothing relevant changed.

---

# 15. Testing surface analysis

| Coverage | Dimension | Invariants | Required evidence |
|---|---|---|---|
| CV-01 | value identity | 01–03 | `Pack()` distinct from Unit/Tuple/Record; nonempty Pack has Pack class. |
| CV-02 | exact type shape | 04–06,09 | element/lane/label order preserved. |
| CV-03 | open typed families | 07,09 | `Pack<*:T>`, `Pack<**:T>`, `Pack<***:T>`. |
| CV-04 | C3 bridge | 08–10 | symbolic/concrete PackFromPacks normalization; C3 continuity. |
| CV-05 | metadata | 09,35 | logical Pack type round-trip/fingerprint stability. |
| CV-06 | StaticValue lineage | 11–12 | foreign children rejected; clone/append preserves refs. |
| CV-07 | static O(1) slice | 14,16 | rest chain shares backing and does not copy/hash suffix. |
| CV-08 | static eq/hash/fp | 13 | equal logical slices over distinct backing compare/fingerprint equal. |
| CV-09 | value static variadic | 24,26 | positional/labeled/complete + zero capture. |
| CV-10 | kinded compatibility | 08,25 | existing C3 pack binders unchanged. |
| CV-11 | runtime empty Pack | 02–03 | EmptyPack class/render/value identity. |
| CV-12 | runtime GC/rooting | 15,30–31 | root/deep slices survive GC; slice normalizes to root. |
| CV-13 | runtime structural eq/hash | 01,04–06 | content equal; lane/order/label differences unequal. |
| CV-14 | builder privacy/finalize | 18 | FinishPack yields Pack; FinishTuplePack remains Tuple. |
| CV-15 | duplicate behavior | 06,28–29,39 | timing preserved; large labels use index. |
| CV-16 | method rest modes | 20–21 | positional/labeled/split/complete Pack captures. |
| CV-17 | block rest | 20–21 | current block rest returns Pack. |
| CV-18 | Fiber rest | 20–21 | Fiber parity after contract verification. |
| CV-19 | forwarding | 19–20 | Pack→call direct lane append; no Tuple/List bridge. |
| CV-20 | Tuple interop | 22 | explicit Tuple→Pack and Pack→Tuple only. |
| CV-21 | fused multi-spread | 19 | one builder/finalizer or invoke, O(total). |
| CV-22 | evaluation/error order | 28 | source order and label validation timing. |
| CV-23 | fixed-call codegen | 17 | no Pack builder/finalizer/allocation machinery. |
| CV-24 | one-step views | 14–16,40 | repeated rest changes ranges/root only. |
| CV-25 | Record isolation | 23,32 | no Pack relation/equality with Record. |
| CV-26 | incrementality | 13,26,35 | exact invalidation and warm/cold parity. |
| CV-27 | P2 compatibility | 11,38 | existing static values/const behavior unchanged. |
| CV-28 | C1–C3 compatibility | 08,25,36 | type-pack/generic/ADT suites stable. |
| CV-29 | dependency boundary | 32–34 | no P4/P5/P6 dependency. |
| CV-30 | budget/cancel | 27 | large static Pack work uses existing budget/cancel. |
| CV-31 | runtime type observation | 37 | root/slice remains sound under runtime type inspection. |
| CV-32 | structural const deferral | 35,38 | unsupported runtime StaticPack const remains explicit deferred behavior. |

---

# 16. Verification execution budget

## BUILD mode

During T1–T7 run only:

- newly changed focused test module(s);
- package `cargo check` when a cross-crate public enum/API boundary is coherent enough to be informative;
- no workspace tests;
- no full language corpus;
- no full semantic/core package test reflexively.

Use serial Cargo commands and clear local wrappers where repository guidance requires:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo ...
```

## STABILIZE mode

Enter only at G1–G4. Run named focused suites and directly affected package checks.

## CERTIFY mode

Not required by P3 unless separately requested.

## Explicitly deferred during BUILD

```text
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
full phalcom-semantic package test
full phalcom-core package test
full language-corpus
full LSP/REPL suite
```

The language corpus is specifically excluded as an edit-loop gate because its existing bootstrap cost and deferred baseline state are already recorded in the checkpoint.

---

# 17. Baseline / unrelated failure policy

Classify:

```text
A — definitely caused by P3
B — probably caused by P3
C — unclear after one bounded classification pass
D — pre-existing/unrelated
```

Known inherited baseline entries are already recorded in the checkpoint. Do not re-investigate them merely because P3 touches adjacent crates.

- A/B: active responsibility.
- C: one bounded comparison against audited revision/clean predecessor; record and defer if nonblocking.
- D: record and continue immediately.
- Never weaken assertions or skip behavior merely to make a broad gate green.

---
# 18. Tasks

# T0 — Takeover, active-plan transition, and baseline discriminators

## Purpose

Establish a clean P3 implementation start without rediscovering P2 or consuming test budget on broad suites.

## Preconditions

- live branch contains P2 closure at or after `5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521`;
- checkpoint still records P2 complete and P3 as next/active candidate;
- P2 walkthrough/handoff exist;
- no unreviewed concurrent edits in P3-owned paths.

## Actions

1. Read repository workflow, this plan, checkpoint, and P2 completion records.
2. Run:

```sh
git status --short
git rev-parse HEAD
git log -5 --oneline --decorate
```

3. Preserve unrelated dirty state.
4. Re-census exact symbols named in §3.1 with `rg`; avoid broad unrelated scans.
5. Confirm fixed-call no-builder discriminator still exists and selects a real test.
6. When source implementation actually begins, change P3 checkpoint row to `IN_PROGRESS / PARTIAL / UNVERIFIED` and record starting revision.

## Focused baseline evidence

List exact core test names if necessary:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core -- --list | rg 'outgoing_packs_completion|static_calls_and_static_tuple_emit_no_pack_machinery'
```

Run only the existing fixed-call discriminator.

Do **not** rerun P2 broad gates merely to reconfirm closed work.

## Acceptance

- lifecycle state is consistent;
- no unexpected P3 implementation already exists;
- fixed-call discriminator is runnable/green or failure classified;
- live ownership seams still match this plan.

## STOP / CONSULT

Stop if post-P2 architectural drift materially changes Pack/type/call/rest ownership or if checkpoint no longer treats P2 as complete.

## Checkpoint update

Yes: record actual P3 start, start revision, and newly discovered baseline facts only.

---

# T1 — Canonical Pack type, typed remainder syntax, declaration identity, and stable metadata

## Purpose

Create the proper semantic Pack type and its stable source/metadata identity before runtime representation work.

## Consumes

- AST `ExpansionMode`, `AnonymousRemainderSyntax`, generic argument syntax;
- C3 type-pack algebra and `GenericParameterMode`;
- current Tuple/open-Tuple relation and metadata patterns.

## Produces

- canonical Pack declaration identity;
- proper Pack TypeStore representation;
- `PackFromPacks` symbolic bridge;
- typed lane-remainder syntax;
- Pack type metadata/scoped-type support;
- deterministic formatting/fingerprinting/substitution/relation behavior.

## Read first

```text
phalcom-ast/src/{ast,parser}.rs
phalcom-semantic/src/types/{store,relation,substitution,instantiation,type_lambda,annotation,parameter}.rs
phalcom-semantic/src/metadata/{export,import}.rs
phalcom-type-meta/src/{type_node,scoped_type,validate}.rs
phalcom-native-meta/src/universe.rs
canonical core declaration identity owner
```

## T1.1 — AST/parser remainder contribution

Add one explicit type-application remainder contribution using existing `ExpansionMode` + `AnonymousRemainderSyntax`.

Parse at minimum:

```telos
Pack<*: T>
Pack<**: T>
Pack<***: T>
```

Continue parsing `Pack<*P>`, `Pack<**P>`, `Pack<***P>` as expansion of existing pack terms rather than typed anonymous remainder.

Do not repurpose expression ellipsis or `...`.

Ordinary generic applications that do not support structural Pack remainder syntax must diagnose the remainder instead of silently treating it as a normal argument.

## T1.2 — proper Pack nodes and formation

Implement §7.2.

Canonical formation behavior:

```text
concrete TypePack + concrete LabeledTypePack
    -> canonical concrete Pack type

symbolic lane term(s)
    -> PackFromPacks

specialization/substitution makes lanes concrete
    -> same concrete Pack type as direct formation
```

Concrete Pack type preserves positional order, labeled order, labels, and typed/open lane remainders. Empty Pack remains Pack.

## T1.3 — relation/visitor integration

Audit every exhaustive TypeData path. Minimum:

```text
kind lookup
interning/content fingerprint/formatting
free-parameter visitation
substitution
instantiation
scoping/type-lambda conversion
inference traversal/occurs checks
subtyping/assignability/equivalence
lane projection
metadata export/import
source-index/diagnostic formatting if exhaustive
```

Extract/reuse ordered-product relation mechanics where profitable. Do not convert Pack to Tuple as semantic implementation.

## T1.4 — canonical declaration identity

Add canonical Pack Universe/declaration key needed by semantic formation and later compiler/runtime recognition. Source declaration may be minimal in T1; runtime behavior is T3/T7.

No string/name based Pack detection.

## T1.5 — stable metadata

Add logical TypeNode/scoped-type forms for concrete/open Pack and PackFromPacks, mirroring open Tuple/C3 pack metadata. Update structural fingerprint tags and validators. Bump schema/version if current metadata rules require it.

Round-trip must preserve typed remainders and symbolic lane terms.

## Forbidden approaches

- Pack as Tuple alias;
- proper Pack as `KindId::TYPE_PACK`;
- new pack-variable arena;
- raw TypeId metadata;
- string-based Pack special casing;
- C3 TypePack deletion/rename.

## Required tests

Coverage: CV-02–CV-05, CV-10, CV-25.

Cases:

```text
parse all three typed remainder modes
`...` remains unrelated ellipsis
closed exact Pack type
empty Pack type != Unit
labeled order affects Pack type
Pack != Tuple under ordinary relation
homogeneous positional/labeled/complete Pack family relation
symbolic PackFromPacks remains symbolic
concrete substitution normalizes to direct Pack type
C3 TupleOf<*P> behavior unchanged
metadata export/import round-trip
```

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast pack_type_remainder -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::pack_types -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta pack -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic -p phalcom-type-meta
```

Adapt filters to actual module names and confirm nonzero selection.

## Acceptance

- Pack is a stable proper semantic Type;
- typed anonymous lane families are expressible;
- symbolic C3 bridge exists without duplicate authority;
- metadata round-trips Pack types;
- existing C3 kinded pack semantics remain unchanged.

## Local STOP / CONSULT

Stop if open Pack families require a new general pack solver or if metadata needs raw session identity.

## Checkpoint update

Yes when proper Pack type/declaration/metadata interface is stable and focused-tested.

---

# T2 — StaticPack storage, static construction, and value-domain variadic static parameters

## Purpose

Extend P2 compile-time semantics so Pack is a first-class static value and value-domain variadic binders receive one Pack value.

## Preconditions

T1 proper Pack type and identity are stable.

## Consumes

- P2 StaticValueStore lineage and structural-value APIs;
- P2 checked static body/application products;
- P2 explicit-frame evaluator, QueryBudget, cancellation;
- T1 Pack type/bridge.

## Produces

- StaticPack backing/slice nodes;
- semantic equality/hash/fingerprint/type projection;
- `StaticParameter.mode`;
- checked value-domain variadic capture;
- evaluator-private StaticPackBuilder;
- static Pack construction/spread support.

## Files

```text
phalcom-semantic/src/static_value/{mod,store}.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/checker/{static_body,expression}.rs or live checked-application owner
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/db/fingerprint.rs
signature metadata paths if StaticParameter crosses modules
focused semantic tests
```

## T2.1 — store-local backing/slice

Whole Pack finalization:

1. reject children from foreign store lineage;
2. validate label spelling/Symbol identity and duplicate labels;
3. allocate or content-intern immutable backing once;
4. create full-range StaticPackData;
5. publish `StaticValueData::Pack`.

Slice/rest:

1. validate ranges;
2. compose ranges against same backing;
3. optionally intern `(backing,ranges)` in O(1);
4. do not scan/copy logical suffix merely to create slice.

## T2.2 — semantic equality/hash/fingerprint

Fingerprint logical visible content:

```text
Pack tag
visible positional length
visible labeled length
positional child fingerprints in order
label spelling + labeled child fingerprint in order
```

Ignore backing ID, slice ID, insertion order, source range, VM symbol ID.

## T2.3 — static value type

For visible Pack contents, derive child proper types, form C3 positional/labeled TypePack terms, then use T1 canonical Pack formation. Empty StaticPack types as empty Pack, not Unit.

## T2.4 — parameter mode

Add direct `GenericParameterMode` to semantic StaticParameter/signature and every relevant fingerprint/metadata product.

Kind-domain parameters keep existing TypeParameterId allocation exactly.

## T2.5 — value-domain variadic application

Reuse/extract source-lane alignment. Validate each captured StaticValue through canonical assignability relation against `ValueType(T)`. Bind one StaticPack to the formal evaluator slot.

Zero capture binds `Pack()`.

No TypeParameterId is fabricated for ordinary value pack capture.

## T2.6 — static Pack construction

Ordinary checker publishes exact canonical Pack construction identity plus contribution facts. Evaluator does no name lookup.

Evaluation path:

```text
source-ordered checked contributions
  -> StaticPackBuilder
  -> append direct positional/labeled values
  -> append existing StaticPack lanes directly for expansion
  -> explicit checked Tuple interoperability where supported
  -> duplicate detection
  -> immutable StaticPack
```

Charge existing QueryBudget proportional to visited/appended elements and check cancellation during large expansions/equality/fingerprint work.

## T2.7 — structural const boundary

`const P = Pack(...)` reduces semantically to StaticPack. P3 does not thereby authorize full runtime structural const materialization. Preserve explicit structural-deferred runtime/compiler behavior until its owning plan.

## Forbidden approaches

- runtime Value/ObjRef inside StaticPack;
- second StaticValueStore;
- O(n) content hash on every rest slice creation;
- static Iterator dependency;
- fabricated TypeParameterId for `*P: Int`;
- evaluator name lookup;
- broad structural materializer work.

## Required tests

Coverage CV-06–CV-10, CV-26, CV-30, CV-32.

Cases:

```text
whole Pack + deep rest chain share backing
foreign child rejected
store clone/append preserves existing Pack refs
representation-distinct equal slices semantic_eq/hash/fingerprint equal
empty StaticPack type is Pack
value-domain positional/labeled/complete variadic capture
zero capture -> Pack()
element mismatch uses ordinary relation diagnostic
kinded variadic static definition unchanged
large Pack budget exhaustion/cancellation is bounded
closed const reduces to StaticPack
runtime use of unsupported StaticPack const remains explicit deferred behavior
```

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_pack -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_pack_definitions -- --nocapture
```

Also run the smallest existing P2 test that exercises kinded pack parameters after signature mode changes.

## Acceptance

Static Pack semantics are complete without VM dependency; value-domain variadics are soundly represented as one Pack.

## Local STOP / CONSULT

Stop if value-domain alignment needs a new ambiguous solver or if store slicing cannot remain O(1) under P2 lineage.

## Checkpoint update

Yes when StaticPack + mode interfaces are stable and focused-tested.

---

# T3 — Runtime Pack root/slice object, EmptyPack, product shape, GC, and floor primitives

## Purpose

Introduce immutable runtime Pack representation with O(1) slices and correct GC lifetime before compiler/rest paths produce it broadly.

## Preconditions

T1 canonical Pack identity exists.

## Consumes

- product shape/layout/storage registries;
- anonymous product descriptor pattern;
- Value immediate-tag pattern;
- canonical Pack class identity.

## Produces

- `Object::Pack` root/slice;
- distinct Pack product kind/shape;
- `Value::empty_pack()`;
- `VM::pack_view_value` or equivalent;
- Pack root finalizer + O(1) slice constructor;
- exhaustive GC/value/class/render integration;
- floor APIs for later tasks.

## Files

```text
phalcom-core/src/heap/pack.rs                        # expected new
phalcom-core/src/heap/{object,mod,accessors,trace}.rs
phalcom-core/src/product/{shape,anonymous,view,mod}.rs
phalcom-core/src/value/{repr,mod,render}.rs
phalcom-core/src/universe/core_classes.rs
phalcom-native-meta/src/universe.rs if not complete in T1
focused core product/memory tests
```

## T3.1 — EmptyPack

Add immediate tag plus exhaustive branches:

```text
constructor/predicate
class mapping -> pack_class
type_name/render/debug
immediate equality/hash branch where Value requires it
call-context classification if relevant
any byte-level serialization only if current Value contract requires it
```

Do not alias Unit/EmptyRecord.

## T3.2 — Pack root

Root contains distinct Pack descriptor + ProductStorage. Shape records positional length + ordered labels. Duplicate labels rejected before publication.

`finish_pack(vm, positionals, labeled, exact_type?)` returns EmptyPack when both lanes empty; otherwise one Pack root.

## T3.3 — Pack slice

Slice contains root ObjRef + active lane ranges. Slice-of-slice resolves to original root and composes ranges immediately. Zero-visible-lane result normalizes to EmptyPack.

## T3.4 — PackView

Registry-aware view exposes O(1):

```text
visible positional length
visible labeled length
positional get(i)
label get(i)
labeled value get(i)
internal root/range info for O(1) slice
```

Avoid eager Vec allocation for simple length/first/rest.

## T3.5 — GC

Root traces ProductStorage Value slots using layout; slice traces root. Add GC stress where an object is reachable only through a Pack slice.

## T3.6 — exact-type audit

Inspect all consumers of `RuntimeAnonymousProductDescriptor.exact_type`. If exact specialized slice type is required for sound runtime dispatch/conformance, STOP AND CONSULT before inventing a shape/layout-inconsistent descriptor.

## Forbidden approaches

- raw pointer into another heap object;
- TupleObject alias;
- recursive linked Pack;
- suffix Vec copy on rest;
- permanent hash map on immutable Pack;
- Iterator conformance as spread prerequisite.

## Required tests

Coverage CV-01, CV-11–CV-13, CV-24, CV-31.

Cases:

```text
EmptyPack immediate identity/class
root preserves lane order/labels
slice shares root and adjusts ranges
slice-of-slice points directly to root
positional-only/labeled-only/complete slices
GC retains child through root
GC retains root through slice
hidden prefix not visible
```

Use `kernel_vm()`/`native_vm()` where source Universe methods are unnecessary.

## Tests to run now

Use exact focused module names created in core; conceptually:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::pack::runtime_storage -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory::pack -- --nocapture
```

## Acceptance

Runtime Pack is immutable, distinct, O(1)-sliceable, and GC-correct independently of call/rest compiler work.

## Local STOP / CONSULT

Any exact-runtime-type or GC ownership uncertainty is an immediate stop.

## Checkpoint update

Yes after root/slice/EmptyPack lifetime invariants are focused-tested.

---

# T4 — First-class Pack construction, builder finalization, scalable duplicates, and bytecode lowering

## Purpose

Make ordinary runtime `Pack(...)` construction efficient while preserving dynamic-call builder architecture and fixed-call fast path.

## Preconditions

T3 runtime finalizer exists; T1 semantic Pack identity/lowering facts exist.

## Consumes

- `ArgumentPackBuilderObject`;
- compiler dynamic pack contribution lowering;
- executable semantic construction specs;
- static Tuple/Record product-build precedent.

## Produces

- builder `FinishPack` path;
- direct known-shape Pack build path;
- thresholded duplicate-label index;
- semantic-identity-selected Pack lowering;
- unchanged `InvokePack` and `FinishTuplePack` behavior.

## T4.1 — scalable label membership

Extend builder with optional thresholded `HashSet<Symbol>` or equivalent. `reserve_label`, direct labeled append, and bulk Pack labeled append use one canonical membership check.

Reservation remains before corresponding value evaluation.

`take_parts`/reset leaves builder state coherent.

## T4.2 — immutable builder finalization

Add bytecode conceptually:

```text
FinishPack [optional exact-type/construction spec]
```

Stack effect:

```text
[..., builder] -> [..., immutable Pack]
```

Pending label is an internal error. Move parts out rather than clone them merely for finalization.

## T4.3 — direct known-shape build

Add `BuildStaticPack { spec }` or equivalent direct product build. Values are evaluated in source order and exactly one Pack root (or EmptyPack) is created without `ArgumentPackBuilderObject`.

## T4.4 — canonical lowering identity

Compiler receives checked semantic lowering fact proving canonical Pack constructor identity. No `Pack` spelling detection.

## T4.5 — preserve existing lanes

Dynamic sends remain:

```text
builder -> InvokePack / SuperSendPack
```

Expanded Tuple remains:

```text
builder -> FinishTuplePack -> Tuple
```

## Forbidden approaches

- surface Pack before InvokePack;
- replacing FinishTuplePack globally;
- universal builder lowering;
- pre-evaluation scan of dynamic labels;
- always-on HashSet for tiny builders if avoidable.

## Required tests

Coverage CV-14–CV-15, CV-21–CV-23.

Extend `outgoing_packs_completion.rs` with:

```text
fixed ordinary call -> no builder/FinishPack/BuildStaticPack
fixed Pack construction -> direct Pack build, no builder
expanded Pack construction -> NewArgumentPack ... FinishPack
expanded Tuple -> FinishTuplePack, not FinishPack
multi-spread Pack construction -> one builder + one FinishPack
```

Keep existing evaluation-order/label-reservation tests green.

## Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::outgoing_packs_completion -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core pack_builder -- --nocapture
```

## Acceptance

First-class Pack construction works; dynamic calls still consume builder directly; fixed calls are untouched; large labeled assembly no longer has unbounded quadratic duplicate scanning.

## Local STOP / CONSULT

Stop if canonical Pack construction cannot be identified semantically or if fixed calls begin touching builder/Pack machinery.

## Checkpoint update

Record durable builder/finalizer architecture when coherent.

---
# T5 — Rest capture convergence across method, block, Fiber, and re-entrant entry

## Purpose

Remove the current Tuple/List rest semantic split and establish one Pack result rule for every language rest-binding entry path.

## Preconditions

T3 Pack finalizer exists. T4 may still be stabilizing, but runtime Pack creation must be stable.

## Produces

- one shared VM rest-Pack capture primitive;
- method RestMode parity;
- block/Fiber parity;
- removal of language-rest Tuple/List materialization.

## T5.1 — central capture helper

Factor canonical helper(s) that accept already-evaluated positional/labeled slices/ranges and return immutable Pack.

Prefer direct copy into final Pack backing over Tuple/List intermediate materialization. A single owned staging buffer required for safe call-frame rearrangement is acceptable; an intermediate guest-visible product is not.

## T5.2 — method rest

Rewrite `VM::call_rest_method_as_with_environments` or live equivalent:

```text
Positional:
  fixed positionals
  Pack(extra positionals)
  fixed labeled arguments

Labeled:
  fixed positionals
  fixed labeled arguments
  Pack(extra labeled arguments)

Split:
  fixed positionals
  Pack(extra positionals)
  fixed labeled arguments
  Pack(extra labeled arguments)

Complete:
  fixed positionals
  fixed labeled arguments
  Pack(extra positionals + extra labeled arguments)
```

Preserve compiled parameter-slot order exactly.

## T5.3 — block rest

Replace `finish_tuple` rest capture with same positional Pack helper. Do not broaden block grammar beyond current supported forms.

## T5.4 — Fiber rest

Verify whether Fiber excess-argument List is the same language-rest contract. If yes, replace with Pack. If it is an intentionally different public Fiber API contract, STOP AND CONSULT rather than silently changing it.

## T5.5 — re-entrant/native census

Search all uses of:

```text
RestMode
RestKind
rest_slot
finish_tuple in call binding
alloc_list in activation
```

Any language-rest binding converges to Pack. Native APIs receiving raw argument slices without binding a language rest parameter may remain unchanged.

## Forbidden approaches

- path-specific Pack/Tuple compatibility shims;
- method Pack + Fiber List split;
- new selector/rest parser;
- duplicate argument-shape authority.

## Required tests

Coverage CV-16–CV-18, CV-20–CV-21.

Cases:

```text
zero/nonzero positional rest
zero/nonzero labeled rest
split rest with both lanes
complete rest with both lanes
labeled encounter order preserved
static and dynamically spread calls capture equivalent Pack
block rest returns Pack
Fiber rest parity after contract verification
captured rest Pack forwards into another call
```

## Tests to run now

Create/use one focused core calling module and run only it. Also run existing `parameters.rs` rest-shape unit tests if touched.

## Acceptance

A repository census finds no language rest capture constructing Tuple/List. Equivalent rest calls yield equivalent Pack semantics across activation paths.

## Local STOP / CONSULT

Stop on Fiber contract ambiguity or if parameter-slot order cannot be preserved without changing method ABI.

## Checkpoint update

Yes after activation parity is focused-tested.

---

# T6 — Direct Pack spreading, forwarding, Tuple interoperability, and fused multi-spread

## Purpose

Make Pack the structural transport substrate for spread/forwarding without requiring Iterator and without intermediate Tuple/List products.

## Preconditions

T3 runtime Pack view/finalizer and T4 builder are stable; T5 rest returns Pack.

## Produces

- direct positional/labeled/complete Pack append into builders;
- generalized positional expansion fast path for Pack plus existing Tuple/Unit;
- Pack→call and explicit Pack↔Tuple interoperability;
- fused multi-spread behavior.

## T6.1 — positional structural fast path

Generalize current positional probe (`PackTryExpandTuplePositionals` or equivalent) so expansion recognizes:

```text
EmptyPack -> zero contributions
Pack      -> visible positional lane
Unit      -> existing empty-Tuple compatibility
Tuple     -> existing positional fast path
other     -> existing checked/Iterator fallback where source syntax permits
```

If List literals already share the positional expansion target, `*pack` there must also use structural fast path rather than Iterator.

## T6.2 — labeled/complete structural fast path

`PackExpandLabels` / `PackExpandComplete` recognize first-class Pack and append visible lane(s) directly. Destination duplicate checks apply as entries are admitted.

Preserve existing explicitly supported Map/Tuple compatibility behavior. Do not infer Pack subtype/equality from spread compatibility.

## T6.3 — forwarding

For:

```telos
relay(***args) {
    target(***args)
}
```

physical path should be:

```text
rest Pack
  -> visible lane reads
  -> one target ArgumentPackBuilder
  -> InvokePack
```

No Pack→Tuple/List→builder bridge.

## T6.4 — Tuple interoperability

Support explicit forms using final source syntax, conceptually:

```telos
let p = Pack(*(1, 2, 3))
let t = (*p)
```

Tuple and Pack remain distinct types/values.

## T6.5 — multi-spread fusion

`target(*a, *b, *c)` and `Pack(*a, *b, *c)` use one mutable destination. Never implement repeated immutable `Pack + segment` copying.

## Forbidden approaches

- Pack Iterator requirement;
- temporary Tuple/List conversion;
- repeated immutable Pack concatenation;
- Record/LabeledPack subtyping;
- pre-evaluating/reordering contributions to optimize fusion.

## Required tests

Coverage CV-19–CV-23, CV-25.

Add/extend `outgoing_packs_completion` cases:

```text
Pack positional spread
Pack labeled spread
Pack complete spread
rest Pack forwarding
explicit Pack/Tuple round trip
multi-spread emits one NewArgumentPack and one InvokePack/FinishPack
earlier spread exhaustion precedes later effects exactly as current semantics require
duplicate label from earlier Pack prevents later labeled value evaluation at correct point
non-Pack/non-iterable failures remain structured
```

## Tests to run now

Run focused outgoing-packs and new Pack integration modules only.

## Acceptance

Pack spreading/forwarding is direct and lane-preserving; no P6 Iterator work was required.

## Local STOP / CONSULT

Stop if compiler can only spread Pack by invoking user Iterator methods or if Tuple/Record semantic identity would need weakening.

## Checkpoint update

Record direct spread/fused-builder invariant after focused pass.

---

# T7 — Canonical Pack source surface and one-step O(1) views

## Purpose

Publish selected Pack vocabulary/views over correct semantic/runtime backing while keeping native code at structural floor.

## Preconditions

T1 type surface and T3/T6 runtime Pack operations are stable.

## Produces

- canonical `Pack`, `PositionalPack`, `LabeledPack`, `CompletePack` source declarations/aliases;
- `PositionalPackView` / `LabeledPackView`;
- `PositionalTypePackView` / `LabeledTypePackView`;
- minimal native raw lane/first/rest primitives;
- source-defined equality/hash/convenience behavior where practical.

## T7.1 — source module

Add `collections/pack.ph` or canonical structural module chosen by live Universe layout; integrate bootstrap/import catalogs.

Pack must not remain Rust-only class metadata.

## T7.2 — one-step views

Expose canonical variants:

```text
PositionalPackView::Empty
PositionalPackView::Entry(value, rest)
LabeledPackView::Empty
LabeledPackView::Entry(label, value, rest)
```

`rest` is O(1) Pack slice.

## T7.3 — minimal native floor

Prefer floor operations equivalent to:

```text
raw positional length
raw labeled length
raw positional first/at
raw labeled label/value first/at
raw positional rest slice
raw labeled rest slice
```

Source methods construct public view variants. Follow current native registration conventions.

## T7.4 — equality/hash/render

Language `==` and hash use visible logical lane contents and recursive Telos value semantics. Rust object identity is not Pack language equality.

Lane/order/label differences affect equality/hash.

## T7.5 — no premature reflection/Iterator

Do not implement Product/OrderedProduct capability or static/runtime Iterator solely for Pack. P5/P6 own those abstractions.

## Forbidden approaches

- recursive physical view chain as Pack storage;
- stale competing view variant `Item` instead of `Entry`;
- competing generic `TypePackView` replacing selected lane-specific aliases;
- large native convenience API that can be ordinary Telos source.

## Required tests

Coverage CV-13, CV-24, CV-29.

Cases:

```text
empty views
one/many positional entries
one/many labeled entries preserving labels/order
repeated rest shares root/backing
structural equality/hash for root and slices
source module bootstrap/import identity
no Iterator/capability dependency
```

Use `universe_vm()` only where source-installed methods are required; use lower bootstrap for storage tests.

## Acceptance

Public source vocabulary is canonical and one-step decomposition remains O(1) over flat/shared storage.

## Local STOP / CONSULT

Stop if canonical source views require P5 structural reflection or P6 static control flow merely to function; implement only minimum P3 floor.

## Checkpoint update

Yes when source surface/view invariant is established.

---

# T8 — Incrementality, metadata closure, compatibility stabilization, and performance evidence

## Purpose

Close cross-layer visitor/fingerprint/metadata gaps, prove warm/cold behavior, and ensure P3 does not regress C1–C3/P2 or fixed-call performance.

## Preconditions

T1–T7 functional slices complete.

## T8.1 — exhaustive enum/visitor audit

Search every exhaustive match for:

```text
TypeData
ScopedTypeData
TypeNode
ScopedTypeNode
StaticValueData
Object
ValueTag
ProductShape / AnonymousProductKind
```

Missing Pack arms are correctness defects.

## T8.2 — incremental semantics

Add edit tests proving:

```text
Pack element type edit invalidates exact dependent product
labeled label/order edit invalidates exact dependent product
StaticParameter mode edit invalidates definition/application
unrelated declaration remains reused
cold/warm Pack type/static results agree
```

## T8.3 — metadata round-trip

Exercise exported Pack signatures through semantic metadata import/export. Structural fingerprints remain stable and no runtime/static backing IDs appear.

## T8.4 — compatibility

Run focused C1–C3 TypePack/generic/ADT and P2 static suites named in G4. Do not broaden to full slow language corpus.

## T8.5 — performance/resource proof

Acquire structural evidence:

```text
fixed call: no builder/Pack bytecodes
fixed no-rest call: no Pack allocation path
multi-spread: one destination builder
runtime Pack rest chain: same root backing
StaticPack rest chain: same static backing
large labeled builder: thresholded index path active
```

Prefer bytecode inspection, backing-identity tests, explicit allocation/path counters already available, and algorithmic proof. Avoid flaky wall-clock assertions.

## T8.6 — specs/documentation

Update effective specs for:

- Pack identity/rest semantics;
- `*`/`**`/`***` lane behavior;
- new/generalized bytecodes and stack effects;
- EmptyPack immediate representation if VM value spec exposes tags.

## Tests to run now

Run G1–G4 serially. No broad workspace certification.

## Acceptance

No missing visitor/fingerprint/metadata arm; warm/cold parity; compatibility gates pass or unrelated failures are classified; fixed-call/resource invariants have concrete evidence.

## Local STOP / CONSULT

Stop if compatibility reveals Pack/C3 semantic collision or fixed-call bytecode changes merely because Pack exists.

## Checkpoint update

Record coherent G1–G4 evidence only.

---

# T9 — Lifecycle closure

## Purpose

Close P3 administratively only after implementation and verification evidence exist.

## Actions

1. Review scoped diff; remove accidental unrelated/whitespace-only changes.
2. Update plan lifecycle truthfully.
3. Update checkpoint:
   - P3 status/completion/verification;
   - durable Pack interfaces/invariants actually implemented;
   - G1–G4 evidence;
   - residual/baseline failures;
   - consultations/amendments;
   - latest completed plan;
   - next action.
4. Create `LANG007.C4.P3-walkthrough.md`.
5. Create `LANG007.C4.P3-handoff.md`.
6. Keep P5 blocked until P4 is also semantically stable per checkpoint DAG.

## Acceptance

No completion claim exceeds actual evidence. Walkthrough records shipped reality; handoff records operational inheritance.

---

# 19. Verification gates

# G1 — Semantic type/static Pack vertical

## Purpose

Prove Pack is a proper type/static value and value-domain variadic static binders work without disturbing C3 pack binders.

## Coverage

CV-02–CV-10, CV-26, CV-30, CV-32.

## Run

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast pack_type_remainder -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta pack -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::pack_types -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_pack -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_pack_definitions -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic incremental::pack_types -- --nocapture
```

Adapt filters to actual module names and confirm nonzero selection.

## Gate result required

All P3-owned semantic/static cases pass. Any unrelated failure is classified before edits.

---

# G2 — Runtime Pack/rest/spread vertical

## Purpose

Prove runtime identity/storage/GC, rest parity, construction, spread, and forwarding.

## Coverage

CV-01, CV-11–CV-22, CV-24–CV-25, CV-31.

## Run

Conceptually:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::pack -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core calling::rest_pack -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core memory::pack -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core collections::outgoing_packs_completion -- --nocapture
```

## Gate result required

All activation paths return Pack, slices survive GC, and forwarding reads Pack lanes directly.

---

# G3 — Fixed-call/resource/performance gate

## Purpose

Prove P3 performance invariants structurally.

## Blocking evidence

1. Existing static call disassembly contains ordinary direct invoke and no Pack assembly machinery:

```text
NewArgumentPack
PackPushPositional
PackReserveStaticLabel
PackReserveComputedLabel
PackFillReservedLabel
PackExpandLabels
PackExpandComplete
FinishPack
InvokePack
SuperSendPack
```

for fixed no-spread calls.

2. Static Tuple retains direct Tuple build behavior.
3. Fixed known-shape Pack construction uses direct Pack build (or proven equivalent), not mutable builder.
4. Multi-spread emits one mutable destination builder.
5. Deep Pack view/rest chains share root/backing.
6. StaticPack view/rest chains share backing.
7. Large labeled builder crosses to indexed duplicate membership.

A systematic fixed-call regression attributable to P3 blocks completion.

No wall-clock microbenchmark is required if path/backing/allocation evidence is conclusive.

---

# G4 — Focused compatibility/stabilization gate

## Purpose

Prove P3 did not regress C1–C3/P2 and directly affected crates compile.

## Run serially

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast -p phalcom-type-meta -p phalcom-semantic -p phalcom-core

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic type_alias -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic adts -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_structural -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic static_constants -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core language::algebraic_data -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core static_const -- --nocapture
```

If metadata affects modules or parser API affects LSP compilation, add `cargo check` for those crates. Do not automatically run their broad suites.

Explicitly do not run full `language-corpus` as P3 closure gate unless separately requested; its bootstrap cost and known deferred state are already recorded.

## Completion classification

P3 may close `COMPLETE / IMPLEMENTED / FOCUSED_TESTED` when P3-owned gates pass and broader unrelated failures are classified. Do not claim `RELEASE_COMPLETE`.

---

# 20. Final acceptance mapping

The walkthrough must map every applicable CV ID to executed evidence.

Minimum non-negotiable acceptance:

| Obligation | Required proof |
|---|---|
| proper Pack type | direct/open/symbolic shapes + C3 bridge |
| StaticPack | lineage, logical equality/fingerprint, O(1) slice |
| value variadic static params | positional/labeled/complete + zero capture |
| runtime Pack | empty/root/slice/GC |
| builder separation | dynamic send no surface Pack; Pack construction finalizes separately |
| rest convergence | all method modes + block + verified Fiber path |
| forwarding | direct Pack lanes, no Tuple/List bridge |
| Tuple interop | explicit only |
| views | canonical Empty/Entry + shared backing |
| performance | fixed call no builder/Pack; fused spread; indexed large labels |
| metadata/incremental | round-trip + warm/cold parity |
| compatibility | C1–C3 and P2 focused gates |

---

# 21. Performance and resource evidence

Required asymptotics:

```text
Pack positional length              O(1)
Pack labeled length                 O(1)
Pack first/rest view                O(1)
Pack slice creation                 O(1)
StaticPack first/rest/slice         O(1)
Pack structural equality/hash       O(n visible elements)
StaticPack equality/hash/fingerprint O(n visible elements when requested)
private builder append              amortized O(1)
whole Pack finalize                 O(n)
fused multi-spread                  O(total contributed elements)
fixed call first-class Pack alloc   0 required
fixed call builder alloc            0 required
```

Accepted memory tradeoff:

```text
small live Pack slice may retain full immutable root backing
```

Rejected alternative:

```text
copy every suffix to avoid backing retention
```

because it violates O(1) rest/view and makes recursive structural algorithms quadratic.

Large-label duplicate detection must become threshold-indexed; wall-clock timing is not primary proof.

---

# 22. Checkpoint bookkeeping

## At actual P3 start

Change P3 row from:

```text
PROPOSED / NOT_STARTED / UNVERIFIED
```

to truthful active state, normally:

```text
IN_PROGRESS / PARTIAL / UNVERIFIED
```

Record start revision and keep P2 as latest completed until P3 closes.

## During P3

Update checkpoint only when:

- T1 proper Pack type/metadata interface stabilizes;
- T2 StaticPack/value-variadic interface stabilizes;
- T3 runtime root/slice lifetime architecture is proven;
- a consultation changes architecture;
- a significant deferred failure is discovered;
- G1–G4 complete.

Do not log every edit/test.

## At P3 completion

Only then set:

```text
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED   # only if named focused gates pass
```

Set `latest_completed_plan: LANG007.C4.P3` and next action according to live P4 state/DAG. P5 remains blocked until P3 and P4 are both stable.

---

# 23. Walkthrough deliverable

Create:

```text
LANG007.C4.P3-walkthrough.md
```

It records:

- final revision/state;
- actual Pack type representation;
- actual StaticPack backing/slice representation;
- actual runtime root/slice/EmptyPack representation;
- compiler bytecodes/lowering actually shipped;
- rest entry paths migrated;
- spread/forwarding paths;
- metadata/schema changes;
- deviations from plan;
- consultations/amendments;
- tests added;
- exact tests actually executed/results;
- deliberately deferred tests;
- performance evidence;
- residual/baseline failures;
- final verification classification.

---

# 24. Handoff deliverable

Create:

```text
LANG007.C4.P3-handoff.md
```

It must give P4/P5 successors:

```text
repository revision
P3 completion/verification state
proper Pack type authority
C3 bridge authority
StaticPack store/slice API
runtime Pack root/slice API
canonical Pack views
rest/spread convergence result
metadata/schema state
performance invariants
known baseline failures
must-read paths
P4/P5 boundaries
things not to redesign
```

Specific downstream rules:

- P4 may generalize Record FieldKey without changing Pack Symbol labels.
- P5 derives ordered-product reflection from stable Pack structure; reflection must not become Pack's source of truth.
- P6 consumes StaticPack/views directly and must not retrofit Iterator into P3 semantics.
- P8 may serialize logical StaticPack DAG content, never P3 backing IDs.
- P9 may retire legacy public TypePack names while preserving optimized C3 internals/bridge.

---

# 25. Completion truth table

Keep distinct:

```text
source written
  != feature semantically implemented
  != focused Pack tests passed
  != C1–C3/P2 compatibility passed
  != P3 administratively complete
  != C4 checkpoint complete
  != release certified
```

P3 is not complete because the tree builds, is clean, committed, or pushed.

---

# 26. Plan self-review

- [x] Proper Pack type and C3 pack terms have distinct authority.
- [x] Pack type metadata is planned before cross-module use.
- [x] StaticPack extends P2 store instead of parallel storage.
- [x] Static slicing remains O(1) without content-hashing every suffix.
- [x] Runtime slices have explicit GC-safe root ownership.
- [x] Empty Pack is distinct and cheap.
- [x] Dynamic call builder stays private and directly consumable.
- [x] `FinishTuplePack` remains Tuple-specific.
- [x] Fixed calls retain blocking no-builder/no-Pack guard.
- [x] Every known rest activation path has an explicit migration task.
- [x] Pack spread does not require Iterator.
- [x] Tuple interop is explicit and Record remains isolated.
- [x] Value-domain static variadics preserve P2 TypeParameterId rules.
- [x] Evaluation/error order has dedicated tests.
- [x] Duplicate-label scaling has algorithmic solution/evidence.
- [x] No P4/P5/P6 feature is required for P3 closure.
- [x] StaticPack durable metadata/materialization remains deferred.
- [x] Tests are separated by semantic/compiler/runtime responsibility.
- [x] Slow broad language-corpus execution is not mandatory.
- [x] checkpoint/walkthrough/handoff lifecycle obligations are explicit.
- [x] STOP / CONSULT triggers cover type identity, GC, metadata, ABI, and performance.

---

# 27. Execution summary for Luna

Execute:

```text
T0  takeover / actual start
 ↓
T1  proper Pack type + syntax + identity + metadata
 ↓
T2  StaticPack + value-domain variadic static parameters
 ↓
T3  runtime Pack root/slice + EmptyPack + GC
 ↓
T4  Pack construction + builder finalization + scalable duplicates
 ↓
T5  rest capture convergence
 ↓
T6  spread/forwarding/interop convergence
 ↓
T7  canonical source views/API
 ↓
T8  incrementality + compatibility + performance closure
 ↓
G1–G4
 ↓
T9  lifecycle closure
```

The central architectural rule is:

```text
unify semantic meaning,
not every representation.
```

C3 type-pack terms, P2 StaticValue storage, runtime immutable Pack storage, and private outgoing-call builders remain specialized representations with one coherent language meaning and explicit bridges. That is the intended P3 architecture.
