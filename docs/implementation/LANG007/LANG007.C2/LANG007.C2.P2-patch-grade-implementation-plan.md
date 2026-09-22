# LANG007.C2.P2 — Patch-Grade Implementation Plan

**Program:** `LANG007`  
**Checkpoint:** `LANG007.C2` — Variadic Generic Binding and Structural Type Expansion  
**Plan:** `LANG007.C2.P2`  
**Planning baseline:** post-P1 `main`, grounded at `cad8a1e32b76a6ab75adcdce42341ba1b88522ae` (`feat(lang007): add canonical pack composition`)  
**Lifecycle at planning:** `C2.P1 COMPLETE / FOCUSED_TESTED`; `C2.P2` is the next plan  
**Plan character:** patch-grade implementation plan; source elaboration over the P1 substrate; no VM ABI work

---

## 1. Objective

Implement the source-facing half of C2 on top of the canonical pack substrate completed by C2.P1.

After this plan, Phalcom source must be able to declare, apply, forward, and structurally unpack variadic generic type packs using the already-ratified operators:

```phalcom
class Positional<*Ts> {}
class Labeled<**Ls> {}
class Complete<***P> {}
class Split<T, *Ts, **Ls> {}
class Forward<T, ***P> {}
```

and source type formation must support pack expansion in generic application, Tuple types, and finite callable domains:

```phalcom
Positional<>
Positional<Int, String>
Target<*P, **L>
Target<***Q>

(*P,)
(**L,)
(***Q,)
(Int, *P, name: String, **L)

(*P,) -> R
(**L,) -> R
(***Q,) -> R
```

The implementation must elaborate these source forms into the C2.P1 canonical semantic vocabulary instead of creating a parallel variadic subsystem.

The fundamental target pipeline is:

```text
source syntax
    Single / * / ** / ***
        ↓
explicit lane-aware AST
        ↓
GenericSignature binder modes + source contribution stream
        ↓
P1 lane projection / pack composition
        ↓
one canonical argument per declared generic binder
        ↓
existing Applied / TupleFromPacks + new bounded CallableFromPacks term
        ↓
checked substitution / type-lambda beta reduction / stable metadata
```

---

## 2. Non-negotiable semantic laws

These are implementation invariants, not optional design guidance.

### INV-P2-01 — Operator meaning is global

```text
*   = positional lane
**  = labeled lane
*** = complete two-lane pack
```

The surrounding grammar selects capture vs. expansion vs. unpacking; it does not change lane meaning.

### INV-P2-02 — `...` is not C2 pack syntax

Do not recognize `...` as a generic binder, generic expansion, Tuple pack expansion, or complete-pack operator. Existing independent `...` semantics remain independent and must not be rewritten by this plan.

### INV-P2-03 — Binder mode is not kind

`P: TypePack` and `*P` remain semantically distinct.

```text
P: TypePack
    mode = Single
    kind = TypePack

*P
    mode = PositionalPack
    kind = TypePack
```

No P2 code may infer variadicity solely from `KindId`.

### INV-P2-04 — P1 owns canonical pack semantics

P2 must reuse:

```text
KindId::TYPE_PACK
KindId::LABELED_TYPE_PACK
KindId::COMPLETE_TYPE_PACK

TypeData::CompleteTypePack
TypeData::TypePackConcat
TypeData::LabeledTypePackConcat
TypeData::TupleFromPacks

TypeStore::project_positionals
TypeStore::project_labeled
TypeStore::project_complete
TypeStore::type_pack_concat
TypeStore::labeled_type_pack_concat
TypeStore::complete_type_pack
TypeStore::tuple_from_packs

GenericParameterMode
GenericSignature::parameter_modes
TypeSubstitution::try_apply
```

Do not introduce `VariadicTypePack`, `GenericPackValue`, `PackBindingStore`, or another canonical concat representation.

### INV-P2-05 — Generic applications remain binder-aligned

A source application may contain an arbitrary number of source entries, but canonical `Applied` arguments contain one semantic term per consumed declared binder.

For:

```phalcom
class C<T, *Ts> {}
C<Int, String, Bool>
```

canonical application is equivalent to:

```text
Applied(C, [Int, TypePack[String, Bool]])
```

not:

```text
Applied(C, [Int, String, Bool])
```

### INV-P2-06 — No pack decomposition solver

P2 may bind a whole symbolic pack. P2 must not solve unknown prefix/suffix partitions.

Allowed:

```text
T = Int
Ts = P
```

Not introduced here:

```text
head(P) = T
rest(P) = Ts
Concat(P, Q) = Concrete  → search all partitions
```

### INV-P2-07 — Concrete identity remains singular

When symbolic source pack formation becomes fully concrete, canonicalization must converge to existing identities.

```text
TupleFromPacks(concrete P, concrete L)
    → existing Tuple / Unit

CallableFromPacks(concrete P, concrete L, R)
    → existing concrete CallableType
```

No second concrete Tuple or callable identity may survive.

### INV-P2-08 — Scoped and canonical pack semantics must agree

Generic aliases and type lambdas use scoped/de-Bruijn type terms. P2 must support the same pack operations in that graph and beta-reduce them into P1 canonical forms.

### INV-P2-09 — Static callable unpacking is not runtime rest capture

`(*P,) -> R` means finite symbolic callable-domain formation from a type pack. It must not be represented as one runtime-style rest parameter whose payload is `P`.

### INV-P2-10 — Runtime remains out of the semantic operation

No type-level pack expansion executes in the VM. No new bytecode, frame convention, runtime TypePack object, or generic pack solver is introduced by this plan.

---

## 3. Frozen P1 substrate

Treat the following C2.P1 work as completed infrastructure, not work to redesign:

| Area | P1 substrate | P2 action |
|---|---|---|
| complete pack domain | `CompleteTypePack` kind + term | consume from source |
| positional composition | `TypePackConcat` | build source residuals through it |
| labeled composition | `LabeledTypePackConcat` | build source residuals through it |
| lane extraction | `project_positionals/labeled/complete` | use as sole expansion authority |
| variadic binder identity | `GenericParameterMode` | populate from AST syntax |
| stable generic modes | metadata mode refs | preserve / extend, do not replace |
| substitution | `try_apply` | make recursive source use fully checked |
| inference | ordinary kinded inference vars | no `PackVarId` |
| Tuple structural authority | `tuple_from_packs` | terminate Tuple expansion here |

Do not reopen P1 unless a P2 test proves a P1 invariant is incomplete.

---

## 4. Explicit scope

### 4.1 In scope

1. `*`, `**`, `***` generic binder syntax.
2. Binder legality and domain derivation.
3. `*`, `**`, `***` generic argument expansion syntax.
4. Binder-aware generic source application.
5. Explicit empty specialization `<>` for all-pack residual signatures.
6. Preservation of existing partial higher-kinded application.
7. Pack expansion in Tuple type syntax.
8. Pack unpacking in finite callable type syntax.
9. Scoped/de-Bruijn pack terms required by aliases and type lambdas.
10. Preservation of variadic binder mode through generated/residual type lambdas.
11. Checked substitution/materialization through all newly source-reachable terms.
12. Diagnostics, source identity, source fingerprint traversal, and source-oriented presentation.
13. Stable metadata for every new durable canonical/scoped form P2 introduces.
14. Focused parser/semantic/metadata/source-index regression tests.

### 4.2 Explicitly out of scope

Do not implement:

```text
runtime variadic parameter ABI
runtime rest-capture transport
VM pack-solving instructions
runtime TypePack/LabeledTypePack/CompleteTypePack objects
pack prefix/suffix decomposition inference
backtracking pack partitioning
Pack::Length
Pack::Slice
public generic pack concatenation syntax
implicit LabeledTypePack ↔ RecordRow conversion
GATs
broader const generics
new public source spelling for CompleteTypePack kind
arbitrary user-defined expansion protocols
full LSP feature presentation
program-wide release certification
```

C2.P3 remains responsible for broad inference/persistence/incremental certification over the P2 source surface.

---

## 5. Primary repository impact map

Paths below are based on the audited post-P1 repository. Treat exact helper placement as flexible, but preserve ownership boundaries.

### 5.1 AST / parser

Primary:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/parser.rs
```

Inspect/update if exhaustive syntax handling requires it:

```text
phalcom-ast/src/error.rs
other focused AST syntax tests/snapshots
```

### 5.2 Semantic parameter and type formation

Primary:

```text
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/substitution.rs
```

Likely exhaustive consumers to inspect and update:

```text
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/rigid.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/traits.rs
```

Do not mechanically change all of these unless the new canonical/scoped nodes reach them; audit exhaustive matches first.

### 5.3 Declaration / alias integration

Primary or likely:

```text
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/type_alias.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/enum_declaration.rs
phalcom-semantic/src/impls.rs
```

### 5.4 Source identity / incremental fingerprints

Primary:

```text
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/db/fingerprint.rs
```

Inspect other annotation visitors for new AST entry variants.

### 5.5 Stable metadata

Primary:

```text
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/validate.rs
phalcom-semantic/src/metadata/export.rs
```

Inspect metadata schema/version ownership and bundle validation before bumping version.

### 5.6 Runtime metadata readers only, if made exhaustive by the schema change

Potentially:

```text
phalcom-core/src/typing/inspect.rs
phalcom-core/src/modules/semantic_lowering.rs
```

The presence of a metadata reader update does **not** authorize runtime execution semantics.

### 5.7 Focused semantic tests

Known high-value targets:

```text
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/foundations/substitution.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/foundations/inference.rs
phalcom-semantic/tests/semantic/integration/metadata.rs
```

Add a dedicated C2/P2 test module if existing files would become unfocused.

---

## 6. Target AST model

Use the already-existing three-lane `ExpansionMode` for source expansion where practical.

### 6.1 Generic parameter syntax

Extend `GenericParameterSyntax` so source binder mode is explicit and independently ranged.

Conceptual target:

```rust
pub struct GenericParameterSyntax {
    pub mode: GenericParameterSyntaxMode, // or Option<ExpansionMode>
    pub mode_range: Option<SourceRange>,
    pub variance: VarianceSyntax,
    pub name: String,
    pub name_range: SourceRange,
    pub kind: Option<KindSyntax>,
    pub range: SourceRange,
}
```

A dedicated source enum is acceptable if it improves readability, but do not duplicate semantic lane meaning unnecessarily.

Semantic mapping must be explicit:

```text
Single      -> GenericParameterMode::Single
*           -> GenericParameterMode::PositionalPack
**          -> GenericParameterMode::LabeledPack
***         -> GenericParameterMode::CompletePack
```

### 6.2 Generic application items

Replace the assumption that `TypeAnnotationExpr::Application` owns only flat `Vec<TypeAnnotation>` arguments.

Conceptual target:

```rust
enum GenericArgumentSyntax {
    Single(TypeAnnotation),
    Expansion(TypePackExpansionSyntax),
}

struct TypePackExpansionSyntax {
    mode: ExpansionMode,
    operator_range: SourceRange,
    operand: TypeAnnotation,
    range: SourceRange,
}
```

The exact names are flexible.

### 6.3 Tuple type items

Represent Tuple pack expansion as an entry variant, not as an element with a special `ty`.

Conceptually:

```rust
enum TypeTupleItem {
    Element(TypeTupleElement),
    Expansion(TypePackExpansionSyntax),
}
```

### 6.4 Callable-domain items

Replace/extend callable type item syntax so finite pack unpacking is explicit.

Do not reuse `rest: bool` to mean `*P`.

Conceptually:

```rust
enum TypeCallableItem {
    Parameter(TypeCallableParameter),
    Expansion(TypePackExpansionSyntax),
}
```

If the existing parser must retain independent open/rest callable syntax, keep that information on the `Parameter` variant. Pack expansion must remain a distinct variant.

---

## 7. Target semantic additions

P1 already owns pack kinds and canonical pack composition. P2 should add only what is required to connect source/scoped/callable formation to that substrate.

### 7.1 Binder-aware application contract

Add a central semantic abstraction/helper that can answer:

```text
What generic binders remain for this application target?
What is each binder's kind?
What is each binder's GenericParameterMode?
How many canonical binders have already been consumed by partial application?
```

Conceptually:

```rust
struct GenericApplicationContract {
    origin: TypeId,
    parameters: Box<[TypeParameterId]>,
    kinds: Box<[KindId]>,
    modes: Box<[GenericParameterMode]>,
    consumed: usize,
}
```

Do not require this exact type. The semantic capability is mandatory.

For declaration-backed nominals, read the real `GenericSignature` from `DeclarationTypeTable`.

For transparent aliases, expose the real alias `GenericSignature` through the alias/type resolver path; do not recover modes from an arrow kind.

For residual type lambdas, preserve binder mode in the lambda representation so the residual itself carries the remaining application contract.

For signature-less arrow-kinded constructors, continue fixed kind application and reject unresolved symbolic expansions that would require unknown partitioning.

### 7.2 Scoped pack terms

Extend `ScopedTypeData` with a scoped equivalent of every pack structure needed inside alias/lambda bodies.

Recommended minimum:

```rust
ScopedTypePack(Box<[ScopedTypeId]>)
ScopedLabeledTypePack(Box<[ScopedLabeledTypePackElement]>)
ScopedCompleteTypePack {
    positionals: ScopedTypeId,
    labeled: ScopedTypeId,
}
ScopedTypePackConcat(Box<[ScopedTypeId]>)
ScopedLabeledTypePackConcat(Box<[ScopedTypeId]>)
ScopedTupleFromPacks {
    positionals: ScopedTypeId,
    labeled: ScopedTypeId,
}
```

If callable unpacking is source-reachable inside aliases/type lambdas, also add:

```rust
ScopedCallableFromPacks {
    positionals: ScopedTypeId,
    labeled: ScopedTypeId,
    return_type: ScopedTypeId,
}
```

These are scoped representations of the same algebra, not new semantic domains.

### 7.3 Type-lambda parameter mode retention

Extend type-lambda data so a parameter's application mode survives alias lowering and partial beta reduction.

Conceptual target:

```rust
pub struct TypeLambdaData {
    pub parameter_kinds: Box<[KindId]>,
    pub parameter_modes: Box<[GenericParameterMode]>,
    pub body: ScopedTypeId,
    ...
}
```

Rules:

```text
user-written ordinary type-lambda parameter
    -> Single

generated alias lambda parameter
    -> mode copied from alias GenericSignature

partial beta reduction
    -> residual lambda retains modes for remaining parameters
```

### 7.4 Callable-from-packs term

Add one bounded canonical term for finite callable domains assembled from pack lanes.

Conceptual target:

```rust
TypeData::CallableFromPacks {
    positionals: TypeId,
    labeled: TypeId,
    return_type: TypeId,
}
```

and a checked constructor:

```rust
fn callable_from_packs(
    &mut self,
    positionals: TypeId,
    labeled: TypeId,
    return_type: TypeId,
) -> Result<TypeId, PackFormationError>
```

Required canonicalization:

```text
positionals = concrete TypePack
labeled     = concrete LabeledTypePack
    ↓
existing CallableType with concrete ordered parameters and RestMode::None
```

Otherwise intern a residual proper Type of kind `Type`.

Do not encode symbolic `*P` as one `CallableParameterType { ty: P, rest: ... }`.

---

## 8. Generic binder legality model

Centralize legality in semantic signature formation. Parser recovery may diagnose obvious malformed syntax, but `resolve_generic_signature` must be the authoritative semantic gate across all declaration sites.

### 8.1 Legal shapes

Support deterministic shapes equivalent to:

```phalcom
C<>
C<T>
C<T, U>
C<*Ts>
C<**Ls>
C<***P>
C<T, *Ts>
C<T, **Ls>
C<T, *Ts, **Ls>
C<T, ***P>
```

`C<>` as a declaration remains whatever the existing grammar permits; the important P2 rule is the shape of non-empty binders.

### 8.2 Illegal shapes

Reject:

```phalcom
C<*A, *B>
C<**A, **B>
C<***P, T>
C<*Ts, T>
C<**Ls, *Ts>
C<*Ts, **Ls, ***P>
C<***A, ***B>
```

Reason: these shapes create duplicate lane ownership, reverse lane order, or require unknown suffix/prefix partitioning.

### 8.3 Kind derivation

A pack binder's operator determines its domain:

```text
*Ts   -> TypePack
**Ls  -> LabeledTypePack
***P  -> CompleteTypePack
```

Do not require or infer a source-visible complete-pack kind name.

### 8.4 Explicit kind annotation on pack binders

Until the language explicitly ratifies redundant pack-binder kind annotations, reject a form equivalent to:

```phalcom
*Ts: TypePack
**Ls: LabeledTypePack
***P: ...
```

with a targeted diagnostic that the pack operator already determines the binder domain.

This avoids two independent authorities for the same binder and avoids inventing a public spelling for the complete-pack kind.

### 8.5 Variance

Reject variance markers on pack binders.

No elementwise pack variance is introduced by P2.

---

## 9. Generic source-application algorithm

This algorithm is the core of P2. Implement it once and reuse it from all type-level generic application paths that support expansion.

### 9.1 Preserve atomic ordinary arguments

Do **not** initially convert ordinary generic arguments into a TypePack.

A `Single` generic argument may itself have any kind accepted by the binder:

```text
Type
TypePack
LabeledTypePack
CompleteTypePack
RecordRow
higher-kinded arrow
...
```

Represent source contributions conceptually as:

```text
Single(term)
PositionalExpansion(pack-term)
LabeledExpansion(labeled-pack-term)
CompleteExpansion(complete-pack-term)
```

### 9.2 Lower expansion operands by semantic projection

For each expansion operand:

```text
*X
    -> resolve X as a type-level form
    -> project_positionals(X)

**X
    -> resolve X
    -> project_labeled(X)

***X
    -> resolve X
    -> project_complete(X)
    -> retain both sub-lanes
```

Never branch on operand syntax such as “is this a parameter name?”.

### 9.3 Consume fixed Single binders first

A `Single` binder consumes one atomic positional source argument.

If the next available source contribution is a symbolic `*P`/`***Q` and the fixed binder would require taking one unknown element out of that pack, reject as unsupported pack decomposition.

Concrete expansions may be flattened because their cardinality is known.

Example:

```phalcom
class C<T, *Ts> {}

C<Int, *P>     // valid: T = Int, Ts = P
C<*P>          // invalid if P is symbolic: would need head/tail decomposition
```

### 9.4 Capture positional remainder

A `PositionalPack` binder consumes all remaining positional contributions after required fixed binders are satisfied.

Build it through P1:

```text
ordinary proper Type source entries
    -> singleton/concrete TypePack segments

*P contributions
    -> existing TypePack terms

***Q positional lane
    -> project_positionals(Q)

all segments
    -> TypeStore::type_pack_concat
```

If an ordinary remaining entry is not a proper `Type`, diagnose kind mismatch; it cannot become a TypePack element.

### 9.5 Capture labeled remainder

A `LabeledPack` binder owns the entire assembled labeled lane.

Build through:

```text
**L contribution
***Q labeled lane
    ↓
TypeStore::labeled_type_pack_concat
```

Concrete duplicate labels fail immediately. Symbolic overlap remains a residual concat until substitution/materialization resolves it.

### 9.6 Capture complete remainder

A `CompletePack` binder owns both remaining lanes:

```text
P = CompleteTypePack(
        assembled remaining positional lane,
        assembled labeled lane
    )
```

Use the P1 checked constructor.

### 9.7 Reject unowned residual contributions

After all binders are processed:

```text
unconsumed positional data -> generic arity/pack ownership error
unconsumed labeled data     -> target has no labeled pack owner
```

Do not silently drop lanes.

### 9.8 Produce binder-aligned canonical arguments

The result of source elaboration must be exactly one canonical/scoped term per consumed declared binder.

Pass that vector into the existing application/beta-reduction machinery.

---

## 10. Partial application and explicit empty specialization

### 10.1 Preserve higher-kinded partial application

Do not make variadic signatures all-or-nothing.

For:

```phalcom
class C<T, U, *Ts> {}
```

require behavior equivalent to:

```text
C
    residual binders: T, U, *Ts

C<Int>
    residual binders: U, *Ts

C<Int, String>
    T = Int
    U = String
    Ts = []
    fully saturated

C<Int, String, Bool>
    T = Int
    U = String
    Ts = [Bool]
    fully saturated
```

Once all required `Single` binders are satisfied, a trailing pack binder may capture zero elements.

### 10.2 Explicit empty `<>`

Preserve the distinction:

```phalcom
class V<*Ts> {}

V       // unsaturated constructor
V<>     // saturated; Ts = []
```

Do not alter the internal rule:

```rust
TypeStore::apply_type_form(origin, &[]) == origin
```

Instead source elaboration of `V<>` must synthesize the binder-aligned empty pack argument:

```text
[TypePack[]]
```

Similarly:

```text
<**Ls>       -> [LabeledTypePack[]]
<***P>       -> [CompleteTypePack([], [])]
<*Ts, **Ls>  -> [TypePack[], LabeledTypePack[]]
```

For fixed-only signatures such as `List<T>`, `List<>` remains invalid because required `Single` binders remain unsatisfied.

### 10.3 Residual type-lambda modes

When a transparent alias/type lambda is partially applied, slice both:

```text
remaining parameter_kinds
remaining parameter_modes
```

Never retain only kinds.

---

## 11. Scoped pack algebra implementation

This task is a hard prerequisite for complete alias/type-lambda support.

### 11.1 Extend `ScopedTypeData`

Add scoped nodes for pack literals/composition/formation as required by Section 7.2.

Every new scoped node must participate in:

```text
hash-consing / equality
kind computation
free/bound shifting
substitution of free scoped terms
partial beta substitution
canonical materialization
rigid/local conversions if applicable
metadata export
validation
formatting/debug output
```

Audit every exhaustive match over `ScopedTypeData` before compilation is restored.

### 11.2 Scoped constructors/canonicalization

Do not duplicate P1 normalization semantics inconsistently.

For scoped terms:

- flatten nested concat nodes;
- eliminate scoped empty concrete segments where structurally known;
- merge adjacent concrete scoped pack segments where safe;
- preserve bound symbolic segments;
- preserve labeled ordering;
- reject concrete duplicate labels that are visible in the scoped graph;
- defer unknown overlap involving bound symbolic lanes.

When beta reduction replaces all bound segments with canonical concrete terms, invoke the P1 canonical constructors so final identity converges.

### 11.3 Alias lowering

Update `lower_scoped_type_alias_form` and the ordinary `lower_scoped_type_form` application/Tuple/callable branches so aliases such as these are valid:

```phalcom
type AddHead<*Ts> = (Int, *Ts)
type Forward<*Ts> = Target<*Ts>
type MapForward<*Ts> = (*(Ts::Map<Option>),)
```

The body must remain scoped until beta reduction; do not prematurely convert bound pack variables into fake canonical free parameters.

### 11.4 Generated lambda modes

When a generic transparent alias is converted to a `TypeLambdaData`, copy `GenericSignature::parameter_modes` alongside parameter kinds.

Explicit ordinary source type lambdas get `Single` for each ordinary binder unless separately supported syntax already says otherwise.

---

## 12. Generic application contract recovery

The current kind-only application path is insufficient for P2. Implement a mode-aware target contract without making `KindId` carry source binder semantics.

### 12.1 Nominal declarations

For a nominal origin or a partially applied nominal origin:

1. recover the base `DeclarationId`;
2. read `DeclarationTypeTable::generic_signature`;
3. inspect existing `TypeData::Applied` canonical arguments;
4. compute the first remaining binder;
5. apply P2 source assembly to remaining modes.

### 12.2 Transparent aliases

`TypeAliasInfo` already owns a `generic_signature`, but the current `TypeResolver` seam exposes only alias form resolution.

Extend the resolver architecture with a semantic application-contract/signature lookup rather than inferring from alias arrow kind.

Preferred shape:

```text
resolver/declaration authority
    -> declaration identity
    -> optional GenericSignature or owned compact application contract
```

A default `None` method is acceptable for simple test resolvers if ordinary fixed kind application remains available.

Do not make `TypeStore` depend on source declarations to solve this; binder awareness belongs in source/type formation, above the canonical store.

### 12.3 Residual lambdas

Read mode directly from the residual lambda data added in Section 10/11.

### 12.4 Signature-less arrow constructors

For a constructor known only by arrow kind:

```text
ordinary atomic source args
    -> existing kind application

fully concrete expansion
    -> may flatten into ordinary arguments if exact cardinality is known

symbolic positional expansion
    -> reject if satisfying fixed arrow slots would require unknown cardinality

labeled expansion
    -> reject; plain arrow kinds have no labeled binder owner
```

No implicit variadic contract may be fabricated.

---

## 13. Tuple type expansion

### 13.1 Parser/AST

Parse pack entries in Tuple type position as explicit `TypeTupleItem::Expansion` equivalents.

Preserve comma/tuple disambiguation already owned by the parser.

### 13.2 Assembly

Build two scoped/canonical lane streams.

For ordinary entries:

```text
Int
    -> positional singleton pack [Int]

name: String
    -> labeled singleton pack [(name, String)]
```

For expansions:

```text
*P
    -> positional projection

**L
    -> labeled projection

***Q
    -> both projections from complete Q
```

Then:

```text
positionals = TypePackConcat(...)
labeled     = LabeledTypePackConcat(...)
result      = TupleFromPacks(positionals, labeled)
```

Use scoped equivalents inside aliases/type lambdas.

### 13.3 Lane-phase validation

Respect the existing positional/labeled source phase:

- ordinary positional items, `*`, and `***` may occur while positional phase remains open;
- an explicit labeled Tuple entry or `**` begins labeled-only phase;
- after labeled-only phase begins, a later source contribution that can add positional elements is invalid;
- `***` does not by itself start labeled-only phase.

### 13.4 Required canonical outcomes

```text
(*TypePack[Int, String],)
    -> existing Tuple(Int, String)

(**LabeledTypePack[name: String],)
    -> existing Tuple(name: String)

(***CompleteTypePack([], []),)
    -> Unit

(*P,)
    -> TupleFromPacks(P, []) when P is symbolic
```

Duplicate concrete labels must fail through existing/P1 pack validation.

---

## 14. Callable finite pack unpacking

### 14.1 Add `CallableFromPacks`

Implement the bounded canonical term described in Section 7.4.

Constructor validation:

```text
positionals.kind == TypePack
labeled.kind     == LabeledTypePack
return.kind      == Type
```

When both pack lanes are concrete:

1. emit ordinary positional callable parameters in pack order;
2. emit labeled callable parameters in labeled-pack order;
3. set each concrete parameter's runtime/rest mode to `None`;
4. call existing `store.callable(...)` so the ordinary concrete callable identity is reused.

When either lane is symbolic, retain `CallableFromPacks` as a proper Type.

### 14.2 Scoped callable formation

Add the scoped equivalent and beta-reduction path.

Example:

```phalcom
type Fn<*Ps> = (*Ps,) -> Bool
```

must preserve the bound pack until application.

### 14.3 Conservative downstream semantics

P2 does not need to implement pack-decomposition callable subtyping/inference.

For symbolic residual callable-from-packs terms:

- equality/reflexivity may use canonical identity;
- normalization/substitution/materialization must preserve the term;
- relation/inference code must not panic or reinterpret the term as one parameter;
- if a relation would require decomposing symbolic lanes, return the existing conservative non-proof/blocked outcome appropriate to that subsystem.

P3 may extend whole-pack inference/certification later.

### 14.4 Preserve independent rest/open callable semantics

Do not delete or repurpose existing callable `RestMode` or independent `...` syntax solely because finite pack unpacking now exists.

The static concepts remain distinct:

```text
finite exact symbolic expansion from a TypePack
    !=
open homogeneous rest lane
```

---

## 15. Checked substitution and rewriting hardening

P2 exposes P1 residual pack terms to ordinary source programs, so every recursive rewrite that can encounter them must preserve validation failures.

### 15.1 Strengthen `TypeSubstitution::try_apply`

Audit every source-reachable `TypeData` container.

The checked path must recurse with `try_apply`, not silently fall back to infallible `apply`, wherever a child can contain C2 pack terms.

At minimum audit:

```text
Applied
Tuple
Callable
CallableFromPacks
AssociatedProjection
TypeMemberProjection
Family/member containers if source-reachable
CompleteTypePack
TypePackConcat
LabeledTypePackConcat
TupleFromPacks
```

Do not over-expand scope to unrelated internal terms if they provably cannot contain pack residuals.

### 15.2 Preserve duplicate-label failure

Required regression:

```text
L1 = symbolic labeled pack
L2 = symbolic labeled pack
Concat(L1, L2) initially valid/residual
substitution makes both contain #name
    -> checked substitution returns duplicate-label failure
```

The error must not be swallowed by an outer `Applied`, Tuple, callable, or alias beta-reduction path.

### 15.3 Other rewriting paths

Audit:

```text
specialize_self_type
materialization
projection normalization
rigid/local materialization
GADT normalization
inference materialization
```

Only update paths that can structurally contain the new residual terms. Preserve existing behavior elsewhere.

---

## 16. Declaration-site integration

`GenericParameterSyntax` is shared by multiple declaration forms. P2 must not support variadic binders selectively by accident.

Audit and cover:

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

### 16.1 Central signature formation

Prefer making all sites inherit behavior from the existing `resolve_generic_signature` rather than writing site-specific pack lowering.

### 16.2 Universe/bootstrap declarations

`phalcom-semantic/src/declarations.rs` currently bootstraps source generic parameters using kind information. Update bootstrap parameter/signature construction to preserve source binder mode where source Universe declarations can contain it.

Do not create bootstrap-only mode inference.

### 16.3 Callable-local generic signatures

Existing `resolve_callable_local_generics` already routes through `GenericBinderSite::Callable`; preserve that architecture.

Pack binder kind/domain rules should be applied before site-specific kind restrictions such as `RecordRow` legality, without conflating the domains.

### 16.4 Enum variant generic signatures

Variant constructors own their own generic signatures. Ensure variant binder mode metadata survives and explicit variant generic application can use the same binder-aware application helper where the syntax supports it.

---

## 17. Exact-case and other angle-bracket paths

Search for every AST field that stores generic type arguments independently of `TypeAnnotationExpr::Application`.

Known example:

```text
TypeAnnotationExpr::ExactEnumCase.generic_arguments
```

If a path represents actual generic binder application, migrate it to the same `GenericArgumentSyntax`/binder-aware assembler.

Do **not** automatically add expansion to unrelated angle-bracket mechanisms such as intrinsic type-member arguments merely because they share punctuation.

Criterion:

```text
Does this source list bind a GenericSignature?
    yes -> reuse P2 generic application machinery
    no  -> leave its grammar/semantics unchanged unless separately required
```

---

## 18. Diagnostics

Add targeted diagnostics rather than collapsing all new failures into generic arity errors.

Required concepts:

| Failure | Diagnostic intent |
|---|---|
| duplicate `*` binder | positional generic lane already captured |
| duplicate `**` binder | labeled generic lane already captured |
| `***` conflicts with split owner | complete pack conflicts with existing lane capture |
| binder after `***` | complete pack binder must be terminal |
| fixed binder after pack binder | declaration would require pack partitioning |
| pack binder with variance | variadic pack binder variance is unsupported |
| explicit kind on pack binder | pack operator already determines binder domain |
| wrong `*` operand | operand has no positional type-pack lane |
| wrong `**` operand | operand has no labeled type-pack lane |
| wrong `***` operand | operand does not represent a complete pack/Tuple shape |
| symbolic expansion before fixed binder | application would require pack decomposition |
| leftover labeled lane | target has no labeled pack binder |
| leftover positionals | target cannot accept remaining positional type arguments |
| fixed-only `<>` | required generic arguments are missing |
| concrete duplicate label | duplicate labeled type-pack key |
| duplicate after substitution | instantiated labeled pack contains duplicate key |
| symbolic expansion into signature-less fixed constructor | cardinality cannot be determined without decomposition |
| `...` in pack context | ellipsis is not a type-pack expansion operator |

Use `operator_range`, operand range, binder name range, and conflicting binder range where available.

Do not print raw `KindId`/`TypeData` debug variants in user diagnostics when source-oriented text is possible.

---

## 19. Source indexing and fingerprinting

### 19.1 Source identity

For:

```phalcom
class C<*Ts> {
    // references to Ts
}
```

there is one parameter identity: `Ts`.

The `*` token is syntax, not a separate target.

Preserve:

```text
name_range      -> TypeParameterId target
operator_range  -> diagnostic/token context only
```

### 19.2 Annotation visitors

Update source-index/type-reference visitors to recurse into operands of:

```text
generic argument expansions
Tuple type expansions
callable-domain expansions
exact-case generic expansions where applicable
```

A reference inside `*(P::Map<Option>)` must retain normal targets for `P`, `Map`, and `Option` according to existing indexing rules.

### 19.3 Fingerprints

The public source/surface fingerprint must change when binder mode changes:

```text
<P: TypePack> -> <*P>
<*P>          -> <***P>
```

P1 already made semantic parameter mode stable metadata. P2 must ensure source-input fingerprinting and declaration-shard invalidation actually observe the newly parsed syntax.

Do not hash formatted semantic strings.

---

## 20. Presentation

Internal canonicalization may produce:

```text
Applied(C, [Int, TypePack[String, Bool]])
```

for source:

```phalcom
C<Int, String, Bool>
```

User-facing presentation must remain source-oriented.

Requirements:

1. Diagnostic formatting understands binder modes.
2. Do not expose `TypePackConcat(...)`, `CompleteTypePack(...)`, or `CallableFromPacks(...)` as ordinary source spelling.
3. Debug/internal dumps may retain canonical names.
4. Fixed generic presentation is unchanged.
5. Reflection metadata may expose structured semantic modes, but source printers should reconstruct language-level shape where they currently provide source-like output.

---

## 21. Stable metadata and schema

P2 may introduce durable nodes beyond P1. These must be serializable if they can occur in a public declaration/alias/member type.

### 21.1 Extend canonical metadata for callable residuals

Add the stable equivalent of `CallableFromPacks` to `phalcom-type-meta/src/type_node.rs` if the canonical term is introduced.

Validation must enforce:

```text
positionals kind = TypePack
labeled kind     = LabeledTypePack
return kind      = Type
node kind        = Type
```

### 21.2 Extend scoped metadata

Mirror every new scoped pack node required by exported type lambdas in `phalcom-type-meta/src/scoped_type.rs`.

Do not serialize query-local IDs.

### 21.3 Type-lambda modes

Extend stable `TypeLambdaRef`/scoped lambda metadata so parameter modes survive export/import when semantically relevant.

Backward/default behavior for metadata without explicit modes must be `Single` only if schema compatibility rules make that valid. Prefer an explicit schema revision rather than ambiguous interpretation.

### 21.4 Schema version

Inspect the current P1 metadata schema/version before modifying. If adding serialized enum variants/fields is a format change, bump once for P2 rather than piecemeal.

Do not guess the exact numeric version in advance; P1 reportedly established schema v3, so a normal additive P2 revision would likely become v4, but implementation must verify the live constant.

### 21.5 Metadata readers

Update exhaustive metadata inspection/import code as required to recognize the new terms.

This is metadata transport/inspection only. It must not add VM evaluation of pack terms.

---

## 22. Task graph

```text
T0  Baseline + invariant probes
 ↓
T1  AST/parser lane-aware syntax
 ↓
T2  Generic binder semantic lowering + legality
 ↓
T3  Generic application contract + binder-aware assembler
 ├──────────────┐
 ↓              ↓
T4  Scoped pack algebra + lambda modes
 │              │
 └──────┬───────┘
        ↓
T5  Tuple type expansion
        ↓
T6  CallableFromPacks + callable unpacking
        ↓
T7  Checked substitution/materialization hardening
        ↓
T8  Source index + fingerprints + diagnostics + presentation
        ↓
T9  Stable metadata completion
        ↓
T10 Focused integration + hostile regression suite
        ↓
T11 Lifecycle docs / walkthrough / handoff
```

T3 and T4 are mutually reinforcing. Do not expose the complete source surface before both are working.

---

# 23. Detailed implementation tasks

## T0 — Freeze baseline and add failing invariant probes

### Goal

Establish a small red test set that proves the exact missing P2 behavior without altering P1 semantics.

### Work

1. Confirm current `main` includes C2.P1 commit/workthrough state.
2. Run focused existing P1 tests for:
   - pack kinds;
   - P1 projections;
   - concat canonicalization;
   - GenericParameterMode metadata;
   - substitution duplicate-label surfacing.
3. Add parser/semantic tests initially marked failing or implement incrementally for:

```phalcom
class V<*Ts> {}
class L<**Ls> {}
class C<***P> {}
```

and:

```phalcom
V<>
V<Int, String>
(*P,)
(*P,) -> R
```

4. Add one alias probe:

```phalcom
type AddHead<*Ts> = (Int, *Ts)
```

5. Add one hostile split probe:

```phalcom
class C<T, *Ts> {}
C<*P>
```

with symbolic `P`, expecting “requires decomposition” rather than arbitrary partitioning.

### Gate G0

Before implementation begins, prove:

- P1 focused tests are green;
- new P2 probes fail for the expected missing syntax/semantic reasons;
- no test requires VM execution.

### STOP / CONSULT

Stop if P1 focused tests are already red on the planning baseline. Diagnose baseline drift before attributing failures to P2.

---

## T1 — Add explicit AST/parser support

### Goal

Make the parser preserve binder and expansion intent without assigning semantic meaning yet.

### Files

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/parser.rs
```

### Steps

1. Extend generic parameter AST with source pack mode/operator range.
2. Add explicit generic argument entry variants.
3. Add explicit Tuple type item expansion variants.
4. Add explicit callable type item expansion variants.
5. Parse one, two, or three consecutive `*` tokens in only the relevant contexts.
6. Do not add a global prefix type-unary `*` expression.
7. Preserve existing pointer/arithmetic/value grammar behavior outside type pack contexts.
8. Preserve existing independent ellipsis syntax.
9. Preserve explicit empty application `Origin<>` as an Application node with an explicitly present empty source list.
10. Update AST visitors/selectors only where compilation requires.

### Parser acceptance cases

```phalcom
class A<*Ts> {}
class B<**Ls> {}
class C<***P> {}
class D<T, *Ts, **Ls> {}

A<*P>
B<**L>
C<***Q>
D<Int, *P, **L>

(*P,)
(**L,)
(***Q,)
(Int, *P, name: String, **L)

(*P,) -> R
(**L,) -> R
(***Q,) -> R
```

### Parser negative/recovery cases

```phalcom
class Bad<****P> {}
Target<****P>
(...P,)          // must not become C2 pack syntax
<...P>           // must not become C2 binder syntax
```

Semantic ordering errors may parse into recoverable AST and be rejected in T2/T3.

### Gate G1

AST snapshots/tests prove `*`, `**`, `***` are explicit structural syntax at all P2-owned sites and are not represented as ordinary unary type operators.

---

## T2 — Lower binder modes and enforce binder legality

### Goal

Make `resolve_generic_signature` produce P1 semantic modes/kinds from source binders uniformly across declaration sites.

### Primary files

```text
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/declarations.rs
```

### Steps

1. Map AST binder mode to `GenericParameterMode`.
2. Derive pack binder kind from mode.
3. Reject explicit kind annotation on pack binders unless a live accepted spec explicitly says otherwise.
4. Reject variance on pack binders.
5. Validate deterministic binder ownership/order before interning parameter IDs.
6. Ensure `TypeParameterData` is interned with mode-sensitive identity through the existing P1 path.
7. Ensure `GenericSignature::parameter_modes` is populated exactly once and aligned with `parameters`/`parameter_kinds`.
8. Preserve existing `Single` generic behavior.
9. Update source Universe/bootstrap signature creation to preserve mode.
10. Exercise the path through declaration, callable-local, alias, impl, and variant generic signatures.

### Binder legality state machine

Implement a simple deterministic state machine rather than ad hoc pairwise checks.

Conceptual states:

```text
FixedPrefix
PositionalCaptured
LabeledCaptured
CompleteCaptured
```

Transitions:

```text
FixedPrefix + Single           -> FixedPrefix
FixedPrefix + PositionalPack   -> PositionalCaptured
FixedPrefix + LabeledPack      -> LabeledCaptured
FixedPrefix + CompletePack     -> CompleteCaptured
PositionalCaptured + LabeledPack -> LabeledCaptured
```

All other transitions fail.

This directly prevents backtracking-oriented declaration shapes.

### Gate G2

Semantic-unit tests prove:

```text
source binder operator
    -> correct P1 GenericParameterMode
    -> correct P1 KindId
    -> stable mode-sensitive TypeParameterId/signature identity
```

and all illegal binder order/ownership cases fail with targeted diagnostics.

---

## T3 — Implement the central binder-aware application assembler

### Goal

Replace flat source argument → flat kind application at declaration-backed variadic targets with deterministic binder-aware elaboration.

### Primary files

```text
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/type_alias.rs
```

### Steps

1. Introduce one internal representation for source generic contributions:

```text
Single(term)
PositionalPack(term)
LabeledPack(term)
CompletePack(term)
```

Use canonical `TypeId` in the ordinary path and scoped equivalents in T4.

2. Add application-contract recovery for:
   - nominal declarations;
   - partially applied nominals;
   - transparent aliases;
   - residual lambdas once T4 lands;
   - signature-less arrow-kinded constructors as a fixed fallback.
3. Extend `TypeResolver`/alias resolver seam only as much as needed to retrieve alias generic binder contract.
4. Resolve expansion operands and call the P1 lane projection APIs.
5. Implement fixed-prefix consumption without symbolic pack decomposition.
6. Implement `*` remainder capture with `type_pack_concat`.
7. Implement `**` remainder capture with `labeled_type_pack_concat`.
8. Implement `***` remainder capture with `complete_type_pack`.
9. Reject leftover lanes.
10. Emit exactly one canonical argument per consumed binder.
11. Feed the result to existing `apply_type_form`/beta reduction.
12. Handle explicit empty application by synthesizing empty pack arguments once all required Singles are satisfied.
13. Preserve partial application before required Singles are satisfied.

### Required examples

#### All-pack signature

```phalcom
class V<*Ts> {}

V
    // constructor

V<>
    // Applied(V, [TypePack[]])

V<Int, String>
    // Applied(V, [TypePack[Int, String]])
```

#### Fixed prefix

```phalcom
class C<T, *Ts> {}

C<Int, *P>
    // T = Int, Ts = P
```

#### Split lanes

```phalcom
class C<T, *Ts, **Ls> {}
C<Int, String, *P, **L>
```

canonical argument shape:

```text
[
  Int,
  TypePackConcat([String], P),
  L
]
```

#### Complete remainder

```phalcom
class C<T, ***P> {}
C<Int, String, ***Q>
```

canonical second argument:

```text
CompleteTypePack(
    TypePackConcat([String], project_positionals(Q)),
    project_labeled(Q)
)
```

### Required hostile case

```phalcom
class C<T, *Ts> {}
C<*P>
```

where symbolic `P: TypePack` must fail because filling `T` would require head/tail decomposition.

### Gate G3

No declaration-backed variadic source application reaches `TypeStore::apply_type_form` until it has a binder-aligned argument vector.

---

## T4 — Extend scoped type algebra and preserve lambda modes

### Goal

Make the same P2 source forms work inside generic aliases and type lambdas without escaping scoped bound variables.

### Primary files

```text
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/annotation.rs
```

### Steps

1. Add scoped pack literal/composition/formation nodes.
2. Add scoped kind resolution for each node.
3. Add scoped lane projection helpers or a shared scoped pack assembler mirroring P1 laws.
4. Update bound-variable shifting through all new nodes.
5. Update free-term substitution through all new nodes.
6. Update beta substitution/materialization so concrete results call P1 canonical constructors.
7. Add `parameter_modes` to `TypeLambdaData` or an equivalent residual application contract.
8. Set alias-generated lambda modes from alias `GenericSignature`.
9. Set ordinary type-lambda modes to `Single` unless source syntax says otherwise.
10. Slice modes together with kinds during partial beta reduction.
11. Update lambda interning/hash identity so mode is semantic identity.
12. Update every exhaustive match over `ScopedTypeData`.

### Required alias tests

```phalcom
type AddHead<*Ts> = (Int, *Ts)
type Forward<*Ts> = Target<*Ts>
type ForwardBoth<***P> = Target<***P>
type Fn<*Ps> = (*Ps,) -> Bool
```

### Partial alias test

Use an alias with fixed prefix plus trailing pack and verify the residual constructor still knows the trailing parameter is variadic after partial application.

### Gate G4

A bound pack parameter can survive source lowering → scoped graph → residual lambda → beta reduction and emerge as the correct P1 canonical pack term without being converted to a fake proper Type.

### STOP / CONSULT

Stop if preserving variadic mode requires replacing the accepted type-lambda identity architecture rather than extending it with aligned mode metadata.

---

## T5 — Implement Tuple type expansion through `TupleFromPacks`

### Goal

Make Tuple structural type syntax consume pack lanes without introducing another Tuple representation.

### Primary files

```text
phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/types/type_lambda.rs
```

### Steps

1. Replace flat Tuple-element lowering with a lane assembler.
2. Lower ordinary positional type entries into singleton positional pack contributions.
3. Lower ordinary labeled entries into singleton labeled pack contributions.
4. Lower `*`, `**`, `***` via canonical/scoped projection.
5. Enforce positional/labeled phase rules.
6. Build concat terms.
7. Finish with `tuple_from_packs` or scoped equivalent.
8. Ensure all-concrete cases collapse to existing Tuple/Unit.
9. Preserve symbolic concat residuals.
10. Map duplicate-label errors to source diagnostics with useful spans.

### Gate G5

All of these hold:

```text
concrete expansion -> existing Tuple identity
empty expansion    -> Unit when whole tuple is empty
symbolic expansion -> TupleFromPacks residual
labels never flatten into positional lane
duplicate concrete labels never survive
```

---

## T6 — Add finite symbolic callable-domain formation

### Goal

Represent `(*P,) -> R`, `(**L,) -> R`, and `(***Q,) -> R` honestly as finite symbolic callable types.

### Primary files

```text
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/types/annotation.rs
```

### Steps

1. Add `TypeData::CallableFromPacks` or exact equivalent.
2. Add a checked `callable_from_packs` constructor.
3. Canonicalize all-concrete lanes into existing `CallableType`.
4. Add scoped callable-from-packs form.
5. Lower callable type AST through the same lane assembly principles as Tuple formation.
6. Keep existing rest/open callable syntax distinct.
7. Update kind computation and all exhaustive `TypeData`/`ScopedTypeData` consumers.
8. Give type relation/inference conservative residual behavior; do not implement sequence decomposition.
9. Add formatting/debug support.

### Gate G6

A symbolic TypePack in callable-domain position is represented as a symbolic **domain splice**, never as one parameter whose type is the pack.

### STOP / CONSULT

Stop if the only apparent implementation path requires changing bytecode calling convention, runtime frame layout, or runtime rest capture. That would violate P2 scope and indicates the static representation seam is wrong.

---

## T7 — Harden checked substitution/materialization

### Goal

Ensure P2 source terms cannot hide pack validation failures during rewriting.

### Primary files

```text
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/instantiation.rs
phalcom-semantic/src/types/environment.rs
```

Inspect/update as required:

```text
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/types/rigid.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/gadt_proof.rs
phalcom-semantic/src/traits.rs
```

### Steps

1. Make `try_apply` recursively checked through every source-reachable container.
2. Add `CallableFromPacks` handling.
3. Ensure `Applied` child substitution uses the checked path and does not suppress a pack error.
4. Ensure Tuple/callable/projection containers preserve pack errors.
5. Re-canonicalize concat/complete/Tuple/callable pack forms after substitution.
6. Preserve a symbolic concat when overlap remains unknowable.
7. Surface duplicate labels once concrete.
8. Ensure no new `unwrap_or(original)` path masks a semantic invalidity introduced by substitution.

### Gate G7

A duplicate label revealed by substitution fails identically whether the labeled concat is top-level or nested under another type constructor.

---

## T8 — Integrate diagnostics, source identity, fingerprints, and presentation

### Goal

Make the new syntax stable as source identity and incremental input, and make failures understandable.

### Primary files

```text
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/diagnostic.rs   # only if new codes live here
```

plus existing presentation helpers discovered by search.

### Steps

1. Recurse into every new AST expansion operand in source-index visitors.
2. Preserve generic parameter target identity on the name token.
3. Add operator/binder conflict spans to diagnostics.
4. Update source fingerprints for generic parameter mode and expansion-bearing type annotations where those annotations participate in product fingerprints.
5. Ensure `<P: TypePack> -> <*P>` changes the appropriate public/source fingerprint.
6. Ensure `*P -> ***P` changes it.
7. Update source-oriented type/application formatting to understand variadic binders.
8. Keep canonical debug formatting separate from source formatting.

### Gate G8

Source references inside `*P`, `**L`, `***Q` resolve to the same semantic operands they would without the operator, while public binder-mode edits invalidate the correct declaration products.

---

## T9 — Complete durable metadata support

### Goal

Make every public P2 canonical/scoped type exportable and validated without introducing runtime evaluation.

### Primary files

```text
phalcom-type-meta/src/type_node.rs
phalcom-type-meta/src/scoped_type.rs
phalcom-type-meta/src/generic.rs
phalcom-type-meta/src/validate.rs
phalcom-semantic/src/metadata/export.rs
```

Potential reader updates:

```text
phalcom-core/src/typing/inspect.rs
phalcom-core/src/modules/semantic_lowering.rs
```

### Steps

1. Add stable canonical callable-from-packs node if T6 introduced it.
2. Add stable scoped pack nodes corresponding to T4.
3. Extend stable type-lambda representation with parameter modes.
4. Include mode/nodes in structural fingerprints.
5. Add metadata kind validation for all new nodes.
6. Bump metadata schema once if required by live format policy.
7. Update deserialization/validation limits/exhaustive matches.
8. Update runtime typing inspection display only as needed to avoid unknown-node failures.
9. Do **not** add bytecode or VM normalization behavior.

### Metadata gate G9

A public alias such as:

```phalcom
type Fn<*Ps> = (*Ps,) -> Bool
```

can be exported to stable metadata without erasing:

```text
Ps mode = PositionalPack
scoped pack splice in body
callable-from-packs residual if still symbolic
```

and without serializing query-local semantic IDs.

---

## T10 — Focused integration and hostile regression suite

### Goal

Prove P2 works as one source feature and does not regress fixed generics or P1 pack semantics.

### 10.1 Positive binder cases

```phalcom
class A<*Ts> {}
class B<**Ls> {}
class C<***P> {}
class D<T, *Ts> {}
class E<T, **Ls> {}
class F<T, *Ts, **Ls> {}
class G<T, ***P> {}
```

### 10.2 Empty captures

```phalcom
A<>
B<>
C<>
```

Verify canonical empty pack values and saturation.

### 10.3 Fixed prefix and partial application

Verify constructor-vs-saturated distinction for signatures with fixed prefix + pack tail.

### 10.4 Source expansion

```phalcom
A<*P>
F<Int, *P, **L>
G<Int, ***Q>
```

### 10.5 Projection composition

Use pack-producing associated type operations such as `Map` where already supported:

```phalcom
A<*(P::Map<Option>)>
```

and corresponding Tuple type expansion.

### 10.6 Tuple forms

```phalcom
(*P,)
(**L,)
(***Q,)
(Int, *P, name: String, **L)
```

### 10.7 Callable forms

```phalcom
(*P,) -> R
(**L,) -> R
(***Q,) -> R
(Int, *P, name: String, **L) -> R
```

### 10.8 Alias/scoped forms

```phalcom
type AddHead<*Ts> = (Int, *Ts)
type Forward<*Ts> = Target<*Ts>
type Fn<*Ps> = (*Ps,) -> Bool
```

Verify beta reduction for concrete specialization and residual mode preservation for partial application.

### 10.9 Hostile binder cases

Reject:

```text
second positional binder
second labeled binder
complete binder plus split binder
binder after complete binder
fixed binder after pack binder
variance on pack binder
explicit kind annotation on pack binder
```

### 10.10 Hostile application cases

Reject:

```text
* wrong-domain operand
** wrong-domain operand
*** single-lane operand
symbolic pack before required fixed binder
leftover labeled lane without labeled/complete owner
leftover positional lane without owner
fixed-only <>
... used as C2 pack syntax
symbolic pack expansion into a signature-less fixed constructor when arity cannot be known
```

### 10.11 Duplicate labels

Cover:

1. immediate concrete duplicate;
2. symbolic concat remains residual;
3. duplicate revealed after substitution;
4. duplicate nested inside another generic applied type;
5. duplicate inside Tuple formation;
6. duplicate inside callable formation.

### 10.12 Compatibility regression

Re-run existing suites for:

```text
fixed nominal generics
fixed generic type aliases
type lambdas
Tuple structural types
callable types
TypePack/LabeledTypePack P1 operations
RecordRow
associated type projections
metadata validation
source indexing
```

### Suggested focused commands

Use the live workspace command conventions. At minimum the final P2 gate should include equivalents of:

```bash
cargo test -p phalcom-ast
cargo test -p phalcom-type-meta
cargo test -p phalcom-semantic
```

During implementation, use focused test filters to keep iteration bounded.

Do not claim whole-program release certification from these commands; that remains P3/checkpoint closure work.

### Gate G10

All new P2 focused tests and all directly affected existing crate suites are green, with no VM/bytecode changes required.

---

## T11 — Record implementation state and handoff to P3

### Goal

Leave repository truth synchronized with the actual implementation.

### Required records

At completion create/update the C2.P2 lifecycle artifacts according to project doctrine, including:

```text
LANG007.C2.P2 walkthrough
LANG007.C2.P2 handoff
LANG007.C2 checkpoint status
LANG007 program/status next-plan pointer
```

The walkthrough should record:

- exact commit range;
- files changed;
- canonical representations actually chosen;
- whether `CallableFromPacks` exact naming was used;
- metadata schema revision, if any;
- test commands/results;
- known residual limitations intentionally deferred to P3.

The P3 handoff must state that P3 owns certification/inference/incremental breadth, not redesign of P2 source semantics.

---

## 24. Recommended commit grouping

Keep commits semantically reviewable. A suitable grouping is:

### Commit 1 — AST/parser source forms

```text
feat(lang007): parse variadic generic and type pack expansion syntax
```

Contains T1 only plus parser tests.

### Commit 2 — binder semantics and application assembly

```text
feat(lang007): lower variadic generic binders and applications
```

Contains T2 + canonical portion of T3, including empty specialization.

### Commit 3 — scoped pack algebra and alias preservation

```text
feat(lang007): preserve pack expansion through type lambdas
```

Contains T4 and scoped application behavior.

### Commit 4 — structural Tuple/callable formation

```text
feat(lang007): lower tuple and callable pack expansions
```

Contains T5 + T6.

### Commit 5 — checked rewriting and durable metadata

```text
fix(lang007): preserve pack validation through substitution and metadata
```

Contains T7 + T9 and required reader exhaustiveness.

### Commit 6 — source identity, diagnostics, integration tests, docs

```text
feat(lang007): complete C2.P2 source integration
```

Contains T8 + T10 + T11.

If compilation dependencies make this exact split impractical, preserve the semantic boundaries even if two adjacent commits must merge.

---

## 25. Patch discipline

### 25.1 Do not widen `TypeStore` into a source authority

`TypeStore` should know canonical kinds/terms and checked formation. It should not know declaration source binder modes.

Binder-aware application belongs in source/type-formation semantics.

### 25.2 Do not make parser ordering the only legality authority

Parser may detect obvious syntax errors, but the semantic signature validator must reject invalid binder shapes consistently for every declaration source.

### 25.3 Do not add a second generic instantiation map

Continue binding:

```text
TypeParameterId -> TypeId/kinded canonical term
```

through existing generic instantiation/substitution infrastructure.

### 25.4 Do not make `ExpansionMode` a kind

Expansion mode is syntax/lane selection. P1 kinds remain canonical semantic domains.

### 25.5 Do not flatten labeled packs

`***` contributes positional and labeled lanes separately. Labels must never become positional payload types.

### 25.6 Do not hide errors with fallback `unwrap_or(original)`

P2 source terms make deferred duplicate-label invalidity observable. Any checked path that can expose the invalidity must return it.

### 25.7 Do not preempt P3 inference

When an operation would require solving a symbolic sequence split, report unsupported/deferred inference rather than introducing a local backtracking solver.

---

## 26. STOP / CONSULT conditions

Implementation must stop and request an architecture decision if any of the following occurs.

### SC-01 — P1 substrate contradiction

A live P1 invariant differs materially from the checkpoint/walkthrough, e.g. pack mode or canonical concat identity changed after the planning revision.

### SC-02 — Complete kind requires public spelling

A required source form cannot be implemented without inventing a public source name for the internal complete-pack kind.

### SC-03 — Alias mode cannot survive residual lambda

Preserving a variadic alias through partial beta reduction appears to require replacing, rather than extending, the accepted type-lambda architecture.

### SC-04 — Application signature lookup creates a dependency cycle

Recovering alias/declaration `GenericSignature` would require an inappropriate semantic product cycle rather than a narrow resolver contract extension.

### SC-05 — Callable formation requires runtime ABI changes

Static `(*P,) -> R` cannot be represented without modifying runtime frames, bytecode, or invocation transport.

### SC-06 — Checked substitution needs global result-type redesign

Safely propagating deferred duplicate-label errors cannot be bounded to the source-reachable checked path and would require a program-wide substitution API replacement.

### SC-07 — Accepted source authority permits different binder grammar

An authoritative spec newer than this plan explicitly allows fixed suffix parameters, multiple pack binders per lane, redundant pack kinds, or another partition model.

### SC-08 — Required acceptance case needs symbolic prefix/suffix decomposition

Do not silently extend C2 into a sequence solver.

### SC-09 — Existing callable-domain authority already has a canonical exact-pack representation

If the repository contains a newer accepted static callable-domain representation, use it rather than adding `CallableFromPacks`; update the plan/walkthrough accordingly.

---

## 27. Definition of done

C2.P2 is complete only when all of the following are true.

### Source syntax

- [ ] `*`, `**`, `***` generic binders parse explicitly.
- [ ] `*`, `**`, `***` generic argument expansions parse explicitly.
- [ ] Tuple type pack expansions parse explicitly.
- [ ] callable-domain pack expansions parse explicitly.
- [ ] `...` is not treated as C2 pack syntax.

### Generic signatures

- [ ] binder mode is lowered to P1 `GenericParameterMode`.
- [ ] pack kind is derived from mode.
- [ ] binder ownership/order is validated centrally.
- [ ] pack binders remain invariant.
- [ ] `Single(TypePack)` remains distinct from `PositionalPack`.

### Generic applications

- [ ] application reads real binder modes, not kind alone.
- [ ] canonical arguments are binder-aligned.
- [ ] symbolic pack expansion after fixed prefix works.
- [ ] symbolic pack expansion before unsatisfied fixed binder fails without decomposition.
- [ ] split `*`/`**` capture works.
- [ ] `***` complete capture works.
- [ ] explicit all-pack `<>` saturates with empty packs.
- [ ] fixed-only `<>` remains invalid.
- [ ] existing partial fixed application remains valid.

### Scoped aliases/lambdas

- [ ] scoped pack literal/composition nodes exist as needed.
- [ ] alias bodies can contain pack expansion.
- [ ] beta reduction canonicalizes into P1 pack terms.
- [ ] residual lambdas preserve remaining parameter modes.

### Tuple types

- [ ] symbolic positional expansion lowers through `TupleFromPacks`.
- [ ] symbolic labeled expansion preserves labels/order.
- [ ] complete expansion contributes both lanes.
- [ ] concrete formation converges to existing Tuple/Unit identity.

### Callable types

- [ ] finite symbolic pack unpacking has an honest canonical representation.
- [ ] concrete lanes converge to existing `CallableType`.
- [ ] symbolic pack is never represented as one rest parameter.
- [ ] runtime ABI remains unchanged.

### Rewriting

- [ ] checked substitution recursively preserves C2 validation.
- [ ] duplicate labels revealed after substitution fail.
- [ ] materialization preserves/canonicalizes all new terms.

### Identity / persistence

- [ ] source references inside expansion operands are indexed.
- [ ] binder mode edits affect appropriate fingerprints.
- [ ] durable metadata represents every public residual P2 term.
- [ ] metadata validation enforces kind correctness.
- [ ] no query-local IDs leak into stable metadata.

### Regression

- [ ] `phalcom-ast` relevant suite green.
- [ ] `phalcom-type-meta` relevant suite green.
- [ ] `phalcom-semantic` relevant suite green.
- [ ] fixed generic behavior unchanged.
- [ ] P1 pack tests unchanged/green.
- [ ] no VM/bytecode changes were needed for type-level pack semantics.

---

## 28. P3 handoff boundary

When P2 is done, P3 should receive a source-complete but intentionally non-decomposing variadic type system.

P3 should be able to assume:

```text
source binders are canonical
source expansion is canonical
scoped aliases preserve packs
Tuple/callable structural formation is canonical
metadata can persist all terms
```

P3 then owns broad certification of:

```text
whole-pack inference interaction
cross-module metadata round-trip
incremental/cold parity
edit invalidation
stress/density behavior
public reflection/presentation consistency
checkpoint-level compatibility and runtime-negative certification
```

P3 must not replace P2's binder/application algebra with another solver or representation.

---

## 29. Final implementation principle

The implementation should make this statement true everywhere in the compiler:

> **Variadicity is a property of source binding and lane assembly; the canonical type system continues to operate on ordinary kinded terms, P1 pack composition, binder-aligned applications, and structural formation nodes.**

The plan succeeds if `*`, `**`, and `***` become fully usable source-level type operators without making packs runtime objects, without creating a second generic system, without requiring known pack length, and without weakening the canonical identities established by C1/C2.P1.
