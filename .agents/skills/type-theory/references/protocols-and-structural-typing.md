# Protocols, Structural Typing, and Conformance

## Purpose

This reference covers structural capability relations, protocol requirement matching, recursive conformance, variance, generics, and open-world caching.

Phalcom's typing design already establishes an important doctrine: protocol descriptors have their own identity, while structural conformance is a separate relation and ordinary selector dispatch remains unchanged. Full conformance algorithms are assigned to later typing work, so this reference teaches the theory without claiming current implementation.

## 1. Structural capability model

Protocol:

```text
Drawable = {
  draw() -> Unit
  bounds() -> Rect
}
```

A candidate satisfies `Drawable` when it exposes every required observable operation with compatible contracts.

A declarative judgment:

```text
Γ ⊢ C conforms P
```

can be defined requirement-wise.

## 2. Protocol identity versus conformance

Two axes:

```text
Protocol identity: declaration-object identity of P
Conformance: relation between candidate type C and requirements of P
```

Structural conformance does not imply protocol descriptors themselves are structurally equal.

This separation supports:

- reflection (`P` remains a first-class protocol object);
- documentation/source identity;
- protocol-specific attributes;
- multiple protocols with same current surface but different domain meaning.

## 3. Basic conformance rule

Let `Req(P)` be protocol requirements. A schematic rule:

```text
for each r in Req(P):
  exists member m in Surface(C)
  selector(m) = selector(r)
  compatible_member(m, r)
────────────────────────────
C conforms P
```

`selector` comparison occurs before type compatibility. Types do not become dispatch keys.

## 4. Member compatibility

For callable requirement:

```text
required: foo(A) -> R
candidate: foo(B) -> S
```

safe callable compatibility normally requires:

```text
A <: B        # candidate accepts at least required inputs
S <: R        # candidate result satisfies required promise
```

plus compatible labels/arity/default/rest/effects/access rules.

Do not check parameter types covariantly.

## 5. Width and depth

Structural **width** allows extra candidate members:

```text
{x, y, z} conforms {x, y}
```

Structural **depth** compares matching member contracts recursively.

Writable fields require special care. Treating fields as pure covariant properties is unsafe if clients can write. Prefer translating capability into getter/setter operations with their correct variance.

## 6. Instance-side versus class-side requirements

Phalcom protocol design supports both instance-side and class-side requirements.

They are different surfaces:

```text
instance candidate: instances of C must answer selectors
class-side candidate: class object C must answer selectors
```

Do not satisfy a class-side requirement by finding an instance method or vice versa.

Type theory must cooperate with metatype/class-object typing. See `metatypes-self-and-class-objects.md`.

## 7. Structural conformance and subtyping

Possible bridge:

```text
C conforms P => C <: P
```

is common but not logically forced. A language might require explicit declarations/coherence for subtype use while still exposing structural conformance queries.

Until Phalcom's later conformance/subtyping documents ratify the bridge, keep APIs separate:

```text
conforms(C,P)
is_subtype(C,P)
```

and connect them only through explicit rules.

## 8. Generic protocols

Example:

```text
Producer<T> = { next() -> T }
```

Candidate conformance to `Producer<Animal>` may depend on generic variance and applied-member substitution.

Pipeline:

1. normalize protocol application;
2. substitute protocol parameters into requirement views;
3. obtain candidate member view in its application context;
4. compare selectors/call shape;
5. compare callable types;
6. solve nested conformance/subtyping obligations.

Do not compare raw unsubstituted annotations.

## 9. Associated types / existential concerns

If protocols later expose associated type members:

```text
Iterator {
  type Element
  next() -> Option<Element>
}
```

existential use of `Iterator` hides a concrete `Element` unless constrained.

This introduces:

- type-member projection;
- associated-type equality obligations;
- existential/skolemization;
- potentially higher-kinded members.

Do not simulate associated types with dynamic metadata before designing these semantics.

## 10. Recursive protocols

Example:

```text
NodeLike = { next() -> Option<NodeLike> }
```

Candidate `Node` may recursively refer to itself.

Conformance algorithm needs memoized obligations:

```text
(C, P, substitution, side)
```

with states:

```text
InProgress
Proven
Disproven
Blocked
```

A guarded revisit can be treated coinductively. An unguarded arbitrary cycle cannot.

## 11. Coinductive conformance example

Protocol:

```text
P { next() -> Option<P> }
```

Candidate:

```text
C { next() -> Option<C> }
```

To prove `C conforms P`:

1. compare `next` selectors;
2. compare results `Option<C> <: Option<P>` under covariance;
3. reduce to `C <: P` / `C conforms P` according to bridge rule;
4. encounter original obligation already `InProgress` under productive constructor `Option`;
5. guarded coinductive rule closes cycle.

This cannot be implemented with naïve recursion.

## 12. Explicit conformance declarations

Even in a structural system, explicit declarations can provide:

- programmer intent;
- early/local diagnostics;
- coherence for extensions;
- optimization/caching hints;
- versioning guarantees;
- documentation.

But an explicit declaration should not make an actually incompatible candidate sound. It is an assertion/obligation, not magical method synthesis unless the language explicitly says so.

## 13. Open-world mutation

If classes can gain methods through reflective APIs, imported module augmentation, or later package versions, structural conformance can change.

A cached result must depend on:

```text
candidate semantic identity
protocol identity
instance/class side
substitution environment
candidate member-surface generation
protocol requirement generation
inheritance/module visibility generation
relation-mode version
```

A negative cache without invalidation is especially dangerous.

## 14. Visibility and access

A member can exist but not be usable by protocol clients.

Conformance rule must decide:

- public/private/protected semantics;
- module/package visibility;
- class-side visibility;
- reflective-only members;
- generated/native members.

Do not let internal/private methods satisfy a public capability accidentally.

## 15. Effects and contracts

If protocol requires:

```text
read() -> String ! pure
```

or requires postconditions/effects in future, candidate compatibility must include them according to effect/contract subtyping.

Result/parameter type compatibility alone is insufficient once effects are part of the callable contract.

Phalcom need not add effect types immediately; keep extension point explicit.

## 16. Protocol intersections

`P & Q` can mean a value satisfies both requirement sets.

For distinct selectors, combine requirements.

For same selector:

```text
P.foo : A -> R1
Q.foo : B -> R2
```

need a contract satisfying both. Possible meet may require:

- domain accepting calls required by both;
- result satisfying both result promises;
- compatible labels/effects.

If no such callable contract exists, intersection may be uninhabitable or require explicit conflict policy.

Do not create runtime overloading by type to resolve the conflict.

## 17. Diagnostics

Conformance diagnostics should be requirement-oriented and causal:

```text
Repository does not conform to KeyValueStore<String,Value>
  requirement: get(key: String) -> Option<Value>
  candidate:   get(key: String) -> Value
  mismatch: candidate result Value is not a subtype of Option<Value>
```

Preserve spans for:

- protocol declaration/requirement;
- candidate member;
- substituted type arguments;
- relation failure.

## 18. Algorithm skeleton

```text
conform(C,P,env,side):
  key = canonical obligation key
  consult cycle/cache state
  mark InProgress

  for requirement r in substituted_requirements(P,env,side):
    candidates = lookup_member_surface(C, selector(r), side)
    if none: fail MissingMember
    if not exactly/validly resolvable: fail AmbiguousSurface
    if !callable_compatible(candidate,r): fail IncompatibleMember

  mark Proven
```

Use semantic member lookup/surface APIs shared with compiler/LSP; do not build another string-based member walker inside conformance.

## 19. Testing obligations

- missing member;
- extra member allowed;
- wrong selector label;
- contravariant parameter/covariant result;
- private member cannot satisfy public requirement if policy says so;
- class-side versus instance-side mismatch;
- generic substitution;
- recursive protocol;
- protocol intersection conflict;
- open-world member addition invalidates negative cache;
- incremental result equals clean analysis;
- explicit declaration with incompatible surface still fails.

## 20. Failure modes

- Conformance installs synthetic methods.
- Types become selector lookup keys.
- Writable fields treated covariantly.
- Recursive protocol uses recursion-depth cutoff.
- Candidate/protocol surface changes do not invalidate cache.
- Explicit `implements` bypasses actual compatibility.
- Source spelling used to match generic parameters.

## 21. Competency questions

1. Why can protocol identity be nominal while conformance is structural?
2. Derive parameter/result directions for candidate method compatibility.
3. Why must class-side requirements use a different candidate surface?
4. What makes a recursive conformance cycle safely coinductive?
5. Which dependencies must a conformance cache record?
6. Why should explicit conformance not synthesize missing methods by default?
