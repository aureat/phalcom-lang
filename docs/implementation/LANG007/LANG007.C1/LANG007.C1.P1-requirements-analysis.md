---
id: LANG007.C1.P1.requirements
category: LANG
program: LANG007
checkpoint: LANG007.C1
plan_target: LANG007.C1.P1
kind: requirements-analysis
status: COMPLETE_FOR_PLANNING
completion: NOT_STARTED
verification: REPOSITORY_GROUNDED
prepared: 2026-09-21
repository: aureat/phalcom-lang
planning_branch: main
planning_revision: 5b63ade8073d13d6b528af4774181665c30cbd47
planning_revision_subject: "fix(core): restore standalone semantic universe"
predecessor_analysis: LANG007.C1-requirements-analysis.md
---

# LANG007.C1.P1 — Requirements Analysis

## Formal Semantic Contract and Decision Closure for Variadic Structural Typing and Intrinsic Type Operations

---

# 0. Purpose

This document is the requirements-analysis input for a future:

```text
LANG007.C1.P1 implementation/specification work plan
```

It is **not** that implementation plan.

Its purpose is to determine, before patch sequencing or specification-edit tasks are written, exactly what `LANG007.C1.P1` must decide, formalize, constrain, and hand to `LANG007.C1.P2`.

`LANG007.C1.P1` is the semantic decision checkpoint inside the larger `LANG007.C1` specification-convergence checkpoint. P1 must establish a mathematically coherent language model for:

```text
TypePack
LabeledTypePack
RecordRow
typed compile-time generic arguments
parameterized type members
non-Type type-member result domains
symbolic projections
Tuple structural projections
type-level Tuple indexing
shape-preserving Map<F>
Record structural mapping
open-row mapping
native compiler-assisted type operations
zero-product semantics
```

without yet implementing the compiler representation, parser changes, normalizer, solver, metadata schema, LSP behavior, or runtime reflection.

The analysis is grounded against the current repository state:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   5b63ade8073d13d6b528af4774181665c30cbd47
commit:     fix(core): restore standalone semantic universe
```

It also consumes the previously prepared checkpoint-level analysis:

```text
LANG007.C1-requirements-analysis.md
```

That prior artifact is design input. It is not assumed to be committed or authoritative repository state.

The governing principle is:

> **P1 must decide what the language means. It must not decide semantic questions accidentally by choosing Rust representations.**

---

# 1. Executive Conclusion

`LANG007.C1.P1` is required because the intended feature surface crosses semantic boundaries that the current authoritative specifications either do not own or currently define too narrowly.

The desired language examples are deceptively compact:

```phalcom
(Int, String)::Positionals
(Int, String, name: Symbol)::Labeled

(Int, String)[1]
(Int, String)::At<1>

(Int, String)::Map<Option>
(Int, String, name: Symbol)::Map<Option>

#{name: String, age: Int}::Map<Option>
```

but a coherent implementation requires the language to distinguish at least five different kinds of semantic object:

```text
proper types
type constructors / higher-kinded type forms
record rows
ordered type packs
typed compile-time values
```

and to define a type-member mechanism whose projections can return more than ordinary proper types.

The current compiler already demonstrates the right architectural direction:

```rust
KindData::Type
KindData::RecordRow
KindData::Arrow { ... }

TypeData::Tuple(...)
TypeData::Record(...)
TypeData::Lambda(...)
TypeData::AssociatedProjection(...)
```

The correct LANG007 direction is therefore **extension of the existing semantic tower**, not a parallel tuple-specific mini type system.

P1 should close the following central decisions:

1. `TypePack` is a distinct type-level kind/domain.
2. `LabeledTypePack` is a distinct ordered type-level kind/domain.
3. `RecordRow` remains a separate unordered key-oriented type-level domain.
4. Tuple labeled structure must not reuse `RecordRow` identity.
5. `I: Int` denotes a compile-time value parameter whose value type is `Int`; `Int` does not become a kind.
6. `:` should be understood as generic-parameter/domain classification in binder contexts, with kind classification as one domain case.
7. `Option` in `Map<Option>` is a type form of kind `Type -> Type`, not a proper type.
8. `Map<F>` accepts any well-formed type form of kind `Type -> Type`, not only nominal declarations.
9. Phalcom needs a general semantic category of **type member**.
10. LANG007 requires parameterized **domain-native** type members, but does not require generic associated types for traits in the first release.
11. Type members must have explicit result domains; results are not restricted to `Type`.
12. Projections must have domain-neutral identity and may remain symbolic.
13. “Stuck” and “invalid” are distinct semantic states.
14. `T[I]` and `T::At<I>` are one semantic operation.
15. Tuple type indexing initially accepts compile-time `Int` arguments only.
16. Negative indexes should match runtime Tuple integer indexing.
17. Tuple indexing is linear across positional entries followed by labeled entries.
18. `Map<F>` is a shape-preserving structural transformation.
19. Tuple `Map` preserves positional/labeled partition, labels, order, and arity.
20. Record `Map` preserves key set and row openness.
21. Mapping an open row must symbolically map its unknown tail.
22. Abstract pack/row mapping must remain a stable symbolic type-level term.
23. The zero-arity Tuple syntax `()` has zero-product semantics and canonical static `Unit` identity.
24. Empty Record syntax `#{}` requires a more careful rule: runtime representation may normalize to Unit while static structural Record type identity may remain distinct.
25. Union-distributive structural projection is not implicit in the initial feature.
26. Native type members execute in semantic normalization, not through VM dispatch.
27. `native` and `@intrinsic` remain distinct facts.
28. LANG007 does not introduce arbitrary user-defined type families or general compile-time execution.
29. The initial heterogeneous generic-argument requirement should be scoped to type-member applications rather than immediately turning all nominal/trait generics into const/pack generics.
30. Structural type members do not make anonymous Tuple/Record types legal ordinary inherent-`impl` targets.

If these decisions are ratified clearly, `LANG007.C1.P2` can convert them into authoritative specification text and `LANG007.C2` can choose efficient canonical representations without reopening language design.

---

# 2. P1 Bounded Acceptance Objective

The bounded acceptance objective for `LANG007.C1.P1` should be:

> **Produce one complete formal semantic contract for the LANG007 type-level domains and structural type operations, close every language-level decision required by that contract, record explicit non-goals, and leave no unresolved semantic question that C2 or C3 would otherwise have to answer through implementation choices.**

P1 is complete only when later implementers can answer questions such as:

```text
What is TypePack?
What is the kind of P?
What is the semantic category of 0 in At<0>?
What does I: Int mean?
What is the kind of Option?
What does Map<Option> require?
What is a type member?
Who owns Tuple::Map?
What is the result domain of Positionals?
What makes a projection symbolic?
When is a projection invalid?
What does T[I] mean?
What is the relationship between [] and At?
What does Map do to an open RecordRow?
What does Map do to ()?
What does Map do to #{}?
Does Map distribute over unions?
Can a generic T: Type use Map?
Can P: TypePack use Map?
Can a trait declare type At<I>?
Does native type run the VM?
```

by citing a P1-ratified semantic rule rather than guessing from code.

---

# 3. What P1 Is Not

P1 is intentionally **not** the implementation phase.

It must not become a task list for:

```text
adding KindData variants
adding new arena IDs
editing parser.rs
editing TypeData
adding GenericArgument
adding metadata nodes
adding LSP hover support
adding inference variables
adding pack solver code
writing intrinsic dispatch tables
changing VM bytecodes
```

Those changes are later implementation concerns.

P1 may use conceptual data structures to test semantic feasibility, but those structures are non-normative unless the semantic distinction itself depends on them.

For example, P1 may say:

```text
GenericParameterDomain ::= Kind(κ) | Const(τ)
```

because that distinction is semantic.

It should not say:

```rust
#[repr(u8)]
enum GenericParameterDomain { ... }
```

or prescribe arena allocation/layout.

---

# 4. Authority Model

P1 sits between current authoritative language specifications, implementation reality, older design material, and the new LANG007 checkpoint analysis.

The authority hierarchy should be explicit.

## 4.1 Current authoritative language specifications

The current `docs/specs/README.md` states that `docs/specs/` owns current language semantics and that implementation plans, historical drafts, VM details, and tests do not override the owning language specification.

Relevant current owners include:

```text
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/source-declarations.md
docs/specs/objects/impl.md
docs/specs/objects/traits.md
docs/specs/objects/values.md
docs/specs/universe.md
```

P1 is allowed to propose semantic changes that P2 will later make authoritative, but P1 must explicitly identify every current rule it intends to supersede or refine.

## 4.2 Current VM specifications

Relevant VM facts include:

```text
Tuple label order is semantically significant.
Record key order is not structural identity.
Unit is the canonical zero product.
Positive Tuple and Record products use separate structural categories.
Empty Record runtime representation may normalize to Unit.
```

These are constraints and evidence.

VM documents do not become the new language-level owner.

## 4.3 Live compiler/runtime source

Live source establishes implementation facts and feasibility:

```text
KindData currently has Type, RecordRow, Arrow.
TypeStore already distinguishes proper types from higher-kinded type forms.
RecordRowData sorts fields by name.
TupleTypeElement preserves ordered labels.
TypeData::AssociatedProjection is currently Type-valued.
GenericInstantiation already has separate type/row domains.
Type lambdas carry parameter/result kinds.
Tuple runtime indexing accepts negative Int indexes.
Tuple runtime equality is label-order-sensitive.
Record runtime equality is key-set based.
```

P1 should preserve useful architecture where possible, but source code does not decide language meaning when a P1 design choice is still open.

## 4.4 Historical/provisional specifications

Older material under:

```text
docs/spec/
docs/implementation/
docs/wiki/
docs/internals/
```

is useful evidence.

For example, older Tuple specifications explicitly state:

```text
duplicate Tuple labels are invalid
```

and product documents discuss empty Record normalization.

These are supporting inputs, not automatic current language authority.

## 4.5 LANG007.C1 requirements analysis

The checkpoint-level analysis established the broad program direction:

```text
C1 = semantic contract and specification convergence
P1 = formal semantic contract and decision closure
P2 = authoritative specification convergence
P3 = architecture baseline and acceptance matrix
```

P1 should refine that direction, especially where the C1 analysis deliberately left broad options open.

---

# 5. Current Repository Facts P1 Must Respect

## 5.1 Current kind model

Current:

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

This proves the language/compiler already recognizes non-`Type` domains.

P1 should extend the semantic model rather than collapsing packs into `Type`.

## 5.2 Proper-type distinction

`TypeStore::proper_type` accepts only forms whose kind is `Type`.

Therefore:

```text
Int
List<Int>
(Int, String)
#{name: String}
```

are proper types.

A constructor such as:

```text
Option : Type -> Type
```

is a type-level form but not yet a proper runtime value type.

This distinction is essential to `Map<Option>`.

## 5.3 Concrete Tuple type representation

Current:

```rust
TypeData::Tuple(Box<[TupleTypeElement]>)
```

with:

```rust
TupleTypeElement {
    label: Option<Box<str>>,
    ty: TypeId,
}
```

Tuple element order is therefore represented directly.

## 5.4 Zero Tuple normalization

Current:

```rust
pub fn tuple(&mut self, elements: Box<[TupleTypeElement]>) -> TypeId {
    if elements.is_empty() {
        return self.unit();
    }
    ...
}
```

Thus:

```text
Tuple([], []) -> Unit
```

already holds at the semantic type-store boundary.

P1 must explicitly account for it.

## 5.5 RecordRow canonicalization

Current:

```rust
pub struct RecordRowData {
    pub fields: Box<[RecordRowField]>, // sorted by name, unique
    pub tail: RecordRowTail,
}
```

Rows are canonicalized by key.

This is incompatible with ordered Tuple labeled-lane identity.

## 5.6 Current Record type representation

Current:

```rust
TypeData::Record(RecordRowId)
```

Unlike Tuple construction, current static record type construction does **not** automatically convert an empty row to `Unit`.

This matters.

P1 must distinguish:

```text
static semantic type identity
runtime zero-product representation
```

rather than assuming all empty products have one static type identity.

## 5.7 Runtime empty Record behavior

The authoritative VM product specification says:

```phalcom
#{}
```

normalizes at runtime to the same canonical zero-product representation as `Unit`, while explicitly noting that this physical normalization does not erase a source/static distinction where static typing retains one.

That wording gives P1 a clean path:

> Static `#{}` may remain an empty structural Record type even when its runtime value representation is Unit.

P1 should ratify this distinction.

## 5.8 Current generic instantiation is already domain-aware

Current:

```rust
GenericInstantiation {
    type_bindings: ...,
    row_bindings: ...,
}
```

This supports a general semantic rule:

> Generic substitution is domain-indexed; not every generic argument is a `TypeId`.

## 5.9 Current associated projections are Type-valued

Current:

```rust
TypeData::AssociatedProjection(...)
```

is interned in the ordinary type store and existing associated types denote proper types.

Therefore P1 must not require non-`Type` projections to masquerade as ordinary proper types.

## 5.10 Type lambdas and higher-kinded forms already exist

Current type-lambda data includes:

```text
parameter_kinds
result_kind
```

This provides the correct semantic basis for:

```text
Option : Type -> Type
```

and future anonymous type constructors.

`Map<F>` should use ordinary higher-kinded application semantics.

---

# 6. P1 Semantic Domain Model

P1 should ratify one formal domain model.

## 6.1 Kind grammar

Recommended:

```text
κ ::= Type
    | RecordRow
    | TypePack
    | LabeledTypePack
    | (κ₁, ..., κₙ) -> κ
```

This is deliberately small.

P1 should **not** add:

```text
Int
String
Symbol
```

as kinds.

Those are proper types.

## 6.2 Type-level kinding judgment

Use:

```text
Γ ⊢ A : κ
```

Examples:

```text
Γ ⊢ Int : Type
Γ ⊢ String : Type

Γ ⊢ Option : Type -> Type

Γ ⊢ P : TypePack
Γ ⊢ L : LabeledTypePack
Γ ⊢ R : RecordRow
```

## 6.3 Compile-time value judgment

Use a separate judgment:

```text
Γ ⊢ c :const τ
```

Examples:

```text
Γ ⊢ 0 :const Int
Γ ⊢ 1 :const Int
Γ ⊢ -1 :const Int
Γ ⊢ I :const Int
```

This is the mathematically important distinction.

The exact term:

```text
0
```

used in:

```phalcom
T::At<0>
```

is a compile-time constant term whose value type is `Int`.

It is not required to be a singleton type named `0`.

If Phalcom later gains singleton literal types, expected argument domain disambiguates:

```text
expected Const(Int)
    -> literal 0 is a compile-time Int term

expected Type
    -> a literal-type feature may interpret 0 as a type if separately specified
```

LANG007 must not conflate the two.

---

# 7. Generic Parameter Domains

The semantic parameter domain should be:

```text
δ ::= Kind(κ)
    | Const(τ)
```

Examples:

```phalcom
T: Type
```

means:

```text
Domain(T) = Kind(Type)
```

```phalcom
F: Type -> Type
```

means:

```text
Domain(F) = Kind(Type -> Type)
```

```phalcom
P: TypePack
```

means:

```text
Domain(P) = Kind(TypePack)
```

```phalcom
I: Int
```

means:

```text
Domain(I) = Const(Int)
```

This gives a single coherent model while preserving the distinction between kinds and proper value types.

---

# 8. Meaning of `:` in P1

This is one of the most important semantic decisions because current `generic-constraints.md` defines `:` specifically as kind classification.

P1 should refine the rule to:

> In a generic-parameter declaration/classification context, `:` classifies the parameter into a semantic parameter domain. A kind expression denotes a kinded type-level domain; a proper type denotes a compile-time value domain.

Thus:

```phalcom
T: Type
P: TypePack
F: Type -> Type
I: Int
```

are all domain classifications.

The RHS elaboration determines which semantic case applies:

```text
Type
    kind expression

TypePack
    kind expression

Type -> Type
    kind expression

Int
    proper type
```

This does **not** make trait-bound shorthand legal.

For example:

```phalcom
T: Printable
```

must not become an alternate spelling for:

```phalcom
T |= Printable
```

unless `Printable` genuinely denotes a proper value type under another language rule.

Trait conformance remains `|=`.

## 8.1 Binder versus `where`

P1 should avoid unnecessarily expanding the `where` grammar.

Recommended first-release rule:

```text
parameter-domain classification
    belongs to the binder

where
    states propositions about already-bound parameters
```

Therefore:

```phalcom
type At<I: Int>: Type
```

is canonical.

P1 does not need to require:

```phalcom
type At<I>: Type
    where I: Int
```

for const-domain declaration.

Existing:

```phalcom
where F: Type -> Type
```

may remain supported for compatibility as a kind proposition, but P1 should avoid inventing a general `where I: Int` const-domain restatement unless there is a concrete use case.

This distinction should be handed to P2 for precise grammar reconciliation.

---

# 9. `TypePack`

P1 should define `TypePack` as:

> An ordered finite sequence of proper type terms.

Mathematically:

```text
P = [T₀, T₁, ..., Tₙ]
```

with:

```text
Γ ⊢ Tᵢ : Type
```

and therefore:

```text
Γ ⊢ P : TypePack
```

Important consequences:

1. `TypePack` is not a proper type.
2. `TypePack` is not a Tuple proper type.
3. Pack order is identity-bearing.
4. Pack equality is elementwise and order-sensitive.
5. A pack may be abstract/symbolic.
6. A pack may be empty.
7. A pack may later participate in variadic expansion.
8. Pack elements are proper `Type` terms, not arbitrary kinds.

P1 should **not** define a pack as:

```text
Tuple<Type, Type, ...>
```

because that creates the wrong kind and conflates a type sequence with a runtime value type.

---

# 10. `LabeledTypePack`

P1 should define `LabeledTypePack` as:

> An ordered finite sequence of unique structural labels paired with proper type terms.

Mathematically:

```text
L = [(ℓ₀, T₀), (ℓ₁, T₁), ..., (ℓₙ, Tₙ)]
```

where:

```text
Γ ⊢ Tᵢ : Type
```

and labels are structural label atoms.

A label may be reified as a runtime `Symbol`, but the semantic definition should not depend on a runtime `Symbol` object existing during type checking.

Required laws:

```text
order(L) matters
labels are unique
payload types are proper Type
```

Examples:

```text
[(name, String), (age, Int)]
```

and:

```text
[(age, Int), (name, String)]
```

are distinct labeled packs.

P1 should explicitly ratify label uniqueness because:

- existing Tuple runtime construction rejects duplicates;
- historical/current collection semantics treat Tuple labels as unique;
- label lookup would otherwise become ambiguous;
- Tuple shape identity becomes cleaner.

---

# 11. `RecordRow`

P1 should preserve:

> `RecordRow` is an unordered/key-oriented finite structural map of unique labels to proper types, optionally with a symbolic open tail.

Conceptually:

```text
R = { name: String, age: Int }
```

with:

```text
Γ ⊢ R : RecordRow
```

The following rows are equivalent:

```text
{ name: String, age: Int }
{ age: Int, name: String }
```

assuming the same openness/tail semantics.

The implementation currently sorts fields by name. P1 should treat sorting as canonicalization, not source identity.

---

# 12. `LabeledTypePack` Is Not `RecordRow`

This distinction must be explicit enough to become an invariant.

They may contain corresponding label/type pairs:

```text
LabeledTypePack:
    [(name, String), (age, Int)]

RecordRow:
    {name: String, age: Int}
```

but:

```text
identity(LabeledTypePack)
    includes order

identity(RecordRow)
    excludes order
```

Therefore:

```text
[(name, String), (age, Int)]
!=
[(age, Int), (name, String)]
```

while:

```text
{name: String, age: Int}
==
{age: Int, name: String}
```

This is the central reason Tuple labeled structure must not reuse `RecordRow`.

Implementation reuse is allowed at the level of generic utilities:

```text
interning patterns
query-local solver architecture
diagnostic infrastructure
budget/cancellation
```

but not semantic identity.

---

# 13. Structural Tuple Type Model

For positive-arity Tuple structural types, P1 should model:

```text
Tuple(P, L)
```

where:

```text
P : TypePack
L : LabeledTypePack
```

and the complete linear element sequence is:

```text
linear(Tuple(P, L))
    =
P ++ eraseLabels(L)
```

The structural lane law is:

```text
all positional elements precede all labeled elements
```

The labeled suffix preserves declaration/source structural order.

## 13.1 Concrete example

```phalcom
(Int, String, name: Symbol, enabled: Bool)
```

means:

```text
P = [Int, String]

L = [
    (name, Symbol),
    (enabled, Bool)
]
```

and:

```text
linear = [Int, String, Symbol, Bool]
```

## 13.2 Tuple identity

Tuple structural identity should include:

```text
positional arity
positional types in order
labeled labels in order
labeled payload types in order
```

This aligns with runtime Tuple equality and VM shape invariants.

---

# 14. Zero-Arity Tuple and `Unit`

Current semantic formation already does:

```text
Tuple([], []) -> Unit
```

P1 should ratify this as language semantics:

```text
() ≡ Unit
```

for the zero-arity Tuple/product syntax.

Important:

> This is static type identity, not merely runtime storage optimization.

Therefore:

```phalcom
()::Map<Option>
```

should reduce to:

```phalcom
()
```

and:

```phalcom
()::Positionals
```

should be the empty `TypePack`.

Likewise:

```phalcom
()::Labeled
```

should be the empty `LabeledTypePack`.

An index:

```phalcom
()[0]
```

is concretely invalid.

## 14.1 Applicability despite runtime class

`()` may have runtime behavior classified through `Unit`, not `Tuple`.

Therefore type-member applicability cannot be defined solely as:

```text
runtime class == Tuple
```

The structural zero-product semantics must be recognized by the semantic type system.

---

# 15. Structural Record Type Model

P1 should model a structural Record proper type as:

```text
Record(R)
```

where:

```text
R : RecordRow
```

For positive records:

```phalcom
#{name: String, age: Int}
```

the static identity is key/type based and order-insensitive.

---

# 16. Empty Record and Zero-Product Representation

This requires a distinct P1 decision from empty Tuple.

Current static type formation can represent:

```text
Record({})
```

as a structural Record type.

Current authoritative VM semantics say:

```phalcom
#{}
```

uses the canonical zero-product runtime representation `Unit`.

The VM specification explicitly says this physical normalization does not erase a source/static distinction where static typing retains one.

P1 should therefore ratify:

> **Empty Record retains Record static structural type semantics unless a separate language rule establishes static equivalence with Unit. Runtime representation may use Unit as the canonical zero-product value representation.**

Recommended initial law:

```text
TypeOf(#{})
    = Record({})

runtime-representation(#{})
    = Unit zero-product representation
```

not:

```text
Record({}) == Unit
```

at the static type level.

This distinction avoids forcing current semantic `TypeData::Record(empty_row)` to disappear merely because runtime storage is optimized.

It also gives clear Map semantics:

```phalcom
#{}::Map<Option>
    ==
#{}
```

as a static Record type transformation, even though the runtime value representation of either empty record may be Unit.

This is a strong example of Phalcom’s broader rule:

```text
static type identity
!= runtime representation identity
```

---

# 17. Type Forms, Proper Types, and `Map<F>`

P1 must clearly distinguish:

```text
proper type
type constructor/type form
```

Examples:

```text
Int
    : Type

Option
    : Type -> Type

Option<Int>
    : Type
```

Therefore:

```phalcom
(Int, String)::Map<Option>
```

is valid because the member parameter expects:

```text
F : Type -> Type
```

P1 should permit any type-level form of the expected kind:

```text
nominal generic type constructor
type lambda
partially applied constructor when residual kind matches
other canonical higher-kinded form
```

It should **not** special-case `Option`.

If Phalcom supports a source type lambda such as:

```phalcom
<T> =>> Result<T, Error>
```

or another canonical spelling, then it should be eligible whenever its kind is:

```text
Type -> Type
```

---

# 18. General Type Member Semantic Category

P1 should introduce the semantic term **type member**.

This is necessary because LANG007 needs operations that are not adequately described as existing trait-associated types.

Recommended semantic categories:

```text
TypeMember
    InherentTypeMember
    AssociatedTypeMember
    DomainNativeTypeMember
```

These are ownership/applicability categories.

Separately, implementation definition can be:

```text
Requirement
Alias
Native
```

The two axes should not be conflated.

For example:

```text
Iterator::Item
    ownership: AssociatedTypeMember
    implementation: Requirement at trait declaration,
                    Binding in conformance

Tuple-family::Map
    ownership: DomainNativeTypeMember
    implementation: Native
```

---

# 19. Domain-Native Type Members

LANG007’s Tuple/Record operations should be specified as **domain-native type members**.

This solves a major coherence problem.

Current `objects/impl.md` says anonymous structural Tuple and Record types are not ordinary inherent-`impl` targets.

P1 should preserve that rule.

Therefore:

```phalcom
(Int, String)::Map<Option>
```

must **not** mean that `(Int, String)` has an ordinary user-extensible inherent member table.

Instead:

```text
the structural Tuple domain
    provides Map

the structural Record domain
    provides Map

TypePack domain
    provides Map

RecordRow domain
    provides Map
```

through compiler-defined semantics.

## 19.1 Universe declarations as anchors

If source declarations such as:

```phalcom
native class Tuple {
    native type Map<F: Type -> Type>: Type
}
```

are used for documentation, reflection, stable declaration identity, or intrinsic registration, P1 should distinguish:

```text
source declaration anchor
from
semantic applicability domain
```

The member can be anchored in canonical Universe source while being applicable to structural Tuple proper types according to domain rules.

This is preferable to pretending every structural Tuple type is a nominal instance of a generic `Tuple<...>` type.

---

# 20. Associated Types Boundary

P1 should **not** broaden LANG007 unnecessarily into generic associated types.

The existing associated-type system currently specifies trait-owned proper-type requirements such as:

```phalcom
trait Iterator {
    type Item
}
```

LANG007 requires parameterized type members for domain-native operations:

```phalcom
type Map<F: Type -> Type>: Type
type At<I: Int>: Type
```

It does **not** require:

```phalcom
trait Indexed {
    type At<I: Int>
}
```

in the first release.

Recommended P1 decision:

> The general type-member semantic model supports parameter lists and result domains conceptually, but declaration categories may restrict which forms are currently legal. LANG007 first requires parameterized domain-native type members. Existing trait-associated types remain non-parameterized proper-`Type` requirements unless a future feature explicitly introduces generic associated types.

This has several advantages:

- no accidental GAT implementation scope;
- `associated-types.md` can remain mostly intact;
- `type-members.md` can become the general owner without forcing every category to expose every capability immediately;
- future GATs can reuse the same member substrate.

P2 should therefore amend `associated-types.md` primarily to delegate common projection/member identity concepts, not necessarily to legalize GAT syntax now.

---

# 21. Type-Member Result Domains

A type member must have a result domain.

Required examples:

```text
Positionals
    result: TypePack

Labeled
    result: LabeledTypePack

At<I>
    result: Type

Map<F> on Tuple/Record
    result: Type

Map<F> on TypePack
    result: TypePack

Map<F> on LabeledTypePack
    result: LabeledTypePack

Map<F> on RecordRow
    result: RecordRow
```

This is a fundamental P1 rule.

A projection is not always a proper type.

Therefore the semantic projection identity should be separable from its result carrier.

---

# 22. Projection Identity

P1 should define projection identity abstractly as:

```text
Projection(
    subject,
    member identity,
    member arguments,
    qualification context where applicable
)
```

Source spelling is not identity.

For a domain-native member:

```text
Projection(
    (Int, String),
    TupleDomain::Map,
    [Option]
)
```

For a trait-associated member:

```text
Projection(
    T,
    Iterator::Item,
    [],
    Iterator evidence/qualification context
)
```

These may share a generic projection concept while using different normalization authorities.

---

# 23. Symbolic Projection

A well-formed projection may be irreducible in the current context.

Examples:

```phalcom
T::At<I>
P::Map<F>
R::Map<F>
```

when some subject/argument remains abstract.

P1 should require:

```text
well-formed + insufficient reduction information
    => symbolic/stuck projection
```

not:

```text
Dynamic
Never
Unknown(NoTypeEvidence)
```

and not an eager diagnostic.

This is essential for generic type checking.

---

# 24. Invalid Projection Versus Stuck Projection

P1 should define the semantic distinction:

```text
Reduced
Stuck
Invalid
```

### Reduced

Enough information exists and the operation yields a canonical result.

### Stuck

The operation is well-formed but depends on unresolved symbolic inputs.

### Invalid

Enough information exists to prove that the operation violates its contract.

Examples:

```phalcom
(Int, String)::At<1>
    Reduced -> String
```

```phalcom
T::At<I>
    Stuck
```

when `T` is known to support `At`, but shape/index is not concrete.

```phalcom
(Int, String)::At<100>
    Invalid
```

```phalcom
(Int, String)::Map<Int>
    Invalid
```

because the argument kind is wrong.

Implementation states such as:

```text
Cancelled
BudgetExceeded
Cycle
```

are not language types.

They may exist later in the normalizer API, but they must not be silently interpreted as `Dynamic` or successful type reduction.

---

# 25. Member Lookup and Applicability

P1 must define enough lookup semantics to avoid accidental name-based behavior.

Recommended rules:

1. A type member has declaration/domain identity.
2. A member is available only when the subject’s semantic domain establishes applicability.
3. A generic `T: Type` does not gain every structural-domain member.
4. A `P: TypePack` may use TypePack-domain members.
5. An `R: RecordRow` may use RecordRow-domain members.
6. A structural Tuple proper type may use Tuple-domain members.
7. A structural Record proper type may use Record-domain members.
8. Trait-associated members remain trait-owned and evidence-sensitive.
9. Ambiguous trait-associated names require qualification under existing rules.
10. Domain-native members should not be shadowed or selected merely by matching spelling in an unrelated trait.

Example:

```phalcom
T::Map<Option>
```

where only:

```text
T : Type
```

is known should not succeed automatically.

By contrast:

```phalcom
P::Map<Option>
```

with:

```text
P : TypePack
```

is well-formed.

---

# 26. Name Collision and Qualification

P1 should explicitly prevent type-member coherence from devolving into “inherent wins by name.”

Suppose:

```text
Tuple domain has Map
Trait Transform has associated type Map
```

The written spelling:

```phalcom
T::Map
```

must resolve according to semantic applicability/qualification, not name coincidence.

Recommended principles:

```text
domain-native member identity
    distinct from trait-associated identity

trait qualification
    remains explicit when trait ambiguity exists

member spelling
    is presentation/lookup input
    not canonical identity
```

P1 does not need to specify every parser lookup detail, but it must specify that ambiguity is not resolved by arbitrary precedence.

---

# 27. Type-Level Subscript and `At<I>`

P1 should ratify:

```text
T[I] ≡ T::At<I>
```

where `≡` means one semantic projection identity.

This is stronger than:

```text
normalize(T[I]) == normalize(T::At<I>)
```

The compiler should not have two independent declaration identities whose reductions merely happen to agree.

---

# 28. Canonical Source Declaration Shape for Type Subscript

P1 should close the source-model question even though P2 will write grammar.

There are two coherent surfaces.

## Option A — named declaration, subscript sugar

```phalcom
native type At<I: Int>: Type
```

and:

```phalcom
T[I]
```

desugars to:

```phalcom
T::At<I>
```

## Option B — subscript-shaped declaration with canonical named alias

```phalcom
native type [I: Int]: Type
```

and the type-member semantic identity has the standardized presentation alias:

```text
At
```

so users may write:

```phalcom
T::At<I>
```

### Recommendation

Prefer **Option B** for the structural Tuple domain if Phalcom wants value-level/type-level operation-shape unification.

Rationale:

- the core operation is indexing;
- the declaration surface mirrors subscript behavior;
- `At` becomes the canonical named projection spelling rather than a separately declared member;
- it avoids pretending the bracket syntax is merely arbitrary parser sugar.

However, P1 must state clearly:

> There is still exactly one semantic type-member identity.

The subscript-shaped member must **not** be modeled through callable `SelectorKind::SubscriptGet`; type-member identity is a separate semantic category even if the surface shape is analogous.

If the project prefers simpler implementation/source grammar, Option A remains coherent. The non-negotiable requirement is one identity.

---

# 29. Tuple `At<I>` Domain

Initial domain:

```text
I :const Int
```

P1 should not implicitly import all runtime Tuple key forms.

Runtime Tuple supports:

```text
Int
Symbol
Range
```

The initial type operation should support only compile-time Int indexing.

This keeps the type-level feature bounded.

Symbol-based type lookup and type-level slicing can be introduced separately if desired.

---

# 30. Negative Tuple Type Indexing

Recommended:

```text
normalizeIndex(i, N) =
    i       if i >= 0
    N + i   if i < 0
```

valid iff:

```text
0 <= normalizeIndex(i, N) < N
```

Examples:

```phalcom
(Int, String)::At<0>   == Int
(Int, String)::At<1>   == String
(Int, String)::At<-1>  == String
(Int, String)::At<-2>  == Int
```

invalid:

```phalcom
(Int, String)::At<2>
(Int, String)::At<-3>
```

This aligns static type indexing with runtime Tuple integer indexing.

---

# 31. Tuple Linear Indexing Across Lanes

For:

```phalcom
(Int, String, name: Symbol, enabled: Bool)
```

P1 should define:

```text
0 -> Int
1 -> String
2 -> Symbol
3 -> Bool
```

The labels do not remove entries from the linear index space.

Formally:

```text
linear(Tuple(P, L))
    =
P ++ eraseLabels(L)
```

and:

```text
At(T, i)
    =
linear(T)[normalizeIndex(i)]
```

when valid.

This should match runtime Tuple component order.

---

# 32. `Positionals`

For:

```text
T = Tuple(P, L)
```

define:

```text
T::Positionals -> P
```

Result domain:

```text
TypePack
```

Examples:

```phalcom
(Int, String, name: Symbol)::Positionals
```

reduces to:

```text
[Int, String]
```

and:

```phalcom
()::Positionals
```

reduces to the empty pack.

---

# 33. `Labeled`

For:

```text
T = Tuple(P, L)
```

define:

```text
T::Labeled -> L
```

Result domain:

```text
LabeledTypePack
```

Example:

```phalcom
(Int, String, name: Symbol, enabled: Bool)::Labeled
```

reduces to:

```text
[(name, Symbol), (enabled, Bool)]
```

Order is preserved.

For:

```phalcom
()::Labeled
```

the result is the empty labeled pack.

---

# 34. `Map<F>` — General Semantic Contract

`Map<F>` should be defined as:

> Apply a unary type constructor to each payload proper type while preserving the structural shape domain and all non-payload structural identity.

Required parameter:

```text
F : Type -> Type
```

This operation is shape-preserving, not a general arbitrary type function.

---

# 35. `TypePack::Map<F>`

Given:

```text
P = [T₀, T₁, ..., Tₙ]
```

define:

```text
MapPack(P, F)
    =
[F<T₀>, F<T₁>, ..., F<Tₙ>]
```

with:

```text
MapPack(P, F) : TypePack
```

Order is unchanged.

Empty pack law:

```text
MapPack([], F) = []
```

---

# 36. `LabeledTypePack::Map<F>`

Given:

```text
L = [(ℓ₀,T₀), ..., (ℓₙ,Tₙ)]
```

define:

```text
MapLabeledPack(L, F)
    =
[(ℓ₀,F<T₀>), ..., (ℓₙ,F<Tₙ>)]
```

with:

```text
MapLabeledPack(L, F) : LabeledTypePack
```

Labels and order are unchanged.

---

# 37. Tuple `Map<F>`

For:

```text
T = Tuple(P, L)
```

define:

```text
MapTuple(T, F)
    =
Tuple(
    MapPack(P, F),
    MapLabeledPack(L, F)
)
```

Examples:

```phalcom
(Int, String)::Map<Option>
```

reduces to:

```phalcom
(Option<Int>, Option<String>)
```

and:

```phalcom
(Int, name: String)::Map<Option>
```

reduces to:

```phalcom
(Option<Int>, name: Option<String>)
```

Required preservation:

```text
arity
positional/labeled partition
label sequence
label identity
component order
```

---

# 38. Tuple `Map` / `At` Commutation Law

For every valid concrete `I`:

```text
(T::Map<F>)::At<I>
    ≡
F<T::At<I>>
```

This is an important semantic law.

Example:

```phalcom
(Int, String)::Map<Option>::At<1>
```

and:

```phalcom
Option<(Int, String)::At<1>>
```

both reduce to:

```phalcom
Option<String>
```

This law should become part of P1’s formal contract.

---

# 39. Zero Tuple `Map`

Because:

```text
() ≡ Unit
```

and the zero product has no payload positions:

```phalcom
()::Map<F>
```

must reduce to:

```phalcom
()
```

for every valid:

```text
F : Type -> Type
```

---

# 40. `RecordRow::Map<F>`

For a closed row:

```text
R = { l₀:T₀, ..., lₙ:Tₙ }
```

define:

```text
MapRow(R, F)
    =
{ l₀:F<T₀>, ..., lₙ:F<Tₙ> }
```

Result:

```text
RecordRow
```

The key set is preserved.

Because row identity is order-independent:

```text
MapRow({a:A,b:B}, F)
==
MapRow({b:B,a:A}, F)
```

after canonical row equivalence.

---

# 41. Open RecordRow `Map<F>`

For:

```text
R = { l₀:T₀, ..., lₙ:Tₙ | Tail }
```

define:

```text
MapRow(R, F)
    =
{
    l₀:F<T₀>,
    ...,
    lₙ:F<Tₙ>
    |
    MapRow(Tail, F)
}
```

This rule is mandatory.

The following is wrong:

```text
{
    l₀:F<T₀>,
    ...,
    lₙ:F<Tₙ>
    |
    Tail
}
```

because it leaves unknown fields untransformed.

This P1 law is what later forces C2 to support stable symbolic row transforms.

---

# 42. Record Proper-Type `Map<F>`

For:

```text
T = Record(R)
```

define:

```text
T::Map<F>
    =
Record(MapRow(R, F))
```

Example:

```phalcom
#{name: String, age: Int}::Map<Option>
```

reduces to:

```phalcom
#{name: Option<String>, age: Option<Int>}
```

The Record’s structural key set is preserved.

---

# 43. Empty Record `Map<F>`

Under the recommended static/runtime distinction:

```text
TypeOf(#{}) = Record({})
```

define:

```phalcom
#{}::Map<F>
```

as:

```phalcom
#{}
```

statically.

Its runtime representation may still be Unit when materialized.

This law should be explicit because otherwise a compiler could incorrectly return the Tuple/Unit zero type from Record mapping merely due to runtime representation convergence.

---

# 44. Shape-Preservation Law

P1 should define a domain-specific shape function.

Conceptually:

```text
shape(TypePack)
    ordered arity

shape(LabeledTypePack)
    ordered label sequence

shape(RecordRow)
    key set + openness

shape(Tuple)
    positional arity + ordered labels

shape(Record)
    key set + openness
```

Then:

```text
shape(S::Map<F>)
    =
shape(S)
```

for every supported structural subject `S`.

The meaning of “shape” respects each domain’s identity rules.

---

# 45. Abstract Mapping Must Remain Symbolic

Generic code requires:

```phalcom
P::Map<Option>
```

where:

```text
P : TypePack
```

before P is concrete.

Likewise:

```phalcom
R::Map<Option>
```

where:

```text
R : RecordRow
```

must remain meaningful.

Therefore P1 should require stable semantic forms corresponding to:

```text
MapPack(P, F)
MapLabeledPack(L, F)
MapRow(R, F)
```

without prescribing C2’s arena representation.

A language model in which packs/rows can only exist when fully concrete is insufficient.

---

# 46. Symbolic Tuple Formation From Packs

Variadic Tuple typing eventually requires a type constructor-like structural formation:

```text
Tuple(P, L)
```

even when P or L is abstract.

P1 should require the semantic operation:

```text
TupleForm(P, L) : Type
```

subject to:

```text
P : TypePack
L : LabeledTypePack
```

When both become concrete, normalization should return the existing canonical structural Tuple type:

```text
TupleData(...)
```

or `Unit` if both are empty.

P1 should explicitly prohibit a second concrete identity such as:

```text
Nominal Tuple<P,L>
```

for the same fully concrete tuple shape.

---

# 47. Symbolic Record Formation

Similarly:

```text
Record(R) : Type
```

must remain meaningful when R is open or symbolic.

The existing row system already supports open row tails.

LANG007 mapping extends the symbolic term space but should preserve one canonical concrete Record identity when the row becomes concrete.

---

# 48. Closed Type-Level Calculus Boundary

P1 should define LANG007 as a **closed structural calculus**.

Required semantic operations include only the bounded set necessary for the feature family, initially:

```text
parameter
explicit pack/row
structural formation
projection
Map
At
Positionals
Labeled
pack expansion/concatenation only when separately specified
```

This does not authorize:

```text
arbitrary recursive user type functions
open rewrite families
arbitrary compile-time method calls
compile-time I/O
compile-time mutation
general dependent pattern matching
arbitrary const arithmetic
type-level loops
```

P1 should explicitly say:

> Adding a compiler-native type operation later extends the closed operation set; it does not implicitly create an open user-programmable type-family language.

---

# 49. Compile-Time Constant Term Scope

P1 needs only a minimal const-term language for:

```phalcom
At<0>
At<-1>
At<I>
```

Required initial terms:

```text
integer literal
compile-time const parameter reference
canonical signed integer literal / unary-negated integer literal
```

Potentially allowed later:

```text
const expressions
arithmetic
symbols
booleans
enum cases
```

but they are not required now.

P1 should explicitly reject arbitrary runtime expressions as type-member const arguments.

Example:

```phalcom
let i = computeIndex()
type X = T::At<i> // invalid unless i is a compile-time term under a future const system
```

---

# 50. Heterogeneous Generic Argument Scope

A full semantic model may include:

```text
TypeForm argument
RecordRow argument
TypePack argument
LabeledTypePack argument
Const argument
```

but P1 should not automatically require every generic declaration in Phalcom to accept every domain immediately.

Recommended scope:

> LANG007’s first required heterogeneous generic argument surface is parameterized type-member application.

Required:

```phalcom
T::At<1>
T::Map<Option>
```

Not required by P1:

```phalcom
class Matrix<T, N: Int>
trait Sized<N: Int>
enum Vector<T, N: Int>
```

This decision avoids turning LANG007 into general const-generics implementation.

The semantic parameter-domain model should nevertheless remain general enough that later programs can extend nominal/trait generic applications without redesigning the underlying theory.

---

# 51. Trait Generic Associated Types Are Not a P1 Requirement

P1 should explicitly prevent this scope leak.

Required now:

```text
parameterized domain-native type member
```

Not required now:

```phalcom
trait X {
    type Family<T>
}
```

or:

```phalcom
trait Indexed {
    type At<I: Int>
}
```

The general type-member theory should be compatible with a future GAT feature, but P1 completion must not depend on specifying GAT coherence, binding, specialization, defaults, or metadata.

---

# 52. Union Semantics

P1 must explicitly decide whether domain-native structural type operations distribute across unions.

Recommended initial rule:

> No implicit structural projection distribution over unions.

Therefore:

```phalcom
((Int, String) | (Bool, Symbol))::At<0>
```

does not automatically become:

```phalcom
Int | Bool
```

and:

```phalcom
(#{a: Int} | #{a: String})::Map<Option>
```

does not automatically distribute unless a later feature explicitly specifies it.

Reasons:

- `Positionals` would require union-of-packs semantics;
- per-branch invalidity rules become nontrivial;
- row openness interactions become broader;
- normalization cost can multiply;
- implicit distribution would silently broaden the calculus.

A later specification may add distributive operations in a controlled way.

---

# 53. Type-Member Projection Through `Dynamic`

P1 should also avoid accidental gradual-typing behavior.

Recommended rule:

```text
Dynamic
```

does not statically prove applicability of:

```text
At
Map
Positionals
Labeled
```

unless a separate dynamic type-member operation is designed.

LANG007 operations are static structural computations.

They should not fall back to runtime dispatch merely because the subject is `Dynamic`.

---

# 54. Native Type Members

P1 should generalize `native` semantically to:

> The declaration’s implementation is supplied by the host Phalcom implementation through the mechanism appropriate to that declaration category.

Thus:

```text
native callable behavior
    host runtime/VM/native callable implementation

native type member
    host semantic/compiler normalization implementation
```

This is compatible with the broader Universe concept that “native” can mean VM/compiler/runtime-assisted implementation.

---

# 55. `native` Versus `@intrinsic`

P1 should preserve the orthogonality:

```text
native
    implementation origin

@intrinsic
    builtin metadata / stable intrinsic association
```

`@intrinsic` should not itself mean:

```text
execute compiler code
```

A native type member may use an intrinsic identity to bind a canonical source declaration to a semantic evaluator.

The exact registry mechanism is C3.

The semantic distinction is P1.

---

# 56. Native Type Members Must Not Execute the VM

P1 should define a hard boundary:

> Type-member normalization executes in the semantic/type system and must not depend on runtime VM dispatch.

Forbidden implementation semantics:

```text
materialize a runtime Tuple
send a message
call native runtime selector
push a VM frame
inspect runtime heap state
perform user code
```

Required semantics:

```text
pure/bounded compiler-side normalization
over canonical semantic terms
```

This ensures:

- deterministic type checking;
- safe memoization;
- incremental reproducibility;
- no runtime dependency for compilation;
- no effectful type normalization.

---

# 57. Structural Members Do Not Authorize Structural `impl`

Current:

```text
anonymous structural Tuple and Record types are not legal direct inherent impl targets
```

P1 should preserve this.

Therefore LANG007 must not imply:

```phalcom
impl (Int, String) {
    type Foo = ...
}
```

or:

```phalcom
impl #{name: String} {
    ...
}
```

becomes legal.

Domain-native type members are compiler-defined structural operations, not user extension points.

A future structural extension mechanism would be a separate coherence feature.

---

# 58. Runtime Reflection Boundary

P1 should state:

```text
TypePack
LabeledTypePack
RecordRow transformation terms
Const terms
symbolic projections
```

are semantic compiler objects.

They do not automatically become runtime values/classes.

Therefore P1 does **not** imply:

```phalcom
TypePack.new(...)
RecordRow.class
somePack.is(TypePack)
```

or other runtime object APIs.

Later reflection may expose descriptors, but that is a separate design.

This prevents value/type-level unification from being misinterpreted as “every compiler semantic node is immediately a runtime heap object.”

---

# 59. Phalcom Type-Level / Value-Level Unification Constraint

Phalcom’s design goal of type/value conceptual unification should be interpreted carefully.

P1 should support coherent surface analogies:

```text
value Tuple indexing
    tuple[index]

type Tuple indexing
    TupleType[index]

value-level member syntax
    value.member

type-level member syntax
    TypeForm::Member
```

but must preserve semantic category distinctions:

```text
runtime value
proper static type
type constructor
kind/domain
compile-time constant term
```

Unification means the language model should be compositional and explainable, not that these categories collapse into one runtime representation.

---

# 60. Type-Level Equality Notation

P1 should distinguish meta-level semantic equivalence from source syntax.

Use:

```text
≡
```

in the specification for definitional/canonical semantic identity or reduction equivalence.

Use source:

```phalcom
==
```

only when discussing Phalcom’s type-equivalence constraint syntax.

Thus:

```text
T[I] ≡ T::At<I>
```

means same semantic operation.

A source constraint might later be:

```phalcom
where T[I] == String
```

if the grammar permits.

The spec should not conflate the two notations.

---

# 61. Required Formal Laws

P1 should ratify a concise set of laws.

## 61.1 Domain laws

```text
Γ ⊢ T : Type
Γ ⊢ P : TypePack
Γ ⊢ L : LabeledTypePack
Γ ⊢ R : RecordRow
Γ ⊢ c :const τ
```

Distinct domains do not implicitly coerce into each other.

## 61.2 Pack formation

```text
Γ ⊢ T₀ : Type ... Γ ⊢ Tₙ : Type
----------------------------------
Γ ⊢ [T₀,...,Tₙ] : TypePack
```

## 61.3 Labeled pack formation

```text
Γ ⊢ T₀ : Type ... Γ ⊢ Tₙ : Type
labels ℓᵢ unique
----------------------------------
Γ ⊢ [(ℓ₀,T₀),...,(ℓₙ,Tₙ)] : LabeledTypePack
```

## 61.4 Tuple formation

```text
Γ ⊢ P : TypePack
Γ ⊢ L : LabeledTypePack
--------------------------------
Γ ⊢ Tuple(P,L) : Type
```

with:

```text
Tuple([],[]) ≡ Unit
```

## 61.5 Record formation

```text
Γ ⊢ R : RecordRow
-----------------
Γ ⊢ Record(R) : Type
```

No P1 static law should equate `Record({})` with `Unit` unless explicitly reconsidered.

## 61.6 Constructor mapping

```text
Γ ⊢ F : Type -> Type
Γ ⊢ T : Type
---------------------
Γ ⊢ F<T> : Type
```

## 61.7 Pack map

```text
MapPack([T₀,...,Tₙ],F)
=
[F<T₀>,...,F<Tₙ>]
```

## 61.8 Labeled pack map

```text
MapLabeledPack([(ℓᵢ,Tᵢ)]ᵢ,F)
=
[(ℓᵢ,F<Tᵢ>)]ᵢ
```

## 61.9 Row map

```text
MapRow({ℓᵢ:Tᵢ}ᵢ,F)
=
{ℓᵢ:F<Tᵢ>}ᵢ
```

## 61.10 Open row map

```text
MapRow({fields | R},F)
=
{map(fields,F) | MapRow(R,F)}
```

## 61.11 Tuple map

```text
MapTuple(Tuple(P,L),F)
=
Tuple(MapPack(P,F),MapLabeledPack(L,F))
```

## 61.12 Record map

```text
MapRecord(Record(R),F)
=
Record(MapRow(R,F))
```

## 61.13 Tuple subscript identity

```text
T[I] ≡ T::At<I>
```

## 61.14 Tuple index normalization

```text
j =
    i      if i >= 0
    N + i  if i < 0
```

valid iff:

```text
0 <= j < N
```

## 61.15 Map/index law

```text
(T::Map<F>)::At<I>
≡
F<T::At<I>>
```

for valid I.

## 61.16 Empty map laws

```text
[]::Map<F> ≡ []
()::Map<F> ≡ ()
#{}::Map<F> ≡ #{}
```

with `#{}` remaining statically Record-shaped under the recommended rule.

---

# 62. Recommended Durable Invariants

P1 should produce invariant names that P2 and later implementation plans can cite.

## `TL-DOMAIN-SEPARATION`

> `Type`, `RecordRow`, `TypePack`, and `LabeledTypePack` are distinct type-level domains.

## `TL-CONST-NOT-KIND`

> A compile-time value parameter typed `Int` inhabits a const/value parameter domain; `Int` does not become a kind.

## `TL-TYPEFORM-PROPER-SEPARATION`

> A form of kind `Type -> Type` is a type constructor/type form, not a proper type until saturated to kind `Type`.

## `TL-PACK-PROPER-ELEMENTS`

> Every TypePack and LabeledTypePack payload element is a proper type.

## `TL-LABELED-PACK-ORDER`

> LabeledTypePack identity preserves label sequence order.

## `TL-ROW-ORDER-INDEPENDENT`

> RecordRow identity is key/type based and independent of source/presentation order.

## `TL-PACK-ROW-DISTINCT`

> LabeledTypePack and RecordRow are not interchangeable domains.

## `TM-IDENTITY`

> Type-member identity is semantic declaration/domain-member identity, not member-name spelling.

## `TM-RESULT-DOMAIN`

> A type member declares a result domain; projection results are not restricted to proper `Type`.

## `TM-SYMBOLIC-PROJECTION`

> A well-formed non-reducible type-member projection remains symbolic.

## `TM-STUCK-INVALID-SEPARATION`

> Lack of reduction information and concrete invalidity are distinct semantic states.

## `TM-DOMAIN-APPLICABILITY`

> A domain-native type member is available only when its subject is known to inhabit the member’s semantic domain.

## `TM-NO-NAME-PRECEDENCE`

> Type-member collisions are not resolved by arbitrary spelling precedence.

## `STRUCT-TUPLE-LANE-ORDER`

> Tuple structural identity consists of a positional prefix followed by an ordered labeled suffix.

## `STRUCT-TUPLE-LABEL-UNIQUE`

> Tuple structural labels are unique.

## `STRUCT-TUPLE-ZERO`

> The zero-arity Tuple/product type is `Unit`.

## `STRUCT-RECORD-STATIC-RUNTIME-SEPARATION`

> Runtime zero-product representation of an empty Record does not by itself collapse its static Record type identity into Unit.

## `STRUCT-AT-IDENTITY`

> `T[I]` and `T::At<I>` denote the same type-member projection.

## `STRUCT-AT-INT`

> Initial type-level Tuple indexing accepts compile-time Int arguments.

## `STRUCT-AT-BOUNDS`

> A concrete out-of-range Tuple type index is statically invalid.

## `STRUCT-MAP-KIND`

> `Map<F>` requires `F : Type -> Type`.

## `STRUCT-MAP-SHAPE`

> Map preserves the subject structural shape and transforms only payload types.

## `STRUCT-MAP-OPEN`

> Map transforms symbolic open remainders instead of leaving them unchanged.

## `STRUCT-MAP-SYMBOLIC`

> Mapping an abstract pack or row is a valid symbolic type-level term.

## `STRUCT-MAP-INDEX`

> Tuple Map commutes with valid Tuple At projection.

## `NATIVE-TYPE-NO-VM`

> Native type-member normalization is semantic/compiler-side and does not invoke the runtime VM.

## `NATIVE-INTRINSIC-ORTHOGONAL`

> `native` denotes implementation origin; `@intrinsic` denotes builtin metadata/identity.

## `LANG007-CLOSED-CALCULUS`

> LANG007 structural type operations do not introduce an open user-defined type-family language.

## `LANG007-NO-STRUCTURAL-IMPL-EXPANSION`

> Domain-native type members do not make anonymous structural Tuple/Record types legal ordinary inherent-`impl` targets.

---

# 63. P1 Decision Register

P1 should close each item below explicitly.

| ID | Decision | Recommended P1 result | Why it must be decided in P1 |
|---|---|---|---|
| `P1-D01` | Is `TypePack` a proper type? | No; distinct kind/domain. | Determines all later representation/kinding. |
| `P1-D02` | Is `LabeledTypePack` distinct? | Yes; ordered domain. | Tuple labels require order-sensitive identity. |
| `P1-D03` | Can `RecordRow` represent Tuple labels? | No. | Rows are order-insensitive. |
| `P1-D04` | What does `I: Int` mean? | `Const(Int)` parameter domain. | Prevents `Int` becoming a kind. |
| `P1-D05` | Meaning of `:` | Parameter/domain classification in binder context. | Reconciles kind and const binders. |
| `P1-D06` | What is `0` in `At<0>`? | Compile-time Int term. | Avoids singleton-type confusion. |
| `P1-D07` | What is `Option` in `Map<Option>`? | Type form of kind `Type -> Type`. | Required for HKT coherence. |
| `P1-D08` | What may Map consume? | Any form of kind `Type -> Type`. | Avoids nominal hard-coding. |
| `P1-D09` | General type member category? | Yes. | Existing associated types are too narrow. |
| `P1-D10` | Parameterized type members? | Yes for domain-native members. | Required by At/Map. |
| `P1-D11` | Generic associated types now? | No; deferred. | Prevents scope explosion. |
| `P1-D12` | Non-Type member results? | Yes. | Positionals/Labeled require packs. |
| `P1-D13` | Projection can remain symbolic? | Yes. | Generic code requires it. |
| `P1-D14` | Stuck vs invalid? | Distinct. | Required for sound diagnostics/normalization. |
| `P1-D15` | One []/At identity? | Yes. | Prevents duplicate semantics. |
| `P1-D16` | Canonical declaration spelling for At | Prefer `type [I: Int]: Type` + named alias `At`. | Supports operation-shape unification. |
| `P1-D17` | Initial At argument domain | Compile-time Int only. | Bounded feature surface. |
| `P1-D18` | Negative index | Yes. | Aligns runtime Tuple semantics. |
| `P1-D19` | Labeled lane participates in numeric indexing | Yes, after positional lane. | Matches Tuple linear order. |
| `P1-D20` | Tuple labels unique | Yes. | Existing product semantics and lookup require it. |
| `P1-D21` | `()` static identity | Unit. | Already true in semantic store. |
| `P1-D22` | `#{}` static identity | Empty Record, runtime may be Unit. | Preserves current static/runtime distinction. |
| `P1-D23` | Map is structural shape transform | Yes. | Shared tuple/record algebra. |
| `P1-D24` | Open-row tail mapping | Tail maps symbolically. | Required for correctness. |
| `P1-D25` | Map abstract packs/rows | Valid symbolic term. | Required for generic code. |
| `P1-D26` | Union distribution | No implicit distribution initially. | Controls complexity. |
| `P1-D27` | Generic `T: Type` has Map? | No. | Prevents type-level duck typing. |
| `P1-D28` | `P: TypePack` has Map? | Yes, as domain operation. | Coherent generic surface. |
| `P1-D29` | Runtime TypePack class/value | Not introduced. | Preserve compiler/runtime boundary. |
| `P1-D30` | Native type implementation site | Semantic/compiler host. | No VM dependency. |
| `P1-D31` | `native` vs `@intrinsic` | Orthogonal. | Preserve LANG006 declaration semantics. |
| `P1-D32` | General user type functions | Not introduced. | Closed calculus. |
| `P1-D33` | General nominal const generics | Not required in first LANG007 scope. | Limits blast radius. |
| `P1-D34` | Anonymous structural inherent impl | Remains prohibited. | Preserve coherence. |
| `P1-D35` | Map fusion | Not required. | Avoid constructor-composition scope. |
| `P1-D36` | Symbol/Range type indexing | Deferred. | Int indexing sufficient. |
| `P1-D37` | Arbitrary const arithmetic | Deferred. | Not required for At. |
| `P1-D38` | Domain member/name collision | Ambiguity/qualification, no arbitrary precedence. | Coherence. |

No P1 implementation plan should be written until this decision register is either accepted or replaced with explicit alternatives.

---

# 64. Specification Impact P1 Must Hand to P2

P1 does not itself need to perform all authoritative spec edits, but it must produce precise semantic deltas for P2.

## 64.1 New specification: `type-level-domains.md`

P1 must provide enough settled content for P2 to specify:

```text
Type
RecordRow
TypePack
LabeledTypePack
Arrow kinds
proper type
type constructor/type form
compile-time const parameter domain
generic parameter classification
```

## 64.2 New specification: `type-members.md`

P1 must settle:

```text
member categories
member identity
parameter domains
result domains
projection identity
symbolic projection
applicability
qualification
domain-native members
native/alias/requirement distinction
```

without requiring GATs to become legal.

## 64.3 New specification: `structural-products.md`

P1 must settle:

```text
Tuple static identity
Record static identity
zero Tuple / Unit
empty Record static/runtime distinction
Tuple lanes
label uniqueness
Positionals
Labeled
At
negative indexing
Map
open row Map
symbolic pack/row Map
no union distribution
```

## 64.4 `generic-constraints.md`

P1 must hand P2 the exact correction to:

```text
":" = only kind classification
```

The revised wording should preserve conformance/subtyping/equivalence distinctions.

## 64.5 `associated-types.md`

P1 should hand P2 a **narrow ownership refactor**, not a surprise GAT expansion:

- delegate common type-member/projection terminology to `type-members.md`;
- preserve trait-associated requirement/binding/evidence semantics;
- keep current associated declarations proper-Type-valued and non-parameterized for this release unless separately changed.

## 64.6 `source-declarations.md`

P1 must hand P2 the generalized native rule:

```text
host implementation appropriate to declaration category
```

with native type-member semantics explicitly compiler-side.

## 64.7 `objects/impl.md`

P1 must hand P2 the explicit distinction:

```text
domain-native structural type member
!= ordinary inherent impl on anonymous structural type
```

## 64.8 `objects/traits.md`

Only delegation/consistency is needed if P1 keeps GATs out of scope.

## 64.9 `universe.md`

P1 must specify that canonical Universe source declarations may anchor compiler-native type members without creating a second semantic ownership database.

## 64.10 `objects/values.md`

Only cross-reference changes should be required.

## 64.11 VM specs

P1 should require compatibility review, not language-rule relocation.

---

# 65. Diagnostic Semantics P1 Must Define

P1 should define error classes semantically even though exact diagnostic codes are later.

## 65.1 Wrong member argument domain

Example:

```phalcom
(Int, String)::Map<Int>
```

Expected:

```text
Type -> Type
```

actual:

```text
Type
```

## 65.2 Non-compile-time At argument

Example:

```phalcom
T::At<runtimeIndex>
```

Error category:

```text
type-member const argument is not compile-time
```

## 65.3 At index wrong const type

Example:

```phalcom
T::At<"1">
```

if String const terms later exist.

Expected:

```text
Const(Int)
```

## 65.4 Concrete bounds error

```phalcom
(Int, String)::At<2>
```

is invalid, not stuck.

## 65.5 Member unavailable for domain

```phalcom
T::Map<Option>
```

where:

```text
T : Type
```

and no stronger structural domain is known.

## 65.6 Ambiguous member identity

If multiple trait-associated candidates are visible without qualification, use existing qualification principles rather than arbitrary selection.

## 65.7 Duplicate Tuple labels

A structural Tuple type with duplicate labels is invalid.

## 65.8 Invalid constructor application during Map

If:

```text
F : Type -> Type
```

is established, each `F<T>` should be well-kinded by ordinary type application.

Any further application invalidity must surface through ordinary type-formation semantics rather than a Map-specific secret rule.

---

# 66. Positive Semantic Matrix

P1 should be able to classify every case below.

| Expression | Classification |
|---|---|
| `Int` | proper type, kind `Type` |
| `Option` | type form, kind `Type -> Type` |
| `Option<Int>` | proper type |
| `0` in `At<0>` | compile-time Int term |
| `P` where `P: TypePack` | TypePack term |
| `R` where `R: RecordRow` | RecordRow term |
| `(Int, String)::Positionals` | `TypePack` |
| `(Int, name: String)::Labeled` | `LabeledTypePack` |
| `(Int, String)::At<1>` | `String` |
| `(Int, String)[1]` | same projection as `At<1>` |
| `(Int, String)::At<-1>` | `String` |
| `(Int, name: String)::At<1>` | `String` |
| `(Int, String)::Map<Option>` | `(Option<Int>, Option<String>)` |
| `(Int, name: String)::Map<Option>` | `(Option<Int>, name: Option<String>)` |
| `()::Map<Option>` | `()` / Unit |
| `#{}::Map<Option>` | empty Record static type |
| `#{name: String}::Map<Option>` | `#{name: Option<String>}` |
| `P::Map<Option>` | symbolic TypePack map |
| `R::Map<Option>` | symbolic RecordRow map |

---

# 67. Negative / Boundary Matrix

| Expression / condition | Required P1 result |
|---|---|
| `(Int, String)::Map<Int>` | invalid constructor kind |
| `(Int, String)::At<2>` | concrete bounds error |
| `(Int, String)::At<-3>` | concrete bounds error |
| `T::Map<Option>` with only `T: Type` | member not established/applicable |
| `T::At<I>` where T’s At applicability is not established | member applicability unresolved/error according to lookup context; do not invent Dynamic |
| `At<runtimeValue>` | invalid non-compile-time argument |
| `Map` over open row with unchanged tail | semantically incorrect implementation |
| Tuple labeled lane represented as RecordRow | semantic violation |
| pack represented as ordinary Tuple proper type | semantic violation |
| non-Type projection forced into TypeData-like proper type | semantic violation |
| `native type` executed by VM call | semantic violation |
| implicit union distribution | out of initial scope |
| user-defined recursive type family | out of initial scope |
| structural Tuple `impl` introduced merely for Map | semantic violation |
| trait GAT support inferred from domain-native Map | out of initial scope |

---

# 68. P1 Acceptance Requirements

P1 should not be considered complete merely because a prose design document exists.

The semantic contract must satisfy all requirements below.

## 68.1 Domain completeness

- [ ] `TypePack` defined.
- [ ] `LabeledTypePack` defined.
- [ ] `RecordRow` relation defined.
- [ ] kind grammar defined.
- [ ] const/value parameter domain defined.
- [ ] proper type versus constructor distinction defined.
- [ ] parameter classification syntax semantics defined.

## 68.2 Structural product completeness

- [ ] Tuple lane model defined.
- [ ] Tuple label uniqueness defined.
- [ ] Tuple label order identity defined.
- [ ] Record key-set identity defined.
- [ ] zero Tuple / Unit law defined.
- [ ] empty Record static/runtime distinction defined.
- [ ] symbolic Tuple-from-packs formation defined semantically.

## 68.3 Type-member completeness

- [ ] general type-member category defined.
- [ ] domain-native member defined.
- [ ] associated-type boundary defined.
- [ ] member parameters defined.
- [ ] result domains defined.
- [ ] projection identity defined.
- [ ] lookup/applicability rule defined.
- [ ] collision/qualification rule defined.

## 68.4 `At` completeness

- [ ] Int const domain defined.
- [ ] subscript/At identity defined.
- [ ] canonical declaration/presentation choice decided.
- [ ] negative indexing decided.
- [ ] positional+labeled linear order defined.
- [ ] concrete invalid index behavior defined.
- [ ] symbolic index behavior defined.

## 68.5 `Map` completeness

- [ ] constructor kind requirement defined.
- [ ] TypePack mapping defined.
- [ ] LabeledTypePack mapping defined.
- [ ] Tuple mapping defined.
- [ ] RecordRow mapping defined.
- [ ] open-row mapping defined.
- [ ] Record mapping defined.
- [ ] empty Tuple mapping defined.
- [ ] empty Record mapping defined.
- [ ] Map/At law defined.
- [ ] abstract map symbolic semantics defined.

## 68.6 Boundary completeness

- [ ] union distribution explicitly dispositioned.
- [ ] GAT scope explicitly dispositioned.
- [ ] general const generics explicitly dispositioned.
- [ ] arbitrary const evaluation explicitly dispositioned.
- [ ] general type families explicitly dispositioned.
- [ ] runtime TypePack objecthood explicitly dispositioned.
- [ ] structural inherent impl explicitly dispositioned.
- [ ] VM execution explicitly prohibited for type normalization.

## 68.7 Spec-handoff completeness

- [ ] every P1 rule has a target owning spec for P2;
- [ ] no semantic concept has two competing proposed owners;
- [ ] all current authoritative conflicting statements are enumerated;
- [ ] P2 can edit specs without reopening P1 decisions.

---

# 69. Required P1 Evidence Artifacts

Without prescribing the implementation plan, P1 should ultimately leave durable evidence equivalent to:

```text
formal semantic contract
decision register
invariant register
positive/negative semantic matrix
current-spec conflict matrix
P2 amendment handoff
deferred-feature register
```

These may live in one or several documents depending on repository convention.

The important requirement is that P1 decisions become durable and citable.

---

# 70. Current-Spec Conflict Matrix P1 Must Resolve

| Current statement | P1 conflict | Required semantic resolution |
|---|---|---|
| `generic-constraints.md`: `:` means kind classification | `I: Int` uses a proper type | Generalize binder classification to parameter domains or explicitly separate binder semantics. |
| `associated-types.md`: associated declarations denote proper types | LANG007 general type members may return packs/rows | Move general member result-domain rules to `type-members.md`; preserve associated-type restrictions for current trait feature. |
| `associated-types.md`: generic associated types excluded | parameterized domain-native members needed | Do not misclassify domain-native members as GATs; GATs remain deferred. |
| `source-declarations.md`: native implementation supplied by native runtime | native type member is compiler-side | Generalize implementation-origin rule by declaration category. |
| `objects/impl.md`: structural Tuple/Record not inherent impl targets | Tuple/Record need type operations | Introduce domain-native members; preserve impl prohibition. |
| `objects/values.md`: structural typing excluded | no language owner currently | Create `structural-products.md`. |
| no current language-level kind/domain owner in spec index | TypePack/const domains need ownership | Create `type-level-domains.md`. |
| `::` documented mainly as associated-type projection | domain-native type members also use `::` | Generalize type-member projection syntax ownership while preserving trait qualification. |

---

# 71. Feasibility Constraints for C2/C3

P1 should remain representation-independent but should reject semantic contracts that would require obviously pathological architecture.

The chosen model should permit C2/C3 to implement:

```text
interned stable pack/row terms
domain-neutral projection identity
domain-specific result carriers
query-local solver variables
ordinary kind application for F<T>
pure normalization
memoization
bounded cycle detection
```

It should **not** require:

```text
VM execution
runtime reflection
global backtracking
arbitrary term rewriting
general theorem proving
```

This is part of P1’s architecture-soundness requirement.

---

# 72. Performance Requirements at Semantic Level

P1 does not own exact algorithms, but the semantic model should permit:

```text
concrete At
    O(1) after shape access

concrete pack/row Map
    O(n)

symbolic Map construction
    O(1) or O(size of explicit prefix)

pack equality after interning
    O(1) identity fast path

row key lookup
    logarithmic/constant according to canonical representation

normalization
    bounded and memoizable
```

P1 should reject semantics that inherently require exponential pack splitting or unbounded user rewrite search.

This is one reason no open type families or implicit union distribution should be introduced now.

---

# 73. Solver Boundary Requirements

Although solver implementation is C5, P1 decisions constrain it.

For packs:

```text
ordered sequence unification/matching
```

For rows:

```text
key-oriented row constraints
```

These must remain distinct.

P1 should explicitly allow shared solver **architecture** while forbidding semantic merger.

A future generic sequence solver may serve:

```text
TypePack
LabeledTypePack
```

with different element identity rules.

`RecordRow` remains a row solver domain.

---

# 74. Metadata Boundary Requirements

Metadata is C6, but P1 should require that all stable semantic terms have a conceptual durable identity independent of query-local inference variables.

Therefore:

```text
stable pack parameter
stable Map projection
stable At projection
stable const parameter
```

must be semantically serializable in principle.

P1 should reject a model in which symbolic structural operations exist only as ephemeral inference variables.

---

# 75. Incrementality Boundary Requirements

P1 should require semantic identity to exclude irrelevant source presentation where appropriate.

Examples:

```text
moving a source span
    does not change type-member identity

renaming a local member parameter
    does not change member identity

changing Map member parameter domain
    does change semantic contract

changing Tuple label order
    does change Tuple type identity

reordering Record fields
    does not change Record type identity
```

Exact fingerprinting is later implementation work.

---

# 76. Reflection Boundary Requirements

P1 should preserve enough semantic information that later reflection can choose to expose:

```text
type member declaration
member parameter domains
member result domain
structural Tuple components
Record fields
normalized projection
```

without requiring reflection to expose raw compiler arena IDs.

P1 should not decide the runtime reflection API.

---

# 77. Source Ergonomics Requirements

P1 should preserve the user-facing coherence that motivated LANG007.

The following should feel like one language model:

```phalcom
(Int, String)[1]
(Int, String)::At<1>

(Int, String)::Map<Option>
#{name: String}::Map<Option>

P::Map<Option>
R::Map<Option>
```

The explanation should be:

```text
[] / At
    structural type-member projection

Map
    shape-preserving domain operation

Option
    higher-kinded type constructor

1
    compile-time Int argument
```

not:

```text
special compiler syntax in five unrelated cases
```

---

# 78. Explicit P1 Non-Goals

P1 should record the following as out of scope unless separately promoted.

```text
general nominal const generics
generic associated types
higher-kinded associated result families
user-defined type functions
open type families
arbitrary const evaluation
const arithmetic solver
symbol-keyed Tuple type indexing
Range/slice Tuple type indexing
union-distributed structural projections
runtime TypePack values/classes
public reflection API for packs
VM bytecodes for type operations
structural user-defined impls
Map fusion through constructor composition
Zip/Filter/Fold type operations
type-level loops
dependent pattern matching
```

Recording these exclusions prevents later plans from reading omission as permission.

---

# 79. Risks if P1 Is Under-Specified

## 79.1 `TypeId` becomes universal by accident

If P1 does not distinguish domains, later implementation may encode packs/consts as fake TypeIds.

Consequence:

```text
kind unsoundness
projection result confusion
metadata hacks
```

## 79.2 Tuple labeled packs reuse RecordRow

Consequence:

```text
label order disappears
distinct Tuple types collapse
runtime/static identity mismatch
```

## 79.3 `I: Int` is called a kind

Consequence:

```text
proper value types and kinds collapse conceptually
generic application becomes incoherent
```

## 79.4 Map implemented only for concrete shapes

Consequence:

```text
generic P::Map<F> impossible
open R::Map<F> incorrect
```

## 79.5 AssociatedProjection is generalized by force

If non-Type domain projections are crammed into current TypeData-associated projections:

```text
pack terms masquerade as proper types
kind checks lie
```

P1 must require a domain-neutral projection concept.

## 79.6 Tuple/Record runtime classes become static structural owners

Consequence:

```text
Unit zero-product exception becomes awkward
anonymous structural type identity becomes nominalized
impl ownership becomes confused
```

P1 should keep source anchors distinct from semantic applicability.

## 79.7 General GATs pulled in accidentally

Consequence:

```text
trait binding/coherence/meta surface explodes
LANG007 scope becomes much larger than needed
```

## 79.8 Empty Record collapses statically to Unit accidentally

Consequence:

```text
Record structural operations lose category information
current TypeData::Record(empty) model conflicts
```

Runtime zero-product optimization should not dictate static identity.

## 79.9 Implicit union distribution

Consequence:

```text
union-of-pack semantics
branch failure semantics
normalization blowup
```

## 79.10 `native type` routed through runtime

Consequence:

```text
effectful/non-deterministic type checking
VM dependency during semantic analysis
poor caching
```

---

# 80. P1 Completion Gate

`LANG007.C1.P1` should be considered **complete for P2 planning** only when:

1. every decision in the P1 decision register has an explicit accepted result;
2. the formal domain grammar is settled;
3. the const/value parameter model is settled;
4. the type-member taxonomy is settled;
5. domain-native member ownership is settled;
6. associated-type scope is explicitly bounded;
7. Tuple/Record static structural semantics are settled;
8. empty Tuple and empty Record semantics are separately settled;
9. At/subscript identity is settled;
10. At index semantics are settled;
11. Map semantics are settled for all required domains;
12. open-row Map is settled;
13. symbolic projection semantics are settled;
14. invalid/stuck distinction is settled;
15. union behavior is settled;
16. native/intrinsic semantics are settled;
17. no-VM normalization boundary is settled;
18. non-goals are explicit;
19. every current spec conflict has a P2 target resolution;
20. no unresolved semantic question is left to C2/C3 representation work.

The lifecycle state after P1 should still be:

```text
LANG007.C1
    specification convergence in progress

LANG007 feature implementation
    not started
```

P1 completion is **semantic closure**, not feature implementation.

---

# 81. Recommended P1 Outcome Summary

If the recommendations in this analysis are ratified, P1 should hand P2 the following concise contract.

```text
1. Phalcom type-level domains include:
       Type
       RecordRow
       TypePack
       LabeledTypePack
       Arrow kinds

2. Generic parameters have domains:
       Kind(κ)
       Const(τ)

3. I: Int means:
       I is a compile-time value parameter typed Int.
   Int is not a kind.

4. TypePack is an ordered sequence of proper types.

5. LabeledTypePack is an ordered sequence of unique labels + proper types.

6. RecordRow is an order-insensitive key/type structural domain.

7. Tuple(P,L) is a proper structural type.
   Tuple([],[]) is Unit.

8. Record(R) is a proper structural type.
   Empty Record may use Unit runtime representation without losing static Record identity.

9. Phalcom has general type members.
   LANG007 requires parameterized domain-native members.
   Existing trait associated types need not become GATs.

10. Type members have explicit result domains.

11. Projections can remain symbolic.
    Stuck != invalid.

12. T[I] and T::At<I> are one semantic operation.

13. Tuple At initially accepts compile-time Int.
    Negative indexes are supported.
    Numeric order is positional lane then labeled lane.

14. Map<F> requires F : Type -> Type.

15. Map preserves shape and transforms payload types.

16. Tuple Map maps both positional and labeled payloads.

17. Record Map maps all fields and symbolically maps open tails.

18. TypePack/LabeledTypePack/RecordRow Map preserve their own domains.

19. Generic P::Map<F> and R::Map<F> are valid symbolic operations.

20. Structural operations do not implicitly distribute over unions.

21. native type members normalize in compiler/semantic code.
    They never execute the VM.

22. native and @intrinsic are orthogonal.

23. LANG007 does not introduce arbitrary user type families,
    runtime TypePack objects, structural user impls,
    or language-wide const generics by implication.
```

That is the semantic foundation later LANG007 checkpoints should implement.

---

# Appendix A — Repository Evidence Reviewed

## Current authoritative specifications

```text
docs/specs/README.md
docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/source-declarations.md
docs/specs/objects/values.md
docs/specs/objects/impl.md
docs/specs/objects/traits.md
docs/specs/universe.md
docs/specs/callables/arguments.md
docs/specs/callables/selectors.md
docs/specs/vm/products.md
docs/specs/vm/values-and-objects.md
```

## Current semantic implementation

```text
phalcom-semantic/src/types/kind.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/row.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/projection.rs
phalcom-type-meta/src/type_node.rs
```

## Current Universe/runtime surfaces

```text
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/collections/record.ph
```

## Workflow / planning exemplars

```text
docs/workflow/implementation-record-lifecycle-convention.md
docs/implementation/LANG005/LANG005.C3/LANG005.C3.P1-requirements-analysis.md
docs/implementation/LANG005/LANG005.C6/LANG005.C6.P4-requirements-analysis.md
```

---

# Appendix B — Formal Notation

## Kinds

```text
κ ::= Type
    | RecordRow
    | TypePack
    | LabeledTypePack
    | (κ₁,...,κₙ) -> κ
```

## Parameter domains

```text
δ ::= Kind(κ)
    | Const(τ)
```

## Kinding

```text
Γ ⊢ A : κ
```

## Compile-time value typing

```text
Γ ⊢ c :const τ
```

## Pack formation

```text
[T₀,...,Tₙ] : TypePack
```

## Labeled pack formation

```text
[(ℓ₀,T₀),...,(ℓₙ,Tₙ)] : LabeledTypePack
```

## Tuple

```text
Tuple(P,L) : Type
```

## Record

```text
Record(R) : Type
```

## Map

```text
MapPack      : TypePack × (Type -> Type) -> TypePack
MapLabeled   : LabeledTypePack × (Type -> Type) -> LabeledTypePack
MapRow       : RecordRow × (Type -> Type) -> RecordRow
MapTuple     : Type × (Type -> Type) -> Type       // restricted by Tuple-domain applicability
MapRecord    : Type × (Type -> Type) -> Type       // restricted by Record-domain applicability
```

## At

```text
AtTuple : TupleShape × Const(Int) -> Type
```

The actual source-level member signature is domain-native rather than a public first-class function.

---

# Appendix C — Example Reduction Set

```text
(Int, String)::Positionals
    -> [Int, String]

(Int, String, name: Symbol)::Labeled
    -> [(name, Symbol)]

(Int, String)[1]
    ≡ (Int, String)::At<1>
    -> String

(Int, String)::At<-1>
    -> String

(Int, name: String)::At<1>
    -> String

(Int, String)::Map<Option>
    -> (Option<Int>, Option<String>)

(Int, name: String)::Map<Option>
    -> (Option<Int>, name: Option<String>)

()::Map<Option>
    -> ()

#{name: String, age: Int}::Map<Option>
    -> #{name: Option<String>, age: Option<Int>}

#{}::Map<Option>
    -> #{}

P::Map<Option>
    -> MapPack(P, Option)        // symbolic

R::Map<Option>
    -> MapRow(R, Option)         // symbolic
```

---

# Appendix D — P1-to-P2 Specification Handoff Table

| P1 semantic result | P2 owning specification |
|---|---|
| kind/domain grammar | `type-level-domains.md` |
| proper type vs type form | `type-level-domains.md` |
| compile-time const domain | `type-level-domains.md` |
| `I: Int` binder semantics | `type-level-domains.md` + `generic-constraints.md` |
| TypePack | `type-level-domains.md` |
| LabeledTypePack | `type-level-domains.md` |
| type member taxonomy | `type-members.md` |
| domain-native member | `type-members.md` |
| projection identity/stuck/invalid | `type-members.md` |
| associated-type delegation | `associated-types.md` |
| Tuple structural type | `structural-products.md` |
| Record structural type | `structural-products.md` |
| Unit/empty Tuple | `structural-products.md` |
| empty Record static/runtime distinction | `structural-products.md` with VM cross-reference |
| Positionals/Labeled | `structural-products.md` |
| At/subscript | `structural-products.md` + `type-members.md` |
| Map | `structural-products.md` |
| native type member | `source-declarations.md` + `type-members.md` |
| Universe source anchor | `universe.md` |
| structural impl boundary | `objects/impl.md` |
| global ownership index | `README.md` |

---

# Appendix E — Questions P1 Must Not Leave Open

Before P1 is marked complete, the answer to each question must be explicit.

```text
Is TypePack a kind?
Is LabeledTypePack a kind?
Are pack elements proper types only?
Are Tuple labels unique?
Does label order matter?
Is a labeled pack a row?
What is I: Int?
Is Int a kind?
What is 0 in At<0>?
What is Option in Map<Option>?
Can Map accept a type lambda?
Can a type member return TypePack?
Can a projection remain symbolic?
What distinguishes stuck from invalid?
Who owns Tuple::Map?
Does Tuple::Map imply structural impl?
Can T: Type call Map?
Can P: TypePack call Map?
Does [] equal At semantically?
Are negative At indexes supported?
Do numeric indexes include labeled Tuple entries?
What is () statically?
What is #{} statically?
What does #{} materialize as?
Does Map transform open row tails?
Does Map distribute over unions?
Does Dynamic allow runtime Map/At?
Does native type invoke the VM?
Is @intrinsic execution semantics?
Are trait GATs part of LANG007?
Are general nominal const generics part of LANG007?
Are arbitrary const expressions part of LANG007?
Are user-defined type families part of LANG007?
```

If any of these remains undecided, P1 has not achieved decision closure.
