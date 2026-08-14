# Products, Sums, Unit, Bottom, `Option`, and `Result`

## Purpose

This reference gives the algebraic foundations for tuple/record/product types, variants/sums, unit, bottom, `Option`, and `Result`. It is especially relevant to Phalcom because tuples have structured lanes, `Option` is the intended explicit absence mechanism, and the normative core type lattice assigns precise meaning to `Never`, `()`, `Any`, and `Dynamic`.

## 1. Product types

A product combines independent information.

```text
A × B
```

An inhabitant contains one `A` **and** one `B`.

For finite types:

```text
|A × B| = |A| · |B|
```

This cardinality law is useful for understanding information content and spotting isomorphisms.

### Introduction

```text
Γ ⊢ a : A    Γ ⊢ b : B
───────────────────────
Γ ⊢ (a,b) : A × B
```

### Elimination

```text
Γ ⊢ p : A × B
────────────────
Γ ⊢ fst(p) : A
Γ ⊢ snd(p) : B
```

A language need not expose `fst`/`snd`; positional/labeled projections implement the same semantic role.

## 2. Tuples as ordered products

For positional tuple types:

```text
(Int, String) != (String, Int)
```

because positions carry meaning.

Subtyping of immutable tuple products, if supported structurally, can be pointwise:

```text
A1 <: B1 ... An <: Bn
──────────────────────
(A1,...,An) <: (B1,...,Bn)
```

Arity must match unless Phalcom defines tuple width subtyping explicitly.

## 3. Records as labeled products

Record:

```text
{x: Int, y: String}
```

is a product indexed by labels. If record type order is semantically irrelevant:

```text
{x:Int,y:String} ≡ {y:String,x:Int}
```

canonical representation should normalize label order while source reflection can preserve original order separately.

### Width subtyping

For immutable/read-only structural records, a record with more fields can often be used where fewer are required:

```text
{x:Int, y:String} <: {x:Int}
```

### Depth subtyping

Field types can vary covariantly only for read-only fields. Writable fields introduce contravariant input and generally force invariance.

Do not apply naïve covariant structural record rules to mutable Phalcom fields.

## 4. Phalcom tuple positional/labeled lanes

If a Phalcom tuple carries a positional lane plus a labeled lane, model its type as a richer product shape:

```text
TupleType {
  positional: [T0, T1, ...],
  labeled: {label_i -> Ti}
}
```

A pure record can correspond to the labeled component, but a projection that forgets positional entries is information loss.

Therefore a mapping:

```text
Tuple(positional,labeled) -> Record(labeled)
```

is generally not an isomorphism unless positional data is empty or recoverable.

This is a good example of why "records are tuples with labels" can be too loose for semantic equality.

## 5. Product associativity and source shape

Mathematically:

```text
(A × B) × C ≅ A × (B × C)
```

but tuples with concrete nesting syntax can preserve nesting as observable structure:

```text
((a,b),c) != (a,(b,c))
```

So associativity is often an isomorphism, not definitional equality, for language tuples.

## 6. Sum types

A sum represents alternatives:

```text
A + B
```

An inhabitant is tagged as left/right and carries one payload.

For finite types:

```text
|A + B| = |A| + |B|
```

### Introduction

```text
Γ ⊢ a : A
──────────────
Γ ⊢ inl(a) : A+B

Γ ⊢ b : B
──────────────
Γ ⊢ inr(b) : A+B
```

### Elimination

To consume `A+B`, handle both cases:

```text
Γ ⊢ e : A+B
Γ,x:A ⊢ e1 : C
Γ,y:B ⊢ e2 : C
────────────────────────
Γ ⊢ case e of ... : C
```

This is the theoretical basis for exhaustive variant pattern matching.

## 7. Tagged sums versus unions

An ADT sum:

```text
Result<T,E> = Ok(T) | Err(E)
```

is tagged: `Ok(x)` and `Err(x)` remain distinguishable even if `T` and `E` are same runtime payload type.

A type union:

```text
T | E
```

may not retain such provenance/tag distinction.

Therefore:

```text
Result<Int,Int>
```

is not equivalent to `Int` or `Int | Int` merely by payload types.

## 8. Unit

A unit type has exactly one inhabitant:

```text
|Unit| = 1
```

Product identity up to isomorphism:

```text
A × Unit ≅ A
```

Phalcom's normative core design identifies unit with the exact empty tuple:

```text
value: ()
type/source spelling: ()
reflective runtime name: Unit
```

and places it under tuple/object hierarchy according to that specification.

Do not conflate "one possible value" with "no value exists".

## 9. Bottom / `Never`

Bottom has no inhabitants:

```text
|Never| = 0
```

and normative Phalcom core design specifies:

```text
Never <: T
```

for every type `T`.

It describes expressions that do not complete normally, such as thrown/fatal/returning control transfers under the exact dynamic semantics.

Algebraically:

```text
A + Never ≅ A
A × Never ≅ Never
```

Phalcom also normatively specifies type-normalization identities:

```text
T | Never = T
T & Never = Never
```

Do not use `Never` for analysis failure. Unknown information has many possible values; bottom has none.

## 10. `Option<T>`

Canonical tagged sum:

```text
Option<T> = Some(T) | None
```

Phalcom's normative core design models `None` as the nullary alternative and gives unconstrained `None` the principal type:

```text
Option<Never>
```

assuming `Option` covariance.

Why:

- `None` contains no `T` payload;
- `Never` is the most specific impossible payload type;
- covariance yields `Option<Never> <: Option<T>` for every `T`.

This is much more precise than inventing a hidden nullable pointer type.

## 11. `Option<Unit>` and booleans

Cardinality:

```text
|Option<Unit>| = |None| + |Some(Unit)| = 1 + 1 = 2
```

A Boolean-like nullary sum also has two inhabitants. Therefore they can be isomorphic.

But they are not automatically semantically equal:

```text
None / Some(())
```

carry different domain meaning from:

```text
false / true
```

Constructors, reflection, APIs, and pattern names remain distinct.

## 12. `Result<T,E>`

```text
Result<T,E> = Ok(T) | Err(E)
```

Useful algebraic cases:

```text
Result<Unit, Never> ≅ Unit
Result<T, Never> ≅ T
Result<Never, E> ≅ E tagged as Err branch
```

Again, isomorphism does not force normalization equality.

## 13. `Result` versus exceptions

`Result<T,E>` is a value sum. Throwing is a control effect.

Differences:

```text
Result: explicit in normal value flow, pattern/combinator handling
throw: abrupt control transfer, stack unwinding/nonlocal effect
```

A checker can model both, but should not rewrite one to the other casually. See `effects-control-and-computation-types.md`.

## 14. Empty products and empty sums

Empty product:

```text
∏ over zero components = Unit
```

Empty sum:

```text
∑ over zero alternatives = Never
```

This gives a clean algebraic relationship:

```text
zero fields -> one possible record/tuple value
zero variants -> zero possible values
```

It explains why unit and bottom are dual-looking but opposite.

## 15. Products, function domains, and calling convention

Mathematically:

```text
(A × B) -> R
```

can represent a function taking a pair. A surface function taking two arguments:

```text
(A,B) -> R
```

need not allocate a tuple at runtime.

Phalcom callable parameter domains can be tuple-shaped semantically while compiler/VM use direct stack slots/packs.

Do not infer runtime allocation from type-theoretic product representation.

## 16. Generic variance of products and sums

Immutable products are covariant in component types:

```text
A <: B => (A,C) <: (B,C)
```

Tagged sum constructors are usually covariant in payloads if they only contain/read them.

`Option<out T>` naturally supports covariance if no API consumes arbitrary `T` in a way violating variance.

Mutable containers are different; their type parameter can become invariant.

## 17. Pattern refinement

For:

```text
x : Option<String>
```

matching `Some(value)` yields branch facts:

```text
x is Some
value : String
```

matching `None` yields:

```text
x is None
```

The declared type of `x` remains `Option<String>`; flow environment can carry refined variant propositions.

## 18. Runtime representation is independent

A VM may optimize:

- `None` as a singleton immediate;
- `Unit` as one singleton/immediate;
- variants by compact tags;
- tuples with specialized layouts.

Type semantics remain tagged/product semantics. Representation equality does not imply type equality.

## 19. Testing obligations

- tuple order sensitivity;
- record label-order equivalence if ratified;
- mutable versus immutable depth subtyping;
- empty tuple/unit identity;
- `Never` branch join behavior;
- `None : Option<Never>` inference;
- widening `Option<Never>` to `Option<T>` via covariance;
- `Option<Unit>` remains distinct from Bool;
- `Result<T,Never>` isomorphism does not erase reflection identity;
- exhaustive variant matching.

## 20. Failure modes

- Treating unit as absence.
- Modeling `None` as hidden null outside `Option` semantics.
- Equating tuple and record types after forgetting positional information.
- Using cardinality to define type equality.
- Treating thrown exceptions as `Result` values without explicit effect transformation.
- Assuming products allocate runtime tuples.

## 21. Competency questions

1. Why is `A × Unit` isomorphic to `A` but not necessarily definitionally equal?
2. Why does `None` naturally synthesize `Option<Never>` under covariance?
3. What information is lost mapping a positional+labeled tuple to a record of labels only?
4. Why is `Result<T,E>` different from `T | E`?
5. Why is `Never` not analyzer unknown?
