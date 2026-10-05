---
id: LANG007.C4.P3-requirements-analysis
category: LANG
program: LANG007
checkpoint: LANG007.C4
plan: LANG007.C4.P3
kind: requirements-analysis
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
repository: aureat/phalcom-lang
branch: main
repository_revision_audited: 5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521
analysis_date: 2026-10-05
predecessor: LANG007.C4.P2
next_plan_target: LANG007.C4.P3
---

# LANG007.C4.P3 — First-class Pack and Call/Rest Convergence
## Detailed Implementation Requirements Analysis and Planning Basis

**Project:** Telos / `aureat/phalcom-lang`
**Checkpoint:** `LANG007.C4` — Structural Static Upgrade
**Plan under analysis:** `LANG007.C4.P3` — First-class Pack and call/rest convergence
**Repository baseline audited:** `main @ 5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521`
**Latest pushed baseline commit:** `feat(lang007): complete C4 P2 static constants pipeline and materialization vertical`
**Analysis date:** 2026-10-05

---

# 1. Executive conclusion

`LANG007.C4.P3` should be implemented as a **convergence plan**, not as a fresh variadic-argument subsystem and not as a replacement for the C1–C3 pack algebra.

The current repository already contains most of the *transport syntax* and much of the *type-level pack algebra* that a naïve P3 design might otherwise attempt to add again:

- source and AST already distinguish `*`, `**`, and `***` expansion/rest lanes;
- C1–C3 already own `TypePack`, `LabeledTypePack`, `CompleteTypePack`, flat concat terms, pack projections, variadic generic modes, inference, substitution, and metadata for those **kinded type-level terms**;
- the compiler/VM already have a private mutable `ArgumentPackBuilderObject` for dynamic outgoing argument assembly;
- dynamic invocation already consumes that builder directly without requiring a surface Pack value;
- P2 already established semantic `StaticValue`, static evaluation, store lineage, exact compile-time operation authority, and the deliberate extension point where P3 adds first-class Pack;
- P2 also generalized static-definition parameter syntax while intentionally **not** reinterpreting pack binders as ordinary value parameters.

P3's job is therefore to make one language-level concept coherent across these existing layers:

```text
semantic call arguments
    ≈ ordered two-lane argument Pack

first-class Pack value
    = immutable, lane-sensitive, label-sensitive structural transport

runtime call staging
    = private mutable builder/window, not the language Pack object

C1–C3 TypePack terms
    = optimized compile-time type-shape representation,
      bridged to the public Pack<Type> ontology rather than discarded
```

The governing architectural rule is:

> **A call semantically has a Pack-shaped argument product, but ordinary fixed calls must not be forced to allocate or materialize a Pack.**

This distinction is the central P3 soundness/performance constraint. The implementation must preserve the existing direct fixed-call path while adding real immutable Pack values only where semantics require them: explicit Pack construction, rest capture, stored/returned Pack values, static Pack values, Pack views, and forwarding/spreading from an already first-class Pack.

P3 also owns an important P2-deferred generalization:

> **A variadic static-definition parameter in an ordinary value domain must capture a first-class Pack value, while a kinded variadic parameter continues to use C1–C3 TypePack machinery.**

That means the final P3 design must preserve the distinction between:

```telos
type F<*P>: Int = ...
// P remains a C3 positional pack of Type terms by default.
```

and a value-domain form such as:

```telos
type F<*P: Int>: Int = ...
// P is one compile-time PositionalPack<Int> value.
```

The live AST already retains the expansion mode, but P2's semantic `StaticParameter` currently does not store that mode directly. That is acceptable for P2's kinded lane because `TypeParameterId` leads back to `GenericParameterMode`; it is insufficient for P3 value-domain variadics because those parameters intentionally have no `TypeParameterId`. P3 should close that seam explicitly rather than introduce a second variadic-static-parameter representation.

The runtime implementation should use a **dedicated Pack surface identity over flat/shared product storage**, not Tuple identity and not recursive cons cells. Pack and Tuple may share lower-level ordered-product storage/layout machinery, but they remain distinct values and distinct runtime classes. A Pack slice/view should share backing storage and retain O(1) slicing/rest formation; equality/hash must operate on the logical visible lanes, not allocation identity or backing identity.

Finally, P3 must not accidentally absorb later checkpoint work:

- P4 owns generalized heterogeneous `FieldKey`; P3 Pack labels remain `Symbol`.
- P5 owns public Product/Sum reflection and compiler-derived structural capabilities.
- P6 owns static structural `for`/`match`/`while` and generalized structural pattern execution.
- P7 owns recursive static computation/residualization/memoization.
- P8 owns durable static Pack/value metadata and general cross-artifact static materialization.
- P9 owns final public retirement of the legacy TypePack/RecordFields ontology and broad tooling/documentation/performance closure.

P3 should nevertheless leave each of those plans a stable, additive seam rather than a representation they must replace.

---

# 2. Source authority and audit basis

## 2.1 Lifecycle authority

The checkpoint record is the single durable lifecycle authority for C4. At the audited state it identifies:

```text
active_plan:          LANG007.C4.P3
latest_completed_plan: LANG007.C4.P2
next_plan:            LANG007.C4.P3
```

and records P2 as:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

while P3 remains:

```text
PROPOSED / NOT_STARTED / UNVERIFIED
```

This analysis therefore treats P3 as legitimately unblocked but not started.

The repository was independently checked against pushed `main`, whose latest audited revision is:

```text
5a5bf8b218c2fbcfd785bfa41bb4eb7fec21c521
```

The latest commit closes the P2 static-constant/materialization vertical and advances the lifecycle records to P3. There is no basis for planning against the older pre-P2 repository state.

## 2.2 Principal requirements/design records

The analysis uses the following as normative or near-normative inputs:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
LANG007.C4.P2-handoff.md
LANG007.C4.P2-walkthrough.md
LANG007.C4.P2-requirements-analysis-main.md
LANG007.C4.P2-luna-patch-grade-implementation-plan.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4-requirements-analysis.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-structural-static-upgrade-handoff.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md
```

C1–C3 implementation records are also relevant wherever they define canonical TypePack identity, generic binder modes, substitution/inference, metadata, or Tuple formation. P3 must reuse those authorities rather than merely copy their behavior.

## 2.3 Live implementation paths audited

The current repository audit traced at least the following paths:

```text
phalcom-ast/src/
    ast.rs
    parser.rs

phalcom-semantic/src/
    static_value/{mod.rs,store.rs}
    static_definition.rs
    static_eval/**
    declarations.rs
    checker/{call.rs,expression.rs,static_body.rs}
    types/{store.rs,parameter.rs,annotation.rs,relation.rs,substitution.rs,
           instantiation.rs,type_lambda.rs,projection.rs}
    metadata/**

phalcom-core/src/
    heap/{object.rs,tuple.rs,pack_builder.rs,trace.rs,mod.rs,accessors.rs}
    product/{mod.rs,shape.rs,storage.rs,layout.rs,view.rs}
    parameters.rs
    method/mod.rs
    bytecode.rs
    compiler/lib/{mod.rs,expr.rs}
    vm/dispatch.rs
    primitive/{block.rs,fiber.rs,tuple.rs}
    value/{mod.rs,repr.rs,render.rs}
    universe/{core_classes.rs,primitives.rs}

phalcom-core/core/universe/src/collections/
    tuple.ph
    record.ph

phalcom-native-meta/src/universe.rs
phalcom-type-meta/src/**

phalcom-core/tests/core/collections/outgoing_packs_completion.rs
```

This analysis distinguishes requirements derived from C4 records from live repository facts. Where the live implementation mechanically differs from prose examples, the current repository wins unless doing so would violate an explicitly ratified C4 invariant.

---

# 3. Lifecycle and predecessor takeover state

## 3.1 P1 is closed and must not be reopened by P3

P3 inherits P1's binding/field architecture. None of the following is P3 scope:

```text
let       runtime immutable
let mut   runtime mutable
const     static

three-state definite initialization
runtime checked-read fallback
immutable-field construction lifecycle
explicit stored-field authority
```

Pack rest capture must use the established binding phase/mutability semantics. A rest parameter is an ordinary runtime binding whose value is an immutable Pack; P3 must not add a special mutable/rest storage category.

## 3.2 P2 is closed and is P3's semantic foundation

P2 established the following relevant seams:

```text
StaticValueDomain::{Kind, ValueType}
StaticValueStore / StaticValueRef / store lineage
StaticEvalOutcome
explicit-frame StaticEvaluator
StaticDefinition / CheckedStaticBody
StaticConstantProduct
content-based static fingerprints
QueryBudget / CancellationToken
MaterializationDescriptor
```

Current `StaticValueData` deliberately ends at:

```rust
Unit
Bool
Int
Float
String
Symbol
Type
StaticTerm
Tuple
Record
Data
Variant
```

P2's own requirements explicitly reserved first-class Pack for P3. P3 must extend this enum/store additively; it must not replace the store, introduce a Pack-specific static arena with independent lifetime, or reroute Pack through runtime `Value` during semantic evaluation.

## 3.3 P2's static parameter rule and P3's deferred extension

P2 deliberately kept:

```text
*P
**P
***P
```

in the C3 kinded/pack-domain lane and explicitly deferred public first-class Pack value semantics to P3.

The live AST is already better than the compressed P2 handoff excerpt implies:

```rust
pub struct StaticParameterSyntax {
    pub mode: Option<ExpansionMode>,
    pub mode_range: Option<SourceRange>,
    pub variance: VarianceSyntax,
    pub name: String,
    pub name_range: SourceRange,
    pub domain: Option<StaticDomainSyntax>,
    pub range: SourceRange,
}
```

But P2's semantic representation is currently:

```rust
pub struct StaticParameter {
    pub name: String,
    pub domain: StaticValueDomain,
    pub index: u32,
    pub type_parameter_id: Option<TypeParameterId>,
    pub range: SourceRange,
}
```

There is no explicit `GenericParameterMode`/expansion mode field.

For kinded parameters this can be recovered through the bridged `TypeParameterId`, whose `TypeParameterData` already stores `GenericParameterMode`. For ordinary value-domain parameters there is intentionally no `TypeParameterId`; therefore P3 must publish the source mode directly in the static parameter/signature if value-domain variadics are to be soundly represented.

This is not a reason to redesign the P2 parameter model. It is the expected additive P3 closure:

```rust
pub struct StaticParameter {
    // existing fields
    pub mode: GenericParameterMode,
}
```

or a mechanically equivalent representation whose fingerprint/application/evaluator semantics retain the same fact.

---

# 4. P3 bounded objective

P3 is accepted when **Pack is a real Telos structural value** across static and runtime execution, and when call/rest/spread behavior converges on that model without regressing fixed calls.

The end-to-end target is:

```text
source Pack / rest / spread
        ↓
existing AST lane syntax (*, **, ***)
        ↓
canonical semantic Pack type + C1–C3 type-pack bridge
        ↓
StaticValue::Pack when compile-time
        ↓
private compiler/VM argument builder when shape is dynamic
        ↓
materialized immutable runtime Pack only when semantically required
        ↓
rest capture / forwarding / Pack views
```

while preserving the optimized fixed-call path:

```text
foo(a, b, c)
    ↓
known selector + known argument window
    ↓
direct/specialized invocation
    ↓
NO NewArgumentPack
NO first-class Pack allocation
```

## 4.1 P3 owns

P3 owns at least:

- first-class Pack source/type/value identity;
- exact positional/labeled/complete Pack lane semantics;
- `PositionalPack`, `LabeledPack`, `CompletePack` public forms;
- additive `StaticValue::Pack` support;
- static Pack value construction/equality/hash/fingerprinting;
- ordinary value-domain variadic static-parameter capture into Pack;
- immutable runtime Pack representation;
- empty `Pack()` identity distinct from Unit/Tuple/Record;
- Pack equality/hash and lane observation;
- one-step Pack views with O(1) rest/slicing;
- dynamic builder finalization into immutable Pack;
- rest capture migration from Tuple/List to Pack;
- Pack spreading/forwarding through the existing argument-building machinery;
- Tuple/Pack interoperability at explicit spread/construction boundaries;
- duplicate-label checking appropriate to dynamic Pack assembly;
- fixed-call no-allocation preservation;
- P3-specific semantic/query/type fingerprints and in-process incremental correctness;
- required ordinary type metadata changes if Pack types must cross module boundaries during P3.

## 4.2 P3 does not own

P3 must not absorb:

```text
P4: generalized FieldKey, Int/Bool/String record keys, unordered-product calculus
P5: Product/OrderedProduct/UnorderedProduct/Sum public reflection/capability system
P6: static structural for/match/while and generalized pattern execution
P7: recursive static computation, residual normalization, memoization
P8: durable StaticValue::Pack DAG serialization and cross-artifact static products
P9: final retirement of legacy TypePack/LabeledTypePack/CompleteTypePack public façade
```

P3 also does not own a general runtime Iterator-based spread protocol. Argument expansion remains a compiler/runtime structural operation over authorized structural values, not arbitrary user iteration.

---

# 5. Canonical language semantics

## 5.1 Pack is an ordered two-lane immutable structural value

Canonical conceptual source:

```telos
intrinsic data Pack<***Es>(***Es)

type PositionalPack<T: Type> =
    Pack<*: T>

type LabeledPack<T: Type> =
    Pack<**: T>

type CompletePack<T: Type> =
    Pack<***: T>
```

The logical shape is:

```text
Pack
  positional lane: [v0, v1, ...]
  labeled lane:    [(#a, va), (#b, vb), ...]
```

Both lane boundaries and order are semantic.

Two Packs compare structurally only when:

1. their visible positional lengths match;
2. their visible labeled lengths match;
3. positional values compare pairwise in order;
4. labeled symbols compare pairwise in order;
5. corresponding labeled values compare pairwise;
6. nested values use ordinary Telos semantic equality.

Pack equality is not Tuple equality, even when both carry the same visible elements.

## 5.2 Pack labels are Symbol in P3

P3 labels remain canonical Symbols.

Do not generalize Pack labels to P4's `FieldKey` merely because the eventual Product system supports heterogeneous keys. Call labels/selectors are Symbol-oriented today and P3 should preserve that efficient authority.

Duplicate labels are invalid within a Pack's labeled lane.

## 5.3 `Pack()` is a distinct empty value

The following must remain distinct:

```telos
()       // Unit / empty Tuple semantics
{}       // Empty Record
Pack()   // empty Pack
```

No P3 normalization may collapse `Pack()` to Unit or EmptyRecord.

Because empty rest capture is common, an allocation-free runtime representation is strongly preferable. A dedicated `ValueTag::EmptyPack` is a good repository-native option if the Value-tag budget and exhaustive-match audit permit it. A canonical singleton Pack object is a secondary option. The implementation plan should select one after auditing the tag layout; **either representation must preserve distinct Pack class/equality/type identity**.

## 5.4 `*`, `**`, `***` remain the canonical lane syntax

P3 must not introduce `...` as Pack syntax.

Existing C2/C3 semantics are:

```text
*     positional lane
**    labeled lane
***   complete two-lane pack
```

The P2 handoff's prose phrase “rest/splat convergence (`...pack`)” should be read descriptively, not as a new token decision.

## 5.5 Rest capture result

Required P3 runtime semantics:

```text
*rest
    -> Positional Pack

**rest
    -> Labeled Pack

***rest
    -> Complete Pack

empty capture
    -> Pack()
```

The normalized internal `Split` rest mode remains legitimate runtime signature metadata for a callable that binds positional and labeled rest into two separate parameters. Its values become two Pack values:

```text
Split
    positional parameter -> Positional Pack
    labeled parameter    -> Labeled Pack
```

`Split` is not a new source Pack lane.

## 5.6 Expansion semantics

Expansion is lane projection, not arbitrary conversion:

```telos
f(*p)      // contribute p's positional lane
f(**p)     // contribute p's labeled lane
f(***p)    // contribute both lanes
```

For a Complete Pack, `*p` does not imply that the labeled lane must be empty; it selects only the positional lane. Likewise `**p` selects only the labeled lane.

Duplicate labels are checked after all labeled contributions are fused in source evaluation order.

## 5.7 Record is not an implicit LabeledPack

A Record is an unordered-product value with different semantic identity. P3 must not silently define:

```text
Record => **arguments
```

as Pack equivalence.

If existing generic labeled-spread syntax accepts Record for historical collection expansion, P3 must distinguish that compatibility behavior from **Pack rest/capture typing**. In particular, a parameter whose semantic type is `LabeledPack<T>` is not satisfied merely because a Record happens to contain Symbol keys. Any future explicit Record→Pack conversion belongs behind a deliberate API/operation, not an implicit subtype/equality rule.

## 5.8 Tuple is not Pack

Tuple and Pack are both ordered structural products, but their intent and identity differ:

```text
Tuple
    fixed product value / collection semantics

Pack
    structural transport / variadic capture / spreading / structural traversal
```

They may share storage/layout algorithms. They must not share language identity.

Interoperability should happen through explicit structural operations:

```telos
Pack(*tuple)
Tuple(*pack)
f(*tuple)
f(*pack)
```

where the syntax/semantic operation supports it.

Do not add implicit `Tuple <: Pack` or `Pack <: Tuple` merely to reuse implementation.

---

# 6. Current implementation audit

## 6.1 AST already has the correct three-lane transport model

`phalcom-ast/src/ast.rs` already contains:

```rust
pub enum ExpansionMode {
    Positional,
    Labeled,
    Complete,
}

pub enum PackItem {
    Positional { ... },
    Labeled { ... },
    Expand { mode: ExpansionMode, expr: Expr, ... },
}

pub enum RestMode {
    None,
    Positional,
    Labeled,
    Complete,
}
```

Method parameters retain `rest_mode`; generic/static parameters retain `mode: Option<ExpansionMode>`.

**Requirement:** P3 must reuse these AST facts. No `PackSpreadExpr`, `PackRestExpr`, or parallel P3-only syntax tree should be introduced.

## 6.2 C1–C3 already own type-level pack terms

The canonical TypeStore already contains:

```rust
TypeData::TypePack(...)
TypeData::LabeledTypePack(...)
TypeData::CompleteTypePack { positionals, labeled }
TypeData::TypePackConcat(...)
TypeData::LabeledTypePackConcat(...)
TypeData::TupleFromPacks { ... }
TypeData::CallableFromPacks { ... }
```

and:

```rust
GenericParameterMode::{
    Single,
    PositionalPack,
    LabeledPack,
    CompletePack,
}
```

The established rule remains:

```text
Single(TypePack) != PositionalPack(TypePack)
```

and kind/domain alone must not be used to infer binder mode.

P3 should not add `PackVarId`, a second variadic substitution table, a second pack concat algebra, or a second lane solver.

## 6.3 Tuple already demonstrates open ordered-product typing

`TupleType` currently carries:

```rust
pub struct TupleType {
    pub elements: Box<[TupleTypeElement]>,
    pub positional_remainder: AnonymousTupleRemainder,
    pub labeled_remainder: AnonymousTupleRemainder,
}

pub enum AnonymousTupleRemainder {
    None,
    Unconstrained,
    Typed(TypeId),
}
```

The parser/semantic layer already has `AnonymousRemainderSyntax` and lane-specific remainder syntax for Tuple types.

This is important because P3's canonical aliases use forms such as:

```telos
Pack<*: T>
Pack<**: T>
Pack<***: T>
```

The current generic-argument grammar supports expansion of an **existing operand** (`*P`, `**P`, `***P`) but does not yet expose this anonymous typed-tail form in generic application syntax. P3 therefore needs a type-syntax extension for Pack family constraints; it should reuse the existing anonymous-remainder semantic concepts rather than invent unrelated “homogeneous Pack” type logic.

## 6.4 P2 StaticValue has an intentional Pack hole

`StaticValueStore` is append-only per lineage and currently interns primitives plus Tuple/Record/Data/Variant structural nodes. It already has the correct lifetime and fingerprint authority for P3.

P3 should add conceptually:

```rust
StaticValueData::Pack(StaticPackId)
```

without creating a new store lineage or runtime dependency.

## 6.5 The VM already has a private mutable outgoing-argument builder

Current runtime structure:

```rust
pub struct ArgumentPackBuilderObject {
    positionals: Vec<Value>,
    labels: Vec<Symbol>,
    labeled_values: Vec<Value>,
    pending_label: Option<usize>,
}
```

It is explicitly compiler-owned and unobservable.

This is exactly the right mutable staging seam for dynamic argument assembly. It must remain private.

P3 should add immutable Pack finalization to this builder, not surface the builder as the Pack object.

## 6.6 Current duplicate-label detection is linear per insertion

`ArgumentPackBuilderObject` currently checks duplicate labels using a linear scan (`labels.contains(...)`).

For large dynamic labeled Packs this can become O(n²).

P3 should preserve the cheap small-n path while adding a spill strategy for larger labeled lanes, for example:

```text
small labeled count
    -> linear scan, zero hash allocation

above threshold
    -> transient HashSet<Symbol> / compact index
```

The duplicate-detection index belongs to the mutable builder, not to every immutable Pack value.

## 6.7 Current rest capture is inconsistent and Tuple/List-based

The live repository exposes several current rest-capture paths:

- normal method dispatch finalizes excess rest arguments through Tuple construction;
- `primitive/block.rs` also materializes positional rest through `finish_tuple`;
- at least one Fiber/closure entry path still materializes extra closure arguments into `List`.

This is a high-value P3 convergence target. Leaving any of these paths unchanged would create observable semantic drift depending on how the same closure/method is entered.

P3 must audit **all activation boundaries**, not only the primary bytecode method dispatch loop.

## 6.8 `FinishTuplePack` is not globally obsolete

Dynamic Tuple literal construction with expansion legitimately uses the private argument builder and finalizes a Tuple. P3 must not mechanically rename every `FinishTuplePack` use to Pack finalization.

Required distinction:

```text
Tuple expression with expansion
    -> builder
    -> FinishTuplePack / finish_tuple
    -> Tuple

Pack expression / rest capture
    -> builder or capture path
    -> FinishPack / finish_pack
    -> Pack

outgoing dynamic call
    -> builder
    -> InvokePack
    -> argument window, usually no surface Pack object
```

This is one of the most important implementation distinctions in P3.

## 6.9 Fixed calls already have a no-builder test seam

`phalcom-core/tests/core/collections/outgoing_packs_completion.rs` already asserts that a fixed call emits none of the dynamic pack-building bytecodes.

P3 should strengthen rather than replace that test. The fixed-call no-allocation property should become an explicit acceptance gate and microbenchmark.

## 6.10 Runtime Tuple/Record already share product infrastructure

Tuple and Record are native heap objects using shared product shape/layout/storage registries. GC tracing is descriptor/layout-directed.

This is a strong repository-native substrate for Pack. A Pack should reuse the storage/layout machinery while adding distinct Pack identity and slice/range semantics.

---

# 7. Required semantic type architecture

## 7.1 Do not equate first-class Pack type with C3 TypePack kinded term

These are different semantic categories:

```text
TypePack / LabeledTypePack / CompleteTypePack
    -> C1–C3 kinded compile-time terms describing sequences of Types

Pack<...>
    -> proper type of a first-class Telos value
```

The names are intentionally converging publicly, but the internal distinction remains critical.

A `TypeData::TypePack` value must not suddenly become a proper runtime `Pack` type merely because the public surface introduces `PositionalTypePack` aliases.

## 7.2 Recommended proper Pack type representation

The most repository-native direction is a dedicated proper ordered Pack type that reuses C3 lane terms for exact symbolic expansion and reuses the existing ordered-product remainder concepts for open/homogeneous families.

Conceptually:

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
    // existing
    Pack(PackType),
    PackFromPacks {
        positionals: TypeId, // Kind::TypePack
        labeled: TypeId,    // Kind::LabeledTypePack
    },
}
```

Exact names are flexible. The requirements are not:

- `Pack` is a proper Type;
- exact fixed Pack shapes canonicalize deterministically;
- open typed/unconstrained tails can represent `PositionalPack<T>` etc.;
- symbolic expansions can remain as lane terms until substitution resolves them;
- when a symbolic lane becomes concrete, normalization returns the canonical concrete Pack type;
- no Pack-specific inference variable arena is added;
- no recursive concat tree is added.

A lower-churn variant may represent Pack proper types as a canonical intrinsic `Applied` form around the Pack declaration plus a complete pack term. This is acceptable **only if** it can represent the anonymous typed-tail families and give relation/inference code exact Pack semantics without string/declaration-spelling hacks. If that cannot be demonstrated cleanly, a dedicated proper Pack node is preferable.

## 7.3 Reuse relation helpers, not Tuple identity

Tuple and Pack ordered-product type relations share significant logic:

```text
fixed positional alignment
fixed labeled alignment
positional/labeled remainder constraints
exact label order
substitution of captured pack lanes
```

P3 should extract/reuse an ordered-product relation helper where profitable. It must not obtain Pack semantics by pretending a Pack type is Tuple.

Correct direction:

```text
ordered-product relation helper
    ├─ Tuple relation policy
    └─ Pack relation policy
```

not:

```text
Pack -> convert to Tuple -> run Tuple relation
```

because the latter risks cross-kind subtype/equality leakage.

## 7.4 Canonical public aliases and the C3 bridge

C4's target public vocabulary is:

```telos
type PositionalTypePack = PositionalPack<Type>
type LabeledTypePack    = LabeledPack<Type>
type CompleteTypePack   = CompletePack<Type>
```

but the C1–C3 optimized internals remain:

```text
TypeData::TypePack
TypeData::LabeledTypePack
TypeData::CompleteTypePack
```

P3 must establish an explicit bridge rather than duplicate meaning.

The implementation plan should decide the precise projection authority, but it must satisfy:

```text
public PositionalPack<Type> semantics
    ↔ canonical TypePack term when used as a type-level pack domain

public LabeledPack<Type>
    ↔ canonical LabeledTypePack term

public CompletePack<Type>
    ↔ canonical CompleteTypePack term
```

The bridge must be based on exact canonical declaration identity and semantic specialization, not textual names.

P3 does **not** have to delete the legacy public intrinsic declarations immediately; P9 owns final ontology retirement. P3 must, however, stop designing new public APIs around them as conceptually separate structures.

## 7.5 Anonymous typed Pack family syntax

The source forms:

```telos
Pack<*: T>
Pack<**: T>
Pack<***: T>
```

mean an anonymous typed remainder/tail, not expansion of an expression named `T`.

The current AST's `GenericArgumentSyntax::Expansion` models `*P`-style expansion and requires an operand. P3 should add a distinct syntax node or generalize the contribution model so the semantic distinction remains explicit.

Recommended conceptual form:

```rust
pub struct TypePackRemainderSyntax {
    pub mode: ExpansionMode,
    pub remainder: AnonymousRemainderSyntax,
    pub range: SourceRange,
}

pub enum GenericArgumentSyntax {
    Single(TypeAnnotation),
    Expansion(TypePackExpansionSyntax), // *P
    Remainder(TypePackRemainderSyntax), // *: T / **: T / ***: T
    Value(Expr),
}
```

Reuse `AnonymousRemainderSyntax::{Unconstrained, Typed}`. Do not create separate `HomogeneousPackConstraint` semantics.

The parser must continue rejecting `...` as pack syntax.

---

# 8. Static Pack value architecture

## 8.1 Add Pack to the existing StaticValueStore

Required additive extension:

```rust
structural_id!(StaticPackId);

pub enum StaticValueData {
    // existing P2 nodes
    Pack(StaticPackId),
}
```

The static Pack value must contain no runtime `Value`, VM Symbol, heap reference, or ProductStorage pointer.

## 8.2 Static Pack backing must support slices

A naïve node such as:

```rust
struct StaticPack {
    positionals: Box<[StaticValueRef]>,
    labels: Box<[StaticSymbolId]>,
    labeled_values: Box<[StaticValueRef]>,
}
```

is sufficient for whole Pack storage but would make every `rest` copy O(n).

P3 should introduce shared backing plus ranges, conceptually:

```rust
struct StaticPackBacking {
    positionals: Arc<[StaticValueRef]>,
    labels: Arc<[StaticSymbolId]>,
    labeled_values: Arc<[StaticValueRef]>,
}

struct StaticPackData {
    backing: Arc<StaticPackBacking>,
    positional_start: u32,
    positional_len: u32,
    labeled_start: u32,
    labeled_len: u32,
}
```

or store-local backing IDs with the same semantics.

Requirements:

- lane length O(1);
- first/rest view construction O(1);
- rest Pack creation O(1) structural work;
- backing is immutable;
- all child refs are from the same `StaticValueStore` lineage;
- no backpatching/cyclic containment;
- content fingerprint sees only the logical visible slice;
- semantic equality/hash sees only the logical visible slice;
- source construction order is preserved;
- duplicate labeled Symbols are rejected before publication.

## 8.3 Canonical interning and O(1) slicing must not conflict

The current P2 store uses content-keyed interners for structural nodes. A Pack slice keyed by its complete logical contents would require O(n) hashing at every rest operation, defeating the O(1) view requirement.

P3 should explicitly separate:

```text
representation identity
vs
Telos semantic equality/hash/content fingerprint
```

just as P2 already separates Float representation identity from language equality semantics.

It is acceptable for two representation-distinct static Pack slice nodes to be semantically equal. `StaticValueRef` identity is never Telos value equality.

Whole newly built Packs may still be opportunistically interned; slices must not be forced through an O(n) canonicalization path solely to preserve one-ID-per-content behavior.

## 8.4 Static Pack construction must use builder-style accumulation

Static code such as repeated structural composition must not repeatedly copy immutable Pack arrays.

P3 should provide an evaluator-private mutable builder:

```text
StaticPackBuilder
    append positional
    append labeled
    append positional slice
    append labeled slice
    append complete slice
    finalize immutable StaticPack
```

The builder is not a StaticValue and not source-visible.

This prevents O(n²) behavior in P6's future static structural loops and P7's recursive algorithms.

## 8.5 Static evaluator integration

P2's evaluator already supports checked constructor identity and fails closed for unsupported expansions. P3 should extend it using checked facts, not AST spelling.

Required capabilities:

- construct canonical Pack from the exact canonical Pack declaration/constructor;
- evaluate Pack argument contributions in source order;
- expand an existing static Pack by lane;
- expand structurally compatible Tuple at the explicitly supported interoperability boundary;
- reject duplicate labels deterministically;
- return `StaticValue::Pack`;
- compute/check the Pack value type through the canonical TypeStore relation engine;
- charge `QueryBudget` proportional to appended/visited elements;
- check cancellation during large fused expansion/equality/hash;
- never invoke runtime constructors or VM Pack primitives.

## 8.6 Value-domain variadic static parameters

P3 should finally define the deferred semantics for ordinary value-domain pack binders.

Conceptual rules:

```telos
type F<*P: Int>: Int = ...
```

binds `P` as one positional Pack whose every value satisfies `Int`.

```telos
type F<**P: Symbol>: Int = ...
```

binds `P` as one labeled Pack whose every labeled value satisfies `Symbol`.

```telos
type F<***P: T>: ...
```

binds one complete Pack constrained by the declared ordinary value type.

This requires:

1. `StaticParameter` retains `GenericParameterMode` directly;
2. checked static application aligns a variable-length argument contribution set to the variadic parameter;
3. the checker validates every captured value against the parameter's `ValueType(TypeId)` using the existing relation engine;
4. evaluator receives one `StaticValue::Pack` argument for that parameter;
5. zero captured contributions bind `Pack()`;
6. fingerprints include parameter mode;
7. kinded pack binders continue to use C3 `TypeParameterId` and canonical pack terms.

Do not attempt to express ordinary value pack capture by fabricating a `TypeParameterId`.

---

# 9. Runtime Pack representation

## 9.1 Pack needs distinct runtime identity

The preferred runtime model is a dedicated native heap arm:

```rust
Object::Pack(PackObject)
```

plus a canonical Pack runtime class in `CoreClasses`/`UniverseKey`.

This mirrors Tuple's native immutable value-product status while preserving distinct class identity.

Pack should not be an ordinary mutable class `InstanceObject`; doing so would lose the compact product substrate and make view/slice semantics unnecessarily indirect.

## 9.2 Reuse ProductStorage/layout machinery

Pack should reuse the lower product substrate:

```text
logical Pack values
    -> ProductLayout
    -> ProductStorage
    -> PackObject
```

but Pack needs slice/view ranges that Tuple does not currently require.

A recommended conceptual representation is:

```rust
pub struct PackObject {
    // Shared immutable packed backing. Exact spelling may use a store/backing ID
    // rather than Arc if heap ownership provides a better repository-native form.
    backing: Arc<ProductStorage>,

    // Metadata for the backing's ordered positional + labeled coordinates.
    shape: RuntimePackShapeId,

    positional_start: u32,
    positional_len: u32,
    labeled_start: u32,
    labeled_len: u32,
}
```

The shape registry can reuse/extract the existing ordered positional+Symbol-label coordinate structure from Tuple shape metadata. The language-level Pack kind must remain distinct even if the lower shape representation is shared.

## 9.3 Slice backing and GC correctness

If Pack slices share an `Arc<ProductStorage>` or equivalent backing that stores `Value` words containing heap references, GC tracing must keep all references stored in the backing safe for as long as the backing exists.

Two sound implementation patterns are plausible:

### Pattern A — shared backing object/Arc

Every Pack slice directly owns a ref-counted immutable backing. GC tracing walks the backing's traceable value slots. A slice may conservatively keep values outside its visible range alive; this is semantically safe and makes slices O(1).

### Pattern B — root Pack heap object + normalized slice reference

A slice Pack stores an `ObjRef` to a root Pack backing plus ranges. GC traces the root. Slice-of-slice creation must normalize to the root to prevent linear chains.

Pattern A is simpler if `ProductStorage` can safely be shared and traced; Pattern B keeps all storage inside the tracing heap but adds root/slice object complexity.

The patch-grade plan must choose after auditing heap clone/storage constraints. What is forbidden is a raw pointer into another movable/collectable object's buffer or a slice that fails to keep its backing's object references rooted.

## 9.4 Empty Pack runtime representation

Recommended order:

1. `ValueTag::EmptyPack` if tag space and exhaustive match cost are acceptable;
2. otherwise canonical zero-lane `Object::Pack` singleton/root.

Do not use `Value::unit()`.

If `EmptyPack` is added, audit at minimum:

```text
Value::class
Value::type_name / rendering
Value equality/hash/sameness
option/sentinel handling
GC object-ref extraction
compiler constants
serialization/debug formatting
all exhaustive ValueTag matches
```

## 9.5 Runtime Pack equality/hash

Pack is transparent structural value semantics.

Equality and hash must include:

```text
positional length
labeled length
visible label order
visible values in lane order
```

Hash must be consistent with ordinary Telos `==` semantics for child values, including Float's established equality/hash rules.

Do not hash backing pointer, ObjRef, `RuntimePackShapeId`, or slice offsets as language value identity.

## 9.6 Runtime primitives should be minimal observation floor

Following Tuple's architecture, Universe source should define ordinary surface behavior over a narrow native floor.

Useful internal primitives may include:

```text
rawPositionalSize
rawLabeledSize
rawSize
rawAtPositional
rawLabelAt
rawLabeledValueAt
rawPositionalsView / rawLabeledView
rawSlicePositionals / rawSliceLabeled
```

Exact names should fit current conventions.

Do not implement high-level Pack map/filter/iteration in Rust merely because P6/P5 will eventually use them. Prefer source-defined helpers over the minimal structural primitives.

---

# 10. Pack views

## 10.1 Selected public view model

Canonical forms:

```telos
intrinsic data PositionalPackView<T> {
    Empty
    Entry(
        _ value: T,
        _ rest: PositionalPack<T>,
    )
}

intrinsic data LabeledPackView<T> {
    Empty
    Entry(
        _ label: Symbol,
        _ value: T,
        _ rest: LabeledPack<T>,
    )
}
```

The canonical terminology is:

```text
Entry
PositionalPackView
LabeledPackView
PositionalEntries
LabeledEntries
```

Do not revive stale alternatives such as `Item`, `TypePackView`, `Positionals`, or `Labeled` as competing public names.

## 10.2 View semantics are one-step, not recursive storage

`Entry(value, rest)` is an observation result. It does not imply Pack itself is a linked list.

Required complexity:

```text
view selection     O(1)
first              O(1)
rest               O(1)
rest backing       shared
no suffix copying  required
```

The view data value itself may allocate as an ordinary enum/data result if not optimized away. The Pack suffix must not copy its elements.

## 10.3 Static and runtime view parity

Static and runtime Pack views should expose the same logical behavior. P2's static evaluator may need compiler-owned Pack view intrinsics so P6 can later perform structural traversal without a second reflection path.

P3 does not need to implement static `for`; it must make one-step view reduction possible and efficient enough for P6/P7 to build on.

---

# 11. Call-builder convergence

## 11.1 Keep three representations conceptually separate

P3 must preserve:

```text
1. semantic argument Pack
2. first-class immutable runtime Pack
3. private mutable outgoing argument builder/window
```

They may share algorithms, but they are not interchangeable objects.

## 11.2 Fixed call fast path

The ordinary fixed call:

```telos
foo(a, b, c)
```

must continue to compile directly to the existing fixed invocation path.

No Pack should be built merely because “calls semantically have a Pack.”

This is a mandatory performance invariant, not an optional optimizer improvement.

## 11.3 Dynamic call assembly

Dynamic contributions continue through the private builder:

```text
NewArgumentPack
  + positional/labeled contributions
  + expansion contributions
  -> InvokePack
```

`InvokePack` should still be able to consume builder lanes directly into dispatch/argument windows without first allocating immutable `Object::Pack`.

## 11.4 Immutable Pack finalization

Add a distinct builder finalization route:

```text
ArgumentPackBuilderObject
    ├─ consume for InvokePack          // no language Pack object
    ├─ finalize Tuple                  // Tuple expression path
    └─ finalize immutable Pack         // Pack value/rest path
```

The builder should transfer/move its vectors where possible rather than copy them before Pack backing construction.

## 11.5 Multi-spread must be fused

Source:

```telos
foo(*a, *b, *c)
```

must not become:

```text
(((a ++ b) ++ c) ++ ...)
```

with immutable intermediate Packs.

Compiler/VM behavior should be one builder with multiple segments:

```text
builder
  append lane from a
  append lane from b
  append lane from c
  invoke/finalize
```

Total construction should be O(total contributed arguments), aside from semantic duplicate-label hashing/equality work.

## 11.6 Builder duplicate detection

Recommended strategy:

```rust
struct ArgumentPackBuilderObject {
    // existing lanes
    duplicate_index: LabelDuplicateIndex,
}

enum LabelDuplicateIndex {
    SmallLinear,
    Hashed(HashSet<Symbol>),
}
```

The transition threshold should be benchmarked, not guessed into language semantics.

The immutable Pack should not permanently carry this HashSet solely for construction validation.

---

# 12. Rest capture convergence

## 12.1 Primary method dispatch

Current method rest binding materializes Tuple values. P3 must replace those capture results with Pack values while preserving parameter-slot order and selector matching.

Required mapping from normalized runtime `RestMode`:

```text
Positional { param_index }
    -> Pack positional slice/value

Labeled { param_index }
    -> Pack labeled slice/value

Split { positional_param_index, labeled_param_index }
    -> two Pack values

Complete { param_index }
    -> one complete Pack
```

## 12.2 Closure/block invocation

Current closure infrastructure is narrower than method signatures; the compiler's closure parameter shape is principally fixed + positional rest.

P3 should:

- migrate the existing positional closure-rest result from Tuple/List to Positional Pack;
- unify all closure entry paths on the same capture helper;
- **not** invent labeled/complete closure-rest syntax unless the current language specification already admits it independently.

The plan should keep language-surface expansion separate from runtime convergence.

## 12.3 Fiber entry

Fiber activation currently exposes a legacy List rest-capture path for excess closure arguments. This must be removed/converged. A closure's rest binding cannot change type merely because the closure runs as a Fiber entry.

Required invariant:

> Same callable + same argument shape ⇒ same rest value semantics across direct, re-entrant native, and Fiber activation.

## 12.4 Native/re-entrant boundaries

Audit every path that manually reconstructs callable arguments, including:

```text
primitive/block.rs re-entrant call
primitive/fiber.rs entry/resume preparation
invoke_method_object / send_dynamic helpers
callback/observer paths if they bind closure parameters
constructor/factory forwarding where rest metadata is used
```

The goal is one canonical capture helper or one canonical `ParameterShape`/binding-plan materializer, not several Pack constructors with subtly different empty/label behavior.

## 12.5 Zero-copy expectations

Zero-copy rest capture is desirable **where ownership permits**, but not a universal requirement.

A fixed call's arguments live in the VM stack/window; a rest Pack escaping the frame must own/share stable backing. Copying the captured remainder once into immutable backing is sound and still O(n).

A dynamic call already assembled in a builder may be able to transfer owned vectors into a Pack when the callee captures all or a suffix of them. That optimization is worthwhile but should not complicate correctness-critical dispatch in the first P3 slice.

---

# 13. Spread and forwarding convergence

## 13.1 Compiler lowering should dispatch by static knowledge

For an expansion expression, the compiler should use the best known structural path:

```text
known Pack
    -> Pack lane expansion opcode/helper

known Tuple at permitted positional/labeled interoperability boundary
    -> Tuple lane expansion helper

known fixed compile-time shape
    -> possibly flatten directly into fixed call

unknown/dynamic structural operand
    -> checked dynamic expansion path
```

Do not route every spread through generic reflection or Iterator.

## 13.2 Pack-to-Pack forwarding

Canonical example:

```telos
relay(*args)
```

where `args` is already a Positional Pack.

The implementation must not convert:

```text
Pack -> Tuple -> builder
```

or:

```text
Pack -> List -> builder
```

It should append directly from the Pack's visible slice.

## 13.3 Complete Pack forwarding

For:

```telos
relay(***args)
```

preserve:

```text
positionals in order
then labeled lane in its own order
labels unchanged
```

Duplicate labels against earlier/later call-site contributions remain errors at the combined call boundary.

## 13.4 Tuple interoperability

Existing `PackTryExpandTuplePositionals` demonstrates an explicit Tuple→argument positional expansion seam. P3 may retain/generalize that helper.

What matters is that Tuple interoperability remains an **operation**, not a type identity rule.

## 13.5 Pack construction from spread

Explicit Pack construction should itself use the same fused assembly logic:

```telos
Pack(*a, x, **b, ***c)
```

should not repeatedly allocate Packs for each contribution.

---

# 14. Parameter and call-shape semantics

## 14.1 `**args: T` means each labeled value has type T

This existing design decision must remain explicit.

```telos
**args: T
```

means:

```text
for every captured labeled entry:
    value <: T
```

It does **not** mean the entire labeled Pack has type `T`.

## 14.2 Exact labeled Pack shape uses a Pack/row shape form

When an API wants an exact labeled shape, use a structural Pack/type shape, e.g. conceptually:

```telos
**args: **ConfigArgs
```

or the final grammar-equivalent spelling.

Do not reinterpret `**args: ConfigArgs` to mean whole-Pack typing; that would collide with the element-wise parameter annotation rule and erase useful uniform capture semantics.

## 14.3 Runtime ParameterShape remains call-binding authority

`phalcom-core/src/parameters.rs` and `phalcom-core/src/method/mod.rs` already carry normalized fixed/rest call shape.

P3 should extend their value-materialization behavior, not add a Pack-specific selector parser or duplicate call-shape model.

A useful final direction is:

```text
ParameterShape / RestLayout
    -> binding plan
    -> fixed argument slot binding
    -> Pack capture(s) for remainder lanes
```

## 14.4 Labeled binding-plan limitation must be audited

The current generic `binding_plan` helper has restrictions around labeled shapes. P3 should decide whether the primary method path already owns richer labeled binding independently or whether consolidating these code paths is necessary to avoid future Pack divergence.

Do not broaden this into a wholesale call dispatcher rewrite unless evidence shows it is required. The implementation plan should identify the narrowest common capture helper.

---

# 15. Semantic relation and type-checking requirements

P3 must add Pack relation cases through the existing relation engine.

At minimum cover:

## 15.1 Exact fixed Pack equality

```text
Pack<Int, String> == Pack<Int, String>
Pack<Int, String> != Pack<String, Int>
```

## 15.2 Lane sensitivity

```text
Pack<Int, x: String>
    !=
Pack<x: String, Int>   // if syntactically formable with different lanes
```

More generally positional/labeled classification is semantic.

## 15.3 Label sensitivity/order

```text
Pack<x: Int, y: String>
    !=
Pack<y: String, x: Int>
```

for ordered labeled Packs.

This differs intentionally from Record's unordered relation semantics.

## 15.4 Homogeneous families

```text
Pack<Int, Int>
    <: PositionalPack<Int>
```

subject to the language's ordinary subtype rules for element types and invariant/covariant Pack policy selected by C4. The implementation should reuse the existing ordered remainder relation logic rather than ad hoc loops scattered across Pack aliases.

## 15.5 Open tails

Support the P3-owned Pack forms corresponding to:

```text
Closed
Typed(T)
Unconstrained
Captured(rest)
```

where the current type system already has an equivalent representation. Captured symbolic pack terms should remain canonical C3 references, not be expanded into recursive structures.

## 15.6 Duplicate labels after specialization

As with C2 labeled type-pack normalization, symbolic forms may remain valid until substitution proves an overlap. Once concrete specialization makes a duplicate label certain, type formation/materialization must fail deterministically.

## 15.7 No second subtype engine

Pack relation must feed through:

```text
Assignability / relation outcomes
QueryBudget
CancellationToken
existing proof/result taxonomy
```

No Pack-specific boolean-only subtype checker is acceptable.

---

# 16. Static type/value bridge for Pack<Type>

This is one of P3's most subtle requirements.

C4's public ontology wants developer-facing concepts such as:

```text
PositionalTypePack = PositionalPack<Type>
LabeledTypePack    = LabeledPack<Type>
CompleteTypePack   = CompletePack<Type>
```

while C3's compiler uses dedicated kinded terms for efficiency and inference.

The implementation should treat this as a **projection/bridge**, not literal duplication of every C3 term into ordinary `StaticValue::Pack(Type(...))` arrays.

Recommended rule:

```text
When the compiler needs canonical type-level pack algebra:
    retain/reuse C3 TypePack identities.

When static Telos observes/materializes a first-class Pack of Type values:
    expose logical Pack semantics through a view/projection over those terms.
```

Possible implementation techniques include:

- a `StaticValue::Pack` backed by a lightweight projection over C3 pack terms;
- a specialized StaticPack backing variant that references canonical TypePack terms and lazily exposes Type child values;
- canonical conversion cached at the StaticValueStore boundary.

Whichever technique is chosen must preserve:

- one semantic TypeId authority;
- no recursive copying of large type packs merely to expose a first-class view;
- structural Pack equality consistent with canonical Type equality;
- no observable distinction based on whether a Pack came from C3 internal representation or explicit Pack construction.

P3 should not force all internal TypePack inference/substitution operations through general StaticValue evaluation. That would be a major performance and architectural regression.

---

# 17. Runtime Universe/source integration

## 17.1 Add canonical Pack declaration in Universe source

A source-owned declaration should be added in the appropriate collections/structural module, conceptually:

```telos
intrinsic data Pack<***Es>(***Es)
```

plus source aliases/views.

Do not implement Pack only as a Rust class with no canonical source declaration; C3 established source/native/intrinsic parity expectations.

## 17.2 Add stable native/runtime identity

Likely changes include:

```text
UniverseKey::Pack
CoreClasses::pack_class
IntrinsicRuntimeOwner::Pack
canonical declaration binding for Pack
Object::Pack
Heap Pack accessors/allocation
Value::class mapping
GC tracing
native surface entries/primitives
```

The exact registry names should follow current Tuple/Record patterns.

## 17.3 Pack special representation must use exact declaration identity

Do not detect Pack by:

```text
name == "Pack"
module path string at arbitrary call sites
selector spelling
```

Resolve the canonical Universe declaration/runtime owner once and carry that semantic identity through lowering.

## 17.4 Source behavior over minimal primitives

Pack's public `size`, `first`, `rest`, lane views, equality helpers, and convenience operations should be written in Telos source where practical, using compiler/native intrinsics only for structural floor operations.

This keeps P5/P6 extensibility aligned with the “ordinary Telos semantics” principle.

---

# 18. Compiler and bytecode requirements

## 18.1 Do not destabilize fixed invoke opcodes

P3 should preserve existing exact call opcodes for statically known calls.

A new first-class Pack feature does not imply replacing all calls with `InvokePack`.

## 18.2 New/adjusted bytecode seams

Likely required additions or generalizations:

```text
FinishPack                 // private builder -> immutable Pack
PackExpandPositionals      // append visible positional lane from Pack
PackExpandLabels           // append visible labeled lane
PackExpandComplete         // append both visible lanes
```

Existing Tuple expansion instructions can remain where Tuple semantics require them. Naming may be generalized if one opcode soundly handles both Tuple and Pack with explicit runtime checks.

## 18.3 BuildStaticPack fast path

If a Pack literal/construction has statically known shape and values are already on the stack, consider a direct:

```text
BuildStaticPack(shape_id/counts)
```

analogous to static Tuple/Record construction, bypassing the mutable builder.

This is recommended if it materially reduces overhead and integrates cleanly with product layout registration. It is not a reason to complicate the first functional slice; a builder-finalize path is semantically sufficient initially.

## 18.4 Bytecode stack effects must remain explicit

New Pack bytecodes must document:

- exact operand order;
- builder/value stack effect;
- source evaluation order;
- duplicate-label failure timing;
- whether operand Pack remains live/consumed;
- GC rooting of partially assembled values.

## 18.5 Compiler virtualization

A Pack value may be:

```text
unmaterialized / scalarized
compiler-virtual
materialized Object::Pack
```

when escape analysis/product scalar replacement can prove equivalence.

P3 need not create a new optimizer solely for Pack. It should ensure Pack representation participates in existing product scalar-replacement/materialization mechanisms instead of blocking them with observable allocation identity.

---

# 19. Evaluation order and error semantics

Pack construction and call expansion must preserve Telos source evaluation order exactly.

For:

```telos
f(a(), *p(), label: b(), **q())
```

required order is the source order of contribution evaluation. Label duplicate errors happen when the labeled contribution is admitted to the builder, after preceding expressions have produced their ordinary effects as already defined by call argument semantics.

Do not pre-scan all dynamic labels in a way that reorders evaluation.

Static evaluator behavior must mirror source order while remaining deterministic and side-effect restricted.

---

# 20. Fingerprints, incrementality, and store lifetime

## 20.1 Static Pack fingerprint

`StaticValueStore::content_fingerprint` must incorporate logical content:

```text
Pack tag
visible positional length
visible labeled length
positional child fingerprints in order
labeled Symbol spelling + child fingerprint in order
```

Do not fingerprint:

```text
StaticPackId
backing Arc pointer
slice backing ID alone
arena insertion order
VM Symbol ID
source range
```

Two equal logical slices over different backings must produce the same content fingerprint.

## 20.2 Static Store lineage

Pack backings/slices must obey P2's existing store-qualified lineage rules:

- children from foreign lineages rejected;
- clone-on-write snapshots preserve old refs;
- append does not renumber prior nodes/backings;
- slice ranges never outlive their store lineage.

## 20.3 Type fingerprints

Any new `TypeData::Pack`/`PackFromPacks` form or equivalent must participate in:

```text
TypeStore interning
semantic formatting
substitution
instantiation
type lambda scoped forms if applicable
inference visitation
relation visitation
dependency/fingerprint hashing
metadata export/import where required
```

A missing visitor arm is a correctness bug, not a benign fallback.

## 20.4 Metadata boundary

P8 owns durable static-value/residual products, but P3 Pack **types** may appear in public ordinary signatures before P8.

Therefore:

- if the selected Pack type representation is already expressible through existing stable `Applied`/pack terms, reuse it;
- if P3 introduces a new ordinary Pack type node that must cross module boundaries, extend type metadata minimally and version it according to current metadata rules;
- do **not** serialize `StaticPackId`, store lineage IDs, runtime Pack backing IDs, or static evaluator frames;
- durable static Pack *values* remain P8 scope unless an explicit P3 gate requires a narrow representation.

---

# 21. Performance and memory requirements

P3 has unusually strong performance obligations because Pack lies directly on call paths.

## 21.1 Mandatory asymptotics

```text
Pack lane length                  O(1)
first/rest view construction     O(1)
slice/rest Pack construction     O(1)
Pack structural equality         O(n)
Pack structural hash             O(n)
mutable builder append           amortized O(1)
finalize/materialize whole Pack  O(n)
fused multi-spread               O(total contributed args)
fixed call Pack allocation       0 required
```

## 21.2 No recursive cons representation

Do not implement:

```text
Pack = Empty | Cons(value, rest)
```

as physical storage.

The public View enum is observational only.

## 21.3 Avoid suffix-copy pathologies

The following must not become O(n²):

```telos
let v1 = pack.View
let v2 = v1.rest.View
let v3 = v2.rest.View
...
```

or future static recursion over Pack tails.

## 21.4 Avoid append-copy pathologies

Do not implement a static/runtime Pack builder by repeatedly constructing:

```text
new_pack = old_pack + [item]
```

with immutable array copies.

Use mutable staging + one final immutable backing.

## 21.5 Memory retention tradeoff for slices

Shared backing means a small suffix can retain a large original Pack backing. This is acceptable for P3 if documented and benchmarked; copying every suffix would violate the core complexity requirement.

A future optimizer may compact long-lived tiny slices when profitable, but this must be an optimization invisible to semantics.

## 21.6 Fixed-call regression is a blocking failure

Any measurable systematic regression in fixed-call throughput/allocation due solely to Pack introduction should block P3 acceptance until explained/fixed.

---

# 22. Required test architecture

P3 needs layered tests, not one giant end-to-end corpus gate.

## 22.1 AST/parser tests

Cover:

```text
Pack type application
* / ** / *** argument expansion remains unchanged
anonymous Pack typed remainders *:T / **:T / ***:T
no `...` pack syntax
value-domain variadic static parameters preserve mode
```

## 22.2 Type-system tests

Cover:

```text
exact Pack type canonicalization
positional/labeled/complete lane distinction
label order sensitivity
duplicate labels
homogeneous PositionalPack/LabeledPack/CompletePack families
open typed/unconstrained remainder relation
symbolic pack expansion -> Pack type
substitution collapses symbolic PackFromPacks when concrete
C3 TypePack bridge identity
no Tuple<->Pack implicit equivalence
```

## 22.3 StaticValue tests

Cover:

```text
Pack() distinct from Unit/Record/Tuple
positionals
labeled
complete
nested Pack
semantic equality/hash
content fingerprint independent of backing identity
foreign child rejection
duplicate label rejection
O(1)-shape slice representation evidence
store clone/lineage stability
```

## 22.4 Static evaluator tests

Cover:

```text
closed const Pack construction
Pack expansion
Tuple->Pack explicit expansion
Pack->Pack forwarding
value-domain *P/**P/***P static parameter capture
zero capture -> Pack()
result type checking
budget/cancellation during large Pack work
```

## 22.5 Runtime representation tests

Cover:

```text
Object::Pack / EmptyPack class identity
GC traces nested object values through Pack backing
Pack slice keeps backing children alive
slice-of-slice remains correct
structural equality/hash
empty Pack hashing/equality
labels preserved exactly
```

## 22.6 Call/rest tests

Cover all activation boundaries:

```text
method positional rest -> Positional Pack
method labeled rest -> Labeled Pack
method complete rest -> Complete Pack
method split rest -> two Packs
empty rest -> Pack()
closure positional rest -> Pack
re-entrant block call rest -> Pack
Fiber closure rest -> same Pack semantics
forward captured rest via *
forward captured labeled rest via **
forward complete via ***
```

## 22.7 Builder/spread tests

Cover:

```text
multiple positional spreads
multiple labeled spreads
multiple complete spreads
mixed fixed + spread contributions
computed labels
late duplicate after earlier side effects
large labeled duplicate detection
no intermediate first-class Pack allocation for InvokePack-only dynamic send
```

## 22.8 Fixed-call codegen guard

Retain and extend the existing assertion that simple fixed calls contain none of:

```text
NewArgumentPack
PackPush*
PackExpand*
InvokePack
FinishPack
```

unless the source explicitly requires a first-class Pack.

## 22.9 Compatibility gates

P3 must preserve C1–C3 suites for:

```text
TypePack/LabeledTypePack/CompleteTypePack
variadic generic application
pack concat
TupleFromPacks
CallableFromPacks
whole-pack inference
metadata validation
source-index identity
```

P2 focused suites for static values/static definitions/static constants must also remain green.

---

# 23. Performance certification matrix

At minimum benchmark or instrument:

| Scenario | Required observation |
|---|---|
| fixed positional call | no Pack allocation; throughput within noise/budget |
| fixed labeled call | no Pack allocation |
| generic fixed call | no forced Pack materialization |
| one positional spread | O(n), one builder or direct segment |
| three positional spreads | O(total), no immutable concat chain |
| labeled spread 10/100/1000 | no O(n²) duplicate scan cliff |
| positional rest 10/100/1000 | one Pack backing, no Tuple/List bridge |
| rest forwarding | no Pack→Tuple/List conversion |
| repeated Pack View/rest | O(1) per step |
| static Pack build 10/100/1000 | near-linear |
| static Pack tail recursion preparation | no suffix copies |
| Pack equality/hash 10/100/1000 | linear |

Use allocation counters or focused heap instrumentation where available; do not rely only on wall-clock timing.

---

# 24. Source-of-truth / ownership table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| `*`/`**`/`***` source mode | AST `ExpansionMode` / `RestMode` | checker/compiler | P3-only spread syntax enum |
| generic binder mode | `GenericParameterMode` | TypeStore/inference/static defs | infer from kind |
| C3 type pack identity | TypeStore `TypePack` family | inference/substitution/bridge | new PackId type-level arena |
| proper first-class Pack type | P3 TypeStore Pack form / exact equivalent | checker/static evaluator/compiler | treating `TypePack` as proper type |
| static Pack value | P2 `StaticValueStore` extended by P3 | evaluator/const products | runtime `Value` in semantic crate |
| runtime Pack value | `Object::Pack` / EmptyPack | VM/source primitives | `TupleObject` identity |
| dynamic outgoing assembly | private `ArgumentPackBuilderObject` | compiler/VM | public mutable Pack builder |
| rest call shape | `ParameterShape` / method `RestLayout` | dispatch/capture | selector-text reparsing |
| Pack labels | canonical Symbol identity | static/runtime Pack | P4 FieldKey in P3 |
| Pack semantic equality | ordinary Telos value equality + lane structure | static/runtime | ObjRef/backing identity |
| static value fingerprint | StaticValueStore content fingerprint | SemanticDb | StaticPackId/backing pointer |
| Pack storage | product/backing substrate | VM | recursive cons cells |
| Pack views | source view types + slice ranges | source/static traversal | recursively materialized suffixes |

---

# 25. Global P3 invariants

The implementation plan should assign stable IDs to these invariants and make tasks/tests reference them.

- **P3-INV-01** — Pack is first-class, immutable, and structurally value-semantic.
- **P3-INV-02** — Pack is distinct from Tuple, Record, and Unit.
- **P3-INV-03** — `Pack()` is not Unit or EmptyRecord.
- **P3-INV-04** — Pack has positional and labeled lanes; lane identity is semantic.
- **P3-INV-05** — labeled Pack order is semantic and labels are Symbol in P3.
- **P3-INV-06** — duplicate Pack labels are invalid.
- **P3-INV-07** — `*`, `**`, `***` remain positional/labeled/complete; `...` is not introduced.
- **P3-INV-08** — C1–C3 TypePack terms remain canonical internal kinded terms.
- **P3-INV-09** — a first-class Pack type is not represented by pretending a C3 TypePack term is a proper Type.
- **P3-INV-10** — public Pack<Type> ontology bridges to C3 terms without duplicate semantic authority.
- **P3-INV-11** — StaticValue Pack lives in the P2 StaticValueStore lineage.
- **P3-INV-12** — static Pack nodes contain no runtime Value/ObjRef/VM Symbol.
- **P3-INV-13** — static Pack fingerprints depend on logical content, not arena/backing identity.
- **P3-INV-14** — static Pack slicing/rest is O(1) structural work.
- **P3-INV-15** — runtime Pack slicing/rest is O(1) structural work.
- **P3-INV-16** — Pack physical storage is flat/shared, never recursive cons.
- **P3-INV-17** — fixed calls require no first-class Pack allocation.
- **P3-INV-18** — private ArgumentPackBuilder remains unobservable.
- **P3-INV-19** — multi-spread is O(total arguments), not repeated immutable concat.
- **P3-INV-20** — rest capture yields Pack, never Tuple/List.
- **P3-INV-21** — all callable activation paths agree on rest semantics.
- **P3-INV-22** — Tuple/Pack interoperability is explicit operation, not implicit type identity.
- **P3-INV-23** — Record is not implicitly a LabeledPack subtype/equivalent.
- **P3-INV-24** — value-domain variadic static parameters bind one Pack value.
- **P3-INV-25** — kinded variadic static parameters continue using C3 TypeParameter/TypePack machinery.
- **P3-INV-26** — static parameter mode is preserved independently of domain/kind.
- **P3-INV-27** — Pack type relations reuse the canonical relation/budget/cancellation engine.
- **P3-INV-28** — Pack construction/spreading preserves source evaluation order.
- **P3-INV-29** — dynamic duplicate-label acceleration is construction-only; no permanent map is required in Pack.
- **P3-INV-30** — GC traces all object references retained by shared Pack backing.
- **P3-INV-31** — slice backing ownership cannot dangle across GC or frame lifetime.
- **P3-INV-32** — no P4 generalized FieldKey semantics leak into P3.
- **P3-INV-33** — no P5 structural capability/reflection dependency is required for basic Pack operation.
- **P3-INV-34** — no P6 Iterator/static-for machinery is required for spreading/rest.
- **P3-INV-35** — durable static Pack serialization is deferred to P8 unless ordinary type metadata requires a narrow Pack type node.
- **P3-INV-36** — P9 may retire old public TypePack names without replacing P3's internal C3 bridge.

---

# 26. Recommended implementation decomposition

This is a requirements-analysis decomposition, not yet the final Luna patch-grade plan. Exact task numbers may change after one final source audit, but the dependency order should remain close to this.

## T0 — takeover and baseline freeze

- verify `main` remains at or after `5a5bf8b...`;
- verify checkpoint still names P3 active and P2 latest completed;
- record baseline focused tests and known Category-D failures;
- census Pack terminology and any post-P2 changes in call/rest paths;
- census all `finish_tuple`/`alloc_list` uses in rest binding;
- census fixed-call Pack-builder bytecode tests.

**Gate:** no P3 source edit until P2 closure/lifecycle state remains valid.

## T1 — Pack source/type syntax and canonical semantic type

- add canonical Universe `Pack` declaration and public aliases;
- add anonymous generic Pack remainder syntax needed by `Pack<*:T>` etc.;
- establish proper Pack TypeStore representation;
- establish exact declaration/runtime owner identity;
- preserve C3 pack terms and define projection bridge;
- add core relation/format/substitution/inference visitor coverage.

**Gate:** type-only Pack examples normalize deterministically; no runtime changes yet required.

## T2 — static Pack value + value-domain variadic static parameters

- extend `StaticValueStore` with Pack backing/slice nodes;
- implement semantic equality/hash/fingerprint;
- add Pack static type projection/check;
- add `mode` to `StaticParameter` or equivalent canonical fact;
- extend checked static applications for value-domain variadic capture;
- extend evaluator with Pack build/spread/view foundation;
- retain budget/cancellation.

**Gate:** closed static Pack constants and variadic static definitions pass focused tests.

## T3 — runtime Pack object, empty representation, GC, primitives

- add `UniverseKey::Pack`/class identity;
- add Pack heap representation and optional `EmptyPack` tag;
- integrate ProductStorage/shared backing/ranges;
- add exhaustive GC tracing;
- add value class/render/equality/hash support;
- expose minimal internal primitives.

**Gate:** Pack values survive GC, compare/hash correctly, and slices remain valid.

## T4 — private builder immutable-Pack finalization

- add `finish_pack`/equivalent;
- add/move-owned builder finalization;
- add thresholded duplicate-label index;
- preserve `InvokePack` direct consumption;
- preserve `FinishTuplePack` for Tuple construction.

**Gate:** dynamic Pack construction works with no fixed-call regression.

## T5 — rest capture convergence

- migrate primary method rest;
- migrate Split/Complete modes;
- migrate block/closure re-entry;
- remove Fiber List capture;
- audit native/callback activation boundaries;
- centralize capture behavior where practical.

**Gate:** same callable produces same Pack rest semantics on every activation path.

## T6 — spread/forwarding convergence

- append Pack lanes directly to outgoing builder;
- preserve Tuple explicit expansion support;
- support Pack→call, Pack→Pack, Pack→Tuple explicit boundaries;
- fuse multi-spread;
- validate duplicate labels after combined contributions;
- maintain evaluation order.

**Gate:** forwarding contains no Tuple/List bridge and is O(total arguments).

## T7 — Pack views and source surface

- add `PositionalPackView`/`LabeledPackView` source data;
- add `View`/entry primitives or compiler intrinsics;
- prove O(1) rest/slice behavior static/runtime;
- add source helpers without broad P5 reflection.

**Gate:** repeated one-step decomposition does not copy suffixes.

## T8 — incrementality, metadata boundary, compatibility

- complete fingerprints/visitor arms;
- add ordinary Pack type metadata if required by cross-module signatures;
- keep StaticPack durable value serialization deferred to P8;
- run C1–C3 pack/type compatibility and P2 static compatibility;
- verify source-index/diagnostic identity.

**Gate:** warm/cold semantic parity for Pack type edits and unchanged C1–C3 behavior.

## G1 — semantic/static Pack vertical

Required examples include:

```telos
const Empty = Pack()
const P = Pack(1, 2, name: #x)

type CountInts<*P: Int>: Int {
    // P is a positional Pack value; concrete body operation limited to P3-supported view operations.
    ...
}
```

plus type relations/bridge tests.

## G2 — runtime call/rest vertical

Required examples include:

```telos
capture(*args) { args }
captureLabeled(**args) { args }
captureComplete(***args) { args }

let p = capture(1, 2, 3)
forward(*p)
```

and method/closure/Fiber parity.

## G3 — fixed-call/performance gate

- fixed call bytecode contains no Pack assembly;
- allocation counters show zero Pack object for fixed no-rest calls;
- large labeled spread avoids quadratic duplicate scan;
- repeated views remain O(1) construction;
- multi-spread is linear in total elements.

## G4 — compatibility/stabilization gate

Focused suites across:

```text
phalcom-ast
phalcom-semantic
phalcom-modules (if metadata/interface touched)
phalcom-core
phalcom-type-meta (if Pack ordinary type node added)
phalcom-lsp (if source AST/token presentation changed)
```

plus C1–C3 pack regressions and P2 static suites.

## T9 — lifecycle closure

Only after actual implementation/evidence:

- update `LANG007.C4-CHECKPOINT.md`;
- mark P3 COMPLETE/IMPLEMENTED with truthful verification classification;
- write P3 walkthrough recording what actually shipped;
- write P3 handoff for P4/P5 takeover;
- do not mark P5 unblocked until both P3 and P4 are semantically stable as required by the checkpoint DAG.

---

# 27. Risk register

| Risk | Severity | Why it matters | Required mitigation |
|---|---:|---|---|
| fixed calls start allocating Packs | Critical | call-path regression everywhere | codegen assertions + allocation benchmark gate |
| Pack collapsed into Tuple identity | Critical | violates language semantics, rest typing, future reflection | dedicated semantic/runtime identity; shared storage only |
| first-class Pack replaces C3 TypePack internals | Critical | breaks inference/substitution/metadata and adds evaluator overhead | explicit bridge, preserve C3 authority |
| static value-domain pack mode lost | High | `*P: Int` cannot be soundly bound | retain `GenericParameterMode` in StaticParameter/signature |
| Pack slices copy arrays | High | P6/P7 traversal becomes O(n²) | shared backing + range view |
| shared backing GC under-traces hidden values | Critical | use-after-free/stale ObjRef | trace complete backing or root object soundly |
| builder duplicate detection remains linear | High | labeled spread construction O(n²) | small-linear + spill hash/index |
| Record treated as LabeledPack | High | ordered/unordered semantics collapse before P4 | explicit distinction/conversion boundary |
| `FinishTuplePack` blindly replaced | High | breaks Tuple expanded literal semantics | separate tuple vs Pack finalization routes |
| Fiber/direct closure rest differ | High | activation-path observable type drift | unified capture helper and parity tests |
| new Pack type misses one TypeStore visitor | High | unsound stale/symbolic types or panics | exhaustive search + visitor checklist |
| metadata freezes session IDs | Critical | cross-process corruption | serialize structural type meaning only; P8 for StaticValue IDs |
| Pack view public API drifts in naming | Medium | P5/P6 source ecosystem churn | use ratified Entry/PositionalPackView/LabeledPackView names |
| P3 implements generalized FieldKey | Medium | entangles P4 and expands blast radius | Symbol-only labels in P3 |
| spread implemented via Iterator | High | dynamic dispatch/perf/semantic mismatch | structural lane primitives only |

---

# 28. Consultation / stop conditions for the patch-grade plan

The implementation agent should STOP AND CONSULT if any of these become true:

1. the latest checkpoint no longer names P3 active;
2. Pack proper type cannot be represented without globally replacing C3 pack kinds;
3. open typed Pack family semantics would require a new general pack partition solver rather than reusing ordered remainders;
4. runtime O(1) slices cannot be made GC-safe with current ProductStorage ownership;
5. fixed calls would need `NewArgumentPack` merely to support Pack semantics;
6. value-domain variadic static parameters cannot be added without changing TypeParameter identity rules from P2;
7. Pack type metadata requires serializing process-local StaticValue/TypeStore IDs directly;
8. Tuple/Record relation code would need to be weakened to make Pack fit;
9. Record must become implicitly ordered to satisfy `**` rest;
10. a proposed optimization changes source evaluation order or duplicate-error timing;
11. Pack runtime identity would require observable mutable builder state;
12. a P3 change requires P5 capability/reflection to type-check basic Pack operations;
13. static Pack evaluation would require arbitrary runtime/native method dispatch;
14. a GC implementation would keep raw borrowed pointers into movable/collectable object storage;
15. compatibility failures indicate P3 changed `Single(TypePack)` vs variadic binder semantics.

---

# 29. Concrete repository changes likely required

This is a targeting map, not a promise that every listed file must change.

## AST/parser

```text
phalcom-ast/src/ast.rs
    generic application anonymous Pack remainder syntax

phalcom-ast/src/parser.rs
    parse *:T / **:T / ***:T in Pack type application contexts

phalcom-ast/tests/parser.rs
```

Existing `ExpansionMode`, `RestMode`, and `PackItem` should be reused.

## Semantic types

```text
phalcom-semantic/src/types/store.rs
    proper Pack type / symbolic Pack-from-lanes representation

phalcom-semantic/src/types/annotation.rs
    Pack application/remainder lowering

phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/checker/inference.rs
    visitor/normalization/relation integration
```

## Static semantics

```text
phalcom-semantic/src/static_value/{mod.rs,store.rs}
    StaticValue::Pack + backing/slice

phalcom-semantic/src/static_definition.rs
    retain parameter mode

phalcom-semantic/src/checker/{expression.rs,static_body.rs}
    checked Pack construction/value variadic application

phalcom-semantic/src/static_eval/**
    Pack build/spread/view/application capture

phalcom-semantic/src/db/fingerprint.rs
    parameter mode / Pack content fingerprints
```

## Declarations/Universe identity

```text
phalcom-semantic/src/declarations.rs
phalcom-native-meta/src/universe.rs
phalcom-core/src/universe/core_classes.rs
phalcom-core/core/universe/src/collections/pack.ph  // likely new
```

## Runtime storage

```text
phalcom-core/src/heap/pack.rs           // likely new PackObject
phalcom-core/src/heap/object.rs
phalcom-core/src/heap/mod.rs
phalcom-core/src/heap/accessors.rs
phalcom-core/src/heap/trace.rs
phalcom-core/src/product/{shape.rs,layout.rs,storage.rs,mod.rs,view.rs}
phalcom-core/src/value/{repr.rs,mod.rs,render.rs}
```

## Call/rest/compiler

```text
phalcom-core/src/heap/pack_builder.rs
phalcom-core/src/parameters.rs
phalcom-core/src/method/mod.rs
phalcom-core/src/bytecode.rs
phalcom-core/src/compiler/lib/{mod.rs,expr.rs}
phalcom-core/src/vm/dispatch.rs
phalcom-core/src/primitive/{block.rs,fiber.rs}
```

## Metadata if required

```text
phalcom-type-meta/src/{type_node.rs,scoped_type.rs,validate.rs,...}
phalcom-semantic/src/metadata/**
```

Do not add durable StaticPack value metadata in P3 merely because Pack type metadata changes.

---

# 30. Key repository findings that should shape the implementation plan

## Finding F1 — P3 is not a parser-first feature

The three expansion/rest lanes already exist in AST and runtime signature metadata. The dangerous work lies in semantic/value/runtime convergence.

**Planning consequence:** keep the parser task narrow; spend design/testing effort on type identity, StaticValue, GC-safe slices, activation-path rest convergence, and call-path performance.

## Finding F2 — the private builder is an asset, not technical debt to delete

`ArgumentPackBuilderObject` already encodes the exact mutable staging abstraction C4 calls for.

**Planning consequence:** extend finalization/large-label duplicate detection; do not replace it with the immutable Pack surface object.

## Finding F3 — rest semantics are currently observably inconsistent internally

Method, block, and Fiber paths do not all use the same result representation.

**Planning consequence:** create one capture semantic/helper and write parity tests before broad surface helpers.

## Finding F4 — Tuple's product substrate is reusable, Tuple identity is not

ProductShape/ProductLayout/ProductStorage and GC tracing are reusable. TupleObject/class/equality identity is not.

**Planning consequence:** share lower storage and relation helpers, create Pack-specific semantic/runtime identity.

## Finding F5 — static Pack mode is the exact P2-deferred seam

AST retains variadic static parameter mode; semantic `StaticParameter` currently does not.

**Planning consequence:** P3 should make parameter mode a first-class semantic signature fact and use it for ordinary value Pack capture, while leaving kinded C3 binding unchanged.

## Finding F6 — `Pack<*:T>` needs syntax/typing work not supplied by C2 expansions

C2's `*P` means expansion of a pack term. C4's `*:T` means an anonymous typed tail.

**Planning consequence:** reuse anonymous remainder concepts; do not try to parse `*:T` as a malformed expansion operand.

## Finding F7 — fixed-call performance is already testable

Existing outgoing-pack codegen tests assert no builder opcodes for fixed calls.

**Planning consequence:** make this an early regression test and final blocking performance gate.

## Finding F8 — P2 StaticValue architecture is ready for additive Pack support

No store replacement is necessary.

**Planning consequence:** extend P2 lineage/fingerprint semantics; do not create `StaticPackStore` as a separate lifetime authority.

---

# 31. Recommended acceptance examples

The final patch-grade plan should turn these into executable tests using actual supported source syntax.

## 31.1 Distinct values

```telos
let u = ()
let r = {}
let p = Pack()

// p is Pack; it is not Unit or Record.
```

## 31.2 Rest capture

```telos
capture(*args) {
    args
}

const_or_runtime_result = capture(1, 2, 3)
// result is positional Pack [1,2,3]
```

## 31.3 Labeled rest

```telos
capture(**args) {
    args
}

capture(a: 1, b: 2)
// ordered labeled Pack: [#a:1, #b:2]
```

## 31.4 Complete rest

```telos
capture(***args) {
    args
}

capture(1, 2, a: 3, b: 4)
```

preserves both lane orders.

## 31.5 Forwarding

```telos
relay(***args) {
    target(***args)
}
```

No Tuple/List conversion is permitted.

## 31.6 Multi-spread

```telos
target(*left, *middle, *right)
```

uses one fused assembly path.

## 31.7 Explicit Tuple interoperability

```telos
let tuple = (1, 2, 3)
let pack = Pack(*tuple)
let tuple2 = (*pack)
```

using the repository's final Tuple construction syntax; no implicit equality is introduced.

## 31.8 Static Pack

```telos
const P = Pack(1, 2, 3)
```

reduces through `StaticValue::Pack` and never runs a VM constructor at compile time.

## 31.9 Value-domain static variadic parameter

```telos
type CountInts<*P: Int>: Int {
    // body uses P3 one-step Pack view semantics
    ...
}
```

The signature must identify `P` as `PositionalPack` mode + `ValueType(Int)` and bind one Static Pack value.

## 31.10 C3 type-pack continuity

Existing kinded code such as:

```telos
type TupleOf<*P> = Tuple<*P>
```

continues to use C3 TypePack binding/inference rather than first-class value Pack capture.

---

# 32. Non-goals and anti-patterns

P3 must reject the following implementation shortcuts:

```text
Pack == Tuple with a new alias
Pack stored as List
Pack stored as recursive cons cells
Pack semantics implemented by Iterator expansion
surface exposure of ArgumentPackBuilderObject
all calls lowered through InvokePack
all fixed calls allocate Pack
rest capture still returns Tuple in one activation path
rest capture still returns List in Fiber path
Record implicitly satisfies LabeledPack
C3 TypePack terms replaced by StaticValue arrays
Pack StaticValue contains VM Value/ObjRef
Pack slice copies the entire suffix
large labeled builder keeps linear duplicate scan forever
new Pack-specific subtype engine
new Pack-specific generic inference variable arena
using `...` as new spread syntax
serializing StaticPackId/backing pointers as durable metadata
```

---

# 33. P3 → later-plan handoff requirements

## To P4

P3 must hand off:

```text
Pack labels fixed to Symbol
ordered labeled-lane APIs clearly isolated
no assumption that Record shares Pack label identity
Pack backing/view APIs that can remain Symbol-specific
```

P4 can then generalize unordered record keys independently.

## To P5

P3 must hand off:

```text
stable first-class Pack identity
stable ordered lane structure
stable O(1) views
canonical Pack type shape authority
```

P5 can derive `OrderedProduct`/reflection from these lower facts rather than Pack reflection proving Pack structure recursively.

## To P6

P3 must hand off:

```text
static Pack value
one-step static Pack views
explicit evaluator Pack nodes
budgeted traversal primitives
```

P6 can implement structural `for` over Pack without Iterator and without inventing static-only storage.

## To P7

P3 must hand off O(1) Pack tails. Otherwise recursive static algorithms such as:

```text
Count<P>
  -> Count<P.rest>
  -> ...
```

would become pathologically expensive even if P7's recursion/memoization is correct.

## To P8

P3 must hand off semantic-content Pack fingerprints and representation-independent StaticPack meaning. P8 can serialize a stable DAG without dependence on store-local IDs/backing pointers.

## To P9

P3 must hand off a working public Pack ontology plus an explicit bridge to legacy C3 TypePack terms. P9 can then remove/retire old public names without destabilizing compiler internals.

---

# 34. Final planning judgment

C4.P3 is substantial but architecturally bounded. The repository is already unusually well prepared for it: syntax lanes, generic modes, type-level pack composition, dynamic argument builders, product storage, and static-value infrastructure are all present. The principal implementation risk is therefore **not missing infrastructure**; it is accidentally collapsing distinct existing abstractions while trying to “unify” them.

The correct convergence is:

```text
C1–C3 TypePack algebra
          │
          │ canonical bridge for Type-valued Pack structure
          ▼
P3 proper Pack type/value semantics
          │
          ├──────────── static ────────────> P2 StaticValueStore
          │
          └──────────── runtime ───────────> immutable Pack backing
                                                 ▲
                                                 │ finalization only when needed
                                       private ArgumentPackBuilder
                                                 ▲
                                                 │
                                      dynamic spread/call assembly
```

not:

```text
replace every existing pack-like concept with one concrete object
```

The patch-grade P3 plan should therefore optimize for **authority preservation**:

1. reuse existing lane syntax;
2. reuse C3 type-pack algebra;
3. extend P2 StaticValue additively;
4. reuse product storage, not Tuple identity;
5. retain the private mutable builder;
6. migrate every rest-capture boundary to immutable Pack;
7. fuse spreads without materializing intermediate Packs;
8. prove fixed calls remain allocation-free;
9. establish value-domain variadic static parameters as the precise P2-deferred Pack integration;
10. leave P4–P9 additive seams rather than premature implementations.

With those constraints, P3 can be implemented without a second call system, second type-pack system, second static-value store, or compile-time/runtime Pack split. It becomes what C4 intends: one coherent structural transport value whose semantic meaning is stable across static evaluation, runtime capture, forwarding, and later structural reflection.

---

# 35. Preparation checklist for the subsequent Luna patch-grade implementation plan

Before converting this requirements analysis into the executable `LANG007.C4.P3` plan, re-read the live repository and confirm each item:

- [ ] checkpoint still names P3 active;
- [ ] latest `main` revision recorded;
- [ ] no post-analysis Pack implementation has landed;
- [ ] all `finish_tuple` rest-capture sites enumerated;
- [ ] all `alloc_list` rest/closure capture sites enumerated;
- [ ] all `RestMode::{Positional,Labeled,Split,Complete}` activation sites enumerated;
- [ ] all Pack builder bytecodes and compiler emission sites enumerated;
- [ ] current fixed-call no-builder tests frozen as baseline;
- [ ] `StaticParameterSyntax.mode` → semantic parameter-mode retention decision fixed;
- [ ] proper Pack type representation selected;
- [ ] `Pack<*:T>` parser/AST representation selected;
- [ ] C3 TypePack bridge direction fixed;
- [ ] runtime Pack backing/slice ownership selected after GC audit;
- [ ] EmptyPack representation selected after ValueTag audit;
- [ ] product shape/layout reuse boundary selected;
- [ ] runtime equality/hash integration points enumerated;
- [ ] StaticValue Pack interning-vs-slice identity rule fixed;
- [ ] metadata responsibility split between P3 ordinary types and P8 static values fixed;
- [ ] focused test budget established before broad crate gates;
- [ ] fixed-call allocation/performance gate made blocking;
- [ ] walkthrough/handoff lifecycle outputs reserved for actual closure, not planning.
