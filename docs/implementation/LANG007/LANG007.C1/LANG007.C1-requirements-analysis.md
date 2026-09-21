# LANG007.C1 — Requirements Analysis
## Semantic Contract and Specification Convergence for Variadic Structural Typing and Intrinsic Type Operations

**Program:** `LANG007` — Variadic Structural Typing and Intrinsic Type Operations
**Checkpoint:** `LANG007.C1`
**Document kind:** Requirements analysis
**Status:** Proposed analysis for checkpoint planning and specification work
**Repository:** `aureat/phalcom-lang`
**Prepared:** 2026-09-21

---

# 1. Executive Summary

`LANG007.C1` should be treated as a **semantic-contract and specification-convergence checkpoint**, not as the checkpoint that implements tuple-pack inference, new compiler arenas, metadata serialization, or runtime behavior. Its purpose is to establish one internally coherent language model that later `LANG007` checkpoints can implement without making local semantic choices.

The feature family covered by `LANG007` is broader than “tuple generics.” The intended surface includes, at minimum:

```phalcom
(Int, String)::Positionals
(Int, String, name: Symbol)::Labeled

(Int, String)[1]
(Int, String)::At<1>

(Int, String)::Map<Option>
(Int, String, name: Symbol)::Map<Option>

#{name: String, age: Int}::Map<Option>
```

and, when the corresponding generic domains are abstract:

```phalcom
P::Map<Option>       // P : TypePack
L::Map<Option>       // L : LabeledTypePack, if publicly exposed
R::Map<Option>       // R : RecordRow
```

This surface requires Phalcom to specify several concepts that are currently either absent from the authoritative `docs/specs/` corpus or specified more narrowly than `LANG007` requires:

1. **Type-level structural domains.** `TypePack` and `LabeledTypePack` must become distinct kind/domain categories alongside the already-existing `Type` and `RecordRow`.
2. **Typed compile-time parameter domains.** `I: Int` must mean “`I` ranges over compile-time values whose value type is `Int`,” not “`Int` is a kind.”
3. **General type members.** Phalcom needs a semantic owner for inherent/compiler-defined type members as well as trait-owned associated types.
4. **Parameterized type members.** `At<I>` and `Map<F>` require type members with their own parameter lists.
5. **Non-`Type` projection results.** `Positionals` and `Labeled` return `TypePack` and `LabeledTypePack`; therefore a projection cannot be defined only as “a proper type expression.”
6. **Structural Tuple/Record typing.** The authoritative language corpus currently lacks a language-level owner for structural Tuple/Record type formation and identity. `objects/values.md` explicitly excludes this domain, while `vm/products.md` explicitly says that VM product representation does not replace tuple/record typing semantics.
7. **Closed structural type operations.** `At` and `Map` should be compiler-known, deterministic, bounded type operations—not an introduction of open type families or arbitrary compile-time Phalcom execution.
8. **Zero-product semantics.** The live implementation and VM specifications normalize the zero-arity tuple to `Unit`. `LANG007` must therefore specify how Tuple-family type operations apply to `()` without inventing a second canonical empty tuple type.
9. **Ordered versus unordered structural domains.** Tuple positional and labeled lanes are ordered; `RecordRow` is canonicalized by field name and is semantically unordered. Tuple labeled packs must not be implemented by reusing row identity.
10. **One semantic identity for type subscript and `At`.** `T[I]` and `T::At<I>` should denote the same type-member operation, not two independently declared members that happen to normalize to the same result.
11. **Shape-preserving `Map`.** Tuple and Record mapping should be specified as the same general structural operation over different shape domains: transform payload types, preserve structural shape and the identity rules of the subject domain.
12. **Native type-member implementation origin.** The current `native` specification says the live implementation is supplied by the native runtime. `LANG007` needs a generalization that also covers compiler/semantic native implementations while preserving the distinction between `native` and `@intrinsic`.

The central recommendation of this analysis is therefore:

> **C1 should establish a small, closed, multi-domain type-level calculus with explicit ownership boundaries. It should not define LANG007 as a collection of Tuple and Record special cases.**

The clean semantic decomposition is:

```text
Type-level domains
    Type
    RecordRow
    TypePack
    LabeledTypePack
    Arrow kinds over kinded domains

Compile-time value domain
    Const<T> / “compile-time value typed T”
    represented as a parameter domain, not as a kind

Type members
    subject-owned/inherent
    trait-owned/associated
    compiler/domain-native

Structural operations
    Positionals
    Labeled
    At<I>
    Map<F>
```

The recommended C1 deliverables are:

- a ratified formal semantic contract for the domains and operations above;
- three new authoritative language specifications:
  - `docs/specs/type-level-domains.md`;
  - `docs/specs/type-members.md`;
  - `docs/specs/structural-products.md`;
- targeted amendments to:
  - `docs/specs/README.md`;
  - `docs/specs/generic-constraints.md`;
  - `docs/specs/associated-types.md`;
  - `docs/specs/source-declarations.md`;
  - `docs/specs/objects/impl.md`;
  - `docs/specs/objects/traits.md`;
  - `docs/specs/universe.md`;
- a cross-specification conformance matrix demonstrating that the new model does not contradict:
  - current Tuple/Record runtime semantics;
  - current `Unit` normalization;
  - current row ordering/identity;
  - current higher-kinded `Type -> Type` model;
  - current conformance and associated-type identity rules;
  - current declaration-origin rules.

C1 should **not** implement the new Rust representation, pack solvers, generic argument migration, metadata schema, LSP behavior, or VM reflection. Those belong to later checkpoints. C1 must, however, make enough representation-independent decisions that those later checkpoints do not need to reopen language semantics.

---

# 2. Checkpoint Purpose

The bounded acceptance objective for `LANG007.C1` should be:

> **Phalcom has one authoritative, internally consistent semantic model for variadic structural typing, typed compile-time generic arguments, parameterized type members, and compiler-native structural type operations, and every existing normative specification that overlaps that model has been reconciled with it.**

C1 owns the **meaning** of the language feature. It does not own the final compiler data structures or solver algorithms.

The checkpoint should answer questions such as:

- What is the kind of a type pack?
- Is a labeled type pack a record row?
- What exactly does `I: Int` mean?
- Is `Int` a kind in that position?
- What is the semantic category of `Option` when passed to `Map<Option>`?
- Is `Map` a Tuple-specific intrinsic or a structural operation over shape domains?
- What does `Map` do to an open record row?
- What is the result domain of `Tuple::Positionals`?
- Can a type member return a non-`Type` result?
- What is the identity of a type projection?
- What is the semantic relationship between `T[I]` and `T::At<I>`?
- Does `()` count as the zero-arity tuple type even though it canonicalizes to `Unit`?
- Does `()::Map<Option>` reduce to `()`?
- Does Tuple label order participate in type identity?
- Does Record field order participate in type identity?
- What happens when `Map` or `At` cannot yet reduce because the subject or argument is generic?
- What happens when a concrete index is invalid?
- Can `Map` execute arbitrary user-defined compile-time code?
- What exactly does `native type` mean?
- Is `@intrinsic` required, descriptive, or semantic?
- Can anonymous structural Tuple/Record types acquire ordinary inherent `impl`s?
- How do compiler-provided structural type members coexist with the current rule that anonymous structural types are not legal inherent-`impl` targets?
- If an inherent/domain type member and a trait-associated type have the same spelling, how is unqualified `::` lookup resolved?
- Does `Map` distribute across unions?
- Are packs runtime values or only semantic type-level objects?
- Which parts are in the first LANG007 release and which are deliberately deferred?

If C1 leaves any of these questions to C2–C5, later implementation plans will be forced to encode language design decisions in Rust representation choices. That would defeat the purpose of the checkpoint.

---

# 3. Current Authoritative Specification Baseline

The current `docs/specs/README.md` establishes two critical rules:

1. `docs/specs/` is the authoritative current semantic corpus.
2. one semantic concept should have one primary owner; implementation plans and VM mechanisms may realize semantics but must not silently redefine them.

That authority model is especially important for LANG007 because several relevant concepts currently have no single language-level owner.

## 3.1 `generic-constraints.md`

The current specification defines:

```text
:   kind classification
|=  trait conformance
<:  subtyping
==  canonical type equivalence
```

and explicitly describes:

```phalcom
where F: Type -> Type
```

as a **kind constraint**.

It also states that ordinary generic parameters have kind `Type` unless another kind is established.

This is compatible with:

```text
F : Type -> Type
P : TypePack
R : RecordRow
```

but is **not sufficient** for:

```phalcom
I: Int
```

because `Int` is a proper type, not a kind. C1 must therefore either:

- generalize the semantic meaning of binder `:` into domain classification; or
- introduce distinct syntax for typed compile-time value parameters.

The previously chosen LANG007 direction is the former: `I: Int` should remain legal and should denote a typed compile-time value parameter. C1 must make that distinction normative and must ensure that it does not turn every proper type into a kind.

## 3.2 `associated-types.md`

The current associated-type specification is intentionally narrower than LANG007 requires.

It says associated types are trait-owned requirements and explicitly excludes:

- generic associated types;
- higher-kinded associated declarations.

It also states that the associated declaration form it defines denotes **one proper type** per conformance.

That model is correct for existing declarations such as:

```phalcom
trait Iterator {
    type Item
}
```

but it cannot own all of:

```phalcom
type Map<F: Type -> Type>: Type
type Positionals: TypePack
type Labeled: LabeledTypePack
type At<I: Int>: Type
```

because LANG007 needs:

- parameters on type members;
- non-`Type` result domains;
- inherent/domain-native type members, not only trait requirements.

The correct response is not to stretch `associated-types.md` until every type-level member becomes an “associated type.” C1 should introduce a more general `type-members.md` owner and make `associated-types.md` the trait/conformance specialization of that model.

## 3.3 `source-declarations.md`

The current declaration-origin rule states that:

> `native` identifies a declaration whose live implementation is supplied by the native runtime.

That is appropriate for ordinary executable native members. LANG007 introduces a second valid host implementation site:

```phalcom
native type Map<F: Type -> Type>: Type
```

whose implementation belongs to the semantic compiler/type normalizer, not to runtime execution.

C1 must therefore generalize the *category-neutral* meaning of `native` while preserving the category-specific implementation site:

```text
native executable declaration
    host/runtime/native execution implementation

native type member
    host/compiler/semantic normalization implementation
```

`native` must continue to mean implementation origin. `@intrinsic` must continue to be metadata rather than an alternate implementation-origin keyword.

## 3.4 `objects/impl.md`

The current specification says:

> Anonymous structural types such as Tuple and Record are not direct inherent `impl` targets.

This is a good coherence rule and should remain.

LANG007 should **not** reinterpret:

```phalcom
(Int, String)::Map<Option>
```

as evidence that the anonymous tuple type was reopened with an ordinary user-authored inherent implementation.

Instead, structural type members must belong to a separate, compiler-defined/domain-owned type-member mechanism. C1 must specify that distinction explicitly so the feature does not accidentally weaken the existing inherent-implementation ownership model.

## 3.5 `objects/values.md`

The foundational value specification explicitly excludes:

- static type formation;
- kinds/type constructors;
- structural Tuple/Record typing.

Therefore it is not the correct owner for LANG007’s structural type rules.

It may need a narrow cross-reference update, but C1 should not move static tuple/record typing into the general value specification.

## 3.6 `vm/products.md`

The VM product specification records implementation-relevant facts that LANG007 must preserve, including:

- `Unit` is the zero product;
- Tuple is an ordered structural product;
- Record is a structural key product;
- Tuple label order is semantically significant;
- Record key order is not structural identity;
- VM product representation does not replace source-language record/tuple typing.

This is strong evidence for the semantic model, but **not the normative language owner**.

The new language-level structural product specification should state the language rules and then `vm/products.md` should remain the realization contract.

## 3.7 `universe.md`

Universe already distinguishes several independent policies:

```text
Universe
Native
Prelude
Primordial
Source-owned
Runtime-support
```

and describes Native broadly as VM/compiler/runtime-assisted implementation.

That broader platform concept is compatible with compiler-native type members, while `source-declarations.md` currently uses narrower “native runtime” wording. C1 should reconcile these two documents so the general platform concept and source modifier contract agree.

---

# 4. Live Implementation Constraints Relevant to C1

C1 should remain representation-independent, but it must not ratify semantics that unnecessarily fight the current architecture.

The present compiler provides useful seams.

## 4.1 Current kind model

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

This already establishes the principle that `RecordRow` is a distinct kind/domain rather than a proper type.

LANG007 should extend that principle:

```text
Type
RecordRow
TypePack
LabeledTypePack
```

rather than encoding packs as nominal proper types.

## 4.2 Current structural type model

`TypeData` already distinguishes:

```rust
Tuple(...)
Record(...)
Parameter(...)
Lambda(...)
AssociatedProjection(...)
```

Concrete structural Tuple and Record types therefore already have canonical representation.

C1 should explicitly preserve the law:

> LANG007 does not replace concrete Tuple/Record structural type identity with nominal `Tuple<...>` or `Record<...>` applications.

Later checkpoints may add symbolic formation terms, but fully concrete results must normalize back to the existing structural representation.

## 4.3 Current row representation

`RecordRowData` contains:

```rust
pub fields: Box<[RecordRowField]>, // sorted by name, unique
pub tail: RecordRowTail,
```

and row formation sorts fields by name.

This confirms an important semantic distinction:

```text
RecordRow
    unordered structural key domain
    canonical field ordering is implementation normalization

LabeledTypePack
    ordered sequence of (label, type)
    order participates in tuple structural identity
```

C1 must state that these are different domains even though both contain labels and types.

## 4.4 Current generic instantiation

`GenericInstantiation` has separate maps for:

```rust
type_bindings
row_bindings
```

This demonstrates that the current compiler already treats generic arguments as domain-specific.

C1 should therefore specify heterogeneous generic argument domains abstractly, while leaving C2 to choose the exact Rust representation.

## 4.5 Current type lambdas

The compiler already represents:

```text
parameter kinds
result kind
Arrow kinds
```

so `Option : Type -> Type` is a coherent existing higher-kinded type form.

This is exactly the kind of argument `Map<Option>` should consume.

C1 must distinguish:

```text
Option
    type-level form of kind Type -> Type

Option<Int>
    proper type of kind Type
```

and specify that `Map<F>` expects the former category.

## 4.6 Current Tuple runtime semantics

The Universe `Tuple` class exposes:

- positional size;
- labels;
- integer indexing;
- negative integer indexing;
- Symbol lookup;
- ordered label-sensitive equality.

This is relevant to C1 because static `At<I>` should intentionally align with ordinary integer indexing for tuple type shapes, including negative indexing if that choice is ratified.

However, runtime Symbol lookup and Range slicing should **not** automatically become type-level `At` semantics. C1 must define the first supported type-level index domain explicitly.

## 4.7 Current Record runtime semantics

The Universe `Record` class implements equality by key set/value equality rather than presentation order, and its hash is intentionally commutative over presentation order.

That matches canonical `RecordRow` semantics and reinforces that Record mapping preserves the field set but not an identity-bearing field order.

---

# 5. Required Semantic Model

C1 should ratify the following conceptual calculus.

## 5.1 Kinded type-level judgment

Use a conventional kinded judgment:

```text
Γ ⊢ a : κ
```

with at least:

```text
κ ::= Type
    | RecordRow
    | TypePack
    | LabeledTypePack
    | (κ₁, ..., κₙ) -> κ
```

Examples:

```text
Γ ⊢ Int : Type
Γ ⊢ Option : Type -> Type
Γ ⊢ P : TypePack
Γ ⊢ R : RecordRow
```

`TypePack` and `LabeledTypePack` are domains/kinds of type-level forms. They are not ordinary runtime classes and they are not aliases for proper Tuple or Record types.

## 5.2 Compile-time value judgment

Typed compile-time values should use a different judgment, conceptually:

```text
Γ ⊢ c :const τ
```

Examples:

```text
Γ ⊢ 0 :const Int
Γ ⊢ -1 :const Int
Γ ⊢ I :const Int
```

The key law is:

```text
Int : Type
```

and **not**:

```text
Int : Kind
```

Therefore:

```phalcom
I: Int
```

does not mean `Kind(I) = Int`.

It means:

```text
Domain(I) = Const(Int)
```

C1 should explicitly reject any wording that calls `Int` a kind in this binder.

## 5.3 Generic parameter domains

The abstract semantic model should support:

```text
GenericParameterDomain
    Kind(κ)
    Const(τ)
```

Thus:

```phalcom
T: Type
```

means:

```text
Domain(T) = Kind(Type)
```

while:

```phalcom
P: TypePack
```

means:

```text
Domain(P) = Kind(TypePack)
```

and:

```phalcom
I: Int
```

means:

```text
Domain(I) = Const(Int)
```

This gives one coherent meaning to parameter classification without conflating kinds and value types.

## 5.4 Generic arguments

C1 should normatively distinguish argument categories even if C2 decides the concrete Rust enum.

Conceptually:

```text
GenericArgument
    TypeForm
    RecordRow
    TypePack
    LabeledTypePack
    Const
```

The term **TypeForm** is preferable to **Type** for the first category because a valid argument may be higher-kinded:

```text
Option : Type -> Type
```

`Map<Option>` is therefore a generic application whose argument is a type-level form, not necessarily a proper type.

---

# 6. Structural Product Domains

## 6.1 Tuple

A non-empty structural Tuple type has:

```text
Tuple(P, L)
```

where:

```text
P : TypePack
L : LabeledTypePack
```

Conceptually:

```text
(Int, String, name: Symbol)

P = [Int, String]
L = [(name, Symbol)]
```

The Tuple invariant is:

```text
positionals first
ordered labeled suffix second
```

The order of labeled Tuple entries is identity-bearing.

The specification must state whether duplicate Tuple labels are invalid; the recommended rule is **yes**, because label lookup and projection should be unambiguous.

## 6.2 Record

A structural Record type is:

```text
Record(R)
```

where:

```text
R : RecordRow
```

A Record row is key-oriented and order-insensitive for structural identity.

The implementation may maintain canonical sorted storage, but sorting is not a source-level “field order” semantic.

## 6.3 LabeledTypePack is not RecordRow

This distinction must be a named invariant.

For example:

```text
[(name, String), (age, Int)] : LabeledTypePack
```

and:

```text
{name: String, age: Int} : RecordRow
```

contain analogous information but have different identity laws.

A labeled pack preserves sequence position; a row preserves only key association.

The compiler may reuse implementation patterns, helper abstractions, solver infrastructure, or element data types, but it must not make their canonical identities interchangeable.

---

# 7. Zero-Arity Product and `Unit`

The current VM contract normalizes the zero-arity tuple to `Unit`.

C1 must therefore correct the simplistic law:

```text
every Tuple shape is TypeData::Tuple
```

into a semantic law:

```text
zero-arity product
    canonical proper type = Unit

positive-arity tuple product
    canonical structural Tuple type
```

At the language level it is still coherent to call `()` the zero-arity tuple/product syntax.

The structural operations should have explicit zero-shape behavior:

```phalcom
()::Positionals
    == []

()::Labeled
    == []

()::Map<Option>
    == ()

()[0]
    // statically invalid
```

This must **not** require the runtime class of `()` to be `Tuple`. The language specification should distinguish:

- zero-product structural semantics;
- runtime behavioral class identity.

This is especially important if Tuple type members are surfaced through a Universe `Tuple` declaration for source discoverability. Semantic applicability must not be defined solely by runtime `.class`.

---

# 8. Type Member Model

C1 should introduce a general semantic category named **type member**.

## 8.1 Required type-member categories

At minimum:

```text
TypeMember
    InherentTypeMember
    AssociatedTypeMember
    DomainNativeTypeMember
```

The exact implementation representation is deferred, but the language distinctions are important.

### Inherent type member

Owned by a nominal semantic declaration.

Example future form:

```phalcom
class Box<T> {
    type Element = T
}
```

### Associated type member

Owned by a trait and supplied/bound by conformance evidence.

Existing example:

```phalcom
trait Iterator {
    type Item
}
```

### Domain-native type member

Owned by a compiler-defined structural semantic domain rather than by an arbitrary anonymous value type declaration.

Examples:

```text
Tuple-family :: Positionals
Tuple-family :: Labeled
Tuple-family :: At<I>
Tuple-family :: Map<F>

Record-family :: Map<F>

TypePack-domain :: Map<F>
RecordRow-domain :: Map<F>
```

A Universe declaration may provide source ownership, documentation, reflection anchor, or intrinsic identity, but it must not turn anonymous structural Tuple types into ordinary nominal inherent-`impl` targets.

## 8.2 Parameterized type members

The grammar must support type-member-local generic parameters:

```phalcom
type At<I: Int>: Type
type Map<F: Type -> Type>: Type
```

or the final equivalent syntax.

These parameters belong to the member application:

```phalcom
T::At<1>
T::Map<Option>
```

They are not hidden generic parameters of the subject type.

## 8.3 Type-member result domains

A type member must declare a result domain, not merely “returns a proper type.”

Examples:

```text
Positionals : TypePack
Labeled     : LabeledTypePack
At<I>       : Type
Map<F>      : Type
```

Future type members may return `RecordRow` or another kinded domain.

This is why the projection model must be domain-neutral at the semantic specification level.

---

# 9. Projection Semantics

## 9.1 Projection identity

A projection should be identified by:

```text
subject
member declaration identity
member arguments
relevant qualification/evidence context where required
```

not by written member name alone.

Conceptually:

```text
Projection(
    subject,
    member,
    arguments
)
```

The result domain is determined by the member declaration.

## 9.2 Symbolic projection

If a valid projection cannot yet reduce because an input is abstract, the result remains symbolic.

Examples:

```phalcom
T::At<I>
P::Map<F>
R::Map<F>
```

must not become:

```text
Dynamic
Unknown
Never
```

merely because they are not currently reducible.

C1 should distinguish:

```text
valid but stuck
concretely invalid
reduced
```

These are different semantic states.

## 9.3 Invalid projection

Examples of concrete invalidity include:

```phalcom
(Int, String)::At<10>
(Int, String)::Map<Int>
```

The first has a concrete out-of-range index. The second supplies a `Type` where `Type -> Type` is required.

These should be static errors, not “stuck” projections.

## 9.4 Normalization boundary

C1 should specify normalization semantically, but not prescribe a Rust evaluator.

Required semantic outcomes:

```text
Reduced
Stuck
Invalid
```

Implementation-level states such as cancellation, budget exhaustion, and cycle detection should be planned later, but C1 should require that they do not silently change a well-formed symbolic type into a different language type.

---

# 10. `At<I>` and Type-Level Subscript

## 10.1 One operation, two spellings

C1 should ratify:

```text
T[I] ≡ T::At<I>
```

where `≡` means same semantic type-member projection identity.

This is stronger than saying the two expressions “often reduce to equal types.”

The compiler should not model them as two separately declared type members.

## 10.2 Index domain

The first LANG007 type-level Tuple subscript should use:

```text
I :const Int
```

C1 should not implicitly enable every runtime Tuple key category.

Specifically, runtime Tuple currently accepts:

- integer indexes;
- Symbols;
- Ranges.

LANG007.C1 should specify only integer type indexing unless Symbol/Range type operations are separately designed.

## 10.3 Negative indexing

Recommended rule:

```text
normalize(i, n) =
    i            if i >= 0
    n + i        if i < 0
```

then require:

```text
0 <= normalize(i, n) < n
```

This aligns compile-time Tuple indexing with existing runtime integer indexing.

Examples:

```phalcom
(Int, String)::At<0>   == Int
(Int, String)::At<1>   == String
(Int, String)::At<-1>  == String
(Int, String)::At<-2>  == Int
```

and:

```phalcom
(Int, String)::At<2>   // invalid
(Int, String)::At<-3>  // invalid
```

## 10.4 Tuple linear index order

For:

```phalcom
(Int, String, name: Symbol, flag: Bool)
```

the linear index sequence should be:

```text
0 -> Int
1 -> String
2 -> Symbol
3 -> Bool
```

The labeled lane follows the positional lane in declared order.

This mirrors Tuple’s structural value semantics.

---

# 11. `Map<F>` Semantics

`Map` should be specified as a **shape-preserving structural transformation**.

## 11.1 Constructor contract

`F` must satisfy:

```text
F : Type -> Type
```

Thus:

```phalcom
(Int, String)::Map<Option>
```

is valid because:

```text
Option : Type -> Type
```

while:

```phalcom
(Int, String)::Map<Int>
```

is invalid because:

```text
Int : Type
```

## 11.2 TypePack mapping

Define:

```text
MapPack([T₀, T₁, ..., Tₙ], F)
    =
[F<T₀>, F<T₁>, ..., F<Tₙ>]
```

Result domain:

```text
TypePack
```

Order is preserved.

## 11.3 LabeledTypePack mapping

Define:

```text
MapLabeledPack(
    [(l₀, T₀), ..., (lₙ, Tₙ)],
    F
)
=
[(l₀, F<T₀>), ..., (lₙ, F<Tₙ>)]
```

Labels and label order are preserved.

## 11.4 Tuple mapping

For:

```text
Tuple(P, L)
```

define:

```text
Tuple(P, L)::Map<F>
    =
Tuple(
    MapPack(P, F),
    MapLabeledPack(L, F)
)
```

Examples:

```phalcom
(Int, String)::Map<Option>
    ==
(Option<Int>, Option<String>)
```

and:

```phalcom
(Int, name: String)::Map<Option>
    ==
(Option<Int>, name: Option<String>)
```

## 11.5 RecordRow mapping

For a closed row:

```text
MapRow(
    { l₀: T₀, ..., lₙ: Tₙ },
    F
)
=
{ l₀: F<T₀>, ..., lₙ: F<Tₙ> }
```

Result domain:

```text
RecordRow
```

The key set is preserved.

## 11.6 Open RecordRow mapping

For an open row:

```text
{ l₀: T₀, ..., lₙ: Tₙ | R }
```

mapping must include the unknown tail:

```text
MapRow(
    { l₀: T₀, ..., lₙ: Tₙ | R },
    F
)
=
{
    l₀: F<T₀>,
    ...,
    lₙ: F<Tₙ>
    |
    MapRow(R, F)
}
```

The following is **incorrect**:

```text
{
    l₀: F<T₀>,
    ...,
    lₙ: F<Tₙ>
    |
    R
}
```

because it leaves the unknown fields unmapped.

This requirement is one of the strongest reasons C1 must define symbolic row transformations even though the concrete representation is deferred to C2.

## 11.7 Record mapping

For:

```text
Record(R)
```

define:

```text
Record(R)::Map<F>
    =
Record(MapRow(R, F))
```

Example:

```phalcom
#{name: String, age: Int}::Map<Option>
    ==
#{name: Option<String>, age: Option<Int>}
```

## 11.8 Required `Map` laws

C1 should ratify at least:

### Shape preservation

```text
shape(S::Map<F>) = shape(S)
```

where “shape” is interpreted according to the subject domain.

### Tuple labels

```text
labels(T::Map<F>) = labels(T)
```

### Tuple indexing commutation

For valid `I`:

```text
(T::Map<F>)::At<I>
    ≡
F<T::At<I>>
```

### Record lookup commutation

For a present key `K`:

```text
Lookup(R::Map<F>, K)
    ≡
F<Lookup(R, K)>
```

### Empty product

```text
()::Map<F> ≡ ()
```

### Domain preservation

```text
Map(TypePack, F)        : TypePack
Map(LabeledTypePack, F) : LabeledTypePack
Map(RecordRow, F)       : RecordRow
```

C1 should **not** require map fusion:

```text
Map(Map(S, F), G) == Map(S, G ∘ F)
```

because introducing type-constructor composition is unnecessary for the initial feature.

---

# 12. Closed Type-Level Calculus Boundary

A critical C1 responsibility is to define what LANG007 does **not** become.

The first release should support a small closed set of compiler-known structural operations.

Conceptually:

```text
stable type-level structural terms
    explicit structure
    parameter
    projection
    Map
    pack expansion/concatenation where separately specified
```

This does **not** imply:

- arbitrary user-defined type functions;
- arbitrary recursive type equations;
- open rewrite rule sets;
- Haskell-style open type families;
- compile-time execution of arbitrary Phalcom methods;
- type-level reflection over arbitrary runtime values;
- unrestricted const arithmetic;
- a general theorem prover.

The reason is architectural, not merely implementation convenience. A closed evaluator gives LANG007:

- deterministic normalization;
- clear stuck versus invalid states;
- bounded diagnostics;
- predictable caching;
- tractable termination;
- straightforward metadata representation;
- limited coherence surface.

C1 should say explicitly that later extensions may add new closed operations without implying a general-purpose type-level programming language.

---

# 13. Union Semantics

C1 must decide how structural type members interact with unions.

Recommended first-release rule:

> Structural intrinsic projection reduces only when the subject is an exact normalized structural shape for which the operation is defined. LANG007.C1 does not implicitly distribute Tuple/Record structural type members over unions.

Therefore:

```phalcom
((Int, String) | (Bool, Symbol))::At<0>
```

should remain unsupported/stuck according to ordinary member applicability rules rather than automatically becoming:

```phalcom
Int | Bool
```

unless a later specification explicitly introduces distributive projection.

This avoids several hidden obligations:

- union-of-packs semantics;
- union-of-record-row transformations;
- per-branch failure rules;
- normalization explosion;
- interaction with structural subtyping.

C1 should record this as a deliberate non-goal rather than leaving it accidental.

---

# 14. Type-Member Lookup and Name Collisions

Generalizing `::` beyond trait-associated types creates a lookup question C1 must resolve.

Potential candidates include:

```text
subject-owned inherent type member
domain-native structural type member
trait-associated type member available through evidence
```

C1 should avoid a magic precedence rule based solely on spelling.

Recommended rule:

1. Every type member has declaration identity.
2. `Subject::Name` is accepted only when the semantic context resolves `Name` to one accessible candidate under the applicable lookup rules.
3. Trait-associated members remain trait-owned.
4. When multiple trait contexts can supply the same name, explicit qualification is required:

```phalcom
<T as Trait>::Item
```

5. Domain-native members such as Tuple `Map` are available only when the subject domain is statically known to support them.
6. A generic proper-type parameter:

```phalcom
T: Type
```

does **not** automatically have `Map`.
7. A pack parameter:

```phalcom
P: TypePack
```

may have the domain operation `Map` because its semantic domain is known.

This preserves static coherence and avoids “duck typed” type-member lookup.

---

# 15. Native Type Members

## 15.1 Required semantic generalization

The source declaration model should define `native` category-neutrally:

> A `native` declaration has an implementation supplied by the host Phalcom implementation rather than by a Phalcom source body, using the implementation mechanism appropriate to that declaration category.

Then specialize:

```text
native executable behavior
    native runtime/VM implementation

native type member
    compiler/semantic normalization implementation
```

## 15.2 `native` versus `@intrinsic`

The distinction must remain:

```text
native
    implementation origin

@intrinsic
    builtin metadata/identity annotation
```

A type member can be native because its evaluator is implemented by the compiler.

An intrinsic identity may be used to associate that declaration with a registered evaluator, but the attribute itself must not be described as arbitrary executable semantics.

## 15.3 No VM dispatch

A native type member:

```phalcom
native type Map<F: Type -> Type>: Type
```

must never normalize by:

- materializing runtime Tuple values;
- sending runtime messages;
- starting VM frames;
- invoking ordinary native call dispatch.

Its evaluation belongs to the semantic/type normalizer.

This is a C1 semantic boundary even though the evaluator is implemented later.

---

# 16. Recommended New Authoritative Specifications

C1 should create three new semantic owners.

---

## 16.1 `docs/specs/type-level-domains.md`

### Purpose

Own the language’s type-level domain/kind model.

### Must define

- `Type`;
- `RecordRow`;
- `TypePack`;
- `LabeledTypePack`;
- arrow kinds;
- kind application;
- proper types;
- type constructors/type forms;
- generic parameter domains;
- typed compile-time value parameter domains;
- distinction between kinded forms and const/value-domain parameters;
- domain well-formedness;
- whether compile-time constants are part of the same type-level expression grammar or a subordinate grammar;
- the semantic category of `Option : Type -> Type`;
- the semantic category of `0 :const Int`.

### Must not define

- solver algorithms;
- interning IDs;
- Rust arenas;
- metadata encoding;
- Tuple-specific `Map`;
- trait conformance.

### Suggested core rules

```text
Γ ⊢ Int : Type
Γ ⊢ Option : Type -> Type
Γ ⊢ P : TypePack
Γ ⊢ R : RecordRow
Γ ⊢ 0 :const Int
```

and:

```text
GenericParameterDomain ::= Kind(κ) | Const(τ)
```

This new spec fills a current ownership gap: `generic-constraints.md` refers to a “kind specification,” but the current authoritative index does not identify a dedicated owner for kinds/type-level domains.

---

## 16.2 `docs/specs/type-members.md`

### Purpose

Own declaration, identity, lookup, application, result domains, and projection semantics for type members generally.

### Must define

- type-member category;
- inherent type members;
- trait-associated type members as a specialization;
- compiler/domain-native type members;
- member parameters;
- member parameter domains;
- member result domains;
- projection identity;
- symbolic projections;
- stuck versus invalid;
- qualification;
- collision/ambiguity rules;
- native/alias/requirement implementation categories at the language-semantic level;
- type-subscript member identity and named aliasing if retained.

### Must not define

- trait conformance evidence itself;
- record-row layout;
- pack inference;
- evaluator implementation;
- runtime dispatch.

### Suggested semantic taxonomy

```text
TypeMember
    Inherent
    Associated
    DomainNative
```

with implementation forms:

```text
Requirement
Alias
Native
```

These axes should remain orthogonal.

---

## 16.3 `docs/specs/structural-products.md`

### Purpose

Own static structural Tuple/Record typing and structural type operations.

### Must define

- Tuple structural type formation;
- positional/labeled lane ordering;
- label uniqueness;
- label-order identity;
- Record structural type formation;
- Record key-set identity;
- row openness;
- zero-product/`Unit` relationship;
- `Positionals`;
- `Labeled`;
- `At<I>`;
- type subscript equivalence;
- negative indexes;
- concrete out-of-range behavior;
- `Map<F>`;
- mapping over open rows;
- mapping over abstract packs/rows at the semantic level;
- no union distribution in the first release;
- relation to runtime Tuple/Record values without conflating runtime and static categories.

### Must not define

- VM product layout;
- `ProductLayoutId`;
- runtime storage;
- GC;
- native descriptor wiring.

### Why this should be a new spec

`objects/values.md` explicitly excludes structural Tuple/Record typing, and `vm/products.md` explicitly says it does not replace source-language record/tuple typing. A new authoritative language owner is therefore required rather than optional cleanup.

---

# 17. Existing Specifications That Should Be Amended

The following table is the recommended C1 specification-change matrix.

| Specification | Required? | Amendment class | Reason |
|---|---:|---|---|
| `docs/specs/README.md` | **Yes** | ownership/index | Add new semantic owners and update cross-spec map. |
| `docs/specs/generic-constraints.md` | **Yes** | normative semantic change | `:` currently means only kind classification; generic argument grammar assumes ordinary type arguments in several places. |
| `docs/specs/associated-types.md` | **Yes** | ownership/refactor | Currently excludes generic/higher-kinded associated declarations and restricts associated values to proper types. |
| `docs/specs/source-declarations.md` | **Yes** | normative semantic generalization | `native` currently speaks specifically of native runtime implementation. |
| `docs/specs/objects/impl.md` | **Yes** | clarification/boundary | Preserve anonymous structural target prohibition while acknowledging compiler/domain type members. |
| `docs/specs/objects/traits.md` | **Yes, narrow** | delegation/consistency | Trait type requirements should delegate general type-member parameter/result-domain rules to `type-members.md`. |
| `docs/specs/universe.md` | **Yes, narrow** | platform integration | Clarify source-owned native type members and compiler/semantic native implementation. |
| `docs/specs/objects/values.md` | Recommended | cross-reference only | Point structural typing to `structural-products.md`; do not move rules into values spec. |
| `docs/specs/callables/arguments.md` | Review; likely no semantic change | terminology/cross-reference only | Ensure Tuple lanes and Unit complete-product terminology remain consistent. |
| `docs/specs/callables/selectors.md` | **No**, unless grammar shares selector machinery | no semantic ownership | Type-member selectors should not be conflated with callable selector identity. |
| `docs/specs/vm/products.md` | Review; no C1 language change | consistency witness | Existing ordered Tuple/unordered Record rules should remain; add cross-link only if useful. |
| `docs/specs/vm/values-and-objects.md` | Review; no C1 language change | consistency witness | Existing zero-arity Tuple -> Unit rule should remain. |
| metadata/LSP specs | **No in C1** | deferred | Owned by later LANG007 checkpoints. |

---

# 18. Detailed Required Amendments

## 18.1 `docs/specs/README.md`

The specification index must add the new owners.

Recommended ownership entries:

```markdown
| Type-level kinds/domains, type constructors, packs, compile-time parameter domains | [`type-level-domains.md`](type-level-domains.md) |
| Type-member declarations, identity, application, projection, and normalization boundary | [`type-members.md`](type-members.md) |
| Structural Tuple/Record typing and structural type operations | [`structural-products.md`](structural-products.md) |
```

The existing associated-types row should be narrowed to:

```text
trait-owned associated type requirements,
conformance bindings,
trait-qualified projection specialization
```

rather than owning all type-member semantics.

The “one concept, one owner” examples should be updated to demonstrate:

```text
type-member projection identity
    type-members.md

trait-associated binding/evidence
    associated-types.md

generic constraint use of a projection
    generic-constraints.md
```

This is important because otherwise `type-members.md` and `associated-types.md` can become overlapping authorities.

---

## 18.2 `docs/specs/generic-constraints.md`

This file requires one of the most substantial C1 amendments.

### Existing conflict

Current:

```text
: = kind classification
```

Required LANG007 syntax:

```phalcom
I: Int
```

where `Int` is not a kind.

### Recommended amendment

Generalize the relevant concept from **kind classification** to **generic parameter-domain classification**, while preserving kind classification as one case.

For example:

```text
ParameterDomain
    Kind
    ConstType
```

Then:

```phalcom
F: Type -> Type
```

classifies `F` into a kind domain.

```phalcom
P: TypePack
```

classifies `P` into a kind domain.

```phalcom
I: Int
```

classifies `I` into a compile-time value domain.

### Important syntax rule

This must **not** revive trait-bound shorthand such as:

```phalcom
T: Printable
```

if Phalcom continues to require:

```phalcom
where T |= Printable
```

The elaborator must distinguish:

- a recognized kind expression;
- a proper type used as a const parameter domain;
- a trait reference, which remains invalid after `:` for this purpose.

### `where` versus binder annotations

C1 should decide whether const domains are legal only on binders:

```phalcom
<I: Int>
```

or can also be restated in `where`.

Recommended initial rule:

- binder annotations establish parameter domains;
- `where` continues to express propositions over bound parameters;
- a `where I: Int` restatement is unnecessary and should not be introduced unless there is a concrete need.

This prevents the constraint language from mixing declarations of parameter category with propositions unnecessarily.

### Generic argument grammar

The spec currently uses type-centric names such as:

```text
OrdinaryTraitArguments := TypeExpression ...
```

C1 should determine whether LANG007 makes heterogeneous ordinary generic arguments globally legal in C1’s semantic model or only type-member arguments in the first release.

Recommended scope:

> The semantic model is heterogeneous, but LANG007 initially requires heterogeneous argument domains for type-member applications. General class/trait/type-alias const and pack arguments are a separately gated source-surface extension unless C1 explicitly includes them.

This avoids forcing `TraitRef.arguments` and all nominal applications to migrate merely to support `Tuple::At<1>`.

---

## 18.3 `docs/specs/associated-types.md`

This document should be refactored, not replaced.

### Preserve

- trait-owned associated declaration identity;
- conformance-owned binding;
- evidence-based normalization;
- contextual `Self::Item`;
- qualified `<T as Trait>::Item`;
- declaration identity not name identity;
- recursive/sibling binding semantics where already defined.

### Delegate to `type-members.md`

- general type-member parameters;
- result domains;
- generic type-member application;
- general projection identity substrate;
- symbolic projection category;
- member argument checking.

### Remove or narrow explicit exclusions

Current exclusions include:

```text
generic associated types
higher-kinded associated declarations
```

After C1, the spec should instead say which parameter/result-domain forms LANG007 actually supports.

If the first release supports parameterized associated members but only result `Type`, say that explicitly.

If it supports result domains such as `TypePack` for trait-associated members too, say that explicitly.

Do not leave an absolute exclusion that conflicts with the general type-member model.

### Proper-type-only statement

The section “Associated declarations denote proper types” must be narrowed.

Possible revised rule:

> An associated declaration’s result domain is part of its declaration contract. Existing unannotated associated types default to result domain `Type`.

This preserves source compatibility:

```phalcom
type Item
```

still means:

```text
Item : Type
```

while allowing future explicit domains.

---

## 18.4 `docs/specs/source-declarations.md`

### Existing rule

```text
native = live implementation supplied by native runtime
```

### Required generalization

Recommended wording:

> `native` identifies a declaration whose implementation is supplied by the host Phalcom implementation rather than by a Phalcom source body. The concrete host subsystem depends on declaration category.

Then explicitly distinguish:

```text
callable native member
    runtime/VM/native callable binding

type native member
    compiler/semantic type evaluator
```

### Keep unchanged

- `native` does not grant visibility;
- `native` does not itself imply `@intrinsic`;
- `@intrinsic` is metadata;
- Universe-only authored intrinsic metadata if that remains the rule;
- source/native descriptor agreement requirements as applicable.

### Clarify bodylessness

C1 should state whether:

```phalcom
native type Map<F: Type -> Type>: Type
```

is bodyless because `native` supplies the implementation.

By contrast:

```phalcom
trait X {
    type Item
}
```

is bodyless because it is a requirement, **not** because it is native.

This distinction should be explicit.

---

## 18.5 `docs/specs/objects/impl.md`

The current prohibition on anonymous structural inherent targets should remain.

Add a clarification:

> Compiler/domain-defined structural type members do not constitute ordinary inherent implementations of each anonymous Tuple or Record shape.

Therefore:

```phalcom
(Int, String)::Map<Option>
```

does not imply that source code may write:

```phalcom
impl (Int, String) {
    ...
}
```

or:

```phalcom
impl #{name: String} {
    ...
}
```

unless a future language revision separately authorizes structural inherent `impl`s.

This protects coherence and keeps LANG007 independent from the inherent-implementation ownership model.

---

## 18.6 `docs/specs/objects/traits.md`

This should receive a narrow delegation amendment.

If the trait grammar permits:

```phalcom
trait Indexed {
    type At<I: Int>: Type
}
```

then trait specification should say:

- trait-owned type members use the general declaration/result-domain/parameter rules from `type-members.md`;
- conformance completeness and witness/binding rules remain owned by `traits.md`/`associated-types.md`.

It should not duplicate the type-member calculus.

---

## 18.7 `docs/specs/universe.md`

Universe should state that source-owned native declarations can include compiler/semantic type members.

A native structural type operation should still have one canonical source/declaration identity if the project chooses a Universe declaration as the anchor.

The native registry must not become a competing declaration database.

This follows Universe’s existing principle:

```text
native metadata enriches canonical declarations;
it does not create competing semantic ownership
```

C1 should extend that principle to native type operations.

---

## 18.8 `docs/specs/objects/values.md`

Only a cross-reference is required.

Its existing exclusion:

```text
structural Tuple/Record typing
```

is correct.

Add the new owner name, for example:

```text
Structural Tuple/Record typing is defined by structural-products.md.
```

No static typing rules should be moved into `values.md`.

---

## 18.9 VM specifications

C1 should not rewrite VM semantics unless the new language contract reveals an actual contradiction.

Required audit only:

### `vm/products.md`

Verify:

- Tuple label order significant;
- Record key order not identity;
- Unit zero product;
- structural Tuple/Record categories remain distinct.

### `vm/values-and-objects.md`

Verify:

```text
zero-arity tuple -> immediate Unit
```

remains consistent with the language-level zero-product rule.

If necessary, add links to the new language spec, but no new type-evaluation machinery belongs in VM specs.

---

# 19. Source Surface Decisions C1 Must Ratify

C1 must not leave syntax only half-specified.

## 19.1 Recommended forms

```phalcom
native type Positionals: TypePack

native type Labeled: LabeledTypePack

native type At<I: Int>: Type

native type Map<F: Type -> Type>: Type
```

If the project prefers subscript declaration syntax:

```phalcom
native type [I: Int]: Type
```

C1 must state how the named form `At<I>` is bound to the same identity.

Do **not** declare:

```phalcom
native type [I: Int]: Type
native type At<I: Int>: Type
```

as two semantically independent declarations.

## 19.2 Recommended canonical identity model for subscript

The cleanest model is:

```text
one TypeMemberId
    presentation aliases:
        subscript syntax [I]
        named syntax At<I>
```

or the semantic equivalent.

C1 need not choose the Rust ID structure, but it must require one identity.

## 19.3 `Map` availability

C1 should distinguish:

```text
Tuple structural subject
    Map available

Record structural subject
    Map available

TypePack subject
    Map available

LabeledTypePack subject
    Map available if exposed

RecordRow subject
    Map available

arbitrary T : Type
    Map not automatically available
```

This prevents accidental structural duck typing.

---

# 20. Specification of Compile-Time Constants

C1 only needs a restricted const-term model sufficient for member arguments such as `At<1>`.

Required initial forms:

```text
integer literal
compile-time parameter reference
possibly unary negation of integer literal if syntax lowers -1 structurally
```

C1 should not require:

- arbitrary arithmetic;
- arbitrary method calls;
- runtime variables;
- mutation;
- effects;
- const evaluation of general Phalcom blocks;
- const inference variables.

The semantic requirement is:

```text
member argument must be provably compile-time
and must have the declared value type
```

For `At<I: Int>`:

```text
1       valid
-1      valid
I       valid if I is a compile-time Int parameter
runtimeIndex invalid
```

If negative integer syntax is represented as unary negation rather than a signed literal in the AST, that is a C2 parser/elaboration detail. C1 only owns the resulting compile-time integer value semantics.

---

# 21. Specification of Symbolic Structural Terms

Although C2 owns representation, C1 must require stable symbolic semantics.

The following must be representable **semantically**:

```text
P                           P : TypePack
R                           R : RecordRow

MapPack(P, F)
MapRow(R, F)

T::At<I>
T::Map<F>
```

because generic code can contain valid type expressions before all inputs are concrete.

C1 must reject any semantic model that only defines:

```text
TypePack = concrete array of types
RecordRow = concrete field map + bare parameter tail
```

as the complete language meaning.

Those may be optimized concrete representations, but the language requires symbolic operations over unknown structures.

This requirement is critical for later architecture.

---

# 22. Tuple and Record Mapping Must Share Algebra, Not Identity

The correct abstraction is:

```text
shape domain
    payload positions
    identity/ordering law

Map<F>
    preserve shape domain
    transform payload types
```

For each domain:

| Domain | Payload | Labels | Ordering identity | `Map` result |
|---|---|---|---|---|
| `TypePack` | proper types | none | ordered | `TypePack` |
| `LabeledTypePack` | proper types | labels | ordered | `LabeledTypePack` |
| `RecordRow` | proper types | keys | unordered/key-set | `RecordRow` |
| Tuple proper type | pack + labeled pack | tuple labels | ordered | Tuple proper type |
| Record proper type | row | record keys | unordered/key-set | Record proper type |

This table should effectively become a normative part of the structural-products specification.

The implementation may share a generic sequence mapper for the two pack domains, but Record rows need row semantics.

---

# 23. Required Invariants

C1 should define durable coded invariants that later plans can cite.

Recommended names follow the repository’s existing specification style.

## `TL-DOMAIN-SEPARATION`

> `Type`, `RecordRow`, `TypePack`, and `LabeledTypePack` are distinct type-level domains. A value from one domain is not implicitly reinterpreted as another.

## `TL-CONST-NOT-KIND`

> A typed compile-time value parameter such as `I: Int` is classified by the proper value type `Int`; `Int` does not thereby become a kind.

## `TL-PROPER-TYPE`

> A proper runtime value type has kind `Type`. Higher-kinded type forms such as `Option : Type -> Type` are type-level forms but are not proper runtime value types until saturated.

## `TM-DECLARATION-IDENTITY`

> Type-member identity is declaration/domain-member identity, not spelling identity.

## `TM-PROJECTION-SYMBOLIC`

> A well-formed type-member projection that lacks enough information to reduce remains symbolic rather than being approximated as `Dynamic`, `Never`, or an unrelated type.

## `TM-INVALID-DISTINCT-FROM-STUCK`

> Concrete invalidity and lack of reduction information are distinct semantic outcomes.

## `STRUCT-TUPLE-LANE-ORDER`

> A Tuple shape consists of an ordered positional prefix followed by an ordered labeled suffix.

## `STRUCT-TUPLE-LABEL-ORDER`

> The ordered sequence of Tuple labels participates in Tuple structural identity.

## `STRUCT-RECORD-KEY-IDENTITY`

> Record structural identity is determined by key/type association, not source or presentation order.

## `STRUCT-PACK-ROW-DISTINCT`

> An ordered labeled type pack is not a record row even when both contain the same label/type pairs.

## `STRUCT-ZERO-PRODUCT`

> `()` denotes the zero-arity structural product and canonicalizes to `Unit`; positive-arity Tuple shapes remain Tuple structural types.

## `STRUCT-AT-IDENTITY`

> Type subscript `T[I]` and named projection `T::At<I>` denote one semantic type-member operation.

## `STRUCT-AT-BOUNDS`

> A fully concrete out-of-range Tuple type index is statically invalid.

## `STRUCT-MAP-SHAPE`

> `Map<F>` preserves the subject’s structural shape domain and applies `F` to each payload proper type.

## `STRUCT-MAP-OPEN`

> Mapping an open structural domain transforms the unknown remainder symbolically; it does not leave that remainder unmapped.

## `STRUCT-MAP-KIND`

> `Map<F>` requires `F : Type -> Type`.

## `NATIVE-TYPE-NO-VM`

> Native type-member normalization is a compiler/semantic operation and does not execute runtime VM dispatch.

## `LANG007-CLOSED-CALCULUS`

> The existence of compiler-native structural type operations does not introduce arbitrary user-defined type families or arbitrary compile-time Phalcom execution.

---

# 24. C1 Acceptance Examples

The specification corpus should be able to classify every example below unambiguously.

## 24.1 Kind/domain examples

```text
Int
    : Type

Option
    : Type -> Type

P
    : TypePack

R
    : RecordRow

0
    :const Int
```

## 24.2 Tuple projection examples

```phalcom
(Int, String)::Positionals
// [Int, String] : TypePack
```

```phalcom
(Int, name: String)::Labeled
// [(name, String)] : LabeledTypePack
```

## 24.3 Index examples

```phalcom
(Int, String)[0]
// Int
```

```phalcom
(Int, String)::At<1>
// String
```

```phalcom
(Int, String)::At<-1>
// String
```

```phalcom
(Int, name: String)[1]
// String
```

```phalcom
(Int, String)::At<2>
// static error
```

## 24.4 Tuple map examples

```phalcom
(Int, String)::Map<Option>
// (Option<Int>, Option<String>)
```

```phalcom
(Int, name: String)::Map<Option>
// (Option<Int>, name: Option<String>)
```

```phalcom
()::Map<Option>
// ()
```

## 24.5 Record map examples

```phalcom
#{name: String, age: Int}::Map<Option>
// #{name: Option<String>, age: Option<Int>}
```

## 24.6 Open symbolic examples

```phalcom
R::Map<Option>
// valid symbolic RecordRow transformation when R : RecordRow
```

```phalcom
P::Map<Option>
// valid symbolic TypePack transformation when P : TypePack
```

```phalcom
T::At<I>
// valid symbolic projection if T's structural domain/member applicability and I's const Int domain are established
```

## 24.7 Invalid kind/domain examples

```phalcom
(Int, String)::Map<Int>
// invalid: Int : Type, expected Type -> Type
```

```phalcom
T::Map<Option>
// invalid/not available when T is only known as T : Type
```

```phalcom
(Int, String)::At<runtimeIndex>
// invalid: argument is not a compile-time Int term
```

## 24.8 Deferred union example

```phalcom
((Int, String) | (Bool, Symbol))::At<0>
```

C1 must explicitly classify this according to the no-implicit-distribution rule rather than leaving it evaluator-defined.

---

# 25. Required Specification Tests

C1 is primarily a specification checkpoint, but it still needs verification.

The minimum verification suite should include a **spec-conformance matrix**, not merely markdown lint.

Each invariant should be traced to:

1. one owning specification section;
2. all dependent specification sections;
3. one or more positive examples;
4. one or more negative/counterexamples;
5. the later LANG007 checkpoint responsible for implementation.

Example matrix:

| Invariant | Owner | Dependent specs | Positive case | Negative case | Implementation checkpoint |
|---|---|---|---|---|---|
| `TL-CONST-NOT-KIND` | `type-level-domains.md` | `generic-constraints.md` | `I: Int` | treating `Int` as `KindId` | C2/C3 |
| `STRUCT-AT-IDENTITY` | `structural-products.md` | `type-members.md` | `T[I] ≡ T::At<I>` | two separate declaration IDs | C3/C4 |
| `STRUCT-MAP-OPEN` | `structural-products.md` | `type-level-domains.md` | map open row tail | unchanged open tail | C2/C4 |
| `NATIVE-TYPE-NO-VM` | `source-declarations.md` + `type-members.md` | `universe.md` | native semantic evaluator | VM frame execution | C3/C4 |

C1 verification should also include repository searches for stale absolute claims, especially:

```text
":" always means kind classification
associated types can only denote proper Type
generic associated types are unsupported
higher-kinded associated declarations are unsupported
native always means runtime implementation
:: always means trait-associated projection
```

Any retained statement must be contextually narrowed so it no longer contradicts LANG007.

---

# 26. C1 Non-Goals and Deferred Implementation

C1 should explicitly defer the following.

## To `LANG007.C2`

- new Rust `KindData` variants;
- `GenericParameterDomain` representation;
- `GenericArgument` representation;
- stable const-term arena;
- stable pack-term arenas;
- stable row-transform terms;
- interning;
- materialization changes.

## To `LANG007.C3`

- parser/AST representation of parameterized type members;
- semantic declaration IDs;
- projection arena;
- type-member lookup implementation;
- native type intrinsic registry;
- normalization engine;
- actual `T[I]` lowering.

## To `LANG007.C4`

- Tuple `Positionals`;
- Tuple `Labeled`;
- Tuple `At`;
- Tuple `Map`;
- Record `Map`;
- concrete reduction algorithms;
- structural Tuple formation hardening.

## To `LANG007.C5`

- pack inference;
- pack expansion;
- ambiguous split diagnostics;
- broad generic integration;
- generic application/session changes;
- any wider const-generic surface.

## To `LANG007.C6`

- metadata schema;
- reflection;
- LSP hovers/completion;
- incremental invalidation/fingerprints.

## To `LANG007.C7`

- migration cleanup;
- corpus convergence;
- performance certification;
- release certification.

C1 should not contain implementation pseudocode so detailed that it becomes a competing C2 plan. Representation sketches are acceptable only where needed to prove semantic feasibility.

---

# 27. Scope Decision: Type-Member-Only Heterogeneous Arguments vs Language-Wide Const/Packs

This is one of the few architectural scope questions C1 must settle.

Two viable strategies exist.

## Option A — LANG007 initially scopes heterogeneous arguments to type members

Required now:

```phalcom
T::At<1>
T::Map<Option>
```

Potentially later:

```phalcom
class Matrix<T, N: Int>
```

### Advantages

- avoids immediate migration of all nominal `Applied` type arguments;
- avoids immediate migration of `TraitRef.arguments`;
- keeps C2/C3 bounded;
- still establishes the general semantic domain model;
- does not block later language-wide const generics.

### Disadvantage

Some internal abstractions are temporarily more specialized.

## Option B — LANG007 immediately makes all generic argument lists heterogeneous

Examples:

```phalcom
class Matrix<T, N: Int>
trait Sized<N: Int>
type Vector<T, N: Int> = ...
```

### Advantages

- one general application model from day one;
- no later migration of generic argument containers.

### Disadvantages

- much larger blast radius;
- significantly increases C2/C5 scope;
- forces trait references, applied nominal types, metadata, diagnostics, inference, and reflection to migrate immediately;
- risks turning LANG007 into a general const-generics program.

## Recommendation

C1 should ratify:

> **The semantic parameter-domain model is general, but LANG007’s required source-surface use of heterogeneous non-Type arguments is initially type-member application and structural type computation. General nominal/trait const generics are outside the minimum C1 contract unless separately promoted into scope.**

This preserves architectural correctness without forcing unrelated migration.

---

# 28. Scope Decision: Public Domain Operations on Packs/Rows

C1 should distinguish semantic availability from source exposure.

Internally, the calculus needs:

```text
MapPack(P, F)
MapLabeledPack(L, F)
MapRow(R, F)
```

Whether source code immediately supports:

```phalcom
P::Map<Option>
R::Map<Option>
```

is a surface decision.

Because the user-facing LANG007 direction explicitly wants `Ts::Map<Option>` “for both records and tuples,” the recommended C1 rule is:

- expose `Map` on the structural domains when the subject domain is statically known;
- do not require packs/rows to be runtime classes;
- `::` is type-level member/domain-operation syntax, not proof of runtime objecthood.

This gives a coherent explanation of:

```phalcom
P::Map<Option>
```

without inventing a runtime `TypePack` object model.

---

# 29. Risks and Failure Modes

## 29.1 Modeling packs as proper Tuple types

Failure:

```text
TypePack == Tuple<Type,...>
```

Consequence:

- confuses sequence-of-type-forms with proper runtime type;
- makes variadic substitution awkward;
- breaks higher-kinded result domains;
- encourages nominal tuple representation.

Requirement:

```text
TypePack is a distinct type-level domain.
```

## 29.2 Reusing `RecordRow` for Tuple labeled entries

Failure:

```text
Tuple labeled lane == RecordRow
```

Consequence:

- loses label order;
- can make distinct Tuple shapes canonicalize together;
- contradicts runtime Tuple equality and VM product semantics.

Requirement:

```text
LabeledTypePack is distinct and ordered.
```

## 29.3 Treating `Int` as a kind

Failure:

```text
I : Int means kind(I) = Int
```

Consequence:

- destroys the distinction between kinds and proper types;
- makes ordinary value types act as meta-level kinds accidentally;
- creates incoherent type application rules.

Requirement:

```text
I : Int means compile-time value domain Const(Int).
```

## 29.4 Encoding every projection in `TypeData`

Failure:

- non-`Type` projections such as `Positionals` are forced into a proper-type node.

Requirement:

- C1 must specify a projection as a domain-neutral semantic concept.

C2 chooses the representation.

## 29.5 Making `Map` concrete-only

Failure:

```text
Map works only after row/pack shape is fully known
```

Consequence:

- generic code cannot preserve symbolic transforms;
- open rows map incorrectly;
- projection normalization becomes eager and lossy.

Requirement:

- symbolic mapped domains are valid semantic forms.

## 29.6 Leaving open row tails unchanged under `Map`

Failure:

```text
Map({a:A | R}, F) = {a:F<A> | R}
```

Consequence:

- transformed record type lies about unknown fields.

Requirement:

```text
tail becomes Map(R,F).
```

## 29.7 Defining `native type` as VM execution

Failure:

- type checking depends on runtime;
- normalization becomes effectful;
- caching and compilation determinism degrade.

Requirement:

- native type members are semantic/compiler evaluators.

## 29.8 Two independent declarations for `[]` and `At`

Failure:

- equality depends on normalization coincidence;
- metadata/reflection can disagree;
- one spelling can be implemented while the other drifts.

Requirement:

- one semantic member identity.

## 29.9 Implicit union distribution

Failure:

- silently introduces union-of-pack and per-branch error semantics;
- can increase normalization complexity substantially.

Requirement:

- no implicit distribution in first release.

## 29.10 Turning C1 into an implementation plan

Failure:

- representation choices become prematurely normative;
- C2 cannot adapt to the current codebase cleanly.

Requirement:

- specify semantic products and laws, not Rust field layouts.

---

# 30. Recommended C1 Work Decomposition

This is a requirements-oriented decomposition, not the detailed implementation plan.

## `LANG007.C1.P1` — Formal Semantic Contract and Decision Closure

Must produce:

- domain/kind formalization;
- typed compile-time value parameter semantics;
- Tuple/Record structural model;
- type-member taxonomy;
- projection model;
- `At` laws;
- `Map` laws;
- zero-product semantics;
- native type-member boundary;
- union behavior;
- scope/deferral decisions;
- durable invariant list.

Gate:

> No unresolved language-level question may be delegated to a representation decision in C2.

## `LANG007.C1.P2` — Authoritative Specification Convergence

Must create:

```text
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/structural-products.md
```

Must amend the existing specs listed in this analysis.

Gate:

> Repository-wide specification search finds no authoritative statement that contradicts the LANG007.C1 semantic contract.

## `LANG007.C1.P3` — Architecture Baseline and Acceptance Matrix

Must produce:

- C2–C7 ownership/dependency map;
- semantic-to-implementation surface inventory;
- conformance matrix;
- positive/negative examples;
- known deferred questions;
- baseline repository evidence;
- handoff requirements.

Gate:

> C2 can begin representation work without inventing source semantics, and C3/C4 can identify the exact language contracts they must implement.

---

# 31. C1 Completion Criteria

`LANG007.C1` should be considered complete only when all of the following are true.

## Semantic completeness

- [ ] `TypePack` is formally defined.
- [ ] `LabeledTypePack` is formally defined.
- [ ] `RecordRow` relationship is explicitly distinguished.
- [ ] typed compile-time value parameters are defined.
- [ ] `I: Int` is unambiguously not kinding by `Int`.
- [ ] `Option : Type -> Type` use in `Map<Option>` is defined.
- [ ] type members have a general semantic owner.
- [ ] associated types are reconciled with that owner.
- [ ] parameterized type members are specified.
- [ ] non-`Type` result domains are specified.
- [ ] projection identity is specified.
- [ ] symbolic/stuck projection semantics are specified.
- [ ] concrete invalid projection semantics are specified.
- [ ] `T[I] ≡ T::At<I>` is specified.
- [ ] negative Tuple indexing is either ratified or explicitly rejected.
- [ ] Tuple linear indexing across positional/labeled lanes is specified.
- [ ] `Map` is specified for Tuple and Record.
- [ ] `Map` is specified for open rows.
- [ ] zero-product behavior is specified.
- [ ] union behavior is specified.
- [ ] native type-member implementation origin is specified.
- [ ] no-VM type normalization is specified.
- [ ] no-general-type-family boundary is specified.

## Specification ownership

- [ ] `type-level-domains.md` exists and owns the new domain model.
- [ ] `type-members.md` exists and owns general type-member semantics.
- [ ] `structural-products.md` exists and owns Tuple/Record static typing.
- [ ] `README.md` ownership map is updated.
- [ ] `generic-constraints.md` no longer contradicts `I: Int`.
- [ ] `associated-types.md` no longer absolutely excludes required parameterized type members.
- [ ] `source-declarations.md` supports compiler-native type declarations.
- [ ] `objects/impl.md` preserves the structural-impl prohibition while allowing domain-native type members.
- [ ] `objects/traits.md` delegates general type-member semantics.
- [ ] `universe.md` agrees with compiler-native declaration ownership.
- [ ] foundational/VM specs are cross-checked for consistency.

## Boundary integrity

- [ ] C1 does not introduce runtime `TypePack` values.
- [ ] C1 does not mandate a concrete Rust enum/arena layout.
- [ ] C1 does not introduce arbitrary user type functions.
- [ ] C1 does not require general nominal const generics unless explicitly promoted.
- [ ] C1 does not make anonymous Tuple/Record types legal ordinary inherent-`impl` targets.
- [ ] C1 does not redefine VM product layout.

## Verification

- [ ] cross-spec contradiction search performed;
- [ ] semantic example matrix completed;
- [ ] all new invariants have one clear owner;
- [ ] all C2–C7 deferred concerns have explicit owners;
- [ ] checkpoint record truthfully identifies feature implementation as not yet complete after C1.

---

# 32. Recommended C1 Output Artifacts

A clean checkpoint directory would contain:

```text
docs/implementation/LANG007/
    LANG007-PROGRAM.md
    LANG007-STATUS.md

    LANG007.C1/
        LANG007.C1-CHECKPOINT.md

        LANG007.C1.P1-formal-semantic-contract-and-decision-closure.md
        LANG007.C1.P1-walkthrough.md
        LANG007.C1.P1-handoff.md

        LANG007.C1.P2-authoritative-specification-convergence.md
        LANG007.C1.P2-walkthrough.md
        LANG007.C1.P2-handoff.md

        LANG007.C1.P3-architecture-baseline-and-acceptance-matrix.md
        LANG007.C1.P3-walkthrough.md
        LANG007.C1.P3-handoff.md
```

The checkpoint record should remain the single durable state authority in accordance with the repository workflow convention.

---

# 33. Recommended Final C1 Semantic Surface

The intended conceptual source surface after specification convergence should read coherently as:

```phalcom
native class Tuple is Collection {
    @intrinsic(...)
    native type Positionals: TypePack

    @intrinsic(...)
    native type Labeled: LabeledTypePack

    @intrinsic(...)
    native type At<I: Int>: Type

    @intrinsic(...)
    native type Map<F: Type -> Type>: Type
}
```

and:

```phalcom
native class Record is Object {
    @intrinsic(...)
    native type Map<F: Type -> Type>: Type
}
```

The exact placement of these declarations may change if C1 chooses a distinct structural-domain declaration mechanism rather than attaching them textually to runtime `Tuple` and `Record` classes.

The semantic contract must nevertheless remain:

```text
Tuple structural type
    Positionals : TypePack
    Labeled     : LabeledTypePack
    At<Int const> : Type
    Map<Type -> Type> : Type

Record structural type
    Map<Type -> Type> : Type
```

with underlying domain operations:

```text
TypePack::Map<F>        : TypePack
LabeledTypePack::Map<F> : LabeledTypePack
RecordRow::Map<F>       : RecordRow
```

The source syntax should present one coherent model without requiring these domains to be runtime objects.

---

# 34. Overall Assessment

`LANG007.C1` is architecturally necessary before any compiler implementation begins.

The existing Phalcom type system already contains several of the right foundations:

- a real kind layer;
- a distinct `RecordRow` kind;
- higher-kinded arrow kinds;
- structural Tuple and Record proper types;
- type lambdas;
- generic row parameters;
- symbolic associated projections;
- clear runtime Tuple/Record identity distinctions;
- zero-product normalization to `Unit`.

The missing pieces are not reasons to replace that architecture. They are reasons to **complete the semantic taxonomy**.

The most important C1 design principle is:

> **Do not model LANG007 as syntax sugar over ad hoc Tuple intrinsics. Model it as a small structural type calculus with distinct domains, general type-member projection, and closed compiler-native transformations.**

Under that model:

```phalcom
(Int, String)::Map<Option>
```

is coherent because:

```text
(Int, String)
    structural Tuple proper type

Option
    type constructor of kind Type -> Type

Map
    shape-preserving type member

result
    canonical structural Tuple proper type
```

Likewise:

```phalcom
#{name: String}::Map<Option>
```

uses the same transformation law over a different structural shape domain.

And:

```phalcom
(Int, String)::At<1>
```

is coherent because:

```text
1
    compile-time value typed Int

At
    parameterized type member

result
    proper Type
```

without ever claiming that `Int` is a kind.

If C1 ratifies these distinctions and assigns each concept to one authoritative specification, C2–C5 can be implemented with relatively direct compiler architecture. If C1 leaves them implicit, later implementation will almost certainly accumulate special cases around `TypeId`, `RecordRowId`, Tuple element arrays, and associated projections that then have to be undone.

The checkpoint should therefore be considered successful when the next implementer can answer every semantic question by citing an authoritative spec, while still having freedom to choose efficient canonical representations and solver machinery in later LANG007 checkpoints.

---

# Appendix A — Proposed Specification Ownership After C1

```text
docs/specs/README.md
    global authority and ownership index

docs/specs/type-level-domains.md
    Type
    RecordRow
    TypePack
    LabeledTypePack
    arrow kinds
    proper type
    type forms
    compile-time parameter domains

docs/specs/type-members.md
    type-member declarations
    member parameters
    result domains
    member identity
    lookup
    projection
    symbolic projection
    native/alias/requirement semantic categories

docs/specs/structural-products.md
    Unit as zero product
    Tuple static typing
    Record static typing
    Positionals
    Labeled
    At
    type subscript
    Map
    ordered/unordered structural laws

docs/specs/generic-constraints.md
    where propositions
    generic constraint relations
    use of projections in constraints
    parameter-domain syntax integration

docs/specs/associated-types.md
    trait-owned associated requirements
    conformance bindings
    evidence-based associated normalization
    contextual and qualified trait projection

docs/specs/source-declarations.md
    visibility
    native/foreign implementation origin
    native type-member declaration origin

docs/specs/objects/traits.md
    trait requirements/conformance
    delegates general type-member declaration rules

docs/specs/objects/impl.md
    inherent/conformance impl
    structural anonymous impl prohibition

docs/specs/universe.md
    canonical builtin declaration ownership
    native compiler/runtime associations

docs/specs/vm/products.md
    runtime product realization only

docs/specs/vm/values-and-objects.md
    runtime Unit/Tuple representation only
```

---

# Appendix B — Proposed Mathematical Summary

Let:

```text
κ ::= Type
    | RecordRow
    | TypePack
    | LabeledTypePack
    | (κ₁, ..., κₙ) -> κ
```

Type-level kinding:

```text
Γ ⊢ A : κ
```

Compile-time value typing:

```text
Γ ⊢ c :const τ
```

Generic parameter domains:

```text
δ ::= Kind(κ) | Const(τ)
```

Tuple shape:

```text
Γ ⊢ P : TypePack
Γ ⊢ L : LabeledTypePack
--------------------------------
Γ ⊢ Tuple(P, L) : Type
```

Record shape:

```text
Γ ⊢ R : RecordRow
-----------------
Γ ⊢ Record(R) : Type
```

Pack mapping:

```text
Γ ⊢ P : TypePack
Γ ⊢ F : Type -> Type
--------------------------------
Γ ⊢ MapPack(P, F) : TypePack
```

Labeled pack mapping:

```text
Γ ⊢ L : LabeledTypePack
Γ ⊢ F : Type -> Type
----------------------------------------
Γ ⊢ MapLabeledPack(L, F) : LabeledTypePack
```

Row mapping:

```text
Γ ⊢ R : RecordRow
Γ ⊢ F : Type -> Type
--------------------------------
Γ ⊢ MapRow(R, F) : RecordRow
```

Tuple mapping:

```text
MapTuple(Tuple(P, L), F)
    =
Tuple(MapPack(P, F), MapLabeledPack(L, F))
```

Record mapping:

```text
MapRecord(Record(R), F)
    =
Record(MapRow(R, F))
```

Tuple indexing:

```text
linear(Tuple(P, L))
    =
P ++ eraseLabels(L)
```

For concrete arity `N` and compile-time integer `i`:

```text
j = i      if i >= 0
j = N + i  if i < 0
```

If:

```text
0 <= j < N
```

then:

```text
Tuple(P, L)::At<i>
    =
linear(Tuple(P, L))[j]
```

otherwise the projection is statically invalid.

Map/index law:

```text
(T::Map<F>)::At<I>
    ≡
F<T::At<I>>
```

for every valid Tuple `I`.

Zero product:

```text
Tuple([], []) ≡ Unit ≡ ()
```

and:

```text
()::Map<F> ≡ ()
```

---

# Appendix C — Repository Evidence Reviewed

The requirements analysis was checked against the current repository sources below.

## Authoritative specifications

- [`docs/specs/README.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/README.md)
- [`docs/specs/generic-constraints.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/generic-constraints.md)
- [`docs/specs/associated-types.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/associated-types.md)
- [`docs/specs/source-declarations.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/source-declarations.md)
- [`docs/specs/objects/values.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/objects/values.md)
- [`docs/specs/objects/impl.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/objects/impl.md)
- [`docs/specs/objects/traits.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/objects/traits.md)
- [`docs/specs/universe.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/universe.md)
- [`docs/specs/vm/products.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/vm/products.md)
- [`docs/specs/vm/values-and-objects.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/specs/vm/values-and-objects.md)

## Current semantic/compiler implementation

- [`phalcom-semantic/src/types/kind.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/types/kind.rs)
- [`phalcom-semantic/src/types/store.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/types/store.rs)
- [`phalcom-semantic/src/types/row.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/types/row.rs)
- [`phalcom-semantic/src/types/instantiation.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/types/instantiation.rs)
- [`phalcom-semantic/src/types/type_lambda.rs`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-semantic/src/types/type_lambda.rs)

## Universe runtime surfaces

- [`phalcom-core/core/universe/src/collections/tuple.ph`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-core/core/universe/src/collections/tuple.ph)
- [`phalcom-core/core/universe/src/collections/record.ph`](https://github.com/aureat/phalcom-lang/blob/main/phalcom-core/core/universe/src/collections/record.ph)

## Workflow convention

- [`docs/workflow/implementation-record-lifecycle-convention.md`](https://github.com/aureat/phalcom-lang/blob/main/docs/workflow/implementation-record-lifecycle-convention.md)

---

# Appendix D — Recommended Decision Register for C1

| ID | Decision | Recommended result |
|---|---|---|
| `D1` | Are packs proper types? | No; distinct type-level domains. |
| `D2` | Is labeled pack a row? | No; ordered versus unordered identity. |
| `D3` | Meaning of `I: Int` | compile-time value parameter typed `Int`. |
| `D4` | Is `Int` a kind? | No. |
| `D5` | `Option` in `Map<Option>` | type form of kind `Type -> Type`. |
| `D6` | Projection result domains | general; not restricted to `Type`. |
| `D7` | General type-member owner | new `type-members.md`. |
| `D8` | Structural Tuple/Record owner | new `structural-products.md`. |
| `D9` | Kind/domain owner | new `type-level-domains.md`. |
| `D10` | `T[I]` vs `T::At<I>` | one semantic member identity. |
| `D11` | Negative Tuple type indexes | supported, matching runtime integer indexing. |
| `D12` | Tuple label order | identity-bearing. |
| `D13` | Record field order | not identity-bearing. |
| `D14` | Empty Tuple | canonical zero product `Unit`. |
| `D15` | `()::Map<F>` | `()`. |
| `D16` | Open-row map | recursively/symbolically maps tail. |
| `D17` | Union distribution | deferred/not implicit. |
| `D18` | `Map` type-function power | closed compiler-native operation only. |
| `D19` | Native type execution | semantic/compiler host; no VM dispatch. |
| `D20` | Anonymous structural inherent `impl` | remains prohibited. |
| `D21` | Heterogeneous generic scope | general semantic model; minimum source requirement limited to type-member applications unless expanded deliberately. |
| `D22` | Runtime TypePack object | not introduced by LANG007.C1. |
| `D23` | Projection invalidity | static error when concretely invalid; distinct from stuck. |
| `D24` | General const evaluation | deferred; only restricted compile-time terms required. |

These decisions should be either explicitly ratified or explicitly replaced by alternatives before C1 is marked complete.
