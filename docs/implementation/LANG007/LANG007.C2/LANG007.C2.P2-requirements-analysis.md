# LANG007.C2.P2 — Requirements Analysis

## 1. Executive conclusion

`LANG007.C2.P2` should be bounded as:

> **Source Variadic Generic Binding and Structural/Callable Type Expansion**

P2 should make the already-implemented P1 pack semantics reachable from source. It should not add another pack representation, another generic application model, a pack solver, or runtime variadic transport.

The required source-to-semantic flow is:

```text
source syntax
    <*Ts>
    <**Ls>
    <***P>

    Target<*P, **L>
    (*P, x: String, **L)
    (***Q,) -> R
        │
        ▼
explicit lane-aware AST
        │
        ▼
P1 GenericParameterMode
P1 lane projections
P1 TypePackConcat
P1 LabeledTypePackConcat
P1 CompleteTypePack
        │
        ▼
binder-aware source application
        │
        ▼
one canonical argument per declared generic binder
        │
        ├── ordinary TypeData::Applied
        ├── existing TupleFromPacks
        └── callable-domain symbolic formation
        │
        ▼
existing substitution / instantiation / metadata authorities
```

The most important repository finding is that P2 has **two deeper requirements** that were easy to underestimate before P1 landed:

1. Variadic generic application cannot be implemented from an arrow `KindId`. P1 deliberately made binder mode orthogonal to kind, so P2 must recover the actual `GenericSignature` of the application target.
2. Generic type aliases and type lambdas use the separate scoped/de-Bruijn type graph. That graph currently has no pack-composition forms. P2 must extend the scoped representation or pack syntax will work in ordinary annotations but fail inside generic aliases and lambdas.

There is also a third important issue: the current callable type representation cannot faithfully represent symbolic finite pack unpacking. P2 needs a bounded static callable-from-packs representation rather than abusing runtime-style rest parameters.

---

# 2. Current lifecycle truth

The audited lifecycle is now:

```text
LANG007.C2
    IN_PROGRESS
    PARTIAL
    FOCUSED_TESTED

LANG007.C2.P1
    COMPLETE
    IMPLEMENTED
    FOCUSED_TESTED

active plan:
    none

next plan:
    LANG007.C2.P2
```

P1 is therefore frozen substrate.

P2 must consume P1 as implemented, not repeat its design work.

---

# 3. Frozen P1 takeover

P1 now provides all of the following canonical authorities.

| Concept | P1 authority | P2 requirement |
|---|---|---|
| complete pack kind | `KindId::COMPLETE_TYPE_PACK` / `KindData::CompleteTypePack` | consume directly |
| complete pack term | `TypeData::CompleteTypePack { positionals, labeled }` | source `***` lowers here |
| lane projection | `TypeStore::project_positionals`, `project_labeled`, `project_complete` | all type-level unpacking uses these |
| positional composition | `TypePackConcat` / `type_pack_concat` | use for symbolic positional assembly |
| labeled composition | `LabeledTypePackConcat` / `labeled_type_pack_concat` | use for symbolic labeled assembly |
| variadic binder identity | `GenericParameterMode` | parser syntax lowers explicitly to it |
| parameter identity | mode-sensitive `TypeParameterId` versioning | do not reconstruct mode from kind |
| generic signature | aligned `parameter_modes` | application binder reads these |
| checked substitution | `TypeSubstitution::try_apply` | source-reachable C2 terms must use it |
| materialization | fallible pack-aware materialization | preserve errors |
| metadata | schema v3 complete/concat/mode nodes | extend, never replace |
| inference unknown | ordinary kind-aware `InferVarId` | no `PackVarId` |
| generic binding | `GenericInstantiation` | no variadic binding store |
| Tuple formation | `tuple_from_packs` | P2 Tuple expansion terminates here |

P2 must preserve the distinctions:

```text
TypePack != Tuple
LabeledTypePack != RecordRow
CompleteTypePack != Tuple
CompleteTypePack != runtime ArgumentPack
GenericParameterMode != KindId
source expansion count != canonical Applied argument count
```

---

# 4. P2 bounded acceptance objective

P2 should be considered implemented when the compiler can perform all of these source-level operations:

```phalcom
class A<*Ts> {}
class B<**Ls> {}
class C<***P> {}
class D<T, *Ts, **Ls> {}
class E<T, ***P> {}
```

and form applications equivalent to:

```phalcom
A<>
A<Int>
A<Int, String>

D<Int, String, Bool, **L>
E<Int, ***Q>

A<*P>
A<*(P::Map<Option>)>
```

while canonical semantic application remains binder-aligned.

For:

```phalcom
class D<T, *Ts, **Ls> {}
```

a source application with:

```text
positionals = Int, String, Bool
labeled     = L
```

must become:

```text
T  = Int
Ts = TypePack[String, Bool]
Ls = L
```

and ultimately:

```text
Applied(
    D,
    [
        Int,
        TypePack[String, Bool],
        L
    ]
)
```

not an applied type retaining arbitrary source item count.

P2 must additionally support symbolic structural formation such as:

```phalcom
(*P,)
(**L,)
(***Q,)
(Int, *P, name: String, **L)
```

and static callable-domain unpacking:

```phalcom
(*P,) -> R
(**L,) -> R
(***Q,) -> R
```

without requiring known pack cardinality.

P2 does **not** need to infer an unknown prefix/suffix pack from an equation. That remains outside C2.

---

# 5. Source operator law remains absolute

The syntax remains:

```text
*     positional
**    labeled
***   complete
```

This meaning does not change between:

```text
generic binder
generic argument expansion
Tuple type expansion
callable-domain unpacking
```

The grammar changes what is being done with the selected lane; it does not change which lane is selected.

`...` is not generic/type-pack syntax.

In particular P2 must reject:

```phalcom
class C<...Ts> {}
C<...P>
```

as C2 pack syntax.

At the same time, P2 must not globally remove or reinterpret `...`: callable-domain specifications give it a separate role. The existing callable parser currently contains legacy `...` handling, so the parser refactor needs to preserve that separation explicitly.

---

# 6. AST requirements

## 6.1 Do not make `*P` a general unary type expression

Pack expansion is contextual syntax.

The AST should not turn:

```phalcom
*P
```

into a generic unary operator accepted wherever any type form is legal.

That would wrongly permit standalone forms such as:

```phalcom
let x: *P
```

with no unpacking context.

Instead the relevant container owns the expansion.

A reusable syntax payload is appropriate:

```rust
struct TypePackExpansionSyntax {
    mode: ExpansionMode,
    operator_range: SourceRange,
    operand: TypeAnnotation,
    range: SourceRange,
}
```

The existing AST already has:

```rust
ExpansionMode {
    Positional,
    Labeled,
    Complete,
}
```

That is the right common three-lane vocabulary for expansion syntax.

Do not introduce:

```text
GenericExpansionMode
TupleExpansionMode
CallableExpansionMode
```

with the same three meanings.

## 6.2 Generic binders

`GenericParameterSyntax` currently has no pack mode.

It needs to retain:

```text
ordinary binder
* binder
** binder
*** binder
```

plus the operator span independently of `name_range`.

Conceptually:

```rust
GenericParameterSyntax {
    mode: Option<ExpansionMode>, // None == Single
    mode_range: Option<SourceRange>,
    variance: ...,
    name: ...,
    name_range: ...,
    kind: ...,
    range: ...,
}
```

or an equivalent explicit syntax enum.

Semantic lowering then maps this source syntax to P1:

```text
None                -> GenericParameterMode::Single
Positional          -> GenericParameterMode::PositionalPack
Labeled             -> GenericParameterMode::LabeledPack
Complete            -> GenericParameterMode::CompletePack
```

Do not use runtime `RestMode` as generic binder identity.

## 6.3 Generic application entries

The current:

```rust
arguments: Vec<TypeAnnotation>
```

is insufficient.

P2 needs the equivalent of:

```rust
enum GenericArgumentSyntax {
    Single(TypeAnnotation),
    Expansion(TypePackExpansionSyntax),
}
```

This is required even if all expansion semantics later canonicalize away, because the AST must preserve source operator identity and spans.

## 6.4 Tuple type entries

The current:

```rust
TypeTupleElement {
    label,
    ty,
}
```

cannot model unknown-cardinality expansion.

P2 needs a container equivalent to:

```rust
enum TypeTupleItem {
    Element(TypeTupleElement),
    Expansion(TypePackExpansionSyntax),
}
```

`*P` must never be represented as one unlabeled Tuple element whose type happens to be `P`.

## 6.5 Callable-domain entries

Callable type syntax similarly needs an explicit finite pack-unpack item. It must not encode `*P` as the current boolean `rest` flag.

That boolean currently means a rest-like callable parameter and the canonical lowering maps it to positional `RestMode`. It is not a representation of “splice this TypePack into the exact callable domain.”

---

# 7. Generic binder semantics

## 7.1 Operator determines pack domain

The source binder:

```phalcom
*Ts
```

means:

```text
mode = PositionalPack
kind = TypePack
```

Likewise:

```text
**Ls
    mode = LabeledPack
    kind = LabeledTypePack

***P
    mode = CompletePack
    kind = CompleteTypePack
```

The kind must not be inferred from a later use.

The mode must not be reconstructed from the kind.

## 7.2 Explicit kind annotations on pack binders

There is no accepted public spelling for the internal complete-pack kind. Consequently syntax such as:

```phalcom
***P: CompleteTypePack
```

cannot be made uniformly meaningful today.

The cleanest P2 rule is therefore:

> A pack operator completely determines the binder domain; a pack binder should not also carry an explicit `: Kind` annotation.

Thus:

```phalcom
*Ts
**Ls
***P
```

are complete binder declarations.

Ordinary:

```phalcom
P: TypePack
```

continues to mean a `Single` binder accepting one `TypePack` argument.

This also prevents two syntactic authorities from disagreeing.

If the language later deliberately wants redundant annotations such as `*Ts: TypePack`, that should be a separate surface decision, not something the P2 implementer invents.

## 7.3 Variance

Pack binders remain invariant.

Reject:

```phalcom
+*Ts
-*Ts
```

and corresponding labeled/complete forms.

P2 does not define elementwise or length-sensitive pack variance.

## 7.4 Constraints

Pack binders do not create quantified constraint semantics.

Do not reinterpret:

```phalcom
where Ts |= Printable
```

as:

```text
every element of Ts conforms to Printable
```

Existing proper-Type constraint requirements remain authoritative.

---

# 8. Binder ordering and ownership

P2 should make legal binder shapes deterministic by construction.

The canonical declaration grammar is conceptually:

```text
Single*
    followed by either:

    nothing

    *Positional
        optionally followed by **Labeled

    **Labeled

    ***Complete
```

Examples:

```phalcom
C<T, U>
C<T, *Ts>
C<T, **Ls>
C<T, *Ts, **Ls>
C<T, ***P>
```

Illegal:

```phalcom
C<*A, *B>          // two positional owners
C<**A, **B>        // two labeled owners
C<***P, T>         // complete is terminal
C<*Ts, T>          // fixed suffix requires pack splitting
C<**Ls, *Ts>       // lane ordering reversed
C<*Ts, **Ls, ***P> // duplicate ownership
```

These rules are not aesthetic restrictions. They are what prevent source application from becoming a partition-search problem.

The parser may recover structurally, but semantic `resolve_generic_signature` should own the authoritative legality check so every declaration site receives the same rules.

---

# 9. All existing generic declaration sites must be covered

The live AST uses `GenericParameterSyntax` for:

```text
ClassDef
EnumDef
DataDef
TraitDef
TypeAliasDef
ImplDef
VariantDecl
MethodDef
GetterDef
SetterDef
IndexMethodDef
```

P2 should not accidentally create:

```text
class generics support *Ts
method generics support *Ts
type aliases silently ignore it
variant generics parse but lose it
```

The central parser and `resolve_generic_signature` paths should make the mode behavior uniform.

If one site genuinely cannot support variadic binders for an architectural reason, it should reject them with an explicit targeted diagnostic. Silent downgrade to `Single` is unacceptable.

This also means Universe/bootstrap source declaration lowering must use the same source mode when it builds generic parameter identities. It currently derives only kind.

---

# 10. Generic application is not simply “build a TypePack first”

There is an important subtlety.

Existing Phalcom single generic arguments may themselves have non-`Type` kinds:

```phalcom
class Holder<P: TypePack> {}
```

and:

```phalcom
Holder<P>
```

passes **one argument whose kind is `TypePack`**.

It does not mean “expand all elements of `P`.”

Therefore generic source arguments cannot initially be represented as one P1 `TypePack`.

The correct intermediate model is a source contribution stream:

```text
Single(term)
PositionalExpansion(pack-term)
LabeledExpansion(labeled-pack-term)
CompleteExpansion(complete-pack-term)
```

where `Single(term)` may have any legal generic kind.

Only the items ultimately captured by `*Ts` are converted/assembled as a P1 `TypePack`.

This preserves the fundamental distinction:

```phalcom
C<P>      // one argument
C<*P>     // expansion of P's positional lane
```

---

# 11. Binder-aware application algorithm

P2 needs one central binder algorithm.

It should conceptually operate over:

```text
declaration GenericSignature
existing canonical arguments of a partial application
new source contribution items
```

## 11.1 Single binders

A `Single` binder consumes exactly one positional source argument.

That source argument may have any kind compatible with the declared binder:

```text
Type
TypePack
LabeledTypePack
RecordRow
Arrow kind
...
```

## 11.2 Expansion before an unsatisfied Single binder

A symbolic pack must not be split to manufacture one Single argument.

For:

```phalcom
class C<T, *Ts> {}
P: TypePack
```

this is unsupported without decomposition:

```phalcom
C<*P>
```

because obtaining `T` requires taking the head of unknown `P`.

But:

```phalcom
C<Int, *P>
```

is deterministic:

```text
T  = Int
Ts = P
```

If a pack expansion has already reduced to a concrete pack, its elements may be consumed normally because no symbolic decomposition is involved.

## 11.3 Positional pack binder

After all preceding Single binders are satisfied, `*Ts` captures the entire remaining positional contribution stream.

Ordinary remaining `Single` source arguments must then be proper Types because they become TypePack elements.

Symbolic positional expansions remain symbolic segments and feed:

```text
TypeStore::type_pack_concat(...)
```

## 11.4 Labeled pack binder

`**Ls` receives the complete assembled labeled lane:

```text
TypeStore::labeled_type_pack_concat(...)
```

No named generic-argument syntax needs to be invented. P2's labeled generic lane is populated by `**` and `***` source expansions.

## 11.5 Complete binder

`***P` receives:

```text
CompleteTypePack(
    remaining positional lane,
    assembled labeled lane
)
```

through P1's constructor.

## 11.6 Leftover data

Any unowned source contribution is an error:

```text
remaining positionals with no binder
remaining labeled lane with no ** / ***
```

No fallback conversion should occur.

---

# 12. Generic application target must expose binder mode

This is one of P2's most important architecture requirements.

Current application lowering essentially does:

```text
resolve origin
resolve argument forms
read origin arrow kind
apply kind
apply_type_form
```

That is insufficient for C2.

For:

```phalcom
P: TypePack
```

and:

```phalcom
*P
```

the kind information alone cannot distinguish whether the target binder is `Single(TypePack)` or `PositionalPack(TypePack)`.

P2 therefore needs a semantic notion equivalent to:

```text
GenericApplicationContract {
    canonical_origin
    generic_signature, if declaration-backed
    already_consumed_binders
}
```

The exact API may differ.

For nominal declarations the signature is already in `DeclarationTypeTable`.

For transparent aliases the signature already exists in `TypeAliasInfo`, but the current `TypeResolver` exposes an alias form without exposing its generic signature. That seam must be extended rather than making application lowering infer binder mode from the alias's arrow kind.

This lookup must also work for partially applied nominal constructors by recovering the base declaration and number of already-bound canonical arguments.

---

# 13. Preserve existing partial higher-kinded application

P2 should not accidentally make all generic constructors “all or nothing.”

Current type application supports partial application.

For:

```phalcom
class C<T, U, *Ts> {}
```

the intended behavior should remain:

```phalcom
C
    // full constructor

C<Int>
    // residual constructor; U is still unsatisfied

C<Int, String>
    // all fixed prefix binders are now satisfied
    // Ts captures []
    // result is saturated

C<Int, String, Bool>
    // Ts = [Bool]
```

The variadic binder should capture empty only once all required preceding fixed binders have been satisfied.

This gives the crucial distinction:

```phalcom
class V<*Ts> {}

V
    // unsaturated constructor

V<>
    // Ts = [], saturated specialization
```

Explicit `<>` is therefore semantically significant.

---

# 14. Empty application semantics

The parser already preserves:

```phalcom
V<>
```

as an explicit `Application` with zero source arguments.

The missing work is semantic.

Do **not** globally change:

```rust
TypeStore::apply_type_form(origin, &[])
```

because it correctly treats an empty internal application as no application.

Instead binder-aware source elaboration should produce canonical pack arguments before calling it.

For:

```phalcom
class V<*Ts> {}
```

source:

```phalcom
V<>
```

becomes:

```text
canonical args = [TypePack[]]
```

For:

```phalcom
class V<*Ts, **Ls> {}
```

it becomes:

```text
[
    TypePack[],
    LabeledTypePack[]
]
```

For:

```phalcom
class V<***P> {}
```

it becomes one empty complete pack.

In contrast:

```phalcom
List<>
```

must not silently become `List`.

Likewise an explicit empty application that leaves required fixed binders unsatisfied must be rejected rather than collapsing to the unsaturated constructor.

---

# 15. Source expansion phases

Generic and structural pack assembly should reuse the established lane-phase law.

## Positional phase

Allowed:

```text
ordinary positional entry
* expansion
*** expansion
```

## Labeled phase

Begins when a labeled-only contribution is encountered:

```text
explicit label in Tuple-type syntax
** expansion
```

After the labeled phase begins, a source construct that introduces positional data is invalid.

Therefore in Tuple-type syntax:

```phalcom
(Int, *P, ***Q, name: String, **L)
```

is structurally coherent.

But:

```phalcom
(name: String, *P)
```

is not.

`***` contributes to both lanes but does **not**, by itself, force the parser into the labeled phase. This follows the already-ratified source pack model.

Generic applications have no new explicit named generic arguments in P2, so `**` is the operation that begins their labeled-only phase.

---

# 16. Operand validation must be semantic, not syntactic

P2 must use the P1 projection authorities.

Examples:

```text
*TypePack
    -> identity

*CompleteTypePack(P,L)
    -> P

*Tuple
    -> positional Tuple lane

**LabeledTypePack
    -> identity

**CompleteTypePack(P,L)
    -> L

**Tuple
    -> labeled Tuple lane

***CompleteTypePack
    -> identity

***Tuple
    -> complete pair
```

And:

```text
***TypePack
    -> invalid

***LabeledTypePack
    -> invalid
```

Expansion must work on any expression whose semantic term supports the lane.

Therefore:

```phalcom
*(P::Map<Option>)
```

must behave exactly like any other `TypePack`-kinded operand.

Do not special-case bare type-parameter names.

---

# 17. Scoped type forms are a mandatory P2 extension

This is the largest requirement not obvious from the headline feature.

Generic type aliases and explicit type lambdas do not lower bodies directly into ordinary canonical free `TypeId` parameters. They use a scoped/de-Bruijn representation.

Today that scoped representation can express things such as:

```text
Bound
Free
Applied
Tuple
Callable
Record
Lambda
...
```

but it has no scoped equivalents of:

```text
TypePack
LabeledTypePack
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
```

Consider:

```phalcom
type AddHead<*Ts> = (Int, *Ts)
```

Inside the alias body, `Ts` is a scoped bound term. It cannot be handed directly to:

```rust
TypeStore::type_pack_concat
```

because that API accepts canonical `TypeId`, not scoped `ScopedTypeId`.

Therefore P2 must extend the scoped type graph with the equivalent pack-composition forms or an exactly equivalent scoped pack-expression representation.

Beta reduction must map these scoped nodes into P1's canonical nodes.

Required law:

```text
scoped pack operation
    ↓ beta reduction
P1 canonical pack operation
    ↓ canonicalization
existing concrete identity when possible
```

Without this, P2 would only work in non-alias annotations and would be architecturally incomplete.

---

# 18. Type-lambda binder modes must survive partial application

A variadic type alias is normally represented through a type lambda.

If an alias has:

```phalcom
type F<T, *Ts> = ...
```

its generated semantic lambda cannot retain only:

```text
parameter_kinds
```

because:

```text
T  : Type
Ts : TypePack
```

does not reveal whether `Ts` is:

```text
Single(TypePack)
```

or:

```text
PositionalPack(TypePack)
```

P2 must therefore ensure generated/residual type-lambda representation retains parameter modes, or preserve an equivalent application contract across beta reduction.

This matters especially for partial application.

A residual variadic alias constructor must not lose the fact that its remaining TypePack parameter is a pack binder.

User-written ordinary type-lambda parameters remain `Single` unless future syntax explicitly adds variadic type-lambda binders.

---

# 19. Tuple type expansion

P2 Tuple formation should be conceptually:

```text
source Tuple items
    │
    ├── ordinary unlabeled Type -> singleton TypePack
    ├── ordinary labeled Type   -> singleton LabeledTypePack
    ├── *P                      -> project_positionals(P)
    ├── **P                     -> project_labeled(P)
    └── ***P                    -> project_complete(P), split lanes
            │
            ▼
TypePackConcat(...)
LabeledTypePackConcat(...)
            │
            ▼
TupleFromPacks(positionals, labeled)
```

This gives the correct behavior automatically:

- concrete packs collapse to the existing concrete Tuple;
- zero total product becomes Unit;
- symbolic unknown-cardinality packs remain `TupleFromPacks`;
- labels remain ordered;
- duplicate concrete labels fail through P1;
- later substitution can surface deferred duplicate overlap.

There must be no second “variadic Tuple” type.

---

# 20. Callable type unpacking exposes a real representation gap

The live implementation currently models a callable type as roughly:

```rust
CallableType {
    parameters: [CallableParameterType {
        label,
        ty: TypeId,
        rest: RestMode,
    }],
    return_type,
}
```

and source `TypeCallableParameter` carries only:

```text
rest: bool
```

The source lowering requires every parameter `ty` to have proper kind `Type`.

Stable callable type metadata currently reduces rest information even further to:

```text
rest: bool
```

This cannot faithfully represent:

```phalcom
(*P,) -> R
```

when:

```text
P : TypePack
```

because `P` is not a proper parameter Type.

Nor should it be encoded as:

```text
one rest parameter whose type is P
```

because that means something semantically different: an open/homogeneous rest lane, not finite symbolic unpacking of a TypePack.

## Required architecture

P2 needs a proper-Type symbolic callable formation term analogous to C1/P1's `TupleFromPacks`.

Conceptually:

```rust
CallableFromPacks {
    positionals: TypeId, // TypePack
    labeled: TypeId,     // LabeledTypePack
    return_type: TypeId, // proper Type
}
```

Exact naming is flexible.

Its constructor should obey:

```text
positionals : TypePack
labeled     : LabeledTypePack
return      : Type
```

If both lanes are concrete, it should canonicalize to the existing concrete `CallableType` parameter representation.

If either lane is symbolic, it remains a symbolic callable proper Type.

That gives one concrete callable identity rather than:

```text
Callable(...)
and
CallableFromPacks(concrete...)
```

coexisting.

The analogy is deliberate:

```text
TupleFromPacks
    finite structural Tuple formation

CallableFromPacks
    finite callable-domain formation
```

This is considerably cleaner than overloading `RestMode`.

## Scope boundary

P2's responsibility is finite/symbolic **type unpacking**:

```phalcom
(*P,) -> R
(**L,) -> R
(***Q,) -> R
```

It should not absorb runtime rest capture or ABI transport.

Likewise open callable-domain syntax such as homogeneous open lanes is a separate semantic concept. It must not be confused with pack unpacking merely because it uses related tokens.

The existing independent `...` callable shorthand remains outside the pack expansion algebra.

---

# 21. Callable source lowering

Callable exact/unpack items should assemble lanes similarly to Tuple types:

```text
fixed unlabeled type
    -> positional singleton

fixed labeled type
    -> labeled singleton

*P
    -> positional pack contribution

**L
    -> labeled pack contribution

***Q
    -> both lane contributions
```

Then:

```text
callable_from_packs(positionals, labeled, return_type)
```

forms the static callable Type.

This gives correct unknown-cardinality behavior without changing VM argument transport.

Full callable subtyping/inference interaction can be certified in P3, but P2 must create a semantically honest canonical Type. It should not introduce a type that downstream code silently interprets as one fixed parameter.

---

# 22. Stable representation cannot be postponed entirely to P3

P3 owns persistence and incremental **certification**, but P2 source syntax can create new durable semantic forms.

If P2 introduces:

```text
CallableFromPacks
scoped pack concat forms
scoped complete packs
scoped TupleFromPacks
type-lambda parameter modes
```

they cannot be silently non-exportable if they can occur in a public alias/member annotation.

Therefore P2 must provide stable metadata representation for every new canonical/scoped form it introduces.

Given that P1 established metadata schema v3, an additive P2 schema revision—likely v4—would be appropriate if new serialized variants are necessary.

P3 should then prove cross-module and cold/incremental equivalence. It should not have to invent P2's missing stable identity after source syntax is already public.

---

# 23. Exact-enum-case and special generic application paths

`TypeAnnotationExpr::Application` is not the only type-level generic argument carrier.

Exact enum-case syntax has its own:

```text
generic_arguments: Vec<TypeAnnotation>
```

while enum variants may themselves own generic signatures.

If P2 permits:

```phalcom
variant<*Ts>
```

declarations, the explicit exact-case generic-argument path cannot remain permanently fixed-arity.

The same binder-aware generic-argument item representation/algorithm should be reused there where explicit variant generic arguments are semantically supported.

Type-member arguments are a different language mechanism. P2 should not automatically add pack expansion to associated type-member argument lists merely because they also use angle brackets.

---

# 24. Higher-kinded/signature-less constructors

Not every constructor-valued expression has a declaration-backed `GenericSignature`.

A type parameter may simply have an arrow kind:

```text
F : (Type, Type) -> Type
```

Such a constructor has parameter kinds but no pack binder modes.

P2 should preserve ordinary fixed application exactly.

For pack expansion against a signature-less constructor:

- if all expansions become fully concrete and can be flattened to an ordinary positional argument sequence, normal kind application may proceed;
- if an expansion remains symbolic and would require deciding how many fixed parameters it fills, reject it as requiring unsupported pack decomposition/unknown arity;
- labeled contributions have no owner in an ordinary arrow kind and are invalid.

Do not fabricate variadic binder semantics from an arrow kind.

---

# 25. Checked substitution must become source-safe before syntax is exposed

P1 intentionally left:

```rust
TypeSubstitution::apply
```

as a compatibility path and added:

```rust
TypeSubstitution::try_apply
```

for C2 terms.

The current `try_apply` directly handles P1 pack forms, but for other container terms it still falls back to the legacy recursive `apply`.

Once P2 allows pack composition to appear nested inside:

```text
Applied
CallableFromPacks
Tuple
Callable
associated projections
other source-reachable containers
```

that fallback may recurse through a C2 child using the unchecked path.

Therefore P2 must audit and strengthen the checked traversal for every source-reachable container that can contain pack terms.

The required invariant is:

> If substitution can make a previously symbolic labeled composition concretely duplicate a label, no containing type form may suppress that error.

This is a P2 prerequisite, not something safe to postpone until P3.

The same principle applies to Self specialization or other rewriting paths that can encounter source-generated P2 terms.

---

# 26. Source identity and indexing

A declaration:

```phalcom
class C<*Ts> {}
```

still has one generic parameter semantic target:

```text
Ts -> TypeParameterId
```

The `*` token is not a second symbol.

AST should preserve:

```text
operator_range
name_range
whole binder range
```

for diagnostics and tooling.

Source-index collectors must recurse into:

```text
generic expansion operands
Tuple expansion operands
callable-domain expansion operands
exact-case generic expansions
```

so references inside:

```phalcom
*P
**L
***Q
```

retain the same target resolution as ordinary `P`, `L`, and `Q`.

The existing source-input fingerprinting already hashes generic parameter source ranges, but AST/fingerprint visitors still need to understand the new shapes explicitly wherever they structurally traverse annotations.

---

# 27. Presentation requirements

Canonical application may internally be:

```text
Applied(
    C,
    [
        Int,
        TypePack[String, Bool]
    ]
)
```

for source:

```phalcom
C<Int, String, Bool>
```

Diagnostics and hover-style presentation should not suddenly expose:

```text
C<Int, TypePack[String, Bool]>
```

as though that were what the user wrote.

Formatting must understand declaration binder modes.

Likewise internal:

```text
CompleteTypePack(...)
TypePackConcat(...)
LabeledTypePackConcat(...)
CallableFromPacks(...)
```

must not leak raw implementation names into ordinary source diagnostics unless a debug/internal diagnostic explicitly intends to show them.

---

# 28. Diagnostic classes P2 requires

| Situation | Required semantic diagnosis |
|---|---|
| second `*` binder | positional generic lane already has an owner |
| second `**` binder | labeled generic lane already has an owner |
| `***` plus split owner | complete binder conflicts with existing lane ownership |
| binder after `***` | complete pack binder must be terminal |
| Single after a pack binder | would require ambiguous/suffix pack partitioning |
| variance on pack binder | pack binder variance unsupported |
| explicit kind on pack binder | pack operator already determines binder domain |
| `*X` wrong domain | operand has no positional pack projection |
| `**X` wrong domain | operand has no labeled pack projection |
| `***X` wrong domain | operand is not complete-pack/Tuple-shaped |
| symbolic pack before needed fixed binder | application would require pack decomposition |
| labeled lane with no owner | target has no labeled pack binder |
| remaining positionals with no owner | too many generic arguments |
| fixed-only `<>` | required fixed generic argument missing |
| concrete duplicate label | duplicate labeled type contribution |
| duplicate revealed after substitution | instantiated pack composition invalid |
| symbolic expansion into fixed signature-less constructor | unknown arity cannot be split |
| `...` in generic pack context | `...` is not generic/type-pack expansion syntax |

These should use the operator span and operand span separately where possible.

---

# 29. P2 declaration/application compatibility requirements

Existing fixed generic programs must remain unchanged.

In particular:

```phalcom
class Box<T> {}
Box<Int>
```

must follow the current fixed-arity path.

Likewise:

```phalcom
class P<X: TypePack> {}
P<SomePack>
```

continues to pass one TypePack-valued argument.

It must **not** become equivalent to:

```phalcom
class P<*X> {}
P<*SomePack>
```

Partial higher-kinded application must also remain available for fixed binders.

P2 should be additive over the existing application semantics, with variadic behavior activated by explicit binder/argument mode.

---

# 30. What belongs in P3, not P2

P2 should not absorb:

```text
infer a pack from arbitrary runtime call arguments
solve prefix/suffix pack equations
solve Concat(P,Q) = Concrete by partition search
pack length arithmetic
Pack::Length / Pack::Slice
runtime rest-capture migration
frame layout changes
selector/bytecode calling-convention work
VM pack objects
full workspace incremental certification
large-scale persistence stress testing
release certification
general LSP UI
```

P3 should certify:

```text
whole-pack inference using P2 source forms
cross-module metadata round-trip
cold/incremental parity
mode/source edit invalidation
dense symbolic pack behavior
fixed-generic compatibility at checkpoint breadth
runtime-negative certification
```

P2 may add metadata structures necessary for correctness; P3 certifies them across the full product.

---

# 31. Principal failure modes

Several implementations could look plausible while being architecturally wrong.

### Failure A — infer pack mode from kind

```text
kind == TypePack
=> assume variadic
```

Wrong because `P: TypePack` is a Single binder.

### Failure B — parse `*P` as a unary type operator

This makes unpacking legal outside unpack contexts and loses container-specific phase rules.

### Failure C — flatten all generic source arguments immediately into TypePack

Wrong because ordinary generic arguments may themselves have kinds other than `Type`.

### Failure D — eagerly expand symbolic packs to `Vec<TypeId>`

This rejects the primary feature: unknown-cardinality source forwarding.

### Failure E — let a symbolic expansion satisfy a fixed binder by taking its head

That silently introduces pack decomposition inference.

### Failure F — use `RestMode` for callable type pack unpacking

That conflates exact finite pack expansion with an open rest lane and cannot represent TypePack operands honestly.

### Failure G — implement only canonical `TypeId` paths

This breaks generic type aliases/type lambdas because scoped bound pack terms need their own representation.

### Failure H — lose binder mode during type-alias beta reduction

A partially applied variadic alias would turn back into an ordinary kind-only constructor.

### Failure I — leave `try_apply` shallow

A duplicate label can become concrete inside `Applied` or another container and be silently hidden by legacy `apply`.

### Failure J — make `V<>` call `apply_type_form(origin, &[])`

That collapses the explicit empty specialization back into the unsaturated constructor.

### Failure K — create a generic-application pack subsystem

P1 already owns canonical pack composition. Source application should assemble into P1 terms.

---

# 32. Recommended architectural decomposition inside P2

The implementation plan should probably organize P2 into roughly these dependency layers:

```text
1. AST/parser source modes
   ↓
2. semantic generic binder formation and legality
   ↓
3. scoped pack-expression support
   ↓
4. binder-aware generic application contract + assembly
   ↓
5. explicit empty/partial specialization semantics
   ↓
6. Tuple type unpacking
   ↓
7. callable-from-packs formation
   ↓
8. checked source-reachable rewriting
   ↓
9. source index / diagnostics / presentation
   ↓
10. stable metadata for any new P2 semantic/scoped nodes
   ↓
focused source/semantic integration
```

This is requirements decomposition, not yet the patch-grade task plan.

---

# 33. Test surface required before planning

P2 planning should cover at least the following dimensions.

### Binder syntax

```phalcom
<*Ts>
<**Ls>
<***P>
<T, *Ts>
<T, **Ls>
<T, *Ts, **Ls>
<T, ***P>
```

and all illegal ownership/order cases.

### Generic application

```phalcom
V<>
V<Int>
V<Int, String>

C<Int, *P>
C<Int, *P, **L>
C<Int, ***Q>
```

including concrete and symbolic expansion.

### Single-vs-pack distinction

```phalcom
Holder<P>
Holder<*P>
```

must not converge.

### Partial application

```phalcom
C
C<Int>
C<Int, String>
```

with a trailing pack binder.

### Empty specialization

```phalcom
Pack<>
```

must saturate an all-pack signature while fixed-only empty application fails.

### Wrong-domain expansion

Each of `*`, `**`, `***` needs negative operand-domain tests.

### No decomposition

A symbolic pack before an unsatisfied fixed binder must fail rather than infer a head/tail.

### Tuple formation

```phalcom
(*P,)
(**L,)
(***Q,)
(Int, *P, name: String, **L)
```

including Unit collapse and duplicate labels.

### Alias/scoped representation

At least:

```phalcom
type AddHead<*Ts> = (Int, *Ts)
```

and a generic alias forwarding its pack into another variadic constructor.

This is essential; it is not an edge case.

### Callable formation

```phalcom
(*P,) -> R
(**L,) -> R
(***Q,) -> R
(Int, *P, name: String, **L) -> R
```

including concrete collapse to existing callable identity and symbolic retention.

### Checked substitution

A source-created labeled concat nested inside another type must become an error when substitution produces a duplicate.

### Source identity

References inside `*P`, `**L`, `***Q` must retain canonical type-parameter targets.

### Compatibility

Existing fixed generic, Tuple, callable, TypePack, TypeAlias, and C1 projection tests must remain green.

---

# 34. Architectural decisions already resolved for the P2 plan

The P2 implementation plan should treat these as fixed unless the live tree changes before planning:

```text
* / ** / *** retain one lane meaning.
... is not generic/type-pack syntax.

P1 semantic modes are authoritative.
P1 pack terms are authoritative.
No source-only pack identity exists.

Generic application must read binder mode from GenericSignature,
not from arrow kind.

Ordinary generic arguments are not themselves a TypePack lane.
They remain atomic until binder ownership is known.

Pack binders capture zero elements.
Explicit <> can therefore be semantically meaningful.

No pack decomposition/backtracking.

All existing generic declaration sites participate uniformly
or reject intentionally.

Tuple expansion lowers through TupleFromPacks.

Generic aliases/type lambdas require scoped pack forms.

Callable finite symbolic unpacking requires a callable-from-packs
representation or an exact architectural equivalent.

Source-reachable substitution must use a genuinely checked traversal.

No runtime/VM ABI work.
```

---

# 35. Matters that should trigger consultation during P2 planning/implementation

Most of P2 is now determined, but the following should still trigger a deliberate architecture check if live implementation contradicts the analysis:

1. A newer commit already provides a canonical callable-domain pack representation different from `CallableFromPacks`.
2. Extending scoped type forms proves impossible without changing the accepted type-lambda identity model.
3. Variadic aliases cannot preserve binder mode through partial beta reduction without a materially new higher-kinded application model.
4. Application target identity cannot recover a declaration/alias generic signature without making `TypeResolver` depend cyclically on inappropriate semantic products.
5. Safe checked substitution requires a global error-channel rewrite rather than a bounded source-reachable migration.
6. Accepted language authority explicitly permits or requires kind annotations on pack binders.
7. A required source example can only work through unknown prefix/suffix decomposition.
8. Callable unpacking can only be implemented by changing runtime callable ABI.
9. An existing accepted callable-domain representation already treats pack unpacking and open rest lanes as one canonical abstraction in a way this audit missed.

None of those is currently evidenced by `main`; they are guardrails against implementation drift.

---

# 36. Requirements verdict

C2.P2 is larger than “add `*` to generic parameters,” but its architecture is still clean.

P1 has already solved the difficult canonical pack problem. P2 should now make source syntax a disciplined elaboration layer over that substrate.

The central invariant is:

```text
source variadicity
    is syntax + binder ownership + pack assembly

canonical semantics
    remain P1 terms and ordinary binder-aligned application
```

The most important additional requirements exposed by the live post-P1 audit are:

```text
1. Generic application must preserve/access GenericSignature modes;
   kind arrows are insufficient.

2. Ordinary generic arguments cannot be prematurely turned into TypePack
   because fixed generic parameters may have arbitrary kinds.

3. Scoped/de-Bruijn type forms must gain pack composition support so
   generic aliases and type lambdas work.

4. Residual type lambdas must retain binder modes where variadic aliases
   can be partially applied.

5. Callable type unpacking needs a symbolic callable-from-packs formation
   term rather than abusing rest-parameter semantics.

6. TypeSubstitution::try_apply must become recursively safe through every
   source-reachable container before P2 exposes symbolic labeled packs.

7. Explicit V<> must be handled by source binder elaboration rather than
   changing TypeStore::apply_type_form's empty-argument behavior.
```

If those seven requirements are designed correctly, the remainder of P2 is comparatively mechanical parser, AST, diagnostics, source-index, and integration work.

The target architecture at P2 completion should be:

```text
        source
          │
          ▼
   Single / * / ** / ***
          │
          ▼
   binder-aware source model
          │
          ├── canonical path
          │      P1 TypePack / LabeledTypePack /
          │      CompleteTypePack / Concat
          │
          └── scoped path
                 equivalent de-Bruijn pack forms
          │
          ▼
   one argument per GenericSignature binder
          │
          ├── Applied
          ├── TupleFromPacks
          └── CallableFromPacks
          │
          ▼
   checked substitution/materialization
          │
          ▼
   stable semantic identity
```

with the VM completely absent from the type-level operation.

That is the correct bounded requirements target for `LANG007.C2.P2`.