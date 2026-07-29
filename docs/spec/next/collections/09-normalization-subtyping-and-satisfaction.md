# Normalization, Subtyping, and Satisfaction

## 1. Purpose

This document defines canonical forms and semantic relations so syntax does not become the source of type identity.

## 2. Tuple normalization

### 2.1 Exact tuples

```phalcom
(Int, String)
```

normalizes to:

```text
TupleType(
  positionals = [Int, String],
  labels = {},
  repeatedTail = None
)
```

### 2.2 Repeated tails

```phalcom
(Context, Request, ...)
```

normalizes to:

```text
TupleType(
  positionals = [Context],
  labels = {},
  repeatedTail = Request
)
```

### 2.3 Labeled tuples

```phalcom
(Int, name: String)
```

normalizes to:

```text
TupleType(
  positionals = [Int],
  labels = { #name: String },
  repeatedTail = None
)
```

## 3. Pack-schema normalization

### 3.1 Homogeneous rest shorthand

```phalcom
*args: T
```

normalizes to:

```text
ArgumentPackType(openPositional = T)
```

```phalcom
**labels: T
```

normalizes to:

```text
ArgumentPackType(openLabeled = T)
```

```phalcom
***arguments: T
```

normalizes to:

```text
ArgumentPackType(
  openPositional = T,
  openLabeled = T
)
```

### 3.2 Reserved-key schemas

In an argument-pack context:

```phalcom
(*: A, **: B)
```

normalizes to:

```text
ArgumentPackType(
  openPositional = A,
  openLabeled = B
)
```

### 3.3 Exact complete schemas

```phalcom
(Int, name: String)
```

in complete-pack context normalizes to:

```text
ArgumentPackType(
  fixedPositionals = [Int],
  fixedLabels = { #name: String }
)
```

## 4. Type-unpack normalization

Given normalized pack type `P`:

```text
P = ⟨Fp, Op, Fl, Ol⟩
```

where `Fp` and `Fl` are fixed lanes and `Op` and `Ol` are optional open-lane types:

```text
*P    = ⟨Fp, Op, {}, None⟩
**P   = ⟨[], None, Fl, Ol⟩
***P  = P
```

Example:

```phalcom
type P = (Request, timeout: Duration)
```

```phalcom
(*P,) -> R
```

normalizes to:

```phalcom
(Request) -> R
```

```phalcom
(**P,) -> R
```

normalizes to:

```phalcom
(timeout: Duration) -> R
```

```phalcom
(***P,) -> R
```

normalizes to:

```phalcom
(Request, timeout: Duration) -> R
```

## 5. Tuple satisfaction

An exact tuple value satisfies an exact Tuple Type when:

1. positional counts match;
2. corresponding positional values satisfy corresponding types;
3. label sets match exactly;
4. corresponding labeled values satisfy corresponding types.

For repeated tails:

1. fixed positionals must match;
2. every remaining positional value satisfies the repeated-tail type;
3. labeled slots still match exactly.

## 6. Record satisfaction

A Record satisfies an exact Record Type when:

1. key sets match exactly;
2. each field value satisfies its field type;
3. key identity uses `LabelKey` equality.

Open Record Types are not defined in this suite.

## 7. Set satisfaction

A Set satisfies `Set<T>` when every element satisfies `T`.

```text
S ⊨ Set<T> iff ∀v ∈ S, v ⊨ T
```

For mutable Sets, satisfaction is a property of the current contents and does not impose future mutation guards.

## 8. Argument-pack satisfaction

For domain:

```text
D = ⟨Fp, Op, Fl, Ol⟩
```

and pack:

```text
P = ⟨p, l⟩
```

`P ⊨ D` when:

```text
|p| ≥ |Fp|
```

and every fixed positional matches, and:

```text
|p| = |Fp| if Op is None
```

otherwise every remaining positional satisfies `Op`.

For labels:

```text
keys(Fl) ⊆ keys(l)
```

all fixed labeled values match, and:

```text
keys(l) = keys(Fl) if Ol is None
```

otherwise every additional label value satisfies `Ol`.

## 9. Tuple subtyping

### 9.1 Exact immutable tuples

**PROVISIONAL:** Exact immutable Tuple Types are covariant slot-wise.

```text
(A₀, …, Aₙ) <: (B₀, …, Bₙ)
```

when:

```text
Aᵢ <: Bᵢ for every i
```

and label sets are equal with covariant value types.

If Tuples are mutable, invariance may be required. This depends on the collection mutability model and remains reviewable.

### 9.2 Repeated tails

An exact Tuple Type is a subtype of a repeated-tail Tuple Type when its fixed prefix matches and all remaining elements satisfy the repeated type.

```phalcom
(Int, Int, Int) <: (Int, ...)
```

## 10. Record subtyping

**OPEN:** Width subtyping for Records has not been ratified.

Candidate A — exact only:

```text
{a: A, b: B} is not a subtype of {a: A}
```

Candidate B — immutable width subtyping:

```text
{a: A, b: B} <: {a: A}
```

The candidate interacts with exact `**record` capture and must be resolved explicitly.

## 11. Set variance

**PROVISIONAL:** Mutable `Set<T>` is invariant. Immutable `FrozenSet<T>` may be covariant.

```text
Set<Dog> </: Set<Animal>
FrozenSet<Dog> <: FrozenSet<Animal>
```

## 12. Callable subtyping

Let `Calls(D)` be the accepted-pack set of domain `D`.

```text
Callable<D₁, R₁> <: Callable<D₂, R₂>
```

when:

```text
Calls(D₂) ⊆ Calls(D₁)
R₁ <: R₂
```

This definition handles fixed, open, positional, and labeled domains uniformly.

## 13. Join and inference

**PROVISIONAL:** Finite Tuple values infer exact Tuple Types.

```phalcom
(1, "a")
```

infers:

```phalcom
(Int, String)
```

Repeated-tail types arise from annotations, widening, or joins rather than initial finite-literal inference.

Possible joins:

```text
join((Int), (Int, Int, Int)) = (Int, Int, ...)
join((), (Int), (Int, Int))  = (Int, ...)
```

Heterogeneous joins remain **OPEN**.

## 14. Cycles and satisfaction

Reflective satisfaction MUST terminate for cyclic values and recursive Types.

Recommended algorithm:

```text
visited = Set<(objectIdentity, typeIdentity)>
```

Before descending, insert the pair. Re-visiting an active pair succeeds coinductively unless a contradiction has already been found.

## 15. Static soundness boundary

The type calculus may be sound relative to declared annotations, but annotations are inert at runtime.

Therefore:

```phalcom
method(*args: Int)
```

is not a runtime guarantee unless the call is statically checked or explicitly validated.

The specification claim is:

> Well-typed checked programs preserve argument-pack compatibility. Dynamically unchecked programs remain permissive.
