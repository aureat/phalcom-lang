# Phalcom User-Facing Type Syntax and Lowering

**Date:** 2026-08-22
**Revision:** post-Spec-01.5 canonical generic semantic model
**Status:** Ratified syntax and lowering specification, with named implementation gates
**Authority:** source-facing type-form grammar, generic declaration grammar, parser/recovery rules, and compiler-authoritative lowering into the canonical semantic model defined by Spec 01.5
**Depends on:** [01 — Implementation Architecture](01-implementation-architecture.md) and [01.5 — Canonical Generic Type Semantics and Declaration Model](01.5-canonical-generic-type-semantics-and-declaration-model.md)
**Coordinates with:** [02 — Runtime Reification and Metadata](02-runtime-reification-and-metadata.md), [03 — Reflection API and Capabilities](03-reflection-api-and-capabilities.md), and [05 — Advanced Kinds, Effects, Contracts, and Proofs](05-advanced-kinds-constraints-effects-and-proofs.md)
**Owners:** `phalcom-ast` for source syntax/recovery, `phalcom-semantic` for source lowering and semantic validation, `phalcom-type-syntax` only for native-metadata textual syntax, and future formatter/source-printer ownership
**Repository snapshot inspected:** `a43f26e0ddd6b1d6e37ddf7a0b9588769bb41f3e` (`main`, 2026-08-22)
**Scope:** annotations, type forms, applications, unions, tuples, callable types, structural record types, generic binders, declaration-site variance, explicit kinds, `where` constraints, type lambdas, partial type application, generic superclass templates, `Self`, transparent aliases, record-row source spelling, type-form value boundaries, parser recovery, lowering, diagnostics, formatting, and tooling-facing syntax provenance
**Non-goals:** defining canonical generic semantics already owned by 01.5; runtime descriptor representation; metadata wire format; reflection capability/security policy; kind polymorphism; effect/proof syntax; protocol coherence; opaque/newtype aliases; recursive type aliases; runtime selector specialization; runtime class specialization; per-instance generic tokens

---

## 0. Revision contract

This document replaces the previous revision of **04 — User-Facing Type Syntax and Lowering** wherever that revision conflicts with Spec 01.5.

The previous document contained several decisions that were reasonable before the canonical generic semantic layer was written but are now stale. This revision deliberately removes them instead of preserving compatibility aliases that would become permanent language debt.

The following changes are normative:

| Previous surface/design | Revised decision |
|---|---|
| `F :: Type -> Type` in a generic binder | `F: Type -> Type` |
| `::` as source kind-ascription syntax | `::` is documentation/judgment notation only; source binder ascription uses `:` |
| inline bound `class Box<T <: Number>` | rejected; all generic constraints use `where` |
| `T in (Int, Float)` finite exact-set constraint | deferred; no initial grammar or metadata tag |
| lower bound listed as unratified | supported by operand order: `Number <: T` |
| equality constraint listed as unratified | ratified as contextual semantic equivalence: `T == U` inside `where` |
| generic parameter grammar always permits variance | variance only on nominal declaration parameters; not method, alias, or type-lambda binders |
| no type-lambda syntax | `<T> =>> Result<T, Error>` |
| generic semantics partly owned by Spec 05 | ordinary generic semantics are owned by 01.5; Spec 05 owns genuinely advanced reasoning |
| Spec 04 depends on runtime metadata/reflection | removed; syntax/lowering depends on 01 and 01.5, while 02/03 consume semantic products downstream |
| type annotations described as incapable of becoming runtime expressions | annotations remain non-executable, but explicit type-form expressions may cross into value space under the rules in §11 |

The following previous decisions remain accepted unless this document says otherwise:

- contextual, side-effect-free type parsing/lowering;
- `()` as source unit spelling;
- `Never`, `Dynamic`, and `Self` contextual type atoms;
- `Unknown` is not a source type;
- union `|`;
- tuple and callable type syntax;
- structural record syntax `#{ ... }`;
- open record-row spelling `#{ field: T, | R }` with mandatory comma before a tail after known fields;
- transparent alias spelling `type Name = ...`;
- source/native parsers remain separate front ends;
- parser recovery must never invent successful semantic types;
- no static type metadata enters selector identity, runtime layout, allocation, or runtime class identity.

---

# 1. Purpose and authority

Phalcom has one programming language and one runtime object model, but not every syntactic occurrence is evaluated by ordinary runtime dispatch. Type annotations and generic signatures are compiler-authoritative semantic declarations. Type forms may also be represented as values for reflection, but the language does not make static type normalization dependent on user-overridable message sends.

This document defines **how source text denotes the semantic forms already defined by Spec 01.5**.

It owns four boundaries:

1. **source grammar** — which type/generic forms may be written and with what precedence;
2. **syntax representation** — which AST/recovery/provenance nodes preserve that source;
3. **semantic lowering** — how syntax resolves into canonical type forms, generic signatures, constraints, type lambdas, superclass templates, and explicit failure states;
4. **source presentation** — diagnostics, formatter rules, source-preserving rendering, and IDE syntax behavior.

It does **not** get to redefine canonical semantic equality, beta reduction, variance laws, generic relation semantics, type-store lifetime, reflection objects, or metadata encoding. Those belong to 01/01.5/02/03.

## 1.1 Two axes and three notations

The semantic distinctions remain:

```text
value.class     runtime class identity
value : T       static value-typing judgment
T :: K          semantic kinding judgment
```

`::` in the third line is mathematical/specification notation. It is not the source token used to annotate a generic parameter's kind.

Source uses a single annotation marker:

```phalcom
value: User
F: Type -> Type
R: RecordRow
```

This is intentional. `:` means “this syntactic binder/value is constrained/described by the form on the right” while the semantic layer records whether the right side is a value type or a kind according to the grammar position.

## 1.2 Compiler authority and runtime invariance

No grammar in this document may cause static type metadata to enter:

- selector identity;
- method lookup key construction;
- runtime class/metaclass identity;
- object layout;
- object allocation;
- ordinary instance `class` results;
- creation of specialized runtime classes.

`List<Int>` is a semantic application of the type form `List` to `Int`. It does not create an `Int`-specialized runtime `List` class.

## 1.3 Source syntax is not the semantic ontology

Syntax preserves spelling, ranges, comments/recovery boundaries, and binder names. Semantic identity does not.

In particular:

- renaming `<T>` to `<U>` changes source presentation, not generic binder identity after owner/index lowering;
- renaming `<T> =>> List<T>` to `<U> =>> List<U>` does not change canonical type-lambda identity;
- alias names retain declaration identity for navigation/provenance but transparent aliases expand for semantic equivalence;
- source ordering may be retained even where canonical semantic representations sort or deduplicate.

---

# 2. Current repository state

This section is observational. It describes the repository at the inspected `main` snapshot and must not be read as claiming the target syntax already exists.

## 2.1 The AST already models more type structure than the parser accepts

`phalcom-ast/src/ast.rs` currently defines `TypeAnnotation` and `TypeAnnotationExpr` around source lines 406–443. The expression enum already includes:

- `Reference`;
- `Application`;
- `Union`;
- `Tuple`;
- `Callable`.

Supporting structs preserve tuple labels, callable labels, rest flags, and source ranges.

However `ClassDef` around `phalcom-ast/src/ast.rs:227-263` has no generic parameter list or `where` clause, and its superclass is still `Option<StaticSymbolRef>`. That is insufficient for a generic superclass template such as:

```phalcom
class Names<T> is Sequence<Option<T>> { ... }
```

`MethodDef` around `phalcom-ast/src/ast.rs:700-727` likewise has no method-generic binder list or `where` clause.

The current `VariantDef` intentionally carries labels but no payload type annotations. Typed variant payload grammar is therefore not smuggled into this specification.

## 2.2 Source annotation parsing is still reference-only

`phalcom-ast/src/parser.rs:1396-1421` implements `Parser::parse_type_annotation` as a static identifier/qualified-reference parser. It does not parse the application, union, tuple, or callable AST variants already present in `TypeAnnotationExpr`.

Therefore source/parser parity is currently incomplete:

```text
AST can represent      List<Int> | None
parser accepts         Int
parser accepts         geometry.Point
parser does not yet accept the complete structural grammar
```

This is a valuable migration seam: core composite parsing can be implemented and tested before all generic declaration features land.

## 2.3 Semantic annotation lowering already contains useful structure, but failure states are too coarse

`phalcom-semantic/src/types/annotation.rs` currently resolves all existing `TypeAnnotationExpr` forms. It:

- resolves static references;
- recognizes implementation-facing `Never`, `Unit`, and `Dynamic` by reference spelling;
- kind-checks generic application through `TypeStore::apply_type_form`;
- lowers tuples, callable types, and unions;
- enforces proper-kind `Type` for tuple/callable/union components.

The important defect is that application/kind failures commonly return:

```rust
TypeFormResolution::Unknown(UnknownReason::UnannotatedDeclaration)
```

That conflates invalid written type syntax/semantics with an actually absent annotation. This revision preserves the previous requirement to replace that collapse with explicit result states from Spec 01.

The current resolver also treats `Unit` as a reference spelling. The source spelling ratified here remains `()`; `Unit` is not canonical source syntax merely because an internal/universe declaration may carry that name.

## 2.4 Generic parameter identity exists, but source generic declarations do not

`phalcom-semantic/src/types/parameter.rs` currently contains:

```rust
pub enum TypeParameterOwner {
    Declaration(DeclarationId),
    Callable(CallableId),
}

pub struct TypeParameterData {
    pub owner: TypeParameterOwner,
    pub index: u16,
    pub name: Box<str>,
    pub kind: KindId,
}

pub struct GenericSignature {
    pub owner: TypeParameterOwner,
    pub parameters: Box<[TypeParameterId]>,
}
```

This already establishes the correct principle that textual parameter names are not global semantic identity. Spec 01.5 extends the model with variance/provenance at the parameter level and signature-owned constraints.

Source lowering in this spec must target that owner/index model. It must not build name-keyed canonical generic semantics.

## 2.5 Native type syntax is a different parser and must remain one

`phalcom-type-syntax/src/lib.rs` currently implements a small independent parser used for native metadata. Its `TypeExpr` supports `Unknown`, `Never`, `SelfType`, named/universe references, parameters, applications, unions, and tuples. `CallableType` supports named type parameters, parameter tuples, and a return type.

It does not have:

- source ranges/recovery;
- declaration owner identities;
- variance;
- kind annotations on its own `TypeParameter` parser record;
- signature-owned constraints;
- record rows;
- type lambdas.

That difference is not a reason to force source and native metadata through one parser. They have different error/recovery requirements. They must converge **below parsing** by lowering to equivalent semantic forms.

## 2.6 Current semantic application already supports partial kind application

`TypeStore::apply_type_form` kind-checks the origin and arguments, computes the residual kind, and flattens repeated applications into one application spine. This directly supports the semantic law required by 01.5:

```text
apply(apply(F, [A]), [B]) ≡ apply(F, [A, B])
```

Source syntax should expose that law rather than inventing a second generic-application mechanism.

## 2.7 Current type forms do not include type lambdas

`TypeData` currently includes `Never`, `Unit`, class-object, nominal, applied, union, tuple, record, callable, parameter, and inference forms. There is no canonical `TypeLambda` or scoped lambda-bound-variable representation yet.

Therefore the parser may be implemented ahead of semantic support only behind a feature/test gate; public acceptance of type-lambda syntax requires the Spec 01.5 type-lambda workstream to exist.

---

# 3. Grammar domains

The previous specification used “type annotation” for several concepts that need to be separated operationally.

This revision defines three syntax domains.

## 3.1 `type-form`

A `type-form` denotes any semantic form classified by a kind. Its result need not be a proper runtime-value type.

Examples:

```text
Int                       :: Type
List                      :: Type -> Type
List<Int>                 :: Type
Map<String>               :: Type -> Type
<T> =>> List<T>           :: Type -> Type
<F: Type -> Type> =>> F   :: (Type -> Type) -> (Type -> Type)
```

A `type-form` is therefore the correct grammar category for:

- type application origins and arguments;
- higher-kinded generic constraints where kinds permit them;
- type-lambda bodies;
- alias bodies whose result kind is not required to be `Type`;
- explicit type-form value expressions.

## 3.2 `proper-type-annotation`

A value annotation uses the same source grammar but imposes a semantic postcondition:

```text
kind(form) == Type
```

This category applies to:

- local/constant/field annotations;
- method parameters/rest parameters;
- method return types;
- tuple element types;
- record field types;
- callable parameter/result types;
- union members;
- superclass templates after normalization.

Thus parsing and kind-checking are separate:

```phalcom
value: Map<String>
```

is syntactically a valid type form but semantically invalid in a value-type position because `Map<String> :: Type -> Type`.

This distinction is essential for good diagnostics and higher-kinded programming.

## 3.3 `kind-expression`

Kinds form a separate restricted grammar, used only after a generic binder's source `:` marker:

```text
Type
Type -> Type
(Type -> Type) -> Type
RecordRow
```

Kinds do not use the ordinary type-form grammar. In particular, no source program writes `Type<Type>` or treats `Type` as a proper runtime value type merely because the word appears in a kind expression.

## 3.4 No ambient generic-call-frame grammar

There is no hidden syntax such as:

```text
Type.currentApplication
current generic arguments
receiver specialization scope
```

Generic binding is lexical and explicit in declarations/type lambdas. Specialization is semantic substitution/viewing as defined by 01.5.

---

# 4. Contextual entry points

## 4.1 Type-mode entry points

The parser enters compiler-authoritative type-form mode at these locations:

- after `:` in a value/field/parameter/return annotation position;
- after `:` in a generic binder when parsing a **kind expression** rather than a type form;
- after `->` in a callable type;
- between `<` and `>` in a type application once type-form mode is active;
- after `is` in a class declaration's generic superclass template;
- after `<:` or `==` in a `where` constraint, and for the left operand as part of the constraint production;
- after `=` in a transparent alias declaration;
- after `=>>` in a type lambda;
- inside record type field/tail syntax;
- in the explicit value-space type-form islands defined in §11;
- in native metadata's **separate** parser entry point.

## 4.2 Annotation parsing is side-effect free

Parsing/lowering an annotation or generic signature must not:

- send user-overridable messages;
- call arbitrary class-side methods;
- execute constructors;
- invoke DNU;
- inspect mutable runtime specialization state;
- allocate runtime `TypeDescriptor` objects;
- mutate runtime class/metaclass tables.

The compiler resolves names and normalizes semantic forms through compiler-owned data.

## 4.3 Contextual reserved spellings

| Spelling | Contextual meaning | Status |
|---|---|---|
| `()` | unit proper type / empty tuple type | Ratified |
| `Never` | bottom proper type | Ratified |
| `Dynamic` | explicit dynamic/open-world type boundary | Ratified |
| `Self` | owner-relative type form | Ratified |
| `Any` | future top proper type | Reserved; not accepted until `Any` semantics are ratified |
| `Unknown` | never a source type | Rejected |
| `Type` | atomic proper-type kind | Ratified only in kind grammar |
| `RecordRow` | row kind | Spelling ratified; parser enablement gated by row semantic support |
| `where` | generic constraint introducer | Ratified |

These words remain ordinary identifiers in ordinary expression grammar where the language otherwise permits them. Their special meaning is contextual.

---

# 5. Core type-form grammar

## 5.1 Normative EBNF

The core grammar is:

```ebnf
type-form           ::= type-lambda
                      | union-type

type-lambda         ::= "<" type-lambda-parameter
                            ("," type-lambda-parameter)* ">"
                        "=>>" type-form

type-lambda-parameter
                    ::= identifier kind-annotation?

union-type          ::= callable-type ("|" callable-type)*

callable-type       ::= postfix-type
                      | callable-domain "->" type-form

postfix-type        ::= type-atom type-arguments*

type-arguments      ::= "<" type-form ("," type-form)* ">"

type-atom           ::= static-type-reference
                      | "()"
                      | "Never"
                      | "Dynamic"
                      | "Self"
                      | parenthesized-type
                      | tuple-type
                      | record-type

static-type-reference
                    ::= identifier ("." identifier)*

parenthesized-type  ::= "(" type-form ")"

tuple-type          ::= "(" tuple-type-element ","
                          (tuple-type-element ("," tuple-type-element)*)? ")"

tuple-type-element  ::= (identifier ":")? type-form

callable-domain     ::= "(" callable-parameter-list? ")"

callable-parameter-list
                    ::= callable-parameter ("," callable-parameter)* ","?

callable-parameter  ::= (identifier ":")? type-form
                      | "..." type-form

record-type         ::= "#{" record-type-body? "}"

record-type-body    ::= record-type-field ("," record-type-field)* ","?
                      | record-type-field ("," record-type-field)* ","
                        "|" record-row-tail
                      | "|" record-row-tail

record-type-field   ::= identifier ":" type-form
record-row-tail     ::= identifier

kind-annotation     ::= ":" kind-expression

kind-expression     ::= kind-atom
                      | kind-atom "->" kind-expression

kind-atom           ::= "Type"
                      | "RecordRow"
                      | "(" kind-expression ")"
```

A parser implementation may internally split this grammar into precedence functions or Pratt parselets. The semantic language is defined by the resulting tree and precedence rules, not the implementation technique.

## 5.2 Precedence and associativity

From strongest to weakest:

1. atoms/grouping;
2. postfix type application `<...>`;
3. callable arrow `->`;
4. union `|`;
5. type-lambda binder `=>>`, whose body extends over a complete `type-form`.

Application associates into one semantic spine:

```text
Map<String><Int>
```

may be accepted by the internal parser if chained postfix application is allowed, but canonical source formatting should prefer:

```phalcom
Map<String, Int>
```

Callable arrow is right-associative:

```phalcom
(Int) -> (String) -> Bool
```

means:

```text
(Int) -> ((String) -> Bool)
```

The result of a callable arrow is a complete type form, therefore:

```phalcom
(Int) -> String | None
```

means:

```text
(Int) -> (String | None)
```

not:

```text
((Int) -> String) | None
```

The latter requires parentheses:

```phalcom
((Int) -> String) | None
```

A type-lambda body also extends to the right:

```phalcom
<T> =>> Result<T, Error> | None
```

means:

```text
<T> =>> (Result<T, Error> | None)
```

To apply a lambda immediately, parenthesize it:

```phalcom
(<T> =>> Result<T, Error>)<Int>
```

## 5.3 Unit/group/tuple distinctions

| Source | Meaning |
|---|---|
| `()` | unit / empty tuple proper type |
| `(T)` | grouped type `T` |
| `(T,)` | one-element tuple type |
| `(T, U)` | two-element tuple type |
| `(name: T, age: U)` | labeled tuple/product type |
| `(T) -> R` | callable with one positional parameter |
| `() -> R` | callable with zero parameters |
| `((),) -> R` | callable with one unit-valued parameter |

The parser must not guess tuple/callable meaning from semantic resolution. The comma and following `->` grammar determine syntax.

## 5.4 Type application

Source application is:

```phalcom
List<Int>
Map<String, Int>
Result<T, Error>
F<T>
```

Multi-argument syntax denotes repeated kind application without requiring intermediate semantic allocation:

```text
Map<String, Int>
≡ apply(Map, [String, Int])
≡ apply(apply(Map, [String]), [Int])
```

Application arguments are **type forms**, not arbitrary runtime expressions.

The following are invalid inside type syntax:

```phalcom
List<1>
List<someRuntimeCall()>
Map<T if cond else U>
```

unless a future dependent-type/value-indexed-type feature explicitly changes the language.

## 5.5 Union syntax

`A | B` is the active union constructor.

The parser preserves written members and ranges. Canonical semantic normalization may flatten, deduplicate, sort, remove `Never`, and collapse a singleton according to the type-store laws. Source formatting must not rely on canonical order when preserving user-written syntax.

Intersection `A & B` remains inactive. It must diagnose as unsupported type syntax rather than silently become a runtime selector inside a type-form context.

## 5.6 Callable types

Callable type syntax describes:

- positional/labeled/rest input types;
- normal return type.

It does **not** encode:

- effects;
- exceptional exits;
- termination;
- contracts;
- proof state.

Those are separate semantic axes and later source surfaces.

## 5.7 Structural record types and rows

Closed structural record:

```phalcom
#{ name: String, age: Int }
```

Open structural record row:

```phalcom
#{ name: String, | R }
#{ name: String | None, | R }
#{ | R }
```

The comma before a tail after known fields is mandatory. This preserves unambiguous union parsing:

```phalcom
#{ value: Int | String, | R }
```

The rejected ambiguous spelling remains rejected:

```phalcom
#{ value: Int | String | R }
```

A tail identifier must resolve to a binder of kind `RecordRow`. It is not an arbitrary type form, not a nominal class, and not `Dynamic`.

The source spelling is ratified now; parser/public enablement is gated on the row domain described by the revised Spec 05.

---

# 6. Generic declaration syntax

## 6.1 Binder categories are intentionally distinct

The language has one generic-signature semantic model, but not every binder supports every surface modifier.

### Nominal declaration parameters

Class/nominal declaration parameters may declare variance:

```ebnf
nominal-generic-parameters
                    ::= "<" nominal-generic-parameter
                            ("," nominal-generic-parameter)* ">"

nominal-generic-parameter
                    ::= variance? identifier kind-annotation?

variance            ::= "+" | "-"
```

Examples:

```phalcom
class Box<T> { ... }
class Producer<+T> { ... }
class Consumer<-T> { ... }
class FunctorBox<F: Type -> Type> { ... }
```

Bare `T` is invariant.

### Callable/method parameters

Method generic parameters do not declare variance:

```ebnf
callable-generic-parameters
                    ::= "<" generic-parameter
                            ("," generic-parameter)* ">"

generic-parameter  ::= identifier kind-annotation?
```

Example:

```phalcom
map<U>(transform: (T) -> U) -> List<U> { ... }
```

The following is invalid:

```phalcom
map<+U>(...)  // invalid: method-generic variance has no declaration-site subtyping role
```

### Alias parameters

Transparent alias binders use `generic-parameter` and do not accept variance markers. Alias applications are semantically transparent rather than separate nominal constructors whose declared variance participates in nominal subtype relations.

### Type-lambda parameters

Type-lambda parameters also use `generic-parameter` and never accept `+`/`-`.

## 6.2 Kind annotations use `:`

Ratified source:

```phalcom
class Functor<F: Type -> Type> { ... }
<F: Type -> Type, T> =>> F<T>
```

Rejected stale source:

```phalcom
class Functor<F :: Type -> Type> { ... }
```

Documentation may still state:

```text
F :: Type -> Type
```

when writing a semantic kinding judgment.

This duality must be reflected in diagnostics and formatting: source fix-its replace binder `::` with `:`; explanatory diagnostics may use `::` in mathematical messages if clearly presented as a judgment.

## 6.3 Omitted kinds

An omitted binder kind defaults to `Type`:

```phalcom
<T>
```

is equivalent at the binder level to:

```phalcom
<T: Type>
```

The formatter should preserve omission when source-preserving mode is used. A semantic/debug renderer may display the inferred/default kind explicitly.

## 6.4 No placeholder HKT syntax

Rejected:

```phalcom
F<_>
F<_, _>
```

Higher-kinded requirements are declared explicitly:

```phalcom
F: Type -> Type
```

Anonymous constructors use type lambdas:

```phalcom
<T> =>> Result<T, Error>
```

This avoids giving `_` two incompatible roles as inference hole and kind-arity placeholder.

---

# 7. Declaration attachment points and ordering

## 7.1 Classes

Target class signature grammar:

```ebnf
class-declaration   ::= "class" identifier
                        nominal-generic-parameters?
                        superclass-clause?
                        where-clause?
                        class-body

superclass-clause   ::= "is" type-form
```

Examples:

```phalcom
class Box<T> { ... }

class SortedBox<T> where T <: Comparable<T> { ... }

class Names<T> is Sequence<Option<T>> { ... }

class SortedNames<T>
  is Sequence<T>
  where T <: Comparable<T>
{
  ...
}
```

`where` follows the optional superclass clause so the declaration header has one predictable order:

```text
name → generic binders → superclass template → constraints → body
```

The parser may accept a single-line form without line breaks.

## 7.2 Generic superclass templates

The current AST's `ClassDef.superclass: Option<StaticSymbolRef>` is not sufficient.

Target syntax representation must preserve a full type-form superclass template, e.g.:

```rust
pub struct ClassDef {
    pub name: String,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub superclass: Option<TypeSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    // existing members/attributes/invariants/ranges...
}
```

Exact field names may vary during migration, but the semantic information may not be lost.

Semantic validation requires the normalized superclass template to denote an admissible proper nominal supertype under the object-model rules. Parsing a syntactically valid but semantically invalid form such as a union superclass produces a semantic diagnostic rather than requiring a special parser grammar for every future supertype category.

## 7.3 Methods

Target method signature order:

```text
method name → generic binders → value parameter list → return annotation → where clause → body
```

Example:

```phalcom
map<U>(transform: (T) -> U) -> List<U>
where U <: Serializable
{
  ...
}
```

A method's generic binders are owned by its canonical `CallableId`, not the surrounding class declaration.

A class and method may reuse the same source name without identity collision:

```phalcom
class Example<T> {
  convert<T>(value: T) -> T { value }
}
```

The two `T`s are distinct semantic binders.

## 7.4 Getter/setter/indexer staging

The initial generic source surface is required for ordinary method declarations. Generic getters/setters/indexers are not automatically authorized merely because they eventually lower to callable signatures.

They may be added later if their source grammar and type-argument inference/call syntax are unambiguous. The canonical semantic model already permits callable-owned generic signatures, so this is a syntax gate rather than an architectural limitation.

## 7.5 Transparent aliases

Alias declaration grammar:

```ebnf
type-alias-declaration
                    ::= "type" identifier
                        alias-generic-parameters?
                        where-clause?
                        "=" type-form

alias-generic-parameters
                    ::= "<" generic-parameter
                            ("," generic-parameter)* ">"
```

Examples:

```phalcom
type UserId = Int

type Pair<T> = (T, T)

type Named<R: RecordRow> = #{ name: String, | R }

type SerializableResult<T>
where T <: Serializable
= Result<T, Error>
```

Alias formatting should normally keep `=` on the signature line unless multiline layout rules choose otherwise.

---

# 8. Generic constraints: `where` only

## 8.1 Normative grammar

```ebnf
where-clause       ::= "where" generic-constraint
                        ("," generic-constraint)*

generic-constraint ::= type-form "<:" type-form
                     | type-form "==" type-form
```

Examples:

```phalcom
class SortedSet<T>
where T <: Comparable<T>
{
  ...
}
```

```phalcom
class Adapt<A, B>
where A <: B
{
  ...
}
```

```phalcom
class Exact<A, B>
where A == B
{
  ...
}
```

```phalcom
class Sink<T>
where Number <: T
{
  ...
}
```

The last form is a lower bound expressed by the same subtype relation. No second `>:` syntax is required.

## 8.2 Inline bounds are rejected

Rejected:

```phalcom
class SortedSet<T <: Comparable<T>> { ... }
```

Preferred:

```phalcom
class SortedSet<T>
where T <: Comparable<T>
{
  ...
}
```

Reasons:

- one place owns all constraints;
- constraints can relate multiple binders;
- lower bounds do not require asymmetric binder syntax;
- semantic equivalence constraints fit naturally;
- higher-kinded constraints are not forced into a per-parameter field;
- reflection can expose constraints from the owning `GenericSignature` instead of pretending every relation belongs to one parameter;
- future diagnostics can point to whole relations and evidence paths.

A parser encountering an inline `<:` inside a binder should emit a targeted diagnostic with a code action to move the relation to `where` when mechanically possible.

## 8.3 `<:` semantics

`A <: B` is the semantic subtype constraint owned by 01.5.

For the initial generic calculus, subtype operands must normalize to proper `Type` forms. A kind mismatch is an invalid constraint, not merely a false relation.

Ordinary F-bounds are allowed:

```phalcom
where T <: Comparable<T>
```

Their recursive relation evaluation inherits Spec 01 budgets/cycle handling. F-bounds are no longer a reason to keep ordinary generic constraints in Spec 05.

## 8.4 `==` inside `where`

Inside a `where` clause:

```phalcom
T == U
```

denotes **semantic type-form equivalence**, not an ordinary runtime `==(_)` message send.

Operands must have compatible kinds. Constructor-kinded forms may participate where 01.5 equivalence allows it:

```phalcom
where F == <T> =>> List<T>
```

provided `F` has kind `Type -> Type`.

Outside `where`, ordinary expression `==` keeps ordinary language semantics. The contextual meaning does not create a new runtime selector or change selector identity.

## 8.5 Finite exact-set constraints are removed from the initial grammar

The previous form:

```phalcom
where T in (Int, Float)
```

is not part of this revision.

The language does not reserve a generic-constraint AST variant or metadata enum tag for it. If a future requirement justifies finite-domain constraints, it must define inference, normalization, reflection, and interaction with unions/protocols before source syntax is accepted.

## 8.6 Constraint ordering

Source order is preserved for:

- diagnostics;
- formatting;
- source navigation;
- provenance.

Canonical semantic equivalence of a signature may normalize/reorder constraints where order is not semantically meaningful. The syntax tree must therefore not be used as the canonical semantic identity.

---

# 9. Type lambdas

## 9.1 Syntax

Ratified:

```phalcom
<T> =>> Result<T, Error>
<A, B> =>> Pair<A, B>
<F: Type -> Type, T> =>> F<T>
```

The operator is exactly:

```text
=>>
```

not `=>`, `->`, or `>>`.

## 9.2 Parameter rules

Type-lambda parameters:

- default to kind `Type` when omitted;
- may carry explicit kind annotations with `:`;
- may not carry `+`/`-` variance;
- may not carry inline bounds;
- do not accept a lambda-local `where` clause in the first tranche.

A lambda body may reference constrained generic parameters from an enclosing declaration or callable.

## 9.3 Semantic identity

Source names are provenance only:

```text
<T> =>> List<T>
```

and:

```text
<U> =>> List<U>
```

must lower to alpha-equivalent canonical forms.

The AST therefore stores binder names/ranges, but lowering into the 01.5 scoped lambda representation assigns positional/scoped bound-variable identity.

## 9.4 Beta reduction and immediate application

```phalcom
(<T> =>> Result<T, Error>)<Int>
```

normalizes semantically to:

```phalcom
Result<Int, Error>
```

The parser does not perform beta reduction. It only preserves the lambda and application syntax. Semantic lowering/application performs capture-avoiding substitution.

## 9.5 Partial application

```phalcom
(<A, B> =>> Pair<A, B>)<Int>
```

has residual kind `Type -> Type` and is equivalent to a residual lambda such as:

```phalcom
<B> =>> Pair<Int, B>
```

No source rewrite is required. Formatter/reflection may display either canonical equivalent representation only if the output mode clearly states it is semantic rendering rather than source preservation.

## 9.6 Tokenization and selector safety

Phalcom's operator/selector space must not be globally damaged merely to add a type-lambda token.

Implementation requirement:

- the lexer may continue producing an operator token carrying the exact lexeme `=>>`;
- the parser recognizes that lexeme as the type-lambda arrow only in the type-lambda production;
- an existing/future ordinary selector with the same characters outside type-lambda syntax is not silently reinterpreted in unrelated expression positions unless a separate language-level reservation decision is made.

The parser must use longest-match/context information without rewriting all `>>` operator behavior globally.

## 9.7 Recovery

Required malformed cases include:

| Input | Required diagnostic/recovery |
|---|---|
| `<T =>> T` | missing `>`; retain lambda binder and recover at `=>>` |
| `<T> => T` | expected `=>>`; do not reinterpret as block arrow |
| `<+T> =>> T` | variance not allowed on type-lambda binder |
| `<F:> =>> F` | missing kind expression |
| `<T> =>>` | missing body; invalid body node with lambda range retained |
| `<T,> =>> T` | missing binder after comma or explicit trailing-comma policy diagnostic |

Recovery must never lower a malformed lambda to `Dynamic` or a successful canonical form.

---

# 10. Partial application and proper-position checking

## 10.1 Source permits constructor-kinded forms where a constructor is expected

Examples:

```text
Map                    :: Type -> Type -> Type
Map<String>            :: Type -> Type
Map<String, Int>       :: Type
```

Valid higher-kinded use:

```phalcom
class Uses<F: Type -> Type> { ... }

Uses<Map<String>>
```

Invalid value annotation:

```phalcom
value: Map<String>
```

because the annotation position requires kind `Type`.

## 10.2 Proper-position table

| Position | Required semantic category |
|---|---|
| local/field/parameter annotation | proper `Type` |
| callable result | proper `Type` |
| tuple element | proper `Type` |
| callable parameter type | proper `Type` |
| union member | proper `Type` |
| record field | proper `Type` |
| superclass template after normalization | admissible proper nominal type |
| application origin | applicable kind |
| application argument | corresponding parameter kind |
| generic binder kind annotation | admitted `Kind` |
| `<:` operands | proper `Type` in initial calculus |
| `==` operands | compatible kinds |
| alias body | any kind derivable from declaration signature |
| type-lambda body | any admitted result kind |
| explicit type-form value | any publishable type form |

The parser does not hard-code this whole table. Semantic lowering receives an expected semantic category and reports a precise mismatch.

## 10.3 No eager intermediate application requirement

Source syntax must not force implementation allocation.

For:

```phalcom
Map<String, Int>
```

the implementation may directly normalize/inter intern the final spine rather than creating a temporary `Map<String>` node.

Likewise lambda application may beta-reduce without interning an intermediate `Applied(TypeLambda, ...)` node when not useful.

---

# 11. Type forms in value space

This is the most important correction to the previous Spec 04's “non-goal: runtime type expressions” wording.

Phalcom does allow type forms to become runtime values for reflection. What remains forbidden is making **semantic type normalization** depend on ordinary user-overridable runtime dispatch.

## 11.1 Three cases

```phalcom
const xs: List<Int> = ...
```

The annotation is compile-time semantic information. No runtime descriptor allocation is implied.

```phalcom
const listType = List<Int>
```

The expression denotes a type form in value position. The compiler normalizes `List<Int>` semantically. If the value must exist at runtime, the runtime registry lazily materializes/reuses an appropriate representation.

```phalcom
const ResultOf = <T> =>> Result<T, Error>
```

The lambda is a type form. It is compiler-authoritative and may be reified as a `TypeLambda` descriptor if it escapes to runtime value space.

## 11.2 Bare nominal class objects remain ordinary runtime values

```phalcom
const listType = List
```

uses the existing class object. No synthetic `NominalTypeDescriptor` object is required solely to represent the nominal form. Reflection may adapt the class object as a nominal type form according to Spec 03.

## 11.3 Value-space angle application disambiguation

`List<Int>` collides syntactically with ordinary `<`/`>` operator selectors if the expression parser is allowed to treat every angle sequence ambiguously.

The ratified initial disambiguation rule is lexical and deterministic:

> In ordinary expression mode, postfix type-form angle application is recognized only when the `<` token is directly adjacent to its eligible origin syntax with no intervening trivia and the parser can consume a balanced type-form argument list ending in `>`.

Thus canonical formatting is:

```phalcom
List<Int>
Map<String, Int>
```

while ordinary comparison/operator style is:

```phalcom
a < b
x > y
```

The compact ambiguous spelling:

```phalcom
a<b>
```

is reserved to postfix angle application if it matches the balanced application grammar. If `a` does not denote a type form, semantic lowering reports `TypeFormValueNotApplicable`; it does not backtrack into a different operator-send parse based on name resolution.

This is deliberate. Grammar must not change meaning depending on imports or whether an identifier later resolves to a class.

Inside an already-entered type-form context, whitespace around `<` is not semantically significant; formatter still emits canonical `Origin<Args>`.

## 11.4 Eligible value-space origins

The initial parser recognizes postfix type-form value application on:

- a static reference-shaped expression (`List`, `module.Type`);
- a parenthesized type-form value expression;
- a type-lambda expression.

It does not reinterpret an arbitrary runtime call result followed by `<...>` as static type application:

```phalcom
makeSomething()<Int>   // not initial type-form value syntax
```

A future reflection API may provide explicit dynamic type-form application for runtime-produced descriptor values. That is a runtime API concern, not compiler annotation semantics.

## 11.5 Root structural forms in ordinary expression grammar

This revision does **not** globally reinterpret every ordinary tuple, `|` send, record literal, or callable expression as a type-form literal merely because its operands happen to be class/type objects.

That would make parse/semantic category depend on runtime-looking expression forms and would collide with ordinary value semantics.

The initial direct value-space syntax therefore guarantees:

- bare nominal class objects;
- angle-applied type forms;
- type lambdas;
- all structural forms nested inside their type-form arguments/bodies.

Spec 03 may expose non-overridable/core `TypeForm` construction operators/builders for directly constructing a root union/tuple/record/callable descriptor in ordinary value code. If a future source shorthand is added, it must preserve the compiler-authoritative semantics and operator compatibility rules stated here.

## 11.6 No ordinary user-overridable `<>` decides semantic application

`List<Int>` may be explained conceptually as application/`<>(_)`, but the compiler does not resolve a normal user method and ask it what the type means.

Consequences:

- semantic analysis is deterministic;
- a user cannot redefine `List<Int>` by adding a method;
- compiler/LSP/CLI agree without executing the program;
- static application can be normalized without allocating runtime objects;
- the runtime reflection boundary can still expose an application operation whose result agrees with canonical semantics.

---

# 12. `Self`, `Dynamic`, `Never`, `Any`, and `Unknown`

## 12.1 `Self`

`Self` is contextual syntax, not a global declaration lookup.

Lowering requires an explicit owner and side:

- instance-side class member → owner-relative instance type under current generic environment;
- class-side member → owner-relative class-object semantic form;
- future protocol-like declarations → feature-specific conforming receiver form after that feature is ratified;
- outside an owning declaration → invalid.

Generic inheritance/specialization applies `Self` through the environment defined by 01.5.

`Self` is not implemented by reading `value.class` during static analysis.

## 12.2 `Never`

`Never` is the canonical bottom proper type and may be written directly.

It does not imply:

- purity;
- totality;
- non-return due only to exception;
- any specific effect set.

## 12.3 `Dynamic`

`Dynamic` is an explicit dynamic/open-world boundary, not a synonym for missing type information.

A written `Dynamic` must remain distinguishable from:

- no annotation;
- unresolved annotation;
- invalid type application;
- cancellation;
- budget exhaustion.

The exact internal representation may remain a knowledge state rather than an ordinary `TypeData` node if the semantic architecture chooses that model. Syntax does not force a `TypeId` merely because a word is source-spellable.

## 12.4 `Any`

`Any` remains reserved but inactive. The parser should produce a targeted “reserved type surface not enabled” diagnostic rather than resolve an arbitrary declaration named `Any` inside type context once the reservation is active.

No implementation may accept `Any` merely as a fallback for unknown/incomplete analysis.

## 12.5 `Unknown`

`Unknown` is rejected as source type syntax.

A diagnostic should explain the distinction:

```text
Unknown is an analysis state, not a source type.
Use Dynamic to declare an intentional dynamic boundary.
Omit the annotation if the program should infer or leave it unspecified.
```

Legacy native metadata `Unknown` is migrated by metadata adapters into an explicit metadata knowledge state; it is not interned as a canonical source type.

---

# 13. Transparent aliases

## 13.1 Semantic role

A transparent alias has stable declaration identity for:

- source navigation;
- references;
- invalidation;
- diagnostics;
- provenance;
- metadata source identity if retained.

It does not create:

- a runtime class;
- a constructor identity;
- a different object layout;
- a distinct nominal subtype;
- a distinct runtime allocation identity.

Semantic equivalence expands aliases through the bounded cycle-aware alias query described by 01.5.

## 13.2 Generic aliases

```phalcom
type Pair<T> = (T, T)
```

creates a generic alias form whose binder kind defaults to `Type`.

```phalcom
type Mapper<F: Type -> Type, T> = F<T>
```

uses the same explicit kind grammar as class/method binders.

Constraints belong to the alias signature:

```phalcom
type SortedPair<T>
where T <: Comparable<T>
= (T, T)
```

## 13.3 Alias kind

The alias body's kind and binders determine the alias constructor kind. Alias right sides are not forced to `Type` merely because many aliases denote proper types.

Example:

```phalcom
type Fallible<E> = <T> =>> Result<T, E>
```

may denote a constructor of kind `Type -> Type` after normalization.

## 13.4 Recursive aliases

Recursive transparent aliases remain rejected until guardedness/recursive-type semantics are ratified. Parser syntax may represent the self-reference normally; semantic publication rejects the cycle with a bounded diagnostic.

## 13.5 Opaque/newtype aliases

No opaque/newtype spelling is introduced by `type`.

`type` always means transparent alias in this revision. A future opaque/newtype feature requires separate syntax and representation/runtime semantics.

---

# 14. Target syntax model

## 14.1 Share a type-form syntax tree instead of duplicating annotation and value-form grammar

The current public AST names the structure `TypeAnnotation`. That is acceptable during migration, but the target conceptual model is a reusable source syntax tree:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct TypeSyntax {
    pub expr: TypeSyntaxExpr,
    pub range: SourceRange,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeSyntaxExpr {
    Reference(StaticSymbolRef),
    Application {
        origin: Box<TypeSyntax>,
        arguments: Vec<TypeSyntax>,
        range: SourceRange,
    },
    Union {
        members: Vec<TypeSyntax>,
        range: SourceRange,
    },
    Tuple {
        elements: Vec<TypeTupleElementSyntax>,
        range: SourceRange,
    },
    Callable {
        parameters: Vec<TypeCallableParameterSyntax>,
        result: Box<TypeSyntax>,
        range: SourceRange,
    },
    Unit { range: SourceRange },
    Dynamic { range: SourceRange },
    Never { range: SourceRange },
    SelfType { range: SourceRange },
    Record {
        fields: Vec<RecordTypeFieldSyntax>,
        tail: Option<RecordRowTailSyntax>,
        range: SourceRange,
    },
    TypeLambda {
        parameters: Vec<TypeLambdaParameterSyntax>,
        body: Box<TypeSyntax>,
        range: SourceRange,
    },
    Invalid {
        diagnostic: ParseDiagnosticId,
        range: SourceRange,
    },
}
```

Exact Rust type names may remain `TypeAnnotation*` during staged migration to avoid unnecessary churn. The invariant is that annotation positions and explicit type-form value expressions reuse the **same structural type syntax**, not two independent grammars that can drift.

## 14.2 Generic syntax nodes

Target syntax-only records:

```rust
pub enum VarianceSyntax {
    Invariant,
    Covariant,
    Contravariant,
}

pub struct GenericParameterSyntax {
    pub variance: VarianceSyntax,
    pub name: String,
    pub name_range: SourceRange,
    pub kind: Option<KindSyntax>,
    pub range: SourceRange,
}

pub enum KindSyntax {
    Type(SourceRange),
    RecordRow(SourceRange),
    Arrow {
        parameter: Box<KindSyntax>,
        result: Box<KindSyntax>,
        range: SourceRange,
    },
    Grouped {
        inner: Box<KindSyntax>,
        range: SourceRange,
    },
    Invalid {
        diagnostic: ParseDiagnosticId,
        range: SourceRange,
    },
}

pub enum GenericConstraintSyntax {
    Subtype {
        lower: TypeSyntax,
        upper: TypeSyntax,
        range: SourceRange,
    },
    Equivalent {
        left: TypeSyntax,
        right: TypeSyntax,
        range: SourceRange,
    },
    Invalid {
        diagnostic: ParseDiagnosticId,
        range: SourceRange,
    },
}

pub struct WhereClauseSyntax {
    pub constraints: Vec<GenericConstraintSyntax>,
    pub range: SourceRange,
}
```

These records contain no `TypeId`, `KindId`, `TypeParameterId`, `CallableId`, solver variable, or runtime object reference.

## 14.3 Declaration AST additions

Conceptual target:

```rust
pub struct ClassDef {
    pub name: String,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub superclass: Option<TypeSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    // existing fields...
}

pub struct MethodDef {
    pub name: String,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub params: Vec<ParameterDef>,
    pub return_annotation: Option<TypeSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    // existing fields...
}

pub struct TypeAliasDef {
    pub name: String,
    pub generic_parameters: Vec<GenericParameterSyntax>,
    pub where_clause: Option<WhereClauseSyntax>,
    pub body: TypeSyntax,
    pub range: SourceRange,
}
```

Aliases use the same `GenericParameterSyntax` type but the parser rejects non-invariant variance spelling in alias context. Method/type-lambda contexts likewise reject it.

## 14.4 Type-form expression AST

The expression AST needs an explicit semantic island rather than encoding applied type forms as ordinary method calls:

```rust
pub enum Expr {
    // existing variants...
    TypeForm(TypeSyntax),
}
```

or an equivalent wrapper with source range/provenance.

A bare class reference remains an ordinary class-object expression. `Expr::TypeForm` is required for structures such as angle application/type lambda whose meaning must be compiler-authoritative.

---

# 15. Parser architecture and recovery

## 15.1 Do not create one parser for source and native metadata

Source parser requirements:

- exact source ranges;
- error recovery;
- continuation after malformed nested forms;
- IDE-stable syntax trees;
- trivia/adjacency inspection for value-space angle application;
- contextual tokens in surrounding language grammar.

Native metadata parser requirements:

- compact deterministic schema text;
- strict failure;
- no source recovery tree;
- no editor source ranges.

They may share test vectors and lowering concepts, not parser state machines.

## 15.2 Precedence-parser shape

Recommended source parser entry points:

```text
parse_type_form
parse_type_lambda_or_union
parse_union_type
parse_callable_type
parse_postfix_type
parse_type_atom
parse_type_arguments
parse_kind_expression
parse_generic_parameters(context)
parse_where_clause
```

The `context` parameter for generic binders states whether variance is permitted:

```rust
pub enum GenericBinderContext {
    NominalDeclaration,
    Callable,
    Alias,
    TypeLambda,
}
```

This prevents later semantic code from having to recover from a parser that eagerly treats `+` as valid everywhere.

## 15.3 Recovery synchronization

Within type forms, synchronization candidates include:

- `,`;
- `)`;
- `>`;
- `}`;
- `|` where appropriate;
- `->`/`=>>` when the partial structure makes recovery unambiguous;
- `where`, `=`, `{` at declaration-signature boundaries;
- outer declaration/expression recovery sets after the type form ends.

Every recovery branch must guarantee forward progress.

## 15.4 Required malformed examples

| Input | Required behavior |
|---|---|
| `List<>` | missing argument node; consume `>` and retain application |
| `Map<String,>` | missing final argument; retain prior argument/range |
| `List<List<Int>>` | parse successfully even if lexer normally tokenizes `>>` |
| `(Int -> String` | diagnose missing `)`/domain boundary; do not swallow following declaration |
| `#{ name String }` | expected `:`; retain field name and invalid type child |
| `#{ name: String | R }` | parse as field union; do not guess row tail |
| `class Box<+>` | missing binder name; recover to comma/`>` |
| `class Box<T <: Number>` | targeted inline-bound diagnostic; recover binder and continue |
| `class Functor<F :: Type -> Type>` | targeted stale kind-ascription diagnostic with `:` fix-it |
| `where T in (Int, Float)` | unsupported/deferred constraint diagnostic; do not create finite-set semantic node |
| `<T> => T` | expected `=>>` |
| `<+T> =>> T` | invalid variance on lambda binder |
| `F<_>` | rejected placeholder application/inference-hole diagnostic |

## 15.5 Nested `>` token fission

Type mode must parse:

```phalcom
List<List<Int>>
Map<String, List<Option<Int>>>
```

without changing ordinary `>>` selector/operator lexing globally.

Preferred strategy:

- inspect the current operator token lexeme/range;
- when type mode needs `>`, consume one leading `>` and retain/fission the remainder with adjusted range;
- test exact ranges at every nesting depth.

The same principle applies if `=>>` and `>>` share lexer machinery.

## 15.6 No semantic backtracking

The parser must not choose between “generic type application” and “comparison send” after consulting name resolution.

That would make grammar depend on imports and create incremental instability.

Value-space adjacency rules in §11 resolve the only initial angle ambiguity syntactically.

---

# 16. Semantic lowering pipeline

## 16.1 Staged lowering

Normative pipeline:

```text
source syntax
  -> recoverable TypeSyntax / declaration syntax
  -> allocate declaration/callable generic binder identities
  -> lower explicit binder kinds
  -> lower type forms into canonical/scoped semantic forms
  -> lower `where` relations owned by GenericSignature
  -> validate proper-position/kind constraints
  -> validate declaration variance
  -> lower/validate generic superclass template
  -> publish declaration/callable interface
  -> check bodies using published signatures
```

This ordering mirrors Spec 01.5 and prevents body checking from repeatedly re-resolving annotations.

## 16.2 Binder allocation before constraint/body lowering

All binders in a declaration signature receive semantic identities before any signature type that may reference them is lowered.

For:

```phalcom
class Pair<A, B>
where A <: B
{
  ...
}
```

lowering performs:

```text
allocate A(owner=Pair,index=0)
allocate B(owner=Pair,index=1)
then lower A <: B
```

not name-keyed substitution after the fact.

Method generic parameters use callable owner identity.

Type-lambda parameters do **not** allocate ordinary declaration `TypeParameterId`s; they lower to the scoped alpha-normalized representation from 01.5.

## 16.3 Result algebra

Source lowering must use explicit result states compatible with Spec 01. A representative shape is:

```rust
pub enum AnnotationStatus {
    Resolved {
        form: TypeId,
        kind: KindId,
    },
    Dynamic {
        reason: DynamicReason,
    },
    Missing,
    Unresolved {
        name: StaticNameId,
    },
    Invalid {
        diagnostics: Box<[DiagnosticId]>,
    },
    Blocked {
        reason: BlockReason,
    },
    Cancelled,
    BudgetExceeded {
        report: BudgetReport,
    },
    InternalFailure {
        incident: IncidentId,
    },
}
```

Exact shared enum names are owned by Spec 01 implementation. This document requires the distinctions, not duplicate incompatible enums.

No invalid written type may become `UnknownReason::UnannotatedDeclaration` as a successful compatibility shortcut.

## 16.4 Expected-kind lowering

Lowering APIs should take an explicit expectation rather than parse separate grammars for every position:

```rust
pub enum TypeExpectation {
    AnyForm,
    ProperType,
    Constructor(KindId),
    SupertypeTemplate,
    RecordRow,
}
```

A semantic implementation may encode expectations differently. The required behavior is that diagnostics can state both actual and expected kind/category.

## 16.5 Static name/binder resolution order

Within a type form:

1. contextual atoms (`()`, `Never`, `Dynamic`, `Self`, later `Any`);
2. lexical type-lambda binders when inside lambda scope;
3. lexical callable generic parameters;
4. lexical declaration generic parameters;
5. local/static alias/declaration references under module rules;
6. qualified static module references;
7. unresolved result.

Shadowing diagnostics/policies must be explicit. A callable binder may intentionally shadow a class binder by name because owner/index identity keeps them distinct; IDE hover/go-to-definition must show which binder was selected.

## 16.6 Constraint lowering

A `where` clause lowers only after all owning binders exist.

```phalcom
where T <: Comparable<T>, F == <U> =>> List<U>
```

becomes signature-owned canonical relation records. It does not mutate fields on `TypeParameterData` to attach “the bound”.

## 16.7 Type-lambda lowering

Parser binder names/ranges lower into:

- binder kind vector;
- scoped positional bound variables;
- scoped body;
- result kind;
- separate provenance names/ranges.

Alpha-renaming cannot alter canonical semantic identity.

## 16.8 Superclass-template lowering

For:

```phalcom
class Names<T> is Sequence<Option<T>> { ... }
```

publication records the unspecialized template:

```text
Names<T> -> Sequence<Option<T>>
```

rather than reducing hierarchy to `Names -> Sequence` only.

Runtime class inheritance remains `Names is Sequence` under the existing object model; no `Sequence<Option<String>>` runtime class is created when specializing `Names<String>`.

## 16.9 Aliases

Alias lowering records:

- alias declaration identity;
- binders/kinds;
- signature-owned constraints;
- normalized body form;
- source provenance/dependencies;
- transparent status.

Semantic expansion is bounded/cycle-aware and records dependency on alias identity/body.

## 16.10 Records/rows

Known record field syntax preserves source order/ranges. Canonical semantic record representation may sort fields.

Duplicate fields generate diagnostics at all relevant source ranges. The canonical semantic node must not contain duplicate ambiguous fields merely because parser recovery retained them.

---

# 17. Generated declarations and source lowering

Compiler transforms/attributes that derive declarations from typed source must preserve semantic signatures.

Example:

```phalcom
@get
const _name: String
```

If an attribute expander synthesizes a getter AST without copying `return_annotation`, the semantic declaration builder must still publish the exact derived signature:

```text
name -> String
```

Likewise generated setters/constructors/data methods/variant accessors must not degrade exact source type knowledge to “unannotated” merely because the generated intermediate AST omitted a textual annotation.

This requirement is semantically owned by 01.5, but Spec 04 owns the source-to-generated provenance expectation: the lowering pipeline must preserve a path from source field/payload signatures into generated callable semantic records.

Do not solve this by serializing/re-parsing synthetic type source strings.

---

# 18. Native/source convergence

## 18.1 Separate front ends, shared semantic target

Source:

```phalcom
List<Int>
```

Native metadata textual syntax expressing the same form must lower to the same store-independent semantic structure/fingerprint before store interning.

This does not require identical AST types.

## 18.2 Native metadata must grow with the semantic model

When 01.5 generic features become publishable, native metadata adapters need equivalents for:

- parameter kind;
- declaration variance where applicable;
- signature-owned subtype/equivalence constraints;
- callable generic binders;
- generic superclass templates;
- `Self`;
- type lambdas or an equivalent canonical serialized lambda form.

Spec 02 owns the durable encoding. This spec only requires that source/native lowering cannot disagree semantically.

## 18.3 Legacy native `Unknown`

`phalcom-type-syntax` currently has `TypeExpr::Unknown`. Migration may decode legacy inputs, but canonical lowering converts that token into an explicit metadata status and never interns it as if it were a normal source/canonical type form.

---

# 19. Formatting and rendering

## 19.1 Canonical source style

- no space between a type application origin and `<`;
- comma + space between type arguments/binders/constraints in single-line form;
- `+T` / `-T` with no internal space;
- generic kind annotation uses `: `;
- spaces around `->`, `|`, `<:`, `==`, `=`, and `=>>`;
- source unit is always `()`;
- open row is `#{ field: T, | R }`;
- `where` constraints may wrap one per line in multiline declarations;
- no source formatter emits binder `::`;
- no source formatter emits inline `<:` bounds;
- no source formatter emits `F<_>`;
- no source formatter emits finite-set `in (...)` constraints.

Examples:

```phalcom
class Functor<F: Type -> Type, +T>
where T <: Serializable, F == <U> =>> List<U>
{
  ...
}
```

The ordering above is syntactically legal only if `T` is a nominal declaration parameter; `F` is invariant because it lacks a sign.

## 19.2 Source-preserving versus semantic rendering

Source-preserving formatter:

- keeps binder names;
- preserves comments;
- can preserve omitted default `: Type`;
- does not alpha-rename lambdas;
- does not reorder written unions/constraints solely because semantic canonicalization does.

Semantic/reflection renderer may:

- show explicit kinds;
- show normalized unions;
- show qualified declaration owners;
- alpha-normalize lambda binder names for display;
- show residual/remaining parameters.

Such output must be described as semantic display, not guaranteed source round-trip text.

## 19.3 Expression-space angle spacing

Because §11 reserves adjacent angle syntax for postfix type-form value application, formatter rules are semantically significant at this boundary:

```phalcom
List<Int>   // type-form application
x < y       // ordinary operator send/comparison
```

The formatter must not collapse `x < y` into `x<y>`.

---

# 20. Diagnostics and fix-its

Required diagnostic families:

| Code/family | Trigger | Required information |
|---|---|---|
| `TypeSyntaxParse*` | malformed type grammar | expected form/token and recovery range |
| `TypeSyntaxUnresolved` | unresolved static type/binder name | written path and scope/module |
| `TypeSyntaxNotATypeForm` | runtime/value declaration in type-form position | resolved declaration category |
| `TypeSyntaxExpectedProperType` | constructor-kinded form in value type position | actual and expected kind |
| `TypeApplicationNotApplicable` | origin kind is `Type`/non-applicable | origin + kind |
| `TypeApplicationTooManyArguments` | arity excess | supplied/remaining arity |
| `TypeApplicationArgumentKindMismatch` | argument kind mismatch | argument index, expected/actual kind |
| `GenericDuplicateParameter` | duplicate binder name in same binder list | both ranges |
| `GenericLegacyKindAscription` | `F :: K` in source binder | fix to `F: K` |
| `GenericInlineConstraint` | `<T <: X>` | fix/move to `where T <: X` |
| `GenericVarianceNotAllowed` | `+`/`-` on method/alias/lambda binder | binder context |
| `GenericConstraintKindMismatch` | invalid relation operand kinds | both kinds/relation |
| `GenericConstraintRejected` | concrete application violates `where` | normalized relation + evidence |
| `GenericConstraintDeferred` | partial application leaves relation undecidable | remaining parameters; normally not an error in constructor context |
| `GenericFiniteSetDeferred` | `T in (...)` | feature not in initial grammar; no silent translation |
| `TypeLambdaExpectedArrow` | malformed `=>>` | expected exact operator |
| `TypeLambdaCapture/Internal` | impossible lowering invariant | incident ID; not user-blaming diagnostic |
| `TypePlaceholderSyntaxRejected` | `F<_>` | suggest kind annotation/type lambda |
| `GenericSupertypeInvalid` | superclass template not admissible proper nominal type | normalized form/kind |
| `RecordDuplicateField` | duplicate structural field | all source ranges |
| `RecordTailKindMismatch` | tail is not `RecordRow` parameter | resolved binder/kind |
| `SelfOutsideOwner` | `Self` lacks owner | enclosing syntax context |
| `UnknownIsNotSourceType` | written `Unknown` | explain `Dynamic` vs omission |
| `TypeFormValueNotApplicable` | adjacent `x<...>` origin does not denote type form | origin resolution + syntax range |
| `TypeFormValueUnsatisfiedConstraint` | runtime-valued form statically invalid | same constraint evidence as annotation path |
| `Cancelled`/`BudgetExceeded` | semantic computation stopped | distinct terminal status; no fabricated type mismatch |

## 20.1 High-value migration fix-its

The first implementation should provide mechanical source actions for stale spec-era forms:

```text
F :: Type -> Type
→ F: Type -> Type
```

```text
class Box<T <: Number>
→ class Box<T> where T <: Number
```

For multiple inline bounds, the code action preserves binder order and appends a `where` clause in source order.

No automatic fix converts `T in (Int, Float)` to a union bound because that would not preserve semantics.

## 20.2 Diagnostics are causal, not generic

Do not report every generic failure as “type mismatch”. Prefer:

```text
Map<String> has kind Type -> Type but this field annotation requires Type.
```

or:

```text
Argument 1 to F expected a constructor of kind Type -> Type; Int has kind Type.
```

or:

```text
Constraint T <: Comparable<T> could not be proven because relation evaluation exceeded the configured recursive budget.
```

Cancellation/budget/internal failure must not become a false semantic rejection.

---

# 21. LSP, CLI, compiler, and REPL contract

## 21.1 One formal semantic source

The parser and `phalcom-semantic` produce formal facts once. Consumers adapt them:

```text
source
  → parser TypeSyntax
  → SemanticDb canonical generic facts
      ├→ compiler
      ├→ phalcom check
      ├→ LSP
      └→ REPL
```

LSP advisory inference may supplement presentation but cannot reinterpret generic grammar or produce a competing formal type answer.

## 21.2 LSP behavior

LSP should expose:

- hover for binder kind/variance/constraints;
- go-to-definition for type parameters;
- completion filtered by expected kind inside `<...>` and kind expressions;
- constraint-aware signature help;
- specialized member type display;
- type-lambda hover with semantic kind/body;
- fix-its for `::`, inline bounds, and rejected placeholder syntax;
- source ranges from recovered invalid nodes even before semantics succeeds.

Completion examples:

```text
class Functor<F: |
```

suggests kind atoms such as `Type`, not ordinary runtime declarations.

```text
List<|
```

suggests forms of the expected argument kind when known.

## 21.3 CLI checker

`phalcom check` consumes the same diagnostics/results. There is no separate generic grammar or native-metadata round-trip in the CLI checker.

## 21.4 REPL

The persistent REPL semantic session may accept the same type syntax in later cells. Generic declarations/lambdas are bound to the cell/module snapshot identity, not an ambient mutable global generic application context.

A type-form value printed by the REPL may trigger explicit reflection materialization for display, but merely checking/parsing the cell does not require runtime descriptor allocation.

---

# 22. Performance requirements

Type syntax is not the runtime hot path, but its lowering can become a dominant IDE/checker cost in large generic programs. The design must preserve the performance principles of 01/01.5.

## 22.1 No runtime allocation for ordinary annotations

Parsing/checking:

```phalcom
value: List<Int>
```

must not allocate a runtime descriptor object.

## 22.2 Reuse canonical semantic nodes

Repeated equivalent resolved type forms should intern/reuse canonical IDs where the store architecture permits it. Source syntax/provenance remains separate so canonical interning does not duplicate source strings/ranges.

## 22.3 Avoid reparsing generated/source annotations

Generated declarations must carry semantic/source references, not stringify a type and parse it again.

## 22.4 Lazy specialization

Generic member lookup after syntax lowering should use 01.5 `TypeView`/environment-style specialization, not eagerly rewrite every annotation/member type for every applied receiver.

## 22.5 Query-level dependencies

A changed binder/constraint/alias should invalidate the semantic products that depend on it, not every type syntax node in the workspace.

Important dependency keys include:

- declaration signature identity;
- callable signature identity;
- alias body identity;
- referenced declaration kind/signature;
- superclass template;
- `Self` owner/side;
- constraint dependencies.

## 22.6 Parser linearity

Balanced generic/type parsing must be linear in token count under ordinary recovery. Avoid repeated speculative reparsing of the same angle-bracket region.

Value-space type application disambiguation should use token/range adjacency and balanced-delimiter scanning with memoized/ordinary parser progression, not semantic lookahead.

---

# 23. Implementation sequence

This section is repository-grounded planning guidance, not evidence that implementation exists.

The units are intentionally staged so some work may proceed while Spec 01 is still landing, but semantic publication gates wait for the corresponding Spec 01.5 core.

## Unit S0 — establish the 01/01.5 integration gate

**Purpose:** prevent parser/source work from freezing a semantic representation inconsistent with the in-progress architecture.

**Prerequisites:**

- Spec 01 stable store/snapshot identity boundary;
- proper-type enforcement API direction;
- relation/cancellation outcome model direction;
- agreed location for canonical generic signatures from 01.5-A.

**No code requirement:** this is a branch/rebase/review gate.

**Acceptance:** the parser branch knows which semantic APIs it targets; it does not invent its own `GenericSignature`/constraint store.

## Unit S1 — core type-form parser parity

**Files:**

- `phalcom-ast/src/ast.rs` around `TypeAnnotation`/`TypeAnnotationExpr`;
- `phalcom-ast/src/parser.rs` around `parse_type_annotation`;
- lexer/token helpers only as needed for nested `>` fission;
- registered parser integration tests.

**Implement:**

- application;
- nested application;
- union;
- grouping/tuple/unit;
- callable types;
- `Never`, `Dynamic`, `Self` explicit nodes;
- invalid/recovery nodes;
- exact precedence/ranges;
- nested `>>` handling.

**Do not enable yet:** declaration generics, rows, aliases, type lambdas.

**Tests first:** source snapshots/AST assertions for every grammar example in §5 and malformed cases in §15.

**Verify:** use the repository's registered `phalcom-ast` integration target rather than assuming Cargo auto-discovers tests.

## Unit S2 — explicit source lowering outcomes

**Files:**

- `phalcom-semantic/src/types/annotation.rs`;
- `phalcom-semantic/src/types/evidence.rs` or Spec-01 replacement result modules;
- diagnostics;
- source annotation tests.

**Implement:**

- distinct missing/unresolved/invalid/dynamic/cancel/budget/internal outcomes;
- proper-position expectation;
- explicit unit node lowering;
- remove `UnannotatedDeclaration` fallback for kind/application errors;
- dependency/provenance recording.

**Deletion criterion:** no written invalid annotation is returned as ordinary missing/unannotated knowledge.

## Unit S3 — generic declaration binders and kind syntax

**Depends on:** 01.5-A generic signature data model.

**Files:**

- `phalcom-ast/src/ast.rs` class/method syntax;
- parser class/method signature code;
- `phalcom-semantic/src/types/parameter.rs` adapters to final 01.5 shape;
- declaration interface builder;
- kind lowering/tests.

**Implement:**

- class `<T>`;
- class `+T`/`-T`;
- method `<U>`;
- `F: Type -> Type`;
- duplicate-name diagnostics;
- binder owner/index allocation;
- context rejection of method variance;
- `F<_>` rejection.

**Do not add:** kind variables/public kind polymorphism.

## Unit S4 — `where` constraints

**Depends on:** 01.5 constraint IR and result-rich relations.

**Files:**

- AST `WhereClauseSyntax`/constraint nodes;
- parser declaration-signature code;
- generic signature lowering;
- relation/constraint diagnostics/tests.

**Implement:**

- `<:`;
- `==`;
- lower bounds via operand order;
- ordinary F-bounds;
- kind validation;
- source-order provenance;
- inline-bound migration diagnostic/fix-it;
- finite-set rejection.

**Deletion criterion:** no canonical per-parameter `bounds` field is created to represent these relations.

## Unit S5 — type lambdas

**Depends on:** 01.5-B canonical scoped lambda representation/beta reduction.

**Files:**

- AST type-lambda node;
- type parser;
- semantic lambda lowering;
- application integration;
- source/semantic property tests.

**Implement:**

- `<T> =>> ...`;
- kinded lambda binders;
- nested lambdas;
- immediate/partial application;
- alpha-equivalence test corpus;
- capture-avoidance corpus;
- `=>>` recovery.

**Gate:** do not publicly accept syntax by lowering it to a fake nominal/alias form before canonical lambda support exists.

## Unit S6 — generic superclass templates and `Self`

**Depends on:** 01.5 generic inheritance/Self semantics.

**Files:**

- `ClassDef.superclass` target representation;
- class parser after `is`;
- declaration type table/hierarchy queries;
- override/inheritance tests.

**Implement:**

- `is Sequence<Option<T>>`;
- kind/proper-supertype validation;
- stored unspecialized template;
- inherited specialization;
- class-side/instance-side `Self` source lowering.

**Regression gate:** runtime superclass/class/metaclass model remains unchanged.

## Unit S7 — transparent aliases and record rows

**Alias dependencies:** 01.5 alias query/fingerprint support.
**Row dependencies:** revised Spec 05 row domain.

**Implement aliases:**

- `type Name = form`;
- generic alias binders;
- alias `where` clause;
- transparent expansion/provenance;
- cycle rejection.

**Implement rows only after semantic row support:**

- `#{ ... }` type nodes;
- `#{ ..., | R }`;
- row-kind checking;
- duplicate field diagnostics;
- union/tail disambiguation tests.

## Unit S8 — type-form values

**Depends on:** 01.5 semantic application/lambda and Spec 03 registry bridge only for actual runtime materialization; parser/static semantics may land before complete reflection UI.

**Files:**

- expression AST;
- expression parser adjacency handling;
- semantic expression synthesis for `Expr::TypeForm`;
- compiler constant/materialization bridge later owned with core runtime integration;
- parser/operator regression tests.

**Implement:**

- `const t = List<Int>`;
- `const f = <T> =>> Result<T, Error>`;
- adjacency disambiguation;
- no-trivia formatter rule;
- non-type origin diagnostic;
- zero descriptor allocation when expression is compile-time-elided.

**Regression tests:** ordinary `a < b`, `x > y`, `>>` selectors, nested angle forms, and generic syntax must retain expected parsing.

## Unit S9 — native/source convergence and formatting

**Files:**

- `phalcom-type-syntax/src/lib.rs`;
- `phalcom-native-meta` types/adapters;
- semantic native lowering;
- formatter/printer module when ownership exists;
- cross-front-end tests.

**Implement:**

- equivalent semantic lowering for generic signatures/forms;
- no native `Unknown` canonical type;
- type-lambda native representation only after Spec 02 schema choice;
- formatter rules from §19;
- semantic/source display distinction.

---

# 24. Verification matrix

## 24.1 Parser laws

Required parser/source tests:

- every current AST core type form has a source spelling;
- `()`/group/tuple/callable ambiguities resolve exactly as §5;
- nested `>` parsing preserves ranges;
- `=>>` does not corrupt ordinary `>>`/operator parsing;
- `where` terminates superclass/return type parsing correctly;
- invalid nodes recover without infinite loops;
- source formatting round-trips valid syntax to equivalent syntax trees.

## 24.2 Semantic laws inherited from 01.5 and exercised through source

Source tests must demonstrate:

- kind uniqueness;
- proper-position rejection of unsaturated constructors;
- application flattening equivalence;
- type-lambda alpha equivalence;
- beta reduction;
- capture avoidance;
- owner/index generic parameter identity;
- constraint alpha/name invariance where appropriate;
- variance soundness;
- generic superclass specialization;
- `Self` specialization by owner/side;
- runtime invariance.

## 24.3 Clean/incremental equivalence

For the same source snapshot:

```text
cold parse/lower/check result == incremental recomputation result
```

including:

- generic signature identities;
- normalized type forms;
- diagnostics;
- constraint outcomes;
- source-to-semantic references.

Changing only a binder name should update presentation/provenance without accidentally conflating it with another owner/index binder. Changing kind/variance/constraint/body must invalidate the correct semantic dependencies.

## 24.4 Ecosystem parity

For the same snapshot, compiler, CLI, LSP formal layer, and REPL must agree on:

- resolved type form;
- kind;
- generic signature;
- constraint status;
- diagnostics.

An LSP advisory fact may provide extra UI help but cannot contradict the formal result.

## 24.5 Runtime invariance tests

Adding/removing/changing type annotations or generic metadata must not, by itself, change:

- selector encoding;
- dispatch-table keys;
- runtime instance class;
- class/metaclass count;
- instance layout;
- allocation size for ordinary values.

Explicitly evaluating a type-form value may allocate/reuse a reflection descriptor; ordinary annotations may not.

---

# 25. Acceptance criteria

| Requirement | Required evidence |
|---|---|
| Core syntax is source-spellable | parser + lowering tests for application/union/tuple/callable/unit/atoms |
| Source kind syntax is final | `F: Type -> Type` tests; binder `::` rejected with fix-it |
| Constraints have one home | only `where`; inline-bound tests rejected; signature-owned IR |
| Lower/equality constraints work | `Number <: T`, `T == U`, kind mismatch/result tests |
| Finite-set constraint is absent | grammar rejects `in (...)`; no canonical tag |
| Type lambdas are real | alpha/beta/capture/kind/partial application tests |
| HKT placeholders are absent | `F<_>` rejected; explicit kind/lambda alternatives documented |
| Generic inheritance is source-capable | applied superclass template AST + specialization tests |
| `Self` is owner-relative | instance/class/inherited specialization tests |
| Variance is contextual | class accepts `+/-`; method/alias/lambda reject |
| Type-form values are explicit | `List<Int>`/lambda value tests; no semantic runtime dispatch |
| Annotation failure states are distinct | missing/unresolved/invalid/dynamic/cancel/budget assertions |
| Record rows remain unambiguous | mandatory comma/tail-kind tests after row gate |
| Aliases remain transparent | equivalence/provenance/cycle tests |
| Source/native semantics converge | cross-front-end structural/fingerprint tests |
| Runtime object model is unchanged | invariant/layout/selector/class-count regressions |
| Performance stays lazy | no runtime descriptor allocation for annotation-only workloads; no eager generic-member substitution requirement |

---

# 26. Intentional gates

Implementation must stop before enabling public syntax for:

- `Any` until top-type/lattice interaction is ratified;
- intersections `&`;
- use-site variance;
- method/type-lambda variance;
- finite exact-set constraints;
- arbitrary boolean/negative/associated-type constraints;
- lambda-local `where` clauses;
- public kind variables/kind-polymorphic binder syntax;
- recursive aliases/ADTs without guardedness policy;
- opaque/newtype aliases;
- protocol declarations/coherence;
- typed variant/ADT payload syntax merely to implement reflection result variants;
- effect rows/effect handlers/termination syntax;
- proof terms/prover backend syntax;
- arbitrary runtime-expression type arguments;
- semantic generic application through user-overridable dispatch;
- per-instance generic runtime tokens;
- specialized runtime classes.

These are semantic boundaries, not parser TODOs.

---

# 27. What this design must not preclude

The source/AST architecture must leave room for:

- public kind polymorphism after Spec 05 ratification;
- `Any` as a real proper top type;
- intersection/overload semantics;
- protocol/trait declarations reusing generic signature syntax;
- algebraic-data declarations reusing generic binders without forcing typed variant syntax now;
- guarded recursive aliases/ADTs;
- opaque/newtype aliases;
- associated types/constraints after a coherence model exists;
- separate effect/variant/record row domains;
- richer type-form value construction through Spec 03;
- formatter ownership that preserves comments/recovery;
- metadata schema evolution without changing source grammar;
- runtime reflection that exposes type forms lazily without changing canonical semantics.

It need not preserve compatibility with the stale syntax explicitly removed by §0.

---

# 28. Take / adapt / reject

## 28.1 Take directly from the current repository

- `TypeAnnotationExpr`'s existing application/union/tuple/callable structural shapes;
- static qualified references;
- source ranges on type syntax children;
- current `TypeStore` kind-checked application and residual kinds;
- owner/index generic parameter identity;
- current separation between source parser and `phalcom-type-syntax` native parser;
- compiler-authoritative semantic diagnostics consumed by CLI/LSP.

## 28.2 Adapt

- `TypeAnnotation` into a reusable type-form syntax tree usable by annotations and explicit type-form value islands;
- `ClassDef.superclass` from static symbol reference to generic type-form template syntax;
- `TypeFormResolution` into the Spec-01 result taxonomy;
- current eager source annotation resolver into staged binder/signature lowering;
- generic source parsing to use `:` kinds and `where` constraints;
- current application syntax into type-lambda application and lazy semantic application;
- record syntax into explicit row-tail source form only after semantic row support;
- native parser structures into equivalent semantic targets without merging parser implementations.

## 28.3 Reject

- `F :: Type -> Type` source binder syntax;
- inline `<T <: Bound>` syntax;
- `T in (...)` in the initial generic calculus;
- `F<_>`/`F<_,_>` kind placeholders;
- source `Unknown`;
- string-matching `Unit`/`Dynamic`/`Self` as the final source AST architecture;
- parser semantics depending on whether a name later resolves to a class;
- one parser shared between recoverable source and strict metadata;
- annotations that execute arbitrary runtime code;
- generic type arguments entering selector identity;
- specialized classes/metaclasses for applied types;
- eager runtime descriptor allocation just because a static type is written.

---

# 29. Final normative summary

1. `Type` and arrow kinds classify type forms; source kind ascription uses `:`, while `::` is only semantic judgment notation.
2. A source `type-form` may be constructor-kinded; a runtime-value annotation additionally requires kind `Type`.
3. Generic classes use `<T>`, declaration-site `+T`/`-T`, and optional explicit binder kinds such as `<F: Type -> Type>`.
4. Method, alias, and type-lambda binders do not accept variance markers.
5. Every generic constraint is written in a `where` clause.
6. Initial constraint relations are subtype `<:` and semantic equivalence `==`.
7. Lower bounds use operand order (`Number <: T`); there is no required `>:` surface.
8. Inline bounds are rejected and should receive migration fix-its.
9. Finite exact-set `T in (...)` constraints are deferred and have no initial canonical grammar/tag.
10. Type lambdas use `<T> =>> ...`, are alpha-equivalent by structure, and normalize through capture-avoiding beta reduction in the semantic layer.
11. `F<_>` placeholder HKT syntax is rejected; explicit kinds and type lambdas cover its intended roles.
12. Type application uses `<...>`, is kind-checked, supports partial application, and may flatten/beta-reduce without intermediate allocation.
13. `Map<String>` may be a constructor of kind `Type -> Type` but cannot annotate a runtime value directly.
14. Generic superclass syntax accepts templates such as `is Sequence<Option<T>>`; runtime inheritance remains origin-class based.
15. `Self` is an owner/side-relative semantic form, not a global type name or `value.class` shortcut.
16. Transparent aliases use `type`, retain declaration identity/provenance, and expand semantically; recursive and opaque aliases remain gated.
17. Open structural records retain `#{ fields, | R }` with mandatory comma before a tail after known fields.
18. `Unknown` is not a source type; `Dynamic` is an explicit dynamic boundary; missing/unresolved/invalid/cancelled/budget states remain distinct.
19. Annotation/type-form parsing is compiler-authoritative and side-effect free.
20. Type forms may become runtime values, but that does not turn annotation normalization into ordinary user-overridable dispatch.
21. Bare nominal type values reuse existing class objects; applied/lambda forms are reified lazily only when runtime value materialization is required.
22. In ordinary expression mode, adjacent balanced `Origin<...>` is reserved for postfix type-form value application; ordinary `<`/`>` operator style is spaced and never chosen by semantic backtracking.
23. Source and native metadata parsers remain separate front ends and converge below parsing on one canonical semantic model.
24. Compiler-generated declarations preserve derivable semantic types instead of degrading to unannotated knowledge.
25. Compiler, CLI, LSP formal analysis, and REPL consume the same canonical semantic facts.
26. No static type metadata changes selector identity, runtime class/metaclass identity, object layout, or allocation.
27. No ordinary annotation allocates a runtime type descriptor.
28. Parser recovery preserves invalid syntax/provenance and never fabricates `Dynamic`, `Any`, or a successful canonical type.
29. This spec owns source spelling/recovery/lowering; Spec 01.5 owns generic semantics; Specs 02/03 own metadata/reification/reflection; revised Spec 05 owns only advanced kinds/rows/effects/contracts/proofs beyond the base generic calculus.

---

# 30. Repository touch map

The following map is the expected implementation surface based on the inspected repository. Line numbers are anchors to the inspected snapshot and may move as Spec 01 lands.

| File | Current role | Required revision |
|---|---|---|
| `phalcom-ast/src/ast.rs:406-443` | existing `TypeAnnotationExpr` | explicit unit/dynamic/never/self/record/lambda/invalid nodes; evolve/reuse as `TypeSyntax` |
| `phalcom-ast/src/ast.rs:227-263` | `ClassDef` with static-symbol superclass | generic binders, full type-form superclass template, `where` clause |
| `phalcom-ast/src/ast.rs:~700-727` | `MethodDef` | method generic binders + `where` clause |
| `phalcom-ast/src/parser.rs:1396-1421` | reference-only `parse_type_annotation` | full precedence/recovery type parser |
| `phalcom-ast/src/parser.rs` class/member parsing | current declarations | generic/header/alias/type-form value grammar |
| `phalcom-semantic/src/types/annotation.rs` | current source resolver; coarse unknown fallback | explicit result taxonomy, binder-aware resolution, type lambda/row/alias/supertype lowering |
| `phalcom-semantic/src/types/parameter.rs:1-24` | owner/index/name/kind | consume final 01.5 variance/provenance/signature-owned constraints model |
| `phalcom-semantic/src/types/kind.rs` | `Type` + arrow | lower explicit source kinds; no public kind-variable syntax here |
| `phalcom-semantic/src/types/store.rs` | canonical forms/application | consume 01.5 TypeLambda/row changes when implemented; no parser semantics in store |
| `phalcom-semantic/src/declarations.rs` | declaration type forms/signatures | source generic signatures, supertype templates, aliases |
| `phalcom-semantic/src/checker/declaration.rs` | repeatedly resolves annotations into surfaces | shift to published canonical declaration/callable signatures |
| `phalcom-semantic/src/checker/call.rs` | checks concrete `CallableSignature` | consume specialized generic callable views/inference from 01.5 |
| `phalcom-type-syntax/src/lib.rs` | native metadata parser | retain separate parser; extend only as Spec 02/native schema requires |
| `phalcom-native-meta/src/types.rs` | symbolic native type specs | add final generic semantic vocabulary downstream of 01.5/02 |
| `phalcom-core/src/compiler/attributes.rs` | generated declarations | preserve/derive semantic type signatures; no stringify/reparse |
| `phalcom-lsp/src/semantic/*` | advisory + published formal adapters | consume formal generic facts, never duplicate parser/semantic rules |

---

# 31. Revision decision register

| Decision ID | Decision | Status |
|---|---|---|
| `DEC-TYPE-SRC-01` | Type syntax remains contextual/compiler-authoritative | Ratified |
| `DEC-TYPE-SRC-02` | Source binder kind annotation is `:` | Ratified |
| `DEC-TYPE-SRC-03` | `::` remains semantic kinding-judgment notation only | Ratified |
| `DEC-TYPE-SRC-04` | All generic constraints use `where` | Ratified |
| `DEC-TYPE-SRC-05` | `<:` and contextual `==` are initial constraint relations | Ratified |
| `DEC-TYPE-SRC-06` | Lower bound uses reversed `<:` operands | Ratified |
| `DEC-TYPE-SRC-07` | Finite `in (...)` constraint deferred | Ratified deferral |
| `DEC-TYPE-SRC-08` | Type lambda spelling is `<...> =>> ...` | Ratified |
| `DEC-TYPE-SRC-09` | Method/alias/lambda variance rejected | Ratified |
| `DEC-TYPE-SRC-10` | HKT placeholder `F<_>` rejected | Ratified |
| `DEC-TYPE-SRC-11` | Generic superclass is full type-form template | Ratified |
| `DEC-TYPE-SRC-12` | Type-form syntax tree is reusable across annotations/value islands | Ratified architecture |
| `DEC-TYPE-SRC-13` | Value-space adjacent `Origin<...>` uses lexical adjacency, never semantic backtracking | Ratified initial disambiguation |
| `DEC-TYPE-SRC-14` | Root structural type-form shorthands do not globally hijack ordinary tuple/record/operator expression grammar | Ratified initial boundary |
| `DEC-TYPE-SRC-15` | Source/native parsers stay separate, semantic target shared | Ratified |
| `DEC-TYPE-SRC-16` | Record row spelling remains `#{ fields, | R }` | Ratified; implementation gated |
| `DEC-TYPE-SRC-17` | `type` denotes transparent alias only | Ratified |
| `DEC-TYPE-SRC-18` | Runtime descriptor allocation is not implied by annotations | Ratified |

