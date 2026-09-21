---
id: LANG007.C1.P1
category: LANG
program: LANG007
checkpoint: LANG007.C1
kind: implementation-plan
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
depends_on:
  - LANG007.C1 semantic/specification convergence already completed
follows: LANG007.C1.P1 requirements analysis
supersedes: null
repository: aureat/phalcom-lang
branch: main
planning_revision: 5b63ade8073d13d6b528af4774181665c30cbd47
planning_revision_subject: "fix(core): restore standalone semantic universe"
prepared: 2026-09-21
executor: GPT-5.6 Luna High/Extra High
scope_note: code implementation only; documentation/specification updates are explicitly excluded by user instruction
---

# LANG007.C1.P1 — Semantic Domains and Compiler-Intrinsic Structural Type Operations

## 0. Executor contract

This is a **Luna-oriented patch-grade implementation plan** for the first code implementation slice of LANG007.

The specifications and planning documentation for LANG007.C1 have already been updated outside this plan. **Do not rewrite, reorganize, “finish,” or opportunistically improve documentation or specification files while executing this plan.** Read the live accepted specifications as semantic authority, but treat documentation edits as outside the patch.

The implementer:

- may adapt mechanical details to the live repository;
- must preserve the architecture and invariants in this plan;
- must not improvise a competing type-level calculus;
- must not encode packs or compile-time constants as fake proper types;
- must not merge ordered Tuple labeled structure into `RecordRow`;
- must not turn compiler-native type computation into VM/runtime dispatch;
- must not broaden this patch into general const generics, generic associated types, open type families, or variadic call inference;
- must preserve the existing query-local/canonical-state separation;
- must follow the testing budget rather than running broad suites reflexively;
- must classify and defer unrelated failures;
- must STOP AND CONSULT when an escalation trigger fires.

This plan is a **code implementation artifact**. It is not evidence that any edit or test has already occurred.

### User-directed lifecycle exception

The repository's normal implementation workflow asks numbered plans to update `CHECKPOINT.md` and create walkthrough/handoff files. The user has explicitly instructed this planning pass to **skip documentation updates because they have already been conducted**.

Therefore, for this plan:

```text
DO NOT modify:
    docs/specs/**
    docs/implementation/**
    docs/decisions/**
    docs/workflow/**
```

unless the user later explicitly re-enables documentation work.

The later lifecycle request superseded this temporary documentation exception.
P1 completion records are now maintained in the checkpoint, walkthrough, and
handoff artifacts.

---

# Checkpoint / evidence map

This plan belongs to repository checkpoint `LANG007.C1`. It does not invent nested `C0/C1/...` plan-local checkpoints; meaningful integration evidence is scheduled at `G` gates.

| Owning checkpoint slice | Tasks | Semantic boundary | Required evidence | Deferred evidence |
|---|---:|---|---|---|
| `LANG007.C1` / syntax + kinded domains | T0–T2 | source can express the new closed type-level domains without collapsing them into proper types | parser regressions, kind/type-store unit/foundation tests, semantic compile check | intrinsic reduction, metadata |
| `LANG007.C1` / type-member identity | T3–T4 | compiler-native type members have canonical identity, typed arguments, domain-aware projections, and symbolic residuals | identity/canonicalization tests, `[]`/`At` identity, wrong-domain diagnostics | full Tuple/Record algebra |
| `LANG007.C1` / structural operations | T5 | `Positionals`, `Labeled`, `At`, and `Map` implement the accepted structural algebra, including open-row residual mapping | focused structural operation matrix and hostile cases | broad semantic package |
| `LANG007.C1` / inference + persistence | T6–T7 | new kinded terms participate in canonical substitution/inference and durable metadata without leaking query-local IDs | inference/materialization tests, metadata validation, source-target parity | workspace certification |
| `LANG007.C1` / source/runtime boundary | T8 | canonical Universe source anchors the members while all type computation remains semantic/compiler-side | source integration tests, negative runtime search, focused crate checks | workspace test/clippy |

---

## 1. Goal

Implement the LANG007.C1.P1 semantic substrate and compiler-intrinsic structural type operations so Phalcom can soundly represent and reduce `TypePack`, `LabeledTypePack`, parameterized type members, compile-time integer type-member arguments, `Tuple` projections/indexing, and shape-preserving `Map<F>` for Tuple/Record-related domains without introducing runtime type dispatch or a general type-family language.

---

## 2. Checkpoint acceptance objective

This plan is **one code implementation slice of LANG007.C1**, not release-wide LANG007 closure.

The plan is accepted when the live compiler can form, preserve, normalize, serialize, and type-check the following accepted semantics through one coherent type-level architecture:

```phalcom
(Int, String)::Positionals
(Int, String, name: Symbol)::Labeled

(Int, String)[1]
(Int, String)::At<1>
(Int, String)::At<-1>

(Int, String)::Map<Option>
(Int, name: String)::Map<Option>

#{name: String, age: Int}::Map<Option>

P::Map<Option>              // P: TypePack
L::Map<Option>              // L: LabeledTypePack

R::Map<Option>              // R: RecordRow, if the accepted C1 spec retains this surface
```

The code must establish all of the following.

1. `TypePack` and `LabeledTypePack` are genuine kinds/type-level domains.
2. A `TypeId` remains the canonical arena handle for **kinded type-level forms**, not merely proper runtime types; `ProperTypeId` remains the guard for kind `Type`.
3. `RecordRow` remains its existing distinct row domain and is not re-encoded as a proper type or pack.
4. Tuple labeled-lane order remains semantic and is never normalized using record-row sorting.
5. Parameterized type-member arguments can be kinded forms (`F: Type -> Type`) or typed compile-time values (`I: Int`) without globally changing ordinary nominal/trait generic application in this patch.
6. `T[I]` and `T::At<I>` resolve to one canonical type-member identity.
7. `Option` in `Map<Option>` is checked as a type form of kind `Type -> Type`.
8. Type-member projections can return `Type`, `TypePack`, `LabeledTypePack`, and—where accepted—`RecordRow`.
9. A well-formed but abstract projection can remain stable and symbolic.
10. A concretely invalid projection is not rewritten to `Dynamic`, `Never`, or an unrelated unknown type.
11. Tuple indexing supports negative `Int` indexes and indexes the complete linear Tuple component order: positional lane first, labeled lane second.
12. Tuple mapping preserves arity, lane boundary, ordered labels, and component order.
13. Record mapping preserves structural keys and open-row semantics.
14. Mapping an open row maps the symbolic tail; the tail is never silently left unmapped.
15. Pack mapping preserves pack order; labeled-pack mapping also preserves label sequence.
16. Concrete `At` remains O(1); concrete mapping remains O(n).
17. Compiler-native type operations execute in semantic/type-normalization code, never through runtime VM sends or bytecodes.
18. Stable metadata can represent the new kinded terms and projections without raw `TypeId`, `KindId`, inference IDs, or query-local row variables.
19. Existing trait associated-type semantics remain intact.
20. Existing ordinary type application, type lambdas, record-row inference, generic call inference, and runtime Tuple/Record behavior remain compatible.

### Explicit scope boundary

This patch does **not** need to make these features generally available everywhere:

```text
class Vector<T, N: Int>
trait X<N: Int>
generic associated types
user-defined recursive type functions
arbitrary compile-time expression evaluation
variadic call-argument inference
pack split/backtracking inference
runtime TypePack objects
union-distributive type members
Symbol/Range type-level Tuple indexing
```

Those are later LANG007 work unless the already-ratified live C1 specifications explicitly require one of them for P1. If the live specifications do require one, that is a VERIFY-FIRST scope conflict and must be handled under the STOP/CONSULT rules rather than silently expanding the patch.

---

## 3. Repository grounding

Prepared against the remote repository state:

```text
repository:          aureat/phalcom-lang
remote branch:       main
remote revision:     5b63ade8073d13d6b528af4774181665c30cbd47
revision subject:    fix(core): restore standalone semantic universe
inspection date:     2026-09-21
local working tree:  NOT VISIBLE through the repository connector
```

The implementation agent must record its actual local `HEAD` and preserve unrelated working-tree changes before editing.

### Verified current repository facts

The following facts were verified at the planning revision.

#### Kinds and canonical type forms

`phalcom-semantic/src/types/kind.rs` currently has:

```rust
pub enum KindData {
    Type,
    RecordRow,
    Arrow {
        parameters: Box<[KindId]>,
        result: KindId,
    },
}
```

`phalcom-semantic/src/types/id.rs::KindId` currently reserves:

```rust
KindId::TYPE       == KindId(0)
KindId::RECORD_ROW == KindId(1)
```

`ProperTypeId(TypeId)` already distinguishes a kind-`Type` form from the broader `TypeId` arena.

`TypeData` already uses `TypeId` for both proper types and higher-kinded forms such as `Lambda`; `TypeId` is therefore already a **kinded-form identity**, not semantically synonymous with “runtime value type.”

#### Tuple and Record

`TypeData` contains:

```rust
Tuple(Box<[TupleTypeElement]>)
Record(RecordRowId)
```

`TupleTypeElement` preserves ordered labels.

`TypeStore::tuple([])` canonicalizes to `Unit`.

`RecordRowData` sorts fields by name and rejects duplicates:

```rust
pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>, // sorted by name, unique
    pub tail: RecordRowTail,
}
```

This is correct for Record identity and unsuitable for Tuple labeled-lane identity.

The accepted TDR-0081 states that `Unit`, `()`, and `#{}` share the sole canonical zero-product semantics. The current semantic store still permits a distinct `TypeData::Record(empty_row)` representation, so the live accepted LANG007 spec must be checked before changing this implementation seam.

#### Generic domains

`GenericInstantiation` is already domain-aware:

```rust
pub struct GenericInstantiation {
    type_bindings: HashMap<TypeParameterId, TypeId>,
    row_bindings: HashMap<TypeParameterId, RecordRowId>,
}
```

`TypeLevelBinding` already distinguishes:

```rust
TypeForm(TypeId)
RecordRow(TypeParameterId)
```

`GenericApplicationSession` already coordinates ordinary kinded inference with a separate row solver:

```rust
pub enum GenericInferenceBinding {
    Type(InferVarId),
    RecordRow(RecordRowVarId),
}
```

Every non-row kind currently receives an `InferVarId` carrying its kind. This is an important reuse seam for `TypePack` and `LabeledTypePack`; do not create a second pack solver merely to represent equality of pack-valued kinded forms.

#### Projection and metadata

Current associated projection identity is:

```rust
AssociatedTypeProjection {
    subject: TypeId,
    trait_ref: TraitRef,
    requirement: AssociatedTypeRequirementId,
}
```

and `TypeStore::associated_projection` always interns it at `KindId::TYPE`.

That representation cannot directly carry `TypePack`, `LabeledTypePack`, or `RecordRow` results.

Stable metadata currently has:

```rust
KindNode::{Type, RecordRow, Arrow}
TypeNode::AssociatedProjection { ... }
```

and must be extended if new projection forms are durable.

#### Parser

`KindSyntax` currently understands only:

```text
Type
RecordRow
Arrow
Grouped
```

`GenericParameterSyntax.kind` is `Option<KindSyntax>`.

`parse_generic_parameters` therefore treats every `:` binder RHS as a kind.

`TypeAnnotationExpr::Application` stores only type-form arguments.

`parse_postfix_type` currently parses `T::Name` through the enum/associated lookup seam; parameterized non-`Self` forms such as `T::Name<X>` are represented as `ExactEnumCase` syntax and are rejected by ordinary type resolution unless they are valid enum-case forms.

There is no neutral parameterized type-member AST and no type-level bracket-subscript AST.

#### Trait associated types

Existing trait associated types are trait-owned proper-Type requirements.

Their identity and conformance binding machinery are already established and must remain semantically unchanged by P1.

#### Runtime Tuple/Record

`phalcom-core/core/universe/src/collections/tuple.ph` confirms:

- positional lane precedes labeled lane;
- label order participates in Tuple equality;
- runtime integer indexing permits negative indexes;
- integer order spans the whole Tuple component sequence;
- Symbol and Range indexing exist at runtime but are outside initial type-level `At`.

`record.ph` confirms Record key identity is order-independent.

### Drift instruction

> Re-read every primary path named in this plan before editing. Adapt mechanical drift locally. Treat architectural drift as an escalation condition.

---

## 4. Required reads before implementation

Documentation is **read-only authority** for this plan.

### 4.1 Workflow and accepted C1 contract — read only

Read:

```text
AGENTS.md
docs/workflow/shared-consultation-escalation-protocol.md

the live LANG007.C1 checkpoint/specification artifacts already created by the user
the accepted type-level-domains specification
the accepted type-members specification
the accepted structural-products specification

docs/specs/README.md
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/source-declarations.md
docs/specs/objects/impl.md
docs/specs/objects/traits.md
docs/specs/universe.md
docs/specs/vm/products.md

docs/decisions/accepted/0081-tuple-and-record-are-transparent-structural-value-products.md
docs/decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md
```

If the live LANG007 specifications are not present in the checkout even though this plan assumes they have already been completed, STOP AND CONSULT rather than reconstructing them in code.

### 4.2 AST/parser

Read:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs

phalcom-ast/tests/parser.rs
phalcom-ast/tests/trait_syntax.rs
```

### 4.3 Semantic type system

Read:

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/mod.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/row_solver.rs
phalcom-semantic/src/types/relation.rs
```

### 4.4 Semantic publication/inference

Read:

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/declaration_signature.rs

phalcom-semantic/src/declarations.rs
phalcom-semantic/src/traits.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
```

### 4.5 Stable metadata/source products

Read:

```text
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/validate.rs

phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/mod.rs
```

### 4.6 Universe/core integration

Read:

```text
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/record.ph
phalcom-core/src/native/source.rs
phalcom-core/src/modules/compile.rs
```

Avoid unrelated compiler/VM exploration unless a planned semantic product is unexpectedly consumed there.

---

## 5. Normative authority

### 5.1 Accepted language behavior

The live, already-updated LANG007 C1 specifications are the primary language authority for this implementation.

This plan depends on the following ratified semantic distinctions:

```text
Type
RecordRow
TypePack
LabeledTypePack
(Type -> Type)
typed compile-time value arguments
```

and the structural laws for:

```text
Positionals
Labeled
At
Map
```

The implementer must not change those semantics to fit current Rust APIs.

### 5.2 Existing accepted decisions that remain controlling

Unless the live LANG007 specifications explicitly supersede them:

- TDR-0081 controls transparent Tuple/Record product identity, Tuple label ordering, Record key-order independence, and canonical zero-product semantics.
- TDR-0082 controls the distinction between `native` implementation origin and `@intrinsic` metadata.

### 5.3 Current implementation

Current Rust types and APIs are implementation reality, not language authority.

The plan intentionally changes several current assumptions:

```text
"the only non-Type kind is RecordRow"
"every type-member argument is a TypeId"
"every projection result is a proper Type"
"every :: projection is trait-associated or enum-case lookup"
```

### 5.4 Future/non-normative proposals

Do not use future LANG007 aspirations to widen P1.

In particular, this plan does not authorize:

```text
general const generics
GATs
user type families
pack arithmetic
arbitrary compile-time evaluation
runtime type-member dispatch
```

---

## 6. Takeover state

### 6.1 Stable interface map

| Concept | Current symbol/path | Owner | Invariant to preserve |
|---|---|---|---|
| canonical kind identity | `KindId`, `KindData` | `phalcom-semantic::types` | kind equality is semantic within one store |
| proper-type proof | `ProperTypeId` / `TypeStore::proper_type` | type store | only kind `Type` is a proper runtime value type |
| canonical kinded-form arena | `TypeId` / `TypeStore` | type store | query-local inference vars never enter it |
| Tuple structure | `TypeData::Tuple`, `TupleTypeElement` | type store | ordered positional + ordered labeled lanes |
| Record structure | `TypeData::Record(RecordRowId)` | type store + row arena | key identity order-independent |
| stable row domain | `RecordRowId`, `RecordRowData`, `RecordRowTail` | row arena | row values are not ordinary TypeIds |
| ordinary higher-kinded application | `TypeStore::apply_type_form` | type store | application is kind-checked |
| generic parameter kind | `TypeParameterData.kind` | type store | kind is declaration-owned |
| generic signature | `GenericSignature` | semantic declarations/callables | constraints belong to signature |
| type/row generic substitution | `GenericInstantiation` | semantic types | domains remain distinct |
| lexical type-level binding | `TypeLevelBinding` | annotation resolver | rows cannot masquerade as proper types |
| query-local type inference | `InferenceSession`, `InferVarId` | checker | inference IDs never become `TypeId` |
| query-local row inference | `RecordRowSolver`, `RecordRowVarId` | checker/row solver | row metavariables never become stable metadata |
| combined generic inference | `GenericApplicationSession` | checker | coordinates domains without merging them |
| trait associated projection | `AssociatedTypeProjection` + projection normalizer | semantic types | declaration/evidence identity, not name identity |
| stable kind metadata | `KindNode` | `phalcom-type-meta` | no raw store IDs |
| stable type metadata | `TypeNode` | `phalcom-type-meta` | durable store-independent graph |
| source type lowering | `types/annotation.rs` | semantic layer | parser syntax is not semantic authority |
| runtime Tuple behavior | Universe `tuple.ph` | runtime behavior | no static type inference authority |
| runtime Record behavior | Universe `record.ph` | runtime behavior | no static type inference authority |

### 6.2 Architecture already worth reusing

The repository already has the right major pattern:

```text
stable canonical term
    !=
query-local inference variable
```

and:

```text
Type domain solver
    coordinated with
RecordRow solver
```

LANG007 should extend this architecture.

Do **not** replace it with a universal untyped “generic value” map.

---

## 7. Architecture

### 7.1 Direction of authority

```text
accepted LANG007 language semantics
        ↓
AST preserves source shape without deciding meaning
        ↓
semantic domain resolution
        ↓
canonical kinded terms / record-row terms / type-member projections
        ↓
pure intrinsic normalization
        ↓
ordinary type relation / inference / metadata publication
        ↓
compiler/runtime consume already-resolved types

VM/runtime:
    no semantic projection search
    no type-member bytecode
    no type-family execution
```

### 7.2 Canonical kinded-form carrier

**FIXED architecture:**

Use the existing `TypeId` arena as the canonical carrier for every type-level form that has a `KindId`, including `TypePack` and `LabeledTypePack`.

This is not a category collapse.

```text
TypeId
    = canonical ID for a kinded type-level form

ProperTypeId
    = proof that TypeId has kind Type
```

Therefore:

```text
Int
    TypeId(kind = Type)

Option
    TypeId(kind = Type -> Type)

[Int, String] as a TypePack term
    TypeId(kind = TypePack)

[(name, String)] as a LabeledTypePack term
    TypeId(kind = LabeledTypePack)
```

`RecordRow` remains the existing separate `RecordRowId` domain because its solver, open-tail semantics, and canonical row machinery are already intentionally separate.

This is the most important implementation efficiency decision in P1: **do not introduce separate canonical TypePack/LabeledTypePack arenas merely because they are not proper types.**

### 7.3 New kinds

Add semantic kinds:

```text
TypePack
LabeledTypePack
```

with stable `KindId` constants allocated immediately after the existing fixed kinds.

STRUCTURAL target:

```rust
pub enum KindData {
    Type,
    RecordRow,
    TypePack,
    LabeledTypePack,
    Arrow {
        parameters: Box<[KindId]>,
        result: KindId,
    },
}
```

Keep `RecordRow` distinct.

### 7.4 Canonical pack forms

Add canonical `TypeData` forms.

STRUCTURAL target:

```rust
pub struct LabeledTypePackElement {
    pub label: Box<str>,
    pub ty: ProperTypeId,
}

pub enum TypeData {
    ...
    TypePack(Box<[ProperTypeId]>),
    LabeledTypePack(Box<[LabeledTypePackElement]>),
    ...
}
```

Formation must be checked:

```text
TypePack:
    every element kind == Type

LabeledTypePack:
    every payload kind == Type
    labels unique
    input order preserved
```

Do not sort a labeled pack.

### 7.5 Symbolic pack terms

An abstract pack parameter uses existing kinded parameter machinery:

```text
P : TypePack
    -> TypeData::Parameter(P) with kind TypePack

L : LabeledTypePack
    -> TypeData::Parameter(L) with kind LabeledTypePack
```

This provides stable symbolic pack terms without a second pack arena.

Later pack-splitting inference may introduce query-local pack variables if needed; P1 does not require them.

### 7.6 Structural Tuple from symbolic packs

P1 must support a stable structural formation term when an eventual Tuple shape depends on pack terms.

STRUCTURAL target:

```rust
TypeData::TupleFromPacks {
    positionals: TypeId, // kind TypePack
    labeled: TypeId,     // kind LabeledTypePack
}
```

Rules:

```text
TupleFromPacks(explicit P, explicit L)
    -> canonical TypeData::Tuple(...)
    -> or Unit for zero product

TupleFromPacks(symbolic P/L)
    -> remains canonical proper-Type term
```

There must never be two different concrete identities for the same fully materialized Tuple.

### 7.7 Domain-neutral type-member projection identity

Generalize projection identity instead of creating one reducer per operation.

STRUCTURAL target:

```rust
ProjectionId

enum ProjectionSubject {
    Kinded(TypeId),
    RecordRow(RecordRowId),
}

enum TypeMemberArgument {
    Kinded(TypeId),
    RecordRow(RecordRowId),
    Const(ConstTerm),
}

enum ProjectionResultDomain {
    Kind(KindId),
    RecordRow,
}

struct TypeLevelProjection {
    subject: ProjectionSubject,
    member: ProjectionMember,
    arguments: Box<[TypeMemberArgument]>,
    result_domain: ProjectionResultDomain,
}

enum ProjectionMember {
    TraitAssociated {
        trait_ref: TraitRef,
        requirement: AssociatedTypeRequirementId,
    },
    Intrinsic(TypeMemberId),
}
```

The exact Rust decomposition may vary mechanically, but the semantic facts above are FIXED.

For kind-valued results:

```text
TypeData::Projection(ProjectionId)
    interned with the declared result KindId
```

For row-valued symbolic results, reuse canonical row identity by allowing a row tail to reference a stable row-valued projection.

Recommended STRUCTURAL shape:

```rust
enum RecordRowTail {
    Closed,
    Parameter(TypeParameterId),
    Projection(ProjectionId), // projection result domain must be RecordRow
}
```

A standalone row-valued projection can be represented canonically as:

```text
RecordRowData {
    fields: [],
    tail: Projection(...)
}
```

This avoids turning `RecordRow` into a fake `TypeId` while still allowing:

```phalcom
R::Map<Option>
```

to remain a stable row term.

### 7.8 Preserve existing associated projection API as a compatibility constructor

Do not force every current call site to know the new generalized representation immediately.

Keep an API semantically equivalent to:

```rust
TypeStore::associated_projection(subject, trait_ref, requirement)
```

but have it construct the generalized projection internally.

Existing trait associated semantics remain:

```text
result domain = Type
normalization authority = conformance/generic evidence
```

### 7.9 Type-member identity

Type-member identity must not be a string.

STRUCTURAL target:

```rust
enum TypeMemberDomain {
    Tuple,
    Record,
    TypePack,
    LabeledTypePack,
    RecordRow,
}

enum TypeMemberSelector {
    Named(Box<str>),
    Subscript,
}

struct TypeMemberId {
    domain: TypeMemberDomain,
    selector: TypeMemberSelector,
}
```

Tuple/Record Universe declarations are **source anchors** for documentation/navigation/intrinsic binding; the semantic member identity remains domain-based so structural types do not become nominal classes.

`At` and bracket indexing must map to:

```text
TypeMemberSelector::Subscript
```

There must not be a separate canonical `At` identity.

### 7.10 Type-member signatures

Do not reuse `GenericSignature` blindly for member arguments because P1 needs both kinds and typed compile-time constants.

Use a dedicated type-member parameter domain.

STRUCTURAL target:

```rust
enum TypeMemberParameterDomain {
    Kind(KindId),
    Const(ProperTypeId),
}

struct TypeMemberParameter {
    name: Box<str>,
    domain: TypeMemberParameterDomain,
}

struct TypeMemberSignature {
    id: TypeMemberId,
    parameters: Box<[TypeMemberParameter]>,
    result: ProjectionResultDomain,
    intrinsic: IntrinsicTypeOperation,
    source_anchor: Option<...>,
}
```

Required signatures:

```text
Tuple::Positionals
    () -> TypePack

Tuple::Labeled
    () -> LabeledTypePack

Tuple::Subscript / presentation alias At<I>
    (Const(Int)) -> Type

Tuple::Map<F>
    (Kind(Type -> Type)) -> Type

Record::Map<F>
    (Kind(Type -> Type)) -> Type

TypePack::Map<F>
    (Kind(Type -> Type)) -> TypePack

LabeledTypePack::Map<F>
    (Kind(Type -> Type)) -> LabeledTypePack

RecordRow::Map<F>
    (Kind(Type -> Type)) -> RecordRow
```

### 7.11 Minimal compile-time constant calculus

P1 needs only:

```text
signed Int literal
type-member const parameter reference
```

STRUCTURAL target:

```rust
enum ConstTerm {
    Int(i64),
    Parameter(TypeMemberConstParameterId),
}
```

Do not introduce:

```text
const arithmetic solver
runtime expression evaluation
general literal singleton types
const values on nominal generic applications
```

in this plan.

### 7.12 Source syntax architecture

Add a neutral type-member source model rather than teaching enum-case AST to carry unrelated semantics forever.

Recommended AST concept:

```rust
TypeMemberProjection {
    subject: Box<TypeAnnotation>,
    selector: TypeMemberSelectorSyntax,
    arguments: Vec<TypeMemberArgumentSyntax>,
    range: SourceRange,
}
```

and:

```rust
TypeMemberDeclaration {
    selector: TypeMemberSelectorSyntax,
    parameters: Vec<TypeMemberParameterSyntax>,
    result_domain: TypeMemberResultSyntax,
    implementation: ImplementationKind,
    attributes: Vec<Attribute>,
    ...
}
```

Required source forms are whatever the accepted live LANG007 spec defines, expected to be equivalent to:

```phalcom
native class Tuple is Collection {
    @intrinsic
    native type Positionals: TypePack

    @intrinsic
    native type Labeled: LabeledTypePack

    @intrinsic
    native type Map<F: Type -> Type>: Type

    @intrinsic
    native type [I: Int]: Type
}
```

If the accepted live spec uses `type At<I: Int>: Type` instead of subscript-shaped declaration syntax, preserve that spelling but still canonicalize `T[I]` and `T::At<I>` to one semantic member ID.

### 7.13 Expected-domain argument elaboration

Parser syntax must not decide whether `<...>` contains a type form or const term.

Semantic elaboration does:

```text
expected Kind(k)
    -> elaborate argument as kinded type-level form and check kind == k

expected Const(Int)
    -> elaborate signed integer literal / const member parameter as compile-time Int term
```

This is how:

```phalcom
Map<Option>
At<1>
```

share angle-bracket syntax without conflating `Option` and `1`.

### 7.14 Intrinsic normalizer

Intrinsic type members reduce through one semantic evaluator.

Conceptual flow:

```text
projection
    ↓
resolve canonical member signature
    ↓
validate subject domain
    ↓
validate/elaborate arguments
    ↓
if concrete enough:
    evaluate intrinsic
else:
    retain symbolic projection
```

Required intrinsic result classification:

```text
Reduced
Stuck
Invalid
Cancelled
BudgetExceeded
InternalFailure
```

The language-level distinction is:

```text
Stuck != Invalid
```

### 7.15 Structural Map evaluator

Required rules:

```text
MapPack([T0..Tn], F)
    = [F<T0> .. F<Tn>]

MapLabeled([(l0,T0)..], F)
    = [(l0,F<T0>)..]

MapTuple(Tuple(P,L), F)
    = Tuple(MapPack(P,F), MapLabeled(L,F))

MapRow({li:Ti | R}, F)
    = {li:F<Ti> | MapRow(R,F)}

MapRecord(Record(R), F)
    = Record(MapRow(R,F))
```

Every `F<T>` uses existing `TypeStore::apply_type_form`.

Do not implement a Map-specific generic application engine.

### 7.16 Tuple indexing evaluator

For total arity `N`:

```text
j = i      if i >= 0
j = N + i  if i < 0
```

valid iff:

```text
0 <= j < N
```

The component sequence is:

```text
positionals ++ labeled payloads
```

Concrete out-of-range is `Invalid`, not `Stuck`.

### 7.17 Union rule

P1 does not implicitly distribute intrinsic structural members over unions.

A union subject remains unsupported/stuck/invalid according to the accepted live spec; do not synthesize branchwise union results unless the spec explicitly says so.

### 7.18 Runtime boundary

No new:

```text
bytecode
VM instruction
runtime TypePack class
runtime type-member message
native callable descriptor for type computation
```

should be necessary.

Universe source declarations anchor the semantic member surface, but `phalcom-core/src/native/source.rs` must not register them as runtime callable members.

---

## 8. Ownership boundaries

### Owns

This plan owns code changes for:

- AST syntax and parser representation required by accepted LANG007 source forms;
- new kind identities;
- canonical pack forms;
- symbolic Tuple-from-pack formation;
- type-member semantic identity/signatures;
- typed compile-time member arguments;
- generalized projection representation;
- intrinsic normalization;
- Tuple/Record/pack/row `Map`;
- Tuple `Positionals`, `Labeled`, and `At`;
- symbolic row-tail mapping;
- substitution/inference/materialization integration;
- stable type metadata;
- source semantic targets required to identify type members;
- Universe source declarations and core parsing changes necessary to accept them;
- targeted regression tests.

### Does not own

This plan does not own:

- changes to accepted specs or decisions;
- general nominal const generics;
- generic associated types;
- user-defined type families;
- arbitrary compile-time evaluation;
- pack split/shape inference;
- variadic method/callable arguments;
- runtime reflection API for packs;
- runtime VM dispatch for type members;
- LSP-specific semantic reimplementation;
- union distribution;
- Symbol/Range type-level Tuple indexing;
- broad unrelated type-system cleanup.

### Source-of-truth table

| Fact | Canonical owner | Consumers | Forbidden duplicate |
|---|---|---|---|
| kind identity | `TypeStore` / `KindData` | annotation, inference, metadata | parser string tests |
| pack canonical identity | `TypeStore` | normalizer, inference, metadata | tuple-as-pack encoding |
| row canonical identity | row arena | Record types, row solver, intrinsic row Map | pack representation |
| type-member identity | semantic type-member registry | resolver, normalizer, source index | name-only lookup |
| member parameter/result domains | semantic type-member signature | argument elaboration, diagnostics | parser heuristics |
| compile-time const term | semantic type-member argument | projection identity, At evaluator | runtime `Value` |
| projection identity | projection arena/interner | normalizer, metadata | ad hoc per-intrinsic nodes |
| trait projection proof | existing conformance/evidence machinery | generalized projection normalizer | intrinsic registry |
| Tuple lane shape | canonical Tuple type | Positionals/Labeled/At/Map | RecordRow |
| Record key semantics | RecordRow | Record Map | Tuple label sequence |
| intrinsic type operation | semantic normalizer | type formation | VM/native callable |
| durable identity | `phalcom-type-meta` stable graph | metadata/reflection tooling | raw `TypeId` |
| source member anchor | canonical Universe source + semantic source index | navigation/introspection | runtime native descriptor |

---

## 9. Global invariants

### Domain and identity

`INV-01 — TypeId is a kinded-form ID, not synonymous with proper Type.`

`INV-02 — ProperTypeId is the only wrapper that certifies kind Type.`

`INV-03 — TypePack and LabeledTypePack are distinct kinds and remain distinct from Tuple proper types.`

`INV-04 — RecordRow remains a separate canonical row domain and never becomes a fake proper Type.`

`INV-05 — LabeledTypePack preserves source/semantic order; RecordRow canonicalization remains order-independent.`

`INV-06 — Tuple labels are unique and Tuple lane order is positional prefix followed by ordered labeled suffix.`

### Members/projections

`INV-07 — Type-member identity is domain + canonical selector, never name text alone.`

`INV-08 — T[I] and T::At<I> produce the same canonical member identity.`

`INV-09 — Projection identity includes subject, member identity, canonical arguments, and result domain.`

`INV-10 — A well-formed unresolved projection remains symbolic; it is not approximated as Dynamic/Never/Unknown.`

`INV-11 — Invalid and Stuck are distinct normalization states.`

`INV-12 — Existing trait associated-type identity and evidence semantics remain unchanged.`

### Mapping/indexing

`INV-13 — Map<F> accepts only a form of kind Type -> Type.`

`INV-14 — Tuple Map preserves arity, positional/labeled boundary, label sequence, and component order.`

`INV-15 — LabeledTypePack Map preserves every label and label position.`

`INV-16 — Record Map preserves key identity and maps symbolic open tails.`

`INV-17 — Concrete Tuple At uses positional + labeled linear order and accepts negative Int indexes.`

`INV-18 — Concrete out-of-bounds At is a static invalid projection.`

`INV-19 — For valid I, (T::Map<F>)::At<I> normalizes equivalently to F<T::At<I>>.`

### Canonicalization/lifetime

`INV-20 — No query-local InferVarId or RecordRowVarId enters TypeStore canonical terms, metadata, or snapshots.`

`INV-21 — Fully concrete TupleFromPacks normalizes to the existing canonical Tuple/Unit identity.`

`INV-22 — No second nominal Tuple<P...> identity is introduced for structural tuples.`

`INV-23 — RecordRow projection tails remain stable semantic terms; row solver metavariables remain query-local.`

### Runtime boundary

`INV-24 — Native type-member normalization performs zero VM dispatch.`

`INV-25 — No new runtime bytecode is introduced for Positionals/Labeled/At/Map.`

`INV-26 — @intrinsic identifies builtin semantic association; native denotes implementation origin. Neither is inferred from spelling.`

### Compatibility

`INV-27 — Ordinary TypeData::Applied and TraitRef remain TypeId-argument APIs in P1 unless live accepted specs explicitly require language-wide heterogeneous generics.`

`INV-28 — Existing generic callable inference remains one coordinated inference architecture; P1 does not create a second call solver.`

`INV-29 — Existing type lambdas and higher-kinded application remain the mechanism used by Map to form F<T>.`

`INV-30 — No structural Tuple/Record inherent impl surface is introduced merely to host compiler-native type members.`

---

## 10. Non-goals

Do not expand P1 into:

```text
general class/data/enum const generics
trait const generics
generic associated types
higher-kinded associated requirements
open user-defined type families
arbitrary compile-time code execution
const arithmetic
pack length arithmetic
pack concatenation algebra beyond what TupleFromPacks concretization requires
pack split/backtracking solver
multiple unknown-length pack solving
variadic method arguments
variadic callable signature lowering
runtime TypePack / LabeledTypePack classes
runtime RecordRow values
reflection API design
VM bytecodes for type members
LSP-local type-member resolution
union distribution
Symbol-key Tuple type indexing
Range Tuple type slicing
Record value-order changes
Tuple runtime behavior redesign
```

---

## 11. Expected impact map

### Expected AST/parser source areas

- `phalcom-ast/src/ast.rs` — kinds, neutral type-member syntax, type-member declaration AST.
- `phalcom-ast/src/parser.rs` — member declarations, type-member projections, type-level subscript, expected-domain-preserving argument syntax.
- `phalcom-ast/tests/parser.rs` and/or a focused new type-member parser test file — syntax evidence.

### Expected semantic type source areas

- `phalcom-semantic/src/types/id.rs` — fixed kind IDs and projection ID if store-owned here.
- `phalcom-semantic/src/types/kind.rs` — new kinds.
- `phalcom-semantic/src/types/store.rs` — pack forms, projections, checked formation, TupleFromPacks.
- `phalcom-semantic/src/types/mod.rs` — exports.
- **new recommended:** `phalcom-semantic/src/types/type_member.rs` — type-member IDs/signatures/domains/intrinsic operation identity.
- `phalcom-semantic/src/types/projection.rs` — generalized normalization driver.
- `phalcom-semantic/src/types/annotation.rs` — type-level member resolution/argument elaboration.
- `phalcom-semantic/src/types/row.rs` — stable row projection tail.
- `phalcom-semantic/src/types/type_lambda.rs` — new kind/form exhaustive handling.
- `phalcom-semantic/src/types/instantiation.rs` — substitution/materialization for pack/projection forms.
- `phalcom-semantic/src/types/environment.rs` — only if new stable bindings require plumbing.
- `phalcom-semantic/src/types/substitution.rs` — exhaustive substitution support.
- `phalcom-semantic/src/types/relation.rs` — only enough relation handling to preserve canonical equality/subtyping behavior for new kinded forms; do not invent subtyping between packs.

### Expected semantic declaration/inference source areas

- `phalcom-semantic/src/checker/inference.rs` — lift/zonk new canonical forms.
- `phalcom-semantic/src/checker/row_inference.rs` — stable projection tail handling and generic session preservation.
- `phalcom-semantic/src/session.rs` and declaration-surface construction — publish type-member source anchors.
- `phalcom-semantic/src/db/fingerprint.rs` — AST/member semantic fingerprinting.
- `phalcom-semantic/src/source_index/builder.rs` — member declaration/projection targets.

### Expected stable metadata source areas

- `phalcom-type-meta/src/kind.rs`
- `phalcom-type-meta/src/type_node.rs`
- `phalcom-type-meta/src/generic.rs` if stable member arguments/IDs live there
- `phalcom-type-meta/src/validate.rs`
- `phalcom-semantic/src/metadata/export.rs`

### Expected Universe/core source areas

- `phalcom-core/core/universe/src/collections/tuple.ph`
- `phalcom-core/core/universe/src/collections/record.ph`
- `phalcom-core/src/native/source.rs` — ensure type members are not runtime callables; only if exhaustive AST handling requires it.

### Expected tests

Primary:

```text
phalcom-ast/tests/parser.rs
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/foundations/record_row_inference.rs
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/capabilities/traits.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
```

Recommended new focused semantic module if current test organization permits:

```text
phalcom-semantic/tests/semantic/foundations/type_members.rs
```

### Unexpected-touch rule

Touching mechanically adjacent exhaustive matches and test helpers is allowed.

STOP AND CONSULT before:

- changing runtime `Value` representation;
- changing bytecode;
- changing VM dispatch;
- changing trait conformance proof representation;
- changing module/import architecture;
- introducing a second semantic database/table that duplicates existing declaration authority;
- generalizing all nominal/trait generic arguments to heterogeneous values merely to make type members work.

---

## 12. Implementer decision authority

### 12.1 FIXED

The implementer must not change:

1. `TypePack` / `LabeledTypePack` as distinct kinds.
2. `RecordRow` as a distinct row domain.
3. `TypeId` as the canonical carrier for kinded non-row forms.
4. `ProperTypeId` as the kind-`Type` proof boundary.
5. domain-neutral projection identity.
6. no name-only type-member identity.
7. one `Subscript` identity for both `[]` and `At`.
8. type-member-specific heterogeneous argument elaboration rather than P1-wide const generics.
9. compiler/semantic normalization rather than VM dispatch.
10. Map's use of ordinary `apply_type_form` for `F<T>`.
11. ordered Tuple/labeled-pack semantics versus unordered RecordRow semantics.
12. stable symbolic open-row Map; unknown tails cannot remain unmapped.
13. no union distribution in P1.
14. query-local inference IDs never becoming canonical.
15. no documentation changes.

### 12.2 MECHANICALLY FLEXIBLE

Luna may adapt:

- exact new Rust module names;
- private helper names;
- whether `ProjectionId` lives in `id.rs` or projection module;
- whether intrinsic descriptors use `BTreeMap`, `HashMap`, or a fixed match table;
- private parser helper decomposition;
- exact diagnostic strings while preserving diagnostic categories;
- whether pack checked constructors accept iterators, slices, or boxed arrays;
- whether compatibility wrappers are deprecated immediately or retained privately;
- test file placement within the current semantic test organization;
- mechanical exhaustive-match changes.

### 12.3 VERIFY-FIRST

| Assumption | Where to verify | If false |
|---|---|---|
| live LANG007 C1 specs exist and are authoritative | local `docs/specs/` + LANG007 records | STOP; do not recreate semantics from this plan |
| accepted source syntax is `native type ...` or equivalent | live type-members/structural-products spec | adapt parser mechanics; STOP if semantic category differs |
| `At` is presentation alias for subscript identity | live structural-products spec | STOP if two identities are explicitly ratified |
| TypePack/LabeledTypePack are kinds | live type-level-domains spec | STOP if changed |
| general nominal const generics are not P1 | live C1 scope | STOP before expanding generic application |
| TDR-0081 zero-product rule remains active | live specs + accepted decision | if superseded, follow new spec; record architecture consequence |
| `TypeData::Applied` / `TraitRef` remain type-only arguments in P1 | live generic spec | STOP if C1 explicitly generalized all generic args |
| source member anchors belong in Tuple/Record Universe source | live type-members/universe spec | adapt only if authoritative owner is different |
| no runtime type-member execution exists | current core/VM | STOP if a canonical existing runtime mechanism is now authoritative |
| no existing generalized type-member registry already landed locally | search `TypeMember`, intrinsic type operations | reuse it; do not create a duplicate |
| no existing projection arena already landed locally | search projection types/store fields | extend it mechanically |
| no existing pack term types already landed locally | search `TypePack`, `LabeledTypePack` | reuse authoritative existing forms |

---

## 13. Global STOP / CONSULT triggers

STOP editing and build a consultation packet if any of these occurs.

1. Live accepted LANG007 specs materially contradict this plan's core architecture.
2. `TypePack`/`LabeledTypePack` cannot be represented as kinded `TypeId` forms without breaking a confirmed invariant.
3. Supporting non-Type projections appears to require making `TypeId` mean a runtime proper type everywhere.
4. `RecordRow::Map` cannot be represented without changing row identity/solver architecture materially beyond the planned stable projection-tail extension.
5. A source/member identity requires a second canonical registry separate from the semantic declaration/type-member authority.
6. `At` and `[]` appear to require separate semantic IDs to satisfy existing accepted behavior.
7. Existing trait associated-type normalization would have to be semantically weakened or rewritten.
8. General nominal const generics become necessary for P1.
9. General GAT support becomes necessary for P1.
10. A VM bytecode/runtime message path appears necessary to implement type-member normalization.
11. The same nontrivial semantic failure persists after one serious correction.
12. Two plausible fixes have materially different canonical identity, row representation, or projection-lifetime consequences.
13. An invariant must be weakened to pass tests.
14. New pack terms appear in stable metadata only by serializing raw store/query IDs.
15. An open-row Map implementation is trending toward leaving the tail unchanged.
16. Empty Record semantics conflict between the live accepted LANG007 spec and TDR-0081/current implementation and cannot be resolved by authority ordering.
17. Implementation requires changing a materially unrelated subsystem not listed in the impact map.
18. Scope grows into broad generic-application migration.

A triggered consultation cannot be self-waived.

---

## 14. Debugging budget

### Mechanical failures

Allow up to three coherent correction cycles while evidence shows progress:

```text
inspect
identify concrete mechanical cause
make one coherent correction
rerun the smallest discriminating command
```

Examples:

- exhaustive match not updated;
- renamed AST field;
- import/re-export mismatch;
- test filter mismatch;
- borrow/type error caused by arena mutability.

### Semantic failures

Before changing code, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Allow one serious correction for the same underlying semantic failure.

If it persists: STOP AND CONSULT.

### Architectural failures

Zero speculative architecture-fix attempts.

Consult immediately.

Never rerun an unchanged failing test when nothing relevant to the failure changed.

---

## 15. Testing surface analysis

| Coverage ID | Dimension | Invariant(s) | Required case |
|---|---|---|---|
| `CV-01` | kind identity | INV-01..05 | `TypePack` and `LabeledTypePack` intern as distinct kinds |
| `CV-02` | proper-type boundary | INV-01..04 | pack form rejected where proper Type is required |
| `CV-03` | pack canonicalization | INV-03,20 | identical explicit packs intern identically |
| `CV-04` | label ordering | INV-05,06 | labeled packs with swapped labels remain distinct |
| `CV-05` | row distinction | INV-04,05 | RecordRow field reorder canonicalizes while labeled pack reorder does not |
| `CV-06` | syntax | INV-07..09 | `T::Map<Option>`, `T::At<1>`, and `T[1]` parse into neutral member syntax |
| `CV-07` | identity alias | INV-08 | `T[1]` and `T::At<1>` produce same projection/member identity |
| `CV-08` | constructor-kind checking | INV-13 | `Map<Option>` accepted, `Map<Int>` rejected |
| `CV-09` | const-domain checking | INV-17,18 | `At<1>` accepted, non-Int/non-const argument rejected |
| `CV-10` | negative index | INV-17 | `At<-1>` selects final component |
| `CV-11` | bounds | INV-18 | positive and negative out-of-range indexes are Invalid |
| `CV-12` | labeled index order | INV-17 | numeric index reaches labeled payload after positional lane |
| `CV-13` | Positionals | INV-03,06 | Tuple positional lane becomes ordered TypePack |
| `CV-14` | Labeled | INV-05,06 | Tuple labeled lane becomes ordered LabeledTypePack |
| `CV-15` | Tuple Map | INV-13,14 | positional+labeled mixed Tuple maps payloads only |
| `CV-16` | Map/At composition | INV-19 | mapped Tuple indexed result equals constructor-applied original result |
| `CV-17` | pack Map | INV-13,15 | TypePack maps order-preservingly |
| `CV-18` | labeled-pack Map | INV-13,15 | labels survive map exactly |
| `CV-19` | Record Map | INV-16 | closed Record maps every field and preserves key identity |
| `CV-20` | open-row Map | INV-16,23 | mapped open row tail is symbolic mapped tail, not original tail |
| `CV-21` | symbolic projection | INV-10,11 | abstract pack/row map remains stable symbolic term |
| `CV-22` | no union distribution | INV-10 | union subject does not silently branch-map |
| `CV-23` | inference lifetime | INV-20,28 | pack-kinded inference variable never appears in snapshot/metadata |
| `CV-24` | existing associated types | INV-12 | trait `Self::Item` and qualified projection tests stay green |
| `CV-25` | metadata | INV-20 | pack/projection graph round-trips/validates without raw local IDs |
| `CV-26` | source target | INV-07 | type-member occurrence targets canonical member declaration/domain |
| `CV-27` | zero product | INV-21 + live spec | empty Tuple/Record behavior matches current accepted authority |
| `CV-28` | runtime boundary | INV-24..26 | no bytecode/runtime type-member implementation is introduced |
| `CV-29` | TupleFromPacks | INV-21,22 | explicit packs collapse to existing canonical Tuple |
| `CV-30` | cold/incremental parity | INV-07,20 | type-member projection semantic identity survives unchanged edit/reanalysis if current query system exposes it |

Do not mechanically add one new test per row if an existing focused test can prove multiple obligations.

---

## 16. Verification execution budget

### Modes

#### BUILD MODE

Default during T1–T7.

Use parser/semantic unit/foundation tests and strategic `cargo check`.

#### STABILIZE MODE

Enter at G4/G5.

Run directly affected test modules and package checks.

#### CERTIFY MODE

Not part of this plan unless the user explicitly requests release/workspace certification.

### Verification ladder

```text
T0 — exact new regression / exact unit
T1 — directly affected parser/foundation test module
T2 — owning semantic integration module
T3 — semantic ↔ metadata / Universe integration
T4 — affected crates
T5 — workspace/release
```

### Mandatory during BUILD

Use only when relevant to the task:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser <focused-filter> -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_annotations -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_members -- --nocapture
```

If `type_members` is not a separate module, use the exact live module containing the new tests.

### Strategic compile checks

After AST fanout:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast
```

After semantic representation fanout:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

After metadata schema fanout:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

After Universe/core exhaustive match updates:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

### Mandatory during STABILIZE

Focused semantic suites:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_annotations -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::record_row_inference -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_application -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::traits -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::integration::metadata -- --nocapture
```

Plus the focused new type-member suite.

### Mandatory final hygiene

```sh
cargo +stable fmt --all -- --check
git diff --check
```

If the repository's pinned toolchain differs, use the live repository convention.

### Only if evidence demands

- full `phalcom-ast` tests;
- full `phalcom-semantic` test binary;
- `phalcom-core` focused Universe/native-source tests;
- incremental type-member tests;
- LSP adapter tests if a source-index change actually breaks the adapter.

### Explicitly deferred / DO NOT RUN during BUILD

Do not run:

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

Do not run:

- the full language corpus;
- unrelated VM/runtime suites;
- all LSP tests;
- broad performance suites.

Reason: P1 is a semantic/type-system patch. Broad workspace evidence adds little during construction and risks consuming time on unrelated baseline failures.

---

## 17. Baseline / unrelated failure policy

Classify failures:

```text
A — definitely caused by this patch
B — probably caused by this patch
C — unclear
D — clearly unrelated/baseline
```

- A/B: active responsibility.
- C: one bounded classification pass. If nonblocking and still unclear, report/defer.
- D: report and continue immediately.
- Do not weaken accepted semantic assertions to make a broad suite green.
- Do not repair unrelated TEST002/corpus failures as part of LANG007.
- Do not consume adviser compute on D unless it blocks focused acceptance.

---

# 18. Tasks

## T0 — Live takeover and semantic-contract verification

### Purpose

Establish the exact local repository state and confirm that the already-completed LANG007 documentation/specification work matches the semantic assumptions this code plan implements.

### Risk

- Semantic: HIGH
- Implementation fanout: none; read-only

### Preconditions

- user states LANG007 documentation updates are already complete;
- this plan's remote grounding is `5b63ade...`, but local tree may be newer.

### Consumes

- live checkout;
- accepted LANG007 C1 specs;
- this plan.

### Produces

A local implementation starting state:

```text
actual HEAD
working-tree status
authoritative LANG007 spec paths
confirmed source spellings
confirmed zero-product ruling
confirmed P1 scope exclusions
```

### Files and symbols

**Read only:**

```text
AGENTS.md
live LANG007.C1 record/spec files
docs/specs/type-level-domains*.md           # exact live name
docs/specs/type-members*.md                 # exact live name
docs/specs/structural-products*.md          # exact live name
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/source-declarations.md
docs/decisions/accepted/0081-*.md
```

Then verify primary source anchors named in §4.

### Required implementation shape

No production edit.

Run:

```sh
git status --short
git rev-parse HEAD
```

Search:

```sh
rg -n 'TypePack|LabeledTypePack|native type|type \[|::Map|::At' docs/specs docs/implementation 2>/dev/null
```

This search is evidence only; **do not edit the results**.

### Forbidden approaches

- do not create missing specs;
- do not normalize documentation layout;
- do not start coding if accepted semantics are missing/ambiguous;
- do not assume remote paths are identical to local.

### Tests to run now

None.

### Acceptance

T0 passes when:

- local HEAD is known;
- unrelated local changes are identified and protected;
- live accepted C1 semantics confirm the FIXED architecture or only require mechanical syntax adaptation;
- no material authority conflict exists.

### Local STOP / CONSULT triggers

- LANG007 specs are absent;
- zero-product semantics are materially unresolved;
- live C1 requires general nominal const generics or GATs in P1;
- live C1 rejects `TypeId` as the canonical kinded-form carrier.

---

## T1 — Add neutral AST and parser support for type-level domains and type members

### Purpose

Make the accepted source syntax representable without abusing enum-case AST or trait-associated-type AST.

### Risk

- Semantic: HIGH
- Implementation fanout: multi-file AST/parser, mechanically broad exhaustive matches

### Preconditions

- T0 PASS;
- exact live syntax confirmed.

### Consumes

- accepted source grammar;
- current `KindSyntax`, `GenericParameterSyntax`, `TypeAnnotationExpr`;
- current class/trait member parsers.

### Produces

AST capable of representing:

```text
TypePack kind
LabeledTypePack kind
domain-native type-member declarations
parameterized type-member arguments
type-level bracket subscript
named type-member projection
typed const-domain member parameter syntax
```

### Files and symbols

**Read:**

```text
phalcom-ast/src/ast.rs
    KindSyntax
    GenericParameterSyntax
    TypeAnnotationExpr
    AssociatedTypeDeclaration
    ClassMember

phalcom-ast/src/parser.rs
    GenericBinderContext
    parse_kind_expression
    parse_generic_parameters
    parse_postfix_type
    parse_associated_type_declaration
    class-member parsing branch
```

**Modify:**

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
```

**Potential mechanical callers:**

```text
phalcom-ast/src/*
phalcom-semantic/src/*
phalcom-core/src/native/source.rs
source visitors that exhaustively match ClassMember / TypeAnnotationExpr / KindSyntax
```

**Tests:**

```text
phalcom-ast/tests/parser.rs
or create phalcom-ast/tests/type_members.rs if test organization favors isolation
```

### Source of truth

Accepted source grammar.

AST nodes preserve authored structure; AST does not decide member applicability or kind validity.

### Required implementation shape

#### 1. Add kind syntax

STRUCTURAL:

```rust
KindSyntax::TypePack(SourceRange)
KindSyntax::LabeledTypePack(SourceRange)
```

Update `KindSyntax::range`.

`parse_kind_expression` must recognize the exact accepted spellings.

#### 2. Introduce a neutral type-member selector syntax

STRUCTURAL:

```rust
enum TypeMemberSelectorSyntax {
    Named {
        name: String,
        range: SourceRange,
    },
    Subscript {
        range: SourceRange,
    },
}
```

Exact shape may vary.

#### 3. Introduce expected-domain-preserving member arguments

STRUCTURAL:

```rust
enum TypeMemberArgumentSyntax {
    TypeForm(TypeAnnotation),
    IntLiteral {
        value: i64,
        range: SourceRange,
    },
    Name {
        name: String,
        range: SourceRange,
    },
}
```

Do not force the parser to decide whether `Name` is a type form or a const parameter. Semantic expected-domain elaboration decides.

If the existing lexer represents signed integer as `Minus` + integer literal, preserve source range and fold it into one const syntax only in the type-member argument parser.

#### 4. Add neutral projection syntax

STRUCTURAL:

```rust
TypeAnnotationExpr::TypeMemberProjection {
    subject: Box<TypeAnnotation>,
    selector: TypeMemberSelectorSyntax,
    arguments: Vec<TypeMemberArgumentSyntax>,
    range: SourceRange,
}
```

Parser behavior:

```phalcom
T::Map<Option>
T::Positionals
T::At<1>
T[1]
```

must use this neutral representation where the accepted grammar says these are type-member forms.

Do not preserve the current accidental rule “all non-Self `T::Name<...>` syntax is an ExactEnumCase” for these forms.

Keep exact enum-case syntax functioning in its real enum-case contexts.

#### 5. Add domain-native type-member declaration AST

STRUCTURAL:

```rust
struct TypeMemberDeclaration {
    selector: TypeMemberSelectorSyntax,
    parameters: Vec<TypeMemberParameterSyntax>,
    result: TypeMemberResultSyntax,
    implementation: ImplementationKind,
    attributes: Vec<Attribute>,
    range: SourceRange,
}
```

Add the appropriate `ClassMember` variant.

Initial legal declaration category is the accepted domain-native/native form. Do not generalize trait `AssociatedTypeDeclaration` into GAT syntax in T1.

#### 6. Parse member parameter domains without changing ordinary binder meaning

STRUCTURAL:

```rust
enum TypeMemberParameterDomainSyntax {
    Kind(KindSyntax),
    ConstType(TypeAnnotation),
}
```

Parse:

```phalcom
F: Type -> Type
```

as a kind domain, and:

```phalcom
I: Int
```

as a typed const domain.

This is dedicated to type-member parameters.

Do not change ordinary `<T: ...>` generic binder parsing to accept arbitrary proper types in T1.

#### 7. Preserve trait syntax

The following must remain unchanged:

```phalcom
trait Iterator {
    type Item
}
```

and existing qualified/contextual associated projection syntax.

### Forbidden approaches

- do not add `Int` to `KindSyntax`;
- do not parse `At<1>` as ordinary generic type application;
- do not use runtime expression AST for const terms;
- do not turn class type members into `MethodDef`;
- do not expand trait associated declarations to GATs;
- do not encode member identity by string in AST consumers.

### Test changes required

Cover:

- `CV-01`, `CV-06`, `CV-07`, `CV-08`, `CV-09`.

Parser tests should inspect AST shape, not only “parses successfully.”

Required cases:

```phalcom
type Alias<P: TypePack> = ...
type Alias<L: LabeledTypePack> = ...

(Int, String)::Map<Option>
(Int, String)::At<1>
(Int, String)[1]

native class Tuple {
    native type Map<F: Type -> Type>: Type
    native type [I: Int]: Type
}
```

Also hostile cases:

- malformed const argument;
- malformed member parameter domain;
- trait `type Item` still produces `AssociatedTypeDeclaration`.

### Tests to run now

After the AST compiles:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser <new-type-member-filter> -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast
```

### Tests explicitly deferred

- semantic tests;
- core/Universe tests;
- workspace tests.

### Acceptance

T1 passes when source syntax is represented neutrally and existing trait/enum syntax remains structurally distinguishable.

### Local STOP / CONSULT triggers

- exact enum-case and type-member syntax are grammatically ambiguous in a way the accepted spec does not resolve;
- supporting `T[I]` would require reusing runtime expression indexing AST;
- accepted source grammar requires arbitrary const expressions.

---

## T2 — Extend the canonical kinded term model with packs and symbolic Tuple formation

### Purpose

Add the canonical semantic domains that later member resolution/normalization consumes.

### Risk

- Semantic: HIGH
- Implementation fanout: cross-module type-system core

### Preconditions

- T1 AST exists;
- T0 confirms kind semantics.

### Consumes

- `KindData`;
- `TypeStore`;
- `ProperTypeId`;
- type parameter kinds;
- scoped type-lambda machinery.

### Produces

Canonical forms:

```text
TypePack
LabeledTypePack
TupleFromPacks
```

with checked construction and stable kind identity.

### Files and symbols

**Modify:**

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/mod.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/relation.rs         # exhaustive/canonical handling only
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/type_model.rs
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
```

### Source of truth

`TypeStore` is canonical owner of kinded forms.

### Edit operations

#### 1. Add fixed kind IDs

Add:

```rust
KindId::TYPE_PACK
KindId::LABELED_TYPE_PACK
```

at deterministic fixed slots after `TYPE` and `RECORD_ROW`.

Initialize `TypeStore::new` in the same order and assert fixed IDs.

Update:

```text
format_kind
apply_kind non-applicable cases
kind metadata/export later
all exhaustive KindData matches
```

#### 2. Add checked pack data

STRUCTURAL:

```rust
pub struct LabeledTypePackElement {
    pub label: Box<str>,
    pub ty: ProperTypeId,
}

TypeData::TypePack(Box<[ProperTypeId]>)
TypeData::LabeledTypePack(Box<[LabeledTypePackElement]>)
```

Add checked constructors such as:

```text
TypeStore::type_pack(...)
TypeStore::labeled_type_pack(...)
```

The exact API is mechanically flexible.

The constructors must:

- validate proper elements;
- preserve order;
- reject duplicate labels for labeled packs;
- hash-cons canonical terms.

#### 3. Add TupleFromPacks

STRUCTURAL:

```rust
TypeData::TupleFromPacks {
    positionals: TypeId,
    labeled: TypeId,
}
```

Formation checks:

```text
kind(positionals) == TypePack
kind(labeled) == LabeledTypePack
```

Normalization during formation:

- explicit pack + explicit labeled pack -> existing checked Tuple formation;
- both empty -> canonical zero product according to live accepted semantics;
- otherwise retain symbolic TupleFromPacks.

#### 4. Harden Tuple formation

Replace the current debug-only proper-type checks with checked formation at the canonical authority.

Required validation:

- every component proper Type;
- all unlabeled elements precede labeled elements;
- labels unique.

If existing parser/lowering already guarantees the lane rule, keep that fast path but canonical TypeStore construction must still not accept malformed internal tuples silently.

A private unchecked constructor may exist only for already-validated internal rebuilding and should be clearly named.

#### 5. Resolve new kind syntax

`resolve_kind_syntax` must map:

```text
TypePack        -> KindId::TYPE_PACK
LabeledTypePack -> KindId::LABELED_TYPE_PACK
```

Ordinary generic binders may therefore use:

```phalcom
<P: TypePack>
<L: LabeledTypePack>
```

No pack expansion syntax is added.

#### 6. Preserve TypeLevelBinding architecture

`type_level_binding_for_parameter` should continue to special-case only `RecordRow`.

For pack-kinded parameters:

```text
TypeData::Parameter(parameter)
    with kind TypePack/LabeledTypePack
```

is valid.

This is intentional.

#### 7. Update substitution/scoped/type-lambda exhaustive handling

Pack terms and TupleFromPacks must:

- survive substitution;
- preserve kind;
- remain canonical;
- never become proper types accidentally.

Do not add pack split logic.

### Forbidden approaches

- no separate canonical PackId arena;
- no Tuple-as-pack encoding;
- no RecordRow reuse;
- no `TypeData::Infer`;
- no automatic pack-to-Tuple coercion;
- no pack subtyping relation.

### Test changes required

Cover `CV-01` through `CV-05`, `CV-29`.

Hostile cases:

- TypePack passed where `Type` required;
- labeled pack order distinction;
- RecordRow reorder equality versus labeled pack reorder inequality;
- malformed Tuple lane/duplicate label rejected by checked formation.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_model -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_annotations -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Acceptance

T2 passes when new domains are canonical, kind-correct, substitutable, and no query-local IDs or row semantics have been merged into them.

### Local STOP / CONSULT triggers

- significant production code assumes all `TypeId != RecordRow` forms are proper runtime types and cannot be fixed by local kind checks;
- adding fixed kind IDs breaks a stable serialization contract that requires a different migration;
- checked Tuple formation reveals existing canonical callers intentionally constructing interleaved/duplicate labels.

---

## G1 — Canonical domain gate

### Purpose

Prove the new kind/domain model exists independently of type-member normalization.

Run the T1/T2 focused commands.

Expected:

```text
TypePack kind: PASS
LabeledTypePack kind: PASS
proper-type rejection: PASS
pack interning/order laws: PASS
Tuple checked formation: PASS
existing Type/RecordRow kind tests: PASS
```

Do not broaden after PASS.

On failure, classify before editing.

---

## T3 — Introduce canonical type-member identity, signatures, and Universe source anchors

### Purpose

Create one semantic authority for compiler-native type members and bind canonical Tuple/Record source declarations to that authority without making them runtime callables.

### Risk

- Semantic: HIGH
- Implementation fanout: semantic publication + Universe source

### Preconditions

- G1 PASS;
- live type-member source syntax confirmed.

### Consumes

- T1 type-member declaration AST;
- T2 kinds;
- canonical Universe declaration identity;
- existing `@intrinsic` and `native` metadata semantics.

### Produces

- canonical `TypeMemberId`;
- `TypeMemberSignature`;
- domain member registry/table;
- canonical source anchors for Tuple/Record intrinsic members;
- no runtime callable registration.

### Files and symbols

**Create recommended:**

```text
phalcom-semantic/src/types/type_member.rs
```

**Modify expected:**

```text
phalcom-semantic/src/types/mod.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/source_index/builder.rs

phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/record.ph
phalcom-core/src/native/source.rs              # exhaustive handling / explicit ignore only
```

Exact publication helper may live beside declaration/surface code if the live repository already has a more appropriate canonical table.

### Source of truth

The semantic type-member registry/table is the canonical operational authority.

Universe source is the canonical authored anchor/provenance.

Runtime native-member census is **not** type-member authority.

### Required implementation shape

#### 1. Define stable semantic member IDs

Implement the `TypeMemberDomain` / `TypeMemberSelector` architecture from §7.

Use a typed enum for builtin domains.

Do not use:

```rust
HashMap<String, ...>
```

as canonical identity.

A map keyed by typed ID is fine.

#### 2. Define signatures

Represent member parameter domains and result domains explicitly.

The registry must answer:

```text
lookup(subject domain, selector)
    -> TypeMemberSignature
```

and preserve source anchor/provenance separately.

#### 3. Bind source declarations

Update canonical Universe source with the accepted declarations for:

```text
Tuple:
    Positionals
    Labeled
    Map
    subscript / At

Record:
    Map
```

Use `native` / `@intrinsic` exactly as accepted by the live spec.

If `@intrinsic` carries a payload/key in the current language, use the accepted canonical key; do not infer intrinsic operation from member spelling alone.

If `@intrinsic` is marker-only today, bind using exact canonical owner + typed selector during semantic publication, and make that association explicit in one table.

#### 4. Keep runtime member census clean

`phalcom-core/src/native/source.rs` must either:

- explicitly ignore `ClassMember::TypeMember` for runtime native-callable registration; or
- route it into a separate semantic-source anchor product if such a product already exists.

It must not create `NativeMemberKey`, runtime selector, callable descriptor, or primitive for type members.

#### 5. Preserve structural applicability

A Tuple domain member applies to structural Tuple proper types according to semantic shape, not because the Tuple `ClassObject` owns ordinary inherent behavior.

Likewise Record.

Unit/zero product applicability follows the live accepted structural-products spec.

### Forbidden approaches

- no type-member registration in `SurfaceDispatchResolver`;
- no runtime method selector for Map/At type members;
- no string-only intrinsic registry;
- no user structural `impl`;
- no separate synthetic nominal `Tuple<...>` declaration.

### Test changes required

Cover:

- `CV-07`, `CV-26`, `CV-28`.

Add tests proving:

- source member declaration gets canonical semantic member ID;
- same name in an unrelated trait/class does not become the same member;
- runtime native source census does not treat type member as callable;
- Tuple and Record `Map` are distinct member identities despite same presentation name.

### Tests to run now

A focused semantic publication/source test only.

If no behavior is meaningful until T4:

> **Do not run broad semantic/runtime tests after this task. The intermediate state is intentionally incomplete.**

A strategic check is allowed:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Acceptance

T3 passes when type-member identity/signatures have one owner and Universe source anchors them without runtime-callable pollution.

### Local STOP / CONSULT triggers

- the only available canonical source publication path is ordinary dispatch surface;
- intrinsic metadata cannot identify the member without a second name database;
- structural applicability would require pretending Tuple/Record are nominal generic types.

---

## T4 — Generalize projection identity and elaborate member arguments

### Purpose

Connect source projections/subscripts to canonical typed projections that can return non-Type domains and remain symbolic.

### Risk

- Semantic: HIGH
- Implementation fanout: type store, annotation resolution, existing associated projection normalization, inference exhaustiveness

### Preconditions

- T3 member registry exists;
- pack kinds/forms exist.

### Consumes

- TypeMemberSignature;
- TypeMemberProjection AST;
- current associated projection infrastructure;
- existing generic evidence normalization.

### Produces

- projection arena/interner;
- typed `TypeMemberArgument`;
- minimal `ConstTerm`;
- unified projection identity;
- domain-aware source resolution;
- symbolic residual projections.

### Files and symbols

**Modify expected:**

```text
phalcom-semantic/src/types/id.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/mod.rs
phalcom-semantic/src/checker/inference.rs
```

### Source of truth

Projection arena + type-member signature.

### Required implementation shape

#### 1. Add projection ID/interner

Add `ProjectionId` and one projection arena/hash-cons table to `TypeStore`.

Canonical key includes:

```text
subject
member identity
arguments
result domain
```

Source ranges are not identity.

#### 2. Migrate associated projections through compatibility constructor

Replace or wrap `TypeData::AssociatedProjection`.

Preferred target:

```rust
TypeData::Projection(ProjectionId)
```

with projection member variant:

```text
TraitAssociated
Intrinsic
```

Keep:

```rust
TypeStore::associated_projection(...)
```

as the current call-site API where practical.

Existing trait projection normalizer code moves under the TraitAssociated dispatch branch with no semantic change.

#### 3. Add typed member arguments

Implement:

```text
Kinded(TypeId)
RecordRow(RecordRowId)
Const(ConstTerm)
```

as required.

Canonical `ConstTerm::Int(i64)` must hash/compare structurally.

No runtime `Value`.

#### 4. Add domain-aware type-level resolution

Introduce an internal resolution carrier if needed:

```rust
enum TypeLevelResolutionValue {
    Kinded(TypeId),
    RecordRow(RecordRowId),
}
```

or an equivalent architecture.

Existing `resolve_type_form` continues to require a kinded `TypeId`.

Member projection resolution may accept a row subject and return a row result.

A use site requiring a proper type still goes through `resolve_type_annotation` and requires final kind `Type`.

#### 5. Elaborate arguments against signature domains

For each member argument:

```text
Kind(k):
    resolve kinded form
    require kind == k

Const(Int):
    accept signed integer literal
    or bound const member parameter
```

Wrong kind/type is `Invalid` with a semantic diagnostic.

#### 6. Canonicalize At/subscript identity

Parser syntax:

```phalcom
T[I]
T::At<I>
```

must call the same semantic member lookup and projection constructor.

Add a direct identity regression comparing canonical projection data/ID, not just formatted type text.

#### 7. Row-valued projection representation

Extend `RecordRowTail` with stable projection tail if the live C1 spec requires row projections.

`record_row_checked` validates that a projection tail:

```text
result_domain == RecordRow
```

The row solver must be able to preserve an unresolved stable projection tail as an opaque stable tail; it must not allocate a `RecordRowVarId` for it merely because it is unresolved.

### Forbidden approaches

- no independent AST-specific evaluator for `[]`;
- no `At` name special case after canonical member lookup except presentation alias mapping;
- no projection result coerced to kind Type;
- no `Dynamic` fallback for unresolved member;
- no raw `ProjectionId` in durable metadata later;
- no second normalizer for intrinsic members.

### Test changes required

Cover:

- `CV-06` through `CV-09`;
- `CV-21`;
- `CV-24`.

Hostile tests:

- same member name under unrelated trait is not intrinsic member identity;
- `Map<Int>` wrong kind;
- `At<Option>` wrong const domain;
- unresolved pack parameter `P::Map<Option>` remains symbolic, not error;
- existing `Self::Item`/qualified associated projections still normalize identically.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_members -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::traits -- --nocapture
```

Then:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Acceptance

T4 passes when projection identity is canonical, non-Type results are representable, typed member arguments are validated, and existing trait projections remain sound.

### Local STOP / CONSULT triggers

- trait projection migration changes conformance semantics;
- row-valued projection requires query-local solver identity in canonical state;
- `At`/subscript identity cannot be unified without changing accepted grammar.

---

## G2 — Type-member identity/projection gate

### Purpose

Prove the architecture before implementing the concrete algebra.

Required evidence:

1. `T[I]` / `T::At<I>` identity test.
2. `Map<Option>` kinded argument acceptance.
3. `Map<Int>` rejection.
4. symbolic pack Map projection preservation.
5. existing associated-type suite remains green.

Expected:

```text
no Dynamic approximation
no runtime dispatch
no name-only identity
```

Do not broaden after PASS.

---

## T5 — Implement intrinsic structural normalization: Positionals, Labeled, At, Map

### Purpose

Implement the actual LANG007 structural type operations in one closed semantic evaluator.

### Risk

- Semantic: HIGH
- Implementation fanout: semantic normalizer + row canonicalization

### Preconditions

- G2 PASS;
- member signatures/identity fixed.

### Consumes

- canonical pack forms;
- projection normalizer;
- checked Tuple/Record formation;
- ordinary type-form application.

### Produces

Working intrinsic reductions and symbolic residuals.

### Files and symbols

**Primary:**

```text
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/row.rs
```

**Secondary if needed:**

```text
phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/annotation.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/type_members.rs
phalcom-semantic/tests/semantic/foundations/type_model.rs
```

### Source of truth

`IntrinsicTypeOperation` evaluator.

No compiler/core/runtime duplicate.

### Required implementation shape

#### 1. Implement one evaluator entry point

STRUCTURAL:

```rust
fn normalize_intrinsic_projection(
    store: &mut TypeStore,
    projection: ProjectionId,
    context: &mut ProjectionNormalizationContext<'_>,
) -> ProjectionNormalizationResult
```

or equivalent.

Dispatch by intrinsic operation enum, not member name.

#### 2. Tuple Positionals

For a concrete Tuple:

```text
collect positional prefix
validate each payload ProperTypeId
intern TypePack
```

For zero product, follow live spec.

For incompatible subject domain: Invalid.

For symbolic TupleFromPacks:

```text
Positionals(TupleFromPacks(P,L))
    -> P
```

without materializing shape.

#### 3. Tuple Labeled

Concrete:

```text
collect ordered labeled suffix
intern LabeledTypePack
```

Symbolic TupleFromPacks:

```text
Labeled(TupleFromPacks(P,L))
    -> L
```

#### 4. Tuple At

Concrete Tuple:

```text
N = component count
normalize negative index
index Box slice directly
return payload TypeId
```

Zero product:

```text
any concrete index -> Invalid bounds
```

TupleFromPacks:

- if packs are explicit enough to materialize, materialize then index;
- otherwise Stuck/symbolic projection unless the accepted spec provides a deterministic known prefix rule.

Do not add pack arithmetic.

#### 5. TypePack Map

Explicit pack:

```text
for each ProperTypeId:
    apply_type_form(F, [Ti])
    require result kind Type
intern TypePack
```

Symbolic pack parameter/projection:

```text
retain Map projection
```

unless an inner intrinsic can be reduced safely.

#### 6. LabeledTypePack Map

Same as TypePack, preserving labels byte-for-byte/order-for-order.

#### 7. Tuple Map

Concrete Tuple:

```text
map every payload through F
preserve every label and lane boundary
checked tuple formation
```

TupleFromPacks:

```text
TupleFromPacks(
    Map(positionals,F),
    Map(labeled,F)
)
```

This is why stable symbolic pack Map and TupleFromPacks belong in P1.

#### 8. RecordRow Map

Closed row:

```text
map each canonical field payload
retain canonical key set
```

Open row:

```text
mapped explicit fields
tail = stable projection Map(original tail, F)
```

Do not copy the original tail unchanged.

If row tail is already a stable projection, mapping wraps/composes it as a new canonical row projection.

Do not implement a generic rewrite/fusion optimizer in P1.

#### 9. Record Map

For `TypeData::Record(row)`:

```text
map row
form canonical Record proper type
```

For zero product, follow the live accepted zero-product rule.

#### 10. Map/At law

The normalizer should naturally satisfy:

```text
(T::Map<F>)::At<I>
≡
F<T::At<I>>
```

for concrete valid I.

Do not hard-code a separate rewrite solely to make the law test pass unless necessary for symbolic canonicalization and accepted by the spec.

#### 11. No union distribution

`Union` subject does not branch-map in P1.

Return the accepted invalid/stuck behavior.

### Forbidden approaches

- no runtime Tuple materialization;
- no `Value` construction;
- no reflection-based shape lookup;
- no `Dynamic` on bad Map/At;
- no label sorting for Tuple/labeled pack;
- no leaving open row tail unchanged;
- no type constructor application by formatting/reparsing;
- no open rewrite family.

### Test changes required

Cover `CV-10` through `CV-22`, `CV-27`, `CV-29`.

Required examples include:

```text
(Int,String)::At<0>      -> Int
(Int,String)::At<-1>     -> String
(Int,name:String)::At<1> -> String

(Int,String)::Map<Option>
    -> (Option<Int>, Option<String>)

(Int,name:String)::Map<Option>
    -> (Option<Int>, name:Option<String>)

TypePack[Int,String] Map Option
LabeledPack[(name,String)] Map Option

#{name:String, age:Int}::Map<Option>
    -> #{name:Option<String>, age:Option<Int>}
```

Hostile cases:

```text
At<too-large>
At<too-negative>
Map<Int>
swapped Tuple labels remain distinct
Record key presentation order does not change result identity
open row tail becomes mapped projection
union does not distribute
```

### Tests to run now

Focused new type-member suite only:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_members -- --nocapture
```

If open-row tests are placed with row foundations:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::record_row_inference -- --nocapture
```

### Acceptance

T5 passes when the structural laws are implemented directly by canonical semantic terms and no runtime support is involved.

### Local STOP / CONSULT triggers

- open-row Map cannot preserve symbolic transformation without redesigning row solver semantics;
- zero-product authority conflict remains;
- a Map result cannot be represented without global heterogeneous generic application;
- symbolic Tuple Map requires pack arithmetic beyond TupleFromPacks + symbolic Map.

---

## G3 — Structural algebra gate

### Purpose

Prove the language-level operations, including hostile cases.

Run the focused type-member + row tests.

Required evidence:

```text
Positionals                         PASS
Labeled                             PASS
positive At                         PASS
negative At                         PASS
bounds errors                       PASS
Tuple Map                           PASS
pack Map                            PASS
labeled-pack Map                    PASS
Record Map                          PASS
open-row mapped tail                PASS
Map/At composition                  PASS
union non-distribution              PASS
```

Do not proceed to persistence/inference work while G3 is red.

---

## T6 — Integrate packs/projections with substitution, materialization, and query-local inference

### Purpose

Make the new canonical forms compose with existing generic code without creating a parallel solver or leaking query-local identities.

### Risk

- Semantic: HIGH
- Implementation fanout: semantic inference/type machinery

### Preconditions

- G3 PASS;
- canonical forms stable.

### Consumes

- TypePack/LabeledTypePack TypeIds;
- TupleFromPacks;
- generalized projection;
- row projection tail.

### Produces

- correct lifting/zonking/materialization;
- generic pack parameters work as kinded variables;
- row projections survive row solving;
- existing generic application remains sound.

### Files and symbols

**Primary:**

```text
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/row_inference.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/type_lambda.rs
```

**Tests:**

```text
phalcom-semantic/tests/semantic/foundations/generic_application.rs
phalcom-semantic/tests/semantic/foundations/record_row_inference.rs
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
```

### Source of truth

- canonical forms in TypeStore/row arena;
- query-local variables in existing inference/row solver.

### Required implementation shape

#### 1. Reuse ordinary kinded inference for pack kinds

`GenericApplicationSession::new_for_domains` already does:

```text
RecordRow kind -> RecordRowVarId
other kind     -> InferVarId(kind)
```

Keep this architecture.

A `TypePack` parameter gets:

```text
InferVarId(kind = TypePack)
```

A `LabeledTypePack` parameter gets:

```text
InferVarId(kind = LabeledTypePack)
```

Do not add `PackVarId` in P1 merely for whole-pack equality.

#### 2. Extend InferenceTerm lifting

Add forms or recursively lift canonical nodes so the solver can traverse:

```text
TypePack
LabeledTypePack
TupleFromPacks
Projection
```

Where a projection is opaque/stuck, it may remain a canonical leaf if no inference variable occurs inside it. If arguments/subject contain inference variables, lift them structurally enough to preserve inference.

Do not stringify/reparse.

#### 3. Enforce kind-aware solving

An `InferVarId(kind=TypePack)` can solve only to a TypeId with kind TypePack.

Same for LabeledTypePack.

Existing kind checks should enforce this; extend exhaustive branches rather than add pack-only exceptions.

#### 4. Materialization/substitution

`GenericInstantiation` can continue using `type_bindings` for any kinded `TypeId`, including pack kinds.

Do not rename it to “proper type bindings.”

When materializing:

```text
Parameter(P: TypePack)
    -> bound TypeId(kind TypePack)
```

TupleFromPacks may then collapse to concrete Tuple.

#### 5. Row projection tails

Row materialization/solver must distinguish:

```text
Parameter tail
query-local Var tail
stable Projection tail
```

A stable projection tail is not underconstrained merely because it is symbolic.

Row equality/unification may compare identical stable projection identity.

P1 does not need to solve arbitrary equations *through* an opaque intrinsic row projection unless the projection can first normalize.

#### 6. Type lambda compatibility

`F: Type -> Type` already uses ordinary type-lambda/HKT application.

Pack-kinded lambda parameters should remain kind-correct if source permits them, but P1 need not add pack expansion inside lambda bodies.

RecordRow arguments remain on the existing distinct limitation path unless the live accepted spec explicitly expanded type lambdas over rows.

### Forbidden approaches

- no second generic application solver;
- no `InferVarId -> TypeId` canonicalization;
- no PackVarId unless a live required case proves whole-pack kinded inference insufficient;
- no solving row projection by converting it to `Dynamic`;
- no language-wide GenericArgument refactor.

### Test changes required

Cover `CV-21`, `CV-23`, `CV-29`.

Required cases:

- generic `P: TypePack` passed through `P::Map<Option>`;
- pack parameter substituted with explicit pack;
- TupleFromPacks materializes after pack substitution;
- wrong-kind inferred candidate rejected;
- row projection tail survives materialization;
- snapshots/debug output contain no query-local inference IDs.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_application -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::record_row_inference -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
```

### Acceptance

T6 passes when generic substitution/inference can carry the new domains without new canonical leakage or a second inference architecture.

### Local STOP / CONSULT triggers

- pack inference requires sequence splitting to pass a P1-required test;
- existing InferenceSession cannot solve a non-Type kind without violating its documented model;
- stable row projection tails require treating query-local variables as durable.

---

## T7 — Extend stable metadata, fingerprints, and semantic source targets

### Purpose

Make the new canonical semantic products durable and incrementally stable.

### Risk

- Semantic: HIGH
- Implementation fanout: cross-crate metadata + semantic exporter

### Preconditions

- T6 PASS;
- canonical data model no longer expected to change mechanically.

### Consumes

- new kinds/forms/projection/member IDs;
- existing stable metadata graph.

### Produces

Stable, validated representation for:

```text
TypePack
LabeledTypePack
TupleFromPacks
generalized projection
type-member identity
typed projection arguments
row projection tails
```

### Files and symbols

**Modify:**

```text
phalcom-type-meta/src/kind.rs
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/validate.rs

phalcom-semantic/src/metadata/export.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/*
```

**Tests:**

```text
phalcom-semantic/tests/semantic/integration/metadata.rs
phalcom-semantic/tests/semantic/integration/source_index.rs
optional focused incremental test if the live query graph exposes member products
```

### Source of truth

Stable metadata encodes canonical semantic identity only.

### Required implementation shape

#### 1. Stable kinds

Add:

```rust
KindNode::TypePack
KindNode::LabeledTypePack
```

Export and validate them.

#### 2. Stable pack nodes

Add stable `TypeNode` forms for explicit packs and TupleFromPacks.

Use stable child `TypeNodeId`s; do not serialize `ProperTypeId` or `TypeId`.

#### 3. Stable member/projection identity

Introduce stable representation for intrinsic member ID:

```text
domain enum
selector enum
```

or a stable source-declaration reference + typed intrinsic operation where the accepted metadata spec requires it.

Do not serialize a store-local `ProjectionId`.

Serialize projection structure:

```text
subject
member
arguments
result domain
```

#### 4. Stable const arguments

Serialize:

```text
Const(Int(i64))
```

and stable parameter refs if const member parameters can survive into durable terms.

No runtime `Value`.

#### 5. Stable row projection tail

Extend stable open-row representation so a tail may be:

```text
Closed
StableParameter
StableProjection
```

Do not degrade it to closed/open unknown.

#### 6. Metadata validator

Validate:

- referenced node IDs exist;
- member result domain matches node kind/row position;
- pack payloads are Type-kind nodes;
- projection arguments match serialized member signature shape where signature is present;
- no cycles violate existing type graph rules.

#### 7. Fingerprints

AST/source semantic fingerprints must include:

```text
new kind spellings
type-member declaration selector
parameter domains
result domain
implementation/intrinsic metadata that changes semantics
type-member projection syntax and arguments
```

Source range movement alone must not become semantic identity.

#### 8. Source index

A type-member declaration and a projection occurrence should target the canonical semantic member identity.

`T::At<I>` and `T[I]` should target the same member.

Do not build an LSP-only resolver.

### Forbidden approaches

- no raw `TypeId`, `KindId`, `ProjectionId`, `InferVarId`, `RecordRowVarId`;
- no display string as identity;
- no separate metadata-only type-member identity;
- no source-index name guessing.

### Test changes required

Cover `CV-23` through `CV-26`, `CV-30` where practical.

Metadata test should inspect node variants/identity, not only successful JSON/binary encoding.

Hostile case:

- two same-spelled Map members in different domains remain distinguishable.

### Tests to run now

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::integration::metadata -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::integration::source_index -- --nocapture
```

Run an incremental focused test only if one was added.

### Acceptance

T7 passes when every durable new semantic form can be exported/validated without local IDs and member source identity is canonical.

### Local STOP / CONSULT triggers

- metadata format cannot represent the new terms without a version/schema migration not anticipated by this plan;
- stable identity appears to require source range or raw arena index;
- source index requires LSP-specific semantic lookup.

---

## G4 — Inference/persistence gate

### Purpose

Prove canonical/query-local lifetime separation and durable identity.

Run T6/T7 focused suites.

Expected:

```text
pack-kinded generic inference: PASS
row stable projection tail: PASS
metadata validation: PASS
source target identity: PASS
existing associated-type metadata: PASS
no query-local ID leakage: PASS
```

On failure, classify before editing.

---

## T8 — Close Universe/core integration and focused stabilization

### Purpose

Ensure real canonical Universe source loads the new declarations, semantic analysis consumes them, and runtime/compiler layers do not acquire a competing execution mechanism.

### Risk

- Semantic: MEDIUM
- Implementation fanout: cross-crate integration

### Preconditions

- G4 PASS.

### Consumes

- semantic implementation;
- Tuple/Record Universe declarations;
- current Universe source provider/native census.

### Produces

End-to-end source-level examples and negative runtime-boundary evidence.

### Files and symbols

**Inspect/modify only if needed:**

```text
phalcom-core/src/native/source.rs
phalcom-core/src/modules/compile.rs
Universe source parsing/provider code
```

**Tests:**

Prefer existing semantic workspace/Universe integration harness.

Add the smallest source-level regression proving real syntax resolves through canonical Universe declarations.

### Required implementation shape

#### 1. Parse/load canonical Universe source

Ensure Tuple/Record files containing type-member declarations load through the normal Universe source path.

#### 2. Do not register type members as runtime methods

Search the native source census/registration path.

Any new `ClassMember::TypeMember` arm should be semantically explicit:

```text
source semantic declaration
    yes

runtime NativeMemberAnchor
    no
```

#### 3. End-to-end semantic fixture

Use actual source aliases/annotations if supported, for example:

```phalcom
type A = (Int, String)::Map<Option>
type B = (Int, String)[1]
type C = #{ name: String }::Map<Option>
```

Assert canonical semantic forms, not only formatted text.

Also verify:

```phalcom
type Bad = (Int, String)::Map<Int>
```

produces the expected semantic diagnostic.

#### 4. Runtime negative search

There should be no new production symbols suggesting runtime type-member execution.

Recommended searches:

```sh
rg -n 'InvokeTypeMember|TypeMemberBytecode|RuntimeTypeMember|MapTypeMember' phalcom-core/src
```

Expected: no execution mechanism; any names introduced solely for semantic metadata must be justified.

Also inspect `Bytecode` enum diff; it should contain no LANG007 type-member bytecode.

### Forbidden approaches

- no VM workaround because source integration fails;
- no native runtime function returning type descriptors as Map/At result;
- no compiler syntax shortcut that bypasses semantic projection;
- no manual Universe name lookup when canonical declaration source is available.

### Test changes required

Cover `CV-08`, `CV-09`, `CV-15`, `CV-19`, `CV-28`.

### Tests to run now

Focused core check:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core
```

Run only the exact new Universe/source integration test if a core test was required.

Then re-run the focused semantic type-member suite once.

### Acceptance

T8 passes when canonical source examples work, incorrect examples fail statically, and the runtime has no new type-member execution path.

### Local STOP / CONSULT triggers

- Universe source parser can only accept native type members by registering them as runtime callables;
- compiler must reconstruct Map/At independently of semantic products;
- a new VM instruction appears necessary.

---

# 19. Verification gates

## G5 — Focused plan acceptance

### Purpose

Prove the coherent P1 code slice without workspace-wide certification.

Run:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test parser <type-member-filter> -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_annotations -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::type_members -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::record_row_inference -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::foundations::generic_application -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::capabilities::traits -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::integration::metadata -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic \
  semantic::integration::source_index -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-type-meta
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core

cargo +stable fmt --all -- --check
git diff --check
```

Adapt module filter names to the live tree if necessary.

### Expected

All focused LANG007 coverage passes.

Existing associated-type tests remain green.

No new runtime execution mechanism exists.

Do not automatically run workspace tests after G5 PASS.

---

# 20. Final focused acceptance

At completion, produce a result table equivalent to:

| Coverage ID | Evidence | Required result |
|---|---|---|
| CV-01..05 | kind/type-model tests | PASS |
| CV-06..09 | parser + member identity tests | PASS |
| CV-10..18 | Tuple/pack intrinsic tests | PASS |
| CV-19..22 | Record/open-row/symbolic tests | PASS |
| CV-23 | generic/inference lifetime tests | PASS |
| CV-24 | existing trait associated tests | PASS |
| CV-25 | metadata tests | PASS |
| CV-26 | source-index tests | PASS |
| CV-27 | zero-product test per live spec | PASS |
| CV-28 | runtime negative search/check | PASS |
| CV-29 | TupleFromPacks canonicalization | PASS |
| CV-30 | incremental parity, if applicable | PASS or explicitly not applicable with reason |

P1 code implementation is **FOCUSED_TESTED**, not release-certified, when this table is complete and G5 passes.

---

## 21. Performance/resource evidence

No benchmark suite is required.

P1 has architectural asymptotic requirements:

```text
Tuple At on concrete shape
    O(1)

explicit pack / Tuple / Record Map
    O(n) in payload count

projection interning
    expected O(1) hash-cons lookup

canonical ID equality
    O(1)

no VM transition
    required
```

Evidence:

- code inspection of direct tuple slice indexing;
- focused tests over multiple arities;
- no repeated linear member-name search in intrinsic evaluation;
- no benchmark ceremony.

STOP AND CONSULT if concrete `At` is implemented by scanning labels/elements from the start despite canonical component index being directly available.

---

## 22. Checkpoint bookkeeping — intentionally excluded

By explicit user instruction, this plan does **not** modify implementation records.

Do not edit:

```text
LANG007 PROGRAM.md
LANG007.C1 CHECKPOINT.md
plan ledger
walkthrough
handoff
STATUS.md
INDEX.md
```

during this patch.

Instead, the implementing agent's completion response must state:

```text
starting revision
ending revision / working-tree state
tasks completed
gates passed
tests actually run
tests deferred
consultations
residual risks
```

A later explicit documentation-only action may transfer those facts into repository records.

---

## 23. Walkthrough deliverable — out-of-band only

Do not create a walkthrough Markdown file in this patch.

At completion, return a concise implementation walkthrough containing:

- final result;
- architecture actually implemented;
- important files/symbols changed;
- mechanical deviations;
- any consultations;
- tests added;
- tests actually run;
- tests deliberately deferred;
- verification classification;
- residual risks.

---

## 24. Handoff deliverable — out-of-band only

Do not create a handoff file.

If successor LANG007 work remains, include in the final response:

- state anchor/revision;
- stable interfaces;
- invariants;
- next objective;
- must-read source paths;
- deferred failures;
- first recommended commands;
- “do not redesign” list.

---

## 25. Completion truth table

Do not conflate:

```text
AST support written
canonical domains implemented
intrinsic evaluator implemented
tests added
focused tests passed
affected crates compile
workspace tests passed
checkpoint accepted
release certified
```

For this plan, the target is:

```text
source written:       YES
focused tests added:  YES
focused tests passed: YES
affected crates check: YES
workspace test:       DEFERRED
release certified:    NO
```

unless the user explicitly requests broader certification.

---

## 26. Suggested commit grouping

Do not force one commit per task.

Coherent groups:

### Commit A — canonical domains and syntax

```text
feat(lang007): add pack kinds and type-member syntax
```

Contains T1–T2.

### Commit B — member/projection architecture

```text
feat(lang007): add intrinsic type-member projections
```

Contains T3–T4.

### Commit C — structural intrinsic algebra

```text
feat(lang007): implement tuple and record type operations
```

Contains T5–T6 where coherent.

### Commit D — persistence and integration

```text
feat(lang007): persist intrinsic type-level terms
```

Contains T7–T8.

Commit grouping is advisory; preserve unrelated working-tree changes.

---

# 27. Tempting wrong fixes

The following solutions may look easier and are prohibited.

### Wrong: encode `TypePack` as a Tuple proper type

Why wrong:

```text
pack kind != Type
pack is a type-level sequence
Tuple is a runtime proper structural type
```

### Wrong: encode Tuple labeled pack as RecordRow

Why wrong:

```text
Tuple label order is semantic
RecordRow key order is not
```

### Wrong: add `Int` as a kind

Why wrong:

```text
I: Int
```

is a typed compile-time value domain, not kind classification.

### Wrong: put `At<1>` argument in TypeData::Applied arguments as a fake TypeId

Why wrong:

`1` is not a type form in this feature.

### Wrong: make every generic argument in the whole compiler heterogeneous in P1

Why wrong:

the P1 need is type-member argument application; broad nominal/trait const generics are separate scope.

### Wrong: create `TupleMapProjection`, `TupleAtProjection`, `RecordMapProjection` node types separately

Why wrong:

duplicates canonical projection identity/normalization infrastructure.

### Wrong: resolve `Map` by member name alone

Why wrong:

Tuple/Record/pack/trait members may share spelling without shared identity.

### Wrong: make `At` and `[]` separate members and compare their outputs

Why wrong:

the accepted law is one operation identity, not accidental equal reduction.

### Wrong: evaluate Map through runtime Tuple/Record APIs

Why wrong:

type checking must be pure and VM-independent.

### Wrong: open-row Map maps visible fields but keeps original tail

Why wrong:

unknown tail fields would remain untransformed and violate Map semantics.

### Wrong: unresolved projection becomes Dynamic

Why wrong:

abstract-but-well-formed type information is not a dynamic escape.

### Wrong: introduce PackVarId now

Why wrong:

whole-pack generic variables already fit the kinded `InferVarId` architecture. A sequence solver is justified only when later expansion/splitting inference requires it.

### Wrong: use LSP/source-index logic as semantic member resolution

Why wrong:

tooling consumes canonical semantic identity; it does not own language semantics.

---

# 28. Failure protocol

If a planned semantic test fails unexpectedly, stop expansion and establish:

## 28.1 Exact reproduction

Record exact command, failing test, assertion/diagnostic.

## 28.2 Direct path

For example:

```text
source T::Map<Option>
    -> AST TypeMemberProjection
    -> annotation resolver
    -> TypeMemberId
    -> ProjectionId
    -> intrinsic normalizer
    -> apply_type_form
    -> wrong result
```

## 28.3 Passing comparator

Find a nearby working case, such as:

```text
Tuple Map passes, Record Map fails
positive At passes, negative At fails
concrete pack Map passes, symbolic pack Map fails
trait associated projection passes, intrinsic projection fails
```

## 28.4 Classification

```text
PRODUCT
FIXTURE
DEPENDENCY/PUBLICATION
BACKEND/HARNESS
BASELINE
PLAN DRIFT
```

## 28.5 Narrow repair boundary

State the exact subsystem permitted to change.

## 28.6 Rejected broad repairs

Never repair by:

- adding Dynamic fallback;
- disabling a diagnostic;
- restoring enum-case ambiguity;
- moving computation into VM;
- generalizing all generics;
- special-casing one test's type spelling.

---

# 29. Consultation packet guidance

If a RED trigger fires, collect cheaply:

```text
exact live spec rule
current source representation
minimal failing source
canonical TypeId/KindId or RecordRow term involved
projection/member identity
normalization state
focused diff
one passing comparator
```

Ask a decision question.

Example:

> The accepted open-row Map law requires mapping the unknown tail. The current `RecordRowTail` supports only `Closed|Parameter`, while the generalized projection arena is already live. Should row-valued residuals be represented as `RecordRowTail::Projection(ProjectionId)` as planned, or does the live row solver expose another canonical stable-term owner that must be reused?

Do not ask an adviser to rediscover the repository.

---

# 30. Final negative gates

At the end, inspect production source.

## No runtime type-member execution

```sh
rg -n 'InvokeTypeMember|TypeMemberBytecode|RuntimeTypeMember' phalcom-core/src
```

Expected:

```text
zero execution-path hits
```

## No pack-as-row shortcut

```sh
rg -n 'TypePack.*RecordRow|LabeledTypePack.*RecordRow' phalcom-semantic/src
```

Review every hit; only explicit distinction/comments/conversion utilities may remain.

## No query-local ID serialization

```sh
rg -n 'InferVarId|RecordRowVarId' phalcom-type-meta/src
```

Expected:

```text
no serialized node fields using those IDs
```

Imports/comments/tests that explicitly reject them must be justified.

## No At/string identity shortcut

Search new type-member implementation for raw `"At"` / `"Map"` comparisons.

Every remaining string comparison must be parser/presentation/source-binding logic, never canonical semantic identity or intrinsic dispatch.

## No type-member runtime callable anchor

Inspect changes in:

```text
phalcom-core/src/native/source.rs
```

and verify type members never produce `NativeMemberKey` or callable descriptors.

---

# 31. Deferred evidence audit

Before declaring P1 focused-complete:

Every deferred gate must be one of:

```text
explicitly deferred to later LANG007 checkpoint
not applicable
known baseline issue
user-requested certification not performed
```

Do not imply workspace tests/clippy passed.

---

# 32. Known scope exclusions after P1

Expected follow-on LANG007 work may include:

- true variadic pack expansion syntax;
- one-unknown-pack sequence inference;
- pack expansion in Tuple/callable formations;
- richer const-term language;
- language-wide const generic parameters;
- additional intrinsic operations (`Zip`, `Concat`, `Filter`, etc.);
- type-level Symbol/Range indexing;
- richer reflection for type members/packs;
- union-distributive operations if ever specified;
- GATs if separately designed;
- runtime reification of new type-level descriptors where required.

P1 must leave clean semantic seams for these without implementing them.

---

# 33. Release-complete criteria for this plan

The **P1 code patch** is complete when:

- [ ] T0 confirmed the live accepted semantic contract.
- [ ] T1 added neutral source syntax without breaking trait/enum syntax.
- [ ] T2 added canonical pack kinds/forms and checked Tuple formation.
- [ ] G1 passed.
- [ ] T3 established canonical type-member identity/source anchors.
- [ ] T4 established typed arguments and generalized symbolic projections.
- [ ] G2 passed.
- [ ] T5 implemented the structural intrinsic algebra.
- [ ] G3 passed.
- [ ] T6 integrated inference/materialization without new solver leakage.
- [ ] T7 added durable metadata/source identity.
- [ ] G4 passed.
- [ ] T8 proved Universe/core integration with no runtime type-member execution.
- [ ] G5 focused acceptance passed.
- [ ] all hostile cases pass.
- [ ] no active RED incident remains.
- [ ] broad workspace certification is truthfully marked deferred.
- [ ] no documentation/specification files were modified by this patch.

---

# 34. Luna execution order

Use this exact semantic order:

```text
T0  live authority/drift verification
 ↓
T1  syntax/AST
 ↓
T2  canonical domains/forms
 ↓
G1  domain gate
 ↓
T3  type-member identity/source anchors
 ↓
T4  projection + typed arguments
 ↓
G2  identity/projection gate
 ↓
T5  intrinsic structural algebra
 ↓
G3  algebra gate
 ↓
T6  inference/materialization
 ↓
T7  metadata/source persistence
 ↓
G4  persistence gate
 ↓
T8  Universe/core integration
 ↓
G5  focused acceptance
```

Do not reorder T5 before T4.

Do not start metadata representation before the canonical projection/member structure is stable.

Do not add runtime support as an intermediate shortcut.

---

# 35. Plan self-review

Before implementation begins, the supervising architect should verify:

### Architecture

- [x] one owner exists for kinded canonical forms;
- [x] RecordRow remains separate;
- [x] one projection identity exists;
- [x] one type-member identity exists;
- [x] member argument domains are explicit;
- [x] type computation stays out of the VM;
- [x] open-row transformation has a stable residual representation;
- [x] existing associated-type proof remains authoritative.

### Luna executability

- [x] tasks name concrete current files and symbols;
- [x] structural pseudocode is labeled STRUCTURAL rather than falsely paste-ready;
- [x] fixed/flexible/verify-first decisions are explicit;
- [x] RED triggers are objective;
- [x] semantic debugging allows one serious correction;
- [x] repository wandering is bounded.

### Testing

- [x] coverage follows semantic risks;
- [x] hostile cases defeat likely shortcuts;
- [x] existing associated-type compatibility is explicitly tested;
- [x] metadata/source identity is tested;
- [x] broad workspace tests are deferred;
- [x] every gate has a semantic purpose.

### Scope

- [x] no documentation edit tasks remain;
- [x] no general const-generic migration is planned;
- [x] no GAT implementation is planned;
- [x] no pack split solver is planned;
- [x] no runtime TypePack object is planned;
- [x] no VM type-member execution is planned.

### Dependency order

- [x] AST before semantic lowering;
- [x] domains before projections;
- [x] identity before intrinsic algebra;
- [x] algebra before inference/persistence stabilization;
- [x] metadata after canonical forms settle;
- [x] core integration last.

---

# 36. Compact implementation doctrine

Luna should keep these sentences visible while implementing:

> `TypeId` identifies a kinded type-level form; `ProperTypeId` proves it is a proper type.

> TypePack is ordered type-level structure, not Tuple.

> LabeledTypePack is ordered; RecordRow is key-canonical.

> A compile-time `Int` member argument is not a kind and not a runtime Value.

> `[]` and `At` are one member identity.

> Map applies ordinary `Type -> Type` constructors; it does not invent a new application engine.

> Symbolic projections stay symbolic.

> Query-local solver IDs never become canonical.

> Source `native type` anchors semantic behavior; it does not create a runtime callable.

> If a correct implementation seems to require general const generics, GATs, VM execution, or a second semantic authority, STOP AND CONSULT.
