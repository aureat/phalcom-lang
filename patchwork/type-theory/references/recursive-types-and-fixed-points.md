# Recursive Types, Fixed Points, Coinduction, and `Self`

## Purpose

Recursive typing appears whenever a type refers to itself directly or through a cycle: recursive ADTs, structural protocols, aliases, F-bounds, `Self`, mutually recursive classes, and recursive callable summaries.

The core engineering rule is: **do not use recursion depth as semantics**. Identify whether the problem is recursive type formation, recursive relation checking, or recursive inference; each needs a different termination argument.

## 1. Fixed-point notation

A recursive type can be written:

```text
μX. F(X)
```

meaning a fixed point of type constructor `F`.

Example list shape:

```text
List<A> = μX. None | Cons(A, X)
```

Unfolding once:

```text
μX.F(X)  ≈  F(μX.F(X))
```

Whether `≈` means definitional equality or explicit fold/unfold is a design choice.

## 2. Equi-recursive versus iso-recursive

### Equi-recursive

```text
μX.F(X) ≡ F(μX.F(X))
```

The recursive type and its unfolding are definitionally equivalent. Relation algorithms need cycle-aware comparison.

### Iso-recursive

The two are isomorphic, and the calculus has conceptual `fold`/`unfold` operations:

```text
fold   : F(μX.F(X)) -> μX.F(X)
unfold : μX.F(X) -> F(μX.F(X))
```

Nominal OO types often avoid exposing either model directly: class/ADT declaration identity acts as a recursion boundary.

Phalcom should only expose explicit `μ`-style types if a real language use case requires them.

## 3. Nominal recursion

```text
class Node {
  next: Option<Node>
}
```

is easy to represent because `Node` refers to a stable nominal declaration ID. The type graph contains a back-edge rather than infinite syntax.

Do not expand a nominal class definition when comparing `Node` to itself; identity can terminate the relation immediately.

## 4. Recursive structural types

Structural protocols can be recursive:

```text
protocol NodeLike {
  next() -> Option<NodeLike>
}
```

Two different recursive structural descriptors may be equivalent/conforming coinductively even though identities differ.

Naive recursive comparison loops:

```text
A.next -> A
B.next -> B
compare A,B
  compare A.next,B.next
    compare A,B
      ...
```

## 5. Coinductive relation checking

For regular recursive structures, relation checking can assume a pair provisionally while checking its children.

Conceptual algorithm:

```text
relate(A,B):
  if (A,B) in Proven: return true
  if (A,B) in Disproven: return false
  if (A,B) in InProgress:
      return true   # ONLY when relation's guarded coinductive rule permits

  mark (A,B) InProgress
  ok = check_outer_constructor_and_children(A,B)
  mark Proven or Disproven
  return ok
```

The in-progress success is justified when revisiting the same obligation occurs under a productive/guarded constructor relationship. Do not apply it to arbitrary cycles such as unresolved type aliases or inconsistent F-bounds.

## 6. Guardedness

A recursive definition is guarded when recursion occurs beneath a constructor that contributes observable structure.

Good conceptual example:

```text
Stream<T> = Cons(T, Stream<T>)
```

Problematic transparent alias:

```text
A = A
```

or mutual aliases with no constructor progress:

```text
A = B
B = A
```

A formation checker can reject unguarded aliases before subtype/equality algorithms see them.

## 7. Contractiveness

Some recursive type systems require `F` to be contractive: recursive variable occurrences must not appear in positions that prevent a well-defined unique fixed point.

The exact condition depends on the type algebra, especially with negation, intersections, and contravariance.

For first Phalcom recursive constructs, prefer nominal/ADT/protocol boundaries with simple guardedness over exposing unrestricted equi-recursive aliases.

## 8. Recursive subtyping

Example:

```text
A = { next: A, value: Int }
B = { next: B }
```

Width/depth structural subtyping intuitively gives:

```text
A <: B
```

because:

1. `A` has required member `next`;
2. compare `A.next : A` with `B.next : B`;
3. recursive obligation `A <: B` is already in progress under a covariant result position;
4. coinductive assumption closes the cycle.

Memoized pairs make this finite.

## 9. Negative/contravariant cycles

Recursive types through function parameters are more subtle:

```text
A = (A) -> Int
B = (B) -> Int
```

Subtyping flips direction through parameters, producing obligation cycles whose parity/polarity matters.

A simplistic "in-progress means true" can prove invalid relations. Track relation direction/polarity or use a well-founded/coinductive algorithm appropriate to the recursive subtype calculus chosen.

Do not implement advanced recursive structural subtyping before defining its exact formal fragment.

## 10. Recursive aliases and normalization

Transparent alias:

```text
type Json = Bool | Number | List<Json>
```

can be useful if represented as a named recursive node.

Normalizer must not fully expand aliases:

```text
normalize(Json)
 -> Bool | Number | List<Json>
 -> Bool | Number | List<Bool | Number | List<Json>>
 -> ...
```

Use alias IDs/fixed-point nodes and memoized unfolding only as needed by relations.

## 11. `Self` is binder-like recursion over receiver identity

`Self` often means a type tied to a receiver/declaration context rather than a globally named class.

Possible semantics:

1. **lexical exact self**: exactly the declaring class type;
2. **dynamic self**: the dynamic subtype of receiver;
3. **F-bounded self**: implicit type variable `S <: C<S>`-like model;
4. **receiver-dependent type**: path/identity tied to current receiver.

These differ for:

- inherited fluent methods;
- class-side constructors;
- override checking;
- protocol requirements;
- applied generic classes;
- metaclasses.

Ratify one before implementation. Do not erase `Self` to lexical `ClassId` during parsing.

See `metatypes-self-and-class-objects.md` for object-model detail.

## 12. F-bounds are recursive obligations

```text
T <: Comparable<T>
```

When validating `User`, solve:

```text
User <: Comparable<User>
```

This may recursively ask about conformance/member types involving `User`. Use obligation states:

```text
Pending
InProgress
Proven
Disproven
Blocked
```

and a guarded/coherence policy. F-bounds do not require creating `μ` types.

## 13. Recursive inference

Suppose:

```text
foo() { return bar() }
bar() { return foo() }
```

Without annotations, result inference creates an SCC.

A fixed-point solver may define:

```text
S0(foo) = seed
S0(bar) = seed
S_{n+1}(foo) = infer(body_foo, S_n)
S_{n+1}(bar) = infer(body_bar, S_n)
```

Need:

- a finite-height domain or widening;
- monotone transfer;
- termination policy;
- meaning of seed (`Never`? unknown? no-return-yet? these are not interchangeable);
- diagnostic policy if no useful fixed point emerges.

A correctness checker can instead require explicit annotations for recursive SCCs. That is often a good first design.

## 14. Fixed points in abstract analysis versus type recursion

Both use fixed-point language but are distinct:

- recursive type `μX.F(X)` is a type-level equation;
- dataflow fixed point solves program-state equations over CFG/call graph;
- recursive inference may solve summary equations over a type-information domain.

Do not reuse `Never` as the initial abstract-analysis bottom unless the abstract domain semantics justify it.

## 15. Worklist SCC strategy

For mutually recursive declarations:

1. build dependency graph using semantic IDs;
2. compute SCCs;
3. acyclic SCCs solve once in topological order;
4. cyclic SCCs use explicit annotation boundary or iterative solver;
5. cache result keyed by participating declaration generations;
6. diagnostics name the cycle.

Good diagnostic:

```text
cannot infer result types for recursive cycle
  A.foo -> B.bar -> A.foo
add an explicit result annotation to break the cycle
```

Bad diagnostic:

```text
recursion limit exceeded
```

## 16. Recursive type interning

Hash-consing cyclic graphs requires care because hash cannot depend on infinite unfolding.

Options:

- nominal recursive nodes hash by declaration identity;
- de Bruijn/indexed `μ` syntax canonicalized structurally;
- strongly connected type graph assigned canonical node IDs after cycle construction;
- guarded recursive aliases remain nominally identified.

Prefer simple nominal IDs unless structural recursive equality is a required feature.

## 17. Open-world invalidation

A recursive structural conformance proof can depend on multiple member surfaces. If any changes, the whole obligation SCC may need recomputation.

Record dependency edges rather than caching `true` forever.

## 18. Testing obligations

- self-recursive nominal type;
- mutually recursive nominal types;
- recursive structural equality/subtyping;
- negative cycle that should fail;
- guarded versus unguarded alias;
- F-bound cycle;
- `Self` under inheritance/application;
- recursive inference SCC with explicit-annotation policy;
- deterministic cycle diagnostics;
- incremental edit within recursive SCC equals clean analysis.

Property: relation algorithms terminate on every finite regular type graph admitted by formation rules.

## 19. Failure modes

- Recursion depth 32 as subtype semantics.
- `InProgress => true` for every relation cycle.
- Fully unfolding recursive aliases in normalization.
- Treating recursive inference seed `Unknown` as language `Dynamic`.
- Erasing `Self` to lexical class too early.
- Caching recursive structural conformance without dependencies.

## 20. Competency questions

1. What is the difference between equi-recursive and iso-recursive types?
2. Why does pair memoization terminate comparison of regular recursive structures?
3. When is an in-progress coinductive assumption justified?
4. Why are F-bounds recursive obligations rather than necessarily recursive types?
5. What information is required to iterate recursive inference soundly?
6. Why is `Self` a binder-like semantic entity rather than a string alias?
