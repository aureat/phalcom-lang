# LANG007.C3.P4 Requirements Analysis — Intrinsic Structural Data, Tuple/Record Semantic Completion, and Structural Literal Convergence

**Status:** Requirements analysis  
**Program:** LANG007  
**Checkpoint:** C3 — unified algebraic/static domains and intrinsic structural families  
**Target plan:** LANG007.C3.P4  
**Repository basis:** `aureat/phalcom-lang` at `71e77727f510fe1f693c084a833e2d5d5ea65745` (`docs(lang): close C3 migration audit gaps`)  
**Predecessor implementation:** C3.P3 plus the corrective semantic/runtime-projection closure at `82c9dd0382ece4932de4a52e98fdd36452b122c8`  
**Document purpose:** Define the complete semantic, syntax, compiler, runtime, metadata, tooling, migration, and verification requirements for C3.P4 before a patch-grade implementation plan is written.

---

## 1. Executive summary

C3.P4 completes the intrinsic structural-data model that C3.P3 exposed but did not yet make semantically self-consistent.

The target model is no longer that bare `Tuple` is “really a runtime class” while `Tuple<...>` is intercepted by a special intrinsic application path, nor that `Record` is a nominal-looking facade over a separately named `RecordRow` domain. Instead, `Tuple` and `Record` are canonical source declarations with independently meaningful semantic projections:

- a **type-family projection** used in static/type formation;
- a **runtime-class projection** used for VM behavior and `.class`;
- one **associated/static surface** used by `::` lookup and intrinsic type operations;
- source declaration identity used by metadata, reflection, navigation, and incremental compilation.

Their canonical source declarations become ordinary-looking structural product declarations whose exceptional behavior is concentrated in `intrinsic`, not in special syntax:

```phalcom
intrinsic data Tuple<***Es: CompleteTypePack>(***Es)

intrinsic data Record<**Fs: RecordFields> {
    **Fs
}
```

The same typed product-shape expansion syntax is available to user-defined nominal `data` declarations:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)

data MyRecord<**Fs: RecordFields> {
    **Fs
}
```

The difference is type-family application semantics, not grammar:

- `MyTuple<Int, String>` remains a nominal `MyTuple` specialization;
- `Tuple<Int, String>` normalizes to the canonical structural tuple type `(Int, String)`;
- `MyRecord<...>` remains nominal;
- `Record<...>` normalizes to the canonical structural Record representation backed by the existing row/field solver.

The source-visible structural kinds are completed and made explicit:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields
```

`RecordFields` is the public source name for the existing record-row semantic domain. The implementation may retain internal names such as `RecordRowId`, `RecordRowData`, `RecordRowTail`, and `RecordRowSolver`; this is a source-vocabulary migration, not a demand to rewrite mathematically appropriate compiler internals.

The public `TupleEntries` facade is removed as a distinct intrinsic domain. `Tuple::Entries` directly returns `CompleteTypePack`. If compatibility requires a temporary source alias, it must be a transparent alias to `CompleteTypePack`, never a second semantic domain.

Expansion topology and semantic kind are explicitly separated. The source glyphs:

```text
single
*
**
***
```

mean:

```text
Single
Positional
Labeled
Complete
```

and do not by themselves determine the operand kind. In particular, `**` may carry either `LabeledTypePack` or `RecordFields` according to the receiving contract. This replaces the current hard-coded equation:

```text
LabeledPack => LabeledTypePack
```

with the more general invariant:

> expansion mode determines how an argument contributes; kind determines what semantic object the argument is.

Literal syntax converges on the ratified product/collection model:

```phalcom
(...)                 // Tuple
{...}                 // Record
[...]                 // List
#{...}                // Set
Map {...}             // Map
Map<K, V> {...}       // explicitly specialized Map
```

Record type/value row expansion uses `**`:

```phalcom
{ name: String, **Fs }

{ name: "Altun", **record }
```

The old structural Record spelling `#{...}` is retired. The old anonymous Map spelling `{...}` is retired. `#{...}` is assigned to Set syntax.

The zero-arity aliases are source declarations, not independent runtime data families:

```phalcom
intrinsic type Unit = Tuple<>
intrinsic type EmptyRecord = Record<>
```

with:

```text
Unit != EmptyRecord
Tuple<> == Unit == ()
Record<> == EmptyRecord == {}
```

All tuple values, including `()`, use one VM `Tuple` runtime class. All record values, including `{}`, use one VM `Record` runtime class. No `EmptyTuple`, `NonEmptyTuple`, `EmptyRecord`, or `NonEmptyRecord` runtime classes are introduced. `Unit` and `EmptyRecord` are exact semantic types/aliases; their runtime classes are respectively `Tuple` and `Record`.

C3.P4 also absorbs the outstanding architectural hardening from the P3 audit:

1. declaration replacement and incremental shell refresh must become explicit operations rather than overloading `DeclarationTypeTable::insert`;
2. runtime lowering must consume the explicit intrinsic runtime-owner projection rather than re-derive VM class identity from source module/name;
3. scoped and canonical Tuple/Record application policy must be consolidated behind one application contract;
4. canonical Universe identity must replace duplicated module/name recognition where practical;
5. broad post-change workspace certification must be run after the corrective architecture and P4 semantic migration are both present.

P4 explicitly stops before any language-level unification between Tuple and Record. There is no mixed destructuring, no public “general product-coordinate” kind, no Record-as-Tuple or Tuple-as-Record conversion model, and no user-visible generalized structural-product protocol in this checkpoint.

---

## 2. Lifecycle note: current repository plan-number collision

The repository currently already contains a completed document named:

```text
LANG007.C3.P4-intrinsic-semantic-application-runtime-owner-corrective-closure.md
```

That work is the bounded correction that separated intrinsic semantic application from runtime/native owner identity after P3 exposed the conflict. It is already implemented at commit:

```text
82c9dd0382ece4932de4a52e98fdd36452b122c8
```

The new work described by this requirements analysis is materially broader and is the intended feature-completion P4.

Before writing the implementation plan, the lifecycle should be normalized in one of two ways:

1. **Preferred:** reclassify the existing corrective closure as `LANG007.C3.P3.R1` (or an equivalent P3 corrective record) and reserve `LANG007.C3.P4` for the intrinsic structural-data completion described here.
2. **If implementation-record identifiers are immutable:** retain the existing P4 identifier and number the new work `C3.P5`.

This document uses **C3.P4** for the new work because that is the intended checkpoint structure established in this design session.

No source implementation should depend on the documentation renumbering; this is a lifecycle/history correction only.

---

## 3. Current baseline and P3 audit findings

### 3.1 Current canonical source

At the repository basis, the Universe exposes:

`phalcom-core/core/universe/src/collections/tuple.ph`:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries

intrinsic data Tuple<***Es>

impl Tuple {
  intrinsic type Positionals: TypePack
  intrinsic type Labeled: LabeledTypePack
  intrinsic type Entries: TupleEntries
  intrinsic type At<I: Int>: Type
  intrinsic type Map<F: Type -> Type>: Type
}
```

The runtime behavior surface is also declared in bare `impl Tuple`, and several methods/native primitives erase element types to `Dynamic`:

```phalcom
internal native rawAt(_ index: Int) -> Dynamic
internal native rawPositionals -> List<Dynamic>
internal native rawLabeled -> Map<Symbol, Dynamic>
```

`phalcom-core/core/universe/src/collections/record.ph` currently exposes:

```phalcom
intrinsic type data RecordRow
intrinsic data Record<R: RecordRow>

impl Record {
  internal native rawSize -> Int
  internal native rawLabelAt(_ index: Int) -> Symbol
  internal native rawValueAt(_ index: Int) -> Dynamic
  ...
}
```

`phalcom-core/core/universe/src/option/unit.ph` currently exposes an independent runtime intrinsic declaration:

```phalcom
intrinsic data Unit()

impl Unit {
  toString -> String { "()" }
  hash -> Int { 0 }
}
```

These declarations are adequate as source anchors for P3 but no longer match the ratified P4 semantics.

### 3.2 Current intrinsic facade AST split

`phalcom-ast/src/ast.rs` currently distinguishes:

```rust
pub enum AlgebraicShapeSyntax {
    Product(ProductShapeSyntax),
    Sum(SumShapeSyntax),
    IntrinsicFacade(SourceRange),
}
```

and `phalcom-ast/src/parser.rs::parse_data_inner` creates `IntrinsicFacade` whenever an `intrinsic data`/`intrinsic type data` declaration omits `(...)` or `{...}`.

Consequently:

```phalcom
intrinsic data Tuple<***Es>
```

is represented as an `IntrinsicFacade`, while:

```phalcom
intrinsic data Unit()
```

is represented as a real empty tuple-shaped `ProductShapeSyntax`.

The P4 model rejects that distinction for runtime intrinsic structural data. `intrinsic` is implementation provenance; product syntax must truthfully describe the logical product shape.

### 3.3 Current binder-mode/kind coupling

`phalcom-semantic/src/types/parameter.rs` already documents that `GenericParameterMode` is conceptually independent from kind, but publication validation currently hard-codes:

```rust
GenericParameterMode::PositionalPack => KindId::TYPE_PACK
GenericParameterMode::LabeledPack    => KindId::LABELED_TYPE_PACK
GenericParameterMode::CompletePack   => KindId::COMPLETE_TYPE_PACK
```

Likewise `phalcom-semantic/src/declarations.rs::source_parameter_kind` currently returns a kind directly from source expansion syntax before consulting an explicit kind annotation:

```rust
Some(ExpansionMode::Positional) => KindId::TYPE_PACK
Some(ExpansionMode::Labeled)    => KindId::LABELED_TYPE_PACK
Some(ExpansionMode::Complete)   => KindId::COMPLETE_TYPE_PACK
```

This is the central blocker for:

```phalcom
<**Fs: RecordFields>
```

because `**` is currently semantically hard-wired to `LabeledTypePack`.

### 3.4 Current Record application special case

`phalcom-semantic/src/types/annotation.rs` currently recognizes `IntrinsicTypeApplication::Record` before ordinary generic application and expects exactly one `GenericArgumentSyntax::Single` whose semantic identity is a row parameter or an already formed Record.

That preserves the critical invariant that row identity stays outside `TypeId`, but it does not support the ratified syntax:

```phalcom
Record<**Fs>
```

P4 must generalize this intrinsic application seam to consume a **labeled expansion whose semantic kind is `RecordFields`**, without converting the row/field domain into `LabeledTypePack` or `TypeId`.

### 3.5 Current Tuple application

Tuple application already uses C2's pack machinery correctly:

```text
CompleteTypePack
    -> project positionals
    -> project labeled
    -> TypeStore::tuple_from_packs
    -> concrete Tuple / TupleFromPacks
```

The canonical and scoped paths are nevertheless duplicated in `types/annotation.rs`, and the P3 audit identified this as a drift risk. P4 adds enough new application semantics that the duplication should be removed rather than extended.

### 3.6 Current declaration binding

`phalcom-semantic/src/declarations.rs` currently carries:

```rust
DeclarationTypeBinding {
    semantic_form,
    runtime_owner,
}
```

which successfully fixes the P3 incident by making semantic intrinsic application and runtime owner sibling projections.

However:

- `DeclarationTypeTable::insert` implicitly preserves an existing intrinsic semantic form when the incoming record says `Ordinary`;
- it implicitly preserves an existing runtime owner when the incoming record says `None`;
- `compile_intrinsic_runtime_data` receives the runtime-owner projection through semantic lowering but still looks up the VM class by source module/name.

P4 must preserve the good architectural split while removing these ambiguous update semantics and finishing runtime-owner authority end-to-end.

### 3.7 Current literal syntax

The parser currently uses:

```text
#{...}   Record
{...}    association Map
```

Bare brace block literals have already been removed; `|| { ... }` is the closure spelling. This is important because it means P4 can assign `{...}` to Record expressions without reintroducing an ordinary block/Record ambiguity.

The lexer currently has a dedicated:

```rust
Token::RecordLBrace
```

for contiguous `#{`.

Set AST/checker/bytecode support exists (`Expr::SetLiteral`, `BeginSetLiteral`, `SetLiteralAdd`, `FinishSetLiteral`), but source Set syntax is currently deferred. P4 can therefore reassign `#{...}` to Set and activate the existing downstream Set literal pipeline.

### 3.8 Current structural Record type syntax

Parser tests still use forms such as:

```phalcom
type UserRow<R> = #{ name: String, age: Int, | R }
```

The AST already has `TypeAnnotationExpr::Record { fields, tail, ... }`, and the semantic row solver already preserves open tails. P4 changes only the source grammar and domain spelling:

```phalcom
{ name: String, age: Int, **Fs }
```

while retaining the existing canonical row/field representation underneath.

---

## 4. Ratified P4 endpoint

### 4.1 Canonical source declarations

The minimum canonical Universe endpoint is:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data CompleteTypePack
intrinsic type data RecordFields

intrinsic data Tuple<***Es: CompleteTypePack>(***Es)

intrinsic data Record<**Fs: RecordFields> {
    **Fs
}

intrinsic type Unit = Tuple<>
intrinsic type EmptyRecord = Record<>
```

`Unit` and `EmptyRecord` are canonical type aliases/specializations, not independent runtime data families.

### 4.2 Canonical structural syntax

```phalcom
()                                  // Unit / empty Tuple
(1, "x")                            // Tuple value
(Int, String)                       // Tuple type

{}                                  // EmptyRecord value/type by phase
{ name: "Altun", age: 42 }          // Record value
{ name: String, age: Int }          // Record type
{ name: String, **Fs }              // open/generic Record type
{ name: "Altun", **rest }           // Record value expansion

[]                                  // List
#{1, 2, 3}                          // Set
Map { name: "Altun", age: 42 }      // Map
Map<String, Int> { "a": 1 }         // explicitly specialized Map
```

### 4.3 Structural family laws

```text
Tuple<> == Unit == ()
Record<> == EmptyRecord == {}

Unit != EmptyRecord
```

and:

```text
runtimeClass(Unit)        == Tuple
runtimeClass((Int,String))== Tuple

runtimeClass(EmptyRecord) == Record
runtimeClass({name:String})== Record
```

### 4.4 Public structural kind laws

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordFields
```

are four distinct static domains.

```text
LabeledTypePack != RecordFields
CompleteTypePack != Type
RecordFields != Type
```

No implicit coercion is introduced between them.

### 4.5 Expansion laws

The source expansion modes are:

```text
Single
Positional   (*)
Labeled      (**)
Complete     (***)
```

Expansion mode is not semantic kind.

Valid core combinations include:

| Expansion mode | Semantic kind | Meaning |
|---|---|---|
| `Single` | `Type` | ordinary type argument |
| `Single` | `TypePack` | one argument whose value is a whole pack |
| `Single` | `LabeledTypePack` | one whole labeled pack |
| `Single` | `CompleteTypePack` | one whole complete pack |
| `Single` | `RecordFields` | one whole record-field domain value |
| `*` | `TypePack` | positional lane expansion/capture |
| `**` | `LabeledTypePack` | labeled tuple-pack expansion/capture |
| `**` | `RecordFields` | Record field expansion/capture |
| `***` | `CompleteTypePack` | complete tuple-shape expansion/capture |

Invalid combinations must produce explicit mode/domain diagnostics.

---

## 5. Scope boundaries and non-goals

P4 explicitly does **not** introduce:

- mixed Tuple/Record destructuring;
- Tuple-style destructuring for Records;
- Record-style destructuring for Tuples;
- a public generalized product-coordinate domain;
- a public “ordered product vs unordered product” relationship between Tuple and Record;
- user-visible `ProductShape`, `Coordinate`, `FieldSet`, or analogous generalized protocols;
- `RecordFields {...}` as a first-class source constructor;
- `RecordFields` as a runtime class/value;
- a `RecordFields -> CompleteTypePack` conversion;
- a `CompleteTypePack -> RecordFields` conversion;
- general dependent types;
- arbitrary compile-time execution;
- per-specialization VM Tuple or Record classes;
- `EmptyTuple`, `NonEmptyTuple`, `EmptyRecord`, or `NonEmptyRecord` runtime classes;
- a second row/field solver;
- moving record-field identities into `TypeId`;
- a second associated namespace for compiler intrinsics;
- a broad redesign of user-visible `.type`/`.class` syntax beyond the declaration/runtime-class semantics required here.

The compiler's internal product representation may be generalized in a future checkpoint, and user-defined shapes may eventually participate in a wider structural protocol. P4 intentionally stops before that work.

---

## 6. Declaration semantics: complete the class/type-family/runtime-class model

### 6.1 One declaration, multiple projections

P4 should make the P3 semantic/runtime split part of the general declaration model rather than an intrinsic workaround.

A canonical runtime type declaration has one declaration identity:

```text
DeclarationId
```

and may expose independently typed projections:

```text
type-family projection
runtime-class projection
associated/static surface
reflection/source declaration projection
```

For `Tuple`:

```text
declaration:
    universe.collections.tuple::Tuple

type-family projection:
    CompleteTypePack -> Type
    intrinsic structural reducer

runtime-class projection:
    canonical VM Tuple class

associated surface:
    Tuple::Entries
    Tuple::Positionals
    Tuple::Labeled
    Tuple::Element
    Tuple::At
    Tuple::Map
    ...
```

For `Record`:

```text
declaration:
    universe.collections.record::Record

type-family projection:
    RecordFields -> Type
    intrinsic structural reducer

runtime-class projection:
    canonical VM Record class

associated surface:
    Record::Fields
    Record::Value
    Record::Map
    ...
```

For ordinary user data:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)
```

the type-family projection has the same **kind**:

```text
CompleteTypePack -> Type
```

but uses nominal application rather than intrinsic structural normalization.

This is the desired semantic endpoint: intrinsic declarations are unusual because of their application/runtime implementation authority, not because their source names follow an ad hoc overload rule.

### 6.2 Bare name resolution

P4 should not require every use of a declaration name to collapse immediately to one `TypeId`.

Conceptually:

```text
Tuple
```

resolves first to the Tuple declaration. Context then requests the appropriate projection:

```text
Tuple<...>
    -> type-family projection

Tuple::Entries
    -> associated/static surface

Tuple in runtime value/class-side context
    -> runtime class-object projection
```

The exact internal refactoring can be staged, but P4 must stop encoding “bare Tuple is a proper Type whose application is a special exception” as the normative model.

### 6.3 Intrinsic family kinds

The static family application signatures are:

```text
Tuple  : CompleteTypePack -> Type
Record : RecordFields -> Type
```

This must be represented explicitly as declaration-owned application contracts.

The runtime class projection is independent:

```text
runtimeClassProjection(Tuple)  = VM Tuple class
runtimeClassProjection(Record) = VM Record class
```

---

## 7. Structural kinds and source-visible intrinsic domains

### 7.1 `CompleteTypePack` becomes public

The semantic implementation already has:

```rust
KindData::CompleteTypePack
KindId::COMPLETE_TYPE_PACK
TypeData::CompleteTypePack
```

P4 makes the kind publicly nameable in source:

```phalcom
CompleteTypePack
```

and adds the canonical Universe anchor:

```phalcom
intrinsic type data CompleteTypePack
```

The kind grammar and parser must accept it wherever kind annotations are legal.

### 7.2 `TupleEntries` is retired as a semantic-domain name

Current source:

```phalcom
intrinsic type data TupleEntries
```

currently maps to `KindId::COMPLETE_TYPE_PACK`.

P4 must not retain two public declarations that appear to denote distinct `type data` domains while actually sharing one kind identity.

Canonical endpoint:

```phalcom
intrinsic type Entries: CompleteTypePack
```

`TupleEntries` may only survive as a transitional transparent alias if compatibility policy requires it:

```phalcom
type TupleEntries = CompleteTypePack
```

Such an alias must not own:

- a new `StaticDomainId`;
- a new `KindId`;
- a new `IntrinsicTypeOwner`;
- separate type-member dispatch;
- separate metadata identity.

### 7.3 `RecordRow` becomes public `RecordFields`

Source-facing declarations, kind syntax, diagnostics, hover text, documentation, examples, and formatted source should use:

```text
RecordFields
```

rather than:

```text
RecordRow
```

The internal compiler may retain:

```text
KindId::RECORD_ROW
RecordRowId
RecordRowData
RecordRowTail
RecordRowSolver
RecordRowTerm
RecordRowFailure
...
```

because “row” is still the appropriate implementation/type-theory vocabulary.

The source-to-internal binding becomes:

```text
source RecordFields
    -> KindId::RECORD_ROW
    -> RecordRowId / RecordRowData / RecordRowSolver
```

This avoids a high-risk mechanical internal rename that would not improve semantics.

### 7.4 Intrinsic static domain registration

`phalcom-semantic/src/declarations.rs::intrinsic_static_facade_kind` currently recognizes:

```text
TypePack
LabeledTypePack
TupleEntries
RecordRow
```

by source module/name.

P4 must migrate to:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordFields
```

and should preferably resolve them through canonical `UniverseKey`/declaration identity rather than duplicate path/name recognition.

---

## 8. Expansion-mode semantics

### 8.1 Separate mode from kind in all generic/static parameter products

The current semantic model already stores mode and kind separately, but validation makes most pack modes imply one exact kind.

P4 requires the pair to remain independent:

```text
(parameter kind, expansion mode)
```

Examples:

```text
<**Ls: LabeledTypePack>
    kind = LabeledTypePack
    mode = Labeled

<**Fs: RecordFields>
    kind = RecordFields
    mode = Labeled

<P: TypePack>
    kind = TypePack
    mode = Single

<*P: TypePack>
    kind = TypePack
    mode = Positional
```

The final two forms remain semantically different.

### 8.2 Replace one-to-one mode validation with compatibility validation

`phalcom-semantic/src/types/parameter.rs` must stop using:

```text
PositionalPack -> TYPE_PACK
LabeledPack    -> LABELED_TYPE_PACK
CompletePack   -> COMPLETE_TYPE_PACK
```

as the complete legality model.

Instead it must validate allowed mode/domain pairs.

At minimum:

```text
Positional + TypePack
Labeled    + LabeledTypePack
Labeled    + RecordFields
Complete   + CompleteTypePack
```

`Single` continues to accept any legal explicitly declared domain.

### 8.3 Source defaulting

To preserve C2 ergonomics:

```phalcom
<*Ts>
```

may continue to default to:

```text
TypePack
```

```phalcom
<**Ls>
```

may continue to default to:

```text
LabeledTypePack
```

```phalcom
<***Es>
```

may continue to default to:

```text
CompleteTypePack
```

but an explicit domain annotation overrides the default when the pair is compatible:

```phalcom
<**Fs: RecordFields>
```

The canonical Universe should spell the kinds explicitly even when the default would suffice.

### 8.4 Reflection/public vocabulary

Public reflection should expose neutral expansion vocabulary:

```text
Single
Positional
Labeled
Complete
```

rather than names that imply semantic kind such as `LabeledPack`.

An internal Rust rename of `GenericParameterMode` is desirable but not semantically mandatory if the implementation can expose the correct public vocabulary and remove kind-coupled validation. The implementation plan should decide whether to:

- rename the enum in P4; or
- retain it internally and add a neutral semantic/public `ExpansionMode` mapping.

What is mandatory is the semantic separation, not a cosmetic Rust rename.

### 8.5 Apply the same model to type-member parameters and type lambdas

The mode/domain separation must be shared by:

- class/data generic binders;
- alias generic binders;
- impl binders;
- type-lambda binders;
- intrinsic/source type-member parameters;
- durable metadata.

P4 must not fix only nominal generic declarations while leaving type-member/type-lambda publication on the old one-mode-one-kind rule.

---

## 9. Product-shape expansion in `data` declarations

### 9.1 Why this is required

The canonical intrinsic declarations:

```phalcom
intrinsic data Tuple<***Es: CompleteTypePack>(***Es)

intrinsic data Record<**Fs: RecordFields> {
    **Fs
}
```

must use ordinary product grammar. They must not rely on a compiler-only AST shape.

Therefore P4 must generalize product-shape declaration syntax itself.

### 9.2 Current AST limitation

`phalcom-ast/src/ast.rs` currently models a product shape as:

```rust
ProductShapeSyntax::Tuple {
    components: Vec<DataComponentSyntax>,
    ...
}

ProductShapeSyntax::Record {
    components: Vec<DataComponentSyntax>,
    ...
}
```

and each `DataComponentSyntax` is one fixed component with:

```rust
local_name
external_label
annotation
```

There is no representation for:

```phalcom
***Es
```

inside a tuple-shaped data declaration or:

```phalcom
**Fs
```

inside a record-shaped data declaration.

### 9.3 Required syntax-level generalization

The AST needs a product-component contribution model. Conceptually:

```rust
enum DataProductComponentSyntax {
    Fixed(DataComponentSyntax),
    Expansion {
        mode: ExpansionMode,
        operand: TypeAnnotationOrStaticReference,
        range: SourceRange,
    },
}
```

The exact Rust shape may differ, but the syntax tree must preserve:

- whether the contribution is fixed or expanded;
- the star mode;
- the referenced parameter/static term;
- source range;
- tuple/record shape context.

Do not desugar expansion into synthetic fixed fields during parsing; symbolic shapes must survive semantic analysis.

### 9.4 Tuple-shaped product legality

User source must be allowed to write:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)
```

and more generally product expansions whose placement is statically legal.

At minimum:

```phalcom
data Prefix<*Ts: TypePack>(
    Int,
    *Ts
)
```

is valid.

A complete expansion is terminal with respect to later product components because it may introduce labeled entries:

```phalcom
data Good<***Es: CompleteTypePack>(
    Int,
    ***Es
)
```

is valid, while:

```phalcom
data Bad<***Es: CompleteTypePack>(
    ***Es,
    Int
)
```

must be rejected: `Es` may contain labeled entries, which would make the trailing positional `Int` violate the Tuple positional-before-labeled invariant.

Likewise labeled expansion belongs in the labeled suffix and must preserve label uniqueness.

### 9.5 Record-shaped product legality

User source must be allowed to write:

```phalcom
data MyRecord<**Fs: RecordFields> {
    **Fs
}
```

and:

```phalcom
data Envelope<**Fs: RecordFields> {
    requestId: RequestId
    **Fs
}
```

The logical product shape is:

```text
{ requestId: RequestId } + Fs
```

under a row/field lacks constraint ensuring `Fs` does not duplicate `requestId`.

P4 should route this through existing record-row/lacks machinery rather than invent duplicate-field logic specific to expanded data declarations.

### 9.6 Ordinary user data remains nominal

For:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)
```

the type:

```phalcom
MyTuple<Int, String>
```

remains nominally `MyTuple<Int, String>`.

It does **not** normalize to:

```phalcom
(Int, String)
```

The expansion determines logical product components and constructor/pattern/layout behavior, not type-family identity.

Similarly:

```phalcom
data MyRecord<**Fs: RecordFields> {
    **Fs
}
```

remains a nominal `MyRecord` family.

### 9.7 Intrinsic Tuple/Record use the same syntax but different application binding

The intrinsic source forms:

```phalcom
intrinsic data Tuple<***Es: CompleteTypePack>(***Es)

intrinsic data Record<**Fs: RecordFields> {
    **Fs
}
```

share the ordinary product-shape syntax. Their declaration binding changes type-family application:

```text
ordinary data:
    nominal specialization

intrinsic Tuple:
    CompleteTypePack -> canonical structural Tuple

intrinsic Record:
    RecordFields -> canonical structural Record
```

This is the intended semantic role of `intrinsic`.

### 9.8 Semantic product representation must support symbolic expanded components

`phalcom-semantic/src/checker/algebraic_declaration.rs` currently enumerates a concrete `components_syntax` slice, assigns each item a `DataComponentId`, and constructs fixed constructor parameters/layout information.

P4 must distinguish:

- **source-declared fixed component identity**, which continues to use `DataComponentId`;
- **expanded structural component families**, which cannot be eagerly assigned one source `DataComponentId` per eventual specialized component.

The semantic representation therefore needs a symbolic product-shape contribution or template representation.

The implementation must not synthesize unstable source identities for components that arise only after generic specialization.

### 9.9 Constructor signatures and raw initialization

For:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)
```

the raw constructor/initializer shape is conceptually:

```text
(***Es) -> MyTuple<***Es>
```

For a concrete specialization:

```phalcom
MyTuple<Int, String>
```

the constructor specializes to:

```text
(Int, String) -> MyTuple<Int, String>
```

Likewise record-shaped expanded data derives record initializer fields from `Fs`.

The P4 implementation must reuse the existing generic specialization environment and data constructor authority rather than manufacturing a new constructor declaration per specialization.

### 9.10 Runtime data lowering for user-defined expanded products

`phalcom-core/src/modules/semantic_lowering.rs::DataDeclarationLoweringSpec` and related `DataComponentLoweringSpec` currently assume a fixed source component list.

P4 needs a specialization-aware representation for expanded product shape.

The implementation plan must inspect the existing generic data representation strategy and choose the smallest correct extension. P4 must **not** introduce monomorphization solely to support expanded products unless the existing runtime already specializes generic data that way.

Acceptable strategies include:

- specialization-time layout materialization using existing generic data specialization;
- an erased/descriptor-driven generic layout if already supported by the runtime;
- symbolic semantic product shape with runtime layout fixed by existing generic representation constraints.

The requirements are semantic correctness and stable identity, not any specific layout strategy.

---

## 10. Remove `IntrinsicFacade` as the runtime structural-data shape

For runtime `intrinsic data`, P4 should eliminate the current meaning:

```text
no product body
    => AlgebraicShapeSyntax::IntrinsicFacade
```

Tuple and Record now have explicit product bodies.

`IntrinsicFacade` may remain temporarily for opaque `intrinsic type data` anchors if the compiler still needs a representation for bodiless primitive static domains, but runtime structural declarations must no longer depend on it.

The parser should reject bodiless runtime intrinsic data where an explicit shape is required unless another language feature explicitly defines such a declaration.

This makes:

```text
implementation provenance
```

orthogonal to:

```text
algebraic/product shape
```

which is the desired model.

---

## 11. Tuple semantic completion

### 11.1 Canonical declaration

Target:

```phalcom
intrinsic data Tuple<***Es: CompleteTypePack>(***Es)
```

The declaration's type-family contract is:

```text
CompleteTypePack -> Type
```

and intrinsic application continues to normalize through the existing:

```text
TupleFromPacks
TypeStore::tuple_from_packs
```

authority.

### 11.2 Structural equivalences

P4 must certify directly at source level:

```text
Tuple<> == Unit == ()
Tuple<Int, String> == (Int, String)
Tuple<Int, name: String> == (Int, name: String)
```

Concrete Tuple application must never become nominal:

```text
Applied<Tuple, ...>
```

Symbolic complete packs continue to use:

```text
TupleFromPacks { positionals, labeled }
```

### 11.3 Source-visible associated type surface

Canonical Tuple associated declarations should be source-visible:

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

The exact declaration organization can split type-level and runtime methods into multiple impl blocks, but both must bind the `Es` specialization environment.

Required laws:

```text
Tuple<***Es>::Entries == Es

Tuple<***Es>::Positionals
    == positionals(Es)

Tuple<***Es>::Labeled
    == labeled(Es)
```

`Element` is the canonical union/join of all payload types:

```text
(Int, String, flag: Bool)::Element
    == Int | String | Bool

()::Element
    == Never
```

When `Es` is symbolic and the union cannot reduce, `Element` remains a valid intrinsic projection; it must not degrade to `Dynamic`.

### 11.4 `At<I>` remains exact compile-time projection

Existing semantics remain:

```text
(Int, String)::At<0>  == Int
(Int, String)::At<-1> == String
```

and:

```text
T[I] == T::At<I>
```

for the existing type-subscript equivalence.

### 11.5 `Map<F>` remains shape-preserving

```text
Tuple(P,L)::Map<F>
    == Tuple(Map(P,F), Map(L,F))
```

with the zero case normalizing back to `Unit`.

### 11.6 User-facing methods must be fully typed

The public Tuple API must stop using `Dynamic` simply because the implementation is VM-generic.

The family-specialized surface should use `Element` and exact associated projections where sound.

Representative target:

```phalcom
impl<***Es: CompleteTypePack> Tuple<***Es> {
    size -> Int

    first -> Option<Element>
    last -> Option<Element>

    at(_ index: Int) -> Element
    get(_ index: Int) -> Option<Element>

    positionals -> List<Element>
    labeled -> Map<Symbol, Element>

    iterator -> TupleIterator<***Es>

    toString -> String
    ==(_ other: Object) -> Bool
    !=(_ other: Object) -> Bool
    hash -> Int
}
```

Literal/static-index expressions may be further refined by the compiler through `At<I>`.

Internal native primitives may retain lower-level representation types only when genuinely internal. Prefer source contracts such as:

```phalcom
internal native rawAt(_ index: Int) -> Element
```

even if the Rust/VM implementation physically returns a generic VM `Value`.

The requirement is:

> representation genericity must not force source-level type erasure.

### 11.7 Iterator typing

Current `TupleIterator`/`CanonicalIterable` paths that use `Dynamic` should become structurally parameterized:

```text
TupleIterator<***Es>
Item = Tuple<***Es>::Element
```

The exact source spelling depends on the current iterator-law declarations, but public iterator item typing must preserve the Tuple payload union.

### 11.8 Runtime class

Every Tuple value uses one runtime class:

```text
runtimeClass(())                 = Tuple
runtimeClass((1,))               = Tuple
runtimeClass((1, "x"))           = Tuple
runtimeClass((1, name: "Altun")) = Tuple
```

No runtime `Unit` class remains semantically observable after the migration.

A VM may retain an internal singleton value for the empty tuple and shape descriptors for optimized layouts; neither creates a distinct language-level runtime class.

---

## 12. Unit semantic completion

### 12.1 Canonical declaration

Replace:

```phalcom
intrinsic data Unit()
```

with:

```phalcom
intrinsic type Unit = Tuple<>
```

The exact parser/AST form for `intrinsic type` aliases must be audited. If existing alias syntax does not currently allow `intrinsic`, P4 must add the minimum provenance support needed for compiler-owned canonical aliases rather than preserving Unit as fake runtime data.

### 12.2 Type identity

```text
Unit == Tuple<> == ()
```

There is one canonical proper semantic type.

### 12.3 Runtime identity

```text
runtimeClass(Unit) == Tuple
```

The current `IntrinsicRuntimeOwner::Unit` branch and tests that certify a preinstalled Unit runtime class must be removed or redirected.

### 12.4 Unit methods

Behavior such as:

```phalcom
toString -> String { "()" }
hash -> Int { 0 }
```

must be owned by the Tuple family/runtime behavior in a way that produces the correct zero-arity behavior.

P4 should avoid retaining a separate runtime Unit behavior class solely to host these methods.

The implementation may use:

- ordinary Tuple methods whose zero case naturally returns these results;
- intrinsic zero-arity branches inside shared Tuple behavior.

The public semantic result must remain one runtime Tuple class.

---

## 13. Record semantic completion

### 13.1 Canonical declaration

Target:

```phalcom
intrinsic data Record<**Fs: RecordFields> {
    **Fs
}
```

The type-family contract is:

```text
RecordFields -> Type
```

but the generic application uses labeled expansion syntax:

```phalcom
Record<**Fs>
```

### 13.2 Preserve row/field identity outside `TypeId`

The internal representation remains:

```text
RecordFields source term
    -> RecordRowId
    -> TypeData::Record(RecordRowId)
```

P4 must not teach `TypeLambdaArena` to encode record fields as ordinary `TypeId` arguments.

The existing record-row solver remains the canonical authority for:

- open field tails;
- field unification;
- subtraction;
- lacks constraints;
- duplicate-field rejection;
- query-local row inference.

### 13.3 Record family application

The current Record special-case in `types/annotation.rs` must be generalized from:

```text
one Single argument of row identity
```

to:

```text
one Labeled expansion contribution whose expected semantic domain is RecordFields
```

so:

```phalcom
Record<**Fs>
```

resolves directly to:

```text
TypeData::Record(FsRowId)
```

The operand must never be projected as `LabeledTypePack`.

### 13.4 Structural Record syntax

Record types:

```phalcom
{}
{ name: String }
{ name: String, age: Int }
{ name: String, **Fs }
```

Record values:

```phalcom
{}
{ name: "Altun" }
{ name: "Altun", age: 42 }
{ name: "Altun", **rest }
```

The source order of fields remains presentation/runtime insertion order where relevant, while structural Record type identity remains governed by the existing row semantics.

### 13.5 `**` field expansion

For:

```phalcom
{ name: String, **Fs }
```

the result is a Record whose field domain is explicit field `name` extended by `Fs`.

For:

```phalcom
{ name: "Altun", **record }
```

the spread source must have a statically representable Record type/field domain.

An arbitrary runtime `Map<K,V>` must not be accepted as a structural Record spread merely because both use `**`.

Explicit fields must impose row-lacks constraints against an open spread:

```text
Fs lacks name
```

rather than using Map-style overwrite semantics.

### 13.6 Source-visible associated surface

Canonical source declarations should include:

```phalcom
impl<**Fs: RecordFields> Record<**Fs> {
    intrinsic type Fields: RecordFields
    intrinsic type Value: Type
    intrinsic type Map<F: Type -> Type>: Type
}
```

Required laws:

```text
Record<**Fs>::Fields == Fs
```

`Value` is the canonical union/join of all field payload types:

```text
{ name: String, age: Int }::Value
    == String | Int

{}::Value
    == Never
```

For symbolic/open field sets, the projection remains symbolic when exact reduction is impossible.

### 13.7 `Map<F>` remains row/field preserving

Conceptually:

```text
Record<**Fs>::Map<F>
    == Record<**(Fs::Map<F>)>
```

using the existing record-row map normalizer, including correct treatment of open tails.

### 13.8 User-facing methods must be fully typed

Representative target:

```phalcom
impl<**Fs: RecordFields> Record<**Fs> {
    size -> Int
    labelAt(_ index: Int) -> Symbol
    get(_ key: Symbol) -> Option<Value>

    toString -> String
    ==(_ other: Object) -> Bool
    hash -> Int
}
```

Internal `rawValueAt` should use `Value` as its source-level result contract where sound, rather than exposing `Dynamic`.

### 13.9 Runtime class

Every Record value uses one runtime `Record` class:

```text
runtimeClass({})                    = Record
runtimeClass({name:"Altun"})        = Record
runtimeClass({name:"Altun",age:42}) = Record
```

No empty/non-empty Record subclasses are introduced.

---

## 14. EmptyRecord semantic completion

### 14.1 Canonical declaration

```phalcom
intrinsic type EmptyRecord = Record<>
```

### 14.2 Type identity

```text
EmptyRecord == Record<> == {}
```

### 14.3 Non-equivalence with Unit

```text
Unit != EmptyRecord
```

This must be directly regression-tested.

### 14.4 Runtime identity

```text
runtimeClass(EmptyRecord) == Record
```

The VM may use a singleton empty Record object internally, but it remains an instance of the common Record runtime class.

---

## 15. Literal syntax convergence

### 15.1 Final literal matrix

| Construct | P4 syntax |
|---|---|
| Tuple | `(a, b)` |
| Empty Tuple / Unit | `()` |
| Record | `{ name: value }` |
| Empty Record | `{}` |
| List | `[a, b]` |
| Set | `#{a, b}` |
| Empty Set | `#{}` |
| Map | `Map { key: value }` |
| Empty Map | `Map {}` |
| specialized Map | `Map<K,V> { key: value }` |

### 15.2 Lexer migration

Current `Token::RecordLBrace` represents `#{`.

P4 should rename/redefine this token as a Set opener or another neutral hash-brace token.

The lexer must continue to preserve existing symbol/selector `#...` tokenization rules. The implementation plan must re-run selector-family lexer tests because `#{` participates in the same prefix namespace.

### 15.3 Bare `{...}` becomes Record in expression position

The current parser only accepts bare braces for association Map forms and otherwise reports that bare brace block literals were removed.

P4 should simplify expression parsing:

```text
Token::LBrace
    -> Record literal parser
```

No content inspection should decide between Record and Map.

This removes the current association-based brace classification.

### 15.4 `Map {...}` becomes explicit nominal collection construction

`Map {...}` and `Map<K,V> {...}` must parse through an explicit Map construction path.

The AST may:

- retain `Expr::MapLiteral` while recording the explicit target/source head; or
- introduce `Expr::MapConstruction`.

What is mandatory is that Map construction has an explicit `Map` type-family origin and no longer arises from anonymous brace classification.

Semantic lowering must resolve the canonical Map declaration rather than rely only on the source spelling string.

### 15.5 `#{...}` becomes Set

Set AST/checker/bytecode support already exists but source grammar is deferred.

P4 must connect `#{...}` to `Expr::SetLiteral`.

Existing Set expansion semantics (`*` positional expansion, where already supported by checker/runtime policy) should be preserved or completed according to the Set subsystem's current contract. Record's old `**` expansion grammar must not leak into Set parsing.

### 15.6 Type syntax

Record type parsing must migrate from:

```phalcom
#{ name: String, | R }
```

to:

```phalcom
{ name: String, **Fs }
```

The AST can retain a normalized `tail` field, but the parser must preserve enough source information for diagnostics, indexing, and formatting.

The old `| R` row-tail spelling should be retired from the canonical grammar.

### 15.7 Map and Record expansions remain semantically distinct

Both may use `**` in their own contexts:

```phalcom
{ name: "Altun", **record }

Map { name: "Altun", **mapping }
```

but:

```text
Record ** spread
    -> static structural field composition
    -> row/field lacks constraints
    -> exact resulting RecordFields

Map ** spread
    -> runtime association expansion
    -> Map generic K/V compatibility
```

Do not unify these operations solely because they share a glyph.

---

## 16. Source-visible intrinsic kind operations

P4's source-owned Universe principle requires every **publicly selectable compiler intrinsic structural operation** to have an authored source declaration.

At minimum, current public operations must be represented:

```phalcom
intrinsic type data TypePack
impl TypePack {
    intrinsic type Map<F: Type -> Type>: TypePack
}

intrinsic type data LabeledTypePack
impl LabeledTypePack {
    intrinsic type Map<F: Type -> Type>: LabeledTypePack
}

intrinsic type data CompleteTypePack
impl CompleteTypePack {
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type Map<F: Type -> Type>: CompleteTypePack
}

intrinsic type data RecordFields
impl RecordFields {
    intrinsic type Map<F: Type -> Type>: RecordFields
}
```

P4 does **not** need to expose every internal pack-concatenation or row-subtraction primitive as source members merely because the compiler uses them. The requirement is source visibility for operations that are intentionally part of the public type-level language.

The compiler operation table and source declarations require bidirectional validation:

```text
every public compiler intrinsic operation
    -> exactly one canonical source declaration

every source member marked intrinsic
    -> exactly one recognized compiler implementation
```

This prevents hidden public operations and source declarations with no evaluator.

---

## 17. AssociatedSurface and member identity

P4 must retain the P3 architecture:

```text
TypeMemberDeclarationId(owner, selector)
AssociatedSurface(owner)
```

remains the one canonical static/associated namespace.

No specialization-specific namespace is created for:

```text
Tuple<Int,String>
Record<...>
```

Instead:

```text
owner = canonical Tuple/Record declaration
specialization environment = Es/Fs
```

Source impls:

```phalcom
impl<***Es: CompleteTypePack> Tuple<***Es> { ... }

impl<**Fs: RecordFields> Record<**Fs> { ... }
```

must publish members under the canonical family owner while type-checking signatures under the bound specialization environment.

This is required both for intrinsic associated types and fully typed runtime methods.

---

## 18. Consolidate intrinsic application policy

P4 should introduce one declaration-owned intrinsic application contract rather than continue duplicating Tuple/Record rules in scoped and canonical lowering.

Conceptually:

```text
Tuple:
    binder mode    = Complete
    operand domain = CompleteTypePack
    result kind    = Type
    reducer        = tuple_from_packs

Record:
    binder mode    = Labeled
    operand domain = RecordFields
    result kind    = Type
    reducer        = record_type(row_id)
```

Scoped lowering and canonical lowering may produce different intermediate representations, but they must consult the same policy.

The contract should also provide enough information for impl-target binder inference:

```phalcom
impl<**Fs> Record<**Fs> { ... }
```

can infer/validate `Fs : RecordFields` from the Record family contract rather than hard-coded declaration-name logic.

---

## 19. Declaration table hardening

### 19.1 Split authoritative replace from shell refresh

Current `DeclarationTypeTable::insert` implicitly merges intrinsic projections.

P4 should introduce explicit update semantics, e.g.:

```text
replace_authoritative(info)
refresh_shell_preserving_binding(info)
```

or equivalent.

Required invariant:

```text
Ordinary
```

must mean ordinary semantic binding, not “preserve whatever was there before,” and:

```text
None runtime owner
```

must mean no owner when used in an authoritative replacement, not “preserve old owner.”

Incremental/source-shell refresh is the operation that explicitly requests preservation.

### 19.2 Generalize the binding model

The current:

```rust
DeclarationTypeBinding {
    semantic_form,
    runtime_owner,
}
```

may remain as a compact implementation, but P4 should treat it semantically as:

```text
type-family/static projection
runtime-class projection
```

rather than an intrinsic-only special case.

The implementation plan should determine whether to generalize the Rust data structure now or simply make the semantics explicit while retaining current fields.

### 19.3 Canonical identity

`intrinsic_binding_for_source` and `intrinsic_static_facade_kind` currently duplicate canonical module/name knowledge.

P4 should prefer:

```text
canonical declaration / UniverseKey
    -> binding descriptor
```

where practical.

Do not introduce another registry. Extend the canonical Universe binding table or declaration metadata already used by bootstrap.

---

## 20. Runtime-owner authority

`phalcom-core/src/compiler/lib/data_decl.rs::compile_intrinsic_runtime_data` currently receives an `IntrinsicRuntimeFacadeLoweringSpec` containing the runtime-owner projection but then re-derives the VM class from source module/name.

P4 must make the runtime-owner projection authoritative:

```text
IntrinsicRuntimeOwner::Tuple
    -> canonical VM Tuple class

IntrinsicRuntimeOwner::Record
    -> canonical VM Record class
```

`Unit` is removed from the runtime-owner enum after the Unit alias migration unless a temporary compatibility state is required during implementation.

The compiler should assert source declaration identity agrees with the projected owner, but should not use source spelling as the primary lookup authority.

---

## 21. VM/runtime migration

### 21.1 One Tuple class

Audit all VM/bootstrap code for:

- Unit class installation;
- Tuple class installation;
- `Value::unit()` class behavior;
- `.class` lookup for the unit singleton;
- native Unit method tables;
- native Tuple method tables;
- class identity tests.

After P4:

```text
Value::unit()
    runtime class -> Tuple
```

The unit singleton representation may stay optimized.

### 21.2 One Record class

Audit:

- empty Record allocation;
- Record literal builder finalization;
- Record class lookup;
- shape descriptors;
- equality/hash/toString behavior.

Empty Record may be singleton-optimized but must return runtime class `Record`.

### 21.3 Shape descriptors are not classes

If Tuple/Record runtime shape metadata is required for:

- arity;
- positional count;
- labels;
- record field order;
- lookup;

it should remain descriptor metadata owned by the runtime representation.

P4 must not expose per-shape hidden subclasses as language `.class` identity.

---

## 22. Map construction migration

### 22.1 Map remains an ordinary nominal generic collection

`Map` does not become an `IntrinsicTypeApplication`.

`Map<K,V>` remains an ordinary generic family with its existing runtime class and generic inference.

`Map {...}` is privileged syntax for constructing that nominal collection.

### 22.2 Generic inference

Examples:

```phalcom
Map {
    #alice: 10,
    #bob: 20,
}
```

should infer the normal Map generic arguments according to existing inference/join rules.

```phalcom
Map {}
```

with no expected type or explicit parameters should produce normal generic underconstraint rather than defaulting to `Dynamic`.

### 22.3 Record vs Map semantic distinction

```phalcom
{
    alice: 10,
    bob: 20,
}
```

has exact structural Record type.

```phalcom
Map {
    alice: 10,
    bob: 20,
}
```

has homogeneous nominal Map type.

The compiler must not recover one from the other via contextual guessing.

---

## 23. Set syntax activation

`#{...}` becomes Set syntax.

Required work includes:

- lexer token/comment rename from Record opener to Set/hash-brace opener;
- parser construction of `Expr::SetLiteral`;
- parser tests for empty, singleton, multi-element, trailing comma, expansion, malformed forms;
- semantic checker tests for element inference and expected-type propagation;
- compiler/bytecode tests exercising `BeginSetLiteral`, `SetLiteralAdd`, `FinishSetLiteral`;
- source index and advisory traversal coverage;
- LSP semantic token updates;
- migration of any current `#{...}` Record fixtures to `{...}`.

Because the AST/runtime already contain substantial Set support, P4 should reuse these paths rather than invent a second Set representation.

---

## 24. Record syntax migration breadth

The `#{...}` -> `{...}` migration affects more than expression parsing.

The implementation plan must inventory and migrate:

- AST/parser unit tests;
- semantic fixtures;
- compiler integration fixtures;
- collection/product specs;
- examples;
- documentation;
- formatter/printer if present;
- LSP semantic token tests;
- source-index fixtures;
- diagnostic snapshots;
- golden programs;
- stdlib/Universe source;
- any parser error strings mentioning `RecordLBrace` or `#{`;
- any source generators in tests;
- any repository scripts with embedded `.ph` snippets.

The migration must be mechanical where semantics are unchanged.

---

## 25. Type metadata and schema requirements

### 25.1 Kind node naming

Stable metadata currently contains `KindNode::RecordRow`.

P4 has two choices:

1. retain the stable/internal metadata enum name `RecordRow` while formatting/public source renders `RecordFields`; or
2. version metadata and rename the serialized public node.

The lower-risk approach is to retain the internal/stable row representation unless external metadata consumers treat enum variant names as public source vocabulary.

The implementation plan must inspect schema compatibility guarantees before choosing.

### 25.2 CompleteTypePack source round-trip

P4 requires source forms such as:

```phalcom
type Foo<***Es: CompleteTypePack> = Tuple<***Es>
```

to round-trip through metadata with:

```text
kind = CompleteTypePack
mode = Complete
```

No `TupleEntries` surrogate is permitted.

### 25.3 Expansion mode and kind are serialized independently

Durable metadata must distinguish:

```text
Labeled + LabeledTypePack
```

from:

```text
Labeled + RecordFields
```

Any validator that currently reconstructs or enforces kind solely from mode must be generalized.

### 25.4 Static argument envelope

The heterogeneous static-argument representation must explicitly support the complete set of semantic categories needed by P4:

```text
Type/static term
TypePack
LabeledTypePack
CompleteTypePack
RecordFields/RecordRowId
Const
static-domain terms
```

Internal representation may continue to use `TypeId` for non-row kinded terms and `RecordRowId` for the row/field domain, provided category information is never lost.

### 25.5 Interface fingerprints

Fingerprints must change when any of these change:

- generic parameter kind;
- expansion mode;
- intrinsic type-family application binding;
- runtime owner;
- source-visible intrinsic member signature;
- RecordFields vs old source facade identity;
- Unit/EmptyRecord alias target;
- product-shape expansion template.

Cold and incremental builds must converge.

---

## 26. Source index, LSP, navigation, and reflection

### 26.1 Canonical source anchors

Go-to-definition/hover/completion for:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordFields
Tuple
Record
Unit
EmptyRecord
Tuple::Entries
Tuple::Element
Record::Fields
Record::Value
...
```

must point to canonical Universe source declarations, not synthetic Rust-only builtins.

### 26.2 Intrinsic member validation

Source index publication must include all intrinsic associated types and parameterized family impl members.

Current source-index tests use the old facade syntax and `Dynamic`; these fixtures need direct migration to the final production source shape.

### 26.3 Runtime class reflection

Where runtime reflection exposes class identity:

```text
().class          -> Tuple
{}.class          -> Record
```

must be testable.

Exact semantic type reflection must remain distinct:

```text
().type           -> Unit
{}.type           -> EmptyRecord / {}
```

The exact `.type` descriptor API is outside P4, but P4 must not preserve VM class assumptions that make the intended distinction impossible.

---

## 27. Diagnostics and migration behavior

P4 should provide explicit diagnostics for obsolete syntax.

### 27.1 Old Record literal/type syntax

Old:

```phalcom
#{ name: value }
```

now parses as Set syntax and should not silently be interpreted as Record.

Where the token sequence clearly resembles an old Record association, a migration-oriented parser/type diagnostic is desirable:

```text
structural Records use `{ name: value }`; `#{...}` is Set syntax
```

Do not make this diagnostic depend on executing semantic Set logic if the parser can identify the legacy shape safely.

### 27.2 Old anonymous Map syntax

Bare:

```phalcom
{ key: value }
```

is now Record.

Where an expected `Map<...>` context makes the mismatch obvious, the diagnostic should suggest:

```phalcom
Map { key: value }
```

The parser should not retain content-based anonymous Map classification for compatibility.

### 27.3 Old row syntax

Old:

```phalcom
#{ name: String, | R }
```

should receive a targeted migration diagnostic toward:

```phalcom
{ name: String, **Fs }
```

where feasible.

### 27.4 `RecordRow`

Source uses of:

```phalcom
R: RecordRow
```

should migrate to:

```phalcom
Fs: RecordFields
```

Whether `RecordRow` remains as a temporary deprecated source alias is a compatibility decision. The clean P4 endpoint has one public name: `RecordFields`.

### 27.5 `TupleEntries`

Likewise, source `TupleEntries` should either:

- be rejected with a migration diagnostic to `CompleteTypePack`; or
- exist only as a deprecated transparent alias during a bounded transition.

It must not retain intrinsic-domain identity.

---

## 28. Key repository hotspots

The following files/subsystems are known current change points from the P3 audit and live repository inspection.

| Area | Current path | P4 concern |
|---|---|---|
| Algebraic AST | `phalcom-ast/src/ast.rs` | `IntrinsicFacade`; fixed-only `DataComponentSyntax`; Record/Map/Set literal comments/shapes |
| Lexer | `phalcom-ast/src/lexer.rs`, `token.rs` | `#{` currently `RecordLBrace`; repurpose for Set |
| Parser | `phalcom-ast/src/parser.rs` | bare braces currently Map classification; `#{}` Record parser; old Record type-tail grammar; data product expansion |
| Parser tests | `phalcom-ast/tests/*` | syntax migration, kind syntax, product-expansion declarations |
| Kinds | `phalcom-semantic/src/types/kind.rs`, `id.rs` | source name `RecordFields`; public `CompleteTypePack` |
| Generic parameters | `phalcom-semantic/src/types/parameter.rs` | mode/kind compatibility instead of one-to-one mapping |
| Declaration bootstrap | `phalcom-semantic/src/declarations.rs` | source facade mapping; Unit removal; RecordFields/CompleteTypePack; explicit replace/refresh |
| Type annotations/application | `phalcom-semantic/src/types/annotation.rs` | `Record<**Fs>`; consolidated intrinsic application; kind syntax |
| Type store | `phalcom-semantic/src/types/store.rs` | preserve existing pack/row identities; source-level equivalence tests |
| Row/field solver | `phalcom-semantic/src/types/row.rs`, `row_solver.rs` | preserve internals; source rename only |
| Algebraic declaration checker | `phalcom-semantic/src/checker/algebraic_declaration.rs` | symbolic product expansion; constructor/product templates |
| Call/inference | `phalcom-semantic/src/checker/{call,inference,row_inference}.rs` | RecordFields labeled binder inference and application |
| Type members | `phalcom-semantic/src/types/type_member.rs`, `projection.rs` | `CompleteTypePack` source surface; Record Fields/Value; public op/source bijection |
| Associated surface | `phalcom-semantic/src/associated.rs`, checker associated paths | parameterized impl family publication |
| Metadata | `phalcom-semantic/src/metadata/*`, `phalcom-type-meta/src/*` | mode/kind independence, CompleteTypePack source round-trip, RecordFields presentation |
| Incremental DB | `phalcom-semantic/src/db/fingerprint.rs` and incremental tests | new binding/product-shape/alias facts |
| Native import | `phalcom-semantic/src/types/native.rs` | runtime-class projection remains separate from type family |
| Universe Tuple | `phalcom-core/core/universe/src/collections/tuple.ph` | final source declaration and typed API |
| Universe Record | `phalcom-core/core/universe/src/collections/record.ph` | `RecordFields`, final declaration, typed API |
| Unit source | `phalcom-core/core/universe/src/option/unit.ph` | replace runtime data with intrinsic alias |
| Core surface IDs | `phalcom-semantic`/`phalcom-core` Universe key tables | Unit/RecordFields/CompleteTypePack canonical identities |
| Semantic lowering | `phalcom-core/src/modules/semantic_lowering.rs` | runtime owner direct lowering; expanded data shape specs |
| Data compiler | `phalcom-core/src/compiler/lib/data_decl.rs` | intrinsic runtime projection; user expanded products |
| Product compiler | `phalcom-core/src/compiler/lib/product_opt.rs`, product layout modules | Record syntax migration; expanded product representation |
| Expression compiler | `phalcom-core/src/compiler/lib/expr.rs` | Map explicit construction; Set syntax; Record bare braces |
| Record literal runtime | `phalcom-core/src/heap/record_literal_builder.rs` | empty Record and spread behavior |
| VM data/runtime class | `phalcom-core/src/vm/*`, class/bootstrap tables | Unit runtime class removal/aliasing; one Tuple/Record class |
| LSP | `phalcom-lsp/src/*` | token rename, definitions, hovers, semantic tokens |
| Source index | `phalcom-semantic/src/source_index/*` | syntax traversal, intrinsic declaration/member anchors |
| Specs | `docs/specs/{type-level-domains,structural-products,type-members,universe}.md`, collection specs | normative migration |
| C3 lifecycle docs | `docs/implementation/LANG007/LANG007.C3/*` | P4 numbering normalization and checkpoint update |

The patch-grade plan should re-read these files at implementation time because later commits may move responsibilities.

---

## 29. Test requirements

P4 requires both migration coverage and new semantic coverage. Tests should be organized by authority, not only by syntax.

### 29.1 Parser/AST tests

Add or migrate tests for:

#### Intrinsic declarations

```phalcom
intrinsic data Tuple<***Es: CompleteTypePack>(***Es)

intrinsic data Record<**Fs: RecordFields> {
    **Fs
}
```

Assertions:

- runtime `intrinsic data` is a real `ProductShapeSyntax`;
- Tuple shape includes complete expansion;
- Record shape includes labeled/RecordFields expansion;
- no runtime `IntrinsicFacade` branch is used.

#### User-defined product expansions

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)

data Prefix<*Ts: TypePack>(Int, *Ts)

data MyRecord<**Fs: RecordFields> {
    **Fs
}

data Envelope<**Fs: RecordFields> {
    id: Id
    **Fs
}
```

Negative parser/semantic tests:

```phalcom
data Bad<***Es: CompleteTypePack>(***Es, Int)
```

and incompatible expansion-kind combinations.

#### Literal syntax

```phalcom
{}
{ name: 1 }
{ name: 1, **rest }

Map {}
Map { name: 1 }
Map<String, Int> { "a": 1 }

#{}
#{1}
#{1, 2}
```

Ensure `{...}` never becomes `MapLiteral` by content inspection.

#### Record type syntax

```phalcom
{}
{ name: String }
{ name: String, **Fs }
```

Reject/migrate old `| R` syntax.

### 29.2 Kind and binder tests

Required cases:

```text
Single(TypePack) accepted
Positional(TypePack) accepted
Single(LabeledTypePack) accepted
Labeled(LabeledTypePack) accepted
Single(CompleteTypePack) accepted
Complete(CompleteTypePack) accepted
Single(RecordFields) accepted
Labeled(RecordFields) accepted
```

Reject:

```text
Positional(RecordFields)
Complete(RecordFields)
Labeled(TypePack)
Positional(LabeledTypePack)
```

Verify parameter identity/fingerprint differs when mode differs with same kind.

### 29.3 Tuple application tests

Direct source-level equivalences:

```text
Tuple<> == Unit == ()
Tuple<Int,String> == (Int,String)
```

Concrete and symbolic complete pack applications.

Verify:

```text
Tuple<...> is never TypeData::Applied
```

for canonical intrinsic Tuple.

### 29.4 Record application tests

Direct source-level equivalences:

```text
Record<> == EmptyRecord == {}
```

Open/symbolic:

```phalcom
Record<**Fs>
```

must preserve the exact `RecordRowId`/field parameter identity.

Verify no `TypeData::Parameter` or ordinary type-lambda encoding is introduced for the row/field binder.

### 29.5 Unit/EmptyRecord tests

```text
Unit != EmptyRecord
```

and:

```text
runtime class of unit value == Tuple
runtime class of empty Record == Record
```

Any old test expecting Unit's own runtime class must be migrated or removed.

### 29.6 Tuple associated type tests

Concrete and symbolic:

```text
Entries
Positionals
Labeled
Element
At
Map
```

`Entries` must have kind `CompleteTypePack`.

`Element`:

```text
(Int,String)::Element == Int | String
()::Element == Never
```

### 29.7 Record associated type tests

```text
Fields
Value
Map
```

Concrete/open/symbolic rows.

`Value`:

```text
{name:String, age:Int}::Value == String | Int
{}::Value == Never
```

### 29.8 Public method typing tests

Compile fixtures that prove:

- Tuple `first`/`last` return `Option<Element>`;
- Tuple `at(runtimeIndex)` returns `Element`;
- Tuple iterator Item is `Element`;
- Record `get(Symbol)` returns `Option<Value>`;
- internal/native implementation does not leak `Dynamic` into the public surface.

Negative fixtures should prove no accidental widening to `Dynamic`.

### 29.9 Product-shape expansion tests

For user-defined data:

```phalcom
data MyTuple<***Es: CompleteTypePack>(***Es)
```

verify:

- specialization remains nominal;
- constructor signature expands correctly;
- product component order is correct;
- empty specialization remains nominal zero-component data, not Unit;
- labeled entries preserve tuple product legality.

For:

```phalcom
data MyRecord<**Fs: RecordFields> {
    **Fs
}
```

verify:

- specialization remains nominal;
- fields specialize correctly;
- duplicate explicit/spread fields are rejected through lacks logic;
- empty specialization is nominal MyRecord, not EmptyRecord.

### 29.10 Runtime class tests

Tuple:

```text
Unit / ()       -> Tuple class
nonempty Tuple  -> Tuple class
labeled Tuple   -> Tuple class
```

Record:

```text
{}              -> Record class
nonempty Record -> Record class
```

Ensure no class split by arity/shape.

### 29.11 Syntax migration corpus

Run repository-wide source search and migrate:

```text
#{ ... } Record
{ ... } Map
RecordRow
TupleEntries
intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
impl Tuple
impl Record
```

Each migration category should have at least one parser fixture demonstrating the new canonical syntax.

### 29.12 Metadata tests

Round-trip:

```phalcom
<***Es: CompleteTypePack>
<**Ls: LabeledTypePack>
<**Fs: RecordFields>
```

and prove mode/kind are both preserved.

Verify cold/imported source has the same intrinsic type-family and member identity.

### 29.13 Incremental tests

Changes that must invalidate:

- `RecordFields` binder domain;
- expansion mode;
- Tuple/Record product expansion shape;
- Unit alias target;
- EmptyRecord alias target;
- intrinsic member signatures;
- runtime owner;
- public kind anchor.

### 29.14 Native/runtime projection tests

Retain the P3 adversarial runtime-owner test pattern but update it for the completed model:

- corrupt or alter the type-family/static projection;
- verify native/runtime import still selects the explicit Tuple/Record runtime owner;
- verify the compiler consumes owner identity rather than source name lookup.

### 29.15 Source index/LSP tests

Definitions and hovers for:

```text
CompleteTypePack
RecordFields
Tuple::Entries
Tuple::Element
Record::Fields
Record::Value
Unit
EmptyRecord
```

must resolve to canonical source declarations.

### 29.16 Full certification

After focused suites, run fresh post-P4 broad gates, not merely predecessor evidence:

```bash
cargo build --workspace --all-targets
cargo test --workspace
```

plus project-specific Clippy/format/release gates according to repository doctrine.

Known predecessor failures must be reproduced on the predecessor and recorded explicitly rather than silently accepted.

---

## 30. Required semantic invariants

The implementation plan should treat the following as normative.

### Declaration/family invariants

**P4-I01** — `Tuple` and `Record` each have one canonical source declaration identity.

**P4-I02** — type-family and runtime-class projections are sibling declaration projections; neither is reconstructed from the other.

**P4-I03** — `Tuple` type-family application has semantic kind `CompleteTypePack -> Type`.

**P4-I04** — `Record` type-family application has semantic kind `RecordFields -> Type`.

**P4-I05** — ordinary user `data` with the same pack/field binders remains nominal.

### Structural-kind invariants

**P4-I06** — `TypePack`, `LabeledTypePack`, `CompleteTypePack`, and `RecordFields` are distinct public static domains.

**P4-I07** — public `RecordFields` binds to the existing internal Record row domain.

**P4-I08** — `TupleEntries` introduces no independent domain; `Tuple::Entries` returns `CompleteTypePack`.

**P4-I09** — pack/field domains are not runtime collection classes.

### Expansion invariants

**P4-I10** — expansion mode and semantic kind are independently represented.

**P4-I11** — `**` is valid for both `LabeledTypePack` and `RecordFields`.

**P4-I12** — `Single(TypePack)` differs from `Positional(TypePack)`.

**P4-I13** — `Single(RecordFields)` differs from `Labeled(RecordFields)`.

**P4-I14** — incompatible mode/kind pairs are rejected explicitly.

### Product declaration invariants

**P4-I15** — runtime intrinsic Tuple/Record declarations use ordinary product-shape AST, not `IntrinsicFacade`.

**P4-I16** — typed product-shape expansion is available to ordinary user `data`.

**P4-I17** — expanded user data retains nominal family identity.

**P4-I18** — expansion-derived specialized components do not receive fabricated unstable source `DataComponentId`s.

### Tuple invariants

**P4-I19** — `Tuple<> == Unit == ()`.

**P4-I20** — concrete `Tuple<...>` application normalizes through existing Tuple structural authority.

**P4-I21** — symbolic Tuple application preserves existing pack lane identities.

**P4-I22** — all Tuple values use one runtime Tuple class.

**P4-I23** — Tuple public methods do not erase known structural payload typing to `Dynamic`.

### Record invariants

**P4-I24** — `Record<> == EmptyRecord == {}`.

**P4-I25** — `Unit != EmptyRecord`.

**P4-I26** — Record field identity remains `RecordRowId`/row-solver backed internally.

**P4-I27** — `Record<**Fs>` consumes `RecordFields` directly; no TypeId row encoding is introduced.

**P4-I28** — explicit Record fields plus spread fields use lacks/duplicate constraints.

**P4-I29** — all Record values use one runtime Record class.

**P4-I30** — Record public methods do not erase known field payload typing to `Dynamic`.

### Literal invariants

**P4-I31** — bare `{...}` is structural Record syntax.

**P4-I32** — `Map {...}` is explicit Map construction.

**P4-I33** — `#{...}` is Set syntax.

**P4-I34** — parser classification does not inspect bare brace content to choose Map vs Record.

**P4-I35** — Record spread and Map spread share syntax only; they retain different semantic authorities.

### Runtime/declaration invariants

**P4-I36** — Unit owns no separate runtime class after migration.

**P4-I37** — EmptyRecord owns no separate runtime class.

**P4-I38** — runtime lowering consumes explicit runtime-owner identity.

**P4-I39** — declaration authoritative replacement and incremental shell refresh are distinct operations.

### Associated/source invariants

**P4-I40** — `AssociatedSurface` remains the sole associated namespace authority.

**P4-I41** — every public compiler intrinsic structural operation has a canonical source declaration.

**P4-I42** — every source intrinsic type member maps to exactly one compiler implementation.

### Persistence invariants

**P4-I43** — metadata preserves mode and kind independently.

**P4-I44** — metadata represents `CompleteTypePack` directly rather than through `TupleEntries`.

**P4-I45** — incremental fingerprints cover family application, runtime owner, expansion mode, kind, aliases, and product-shape templates.

**P4-I46** — cold and incremental semantic products converge.

---

## 31. Risk analysis

### 31.1 Highest-risk change: user-defined expanded data products

This is the largest semantic addition because existing data semantics and lowering assume a fixed component list. The patch-grade plan should isolate this work early and prove a minimal symbolic product template before migrating intrinsic Tuple/Record source declarations.

A tempting but incorrect shortcut would be to support expansion only in intrinsic Tuple/Record syntax. That would recreate the very special-case grammar P4 is meant to eliminate.

### 31.2 High-risk change: Unit runtime-class collapse

Current code and tests explicitly register `IntrinsicRuntimeOwner::Unit` and certify reuse of a preinstalled Unit class. P4 deliberately changes this invariant.

The plan must inventory all Unit runtime dispatch/class identity paths before removing the owner. `Value::unit()` representation can remain; class identity must become Tuple.

### 31.3 High-risk change: Record source syntax/Map syntax migration

Bare braces currently have Map-specific parsing logic. Record syntax currently has a dedicated lexer token. Changing both at once touches a wide fixture corpus and can create misleading parse failures.

The migration should be performed with parser tests first, then semantic/compiler fixture migration.

### 31.4 Medium risk: mode/kind metadata compatibility

The current semantic model already stores kind and mode independently, which reduces implementation risk, but stable metadata validators may encode the old one-to-one assumption. Schema compatibility needs explicit inspection.

### 31.5 Medium risk: declaration projection generalization

The existing P3 corrective binding is sound. P4 should generalize semantics without destabilizing native import or associated lookup. Avoid replacing the entire declaration table in one patch if smaller explicit APIs can achieve the goal.

### 31.6 Medium risk: fully typed Tuple/Record public API

Adding `Element`/`Value` introduces union/join normalization dependencies and may reveal under-specified method signatures in iterators and collection protocols. This should be implemented after the associated projections normalize correctly.

### 31.7 Lower risk: CompleteTypePack source name and RecordFields facade rename

The underlying semantic kinds already exist. Most work is parser/source/metadata presentation and canonical declaration binding.

---

## 32. Recommended dependency order for the eventual P4 implementation plan

This requirements analysis is not the patch-grade plan, but the dependency graph is clear enough to constrain task ordering.

### Foundation A — lifecycle and RED tests

- normalize the existing corrective P4 record to P3.R1 or choose P5 numbering;
- add RED parser/semantic tests for final declarations, modes, aliases, literal syntax, and runtime-class laws.

### Foundation B — kind/mode model

- expose `CompleteTypePack`;
- bind `RecordFields` to the existing row kind;
- generalize mode/kind compatibility;
- update metadata/fingerprints.

### Foundation C — product-shape expansion

- generalize AST/parser;
- introduce semantic symbolic expanded product shape;
- make user-defined expanded data compile semantically;
- extend lowering/runtime data representation as necessary.

### Foundation D — declaration/family application

- consolidate intrinsic application contract;
- migrate Tuple and Record declarations;
- make runtime owner authoritative;
- split declaration replace/refresh APIs.

### Foundation E — literal syntax

- `{}` Record;
- `Map {}` Map;
- `#{}` Set;
- Record type `**Fs` syntax;
- migrate source corpus.

### Foundation F — Tuple/Record source and method completion

- source-visible kinds and intrinsic type members;
- `Element`, `Value`, `Fields`;
- parameterized impls;
- fully typed methods and iterators.

### Foundation G — zero specializations

- `intrinsic type Unit = Tuple<>`;
- `intrinsic type EmptyRecord = Record<>`;
- collapse Unit runtime class into Tuple;
- certify empty Record runtime class is Record.

### Foundation H — tooling, metadata, broad certification

- source index/LSP;
- docs/specs;
- cold/incremental equivalence;
- workspace build/test/Clippy/release gates.

This order minimizes the period during which source declarations claim semantics the compiler cannot yet represent.

---

## 33. Acceptance matrix

P4 should not close until all rows below are proven.

| Area | Required endpoint |
|---|---|
| Tuple source declaration | `intrinsic data Tuple<***Es: CompleteTypePack>(***Es)` |
| Record source declaration | `intrinsic data Record<**Fs: RecordFields> { **Fs }` |
| User Tuple-like data | `data MyTuple<***Es: CompleteTypePack>(***Es)` works nominally |
| User Record-like data | `data MyRecord<**Fs: RecordFields> { **Fs }` works nominally |
| Complete kind | `CompleteTypePack` is source-visible |
| Record field kind | source name is `RecordFields`; internal row solver retained |
| Tuple Entries | returns `CompleteTypePack` |
| Unit | `intrinsic type Unit = Tuple<>` |
| EmptyRecord | `intrinsic type EmptyRecord = Record<>` |
| Empty distinction | `Unit != EmptyRecord` |
| Tuple runtime class | one `Tuple` class for empty/non-empty/labeled Tuples |
| Record runtime class | one `Record` class for empty/non-empty Records |
| Record literal | `{...}` |
| Map construction | `Map {...}` |
| Set literal | `#{...}` |
| Record type spread | `{ field: Type, **Fs }` |
| Record value spread | `{ field: value, **record }` |
| Expansion model | mode independent from kind |
| Labeled pack | `**Ls: LabeledTypePack` |
| Record fields | `**Fs: RecordFields` |
| Tuple methods | public surface typed without unnecessary `Dynamic` |
| Record methods | public surface typed without unnecessary `Dynamic` |
| Associated namespace | one `AssociatedSurface` |
| Runtime owner | explicit owner projection consumed end-to-end |
| Declaration updates | authoritative replace distinct from shell refresh |
| Metadata | mode/kind independent; CompleteTypePack direct |
| Incremental | cold/incremental convergence |
| LSP/source index | canonical source anchors |
| Certification | fresh post-P4 workspace-wide gates |

---

## 34. Final architectural statement

C3.P4 should leave Phalcom with the following coherent model:

```text
Source declaration
    |
    +-- typed static/type-family projection
    |
    +-- runtime-class projection
    |
    +-- associated/static namespace
    |
    +-- reflection/source identity
```

Tuple:

```text
intrinsic data Tuple<***Es: CompleteTypePack>(***Es)

type family:
    CompleteTypePack -> Type

application:
    intrinsic structural Tuple normalization

runtime class:
    Tuple

zero specialization:
    Tuple<> == Unit == ()
```

Record:

```text
intrinsic data Record<**Fs: RecordFields> {
    **Fs
}

type family:
    RecordFields -> Type

application:
    intrinsic structural Record normalization

runtime class:
    Record

zero specialization:
    Record<> == EmptyRecord == {}
```

and:

```text
Unit != EmptyRecord
```

The language-level structural syntax is correspondingly explicit:

```text
Tuple   (...)
Record  {...}
List    [...]
Set     #{...}
Map     Map {...}
```

The deepest semantic correction is not any individual syntax change. It is the removal of accidental coupling between:

```text
declaration identity
type-family application
runtime class
structural shape
expansion mode
semantic kind
```

P3 established the first necessary separation between intrinsic semantic application and runtime owner. P4 completes that separation across the source language, generic kind system, product declarations, Tuple/Record families, zero specializations, literals, runtime behavior, metadata, and tooling.

When P4 closes, Tuple and Record should no longer be “compiler exceptions with source facades.” They should be ordinary-looking canonical source declarations whose intrinsic bindings tell the compiler how their otherwise fully typed, fully source-visible structural families normalize and execute.
