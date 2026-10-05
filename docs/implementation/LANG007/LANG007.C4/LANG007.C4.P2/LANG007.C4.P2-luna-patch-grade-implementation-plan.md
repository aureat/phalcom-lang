---
id: LANG007.C4.P2
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
depends_on:
  - LANG007.C4.P1
follows: LANG007.C4.P1
supersedes: null
repository: aureat/phalcom-lang
prepared: 2026-10-04
prepared_against_revision: c2a938c7c195b1098891ecf0cd6940e89e32ee17
title: "Static Value Model, Static Definitions, Evaluator Foundation, and Closed const"
---

# LANG007.C4.P2 — Static Value Model, Static Definitions, Evaluator Foundation, and Closed `const`

## Luna Patch-Grade Implementation Plan

This plan converts the ratified C4.P2 requirements into an implementation-ready work package for Telos / `aureat/phalcom-lang`.

Planning re-verified the repository before fixing this plan:

```text
branch:   main
revision: c2a938c7c195b1098891ecf0cd6940e89e32ee17
compare:  c2a938... == main
```

There is no post-analysis source drift from the baseline audited by the supplied P2 requirements analysis.

There is a documentation-state distinction to preserve. The supplied checkpoint/P1 walkthrough contain later P1 evidence than the repository-committed copies at the same source revision, but both supplied and committed state still agree on the material fact: **P1 is not administratively closed**. P2 implementation must not begin until P1 G2/T11 closure is complete and the checkpoint names P2 as active.

This plan follows `docs/workflow/luna-patch-grade-plan-schema.md` and the Phalcom Implementation Record & Work-Lifecycle Convention.

---

## 0. Executor contract

This is a patch-grade execution record, not an invitation to rediscover the design.

The implementer may adapt private helper names, module decomposition, exact test placement, and mechanically equivalent APIs to live source. The implementer must not improvise ownership, identity, lifetime, phase, type-system, evaluator, dependency, or materialization architecture.

Required behavior:

- verify P1 administrative closure before source edits;
- treat semantics as the authority for static identity, values, dependencies, evaluation and materialization descriptions;
- reuse C3 kinded/static-term semantics rather than duplicating them;
- keep ordinary compile-time values out of `TypeData` unless they already are genuine C3 static terms;
- keep `phalcom_core::Value`, `ObjRef`, VM `Symbol`, heap addresses and runtime object identity out of semantic `StaticValue`;
- preserve P1's rule that `const` has no ordinary runtime slot and its initializer is not emitted as runtime code;
- use ordinary semantic call/operator resolution before deciding whether an operation is statically executable;
- authorize static operations by exact semantic identity, never by selector/operator spelling;
- use the existing `SemanticDb`, dependency graph, `QueryBudget`, cancellation and current/LKG machinery;
- implement evaluation as a bounded explicit-frame semantic machine, not host-recursive guest evaluation and not bytecode execution;
- preserve source order and branch/short-circuit laziness;
- fail unsupported behavior only when the unsupported path is actually executed;
- require a closed `const` to reduce to a concrete value;
- preserve C1–C3 alias/kind/type-lambda/substitution behavior outside the intended generalization;
- make P3–P8 extensions additive;
- update the checkpoint only at durable gates;
- produce P2 walkthrough and handoff only after actual implementation and evidence;
- classify unrelated failures A/B/C/D and defer D;
- STOP AND CONSULT on the triggers in §13.

### 0.1 Hard P1 execution gate

Before editing P2 source, require:

```text
LANG007.C4.P1
    status: COMPLETE
    completion: IMPLEMENTED
    verification: truthful final classification

LANG007.C4-CHECKPOINT.md
    latest_completed_plan: LANG007.C4.P1
    active_plan: LANG007.C4.P2
```

Preparing this plan while P1 remains open is valid. Executing it is not.

---

## 1. Goal

Implement C4.P2 as the semantic foundation for compile-time Telos by establishing:

1. semantic, snapshot-safe `StaticValue`;
2. generalized source `type` definitions with kinded and ordinary-value domains;
3. mixed static parameters without corrupting C3 generics;
4. checked static definitions with C3 alias compatibility;
5. exact compiler-owned static intrinsic authorization;
6. deterministic, bounded, cancellable explicit-frame evaluation;
7. closed `const` as a semantic DB product;
8. content-based dependencies/fingerprints and deterministic cycles;
9. semantic materialization descriptors;
10. a bounded same-module primitive materialization vertical.

The governing law is:

> **Static Telos is ordinary Telos semantic meaning executed by a bounded deterministic semantic evaluator—not a second VM, a second pattern language, or a parallel type system.**

---

## 2. Checkpoint acceptance objective

P2 is one implementation plan inside `LANG007.C4`; it does not close the checkpoint.

P2 is accepted when a closed static program can flow through:

```text
source
  ↓
ordinary semantic resolution/type checking
  ↓
StaticDefinition / StaticConstant products
  ↓
StaticValueDomain contracts
  ↓
exact StaticOperationResolution
  ↓
bounded StaticEvaluator
  ↓
StaticEvalOutcome::Reduced
  ↓
MaterializationDescriptor
  ↓
ModuleLoweringSemantics
  ↓
compiler materialization for the bounded P2 primitive subset
```

without any of the following:

```text
runtime execution of initializer AST
bytecode interpretation during semantics
arbitrary native/user method execution
VM heap identity inside semantic values
a parallel subtype/type relation engine
selector-spelling authorization
stale-value reuse after current semantic invalidation
```

---

## 3. Repository grounding

### 3.1 P1 seams P2 must consume

Current source establishes:

```text
BindingKind::{Let, LetMut, Const}

let      -> runtime immutable
let mut  -> runtime mutable
const    -> static phase
```

and module binding semantics equivalent to:

```rust
ModuleBindingSemantics::Runtime { mutable: bool }
ModuleBindingSemantics::StaticConstant
ModuleBindingSemantics::Declaration
```

P1 compiler barriers:

```text
global_static_constants
static_const_scopes
CompilerError::StaticConstUnmaterialized
```

remain useful defensive lowering seams, but they are not P2's value/identity authority.

### 3.2 Live facts verified during planning

#### `type` is still alias-shaped

`TypeAliasDef` still conceptually contains:

```rust
result_kind: Option<KindSyntax>
generic_parameters: Vec<GenericParameterSyntax>
where_clause: Option<WhereClauseSyntax>
body: TypeAnnotation
```

and the parser still effectively accepts:

```text
type Name : ResultKind <Generics> where ... = TypeForm
```

P2 therefore requires a real syntax/AST migration.

#### Existing generic parameters are kinded

`TypeParameterData` retains `kind: KindId`; `GenericSignature` is the C3 kinded generic authority. P2 must not globally turn it into a value-parameter system.

#### Type-member syntax already exposes the conceptual distinction

AST type-member parameter/result syntax distinguishes kind-like and const-type-like forms, while semantic `TypeMemberDomain` remains kind-only. P2 should establish one conceptual `StaticValueDomain` and use an adapter rather than add another competing model.

#### Operators are ordinary semantic dispatch

`apply_binary_operation_from_typed(...)` maps syntax to selectors, considers direct/reflected dispatch, and funnels through canonical callable application. Static execution therefore must consume the exact selected semantic target.

#### Expression analysis retains callable identity

`ExpressionAnalysis.callable: Option<CallableId>` and the checking context's resolved-callable map provide the correct authority for static intrinsic selection.

#### Semantic interning has persistent session lifetime

`SemanticWorkspaceSession` owns `Arc<TypeStore>` across revisions and snapshots. `TypeStore` is cloneable and IDs index append-oriented canonical vectors. `StaticValueStore` should mirror this lifetime model.

#### Exact Int support already exists in the workspace

The workspace already uses `num-bigint = 0.4`. Runtime Int is small `i64` plus heap `Object::LargeInt(BigInt)`. `phalcom-semantic` may depend directly on workspace `num-bigint`, but must not depend on `phalcom-core`.

#### Constant pools are GC trace edges

`Chunk::constants: Vec<Value>` is traced from callable/closure heap edges. This makes P2 primitive LargeInt materialization viable, subject to a focused rooting regression.

### 3.3 Re-read-before-edit rule

Re-read the named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as a consultation trigger.

---

## 4. Required reads

At minimum:

```text
docs/workflow/luna-patch-grade-plan-schema.md
LANG007.C4-CHECKPOINT.md
LANG007.C4-requirements-analysis.md
LANG007.C4.P1-walkthrough.md
LANG007.C4.P1-handoff.md
LANG007.C4.P2-requirements-analysis-main.md
LANG007.C4-structural-static-upgrade-handoff.md
implementation-record-lifecycle-convention.md
```

Inspect predecessor C3 records wherever they define:

```text
Kind
TypeData
StaticDomain
StaticArgument
TypePack / RecordRow internal terms
TypeAliasInfo
type lambdas
type members
generic signatures
semantic query products
metadata stability
```

Inspect live source:

```text
phalcom-ast/src/{ast.rs,parser.rs,token.rs,lexer.rs}

phalcom-semantic/src/
  {lib.rs,session.rs,snapshot.rs,semantic_shard.rs,declarations.rs,type_alias.rs,identity.rs}
  core_surface/identity.rs
  source_index/**
  metadata/**
  checker/{analysis.rs,typed_expr.rs,body.rs,statement.rs,expression.rs,control.rs,context.rs,call.rs,associated.rs,binding.rs,flow/**}
  db/{key.rs,product.rs,query.rs,state.rs,fingerprint.rs}
  types/{store.rs,kind.rs,annotation.rs,parameter.rs,type_member.rs,denotation.rs,outcome.rs,relation.rs,substitution.rs,type_lambda.rs,row.rs}
  {static_data.rs,static_case.rs,data_semantics.rs,enum_semantics.rs,algebraic_semantics.rs}

phalcom-modules/src/{interface.rs,linker.rs,graph.rs,source.rs}

phalcom-core/src/
  modules/{semantic_lowering.rs,compile.rs,linkage.rs}
  compiler/lib/{mod.rs,expr.rs,scope.rs,error.rs}
  {chunk.rs,bytecode.rs}
  value/**
  heap/**
  product/**
  vm/**
  primitive/{number.rs,int.rs}

phalcom-type-meta/src/{header.rs,bundle.rs}
```

---

## 5. Normative decisions fixed by this plan

### 5.1 Static domains

Use one semantic distinction equivalent to:

```rust
pub enum StaticValueDomain {
    Kind(KindId),
    ValueType(TypeId),
}
```

Examples:

```text
T: Type       -> Kind(Type)
P: TypePack   -> Kind(TypePack)
S: Shape      -> Kind(static-domain kind for Shape)
N: Int        -> ValueType(Int)
Flag: Bool    -> ValueType(Bool)
result: Type  -> Kind(Type)
result: Bool  -> ValueType(Bool)
```

The parser should use a **neutral** source carrier for domains, conceptually:

```rust
pub struct StaticDomainSyntax {
    pub form: TypeAnnotation,
    pub range: SourceRange,
}
```

Semantic resolution decides Kind vs ValueType. Do not make the parser guess whether `Shape` or `Bool` is a kind/static-domain versus ordinary proper type.

### 5.2 Static definition parameters

Do not mutate C3 `TypeParameterData` globally.

Use definition-owned parameters:

```rust
pub struct StaticParameterId {
    definition: DeclarationId,
    index: u32,
}

pub struct StaticParameter {
    id: StaticParameterId,
    name: Box<str>,
    mode: GenericParameterMode,
    domain: StaticValueDomain,
    type_parameter: Option<TypeParameterId>, // kinded lane only
}

pub struct StaticDefinitionSignature {
    parameters: Box<[StaticParameter]>,
    constraints: ...,
}
```

`T: Type` retains a C3 TypeParameter bridge. `N: Int` does not fabricate one.

`*P`, `**P`, `***P` remain C3 kinded/pack binders in P2; P3 owns first-class Pack values.

### 5.3 `type` source model

Preferred final AST:

```rust
pub struct TypeDefinitionDef {
    name: String,
    name_range: SourceRange,
    parameters: Vec<StaticParameterSyntax>,
    result_domain: Option<StaticDomainSyntax>,
    where_clause: Option<WhereClauseSyntax>,
    body: StaticDefinitionBody,
    ...
}

pub enum StaticDefinitionBody {
    Expression(Expr),
    Block(LexicalBlock),
}
```

Preferred statement spelling:

```rust
Statement::TypeDefinition(TypeDefinitionDef)
```

A temporary compatibility alias during a compile-preserving patch sequence is acceptable; two permanent independently populated AST authorities are not.

Canonical grammar:

```telos
type Optional<T> =
    T | None

type IsZero<N: Int>: Bool =
    N == 0

type Count<T>: Int {
    let mut n = 0
    n = n + 1
    n
}
```

Rules:

- generics/parameters precede result domain;
- block body has no `=`;
- block is ordinary `LexicalBlock`, not Closure semantics;
- old `type Name<T> = TypeForm` remains legal with implicit `Kind(Type)` result.

Use one explicit type-form bridge in static expression contexts, conceptually `Expr::StaticForm(TypeAnnotation)`, or an equivalent checked representation. Do not create a second semantic expression language.

### 5.4 Static definition authority

Canonical semantic product:

```rust
pub struct StaticDefinition {
    id: StaticDefinitionId, // thin DeclarationId wrapper is fine
    signature: StaticDefinitionSignature,
    result_domain: StaticValueDomain,
    body: CheckedStaticBody,
    dependencies: Arc<[SemanticDependency]>,
    source: SemanticSourceSpan,
    fingerprint: ProductFingerprint,
}
```

A generic definition is a checked definition, not an automatic `Symbolic` evaluation result.

### 5.5 TypeAliasInfo compatibility

Authority direction:

```text
StaticDefinition
    ↓
TypeAliasInfo compatibility/optimized projection
```

for Type-valued definitions representable by existing C3 alias machinery.

Do not independently derive two competing semantic products from source. Preserve efficient TypeStore/type-lambda normalization for ordinary aliases; do not route every alias through the general evaluator.

### 5.6 Static applications

Semantic application arguments:

```rust
pub enum StaticApplicationArgument {
    KindedTerm(StaticArgument),
    Value(StaticValueRef),
}
```

P2 must establish argument syntax that can represent ordinary values generally, not an Int-only ABI.

Required early examples:

```telos
IsZero<3>
Choose<true, Int, String>
```

and the design must naturally admit a later `#Symbol` argument.

Ambiguous names/literals are resolved against the target definition's expected `StaticValueDomain`.

### 5.7 StaticValueStore lifetime

Use:

```rust
StaticValueStoreId(u64)
StaticValueId(u32)

pub struct StaticValueRef {
    store: StaticValueStoreId,
    value: StaticValueId,
}
```

Published products use store-qualified refs.

Ownership mirrors TypeStore:

```text
SemanticWorkspaceSession -> Arc<StaticValueStore>
SemanticSnapshot         -> Arc<StaticValueStore>
mutation                  -> lineage-preserving Arc::make_mut / persistent clone
```

Within one lineage:

- IDs are append-only;
- old indices are never reused/compacted while reusable products can reference them;
- clone-on-write preserves store ID and old prefix;
- unrelated new store gets new store ID;
- dereference validates store identity.

### 5.8 StaticValue model

P2 variants:

```rust
pub enum StaticValueData {
    Unit,
    Bool(bool),
    Int(StaticInt),
    Float(StaticFloat),
    String(StaticStringId),
    Symbol(StaticSymbolId),
    Type(TypeId),
    StaticTerm(TypeId),
    Tuple(StaticTupleId),
    Record(StaticRecordId),
    Data(StaticDataValueId),
    Variant(StaticVariantValueId),
}
```

P3 adds Pack; P4 generalizes Record keys; P5 adds reflection values.

#### Int

Recommended:

```rust
pub enum StaticInt {
    Small(i64),
    Large(Arc<num_bigint::BigInt>),
}
```

Normalize back to Small if representable. Semantics are exact/unbounded.

#### Float

Store exact binary64 bits:

```rust
pub struct StaticFloat { bits: u64 }
```

Separate storage/interner identity from Telos semantic equality/hash:

```text
NaN != NaN
+0.0 == -0.0
signed-zero bits preserved for materialization
```

#### String / Symbol

String is immutable UTF-8 content semantics.

Symbol is stable semantic spelling/content, never VM `Symbol(u32)`.

#### Type / StaticTerm

`Type(TypeId)` requires proper-Type validation. `StaticTerm(TypeId)` is for other C3 terms. Do not allow every TypeId-backed term to satisfy result domain `Type`.

### 5.9 Structural values

Tuple/Record/Data/Variant are immutable acyclic DAG nodes with child StaticValue refs.

Requirements:

- preserve source evaluation order;
- exact `VariantId`;
- canonical Data/constructor identity;
- no runtime ProductStorage pointer;
- no arbitrary cyclic containment;
- no arbitrary user constructor/factory execution;
- current Record key semantics only until P4.

### 5.10 Static operation authorization

Build a compiler-owned `StaticIntrinsicRegistry` keyed by exact semantic callable identity:

```rust
BTreeMap<CallableId, StaticIntrinsicOperation>
```

Ordinary semantic resolution runs first. A `StaticOperationResolution` is published only if the exact selected callable/structural operation is authorized.

Authority rule:

```text
operator token / selector spelling   != authority
selected exact semantic identity     + registry membership == authority
```

User overloads and non-whitelisted native methods must not execute statically. Reflected binary dispatch is allowed only when the exact reflected target is independently authorized.

### 5.11 Checked static bodies

Use ordinary checker facts, conceptually:

```rust
pub struct CheckedStaticBody {
    owner: StaticBodyId,
    root: StaticBodyRoot,
    expressions: Arc<ExpressionAnalysisIndex>,
    bindings: Arc<BindingAnalysisIndex>,
    static_operations: Arc<StaticOperationResolutionIndex>,
    dependencies: Arc<[SemanticDependency]>,
    result: StaticValueDomain,
}
```

The evaluator consumes checked facts; it does not type-check or resolve dispatch itself.

### 5.12 Evaluator

Explicit-frame semantic machine. Future-ready control seam:

```rust
pub enum EvalControl {
    Value(StaticValueRef),
    Return(StaticValueRef),
    Break,
    Continue,
}
```

P2 executes only its owned subset.

Static locals:

```rust
pub enum StaticSlot {
    Uninitialized,
    Initialized(StaticValueRef),
}
```

P2 evaluator supports:

```text
literals/static values
static parameter reads
static local reads/writes
lexical blocks/trailing value
if
and/or short-circuit
static const reads
non-recursive static definition application
authorized static intrinsics
Tuple/Record/Data/Variant pure construction
phase-safe projections needed by supported cases
```

P2 does not implement match/for/while, general destructuring, return/break/continue execution, proof predicates, general recursion or residual normalization.

### 5.13 Path-sensitive executability

Do not eagerly reject a whole body because runtime-only syntax exists in an untaken branch:

```telos
const X =
    if false {
        runtimeOnly()
    } else {
        1
    }
```

must reduce if ordinary semantic checking succeeds and the forbidden call is not executed.

### 5.14 Recursion and outcomes

P2 conservatively rejects active static definition/application re-entry with a distinct `Recursive` terminal. P7 later adds exact application-key recursion/memoization.

Outcome seam:

```rust
pub enum StaticEvalOutcome {
    Reduced(StaticValueRef),
    Symbolic(StaticResidualId),
    Invalid(StaticEvalError),
    Recursive(StaticCycle),
    BudgetExceeded(BudgetReport),
    Cancelled,
}
```

P7 owns real residual IR. A closed `const` may not publish success from `Symbolic`.

### 5.15 Query products

Add:

```rust
QueryKey::StaticDefinition(DeclarationId)
QueryKey::StaticConstant(SymbolId)

SemanticProduct::StaticDefinition(...)
SemanticProduct::StaticConstant(...)

SemanticDependency::StaticDefinition(DeclarationId)
SemanticDependency::StaticConstant(SymbolId)
```

Top-level static constant identity is `SymbolId(module,name)` or a thin `StaticConstantId` wrapper.

`StaticConstantProduct` carries current outcome, domains, dependencies, materialization descriptor/provenance and content fingerprint. Failed/cancelled/recursive/budget/symbolic closed constants do not expose a current successful value.

### 5.16 Source shards and invalidation

Generalize alias-only source retention into static definition inputs and static constant inputs with targeted fingerprints. Do not require whole-workspace rescans for one definition/constant query.

### 5.17 Content fingerprints

Constant fingerprints are semantic-content fingerprints, never:

```text
StaticValueId
arena insertion order
ObjRef
VM Symbol ID
source range
type only
```

Use:

- canonical exact Int bytes;
- exact Float bits for representation/materialization invalidation;
- String UTF-8 content;
- Symbol spelling;
- canonical Type/static-term fingerprint;
- structural tag + stable nominal identity + child fingerprints.

Telos Float equality remains separate from representation fingerprinting.

### 5.18 Cancellation/LKG

Cancelled/incomplete evaluation never publishes semantic success.

LKG may remain editor presentation data under existing DB policy, but compiler lowering consumes current successful products only. A previous `const X = 10` may not silently compile as 10 after current source becomes invalid.

### 5.19 Resource budgeting

Reuse `QueryBudget` and `CancellationToken`.

Charge at least:

```text
executed expression
executed statement
static definition application
intrinsic operation
structural child construction
projection/traversal proportional to work
```

Add a shared `StaticValueNodes` budget dimension if needed to bound semantic DAG growth.

BigInt shifts/powers require preflight bit-growth/work estimation so semantic budget exhaustion occurs before pathological host allocation. Do not introduce an unrelated `MAX_CONST_STEPS` or language-semantic bit ceiling.

### 5.20 Phase-correct dependencies

Static evaluation may read static parameters/locals/constants, Types, C3 static terms/domains, static definitions and authorized intrinsics.

It may not read runtime let/let mut, runtime method parameters, `self`, runtime fields/globals or runtime closure captures. Optimizer knowledge does not change phase.

### 5.21 Cross-module P2 policy

Live-source provider in the same semantic workspace may be queried/reused.

Artifact-only provider without P8 static metadata fails closed. Never recover a static value from runtime global storage or initializer bytecode.

### 5.22 Materialization architecture and scope decision

Semantic descriptor is VM-independent:

```text
StaticValue
    ↓
MaterializationDescriptor
    ↓
ModuleLoweringSemantics
    ↓
compiler/runtime representation
```

**M2 is adopted for P2**: same-module runtime materialization is completed for:

```text
Unit
Bool
Int
Float
String
Symbol
```

Semantic descriptors may represent all P2 values, but runtime consumption of:

```text
Type
Tuple
Record
Data
Variant
```

is explicitly deferred to P8 unless a separately ratified amendment changes the boundary.

P8 retains durable metadata, artifact import/export, general structural constant pools, Pack/FieldKey-era values and full cross-module/general materialization.

### 5.23 ModuleLoweringSemantics

Add a frozen semantic static-constant/materialization projection. The compiler consumes it and must not resolve const semantics, rerun initializer AST, rerun inference, infer static operations or construct its own semantic value graph.

Runtime references to a P2-materializable const load/materialize the already-reduced value. `StaticConstUnmaterialized` becomes either an internal missing-product assertion or a specific deferred-structural-materialization diagnostic; it never triggers initializer fallback.

---

## 6. Stable takeover map

| Fact | Canonical owner | Consumer | Forbidden duplicate |
|---|---|---|---|
| static definition identity | `DeclarationId` / thin ID | DB/evaluator/source index | source-range/hash identity |
| static constant identity | `SymbolId` / thin ID | DB/lowering | VM Symbol/compiler slot |
| static parameter identity | definition + index | checker/evaluator | string lookup identity |
| static domain | `StaticValueDomain` | checker/evaluator | evaluator-local type taxonomy |
| type/kind relation | TypeStore/relation engine | static checker | second subtype engine |
| ordinary static value | `StaticValueStore` | evaluator/products | runtime `Value` |
| static operation eligibility | `StaticOperationResolution` | evaluator | operator spelling |
| dependencies | SemanticDb graph | incremental | private const cache graph |
| budget | `QueryBudget` | evaluator | hidden static limits |
| value fingerprint | semantic content hash | DB | arena ID |
| materialization descriptor | semantic P2 | core lowering | compiler reconstructed graph |
| runtime primitive value | core materializer | VM | semantic heap object |

---

## 7. Global invariants

- **P2-INV-01** — `const` keeps no ordinary runtime slot and emits no initializer runtime code.
- **P2-INV-02** — semantic StaticValue contains no runtime Value/ObjRef/VM object identity.
- **P2-INV-03** — kinded terms and ordinary compile-time values remain distinct domains.
- **P2-INV-04** — C3 Type/static-term identity is reused, not duplicated.
- **P2-INV-05** — ordinary static Int is exact/unbounded.
- **P2-INV-06** — Float bits and Telos semantic equality/hash are separate concerns.
- **P2-INV-07** — String semantics are content semantics.
- **P2-INV-08** — Symbol semantic identity is stable spelling/content.
- **P2-INV-09** — published StaticValue references are store-qualified or equivalently protected from stale-ID aliasing.
- **P2-INV-10** — StaticValueStore lineage is append-only while reusable products reference it.
- **P2-INV-11** — definitions are declaration-identified; ranges are provenance only.
- **P2-INV-12** — ordinary static value parameters do not masquerade as TypeParameters.
- **P2-INV-13** — existing Type aliases retain optimized C3 semantics.
- **P2-INV-14** — StaticDefinition is the generalized source-definition authority; TypeAliasInfo is compatibility/optimized projection where applicable.
- **P2-INV-15** — static operation authorization uses exact semantic identity.
- **P2-INV-16** — user overloads never acquire compile-time authority by selector spelling.
- **P2-INV-17** — reflected dispatch authorization uses the exact reflected target.
- **P2-INV-18** — evaluator consumes checked semantics and performs no independent type/dispatch resolution.
- **P2-INV-19** — no compile-time VM/arbitrary native dispatch.
- **P2-INV-20** — source evaluation order is exact.
- **P2-INV-21** — branches/short-circuiting are lazy.
- **P2-INV-22** — unsupported behavior fails only if executed.
- **P2-INV-23** — guest computation uses explicit frames, not unbounded host recursion.
- **P2-INV-24** — evaluator local storage is semantic, not VM slot/sentinel storage.
- **P2-INV-25** — uninitialized static local read is a static error, never `None`.
- **P2-INV-26** — recursion is distinct from budget/cancel/invalid.
- **P2-INV-27** — closed `const` must reduce concretely.
- **P2-INV-28** — constant cycles are deterministic, not stack/budget accidents.
- **P2-INV-29** — phase is semantic, not optimizer-derived.
- **P2-INV-30** — QueryBudget is the resource authority.
- **P2-INV-31** — cancellation/incomplete outcomes are never cached as successful reductions.
- **P2-INV-32** — LKG cannot become compiler truth after current invalidation.
- **P2-INV-33** — constant value contents participate in fingerprints/invalidation.
- **P2-INV-34** — fingerprints never depend on StaticValue arena insertion order.
- **P2-INV-35** — structural StaticValues form immutable acyclic DAGs.
- **P2-INV-36** — variant values retain exact `VariantId`.
- **P2-INV-37** — static data construction is structural semantic construction, not arbitrary constructor execution.
- **P2-INV-38** — materialization never reruns initializer source.
- **P2-INV-39** — semantic descriptors contain no VM object identity.
- **P2-INV-40** — primitive materialization preserves Int precision, Float bits, String content and Symbol spelling.
- **P2-INV-41** — LargeInt materialized constants are proven GC-safe.
- **P2-INV-42** — P2 does not grow an ad hoc structural materializer that belongs to P8.
- **P2-INV-43** — source indexing traverses generalized static definition bodies.
- **P2-INV-44** — P3/P4/P5 extend static value/domain seams additively.
- **P2-INV-45** — P6/P7 extend the evaluator rather than replace it.

---

## 8. Non-goals

P2 does not own first-class Pack, generalized FieldKey, Product/Sum reflection, static match/for/while, general pattern destructuring, proof predicates, general recursive static computation, normalized residual IR, durable static metadata, artifact-only static execution, general structural materialization or P9 tooling/ontology closure.

It also does not own arbitrary compile-time I/O, time, randomness, network, filesystem, process, fibers, scheduler or arbitrary runtime/native method execution.

---

## 9. Expected impact map

Expected new semantic modules, exact private decomposition flexible:

```text
phalcom-semantic/src/static_domain.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/static_value/**
phalcom-semantic/src/static_intrinsic.rs
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/materialization.rs
```

Expected integration:

```text
phalcom-ast/src/{ast.rs,parser.rs}
phalcom-semantic/src/{lib.rs,session.rs,snapshot.rs,semantic_shard.rs,declarations.rs,type_alias.rs}
phalcom-semantic/src/source_index/**
phalcom-semantic/src/checker/{analysis.rs,body.rs,statement.rs,expression.rs,context.rs,call.rs}
phalcom-semantic/src/db/{key.rs,product.rs,query.rs,state.rs,fingerprint.rs}
phalcom-semantic/src/types/{annotation.rs,parameter.rs,type_member.rs,outcome.rs,store.rs}
phalcom-semantic/src/{data_semantics.rs,enum_semantics.rs,algebraic_semantics.rs}
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/{mod.rs,expr.rs,scope.rs,error.rs}
```

Metadata schema evolution remains P8 unless P2 uncovers a genuine existing-schema correctness dependency.

---

## 10. Decision authority

### FIXED

Do not change:

- Kind vs ValueType distinction;
- definition-owned ordinary value parameters;
- persistent/session-owned StaticValueStore model;
- stale-ID prevention;
- exact semantic operation authorization;
- explicit-frame evaluator;
- path-sensitive execution;
- QueryBudget/Cancellation authority;
- closed-const concrete reduction;
- no initializer replay;
- M2 primitive materialization scope;
- P8 structural/artifact materialization boundary;
- exact/unbounded Int;
- Float bit preservation + separate semantic equality;
- content-based constant fingerprints;
- current-vs-LKG compiler truth rule.

### MECHANICALLY FLEXIBLE

May adapt:

- helper/module names;
- private arena split;
- exact internal ID wrappers;
- descriptor store placement;
- frame/continuation representation;
- parser lookahead/backtracking details;
- diagnostic code names consistent with repository convention;
- bounded temporary compatibility typedefs during AST rename;
- test file placement.

### VERIFY-FIRST

| Assumption | Verify | If false |
|---|---|---|
| P1 is administratively complete | checkpoint/final P1 records | do not begin P2 |
| exact callable identity is still published | checker analysis/call path | locate replacement; consult if absent |
| session still owns persistent TypeStore | session/snapshot | mirror live owner; consult on lifetime change |
| current/LKG remain distinct | db state/query | adapt; stale compiler value remains forbidden |
| `num-bigint` remains workspace dependency | Cargo.toml | add lower dependency; never semantic→core |
| chunk constants remain GC roots | heap trace/tests | block M2 LargeInt until fixed |
| operator dispatch still uses canonical callable application | checker | adapt to live exact target product |
| alias corpus permits hard grammar cutover | T0 census | bounded migration compatibility only |
| artifact static metadata is not yet present | metadata | keep artifact-only P2 fail-closed |


---

## 11. Global STOP / CONSULT triggers

Stop editing and prepare a consultation packet if any of the following occurs:

1. P1 is still active/incomplete.
2. `T: Type` vs `N: Int` cannot be represented without globally changing C3 generic identity.
3. Generalized `type` syntax requires a second semantic expression language rather than a bounded syntax bridge.
4. Ordinary static arguments require fake `TypeData`.
5. Published StaticValue IDs cannot be proven safe across snapshot/query reuse.
6. Static operation eligibility would require selector/operator-spelling authority.
7. Canonical core primitive callable identities are unavailable/unstable.
8. Required P2 behavior would need arbitrary runtime/native method execution.
9. Required evaluation would need host-recursive guest computation.
10. Result-domain validation would require a second relation engine.
11. BigInt operations cannot be bounded before potentially pathological allocation.
12. M2 LargeInt materialization cannot be proven rooted.
13. Compiler lowering would need to reevaluate initializer source.
14. Current/LKG behavior would permit stale values as compiler truth.
15. Static structural values require arbitrary user constructor execution.
16. A P3–P8 feature becomes necessary for P2 coherence rather than convenience.
17. Two viable solutions differ materially in identity/lifetime/serialization architecture.
18. A test oracle contradicts a ratified C4 invariant.
19. The same nontrivial semantic failure persists after one serious architecture-consistent correction.
20. Passing tests would require weakening an invariant in this plan.

A triggered consultation is mandatory.

---

## 12. Debugging and verification budget

### 12.1 Mechanical failures

For imports, renamed enums, local type errors, parser exhaustiveness and harness placement:

1. inspect the nearest live convention;
2. make one direct correction;
3. continue if architecture is unchanged.

### 12.2 Semantic failures

For wrong domain, operation identity, evaluation result, cycle, materialization or invalidation:

1. reproduce with the smallest focused test;
2. identify the authority producing the wrong fact;
3. make one architecture-consistent correction;
4. rerun the focused test;
5. consult if it persists or reveals competing authority.

Do not accumulate evaluator special cases.

### 12.3 Testing modes

**BUILD MODE** — smallest focused selector:

```sh
cargo test -p phalcom-ast <focused>
cargo test -p phalcom-semantic --test semantic <focused>
cargo test -p phalcom-core --test core <focused>
cargo check -p phalcom-semantic
cargo check -p phalcom-core
```

**STABILIZE MODE** — coherent P2 suites after a vertical lands.

**CERTIFY MODE** — affected package suites at G3; do not run workspace-wide tests after every task.

Designed coverage is not executed evidence.

---

## 13. Baseline / unrelated failure policy

Use the repository classification:

```text
A — definitely caused by current change
B — probably caused by current change
C — unclear
D — clearly pre-existing/unrelated
```

A/B are P2 responsibility. C gets one bounded classification pass. D is recorded and deferred.

Inherited P1 baseline issues do not become P2 scope merely because a broad gate encounters them.

---

## 14. Implementation sequence

```text
P1 administrative closure
        │
        ▼
T0  takeover + migration/source census
        │
        ▼
T1  shared static domain model
        │
        ▼
T2  generalized type-definition AST/grammar
        │
        ▼
T3  StaticDefinition signatures/products + alias projection
        │
        ├──────────────┐
        ▼              │
T4  StaticValueStore  │
        │              │
        ▼              │
T5  structural values │
        │              │
        └───────┬──────┘
                ▼
T6  checked static bodies + exact static intrinsics
                │
                ▼
T7  explicit-frame evaluator
                │
                ▼
T8  StaticConstant queries/dependencies/cycles
                │
                ▼
T9  descriptors + M2 primitive materialization
                │
                ▼
G1  closed evaluator/materialization vertical
                │
                ▼
T10 resource/incremental/performance stabilization
                │
                ▼
G2  C1–C3 compatibility
                │
                ▼
G3  affected-crate stabilization
                │
                ▼
T11 checkpoint/walkthrough/handoff
```

---

# T0 — Establish P2 takeover state and freeze migration census

## Purpose

Prove P1 closure, freeze actual source baseline, and quantify the `type` grammar/AST migration before edits.

## Preconditions

- final P1 walkthrough/handoff exists;
- checkpoint marks P1 complete and P2 active.

## Consumes

```text
C4 checkpoint
P1 walkthrough/handoff
P2 requirements analysis
current main
```

## Produces

- exact P2 start SHA;
- P1 closure evidence;
- source census for old result-domain ordering;
- census for direct `TypeAliasDef` construction/exhaustive matches;
- stale `= { ... }` source census;
- exact affected-crate candidate list;
- checkpoint P2 start entry.

## Expected reads

```text
docs/implementation/LANG007/LANG007.C4/**
phalcom-ast/src/{ast.rs,parser.rs}
phalcom-semantic/src/{semantic_shard.rs,type_alias.rs,session.rs}
```

## Required procedure

Run bounded searches equivalent to:

```sh
rg -n '^\s*(native\s+|foreign\s+)?type\s+' --glob '*.ph'
rg -n 'TypeAliasDef|Statement::TypeAlias|alias_sources' phalcom-* docs/implementation/LANG007
rg -n '=\s*\{' --glob '*.ph'
```

Record:

- uses of historical result-before-generics syntax;
- stale block-bodied `= {` forms;
- generated/compiler AST constructors;
- all public exhaustive matches requiring mechanical rename;
- any live P2 code not reflected in the supplied analysis.

## Forbidden

- source edits while P1 remains open;
- broad codemod before census;
- fixing P1 D failures;
- treating mechanical AST rename fallout as architecture.

## Verification now

No broad suite. Run only enough parser/semantic baseline probes to confirm the observed grammar if records are insufficient.

## Acceptance

- P1 administrative gate passes;
- exact SHA recorded;
- migration scope known;
- no unaccounted architecture drift.

## STOP if

- P1 remains open;
- corpus usage makes this plan's cutover policy materially wrong;
- a competing live static-definition implementation exists.

## Checkpoint update

Mark P2 active/start SHA only after P1 closure.

---

# T1 — Introduce one shared static-domain model

## Purpose

Establish the Kind-vs-ordinary-value distinction before generalized `type` checking.

## Consumes

```text
KindId / KindData::StaticDomain
TypeId / ProperTypeId
type annotation resolution
type-member domain syntax
canonical declaration tables
```

## Produces

```text
StaticDomainSyntax
StaticValueDomain
resolve_static_domain(...)
proper-Type/domain validation helpers
type-member adapter seam
```

## Expected write set

```text
phalcom-ast/src/ast.rs
phalcom-semantic/src/static_domain.rs              # preferred new module
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/lib.rs
```

## Required implementation

### T1.1 Source carrier

Add a neutral source domain node. It must represent the syntactic form without prematurely deciding `Kind` vs `ValueType`.

### T1.2 Semantic resolution

Rules:

```text
canonical Kind/static-domain declaration
    -> StaticValueDomain::Kind(kind)

proper ordinary Telos type
    -> StaticValueDomain::ValueType(type_id)

invalid/non-proper/non-domain form
    -> normal semantic failure/diagnostic
```

### T1.3 Proper Type

Result domain `Type` is a meta-level Type term requirement. Do not accept any arbitrary `TypeId`-backed static term.

### T1.4 Type-member convergence seam

If broadening semantic `TypeMemberDomain` to ordinary value domains would pull P5 work forward, keep current behavior but route it through a common conversion/helper and document the adapter. Do not create another long-lived domain enum.

## Forbidden

- fake `KindId` for `Int`;
- treating `T: Type` as ordinary value type `Type`;
- evaluator-local type/domain checking;
- independent static-definition/type-member domain resolvers.

## Required tests

```text
Type -> Kind(Type)
TypePack -> Kind(TypePack)
Shape -> Kind(static domain)
Int -> ValueType(Int)
Bool -> ValueType(Bool)
String -> ValueType(String)
Symbol -> ValueType(Symbol)
invalid domain
non-proper ValueType rejection
```

Also run the smallest existing type-member static-domain regression.

## Acceptance

- one shared semantic distinction exists;
- C3 kind behavior remains unchanged;
- ordinary value domains resolve without TypeParameter redesign.

## STOP if

- source domain cannot be disambiguated without changing source syntax;
- global kind semantics would have to change.

## Checkpoint update

Record `StaticValueDomain` as a durable P2 seam after focused evidence.

---

# T2 — Generalize `type` AST/grammar and static application syntax

## Purpose

Replace alias-only syntax representation with the C4 static-definition source model while retaining ordinary legacy aliases.

## Consumes

```text
TypeAliasDef
GenericParameterSyntax
TypeAnnotation
LexicalBlock
type alias parser
source index traversal
```

## Produces

```text
TypeDefinitionDef
StaticParameterSyntax
StaticDefinitionBody
canonical grammar order
legacy alias compatibility lane
domain-neutral static application arguments
generalized source traversal
```

## Expected write set

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast parser tests
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/source_index/{builder.rs,occurrence.rs,...}
```

plus mechanical exhaustive-match consumers found by T0.

## Required implementation

### T2.1 Rename/generalize AST

Preferred final state:

```rust
Statement::TypeDefinition(TypeDefinitionDef)
```

Do not retain two independently populated source authorities.

### T2.2 Parameters

Use `StaticParameterSyntax` with domain-neutral `StaticDomainSyntax`, preserving source order and modes.

### T2.3 Canonical grammar

Accept:

```telos
type F<T>: Result where ... = expression
type F<T>: Result where ... { lexical statements }
```

not the old result-before-generics ordering.

Hard cut over if T0 shows migration is bounded. If temporary compatibility is genuinely required, bound it and record removal before C4 closure; never maintain indefinite dual grammar.

### T2.4 Bodies

Expression body is `= ...`.

Block body is `LexicalBlock` with **no `=`**.

Do not use first-class Closure AST as the block's meaning.

### T2.5 Legacy aliases

Keep:

```telos
type Optional<T> = T | None
```

with implicit `Kind(Type)` result.

Reuse the existing type-form parser as an optimized source path and wrap it into the generalized static expression/body representation rather than maintaining alias-only semantic authority.

### T2.6 Type-form / ordinary-expression bridge

Static definitions must permit both C3 type/static-form syntax and ordinary Telos expressions/control.

Use one explicit bridge, conceptually:

```rust
Expr::StaticForm(TypeAnnotation)
```

or a repository-native equivalent. Parser internals may use bounded lookahead/backtracking; semantics must converge before checking/evaluation.

If doing this correctly requires a second expression language, consult.

### T2.7 Static arguments

Establish a source carrier that can represent:

```telos
IsZero<3>
Choose<true, Int, String>
```

and later Symbol-valued arguments without an ABI redesign.

Do not force all arguments into `TypeAnnotation`. Do not force all kinded/type arguments into runtime expression semantics. Ambiguous atomics are resolved against the target parameter domain.

### T2.8 Where clauses

Existing C3 generic constraints remain valid for kinded parameters. P2 does not invent value predicates. If a value parameter appears in an unsupported existing `where` relation, diagnose it rather than adding a second constraint engine.

### T2.9 Source indexing

Traverse parameters, result domain, expression/block bodies, locals, calls/applications and associated references.

## Forbidden

- Closure body semantics;
- permanent old/new grammar orders;
- second static semantic expression language;
- fake type arguments for ordinary values;
- general static destructuring in P2.

## Required parser/AST tests

```text
legacy alias
legacy generic alias
canonical generics-before-result order
N: Int
T: Type
S: Shape
mixed parameters
expression body
block body
block without =
stale = { policy
where placement
ordinary static arguments 3/true/string/symbol carrier
kinded arguments
LexicalBlock vs Closure
invalid/malformed recovery
```

## Verification now

Focused parser tests, then `cargo check -p phalcom-semantic` after mechanical exhaustive-match updates.

## Acceptance

- all canonical forms parse;
- old ordinary aliases remain legal;
- one generalized AST authority exists;
- ordinary-value static arguments are not Int-only.

## STOP if

- parser requires a parallel Pratt/type expression engine;
- ambiguity cannot be carried/resolved without leaking two semantic systems.

## Checkpoint update

Record canonical `type` grammar when focused parser gate passes.

---

# T3 — Publish StaticDefinition signatures/products and preserve TypeAliasInfo

## Purpose

Make generalized `type` declarations first-class semantic products with mixed kinded/value parameters while preserving C3 alias behavior.

## Consumes

```text
DeclarationId
GenericSignature / TypeParameterData
TypeAliasInfo / TypeAliasTable
source shards
SemanticDb
StaticValueDomain
```

## Produces

```text
StaticDefinitionId
StaticParameterId
StaticParameter
StaticDefinitionSignature
StaticDefinition
QueryKey::StaticDefinition
SemanticProduct::StaticDefinition
StaticDefinition dependency/fingerprint
TypeAliasInfo projection for compatible Type definitions
```

## Expected write set

```text
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/type_alias.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/db/{key.rs,product.rs,query.rs,fingerprint.rs}
phalcom-semantic/src/types/{annotation.rs,parameter.rs}
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/lib.rs
```

## Required implementation

### T3.1 Identity

Underlying definition identity remains `DeclarationId`.

### T3.2 Parameter publication

For each source parameter:

1. resolve `StaticValueDomain`;
2. assign stable definition-local index;
3. allocate/reuse `TypeParameterId` only for the C3 kinded lane;
4. leave ordinary value parameters outside TypeStore parameter identity;
5. preserve mode/order.

### T3.3 Mixed signatures

Prove:

```telos
type Select<N: Int, T: Type, S: Shape>: Type = ...
```

publishes one ordered signature while only the kinded lane bridges to C3 parameter machinery.

### T3.4 Result domain

Explicit source domain resolves through T1.

No explicit result on legacy alias => `Kind(Type)`.

### T3.5 TypeAliasInfo

For a definition whose result is Type and body fits existing C3 alias machinery:

```text
StaticDefinition -> TypeAliasInfo optimized compatibility projection
```

Preserve:

```text
TypeId normalization
generic alias substitution
type lambdas
kind checking
alias cycles
metadata-facing alias behavior
```

General value-valued definitions do not fabricate TypeAliasInfo.

### T3.6 DB product

Add typed query/product and semantic fingerprint. Do not evaluate an open generic definition merely to publish it.

### T3.7 Source retention

Generalize `alias_sources` into static-definition source retention and targeted fingerprints without losing fine-grained invalidation.

## Forbidden

- globally adding StaticValue to TypeParameterData;
- independently checking/publishing alias and static-definition authorities;
- definition identity from source range/body hash;
- evaluating generic bodies at definition publication.

## Required tests

```text
T: Type retains TypeParameterId
N: Int has no fake TypeParameterId
Flag: Bool ordinary lane
S: Shape kind lane
mixed signature order
invalid domain
wrong result domain
legacy alias canonical Type unchanged
legacy generic alias substitution unchanged
alias cycle behavior unchanged
type lambda behavior unchanged
unrelated edit reuses definition product
```

## Acceptance

- StaticDefinition is canonical generalized definition product;
- C3 alias behavior remains stable;
- ordinary value parameters coexist without generic-system mutation.

## STOP if

- TypeAliasInfo can only be preserved by duplicating semantic checking;
- mixed parameters require global trait/generic inference redesign.

## Checkpoint update

Record StaticDefinition/StaticParameter takeover interfaces.

---

# T4 — Implement snapshot-safe StaticValueStore and primitive values

## Purpose

Create semantic static values with correct lifetime and exact primitive semantics.

## Consumes

```text
SemanticWorkspaceSession/Snapshot lifetime
TypeStore IDs
num-bigint
numeric language contract
```

## Produces

```text
StaticValueStoreId
StaticValueId
StaticValueRef
StaticValueStore
StaticValueData primitives
StaticInt
StaticFloat
StaticStringId
StaticSymbolId
semantic equality/hash helpers
content fingerprint helpers
```

## Expected write set

```text
phalcom-semantic/Cargo.toml
phalcom-semantic/src/static_value/{mod.rs,store.rs,numeric.rs,...}
phalcom-semantic/src/session.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/lib.rs
```

## Required implementation

### T4.1 Lifetime

Add session-owned and snapshot-published `Arc<StaticValueStore>`.

Mutation follows the TypeStore clone-on-write pattern while preserving lineage ID and prior prefix.

### T4.2 Published refs

Reusable products use store-qualified `StaticValueRef`. Store mismatch is explicit internal failure, not reinterpretation.

### T4.3 Interning

Hash-cons immutable nodes where useful. Internal sharing is an optimization, never language-visible identity.

### T4.4 Int

Small/BigInt exact normalization. No permanent i64 ceiling. No semantic→core dependency.

### T4.5 Float

Preserve exact bits. Implement representation identity separately from Telos equality/hash.

Required semantics:

```text
NaN != NaN
+0.0 == -0.0
signed-zero materialization bits preserved
```

### T4.6 String / Symbol

Immutable content/spelling semantics; no VM object/interner authority.

### T4.7 Type bridge

Add proper `Type(TypeId)` and other `StaticTerm(TypeId)` with validation.

Implemented API adaptation: `domain_of(ref, &TypeStore) -> Result<StaticValueDomain, _>`.
The canonical value domain stores `TypeId`/`KindId`, so resolving primitive
domains requires the paired TypeStore; this keeps built-in type identity in its
existing authority instead of inventing a second registry. `intern_type` and
`intern_static_term` likewise return checked results for invalid kind admission.

## Forbidden

- per-query arenas referenced by DB products;
- naked reusable raw IDs;
- runtime LargeInt/Value reuse;
- derived bitwise Float Eq as language Eq;
- VM Symbol reuse.

## Required tests

```text
Unit/Bool
small and >i64 exact Int
negative large Int
Float finite/NaN payload/+0/-0/infinity
String UTF-8 content semantics
Symbol spelling semantics
Type proper validation
StaticTerm distinction
store clone preserves refs
unrelated store rejects foreign refs
append preserves previous IDs
```

## Acceptance

- snapshot/query reuse cannot stale-alias values;
- primitive semantics match Telos;
- no runtime identity leaks.

## STOP if

- TypeStore-like COW cannot safely preserve value lineage;
- BigInt dependency creates a forbidden cycle;
- Float language hash policy is insufficiently specified.

## Checkpoint update

Record StaticValueStore lifetime and published reference model.

---

# T5 — Add structural StaticValues and value-domain typing

## Purpose

Complete P2 Tuple/Record/Data/Variant semantic values and a single ordinary-value type classification path.

## Consumes

```text
StaticValueStore
TypeStore
Data semantic products
Enum/Variant identities
Tuple/Record type formation
```

## Produces

```text
StaticTupleId
StaticRecordId
StaticDataValueId
StaticVariantValueId
static_value_type(...)
structural equality/fingerprint
pure structural constructors
```

## Expected write set

```text
phalcom-semantic/src/static_value/**
phalcom-semantic/src/{data_semantics.rs,enum_semantics.rs,algebraic_semantics.rs}
phalcom-semantic/src/types/store.rs      # only canonical lookup helpers if necessary
```

## Required implementation

- Tuple stores ordered child refs.
- Record uses current P2 key semantics; P4 owns FieldKey.
- Data retains exact declaration/constructor identity; no constructor execution.
- Variant retains exact `VariantId`; never resolves identity by name.
- Nodes are immutable DAGs; no mutable backpatch API that can create containment cycles.
- `static_value_type(...)` derives ordinary Telos type using existing TypeStore/data/enum semantics.
- Result-domain checks call existing relation/assignability machinery.

## Forbidden

- runtime ProductStorage pointers;
- recursive cons structures/deep-copy values;
- variant-name identity;
- arbitrary class constructor execution;
- preemptive FieldKey implementation.

## Required tests

```text
Tuple order/equality/fingerprint
Record order/current-key semantics
nested/shared structural DAG
Data canonical component mapping
Variant exact ID with repeated-name family cases
wrong arity rejection
acyclic construction
ordinary static value type derivation
ValueType result mismatch uses existing relation engine
```

## Acceptance

- every P2 value category has semantic representation;
- P3/P4/P5 can extend without replacement;
- no second relation/type system appears.

## STOP if

- structural construction requires user code;
- current Record model cannot be represented without P4.

## Checkpoint update

Record DAG/exact VariantId invariants. T5 execution record (2026-10-04):

- **COMPLETE / IMPLEMENTED / FOCUSED_TESTED**.
- Added immutable store-owned Tuple, P2 string-keyed Record, Data, and Variant
  nodes. Constructors accept only existing same-lineage children.
- Data payload order is canonicalized through `DataComponentId`; enum values
  retain and validate exact `VariantId` and exact-case type identity.
- `static_value_type` derives structural Tuple/Record types through `TypeStore`
  and checks canonical Data/Enum products. `check_value_type` calls existing
  assignability machinery.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_structural -- --nocapture`
  selected 1 test and passed.
- The paired-TypeStore-only `domain_of` method reports that structural values
  need semantic products. Structural type-domain callers must use
  `static_value_type` with Data/Enum tables; no second type authority was added.
- Next task: T6 (ordinary checker-backed static body and exact intrinsic
  authorization). P2 remains IN_PROGRESS / PARTIAL / UNVERIFIED.


---

# T6 — Check static bodies and publish exact StaticOperationResolution

## Purpose

Reuse ordinary semantic checking to create evaluator-ready body facts and authorize only exact compiler-owned static operations.

## Preconditions

T3–T5 accepted.

## Consumes

```text
ExpressionAnalysis
CallableId
canonical call application
operator/reflected dispatch
CoreDeclarationIds / Universe identities
BindingId
StaticDefinitionSignature
StaticValueDomain
```

## Produces

```text
CheckedStaticBody
CheckedStaticBodyProduct (separate SemanticDb query)
ModuleStaticBindingIndex (ordinary-module-checker handoff)
StaticBodyId
StaticOperationResolution
StaticOperationResolutionIndex
StaticIntrinsicRegistry
static parameter/local binding maps
phase/result-domain diagnostics
```

## Expected write set

```text
phalcom-semantic/src/static_intrinsic.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/checker/static_body.rs        # or equivalent
phalcom-semantic/src/db/{key.rs,product.rs,query.rs,fingerprint.rs,mod.rs}
phalcom-semantic/src/checker/{analysis.rs,body.rs,statement.rs,expression.rs,context.rs,call.rs}
phalcom-semantic/src/core_surface/identity.rs       # registry bootstrap helpers only if needed
phalcom-semantic/src/diagnostic.rs
```

## Required implementation

### T6.1 Ordinary checker first

Static definitions/const bodies use the normal semantic checker with a static-phase environment.

Reuse:

```text
type resolution
call resolution
direct/reflected operator resolution
assignability
binding identity
diagnostics
```

### T6.1a Checked-body query ownership — implementation amendment

The live session resolves and queries `StaticDefinitionProduct` before the
canonical dispatch surfaces needed to check static bodies are available. That
query's input fingerprint is currently declaration-only, it records no body
semantic dependencies, and reuse returns the early product. Do not enrich and
query that same key a second time.

Give checked bodies a distinct query identity (`StaticBodyId`, backed by the
definition's `DeclarationId`) and a distinct `CheckedStaticBodyProduct` in
`SemanticDb`. Its input fingerprint covers the source body, signature/result
domain, and phase-readable outer binding inputs. The ordinary checker supplies
precise semantic dependencies; the output fingerprint covers exact selected
callables, binding/slot mappings, operation resolutions, result facts, and
diagnostics. Invalidate/recompute this product when any such input or
dependency changes.

### T6.1b Static-constant binding handoff — implementation amendment

Static body checking must not reconstruct module binding names/types. The
ordinary module checker currently retains `BindingAnalysisIndex` on `<main>`,
but that state alone does not mark which immutable bindings are static
constants, and its `BindingId` is body-local. Publish an explicit
`ModuleStaticBindingIndex` alongside the ordinary module analysis. It maps
stable module binding identity (`SymbolId`, or an equivalent `StaticConstantId`)
to checked binding facts qualified by the owning `<main>` body, type/contract,
source site, and a declaration-time phase category (`StaticConstant` versus
runtime binding).

Static-body setup consumes only this checker-produced product, seeds
body-local environment bindings, and keeps their association to stable
constant identities. It records which constants each body reads. The
checked-body query fingerprints the consumed module-binding product and
depends on it, so a changed constant declaration/type invalidates the body.
T8 consumes the same product and the body's recorded stable constant reads
when building `StaticConstant` queries; it must not resolve names or checked
types again. Determine imports/aliases through the existing linked module
identity products, not by spelling-based fallback.

The snapshot-facing `StaticDefinitionProduct` remains the single public
definition view and references the checked-body product after body checking.
The checked-body query is the sole cached owner of checker facts; the snapshot
view only composes it with the already-resolved signature/alias projection.
Do not publish the early definition query as if its body were checked. This
amendment preserves §5.4's canonical checked-definition contract while making
the query lifecycle match live session ordering.

### T6.2 Static readability

The checker consumes `ModuleStaticBindingIndex` facts published by ordinary
module checking. It seeds fresh body-local binding identities for static
constants, retains their mapping to stable `SymbolId`/`StaticConstantId`, and
distinguishes static-readable identities from runtime-only bindings.

A reference to runtime `let`, `let mut`, method parameter, `self`, field,
runtime global or runtime closure capture is a phase error even when optimizer
knowledge could prove a value constant. Static-constant references are recorded
by stable identity for T8 dependency construction; imports/aliases follow
canonical linked identities.

### T6.3 Exact registry

Build the `StaticIntrinsicRegistry` from canonical Universe semantic identity. Selector text may help bootstrap canonical lookup, but the published/evaluator authority is exact `CallableId`/canonical structural operation identity.

### T6.4 Per-expression resolution

Publish conceptually:

```rust
pub struct StaticOperationResolution {
    pub expression: ExpressionId,
    pub callable: CallableId,
    pub operation: StaticIntrinsicOperation,
}
```

For a pure structural operation that is not modeled as a callable, use a distinct canonical structural-operation identity.

### T6.5 User overload negative path

If ordinary dispatch selects a user overload of `+`, the expression may remain ordinarily well-typed, but it receives no authorized static intrinsic. Static evaluation fails only if that path executes.

### T6.6 Reflected binary dispatch

Record the exact target selected by ordinary semantics. Do not assume the direct-side primitive from the source operator.

### T6.7 Result domains

Check body results using existing systems:

```text
Kind(...)      -> existing kind/C3 static-term validation
ValueType(...) -> existing ordinary type assignability/relation
```

The evaluator does not repeat the proof.

### T6.8 Locals

Publish binding IDs and compact evaluator slot indices once. P2 local binding patterns are name-only.

## Forbidden

- token-to-arithmetic interpretation without exact callable identity;
- selector/class-name string authorization;
- “all native methods are const-safe” rules;
- eager whole-body static-eligibility rejection;
- a second assignability checker;
- evaluator string-name lookup.

## Critical tests

```text
canonical core Int + is authorized
user overloaded + is not authorized
same selector spelling gives no authority
same rendered class name gives no authority
reflected exact target honored
non-whitelisted native method rejected
ordinary user method rejected
runtime binding dependency rejected
static parameter read allowed
static const read allowed
Type/static term allowed
wrong result domain diagnosed
untaken unsupported call remains checkable
```

## Verification now

Focused call/operator/static-body tests only.

## Acceptance

- evaluator-ready body facts exist;
- exact negative overload tests prove spelling is not authority;
- static phase and result-domain facts come from semantic checking.

## STOP if

- exact selected call identity is lost before body publication;
- core static-safe operations cannot be identified without trusting names/spelling.

## Checkpoint update

Record exact semantic static-operation authorization as a durable invariant.
T6 has started; the current implementation slice is not task completion:

- `StaticIntrinsicRegistry` is private to additions and keyed by complete
  `CallableId`; its initial `NumberAdd` entry is bootstrapped from the exact
  canonical Universe `Number.+(_)` native surface row.
- `resolve_ready_expressions` is crate-private, consumes ordinary
  `ExpressionAnalysis`, and projects only ready entries whose exact selected
  callable is allowlisted.
- Focused integration test `foundations::static_intrinsics` passed 1/1,
  covering exact core identity, same-spelling user rejection, and another
  native owner rejection. Focused unit test
  `static_intrinsic::tests::ready_analysis_projection_keeps_exact_callable_and_skips_invalid_sites`
  passed 1/1, covering exact target preservation and invalid-expression
  suppression. A focused `foundations::binding_init::read_after_assignment_is_proven`
  test passed 1/1 after expression analysis began retaining the exact local
  `BindingId`; the focused incremental fingerprint regression passed 1/1 for
  identity-sensitive product hashing; `cargo check -p phalcom-semantic`
  completed.
- **T6 remains IN_PROGRESS / PARTIAL**. The exact registry and standalone
  ordinary-checker body analyzer exist. `ModuleStaticBindingIndex` now carries
  final checked main-body facts, linked import aliases follow canonical
  `SymbolId`s, and a new analyzer entry point consumes them under fresh local
  identities while recording stable constant reads. A distinct checked-body
  query, query key/product, input/output fingerprints, semantic dependency
  edges, and final session composition are now implemented. Snapshot-facing
  definition products reference the checked-body result while the early
  definition query remains definition-only. These latest changes are
  implementation-only and unverified; focused query, invalidation, import,
  phase, and result-domain regressions remain. P2 remains IN_PROGRESS /
  PARTIAL / UNVERIFIED.

### Consultation and plan amendment — checked-body query lifecycle

The live session resolves `StaticDefinitionProduct` around
`session.rs:3636–3659`, before final canonical checker/dispatch products exist.
`query_static_definition` uses `QueryKey::StaticDefinition`, an
ID-only input fingerprint, and no semantic dependencies; a reused query
returns its earlier product. Therefore enriching that query after the final
checker pass would permit stale body facts and would not invalidate them when
dispatch/type dependencies change.

The architectural consultation recommended a distinct incremental query keyed
by `StaticBodyId`/`DeclarationId`, with precise checker dependencies and an
output fingerprint, then composing that one checked-body result into the
snapshot-facing static-definition product. The decision preserves
`StaticDefinition` as the definition authority and prevents a second computed
body authority. See the checkpoint consultation/amendment ledger for the
evidence and decision.

---

### T6 completion update — checked-body publication

The separate checked-body query, module binding handoff, phase read/write
checks, exact operation projection, and session snapshot composition are
implemented. Focused evidence includes 4 selected
`checker::static_body::tests`, 19 selected `foundations::static_` tests,
and 3 selected `integration::static_definition_bodies` tests. The integration
cases cover same-module and imported const identity plus provider-change
invalidation and body recomputation. T6 is COMPLETE / IMPLEMENTED /
FOCUSED_TESTED. The earlier partial-state note in this section records the
intermediate point before those focused gates completed.

# T7 — Implement bounded explicit-frame StaticEvaluator

## Purpose

Execute the P2 static subset deterministically without VM execution or host-recursive guest computation.

## Preconditions

T4–T6 accepted.

## Consumes

```text
CheckedStaticBody
StaticValueStore
StaticOperationResolution
StaticDefinition products
QueryBudget
CancellationToken
```

## Produces

```text
StaticEvalOutcome
StaticEvalError
StaticCycle
StaticEvaluator
frame/continuation machine
StaticSlot
non-recursive definition application
if / and / or execution
```

## Expected new module tree

```text
phalcom-semantic/src/static_eval/mod.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/frame.rs
phalcom-semantic/src/static_eval/error.rs
```

Exact decomposition is flexible.

## Required implementation

### T7.1 Explicit work machine

Nested expressions/blocks/applications are represented by explicit work items/continuations/frames.

Do not recursively descend guest computation in a way whose Rust call depth grows with guest nesting/recursion.

Small helper recursion over fixed metadata is acceptable only when it cannot scale with guest execution depth.

### T7.2 Budget/cancel checkpoints

At minimum:

```text
before/after executed expression step
before static application frame push
before expensive intrinsic
before structural node creation
periodically within work-proportional traversals
```

Cancellation returns `Cancelled` and cannot publish success.

### T7.3 Local slots

Preallocate indexed slots from checked binding layout.

```text
let      initializer required, immutable after init
let mut  evaluator-local mutable slot
read Uninitialized -> static evaluation error
```

Do not reuse VM `UninitializedBinding`.

### T7.4 Literals

Form StaticValues directly; no VM allocation.

### T7.5 Parameters

Kinded parameters bind canonical C3 static arguments/terms. Ordinary value parameters bind StaticValue refs.

### T7.6 Lexical blocks

Execute source statements in order; trailing expression is the block value.

### T7.7 `if`

Evaluate condition, require static Bool, then execute exactly one branch.

### T7.8 `and` / `or`

Implement lazy Bool control:

```text
and: false skips rhs; true evaluates rhs
or:  true skips rhs; false evaluates rhs
```

Do not lower to eager ordinary binary intrinsic execution.

### T7.9 Intrinsics

The evaluator looks up prepublished `StaticOperationResolution`; it does not resolve selectors/callables.

Missing authorization on an executed operation -> `Invalid(UnsupportedOperation...)`.

### T7.10 Static definition applications

Validate arguments against the already-resolved signature, bind by identity/order and push a frame.

P2 supports nested **non-recursive** applications.

Track active definitions/applications. Any reentry in P2 -> `Recursive`. P7 later refines this into exact application-key cycle detection and changing-argument recursion.

### T7.11 Structural construction

Use T5 pure Tuple/Record/Data/Variant constructors.

### T7.12 Diagnostic trace

Maintain a bounded static evaluation trace of definition/application/source frames. Charge diagnostic note growth through query budget/current policy.

## Forbidden

- bytecode/VM interpreter;
- runtime sends;
- host-recursive guest evaluation;
- string-keyed local environment in hot loop;
- both-branch evaluation;
- missing static intrinsic -> runtime fallback;
- recursion -> budget failure.

## Required tests

```text
literal
nested arithmetic
let
let mut
assignment
immutable local reassignment failure
uninitialized static local read
lexical trailing value
if true/false
nested if
and short-circuit
or short-circuit
untaken forbidden operation succeeds via other branch
taken forbidden operation fails
nested non-recursive static definition application
mixed parameter application
wrong argument domain
recursive self application -> Recursive
mutual static definition recursion -> Recursive
source evaluation order
cancellation
step budget exhaustion
value-node budget exhaustion when enabled
```

## Verification now

Focused evaluator suite only.

## Acceptance

- P2 control/value subset executes without VM/runtime identity;
- path-sensitive positive/negative pair passes;
- Recursive/BudgetExceeded/Cancelled/Invalid remain distinct;
- machine shape is P6/P7-extensible.

## STOP if

- implementation accumulates AST-specific semantic special cases that duplicate checker facts;
- Return/Break/Continue cannot later be added without evaluator replacement.

## Checkpoint update

Record evaluator frame/control seam and exact P2 supported subset.

### T7 progress update — evaluator foundation slice

The new `phalcom-semantic/src/static_eval/` module currently has an explicit
work queue for primitive literals, checked parameter reads, checked local-slot
reads/initialization, ordered expression-only lexical blocks, `if`, lazy
`and`/`or`, and exact-authorized integer `Number.+`. Integer literal
parsing and BigInt addition charge query steps proportionally to input size.
Primitive Symbol values use the shared canonical selector encoder; positional
Tuple and bare-label Record literals use the T5 StaticValueStore constructors.
Evaluator arguments
retain the checked interleaving of kinded TypeId arguments and ordinary
StaticValue arguments; wrong argument counts or kinds fail before execution.
Local declaration sites are joined to checked BindingId facts only when the
checker-provided name and source range identify one unique local binding.

Focused evidence: the checked static-body unit selector passed 4 tests,
including evaluator parameter reduction, local initialization/read, argument
count/kind rejection, budget exhaustion, and cancellation. The session
integration selector passed 3 tests, covering exact-authorized integer
addition, selected `if` branch reduction, paired lazy `and`/`or`
positive/negative paths, and positional Tuple/bare-label Record construction.
The same integration also reduces primitive Symbol literals into the semantic
store, confirms that assignment through the checker-selected mutable BindingId
updates the later read, and constructs positional Tuple and bare-label Record
values through the semantic store.
The semantic crate check completed. This is a partial implementation only.
Static-definition applications/re-entry, Data/Variant construction, diagnostic
traces, and remaining resource-boundary tests are open. Do not count local
mutation or Tuple/Record construction among the remaining T7 work.
Do not advance to T8.

### T7.10 — checked static-application authority and publication resolution

**Decision (2026-10-04):** add a checked static-application product carrying
the exact target declaration/signature/domain and argument identities. T7 must
consume that product; value-parameter applications are not deferred. Preserve
C3 `TypeAliasInfo` as the optimized compatibility projection for ordinary
Type-valued definitions. The product must be produced by ordinary semantic
checking so it agrees with the canonical TypeForm result. Do not add evaluator
name resolution or change static-definition identity.

The checker records `CheckedStaticApplication` under `CheckedStaticBody`, adds
a formal `SemanticDependency::StaticDefinition`, and fingerprints the
denotation. The evaluator consumes this product without source-name lookup.

**Original observation (2026-10-04, resolved):** adding `type Literal: Int = 3` and
`type Applied: Int = Literal<>` to the previously passing
`integration::static_definition_bodies` fixture causes its existing
`ReadBase` lookup to fail because the published snapshot's static-definition
table is empty. Removing the application makes that lookup pass. The focused
selector therefore fails before it can assert the new checked product. The
minimal isolated fixture is non-discriminating because it also produces no
static-definition entries without an application. No application evaluator,
spelling lookup, or corrective workaround has been added.

The focused consultation identified the cause: `TrackingTypeResolver::resolve_type_name`
recorded a `DeclarationShell` edge for value-domain static definitions. Its
replay path accepts declaration/alias products, while these definitions publish
neither. The failed replay prevented checked-body publication and left the
prior empty snapshot. `resolve_static_definition_name` now preserves
`LinkedName` and `PublicExport` edges while recording the canonical
`StaticDefinition` edge without the invalid shell edge. The three-test
`integration::static_definition_bodies` selector passed after this correction.

The evaluator now reduces zero-argument, value-parameter, directly resolved
kinded-parameter, mixed kinded/value, and nested nonrecursive applications
through explicit frames; exact self-reentry returns `Recursive`. The same
focused selector passed 3/3 after adding the mixed-parameter regression. T7.10
publication and these application paths are established; T7 remains open for
kinded-expression applications and remaining resource-boundary tests. Bounded
trace coverage is recorded under T7.12 below.

### T7.11 — canonical checked-constructor projection

The ordinary checker owns the construction decision. `CheckedStaticBody` must
retain body-local associated-resolution facts and publish a compact static
construction projection with exact `InvocationTargetId`, result `TypeId`,
argument expression identities, and stable payload identities. For Data,
convert the checker-produced argument/component mapping to exact
`DataComponentId`s while `DataInfo` is available. For Variant, publish exact
argument/`VariantFieldId` mapping when the accepted syntax can reorder or label
payloads; never infer it from AST labels in the evaluator.

`DataInfo` and `VariantInfo` stay in their canonical semantic tables. Supply
borrowed canonical tables (and the canonical `TypeStore` required by T5's
interners) through the evaluator's semantic environment. The evaluator
consumes checked constructor facts and calls only the existing pure
`StaticValueStore::intern_data` / `intern_variant` APIs; it does not resolve
constructors or duplicate catalog metadata.

Discriminating regressions must include reordered labeled Data payloads whose
stored values follow exact component identity, and an exact Variant identity
test that cannot pass by matching a case name alone. If the ordinary checker
cannot publish exact Variant payload mapping for the accepted syntax, surface
that gap at the checker product before implementing evaluator-side guesses.

**Progress (2026-10-04):** `CheckedStaticBody` now retains body-local ordinary
associated resolutions and a compact constructor product with exact invocation
target, result type, argument expression IDs, and `DataComponentId` or
`VariantFieldId` payload identities. The evaluator receives borrowed canonical
TypeStore/Data/Enum tables through `with_semantic_tables` and calls the T5
interners. The `integration::static_definition_bodies` selector passed 3/3
with a reordered labeled `Point` constructor and two same-named `Same` cases
under different enum owners. The evaluated Point matched canonical component
identity/order, and the two Variant values remained distinct. No source-name or
label fallback is used by the evaluator. T7 remains partial pending
kinded-expression application and resource-boundary acceptance.

### T7.12 — bounded evaluator trace and application cycle/resource evidence

`StaticEvaluator::trace()` exposes the active/terminal definition,
application, and source-expression path. Every trace push charges the existing
`QueryBudget`; cancellation is checked before growth. There is no independent
depth ceiling or new diagnostic budget. The evaluator outcome remains
orthogonal to trace provenance.

Focused `integration::static_definition_bodies` passed 3/3 after adding
mutually recursive A→B→A definitions, application step-budget exhaustion, and
trace assertions for the recursive target/source path. `Recursive`,
`BudgetExceeded`, and `Cancelled` remain separate outcomes; the existing unit
test covers pre-cancelled evaluation. T7 is still partial because kinded
expression applications and remaining resource/value-node boundaries are
open.

### T7.10 plan amendment — kinded-expression result boundary (2026-10-04)

An architectural consultation resolved the evaluator result shape: body
completion and nested-application continuations need a tagged result with
`StaticValue(StaticValueRef)` and `KindedTerm(TypeId)` cases. Keep the ordinary
expression stack value-only. A kinded result may bind only the exact checked
kinded parameter slot; the existing direct checked-`TypeId` argument path is
unchanged. T7 support is limited to checked TypeForm/static-application
results, and unsupported TypeForm computations fail closed. The evaluator
must not resolve names/types or synthesize type products.

The canonical TypeStore is borrowed immutably during evaluation, and general
`TypeView` materialization requires mutable access. T7 therefore accepts a
TypeForm result only when its checker-published canonical `TypeId` is
independent of the callee's kinded parameters, or is exactly one of those
parameter nodes; in the latter case, forward the supplied canonical `TypeId`
unchanged. Reject composite results containing any callee kinded parameter,
because specialization would require mutation/interning. Admit other already
canonical results only when their kind matches the checked result domain. Do
not create a cloned/parallel TypeStore or publish clone-qualified identities.
Focused discriminators cover independent result, exact parameter forwarding,
and composite parameter-dependent rejection. No wider TypeStore ownership
change or TypeForm evaluator is authorized.

---

# T8 — Publish StaticConstant queries, dependencies, fingerprints and cycles

## Purpose

Attach closed static evaluation to canonical `const` identity and the incremental semantic DB.

## Preconditions

T7 accepted.

## Consumes

```text
ModuleBindingSemantics::StaticConstant
SymbolId
const initializer sources
StaticEvaluator
SemanticDb
source shards
current/LKG policy
```

## Produces

```text
StaticConstantId
QueryKey::StaticConstant
SemanticProduct::StaticConstant
StaticConstantProduct
StaticConstant dependency edges
value-content product fingerprints
constant cycle diagnostics
snapshot/query accessors
```

## Expected write set

```text
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/db/{key.rs,product.rs,query.rs,state.rs,fingerprint.rs}
phalcom-semantic/src/checker/analysis.rs         # SemanticDependency additions
phalcom-semantic/src/source_index/**
phalcom-semantic/src/diagnostic.rs
```

## Required implementation

### T8.1 Identity

Use `SymbolId(module,name)` or thin `StaticConstantId(SymbolId)`. Never VM Symbol/compiler slot/source range.

### T8.2 Targeted source input

Retain initializer source and an input fingerprint keyed by static constant identity.

### T8.3 Query flow

Conceptually:

```text
query_static_constant(id)
  -> resolve/check initializer as static body
  -> record semantic dependencies
  -> evaluate under passed QueryBudget/CancellationToken
  -> require concrete closed result
  -> publish value/outcome + semantic fingerprint
```

Descriptor construction can be invoked here after T9 or represented as a later projection field while T8 is being built; final P2 product must expose descriptor/capability.

### T8.4 Const-to-const dependency

For:

```telos
const A = B + 1
```

record:

```text
StaticConstant(A) -> StaticConstant(B)
```

Changing B from 10 to 20 invalidates A even though B remains Int.

### T8.5 Definition dependency

A constant applying static definition F depends on `StaticDefinition(F)`.

### T8.6 Constant cycles

For:

```telos
const A = B
const B = A
```

use semantic query/dependency cycle authority where available. Produce deterministic participants/source provenance.

Do not allow stack overflow or incidental budget exhaustion to be the diagnostic.

### T8.7 Current vs LKG

If previous source reduced successfully and current source is invalid/cancelled/budget-exhausted:

- LKG may remain accessible under editor policy;
- current product is not successful;
- compiler projection sees no current successful constant.

### T8.8 Fingerprint

Fingerprint reduced content, not StaticValueRef ID or type alone.

### T8.9 Local const

Body-local constants may remain evaluator/checker-local rather than receiving global DB query keys. Use owner body identity + BindingId where identity is required.

## Forbidden

- P1 compiler HashSet as semantic identity;
- type-only constant fingerprint;
- StaticValue ID fingerprint;
- stale LKG compile fallback;
- imported runtime global read as a static value.

## Required tests

```text
literal const
computed const
const-to-const dependency
const-to-definition dependency
runtime dependency rejection
self cycle
mutual cycle
10 -> 20 invalidates dependent with same type
unrelated edit reuses unaffected product
cancelled query does not publish success
budget-exhausted query does not publish success
LKG may remain presentation but current compile product is invalid
StaticValueRef remains valid across reused snapshot product
```

## Verification now

Focused static-constant + incremental tests.

## Acceptance

- closed constants are semantic DB products;
- value changes invalidate dependents;
- cycles/cancel/budget are deterministic terminals;
- no stale compiler truth.

## STOP if

- SemanticDb cannot model const cycles without a parallel cache;
- current/LKG API cannot prevent stale compiler use.

## Checkpoint update

Record StaticConstant identity/query/fingerprint seams.

---

# T9 — Add materialization descriptors and M2 primitive runtime vertical

## Purpose

Prove reduced static values can cross into runtime code without source reevaluation while structural/artifact materialization remains P8-owned.

## Preconditions

T8 accepted and constant-pool GC trace path re-verified.

## Consumes

```text
StaticConstantProduct
StaticValueStore
ModuleLoweringSemantics
P1 compiler const barrier
Chunk constants
runtime Value/String/Symbol/BigInt normalization
```

## Produces

```text
MaterializationDescriptor / descriptor store
StaticConstantLoweringSpec
ModuleLoweringSemantics static constant map
primitive compiler materializer
explicit deferred-structural diagnostic
LargeInt GC-rooting regression
```

## Expected write set

```text
phalcom-semantic/src/materialization.rs
phalcom-semantic/src/static_constant.rs

phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/compiler/lib/{mod.rs,expr.rs,scope.rs,error.rs}
phalcom-core/src/chunk.rs                         # helper only if needed
phalcom-core/tests/core/compiler/static_const.rs
phalcom-core/tests/core/execution/static_const.rs
phalcom-core/src/heap/trace.rs                   # read/verify; edit only if a test proves a defect
```

## Required implementation

### T9.1 Pure descriptor conversion

```text
StaticValueRef -> MaterializationDescriptorRef
```

No AST/source execution.

### T9.2 Descriptor breadth

Semantic descriptors should have variants for all P2 values so P8 can add durability/physical materializers without changing semantic meaning.

### T9.3 Capability

Mark whether a descriptor is:

```text
P2PrimitiveMaterializable
DeferredStructural
```

or equivalent.

### T9.4 ModuleLoweringSemantics

Project only **current successful** module constants. Carry canonical constant identity and descriptor/capability. Do not transport initializer AST as compiler authority.

### T9.5 Compiler lookup

On a runtime reference to a static constant:

1. consume semantic lowering attachment;
2. emit/materialize if P2 primitive-supported;
3. internal consistency error if semantic product is unexpectedly missing;
4. explicit deferred-materialization diagnostic for valid P8-owned structural descriptor;
5. never compile initializer source.

### T9.6 Unit / Bool

Immediate runtime values.

### T9.7 Int

Convert semantic exact integer by value:

```text
fits i64 -> Value::int
otherwise -> canonical core BigInt normalization / Object::LargeInt
```

Do not duplicate core small/large normalization policy.

### T9.8 Float

Construct runtime float from exact preserved bits.

### T9.9 String

Allocate/intern runtime string from semantic content at materialization time.

### T9.10 Symbol

Intern runtime VM Symbol from semantic spelling at materialization time.

### T9.11 LargeInt GC rooting

Add a behavioral test:

1. compile source with a large materialized static Int;
2. ensure runtime representation is heap LargeInt;
3. force/trigger collection after compilation but before/use across execution;
4. evaluate the runtime reference;
5. verify exact numeric value survives.

Code inspection is not enough.

### T9.12 No slot / no initializer bytecode

When runtime code references a const, still prove:

- no `DefineGlobal`/runtime local slot for the const;
- no initializer bytecode;
- only reduced materialization/load is emitted.

## Forbidden

- runtime Value embedded in semantic descriptor/product;
- compiler reevaluation;
- hidden const global slot;
- ad hoc Tuple/Record/Data/Variant materialization;
- imported runtime global as static source.

## Required tests

```text
Unit runtime use
Bool runtime use
small Int runtime use
large Int runtime use
Float exact bits incl -0.0/infinity
String content
Symbol spelling
no initializer bytecode
no const runtime slot
repeated reference does not rerun initializer
LargeInt survives GC
structural const can semantically reduce without runtime use
runtime structural const use gets explicit P8-deferred diagnostic
```

## Verification now

Focused compiler static-const tests + LargeInt GC regression.

## Acceptance

- primitive consts can be consumed same-module at runtime;
- initializer is never rerun;
- exactness/GC rooting proven;
- structural runtime use fails explicitly at plan boundary.

## STOP if

- BigInt materialization opens an unrooted lifetime;
- compiler must re-resolve semantic meaning;
- VM interner order leaks back into semantic String/Symbol identity.

## Checkpoint update

Record M2 breadth and P8 deferred materialization boundary.

---

# G1 — Closed evaluator/materialization vertical

## Purpose

Prove the complete P2 flow before non-functional stabilization.

## Required definition cases

```telos
type IsZero<N: Int>: Bool =
    N == 0

type Choose<Flag: Bool, A: Type, B: Type>: Type =
    if Flag { A } else { B }
```

## Required const cases

```telos
const Answer = 40 + 2
const Enabled = true
const Big = <integer beyond i64>
const Message = "hello"
const Key = #name
```

## Required dependency case

```telos
const A = 10
const B = A + 2
```

## Required path-sensitivity pair

```telos
const Good =
    if false { runtimeOnly() } else { 1 }
```

must reduce, while the taken forbidden-operation counterpart must fail.

## Required soundness case

A user overload of `+` with matching selector spelling must not execute statically.

## Required runtime use

Runtime code reads a primitive reduced const successfully with no initializer bytecode and no runtime const slot.

## Gate result

All P2-owned cases pass; P8-deferred structural runtime use fails with the explicit bounded diagnostic. A failure is not permission to broaden P2.

---

# T10 — Resource, incrementality and performance stabilization

## Purpose

Close memory/resource/incremental risks after the semantic vertical works.

## Preconditions

G1 passes.

## Produces

- final QueryBudget charging rules;
- StaticValue node-growth budget if needed;
- BigInt adversarial preflight;
- incremental reuse evidence;
- evaluator memory/performance evidence;
- final affected-crate list for G3.

## Required work

### T10.1 Query budget coverage

Every evaluator work path charges existing query steps. If required, add a shared budget dimension:

```rust
BudgetKind::StaticValueNodes
```

plus corresponding counters/limits.

Do not add evaluator-private hidden limits.

### T10.2 BigInt preflight

Focused cases:

```text
huge shift
huge exponent
rapid repeated multiplication
invalid/negative shift semantics according to Telos
```

Use overflow-safe result-growth estimation and charge before unreasonable allocation.

### T10.3 StaticValue memory

Verify:

```text
structural sharing
child IDs instead of deep recursive ownership
append-only store lineage
string/symbol interning or equivalent dedup
no deep clone per intrinsic
```

### T10.4 Evaluator memory

Verify:

```text
indexed locals
explicit frame vector
no body AST clone per application
no TypeStore clone per application
bounded diagnostic trace
```

### T10.5 Incremental scenarios

Required edits:

1. unrelated module edit -> unaffected definitions/constants reused;
2. static definition body edit -> dependent const invalidated;
3. constant 10 -> 20 with same type -> dependent invalidated;
4. non-semantic range/comment change avoids avoidable recompute where existing DB policy supports it;
5. cancellation -> no current success, optional LKG presentation only;
6. reused product StaticValueRef dereferences identically after snapshot update.

### T10.6 Complexity targets

```text
StaticValue lookup               O(1)
Tuple child access               O(1)
local read/write                 O(1)
frame push/pop                   O(1) amortized
structural construction          O(n) children
structural fingerprint/equality  O(n) reachable content, memoizable
String/Symbol interner lookup    expected O(1)
```

No repeated whole-body/name scans in evaluator hot paths.

## Forbidden

- hidden evaluator caps;
- incomplete outcome cached as success;
- exposing arena identity as semantic equality to gain speed.

## Verification now

Focused incremental/resource/adversarial tests. Run existing semantic allocation/query probes if available; do not invent unstable wall-clock gates.

## Acceptance

- obvious adversarial BigInt host-OOM routes are bounded by query authority;
- store lineage and reuse tests pass;
- evaluator uses indexed/explicit structures;
- current/LKG behavior proven.

## STOP if

- resource bounds require changing Int semantics rather than evaluation resources;
- memory bounding conflicts with incremental ID safety.

## Checkpoint update

Record resource/accounting and incremental evidence.

---

# G2 — C1–C3 compatibility gate

Prove P2 did not corrupt predecessor static/type semantics.

Required coverage:

```text
legacy type aliases
generic aliases
type lambdas
Kind / StaticDomain
type data / static cases
StaticArgument
TypePack / CompleteTypePack internals
RecordRow
static type members
associated static domains
generic substitution/inference
alias cycles
metadata paths that should remain unchanged
canonical call/operator resolution tests
```

Expected result: no C1–C3 semantic regressions.

Do not serialize new P2 StaticValues into type metadata merely to appease a test; P8 owns durable static metadata.

---

# G3 — Affected-crate stabilization

After focused gates, run package suites based on actual touch set.

Minimum expected:

```sh
cargo test -p phalcom-ast
cargo test -p phalcom-semantic
cargo test -p phalcom-modules
cargo test -p phalcom-core
```

Also run when touched/transitively exposed:

```sh
cargo test -p phalcom-type-meta
cargo test -p phalcom-lsp
cargo test -p phalcom-repl
```

Run serially if required by repository/P1 gate convention.

Classify failures A/B/C/D. Do not repair D. Do not claim the gate passes while a P2-caused failure remains.

---

# T11 — Completion records and P3/P4 handoff

## Preconditions

- G1 passed;
- T10 accepted;
- G2 complete;
- G3 complete with residuals classified.

## Produce

```text
LANG007.C4-CHECKPOINT.md updated
LANG007.C4.P2-walkthrough.md
LANG007.C4.P2-handoff.md
```

## Checkpoint truth

Only if actual work is complete:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED | RELEASE_COMPLETE
```

Use `RELEASE_COMPLETE` only if its broad release-level evidence actually ran.

Do not close C4; P3–P9 remain.

Set next plan according to checkpoint dependency state. P3/P4 may proceed in parallel only if the P2 shared static-value/domain seams are genuinely stable.

## Walkthrough must record

1. actual start/final revision;
2. architecture actually landed;
3. changed components;
4. final StaticValueDomain/parameter model;
5. actual StaticValueStore lifetime/ref model;
6. evaluator machine shape;
7. intrinsic authorization;
8. materialization breadth;
9. query/fingerprint/dependency behavior;
10. tests added;
11. tests actually executed;
12. tests deferred;
13. performance/resource evidence;
14. residual/baseline failures;
15. deviations and consultations.

## Handoff must record

```text
StaticValue extension procedure
StaticValueStore lifetime law
StaticValueDomain usage
StaticDefinition parameter/application seams
descriptor extension procedure
query dependency/fingerprint rules
evaluator extension API
P3 Pack extension point
P4 FieldKey extension point
what must not be redesigned
```


---

## 15. Required correctness checklists

### 15.1 Type/domain correctness

Before P2 completion:

- [ ] `T: Type` resolves to kinded Type domain.
- [ ] `N: Int` resolves to ordinary compile-time Int value domain.
- [ ] `Flag: Bool` resolves to ordinary Bool value domain.
- [ ] `S: Shape` resolves through existing static-domain authority.
- [ ] mixed parameters preserve source order.
- [ ] ordinary value parameters do not get fake TypeParameter IDs.
- [ ] kinded parameters retain existing substitution/inference.
- [ ] result `: Type` requires proper Type.
- [ ] result `: Bool`/`: Int` uses existing ordinary type relation.
- [ ] evaluator owns no subtype/assignability implementation.
- [ ] legacy aliases preserve TypeId normalization.
- [ ] alias cycles remain deterministic.
- [ ] type lambdas remain intact.
- [ ] existing pack-mode C3 binders retain meaning.

### 15.2 StaticValue correctness

- [ ] no runtime Value/ObjRef/VM Symbol in semantic nodes;
- [ ] store-qualified reusable refs;
- [ ] store lineage survives COW;
- [ ] unrelated stores have distinct IDs;
- [ ] prior indices are not reused/compacted;
- [ ] exact Int beyond i64;
- [ ] small/large Int normalization;
- [ ] exact Float bits;
- [ ] NaN semantic inequality;
- [ ] signed-zero semantic equality;
- [ ] signed-zero representation preserved;
- [ ] String content semantics;
- [ ] Symbol spelling semantics;
- [ ] proper Type vs StaticTerm distinction;
- [ ] Tuple order;
- [ ] current P2 Record key semantics only;
- [ ] exact Data identity;
- [ ] exact VariantId;
- [ ] acyclic structural DAG;
- [ ] sharing requires no deep subtree copy.

### 15.3 Static intrinsic security/soundness

Acceptance-critical negative tests:

- [ ] canonical core Int `+` executes statically;
- [ ] user-defined `+` does not;
- [ ] selector spelling alone grants no authority;
- [ ] rendered class name grants no authority;
- [ ] reflected target must itself be authorized;
- [ ] arbitrary native method does not execute;
- [ ] runtime getter/setter does not execute;
- [ ] runtime constructor method does not execute;
- [ ] runtime closure does not execute;
- [ ] I/O/time/randomness cannot execute;
- [ ] missing authorization on taken path is explicit failure;
- [ ] same unsupported syntax on untaken path does not block reduction.

### 15.4 Runtime/materialization correctness

- [ ] no static initializer executes at runtime;
- [ ] no duplicate side effects;
- [ ] no runtime static-const slot;
- [ ] exact Int precision preserved;
- [ ] Float bits/sign preserved where observable;
- [ ] String/Symbol convert only at materialization;
- [ ] no semantic arena ID leaks into runtime metadata;
- [ ] LargeInt constant survives GC;
- [ ] structural runtime use fails with bounded P8-deferred diagnostic;
- [ ] compiler never uses stale LKG as current value.

---

## 16. Evaluator laws

### 16.1 Source order

Statements/operands execute in Telos source order unless ordinary Telos control semantics explicitly short-circuit.

### 16.2 `if`

```text
evaluate condition
require Bool
select one branch
execute only selected branch
```

### 16.3 `and`

```text
lhs false -> false, rhs skipped
lhs true  -> evaluate rhs
```

### 16.4 `or`

```text
lhs true  -> true, rhs skipped
lhs false -> evaluate rhs
```

### 16.5 Static local slots

```text
Uninitialized -> read error
Initialized(v) -> return v
```

Immutable local re-assignment is invalid.

### 16.6 Static definition application

```text
consume checked definition
validate/bind arguments by static parameter identity/domain
push explicit frame
execute checked body
validate reduced result against declared domain
pop frame
```

No runtime dispatch.

### 16.7 Recursion

P2 active definition/application re-entry -> `Recursive`.

P7 later upgrades to exact application keys, memoization and changing-argument recursive computation.

---

## 17. Numeric resource policy

### 17.1 Exactness

Resource exhaustion terminates evaluation; it never changes Int arithmetic semantics.

### 17.2 Small fast path

Use i64 where safe; promote exactly on overflow. Never use wrapping Int arithmetic.

### 17.3 Shift preflight

Before a large shift:

- validate Telos shift semantics;
- estimate result bit length with overflow-safe host arithmetic;
- charge work/value growth;
- return budget exhaustion before unreasonable allocation.

### 17.4 Power preflight

Before large exact integer exponentiation:

- apply ordinary Telos validity semantics;
- estimate growth conservatively;
- charge algorithmic/limb work;
- terminate by query budget before pathological allocation.

### 17.5 No language-semantic fixed ceiling

Do not invent `MAX_STATIC_INT_BITS` or equivalent as a language limit unless separately ratified. Use query resources.

---

## 18. Fingerprint laws

Semantic constant fingerprinting is content-based.

Conceptually:

```text
Unit        -> H(tag)
Bool        -> H(tag, bool)
Int         -> H(tag, canonical exact integer encoding)
Float       -> H(tag, exact f64 bits)
String      -> H(tag, UTF-8 content)
Symbol      -> H(tag, spelling)
Type        -> H(tag, canonical semantic type fingerprint)
StaticTerm  -> H(tag, canonical static-term fingerprint)
Tuple       -> H(tag, len, child fingerprints in order)
Record      -> H(tag, canonical/current key identity + ordered semantic contents)
Data        -> H(tag, stable declaration/constructor identity, children)
Variant     -> H(tag, exact VariantId identity, children)
```

Do not use process-local store insertion indices as semantic fingerprints.

Changing `+0.0` to `-0.0` may preserve Telos equality while changing representation/materialization fingerprint because exact sign bits are intentionally retained.

---

## 19. Source index / denotation contract

Introduce stable source targets conceptually equivalent to:

```text
SemanticDenotation::StaticDefinition(StaticDefinitionId)
SemanticDenotation::StaticConstant(StaticConstantId)
```

Do not place raw StaticValue arena IDs into general source occurrences.

Index generalized definition parameters, result domains, locals, references, applications and calls so later tooling consumes semantic identity rather than reparsing source.

---

## 20. Cross-module P2 rules

### Live-source provider

If provider semantic source/products are present in the same workspace, consumer evaluation may depend on:

```text
StaticConstant(provider)
StaticDefinition(provider)
```

through the normal query DB.

### Artifact-only provider

If P8 static metadata is not available:

```text
artifact-only static semantic value/definition -> fail closed / explicitly unsupported
```

Never use provider runtime globals, bytecode or runtime constant pool as semantic static authority.

---

## 21. Materialization laws

```text
StaticValue -> MaterializationDescriptor -> runtime representation
```

is one-way authority.

- Int crosses by exact numeric value, never heap identity.
- Float crosses by exact bits.
- String crosses by content.
- Symbol crosses by spelling.
- Runtime interning/allocation occurs only at the materialization boundary.
- Repeated runtime references may share compiler/runtime storage as an optimization, but semantic equality never depends on that sharing.
- P2 structural descriptors may exist semantically while runtime materialization remains explicitly deferred to P8.

---

## 22. Diagnostic contract

Exact diagnostic codes should follow repository naming conventions, but P2 must preserve these distinct semantic categories:

```text
StaticDomainInvalid
StaticArgumentDomainMismatch
StaticResultDomainMismatch
StaticRuntimeDependency
StaticUnsupportedOperation
StaticRecursion
StaticConstantCycle
StaticBudgetExceeded
StaticCancelled                    # terminal; may follow existing non-noisy editor policy
ClosedConstNotReduced
StaticMaterializationDeferred
InternalMissingStaticLoweringProduct
```

Do not reuse `StaticConstUnmaterialized` as a catch-all for semantically different failures.

Diagnostics should retain bounded static definition/application trace/provenance.

---

## 23. Detailed test matrix

### 23.1 Parser / AST

```text
legacy alias
generic legacy alias
canonical generics-before-result ordering
N: Int
T: Type
S: Shape
mixed domains
expression body
lexical block body
block without =
stale = { ... } policy
where-clause placement
ordinary static arguments
kinded arguments
invalid domains
recovery
LexicalBlock vs Closure
```

### 23.2 Static values

```text
Unit
Bool
small Int
large positive/negative Int
Float finite/NaN/signed zero/infinity
UTF-8 String
Symbol
Type
StaticTerm
Tuple
Record
Data
Variant
nested structural values
sharing
store lineage
```

### 23.3 Static domains

```text
T: Type
N: Int
Flag: Bool
S: Shape
wrong argument domain
wrong result domain
mixed definition
proper-Type guard
ordinary assignability reuse
```

### 23.4 Operations

```text
canonical primitive operation
user overload negative
selector spoof negative
reflected target exactness
non-whitelisted native negative
unsupported taken path
unsupported untaken path
```

### 23.5 Evaluator

```text
let
let mut
assignment
uninitialized local
immutable reassignment
lexical result
if
nested if
and/or short-circuit
nested static definition application
recursion terminal
source order
structural construction
budget
cancellation
```

### 23.6 Const

```text
literal
computed
const-to-const
const-to-definition
runtime dependency rejection
cycle
closed symbolic rejection
no runtime slot
no initializer bytecode
repeated runtime use
current invalid vs LKG
```

### 23.7 Incremental

```text
constant value edit invalidates dependent
same ordinary type still invalidates
definition signature change invalidates consumers
definition body change invalidates dependent const
unrelated edit reuses
cancellation not cached as success
budget failure not cached as success
StaticValueRef survives lineage
foreign-store ref rejected
```

### 23.8 Materialization

```text
Unit
Bool
small Int
LargeInt
Float bits
String
Symbol
LargeInt GC survival
structural deferred diagnostic
no initializer replay
```

### 23.9 C3 compatibility

```text
Type alias normalization
generic alias substitution
TypeLambda
kind checking
type data/static domain
TypePack arguments
type members
RecordRow
alias cycles
metadata tests that should remain unchanged
```

---

## 24. Performance/resource evidence required

P2 need not over-optimize prematurely, but completion must record evidence for its cost centers.

### StaticValue store

Prove:

- child nodes are ID-referenced;
- shared children do not clone subtrees;
- store prefix survives incremental clone;
- String/Symbol storage is not duplicated per reference if interned.

### Evaluator

Prove:

- locals are indexed;
- guest nesting uses explicit vector/arena frames;
- body AST/analysis is referenced rather than cloned per application;
- diagnostic trace is bounded.

### Queries

Include an incremental test showing targeted static edits recompute the appropriate definition/constant frontier rather than the whole workspace.

### BigInt

Include adversarial resource tests showing query budget termination before unreasonable allocation.

### Compiler

Inspect bytecode/compiled representation to prove:

- initializer absent;
- no runtime const slot;
- value is materialized from semantic lowering data.

---

## 25. Checkpoint bookkeeping

### At P2 start

Only after P1 closure:

```text
active_plan: LANG007.C4.P2
latest_completed_plan: LANG007.C4.P1
start revision: <actual SHA>
```

### Durable mid-plan updates

Update checkpoint when a durable interface/gate lands:

- T1 shared domain authority;
- T3 StaticDefinition authority;
- T4 StaticValueStore lifetime;
- T6 exact intrinsic authority;
- T7 evaluator machine;
- T8 StaticConstant query/fingerprint authority;
- T9 M2 materialization;
- T10/G gate evidence;
- any consultation/amendment;
- any new deferred/baseline issue.

Do not log every edit/test.

### At completion

Update P2 ledger truthfully, add walkthrough/handoff references, record residual failures/evidence and set next action.

---

## 26. Completion truth table

| Condition | P2 state |
|---|---|
| architecture only planned | PROPOSED / NOT_STARTED / UNVERIFIED |
| partial source implementation | IN_PROGRESS / PARTIAL / truthful verification |
| intended code landed, evidence incomplete | IN_PROGRESS or COMPLETE per repo lifecycle / IMPLEMENTED / UNVERIFIED |
| intended code + focused P2 gates pass | COMPLETE / IMPLEMENTED / FOCUSED_TESTED |
| actual release-level broad certification passes | COMPLETE / IMPLEMENTED / RELEASE_COMPLETE |
| P1 still open | P2 remains non-executable / blocked by predecessor |

A clean Git tree is not evidence.

---

## 27. Plan self-review

This plan resolves the architectural decisions the handoff required:

- [x] current main re-verified against P2 analysis baseline;
- [x] P1 implementation vs administrative closure distinguished;
- [x] Kind vs ordinary-value static domains fixed;
- [x] definition-owned static parameters chosen;
- [x] C3 generic mutation rejected;
- [x] generalized `type` AST/grammar migration defined;
- [x] block body uses LexicalBlock;
- [x] legacy Type aliases retain optimized C3 path;
- [x] general static-value argument syntax required;
- [x] StaticValueStore lifetime solved in P2, not deferred to P8;
- [x] published store-qualified StaticValue refs selected;
- [x] exact/unbounded Int representation selected;
- [x] Float storage vs Telos equality separated;
- [x] String/Symbol runtime identity excluded;
- [x] structural values defined as immutable DAGs;
- [x] exact VariantId preserved;
- [x] operation authorization tied to exact semantic identity;
- [x] user overload negative tests made acceptance-critical;
- [x] compile-time VM/arbitrary dispatch rejected;
- [x] path-sensitive execution required;
- [x] explicit-frame evaluator specified;
- [x] local semantic storage specified;
- [x] P2 control subset bounded;
- [x] recursion outcome separated;
- [x] StaticEvalOutcome prepared for P7;
- [x] query keys/products/dependencies specified;
- [x] value-content fingerprints specified;
- [x] cancellation/current/LKG policy specified;
- [x] BigInt resource accounting addressed;
- [x] constant cycles specified;
- [x] live-source vs artifact cross-module boundary explicit;
- [x] semantic materialization descriptor specified;
- [x] M2 primitive materialization breadth adopted;
- [x] ModuleLoweringSemantics remains semantic-to-core projection;
- [x] source identity/indexing seam specified;
- [x] runtime/type/performance/incremental requirements included;
- [x] detailed test matrix included;
- [x] lifecycle checkpoint/walkthrough/handoff obligations included.

---

## 28. Final implementation directive

Execute the plan in task order. Do not collapse semantic authority boundaries merely because several tasks touch the same Rust files.

The highest-risk mistakes to prevent are:

```text
1. forcing ordinary compile-time values into TypeData;
2. globally changing C3 generic parameter semantics for N: Int;
3. publishing snapshot-local StaticValue IDs without lineage qualification;
4. executing operators by token/selector spelling;
5. building a compile-time VM or arbitrary native dispatch route;
6. eagerly rejecting unsupported untaken branches;
7. using host recursion for guest static computation;
8. fingerprinting constants only by type or arena ID;
9. using last-known-good as compiler truth after current invalidation;
10. rerunning const initializers during lowering/runtime;
11. pulling P8 structural/artifact materialization into P2 ad hoc;
12. allowing BigInt growth to bypass semantic resource budgets.
```

If live implementation pressure requires changing any of those boundaries, STOP AND CONSULT rather than improvising.


### T7 focused acceptance closure — 2026-10-04

The focused `integration::static_definition_bodies` selector passed 3/3 after
adding the remaining discriminators for false and nested `if`, nested
arithmetic, source-ordered mutable writes, uninitialized local reads, and
immutable reassignment rejection. Existing coverage includes both lazy boolean
operators and executed/untaken unsupported operations, application argument
validation, self/mutual recursion, cancellation, trace growth, and structural
construction identity. A Data-construction budget boundary proves exhaustion
before interning the Data value node. The plan's optional separate value-node
quota is not enabled: `QueryBudget` exposes no such quota. T7 is therefore
`IMPLEMENTED / FOCUSED_TESTED`; proceed to T8. P2 remains
`IN_PROGRESS / PARTIAL / UNVERIFIED` until T8–T11 and their gates complete.


### T8 plan amendment — constant-owned initializer source/result (2026-10-04)

The stable query identity is `StaticConstantId(SymbolId)`. Add a
constant-specific initializer source/check result, fingerprint, and product
keyed by that identity. Do not fabricate a `StaticDefinition` or reuse
`StaticBodyId(DeclarationId)` for module constants. Extend `ModuleBindingFact`
with the exact const declaration/initializer source site as it is built beside
ordinary `<main>` analysis. Retrieve syntax from `ParsedModuleUnit` using that
site and stable `SymbolId`; never re-resolve its name.

`CallableAnalysis` provides checked expression facts but does not provide the
initializer AST subtree or expression-tree relationship required to construct
the evaluator body. Check the exact initializer with the shared ordinary
checker in a static-body context, consuming canonical module bindings and
existing dispatch/semantic products. This is another analysis invocation, not a
second checker or resolution authority. The constant-owned product carries
checked body facts, dependencies, evaluation outcome, and targeted source
fingerprint.

Amend the write set to include `phalcom-semantic/src/module_bindings.rs` and
`phalcom-semantic/src/static_constant.rs` alongside the listed DB/session/
snapshot files. Required discriminator: for two constants in one module,
changing one initializer invalidates only its `StaticConstant(SymbolId)` result
while reusing the other; assert checked reads preserve stable `SymbolId`s.
T8.1/T8.2 are the next implementation slice. T9 remains blocked until T8 is
accepted.
