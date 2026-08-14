# Functions, Blocks, Callables, and Variance

## Purpose

This reference owns the type theory of callable contracts and variance. Use it when specifying function/block/method types, generic variance, override compatibility, or any rule where a type parameter occurs in positive/negative positions.

Phalcom callables are not all interchangeable runtime values. Ordinary message sends, `Method` descriptors, bound methods, blocks/closures, selector families, constructors, and reflective callable objects may have different identities and control behavior. The type system may share a callable-contract abstraction while preserving those runtime distinctions.

## 1. Callable contract as parameter domain plus result

The mathematical core is:

```text
A -> R
```

For multiple parameters:

```text
(A1, A2, ..., An) -> R
```

A Phalcom callable domain may also include:

```text
parameter order
positional versus labeled lanes
selector/label identity
optional/default presence
rest/pack acceptance
receiver binding state
possibly effects/control behavior
```

Do not reduce compatibility to `Vec<TypeId>` plus return type if call shape matters semantically.

A conceptual callable descriptor:

```text
CallableType {
  domain: ParameterDomain,
  result: TypeId,
  control/effects: optional computation summary,
  callable_kind: block | bound-method | abstract callable ...
}
```

The exact public Phalcom representation is a design question.

## 2. Function subtyping derivation

For safe ordinary function subtyping:

```text
B1 <: A1 ... Bn <: An     R1 <: R2
────────────────────────────────────
(A1,...,An) -> R1 <: (B1,...,Bn) -> R2
```

Why parameters reverse:

A context expecting:

```text
Animal -> Animal
```

may call its value with *any* `Animal`. Replacing it with `Cat -> Animal` is unsafe because the context may pass `Dog`.

A replacement `Object -> Cat` is safe:

```text
Animal <: Object      # replacement accepts at least all expected inputs
Cat <: Animal         # replacement returns a value satisfying expected result
```

Therefore:

```text
(Object -> Cat) <: (Animal -> Animal)
```

This is a substitutability theorem, not stylistic convention.

## 3. Labels and selector identity

Phalcom selector/label identity is a separate axis from type compatibility.

Suppose callable requirements are conceptually:

```text
fetch(key: String) -> Value
fetch(id: Int) -> Value
```

If Phalcom selector identity includes the label `key:` versus `id:`, those are different selectors independent of types. If selector identity does not distinguish another syntactic detail, typing must not silently add it.

Callable compatibility should first establish compatible call shape/selector requirements, then compare types.

Never use parameter type annotations as hidden overload keys unless explicit future semantics ratify type-directed dispatch.

## 4. Parameter-domain subtyping

A robust rule separates **accepted call set** from individual type variance.

Let `Calls(D)` be the set of argument packs valid for parameter domain `D` ignoring value types. A replacement callable domain `D_r` is safe for expected domain `D_e` when:

```text
Calls(D_e) ⊆ Calls(D_r)
```

Then for every corresponding accepted argument position, replacement parameter types must be supertypes/acceptable in the contravariant direction.

This formulation handles:

- optional parameters;
- defaults;
- rest parameters;
- labels;
- fixed arities;
- keyword/pack shapes.

It is better than a pile of special cases because it states the observable contract.

## 5. Optional/default parameters

Example expected callable:

```text
(x: Int) -> R
```

Replacement:

```text
(x: Int, y: String = "") -> R
```

can often accept every call the expected callable accepts, so call-shape substitution may be safe.

Reverse substitution is generally unsafe if the expected context is allowed to provide `y`.

Defaults are about call acceptance; they do not make a parameter's type covariant.

## 6. Rest parameters

A rest parameter can cover many arities:

```text
(Int, *String) -> R
```

To compare against fixed calls, define which pack shapes the rest parameter accepts and how element type constraints apply.

Avoid "rest means compatible with everything". A rest element type still constrains each supplied value, and labeled/rest lanes may differ.

## 7. Constructor callables

A constructor often has two semantic layers:

```text
initializer body: receiver already allocated; normal completion may be Unit
class-side constructor surface: arguments -> Self/instance
```

The callable type exposed to users should match the observable constructor operation, not an internal initializer's fallthrough value.

If Phalcom's normative constructor semantics distinguish these, checker and reflection must do the same.

## 8. Blocks and non-local control

A block is not necessarily a pure function. Invoking it may:

- return normally;
- throw;
- non-locally return from a home frame;
- mutate captured bindings;
- yield/suspend through fiber machinery;
- perform dynamic sends and reflection.

A plain result type `A -> B` describes only normal input/output behavior. If an analysis/checker needs to reason about non-local return or suspension, add a separate computation/effect dimension rather than pretending those effects are variants of `B`.

See `effects-control-and-computation-types.md`.

## 9. Variance as polarity

A type parameter occurrence has polarity relative to a constructor result.

Start at positive (`+`) at the outer type. Traverse type constructors:

- covariant parameter preserves sign;
- contravariant parameter flips sign;
- invariant parameter yields invariant/mixed;
- callable parameter flips sign;
- callable result preserves sign.

Sign algebra:

```text
+ × + = +
+ × - = -
- × + = -
- × - = +
anything × 0 = 0
```

where `0` means invariant/mixed.

## 10. Variance-position validation algorithm

Given declaration:

```text
class Producer<out T> { ... }
```

validate every use of `T` under public/member contracts.

Conceptual traversal:

```text
visit(type, polarity):
  TypeParam(T): record occurrence(polarity)
  Applied(F,args):
    for arg_i:
      visit(arg_i, compose(polarity, variance(F,i)))
  Callable(params,result):
    for p in params: visit(p, flip(polarity))
    visit(result, polarity)
  Union/Intersection(members): visit each with same polarity
  MutableCell<T>: invariant parameter => record 0
```

For declared covariance `out T`, all relevant occurrences must be positive (or absent). A negative or invariant occurrence violates the declaration.

For contravariance `in T`, occurrences must be negative.

For invariant parameters, no position restriction is needed for safety because no cross-argument subtyping is promised.

## 11. Worked nested variance

Suppose:

```text
class C<out T> {
  use(f: (T) -> String)
}
```

Start `T` inside method parameter type:

1. member method parameter position is negative relative to object capability;
2. inside function `(T) -> String`, function parameter flips again;
3. negative × negative = positive.

So `T` may be positive overall in this nested position, depending on exact object/member variance model.

This is why counting "T appears in a parameter" is insufficient. Use polarity composition through the entire type structure.

## 12. Mutable fields force invariance

A readable field `value: T` is producer-like: positive.

A writable field can be modeled as setter operation:

```text
setValue(T) -> Unit
```

which consumes `T`: negative.

Read + write means both polarities, therefore invariance.

This is the formal source of mutable covariance unsoundness.

## 13. Variance and protocols

Protocol requirement:

```text
P<out T> {
  next() -> T
}
```

is naturally covariant if requirements only produce `T`.

But structural conformance adds another layer: the candidate's callable must itself satisfy contravariant parameter/covariant result compatibility.

Do not duplicate variance logic inside protocol conformance. It should ask the canonical callable relation.

## 14. Method override compatibility

Dispatch chooses an implementation according to Phalcom's receiver/selector rules. Override checking asks whether the overriding implementation honors the inherited static contract.

If base declares:

```text
handle(Animal) -> Animal
```

safe override can be:

```text
handle(Object) -> Cat
```

under ordinary function subtyping, not:

```text
handle(Cat) -> Object
```

Some OO languages deliberately use unsound method-parameter bivariance/variance exceptions. Phalcom should adopt such behavior only as an explicit ergonomic tradeoff, not because implementation is easier.

## 15. Bivariance

A bivariant parameter accepts both covariance and contravariance. This is generally unsound for mutable/callable contracts.

If a checker uses bivariance to reduce errors, document:

- exact positions where it applies;
- safety property lost;
- runtime checks, if any;
- why Phalcom benefits enough to justify it.

Never let accidental "either direction passes" become hidden bivariance.

## 16. Union/intersection callable surfaces

For receiver type:

```text
A | B
```

a send is statically safe only if every possible alternative supports a compatible selector/call contract, unless a dynamic boundary is explicitly allowed.

Combining return types may use a join:

```text
return(A.foo) ⊔ return(B.foo)
```

Combining parameter requirements is subtler because callable parameters are contravariant. A member-surface algorithm must derive a domain accepted across all alternatives rather than simply union parameter types.

For intersection `A & B`, capabilities generally accumulate, but same-selector incompatible requirements need a defined meet/overload policy. Do not silently convert intersections into type-directed overload sets.

## 17. `Never` in callable results

If `Never <: T`, then result covariance gives:

```text
D -> Never <: D -> T
```

A callable that never returns normally can satisfy any promised normal result type, assuming its other effects (throw/termination) are permitted by the context.

This is a clean example of why effect policy may matter in addition to result subtyping.

## 18. `Self` in callable types

A fluent method:

```text
clone() -> Self
```

may intend covariant dynamic receiver return, lexical exact class, or another self-type semantics. Variance/override behavior depends on that choice.

Do not replace `Self` with lexical `ClassId` early. Preserve its binder/owner until the relevant receiver/application context is known. See `metatypes-self-and-class-objects.md`.

## 19. Representation and canonicalization

Callable type identity should include semantically relevant call shape, not incidental source details.

Possible canonical key:

```text
CallableKey {
  positional: [ParamContract],
  labeled: ordered/interned label+contract lane,
  rest: Option<RestContract>,
  result: TypeId,
  control/effects: EffectId if part of type equality,
}
```

Whether labels/order/default markers participate in canonical equality must be defined by Phalcom callable semantics.

## 20. Testing obligations

- simple contravariant parameter/covariant result cases;
- negative reversed cases;
- nested polarity composition;
- mutable field invariance;
- labels/selector non-interference;
- optional/default/rest call-set substitution;
- override compatibility;
- union receiver member safety;
- `Never` results;
- `Self` substitution;
- block non-local return/effect distinctions;
- recursive callable types with relation memoization.

Property: when `F <: G`, every statically permitted call to `G` must also be a permitted call to `F`, and `F`'s normal result must satisfy `G`'s promised result.

## 21. Failure modes

- Covariant method parameters because "subclasses are more specific".
- Ignoring labels/rest/defaults in callable compatibility.
- Treating block, method descriptor, and bound method as the same runtime type because their signatures match.
- Encoding non-local return as ordinary result union.
- Validating variance by scanning only immediate syntax.
- Making every method parameter bivariant to avoid override diagnostics.
- Combining union-member callables by naively unioning parameters.

## 22. Competency questions

1. Derive why `(Object -> Cat) <: (Animal -> Animal)`.
2. Why does a writable `T` field make `T` negative as well as positive?
3. How should optional/default parameters be compared in terms of accepted call sets?
4. Why can labels be invariant identity while parameter annotations are contravariant?
5. What happens to polarity through a function parameter nested inside another parameter?
6. Why is a block's non-local return not merely another result type?
