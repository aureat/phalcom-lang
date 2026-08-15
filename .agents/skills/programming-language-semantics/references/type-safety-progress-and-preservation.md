# Type Safety, Progress, and Preservation

Progress and preservation connect static typing to dynamic semantics. Phalcom is dynamic-first, so any theorem must be scoped to a checked subset/mode rather than claimed for all source programs.

## 1. Classic theorems

```text
Progress:
if ∅ ⊢ e : T
then e is a value or ∃e'. e → e'
```

```text
Preservation:
if Γ ⊢ e : T and e → e'
then Γ ⊢ e' : T
```

Together they rule out certain unmodeled stuck states for closed well-typed programs.

## 2. What stuck means

A configuration is stuck when it is not final and no semantic rule applies.

Defined outcomes are not stuck:

```text
MessageNotUnderstood
RangeError
user throw
IO error
runtime contract failure if modeled
```

Type safety excludes only failures type system promises to prevent.

## 3. Canonical forms

Progress proofs rely on facts such as:

```text
if v : Bool then v has one of runtime boolean forms permitted by semantics
if v : callable type then v supports compatible call behavior or declared dynamic boundary
```

Phalcom complicates canonical forms with class inheritance, protocols, metaclasses, dynamic descriptors, and future gradual types. Define interpretations carefully.

## 4. Substitution lemma

Classic form:

```text
Γ,x:S ⊢ e : T
Γ ⊢ v : S
----------------
Γ ⊢ e[x:=v] : T
```

Runtime may use environments/locations rather than textual substitution, but substitution remains useful in metatheory for local/pure fragments.

## 5. Preservation with subtyping

Often prove:

```text
Γ ⊢ e' : T'
T' <: T
```

rather than exact syntactic type equality. Clarify subsumption and normalization.

## 6. Store/heap preservation

With mutable objects, theorem may quantify over store typing/heap validity:

```text
Γ; Σ ⊢ e : T
wellFormedHeap(Σ,H)
```

and after step obtain extended `Σ'`/`H'` preserving invariants. Allocation grows heap; mutation must respect allowed contracts.

## 7. Dynamic-first Phalcom theorem shape

A plausible scoped claim is:

> In checker-accepted code without dynamic escape, unchecked native assumptions, or unmodeled reflective mutation, every selector send statically guaranteed by checker performs compatible ordinary dispatch or reaches another explicitly modeled safe outcome; it does not fail solely because guaranteed selector is absent/incompatible.

This is a theorem shape, not a ratified guarantee.

## 8. Gradual typing

With `Dynamic`/casts, safety can permit cast/contract failure as explicit outcome. The theorem becomes "does not go wrong except at declared dynamic boundaries" rather than "never fails."

## 9. Protocol conformance

If conformance guarantees required selectors, preservation must account for:

- instance/class side;
- generic substitution;
- visibility;
- parameter/result compatibility;
- runtime method mutation.

## 10. Override variance

Unsafe parameter narrowing can break substitutability:

```text
Base-typed caller may pass A
runtime receiver is Subclass
Subclass override only accepts narrower B
```

This is semantic safety issue, not style.

## 11. `Self` and recursive relationships

Future `Self` types need interpretation across inheritance and class-side behavior. Preservation must ensure methods returning `Self` produce values matching receiver-dependent semantics, not merely lexical declaring class.

## 12. Native trust

Native primitives are axioms unless verified/checked. False native signature can violate preservation immediately.

Options:

```text
trust native contract
runtime validate native boundary
generate checked wrappers
prove native implementation selectively
```

## 13. Reflection/open world

If method mutation is allowed after checking, proof selector exists may stop holding. Sound modes need restrictions, invalidation, guards, or weakened guarantees.

## 14. Concurrency assumptions

Flow-sensitive type refinements over shared mutable fields may not survive yield. Safety theorem must state ownership/effect/scheduler assumptions if it relies on such refinements.

## 15. Testing theorem premises

Create negative fixtures for every premise:

- remove required method after analysis;
- wrong native return;
- Dynamic boundary;
- override variance violation;
- private access from wrong context;
- shared field changed across await.

This exposes hidden assumptions before formal proof.

## 16. Competency checks

1. Why is `MessageNotUnderstood` not automatically type-safety violation in untyped Phalcom?
2. What does preservation mean when result has subtype of expected type?
3. Why can reflection invalidate progress-like guarantees?
4. Which lemma supports function application preservation?
5. What must theorem say about native code it does not verify?
6. Why can yield require theorem premises about shared state?
