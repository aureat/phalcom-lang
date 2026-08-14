# Polymorphism, Generics, Binders, and Substitution

## Purpose

Generic typing is mostly about **binding discipline** and **relation-preserving substitution**. Source syntax such as `Box<T>` is easy; the difficult parts are identity, capture avoidance, constraints, member views, inference, recursion, reflection, and deciding which aspects of specialization are semantic versus runtime.

## 1. Three forms of polymorphism

### Parametric polymorphism

A definition behaves uniformly for arbitrary type arguments:

```text
identity<T>(x: T) -> T
```

Conceptually:

```text
identity : ∀T. T -> T
```

Uniformity can be weakened by reflection, bounds, specialization, or dynamic tests. If a generic body can inspect reified `T`, classic parametricity theorems may not apply unchanged.

### Subtype polymorphism

A function written for a supertype accepts subtype values:

```text
Animal parameter accepts Cat
```

This is driven by `<:`, not universal quantification.

### Ad-hoc polymorphism

Different implementations selected by some dispatch/overload rule. Ordinary Phalcom message dispatch is receiver/selector based. Type metadata must not silently introduce type-directed overload selection.

Keep these mechanisms distinct.

## 2. Universal quantification

A generic method signature can be modeled as:

```text
∀α. T
```

Formation rule:

```text
Δ, α:κ ⊢ T type
────────────────
Δ ⊢ ∀α:κ. T type
```

Instantiation introduces a chosen/admissible type argument `U`:

```text
f : ∀α. T
Δ ⊢ U type
────────────────
f[U] : T[U/α]
```

In inference, `U` may initially be a fresh metavariable `?α`, later solved.

## 3. Binder identity

Source names are presentation. Semantic identity belongs to the declaration.

Conceptual Phalcom identity:

```text
TypeParamId {
  owner: TypeParamOwnerId,
  index: u32,
}
```

Why owner/index matters:

- two declarations can both spell `T`;
- nested generic methods can shadow class parameters;
- rename should not change binding meaning;
- reflection needs stable ownership;
- substitution needs exact target identity.

Do not substitute by `Symbol("T")` after name resolution.

## 4. Free and bound variables

`FV(T)` returns free type parameters occurring in type expression `T`.

Rules:

```text
FV(Int) = ∅
FV(α) = {α}
FV(F<A,B>) = FV(A) ∪ FV(B)
FV(∀α. T) = FV(T) - {α}
```

For a bare generic declaration object such as `Box`, Phalcom's current proposed type-expression design treats declaration-owned parameters as metadata, not free occurrences in the bare origin. Therefore:

```text
Box.typeParameters = [Box.T]
FV(Box) = ∅
```

An applied/open expression can contain actual free occurrences separately.

## 5. Capture-avoiding substitution

Notation:

```text
T[U/α]
```

means replace free occurrences of binder `α` in `T` with `U`.

Basic rules:

```text
α[U/α] = U
β[U/α] = β                 if β != α
C[U/α] = C
F<T1,...,Tn>[U/α] = F<T1[U/α],...,Tn[U/α]>
```

Under a quantifier:

```text
(∀α. T)[U/α] = ∀α. T       # α is rebound/shadowed
```

For distinct binder `β`:

```text
(∀β. T)[U/α]
```

must avoid capturing free `β` occurrences in `U`. Traditional calculi alpha-rename binders. An implementation with globally/owner-unique binder IDs can avoid spelling-based capture by construction.

## 6. Substitution over semantic descriptors

A production implementation traverses a graph, not source text.

Conceptual algorithm:

```text
substitute(type_id, env, memo):
  if memo contains type_id: return memo[type_id]

  match TypeData(type_id):
    TypeParam(p):
      return env.get(p).unwrap_or(type_id)

    Applied(origin,args):
      new_args = args.map(|a| substitute(a, env, memo))
      return canonical_apply(origin, new_args)

    Union(members):
      return canonical_union(members.map(substitute))

    Callable(params,result,effects):
      substitute each type-bearing component

    ForAll(binders, body):
      env2 = env.without(binders)
      return rebuild(... substitute(body, env2) ...)

    Nominal/Special:
      return type_id or recurse into explicit parameters if semantic form requires
```

Memoization prevents repeated work/cycles. Rebuilding through canonical constructors preserves normalization invariants.

## 7. Substitution environment

Use semantic IDs:

```text
TypeEnvironment = Map<TypeParamId, TypeId>
```

Properties:

- immutable/persistent environments are easy to cache and share;
- environment equality/order should be deterministic;
- unresolved inference variables belong in solver terms, not permanent reflection environments;
- nested substitution composition must define precedence.

Composition:

```text
(T[S/β])[U/α]
```

is not generally the same operation as a single unordered string replacement. Define environment composition in terms of binder identities and substituted range values.

## 8. Substitution lemma for generic application

If:

```text
Δ, α:κ ⊢ T type
Δ ⊢ U : κ
```

then:

```text
Δ ⊢ T[U/α] type
```

assuming `U` satisfies bounds/constraints associated with `α`.

This property explains why application validation occurs before publishing an applied member view. A substitution environment containing an invalid type argument can break all downstream assumptions.

## 9. Generic application is not runtime specialization

Separate five dimensions:

```text
1. semantic applied type identity      Box<Int>
2. reflected applied descriptor        object visible at runtime?
3. member type substitution            value: T -> value: Int
4. runtime class/allocation identity    same Box origin class or specialized class?
5. code specialization                  same bytecode/JIT code or specialized version?
```

They need not move together.

Phalcom's typing design direction explicitly keeps type metadata from implicitly changing ordinary dispatch/layout/allocation. If applied-type canonicalization is ratified, it concerns semantic descriptor identity, not automatic creation of runtime subclasses.

## 10. Applied member views

Given:

```text
class Pair<A,B> {
  first -> A
  second -> B
  swap() -> Pair<B,A>
}
```

for `Pair<Int,String>`:

```text
env = { Pair.A ↦ Int, Pair.B ↦ String }
first : Int
second : String
swap : () -> Pair<String,Int>
```

Prefer lazy substituted views keyed by `(member identity, applied type/environment)` over cloning entire member graphs per specialization, unless runtime architecture requires materialization.

Source reflection may still need original annotation `A`; normalized semantic view can expose `Int`. Preserve both if the spec requires source fidelity.

## 11. Bounds

Upper bound:

```text
T : Base
```

means admissible argument `U` must satisfy:

```text
U <: Base
```

If bound mentions outer parameters:

```text
class Outer<T> {
  method<U: Container<T>>(...)
}
```

resolve/substitute outer `T` before validating `U` in an applied `Outer<Int>` context.

## 12. F-bounded polymorphism

F-bound:

```text
T <: Comparable<T>
```

creates a recursive obligation, not necessarily an infinite type.

Validation of candidate `U` asks:

```text
U <: Comparable<U>
```

The relation solver needs cycle-aware obligation handling. Guardedness/coherence rules determine which recursive constraints are legal.

A current Phalcom typing document intentionally defers same-signature recursive restrictions such as this until the later constraint/inference design. The skill teaches the machinery without claiming current support.

## 13. Finite exact constraint sets

A finite constraint set:

```text
T in (A, B, C)
```

can mean admissible explicit arguments are exactly equivalent to one listed member:

```text
U ≡ A ∨ U ≡ B ∨ U ≡ C
```

This is not the same as:

```text
U <: A | B | C
```

because subtype values below `A` might satisfy the latter but not exact membership. Keep representation and diagnostics separate from bounds.

## 14. Variance belongs to the constructor relation

For declaration `F<out T>`, covariance promises:

```text
A <: B => F<A> <: F<B>
```

This promise is safe only if occurrences of `T` obey variance-position rules. See `functions-callables-and-variance.md`.

Method-owned generic parameters do not automatically need declaration-site variance because a generic method is universally quantified, not itself necessarily a subtype-forming type constructor.

## 15. Existential types

An existential hides a concrete type while exposing constraints:

```text
∃T. Package<T>
```

Intuition:

> There exists some type `T`; callers may use only operations valid without knowing which `T` it is.

Existentials arise naturally in:

- heterogeneous containers with a hidden element type;
- protocol/existential values with associated types;
- opaque module/package boundaries;
- FFI handles that preserve an invariant hidden type.

### Pack/unpack

Introduction (pack):

```text
U type     v : Package<U>
──────────────────────────
pack[U,v] : ∃T.Package<T>
```

Elimination (unpack) introduces a fresh abstract/skolem type `κ`:

```text
x : ∃T.Package<T>
unpack x as [κ,p] in e
```

Inside `e`, `κ` is unknown but fixed. It cannot escape in ways that reveal hidden identity.

## 16. Skolemization

When checking a universally quantified requirement or unpacking an existential, replace a bound type variable with a fresh rigid skolem constant.

Difference:

```text
?α   inference metavariable: solver may choose it
κ    skolem: fixed unknown type, solver may not solve it to Int/String
```

Confusing skolems with metavariables causes unsoundness: the checker can "prove" a polymorphic function by choosing a convenient concrete type.

## 17. Rank and higher-rank polymorphism

Rank-1:

```text
∀T. T -> T
```

quantifier appears at outer level; call sites instantiate it.

Higher-rank example:

```text
(∀T. T -> T) -> String
```

The function argument itself must be polymorphic.

Inference for higher-rank types is substantially harder and often requires annotations/bidirectional checking. Do not add higher-rank inference merely because the internal representation can nest `ForAll`.

## 18. Parametricity and reflection

Classic "free theorems" rely on polymorphic code being unable to inspect type arguments.

A pure:

```text
∀T. T -> T
```

has very limited possible total implementations.

If Phalcom reifies type descriptors and generic code can inspect `T`, code may branch on type identity. Then parametricity weakens. This is not automatically bad, but it changes what the theory guarantees.

When using parametric reasoning for optimization/proof, include reification/reflection in assumptions.

## 19. Generalization and mutable state

If future Phalcom supports implicit local polymorphism, classic ML's value restriction is relevant. A mutable reference generalized too freely can be used at several types unsafely.

Sketch:

```text
r = Ref([])       # if generalized as ∀T. Ref<List<T>>
use r as Int list
use r as String list
```

Phalcom's object mutability makes this a real hazard. Declaration-site generics avoid this specific implicit-generalization issue because quantification is explicit.

## 20. Recursive substitutions

Type graphs may be recursive:

```text
Node<T> contains Option<Node<T>>
```

Substitution should use memoized graph rebuilding and canonical constructors. Do not recursively clone until stack overflow.

For transparent recursive aliases, define guardedness/unfolding policy before substitution normalization, or `Alias<T> = Alias<T>`-style cycles can loop forever.

## 21. Reflection and source fidelity

Potential views:

```text
source annotation: T
owner: Box.value
normalized declaration annotation: TypeParam(Box,0)
applied semantic view at Box<Int>: Int
```

All three may be useful. Do not overwrite source metadata with substituted results if reflection/documentation needs original spelling/ownership.

## 22. Implementation hazards

- `HashMap<String, TypeId>` as substitution environment.
- Cloning full class/member objects for every applied type.
- Treating `Box` as implicit `Box<T>` even when bare origin is a closed declaration descriptor.
- Storing inference metavariables in reflective `TypeParameter` objects.
- Applying method-owned substitution to class-owned `T` with same spelling.
- Concluding reified `Box<Int>` requires a distinct runtime class.
- Allowing F-bounds through recursive calls without cycle state.
- Treating a finite constraint set as a union bound.
- Leaking a skolem type outside existential scope.

## 23. Testing obligations

Test:

- owner/index identity stability;
- shadowing of same spelling;
- nested applications;
- substitution composition;
- free-variable computation;
- bounds after outer substitution;
- finite exact constraints;
- recursive structures;
- `Self` substitution rules where ratified;
- existential pack/unpack escape rejection if supported;
- reflection source form versus applied view;
- canonical repeated application identity when canonicalization is specified.

Property: substituting a well-formed admissible argument into a well-formed type should produce a well-formed type.

## 24. Competency questions

1. Why is generic type-parameter identity owner/index rather than spelling?
2. What is capture avoidance, and how do semantic IDs simplify it?
3. Why is `Box<Int>` semantic application separate from code specialization?
4. What is the difference between a bound and a finite exact constraint set?
5. What is a skolem, and why must it be rigid?
6. Why does runtime reification weaken classic parametricity assumptions?
7. When would implicit generalization require a value restriction?
