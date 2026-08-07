# Foundations and Formal Notation

## 1. Semantic universes

This suite uses the following abstract domains:

```text
Value       runtime Phalcom value
Type        first-class reflective type value
Symbol      interned symbolic name
Selector    complete message selector
LabelKey    structural Tuple/Record key
CallLabel   key legal in a call's labeled lane
```

**RATIFIED:** Structural labels may include at least:

```text
LabelKey ::= Symbol | Selector
```

**PROVISIONAL:** Call labels remain narrower:

```text
CallLabel ::= Symbol
```

Selector-valued call labels are reviewed separately in the open-questions document.

## 2. Lane notation

A pack is written:

```text
P = ⟨p, l⟩
```

Where:

```text
p = [v₀, v₁, …, vₙ]
l = { k₀ ↦ v₀, k₁ ↦ v₁, …, kₘ ↦ vₘ }
```

The labeled lane preserves insertion order for reflection and deterministic call assembly, while lookup and duplicate detection use key identity.

Projection functions:

```text
π*(⟨p, l⟩)   = ⟨p, ∅⟩
π**(⟨p, l⟩)  = ⟨[], l⟩
π***(⟨p, l⟩) = ⟨p, l⟩
```

For source-level operators:

```text
*P    denotes π*(P)
**P   denotes π**(P)
***P  denotes π***(P)
```

The notation describes lane selection only. The surrounding grammar determines whether the operation is capture, expansion, or type unpacking.

## 3. Partial pack concatenation

Call assembly uses a partial operation `⊕`:

```text
⟨p₁, l₁⟩ ⊕ ⟨p₂, l₂⟩ = ⟨p₁ ++ p₂, l₁ ∪ l₂⟩
```

This operation is defined only when:

```text
keys(l₁) ∩ keys(l₂) = ∅
```

If labels overlap, assembly fails with a duplicate-label error.

Properties:

```text
identity:      P ⊕ ⟨[], ∅⟩ = P
associativity: (A ⊕ B) ⊕ C = A ⊕ (B ⊕ C), when all are defined
non-override:  duplicate keys never choose a winner
```

The operation is not commutative because positional ordering and labeled insertion order are observable.

## 4. Structural type notation

An exact Tuple Type is represented abstractly as:

```text
TupleType {
  positional: [T₀, T₁, …, Tₙ]
  labels: OrderedMap<LabelKey, Type>
  repeatedTail: Option<Type>
}
```

An exact Record Type is represented as:

```text
RecordType {
  fields: OrderedMap<LabelKey, Type>
}
```

An argument-pack type is represented as:

```text
ArgumentPackType {
  fixedPositionals: [T₀, T₁, …, Tₙ]
  openPositional: Option<Type>
  fixedLabels: OrderedMap<CallLabel, Type>
  openLabeled: Option<Type>
}
```

A Set Type is represented as:

```text
SetType {
  element: Type
  mutability: SetMutability
}
```

## 5. Satisfaction notation

```text
v ⊨ T
```

means value `v` satisfies type `T` under explicit reflective satisfaction.

```text
P ⊨ D
```

means call pack `P` is accepted by argument domain `D`.

Annotations do not automatically enforce either relation at runtime.

## 6. Contextual interpretation

A tuple expression `E` may be interpreted differently by a consuming context:

```text
C ⊢ E ⇝ X
```

Meaning:

> In context `C`, expression `E` is interpreted as semantic object `X`.

Relevant contexts:

```text
ValueContext
TupleTypeContext
RecordTypeContext
ArgumentPackContext
CallableDomainContext
PositionalRestContext
LabeledRestContext
CompleteRestContext
```

Example:

```text
ValueContext ⊢ (*: Int) ⇝ Tuple([#* ↦ Int])
TupleTypeContext ⊢ (*: Int) ⇝ TupleType(labels = {#* ↦ Int})
PositionalRestContext ⊢ (*: Int) ⇝ ArgumentPackType(openPositional = Int)
```

The source tuple remains ordinary. The consuming context determines the produced semantic Type.

## 7. Source preservation and normalization

Reflection SHOULD preserve:

```text
sourceForm       original syntax or source-range-backed representation
normalizedType   canonical semantic Type value
```

Two source forms MAY normalize to equal Type values:

```phalcom
*args: Int
*args: (*: Int)
```

Both normalize to a positional open-lane schema of `Int`.

Source preservation is diagnostic metadata and MUST NOT alter type equality.
