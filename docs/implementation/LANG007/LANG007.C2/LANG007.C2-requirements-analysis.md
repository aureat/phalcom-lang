---
id: LANG007.C2
category: LANG
program: LANG007
checkpoint: LANG007.C2
kind: requirements-analysis
status: ACCEPTED
prepared: 2026-09-22
source: user-supplied C2 requirements analysis (verbatim)
---

# LANG007.C2 — Detailed Implementation Summary and Requirements Analysis

With C1 fully implemented and closed, C2 should now implement the first genuinely **variadic** layer on top of C1's structural type calculus.

The corrected checkpoint objective is:

> **LANG007.C2 — Variadic Generic Pack Binding and Type-Level Pack Expansion**
>
> Extend C1's `TypePack`, `LabeledTypePack`, Tuple shape, symbolic-term, substitution, metadata, and incremental infrastructure so generic declarations and type-level structural expressions can capture and expand positional (`*`), labeled (`**`), and complete (`***`) packs using the already-ratified lane algebra. C2 must preserve lane identity, support abstract/symbolic packs without requiring their lengths or labels to be concrete, and canonicalize all concrete results back into C1's existing structural types.

The critical correction from the earlier analysis is that C2 does **not** introduce `...` as pack syntax. The established operators remain:

```text
*     positional lane
**    labeled lane
***   complete two-lane pack
```

and `...` retains its existing independent language role. C2 should not reinterpret it anywhere as variadic generic syntax.

---

## 1. What C2 actually adds

C1 already answered:

```text
What is a TypePack?
What is a LabeledTypePack?
How are Tuple lanes represented?
How are symbolic type-level operations represented?
How are these terms substituted, persisted, indexed, and normalized?
```

C2 answers:

```text
How does source code bind an unknown number of types?
How does it bind an unknown labeled type sequence?
How does it bind both lanes together?
How are those packs expanded into another generic/type-level structure?
How are several concrete and symbolic contributions composed?
How do those operations remain canonical through substitution,
metadata, inference, and incremental recompilation?
```

So C2 is primarily about **binding and composition**, not introducing packs themselves.

The conceptual progression becomes:

```text
C1
    TypePack / LabeledTypePack exist.
    Tuple(P, L) exists.
    Structural type operations exist.

C2
    Source code can capture P/L/both lanes variadically.
    Source code can expand those lanes into other type structures.
    Symbolic pack compositions remain canonical.

Later
    The solver can decompose unknown packs.
    Call-site inference can infer arbitrary pack lengths.
    Runtime callable ABI can transport variadic packs where needed.
```

---

# 2. The three ratified lane meanings become type-level laws

C2 must use exactly the existing lane algebra.

For a complete pack with positional lane `P` and labeled lane `L`:

```text
π*(⟨P, L⟩)   = P
π**(⟨P, L⟩)  = L
π***(⟨P, L⟩) = ⟨P, L⟩
```

The corresponding source forms remain:

```phalcom
*pack
**pack
***pack
```

The surrounding grammar determines whether this is capture, expansion, or type unpacking.

That uniformity should be a core C2 invariant. There should not be a “generic `*`” with semantics different from the existing positional projection, nor a generic `***` that secretly means “flatten everything into a positional sequence.”

`***` must retain both lanes.

---

# 3. C2 needs a canonical complete type-pack domain

This is the most important type-theoretic requirement that follows from applying the three-lane algebra to generic binding.

C1 currently has:

```text
TypePack
    ordered positional sequence of proper types

LabeledTypePack
    ordered sequence of (label, proper type)

Tuple(P, L)
    proper structural Tuple Type
```

For C2:

```phalcom
*Ts
```

can naturally bind:

```text
Ts : TypePack
```

and:

```phalcom
**Ls
```

can naturally bind:

```text
Ls : LabeledTypePack
```

But:

```phalcom
***P
```

cannot soundly be represented as either one.

It represents:

```text
both lanes together
```

C2 therefore needs a canonical **complete type-pack semantic domain**, conceptually:

```text
CompleteTypePack(P, L)

P : TypePack
L : LabeledTypePack
```

The exact internal/source-visible name can follow the repository's accepted terminology; the important requirement is the semantic distinction.

It must be a type-level domain, not a runtime class.

This matters for kind correctness. Treating `***P` merely as:

```text
P : Type
```

with an informal “it should be a Tuple” note would be insufficient. A constructor such as:

```phalcom
class C<***P> {}
```

would otherwise appear to have ordinary kind:

```text
Type -> Type
```

and could incorrectly be substituted where a constructor accepting an arbitrary proper Type is expected.

The actual domain is narrower:

```text
CompleteTypePack -> Type
```

or its equivalent canonical representation.

This maintains the type theory instead of hiding a structural constraint outside the kind/domain model.

---

# 4. Complete type packs do not create another Tuple identity

Adding a complete-pack domain must not violate C1's canonical Tuple model.

The relationship should be:

```text
CompleteTypePack(P, L)
    = a type-level two-lane shape

Tuple(P, L)
    = the proper structural Tuple Type formed from that shape
```

Conversion into a Tuple type uses the existing C1 authority:

```text
TupleFromPacks(P, L)
```

or its final equivalent.

For an empty complete pack:

```text
CompleteTypePack([], [])
```

forming a Tuple proper type must canonicalize to:

```text
Unit
```

because C1 already owns:

```text
Tuple([], []) ≡ Unit
```

So C2 must not introduce:

```text
EmptyCompletePackType == new proper Type
```

or any second concrete Tuple identity.

The complete-pack term and Tuple proper type inhabit different semantic domains.

---

# 5. Generic binders become lane-aware

C2 should extend generic binder syntax with the already-ratified pack operators.

Conceptually:

```phalcom
class Positional<*Ts> {}
class Labeled<**Ls> {}
class Complete<***P> {}
```

with semantic bindings:

```text
*Ts
    Ts : TypePack

**Ls
    Ls : LabeledTypePack

***P
    P : CompleteTypePack
```

An ordinary existing parameter:

```phalcom
P: TypePack
```

remains different from:

```phalcom
*P
```

even though both ultimately involve the `TypePack` domain.

The distinction is:

```text
P: TypePack
    one source generic argument whose value is a TypePack

*P
    variadic source binding that captures a positional sequence
    into one TypePack value
```

This distinction must be represented explicitly in the generic signature.

It must not be inferred from the parameter kind.

---

# 6. Generic parameter shape becomes part of the signature

The current generic signature is essentially a fixed list of parameters and kinds.

C2 needs an orthogonal parameter/binder mode.

Conceptually:

```rust
enum GenericParameterMode {
    Single,
    PositionalPack, // *
    LabeledPack,    // **
    CompletePack,   // ***
}
```

The effective domains are:

```text
Single
    declaration-specific kind/domain

PositionalPack
    TypePack

LabeledPack
    LabeledTypePack

CompletePack
    CompleteTypePack
```

This mode is semantically observable.

For example:

```phalcom
class A<P: TypePack> {}
class B<*P> {}
```

must not have interchangeable public interfaces.

Their generic application rules differ even if both ultimately contain a `TypePack` value.

Therefore the mode must participate in:

```text
generic signature identity
stable metadata
public API fingerprints
incremental invalidation
reflection/tooling
pretty-printing
```

---

# 7. Binder ordering should mirror pack capture semantics

C2 should preserve the same lane logic already established for pack capture rather than invent a generic-specific ordering system.

The useful forms are conceptually:

```phalcom
class A<T, *Ts> {}
class B<T, **Ls> {}
class C<T, *Ts, **Ls> {}
class D<T, ***P> {}
```

The split form:

```text
*Ts, **Ls
```

captures the two lanes independently.

The complete form:

```text
***P
```

captures the remaining complete pack as one value.

`***P` should be terminal.

A signature should not combine:

```text
*Ts
**Ls
***P
```

in ways that create two owners for the same remaining lane.

The binder legality rules should therefore mirror the established rest/capture algebra rather than becoming a pack-partitioning language.

Most importantly, C2 must avoid syntax that requires the compiler to guess how many elements belong to which unknown pack.

---

# 8. Empty captures are valid

Pack binders naturally permit zero elements.

For:

```phalcom
class V<*Ts> {}
```

the following specialization must be representable:

```phalcom
V<>
```

with:

```text
Ts = []
```

Likewise:

```phalcom
class V<**Ls> {}
```

may bind:

```text
Ls = []
```

and:

```phalcom
class V<***P> {}
```

may bind the empty complete pack.

This creates an important distinction:

```phalcom
V
```

is the unsaturated generic constructor.

```phalcom
V<>
```

is the explicitly saturated zero-pack specialization.

Those cannot be conflated.

Otherwise the language would make one source form simultaneously mean:

```text
a higher-kinded constructor
```

and:

```text
a proper specialized type
```

The parser therefore needs to admit an explicit empty generic argument list, while semantic validation continues rejecting it for ordinary fixed-arity generic declarations such as:

```phalcom
List<>
```

if `List` still requires a fixed argument.

---

# 9. Generic application itself becomes a pack-assembly context

Today generic applications are essentially:

```text
origin + Vec<TypeAnnotation>
```

C2 can no longer model them as a flat vector.

A generic application must be assembled using the same conceptual two-lane structure:

```text
GenericArgumentPack {
    positional
    labeled
}
```

This is a compile-time semantic pack, not a runtime argument pack object.

Source items need to distinguish:

```text
ordinary generic argument
* expansion
** expansion
*** expansion
```

Conceptually:

```rust
enum GenericArgumentItem {
    Single(TypeAnnotation),
    PositionalExpansion(TypeAnnotation),
    LabeledExpansion(TypeAnnotation),
    CompleteExpansion(TypeAnnotation),
}
```

The exact Rust names can differ.

The crucial requirement is that `*`, `**`, and `***` remain explicit AST structure. They must not be represented as ordinary unary arithmetic operators.

---

# 10. Generic argument assembly should mirror existing source-phase rules

The already-ratified expansion rules say that source assembly has positional and labeled phases.

C2 should reuse that model.

Conceptually:

```text
positional phase:
    ordinary positional items
    * expansion
    *** expansion

labeled phase:
    ** expansion
    any other generic labeled forms admitted by the language

*** does not itself start the labeled phase.
```

The point is not necessarily to reuse runtime code, but to share the same semantic law.

A compiler should not have:

```text
CallPackBuilder semantics
TuplePackBuilder semantics
GenericPackBuilder semantics
```

that disagree about what the operators mean.

They may have different element domains—runtime `Value` versus compile-time type-level terms—but the lane algebra should be common.

---

# 11. Binder-aware generic application

After source pack assembly, application must bind the resulting lanes against the generic signature.

For:

```phalcom
class C<T, *Ts, **Ls> {}
```

a source application conceptually assembled as:

```text
positionals = [Int, String, Bool]
labeled     = L
```

binds as:

```text
T  = Int
Ts = [String, Bool]
Ls = L
```

For:

```phalcom
class C<T, ***P> {}
```

the same assembled source pack binds:

```text
T = Int

P = CompleteTypePack(
    [String, Bool],
    L
)
```

This binding is deterministic.

C2 must not use a backtracking solver to decide where a pack begins.

Fixed positional parameters consume their required prefix first; pack binders own the remaining lanes according to their declared operator.

---

# 12. Canonical applications stay binder-aligned

This is important for reusing C1 machinery.

Source may contain many generic items:

```phalcom
C<Int, String, Bool>
```

but if the declaration is:

```phalcom
class C<T, *Ts> {}
```

the canonical semantic application should contain one argument per declared binder:

```text
T  -> Int
Ts -> TypePack[String, Bool]
```

not:

```text
[Int, String, Bool]
```

as an indefinitely source-shaped application.

Conceptually:

```text
Applied(
    C,
    [
        Int,
        TypePack[String, Bool]
    ]
)
```

That allows C2 to continue using the existing C1 infrastructure:

```text
GenericInstantiation
TypeSubstitution
parameter -> canonical term binding
TypeData::Applied
```

rather than inventing a second variadic applied-type representation.

For a complete pack:

```text
Applied(
    C,
    [
        Int,
        CompleteTypePack(P, L)
    ]
)
```

or the equivalent canonical binding.

---

# 13. Positional symbolic composition needs a canonical term

Consider:

```text
P : TypePack
Q : TypePack
```

and a source expansion assembling:

```phalcom
<Int, *P, String, *Q>
```

The result cannot necessarily be flattened because `P` and `Q` may be abstract.

C2 therefore needs a durable symbolic positional composition term, conceptually:

```text
TypePackConcat(
    [Int],
    P,
    [String],
    Q
)
```

It must have kind:

```text
TypePack
```

and obey canonical normalization laws:

```text
Concat()             ≡ []
Concat(P)            ≡ P
Concat([], P)        ≡ P
Concat(P, [])        ≡ P

Concat([A], [B, C])
    ≡ [A, B, C]

Concat(Concat(A,B), C)
    ≡ Concat(A,B,C)
```

Adjacent concrete segments should merge.

If substitution later makes every segment concrete, the result must canonicalize to C1's ordinary `TypePack`.

There must not be two identities for a concrete pack.

---

# 14. Labeled composition needs its own canonical term

The same applies to:

```text
L₁ : LabeledTypePack
L₂ : LabeledTypePack
```

C2 needs the equivalent of:

```text
LabeledTypePackConcat(L₁, L₂, ...)
```

with ordered labels preserved.

This operation has one extra semantic obligation:

```text
labels must remain unique
```

For fully concrete input:

```text
[(x, Int)] ++ [(x, String)]
```

is invalid.

For abstract input:

```text
L₁ ++ L₂
```

the compiler may not yet know whether labels overlap.

That must not be approximated as valid forever, nor rejected merely because it is abstract.

The correct C1-compatible outcome is a stable symbolic residual:

```text
Stuck / unresolved duplicate-label obligation
```

which becomes:

```text
Reduced
```

when disjointness is known, or:

```text
Invalid
```

when an overlap becomes concrete.

This is directly analogous to C1's `Reduced / Stuck / Invalid` distinction.

No new theorem prover is required.

---

# 15. Complete pack assembly should be lane-wise

A complete pack does not need a giant flattened sequence.

It should be represented as:

```text
CompleteTypePack(
    positionals = PositionalPackExpression,
    labeled     = LabeledPackExpression
)
```

Therefore several contributions such as:

```text
*P
***Q
**L
```

compose lane-wise:

```text
positionals =
    concat(
        P,
        π*(Q)
    )

labeled =
    labeledConcat(
        π**(Q),
        L
    )
```

This matches the ratified rule that `***` contributes each sub-lane to its corresponding destination lane.

C2 must never implement `***` by flattening:

```text
positionals ++ labeled payloads
```

into one positional sequence.

That would destroy labels.

---

# 16. Operator applicability at the type level

C2 should formalize the structural type-level applicability of the operators.

A clean model is:

```text
*TypePack
    -> same TypePack

*CompleteTypePack
    -> positional TypePack

*Tuple(P,L)
    -> P

**LabeledTypePack
    -> same LabeledTypePack

**CompleteTypePack
    -> labeled LabeledTypePack

**Tuple(P,L)
    -> L

***CompleteTypePack
    -> same complete pack

***Tuple(P,L)
    -> CompleteTypePack(P,L)
```

The zero product follows naturally:

```text
*Unit
    -> []

**Unit
    -> []

***Unit
    -> CompleteTypePack([], [])
```

Critically, the ratified rule says `***` is not redundant spelling for single-lane sources.

Therefore C2 should not silently make:

```text
***TypePack
```

an alias for:

```text
*TypePack
```

nor:

```text
***LabeledTypePack
```

an alias for:

```text
**LabeledTypePack
```

That would violate the complete-pack semantics.

---

# 17. Structural Tuple type formation becomes pack-aware

C1 already owns:

```text
Tuple(P,L)
```

C2 should make source tuple-type formation able to consume symbolic packs.

Examples conceptually include:

```phalcom
(*P,)
```

where:

```text
P : TypePack
```

and:

```phalcom
(**L,)
```

where:

```text
L : LabeledTypePack
```

and:

```phalcom
(***Q,)
```

where `Q` carries both lanes.

Mixed forms become possible according to the ratified lane/source-order rules.

For example, a type-level Tuple shape assembled from:

```text
Int
*P
***Q
**L
```

should lower to:

```text
TupleFromPacks(
    TypePackConcat(
        [Int],
        P,
        π*(Q)
    ),
    LabeledTypePackConcat(
        π**(Q),
        L
    )
)
```

When all terms become concrete, it canonicalizes to the existing C1 Tuple proper type.

No new structural Tuple identity is introduced.

---

# 18. `Map<F>` must compose with pack expansion

C1 already guarantees:

```text
P::Map<F> : TypePack
L::Map<F> : LabeledTypePack
```

Therefore C2 must permit pack expansion of those symbolic results.

For example:

```phalcom
(*(P::Map<Option>),)
```

must be semantically valid if the surrounding grammar is the type-unpacking context and:

```text
P : TypePack
```

Likewise labeled mapping can be expanded through `**`.

Pack expansion is based on the operand's semantic domain, not on whether the operand is syntactically just a parameter name.

This is an important composability requirement.

The following should be equally valid as positional pack sources:

```text
P
P::Map<F>
some other stable projection whose result domain is TypePack
```

provided their type-level domain is proven.

---

# 19. Callable-domain type unpacking belongs in C2

The pack specifications already ratify:

```phalcom
(*P,) -> R
(**P,) -> R
(***P,) -> R
```

as type unpacking forms.

C2 is the natural implementation checkpoint for the **static type formation** side of those rules.

That means C2 should implement the semantic transformation of these forms into the compiler's callable-domain representation.

This is distinct from runtime variadic call ABI work.

C2 may know that a callable Type accepts a domain assembled from positional/labeled lanes without implementing new runtime rest capture.

The separation is:

```text
C2:
    form and normalize the callable Type

later callable checkpoint:
    infer/capture/transport runtime argument packs
```

This avoids mixing static type theory with VM calling-convention work.

---

# 20. Generic declarations on functions and types

The current program roadmap explicitly calls out variadic generic parameters on functions/types.

C2 should therefore make the generic signature model capable of all four binder modes at declaration sites that already support generic parameters:

```text
Single
*
**
***
```

However, this does **not** imply arbitrary runtime pack inference.

A callable may have a compile-time generic pack binder and use that binder in its type annotations.

What C2 guarantees is:

```text
declaration representation
explicit/syntactically determined pack binding
whole-pack substitution
stable metadata
```

What C2 does not need to solve is:

```text
infer unknown pack length by examining an arbitrary runtime call
```

That remains later pack inference.

This distinction prevents C2 from expanding into the callable ABI checkpoint.

---

# 21. Whole-pack inference is allowed; decomposition is not

C1 already has kind-aware inference variables.

C2 may support equations such as:

```text
P : TypePack

P = [Int, String]
```

as one whole-pack binding.

Likewise:

```text
L : LabeledTypePack

L = [(name, String)]
```

and the complete equivalent.

But C2 does not need to solve:

```text
Concat(P, [String]) = [Int, Bool, String]
```

for:

```text
P = [Int, Bool]
```

That is pack decomposition.

Nor does it need to solve:

```text
Concat(P, Q) = ConcretePack
```

by trying all partitions.

That would introduce backtracking and a sequence solver.

C2's solver boundary should therefore be:

```text
whole-pack equality/unification:
    yes

pack decomposition / split inference:
    no
```

This is a critical complexity boundary.

---

# 22. No new `PackVarId` by default

C1 deliberately made pack-valued variables ordinary kinded semantic variables.

C2 should preserve that architecture.

For:

```text
*Ts
```

the bound semantic variable has kind:

```text
TypePack
```

and can continue using the existing kinded `InferVarId`.

For:

```text
**Ls
```

it similarly uses:

```text
LabeledTypePack
```

For the new complete-pack domain, the ordinary kinded inference infrastructure should be extended to that kind.

A dedicated `PackVarId` should only be introduced if there is a mathematically distinct solver lifetime, which C2 does not require.

The later pack-decomposition solver may justify specialized sequence constraints, but C2 should not preemptively add them.

---

# 23. GenericInstantiation remains the canonical binding store

C1 already has an instantiation model based on:

```text
TypeParameterId -> canonical TypeId-like term
```

C2 should preserve that abstraction.

Binder-aligned canonical bindings mean:

```text
Single T
    -> proper type term

*Ts
    -> TypePack term

**Ls
    -> LabeledTypePack term

***P
    -> CompleteTypePack term
```

The storage representation may need to generalize from “type binding” terminology to “kinded binding” if it currently assumes all non-row entries are ordinary type forms, but it should remain one generic-instantiation authority.

Do not create:

```text
VariadicGenericInstantiation
PackInstantiation
PackBindings
```

as competing systems unless required by the existing architecture.

---

# 24. AST requirements

C2 needs two explicit syntax concepts.

First, generic parameters need a capture mode:

```rust
GenericParameterSyntax {
    ...
    pack_mode: Single | Positional | Labeled | Complete
}
```

Second, generic/type-level argument entries need expansion mode:

```rust
GenericArgumentItem {
    Single(...)
    PositionalExpansion(...)
    LabeledExpansion(...)
    CompleteExpansion(...)
}
```

Tuple/callable type entries may use the same shared pack-operator representation.

The implementation should ideally expose one reusable:

```text
PackOperator
    Positional
    Labeled
    Complete
```

rather than separately defining:

```text
GenericPackOperator
TuplePackOperator
CallablePackOperator
```

The surrounding AST node supplies the context.

That reflects the ratified rule:

> one lane algebra, multiple contexts.

---

# 25. `...` must not enter any C2 pack grammar

This deserves an explicit acceptance invariant because the current LANG007 roadmap/handoff still contains stale `...P` examples from earlier planning.

C2 must eliminate those examples from **current active LANG007 planning/state**.

It must not rewrite completed historical P1/P2/P3 records merely to pretend they never existed, but active C2 documentation and `PROGRAM.md` should use the correct operators.

C2 parser work must not:

```text
recognize <...T> as a pack binder
recognize ...P as generic expansion
lower ... to pack projection
```

`...` remains semantically separate.

---

# 26. Stable metadata requirements

C2 adds several pieces of durable semantic identity:

```text
binder mode
CompleteTypePack kind/domain
TypePackConcat
LabeledTypePackConcat
complete pack terms
pack-expansion structure where semantically retained
```

Stable metadata therefore needs corresponding representation.

At minimum public generic parameter metadata must distinguish:

```text
T
*Ts
**Ls
***P
```

because changing one into another is an API change.

A public declaration changed from:

```phalcom
class C<P: TypePack>
```

to:

```phalcom
class C<*P>
```

must produce a different interface fingerprint.

Likewise:

```phalcom
class C<*P>
```

to:

```phalcom
class C<***P>
```

is a semantic API change even if a particular application happens to contain only positional elements.

No fingerprint may be based on formatted source strings.

---

# 27. Incremental requirements

C2 should extend the incremental parity guarantees established by C1.

Representative edits should include transitions such as:

```text
<T>
    → <T, *Ts>

<*Ts>
    → <***P>

C<Int, String>
    → C<Int, String, Bool>

C<*P>
    → C<*P, **L>
```

and symbolic pack-source edits.

Required outcome:

```text
incremental final semantic state
    ==
cold final semantic state
```

for:

```text
canonical generic application
diagnostics
metadata fingerprint
source targets
downstream dependent types
```

The compiler must not solve C2 by globally invalidating the semantic workspace after any variadic edit.

---

# 28. Source identity and tooling requirements

Each pack binder has one canonical generic-parameter identity.

For:

```phalcom
class C<*Ts>
```

references to:

```text
Ts
```

inside the declaration must source-target the same `TypeParameterId` or equivalent canonical parameter identity as the binder.

The `*` token is a binder/pack operator; it is not a separate variable identity.

Likewise expansions such as:

```phalcom
*Ts
```

should navigate from `Ts` to the declaration parameter.

The same applies to:

```text
**Ls
***P
```

C2 need not deliver all rich LSP presentation, but it must preserve enough source-index identity for future tooling.

---

# 29. User-facing formatting must remain source-oriented

Internally:

```phalcom
C<Int, String, Bool>
```

may become:

```text
T  = Int
Ts = TypePack[String, Bool]
```

But a diagnostic should not suddenly print:

```text
C<Int, [String, Bool]>
```

unless that is actual source syntax.

Presentation must reconstruct the declaration's binder shape.

The same applies to complete packs.

Internal:

```text
CompleteTypePack(
    [Int],
    [(name, String)]
)
```

should be presented through the source-level structural form appropriate to the context, not as an implementation enum dump.

Canonical equality and hashing remain independent of presentation.

---

# 30. Duplicate-label semantics are load-bearing

Labeled and complete packs introduce a correctness obligation absent from purely positional packs.

Given concrete expansions:

```text
**L₁
**L₂
```

if both contain:

```text
#name
```

the application/type assembly is invalid.

There is no “last wins.”

This matches the established pack concatenation law:

```text
keys(L₁) ∩ keys(L₂) = ∅
```

For symbolic packs, C2 needs deferred validation rather than guessing.

That means duplicate-label checking must happen:

```text
during concrete normalization
after substitution makes labels known
during generic instantiation when necessary
before publishing an invalid concrete structural type
```

It must not be deferred into VM execution.

---

# 31. Performance requirements

C2 should remain deterministic and linear.

For `n` source generic/type-pack items:

```text
assembly:
    O(n)

positional canonicalization:
    O(number of segments + concrete elements)

labeled canonicalization:
    O(number of entries)
    expected O(1) duplicate lookup per label

complete pack assembly:
    O(positional + labeled contributions)
```

No:

```text
pack partition search
recursive permutation testing
backtracking over binder boundaries
exponential normalization
```

should exist.

Nested concat terms should be flattened when interned:

```text
Concat(A, Concat(B, C))
```

becomes one segment sequence.

Repeated substitutions should not produce a deeply nested concat tree.

---

# 32. Runtime boundary

C2 remains a compiler/type-system checkpoint.

It must not introduce:

```text
runtime TypePack objects
runtime LabeledTypePack objects
runtime CompleteTypePack objects
VM pack-solving bytecodes
runtime generic pack inference
heap allocations for type-level pack terms
VM dispatch to evaluate type-level *
VM dispatch to evaluate type-level **
VM dispatch to evaluate type-level ***
```

The same tokens can have runtime argument-expansion semantics in their already-owned call contexts, but C2's static generic/type-level processing remains compiler-semantic work.

This distinction is crucial:

```text
same operator algebra
≠
same runtime representation
```

---

# 33. Callable runtime capture remains a separate concern

The ratified argument-pack specification includes runtime capture forms such as:

```phalcom
method(*args)
method(**labels)
method(***arguments)
```

Some of that runtime implementation has its own existing rollout state.

C2 should not absorb outstanding runtime F.3/calling-convention work simply because the syntax shares the same operators.

C2 owns:

```text
generic/type-level binding
type unpacking
callable Type formation
canonical type-pack semantics
```

The runtime callable checkpoint owns:

```text
actual incoming runtime pack capture
frame layout
calling convention
bytecode/VM transport
```

Shared semantics, separate implementation ownership.

---

# 34. RecordRow remains distinct

C2 must preserve one of C1's strongest structural distinctions:

```text
LabeledTypePack != RecordRow
```

Therefore:

```text
**L
```

in a Tuple/type-pack context produces or consumes:

```text
LabeledTypePack
```

not `RecordRow`.

C2 must not silently allow:

```text
LabeledTypePackConcat
```

to become a record row because both happen to contain labels and types.

If future Record type spread wants:

```text
RecordRow
```

conversion, that requires an explicit structural operation or separately owned rule.

This checkpoint should not blur the domains merely to reuse syntax.

---

# 35. Complete pack and ArgumentPackType are also different categories

The ratified runtime argument-pack specification describes a call-shaped argument pack and callable-domain semantics.

C2's complete type-pack term is a **type-level pack shape**.

These should not be conflated automatically with a reflective proper:

```text
ArgumentPackType
```

if/where that type exists.

Conceptually:

```text
CompleteTypePack
    type-level domain used by compile-time generic/structural composition

ArgumentPack
    runtime/pre-dispatch value-shape concept

ArgumentPackType
    reflective proper Type classifying accepted runtime argument packs,
    if/where the type system exposes it
```

They are related but occupy different semantic levels.

C2 should provide explicit normalization/conversion at the callable-domain boundary rather than collapse them into one Rust enum.

---

# 36. Diagnostics required for completeness

C2 should have specific diagnostics for the new semantic failures rather than funneling them into generic arity errors.

| Failure | Expected diagnostic concept |
|---|---|
| illegal binder ordering | pack binder violates lane/capture order |
| duplicate `*` binder | positional pack already captured |
| duplicate `**` binder | labeled pack already captured |
| `***` combined with conflicting split capture | complete pack conflicts with lane captures |
| binder after terminal `***` | complete pack binder must be terminal |
| `*X` wrong domain | positional expansion requires positional/complete-compatible pack |
| `**X` wrong domain | labeled expansion requires labeled/complete-compatible pack |
| `***X` single-lane domain | complete expansion requires complete two-lane source |
| extra positionals without `*`/`***` | generic positional arity mismatch |
| extra labels without `**`/`***` | generic labeled shape mismatch |
| duplicate concrete labels | duplicate labeled generic/type contribution |
| symbolic label conflict after substitution | instantiated pack labels overlap |
| empty application on fixed-only generic | missing required generic argument |
| `...` used as pack operator | not valid pack syntax |
| pack decomposition required | unsupported pack decomposition/inference |
| wrong structural context | expansion not admissible in this type context |

Diagnostics should preserve source ranges for the operator and operand independently.

---

# 37. What C2 explicitly does not implement

The checkpoint should stop before these features:

```text
pack decomposition inference
prefix/suffix pack equation solving
unknown pack-length arithmetic
Pack::Length
Pack::Slice
public pack concatenation operator unless already separately accepted
arbitrary user-defined expansion protocols
general nominal const generics
GATs
open type families
implicit union distribution
runtime generic type solving
runtime TypePack/CompleteTypePack values
new callable ABI
general LSP UI for packs
```

C2 may create internal concat terms because expansion requires them.

That does **not** mean C2 exposes public `Concat` type members.

---

# 38. Recommended implementation architecture

The clean implementation should extend the existing compiler in place rather than create a “variadic subsystem.”

The core flow should become:

```text
source generic/type syntax
        │
        ▼
lane-aware syntax items
  Single / * / ** / ***
        │
        ▼
type-level pack assembler
        │
        ├── positional → TypePack / TypePackConcat
        │
        ├── labeled    → LabeledTypePack / LabeledTypePackConcat
        │
        └── complete   → CompleteTypePack
        │
        ▼
binder-aware generic signature application
        │
        ▼
one canonical argument per generic binder
        │
        ▼
existing GenericInstantiation / TypeSubstitution
        │
        ▼
existing C1 structural terms
        │
        ├── TupleFromPacks
        ├── Map
        ├── projections
        └── ordinary Applied types
        │
        ▼
metadata / fingerprint / incremental / source index
```

The significant new semantic primitives are therefore relatively small:

```text
GenericParameterMode
CompleteTypePack domain/term
TypePackConcat
LabeledTypePackConcat
lane-aware generic argument assembly
pack expansion/unpack AST nodes
```

Everything downstream should reuse C1.

---

# 39. Likely C2 plan decomposition

I would probably split C2 into three numbered plans.

### C2.P1 — Canonical Pack Composition and Binder Semantics

This should establish the semantic substrate:

```text
CompleteTypePack kind/domain
GenericParameterMode
TypePackConcat
LabeledTypePackConcat
lane projection rules
canonicalization
substitution
metadata nodes
```

This plan should finish with semantic-unit tests and no broad source surface dependency.

### C2.P2 — Variadic Generic Binding and Structural Type Expansion

This should wire source syntax and applications:

```text
* / ** / *** generic binders
generic argument expansions
empty specialization <>
binder-aware application
Tuple type expansion
callable-domain type unpacking
declaration-site integration
diagnostics
source identity
presentation
```

### C2.P3 — Inference/Persistence/Incremental Integration and Checkpoint Closure

This should certify:

```text
whole-pack inference
substitution/materialization
metadata/module boundary
fingerprints
incremental/cold parity
dense packs
duplicate-label residuals
no VM path
compatibility with fixed generics
roadmap re-baselining
```

That mirrors the successful C1 pattern without making C2 artificially monolithic.

---

# 40. Representative positive acceptance cases

A complete C2 should be able to express and prove behavior equivalent to:

```phalcom
class Positional<*Ts> {}
class Labeled<**Ls> {}
class Complete<***P> {}
```

Zero and positional capture:

```phalcom
Positional<>
Positional<Int>
Positional<Int, String>
```

with:

```text
Ts = []
Ts = [Int]
Ts = [Int, String]
```

Split capture:

```phalcom
class Split<T, *Ts, **Ls> {}
```

with a source argument assembly equivalent to:

```text
T  = first positional
Ts = remaining positional pack
Ls = complete labeled pack
```

Complete capture:

```phalcom
class Forward<T, ***P> {}
```

where `P` retains both remaining lanes.

Symbolic forwarding:

```text
P : TypePack
L : LabeledTypePack

target<*P, **L>
```

Complete structural expansion:

```text
Q : CompleteTypePack

target<***Q>
```

Structural Tuple formation:

```phalcom
(*P,)
(**L,)
(***Q,)
```

Composition:

```text
*P
*P::Map<Option>

**L
**L::Map<Option>
```

and callable-domain forms:

```phalcom
(*P,) -> R
(**L,) -> R
(***Q,) -> R
```

all without needing runtime pack objects.

---

# 41. Representative hostile cases

The acceptance suite should also deliberately reject or preserve symbolic failure for cases equivalent to:

```text
second positional pack binder
second labeled pack binder
parameter after terminal ***
conflicting *** and split-rest ownership

*operand whose domain has no positional lane
**operand whose domain has no labeled lane
***operand that carries only one lane

concrete duplicate labels across ** / *** expansions

fixed-only generic with extra positional pack data
fixed-only generic with labeled pack data

fixed-only generic <>
when required parameters remain

a pack equation that requires unknown prefix/suffix decomposition

...P
<...T>
as pack syntax

implicit LabeledTypePack -> RecordRow conversion

implicit union distribution

runtime invocation of type-level pack normalization
```

For abstract duplicate-label situations, the expected result should be symbolic/stuck rather than premature acceptance or rejection.

---

# 42. C2 checkpoint invariants

I would make these the durable C2 laws:

```text
C2-01
* always denotes the positional lane.

C2-02
** always denotes the labeled lane.

C2-03
*** always denotes the complete two-lane pack.

C2-04
The surrounding grammar changes capture/expansion/unpacking context,
not the lane meaning of the operator.

C2-05
... is not pack syntax.

C2-06
* binders produce TypePack terms.

C2-07
** binders produce LabeledTypePack terms.

C2-08
*** binders produce complete type-pack terms preserving both lanes.

C2-09
Complete packs are distinct from TypePack and LabeledTypePack.

C2-10
Complete pack formation must not create a second concrete Tuple Type identity.

C2-11
Generic parameter mode is distinct from its semantic kind/domain.

C2-12
Canonical generic applications have one semantic binding per declared binder.

C2-13
Positional symbolic composition is stable and canonical.

C2-14
Labeled symbolic composition preserves order and uniqueness obligations.

C2-15
Concrete duplicate labels are invalid; unknown overlap remains symbolic.

C2-16
Generic expansion is deterministic and does not backtrack over pack partitions.

C2-17
Whole-pack inference is permitted; pack decomposition inference is not C2.

C2-18
Tuple/type formation reuses C1 TupleFromPacks and zero-product rules.

C2-19
Map/projection results can be expanded according to their result domain.

C2-20
LabeledTypePack remains distinct from RecordRow.

C2-21
Type-level complete packs are not runtime ArgumentPack objects.

C2-22
No type-level pack operation executes in the VM.

C2-23
Pack binder mode and pack terms are stable metadata/fingerprint inputs.

C2-24
Cold and incremental compilation produce identical canonical results.

C2-25
All fixed-arity generic programs retain their pre-C2 semantics.
```

---

# 43. Overall implementation requirement

The checkpoint should be considered complete when Phalcom can take code that conceptually says:

```text
“I accept an unknown positional sequence of types.”
“I accept an unknown labeled sequence of types.”
“I accept an unknown complete two-lane structural type pack.”
“I forward/project/expand those lanes elsewhere.”
```

and represent all four stages coherently:

```text
source syntax
→ canonical lane-aware semantic terms
→ substitution/normalization
→ stable metadata/incremental identity
```

without:

```text
flattening labels
inventing parallel pack types
requiring concrete pack lengths
backtracking over pack splits
turning packs into runtime objects
or repurposing ...
```

The central architectural achievement of C2 should therefore be:

> **Variadicity becomes source-level binding and composition over C1's canonical structural domains, while `*`, `**`, and `***` retain exactly one lane meaning throughout the language.**

That gives later pack inference and callable ABI work a mathematically stable substrate rather than forcing those future checkpoints to reverse-engineer variadic semantics from parser or runtime special cases.