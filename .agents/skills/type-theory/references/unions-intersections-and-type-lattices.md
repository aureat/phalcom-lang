# Unions, Intersections, Type Lattices, and Normalization

## Purpose

Use this reference for explicit union/intersection types, type joins/meets, normalization, member surfaces, or any design that talks about "the type lattice".

The central warning is that several orders coexist:

```text
subtyping order
analysis precision order
proof implication order
runtime class hierarchy
```

They are not automatically the same lattice.

## 1. Set-like interpretation

A useful semantic intuition treats a type as a set of values satisfying its contract.

Then:

```text
[[A | B]] = [[A]] ∪ [[B]]
[[A & B]] = [[A]] ∩ [[B]]
A <: B  roughly means [[A]] ⊆ [[B]]
```

This intuition is powerful for algebraic laws, but actual Phalcom types also carry nominal/reflection structure. Do not erase named identities merely because their value sets coincide extensionally.

## 2. Union rules

Declarative introduction:

```text
A <: A | B
B <: A | B
```

Elimination into a common supertype:

```text
A <: C     B <: C
─────────────────
A | B <: C
```

This makes `A | B` a least upper bound if the type algebra guarantees no smaller common supertype under the chosen quotient/equivalence.

## 3. Intersection rules

Elimination:

```text
A & B <: A
A & B <: B
```

Introduction from a common subtype:

```text
C <: A     C <: B
─────────────────
C <: A & B
```

An intersection can describe a capability combination even if no current nominal class explicitly declares both, assuming structural/open-world semantics allow it.

## 4. Join and meet are order-relative

For subtype order:

```text
lub(A,B) = least T such that A <: T and B <: T
glb(A,B) = greatest T such that T <: A and T <: B
```

If explicit unions/intersections are unrestricted and normalized appropriately:

```text
lub(A,B) ≡ A | B
glb(A,B) ≡ A & B
```

But a nominal-only subsystem may use nearest common superclass for LUB and have no GLB except bottom. Always name the domain/order.

## 5. Analysis precision order is different

An abstract value-shape domain may order by information precision:

```text
{Int} ⊑ {Int,String} ⊑ Unknown
```

This resembles set inclusion but has different objects and widening rules from language types.

An LSP cap such as "more than N shapes -> Unknown" can be acceptable for latency. A correctness type checker cannot copy that rule unless `Unknown` has explicit language/checker semantics preserving guarantees.

## 6. Common algebraic laws

Potential union laws:

```text
A | A ≡ A                         idempotence
A | B ≡ B | A                     commutativity
(A | B) | C ≡ A | (B | C)         associativity
A | Never ≡ A                     bottom identity
A | Any ≡ Any                     top absorption
```

Potential intersection laws:

```text
A & A ≡ A
A & B ≡ B & A
(A & B) & C ≡ A & (B & C)
A & Any ≡ A
A & Never ≡ Never
```

Absorption:

```text
A | (A & B) ≡ A
A & (A | B) ≡ A
```

Distributivity:

```text
A & (B | C) ≡ (A & B) | (A & C)
A | (B & C) ≡ (A | B) & (A | C)
```

Do not assume all of these if nominal/refinement/gradual types make normalization expensive or semantics non-set-theoretic. Ratify laws deliberately.

## 7. Normalization pipeline

A practical canonical union constructor can:

1. normalize children;
2. flatten nested same-kind nodes;
3. remove duplicates by semantic equivalence/canonical identity;
4. apply top/bottom identities;
5. remove members subsumed by another **if** subtype checking is stable and affordable;
6. sort by stable semantic key;
7. intern canonical member sequence;
8. preserve source syntax separately if reflection needs it.

Pseudo-code:

```text
make_union(xs):
  work = flatten_union(normalize(x) for x in xs)
  if Any in work: return Any
  remove Never
  dedupe
  optionally remove A when A <: B and B also present
  sort stable
  if empty: return Never
  if one: return only member
  return intern Union(work)
```

Intersection is dual with appropriate top/bottom handling.

## 8. Subsumption elimination cost

Removing a union member `A` when another member `B` satisfies `A <: B` improves canonicality:

```text
Int | Number ≡ Number
```

But it invokes subtyping during normalization. Risks:

- recursive relation cycles;
- expensive structural conformance;
- open-world invalidation if subtyping depends on mutable surfaces;
- normalizer/relation mutual recursion.

Possible policy:

- cheap canonical structural laws always;
- semantic subsumption in a separate normalization/query layer with memoization;
- avoid relying on open-world conformance to define permanent type identity.

The exact Phalcom policy must preserve stable reflective equivalence.

## 9. Canonical ordering

If `A | B ≡ B | A`, representation needs deterministic ordering for hashing/interning.

Sort by stable semantic key such as:

```text
(TypeKindTag, stable origin identity, canonical child IDs)
```

Do not sort by allocation address or source position. Those make equality unstable across runs/edits.

## 10. Member access on unions

For:

```text
x : A | B
```

statically guaranteed member `m` must be supported by every possible alternative under compatible call contracts.

Conceptually:

```text
surface(A | B) = compatible_common_surface(surface(A), surface(B))
```

For zero-argument getter result:

```text
A.m : R1
B.m : R2
```

result may be:

```text
R1 | R2
```

if both members are callable/accessible in the same way.

For method parameters, common compatibility requires contravariant reasoning; do not simply union parameter types.

## 11. Member access on intersections

For:

```text
x : A & B
```

capabilities generally combine:

```text
surface(A & B) ≈ union of requirements/capabilities
```

If both define same selector with incompatible contracts, options include:

- intersection considered uninhabitable;
- compute a callable meet if one exists;
- require explicit conflict resolution;
- treat as unsupported.

Do not silently create overload sets keyed by argument types; that would change dispatch semantics.

## 12. Empty intersections and inhabitation

`A & B` can be well formed yet have no known/common runtime inhabitants.

Examples:

- two sealed disjoint nominal classes;
- conflicting literal refinements;
- structurally incompatible requirements.

If the checker can prove no inhabitant exists, it may normalize to `Never`. But "no current class observed" is not proof in an open world.

Distinguish:

```text
provably empty
currently no known inhabitant
open-world unknown
```

## 13. Union narrowing

Given:

```text
x : Cat | Dog
```

and a trusted runtime test proving `x is Cat`, the true branch can refine:

```text
x : Cat
```

The false branch may refine to `Dog` only if the original union is closed enough and the predicate is exact.

General rule resembles set difference:

```text
true  = T ∩ P
false = T \ P
```

but not every type language has a representable complement/difference type. Flow refinements can store propositions separately when canonical type algebra cannot express them exactly.

## 14. `Never` as unreachable normal result

Normative Phalcom core rules include:

```text
T | Never = T
T & Never = Never
```

This is operationally useful:

```text
if condition then 42 else fail()
```

where `fail() : Never` can synthesize/join to `Int` for normal completion.

Do not reinterpret an analyzer failure as `Never`; bottom means no value/path, not unknown value.

## 15. `Any` versus `Dynamic`

If `Any` is safe top:

```text
T <: Any
```

then:

```text
T | Any ≡ Any
T & Any ≡ T
```

`Dynamic` is not ordinary top in Phalcom's normative core design. Do not apply these algebraic laws to `Dynamic` unless gradual-typing semantics define them.

A gradual type system may have a separate precision lattice where `Dynamic` is least/most precise depending notation. Keep that order named separately.

## 16. Distribution can explode

Normalizing:

```text
(A | B) & (C | D) & (E | F)
```

by full distribution produces up to eight combinations; larger expressions grow exponentially.

Canonicalization should not eagerly distribute without a strong semantic/algorithmic reason.

Alternatives:

- keep DAG syntax with local normalization;
- BDD-like/set-theoretic representation for a deliberately advanced type algebra;
- relation procedures that reason structurally without global DNF/CNF.

Do not choose an advanced set-theoretic representation before Phalcom needs it.

## 17. Literal and singleton types

If Phalcom later introduces literal types, unions can express finite sets:

```text
#red | #green | #blue
```

This improves exhaustiveness and finite generic constraints, but increases normalization and widening concerns.

A finite generic constraint set is still semantically distinct from a union type unless the specification deliberately equates them.

## 18. Recursive unions/intersections

Recursive aliases/types can contain unions:

```text
Json = Null | Bool | Number | String | List<Json> | Map<String,Json>
```

Normalization must be graph-aware. Expanding aliases naively can be nonterminating.

Use canonical recursive nodes, memoized relation queries, and guarded alias policy.

## 19. Caching and invalidation

Canonical union structure based only on child `TypeId`s can be stable. A normalized elimination step based on open structural conformance may not be.

Separate:

```text
stable canonical syntax/type expression
context-sensitive relation simplification
```

unless Phalcom's semantic universe guarantees the relation is immutable for the lifetime of those IDs.

## 20. Testing obligations

Test ratified algebraic laws as properties:

```text
normalize(A | A) == normalize(A)
normalize(A | Never) == normalize(A)
normalize(A | B) == normalize(B | A)
```

Also test:

- nested flattening;
- deterministic ordering;
- duplicate removal;
- recursive types;
- union member lookup;
- conflicting intersection members;
- narrowing true/false branches;
- open-world non-emptiness uncertainty;
- no exponential accidental normalization on stress cases;
- checker behavior differs correctly from LSP widening.

## 21. Failure modes

- Calling every join a union without defining the subtype lattice.
- Using LSP `Unknown` as language top.
- Eager distributive normalization causing exponential blowup.
- Normalizing open-world empty intersections to `Never` from current observations.
- Treating `Dynamic` with `Any` algebraic laws.
- Making intersection member conflicts into hidden type overloads.
- Sorting union members by source location or pointer address.

## 22. Competency questions

1. Under what assumptions is `A | B` the LUB of `A` and `B`?
2. Why can subtype join and abstract-analysis join be different operations?
3. Why might subsumption elimination be unsafe as a permanent canonicalization rule in an open world?
4. How do you compute the member surface of a union receiver?
5. Why is a proven-empty intersection different from "no current implementor"?
6. What is the complexity danger of full distributivity?
