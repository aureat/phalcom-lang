# Equality, Equivalence, Subtyping, Acceptance, and Related Relations

## Purpose

This reference prevents the most expensive class of type-system bugs: implementing several semantic relations as one permissive `compatible(a, b)` predicate.

For Phalcom, relation separation is not academic. Runtime descriptor identity, reflective `equivalentTo`, nominal inheritance, structural protocol conformance, gradual `Dynamic`, assignment/call acceptance, and runtime `isA` can all disagree legitimately.

## 1. Relation taxonomy

For two type expressions `A` and `B`, an implementation may need independent answers:

| Relation | Typical notation | Question |
|---|---|---|
| source/syntax equality | `syntax(A)=syntax(B)` | Were the source annotations structurally/textually equal? |
| runtime descriptor identity | `A === B` (conceptual) | Same reflected descriptor object identity? |
| semantic equivalence | `A ≡ B` | Do they denote the same normalized type expression? |
| nominal identity | `origin(A)=origin(B)` | Same declaration identity? |
| subtype | `A <: B` | Can every `A` value be used under contract `B`? |
| acceptance/assignability | `A ⊣ B` | Does language permit `A` in a context expecting `B`, including defined conversions/dynamic policy? |
| gradual consistency | `A ~ B` | Are the types compatible through a gradual boundary? |
| protocol conformance | `A : P` | Does `A` satisfy protocol requirements? |
| runtime membership | `v isA C` | Does this runtime value satisfy runtime class/protocol test? |
| representation compatibility | `repr(A) ≈ repr(B)` | Can the VM/ABI use same layout/calling convention? |
| isomorphism | `A ≅ B` | Are there total information-preserving conversions both ways? |

Never infer one relation from another unless a rule explicitly connects them.

## 2. Equivalence as an equivalence relation

Semantic equivalence normally requires:

```text
Reflexive:   A ≡ A
Symmetric:   A ≡ B  => B ≡ A
Transitive:  A ≡ B and B ≡ C => A ≡ C
```

Equivalence may include normalization laws deliberately ratified by the type algebra.

Examples often desirable for unions:

```text
A | A        ≡ A
A | Never    ≡ A
A | B        ≡ B | A
(A | B) | C  ≡ A | (B | C)
```

Do not import these laws automatically. If reflection preserves syntactic union grouping/order, semantic equivalence can still ignore it while source metadata remains separate.

### Canonical identity versus equivalence

A canonical interner may make:

```text
A ≡ B  => TypeId(A) == TypeId(B)
```

for canonicalized synthetic types. The reverse mapping is an implementation invariant, not the definition of equivalence.

Bare Phalcom class/protocol type expressions may deliberately use descriptor identity as their equivalence rule. Synthetic types can use structural semantic equivalence.

## 3. Isomorphism is weaker than equality

`A ≅ B` means there exist total functions:

```text
f : A -> B
g : B -> A
```

such that compositions behave as identities under chosen observational equality.

Examples by cardinality/information structure:

```text
A × Unit          ≅ A
Result<A, Never>  ≅ A
Option<Unit>      ≅ two-case nullary sum
```

This does not justify canonical equality. Named types can preserve API meaning, reflection identity, selector surface, diagnostics, and abstraction boundaries.

A normalizer must not erase `Result<A, Never>` to `A` merely because values are isomorphic unless Phalcom explicitly makes that definitional equality.

## 4. Subtyping is a preorder unless quotienting by equivalence

A subtype relation typically has:

```text
A <: A
A <: B and B <: C  => A <: C
```

Antisymmetry is not required on raw syntax. If both `A <: B` and `B <: A`, the system may consider them equivalent, or it may preserve distinct nominal identities with mutually substitutable contracts.

Type-theoretic designs often quotient a preorder by equivalence to obtain a partial order. Implementation should not assume pointer equality from mutual subtyping.

## 5. Declarative subtyping rules

The exact Phalcom rules belong in normative typing specs. Common forms include the following.

### Bottom and top

If `Never` is bottom and `Any` is safe top:

```text
──────────
Never <: T

────────
T <: Any
```

Phalcom's normative core lattice currently specifies these relationships. `Dynamic` is explicitly not an ordinary top type.

### Nominal inheritance

If class `C` inherits `B` under the language's nominal type rule:

```text
C inherits* B
─────────────
C <: B
```

This rule must specify whether it is about instance types, class objects, or both. Do not confuse runtime metaclass inheritance with instance-type subtyping.

### Union introduction/elimination

```text
A <: A | B
B <: A | B

A <: C    B <: C
────────────────
A | B <: C
```

### Intersection introduction/elimination

```text
A & B <: A
A & B <: B

C <: A    C <: B
────────────────
C <: A & B
```

### Function subtyping

```text
B1 <: A1  ...  Bn <: An    R1 <: R2
────────────────────────────────────
(A1,...,An) -> R1 <: (B1,...,Bn) -> R2
```

The direction on parameters is contravariant. Parameter-domain/label compatibility is an additional premise in Phalcom.

### Variance for applied type constructors

For covariant `F`:

```text
A <: B
────────────
F<A> <: F<B>
```

For contravariant `F`:

```text
A <: B
────────────
F<B> <: F<A>
```

For invariant `F`, argument equivalence is typically required.

## 6. Why `Dynamic` consistency must not become subtyping

A gradual consistency relation often has:

```text
Int ~ Dynamic
Dynamic ~ String
```

but not:

```text
Int ~ String
```

If the implementation closes `~` transitively, unrelated types become compatible through `Dynamic`.

This is a classic reason to provide separate APIs:

```text
is_subtype(A, B)
is_equivalent(A, B)
is_consistent(A, B)
accepts(expected, actual, mode)
conforms(candidate, protocol)
```

A single relation with flags is possible but usually obscures invariants and cache semantics.

## 7. Acceptance/assignability is a language relation

A call/assignment context may intentionally accept more than strict subtyping:

```text
actual A
expected B
```

Potential cases:

- `A <: B`;
- literal-specific adaptation;
- numeric promotion if Phalcom defines it;
- explicit gradual `Dynamic` boundary;
- explicit coercion protocol if ratified;
- contextual `None`/`Option` rules if ratified.

If Phalcom defines such a relation, specify it declaratively. Do not implement it by weakening `<:` until examples pass.

## 8. Structural conformance is not automatically subtyping

Suppose protocol:

```text
P = { foo(Int) -> String }
```

and class `C` happens to provide a compatible `foo`.

Possible designs:

1. `C conforms P` and therefore `C <: P`.
2. Conformance and subtyping are separate; contexts explicitly ask for conformance.
3. Explicit declaration is required for subtype relation even if structural conformance can be queried.

Phalcom's typing series currently separates protocol identity from structural conformance and defers full conformance/subtype integration to later documents. Do not guess the final bridge.

## 9. Algorithmic subtyping

A terminating implementation should treat subtyping as an obligation graph rather than uncontrolled recursion.

Conceptual state:

```text
enum RelState { InProgress, Proven, Disproven }
cache: Map<(TypeId, TypeId, RelationMode), RelState>
```

Algorithm skeleton:

```text
subtype(A, B):
  A = normalize(A)
  B = normalize(B)

  if A == B: return true
  if cache[A,B] == Proven: return true
  if cache[A,B] == Disproven: return false
  if cache[A,B] == InProgress:
      return cycle_rule(A,B)

  cache[A,B] = InProgress
  result = apply_subtyping_rules(A,B)
  cache[A,B] = result ? Proven : Disproven
  return result
```

The crucial part is `cycle_rule`. Recursive structural relations often use a coinductive assumption for revisiting a guarded pair. Nominal cycles may be invalid earlier at formation. F-bounds use obligation cycles with different semantics. One global "if in progress, true" is unsound.

## 10. Rule ordering and completeness

Implementation rule order can accidentally change semantics if a rule returns `false` before all applicable declarative alternatives are tried.

For example, with unions:

```text
A <: B | C
```

may require proving `A <: B` **or** `A <: C`, but more expressive systems can admit `A` that is itself a union and require decomposition.

Define algorithmic cases carefully and prove/argue that they cover the declarative relation for the supported fragment.

Avoid ad-hoc fallback:

```text
if complex: return false
```

unless the checker explicitly documents incompleteness and diagnostics distinguish "unsupported/blocked" from "not a subtype".

## 11. Transitivity and memoization traps

Naively searching for an intermediate `X` such that:

```text
A <: X <: B
```

is not a practical implementation of transitivity in an open type universe.

Instead encode subtype constructors/edges and rely on:

- nominal ancestor traversal;
- structural decomposition;
- generic variance;
- union/intersection rules;
- normalized aliases;
- protocol/conformance obligations.

Cache relation results with all semantic dependencies. If class inheritance or protocol surface can change during development, the cache must be generation-sensitive or dependency-invalidated.

## 12. Negative caching needs validity conditions

A cached `false` can be more dangerous than a cached `true` in an open world:

```text
C does not conform to P
```

may become false after a method is added, a module is loaded, or an annotation changes.

A relation cache key/value must encode or depend on:

```text
candidate identity
expected identity
substitution environment
relevant member-surface/inheritance generation
checker relation mode
```

Do not key only by `(TypeId, TypeId)` if the relation depends on mutable semantic surfaces.

## 13. Worked derivation: function subtyping

Assume:

```text
Cat <: Animal <: Object
```

Check whether:

```text
(Object -> Cat) <: (Animal -> Animal)
```

Function rule asks:

Parameter premise (reversed):

```text
Animal <: Object     ✓
```

Result premise:

```text
Cat <: Animal        ✓
```

Therefore:

```text
Object -> Cat <: Animal -> Animal
```

Operational intuition: a context expecting a function that handles any `Animal` can safely receive a function that handles every `Object`; and it accepts a more specific `Cat` result wherever `Animal` was promised.

## 14. Worked failure: mutable covariance

Assume:

```text
Cat <: Animal
List<Cat> <: List<Animal>    # proposed covariance
```

Then:

```text
cats: List<Cat>
animals: List<Animal> = cats
animals.add(Dog)
```

Now `cats` contains `Dog`, violating `List<Cat>`.

Therefore a mutable read/write type parameter appears both positively and negatively and is generally invariant unless writes are restricted or mediated.

## 15. Worked failure: consistency laundering

Suppose implementation defines:

```text
compatible(A,B) = subtype(A,B) or A==Dynamic or B==Dynamic
```

and then memoizes compatibility transitively. It can derive:

```text
Int compatible Dynamic
Dynamic compatible String
=> Int compatible String
```

This turns `Dynamic` into a proof bridge. Correct design keeps consistency non-transitive and applies it only at explicit gradual boundaries.

## 16. Isomorphism and named APIs

`Option<Unit>` and a boolean-like two-case sum have the same cardinality. This says they carry comparable amounts of information; it does **not** say:

- their constructors are the same;
- their pattern labels are the same;
- their methods are the same;
- reflection reports the same type;
- a conversion is implicit.

Use isomorphism for reasoning, not automatic normalization.

## 17. Relation result design

A boolean is often insufficient for tooling/diagnostics. Prefer an internal result like:

```text
RelationResult =
  Proven(proof_path)
  Disproven(reason_tree)
  Blocked(missing_dependency)
  Ambiguous(candidates)
  BudgetExceeded
  Recovery(error_id)
```

The public checker may collapse some of these to diagnostics, but preserving structure enables explanations such as:

```text
List<Int> is not a subtype of List<String>
  because List is invariant in T
  and Int is not equivalent to String
```

## 18. Implementation representation

Keep relation logic over canonical semantic types, not runtime objects directly.

Conceptual Rust boundary:

```text
TypeId -> TypeData
TypeData::Nominal { class: ClassId }
TypeData::Protocol { protocol: ProtocolId }
TypeData::Applied { origin: TypeId, args: Vec<TypeId> }
TypeData::Union { members: InternedSlice<TypeId> }
TypeData::Callable { ... }
TypeData::TypeParam(TypeParamId)
TypeData::SelfType(SelfOwnerId)
TypeData::Special(SpecialType)
```

The exact representation must follow current Phalcom design; this shape illustrates why `ClassId` alone cannot be the entire type domain.

## 19. Testing obligations

For every relation rule add:

- reflexivity where required;
- transitivity probes for `<:` but **not** for consistency;
- symmetry probes for `≡`/`~` where required;
- negative nearby cases;
- union/intersection decomposition;
- recursive cycles;
- generic variance;
- shadowed generic parameters;
- source-equivalent versus canonical-equivalent cases;
- runtime representation-equal but semantically distinct cases;
- cache invalidation after relevant semantic change.

Property tests are especially useful for algebraic laws, but only test laws the language has ratified.

## 20. Competency questions

1. Can two types be mutually subtypes but not pointer-identical? Explain.
2. Why is `A ~ Dynamic ~ B` not a proof that `A ~ B`?
3. What is the difference between protocol conformance and structural subtyping?
4. Give an example of isomorphic but intentionally non-equivalent Phalcom-relevant types.
5. Why must recursive subtype memoization distinguish different cycle policies?
6. What invalidates a cached negative structural-conformance result?
7. Why should assignment acceptance not be implemented by adding exceptions to `is_subtype`?
