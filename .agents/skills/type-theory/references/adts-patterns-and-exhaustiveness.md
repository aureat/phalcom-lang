# Algebraic Data Types, Patterns, and Exhaustiveness

## Purpose

This reference covers closed sum types, variants, pattern typing/refinement, exhaustiveness, usefulness/redundancy checking, guards, generic ADTs, and open-world limits.

The central distinction is:

```text
closed constructor universe -> exhaustiveness can be proved
open inheritance universe    -> today's subclasses are not exhaustive evidence
```

## 1. ADT as a closed tagged sum

Example:

```text
Expr = Literal(Int)
     | Add(Expr, Expr)
     | Name(String)
```

A value is one constructor tag plus its payload product.

Type-theoretically:

```text
Expr ≅ Int + (Expr × Expr) + String
```

but constructor names/tags remain semantically observable; this is structural intuition, not necessarily definitional equality.

## 2. Closedness is a semantic promise

Exhaustiveness requires a finite known constructor set at the checking boundary.

Closed mechanisms include:

- sealed ADT declaration;
- enum/variant declaration with no external extension;
- sealed/final class hierarchy if language guarantees all subclasses are known.

A normal open class hierarchy is not closed merely because repository search finds three subclasses.

Versioning matters: adding a variant to a public sealed ADT can break downstream exhaustive matches, which may be desirable source compatibility behavior.

## 3. Pattern typing judgment

A useful judgment:

```text
Γ ⊢ p : T ⇒ Γ_p ; Φ_p
```

means pattern `p` can match values of `T`, introducing bindings `Γ_p` and propositions/refinements `Φ_p` on success.

Example:

```text
Some(x)
```

against `Option<String>` yields:

```text
x : String
fact: scrutinee has variant Some
```

Bindings must have semantic IDs and scopes like ordinary declarations.

## 4. Constructor pattern rule

If constructor:

```text
K : A1 × ... × An -> T
```

and subpatterns check against each payload type:

```text
Γ ⊢ p1 : A1 ... Γ ⊢ pn : An
─────────────────────────────
Γ ⊢ K(p1,...,pn) : T
```

For generic ADTs, substitute type arguments first.

Example:

```text
Option<T>.Some(value:T)
```

under `Option<Int>` gives payload `Int`.

## 5. Pattern refinement

Successful patterns can refine:

- constructor/tag;
- runtime class if pattern semantics guarantee it;
- literal equality;
- tuple/record shape;
- payload types;
- generic equality constraints in advanced GADT designs.

Store branch-local facts. Do not mutate the declaration's canonical type globally.

## 6. Exhaustiveness as uncovered space

A match is exhaustive when no value in scrutinee type is unmatched.

For finite constructor sums, this can be decided structurally.

Naïve enumeration of all values is impossible because payloads may be infinite (`Int`, recursive trees). Instead reason by constructor patterns.

## 7. Pattern matrix model

Maranget-style usefulness algorithms represent prior arms as rows and scrutinee components as columns.

Example:

```text
match x {
  Some(0) => ...
  Some(_) => ...
  None    => ...
}
```

Matrix after first two arms covers the `Some` constructor payload space; `None` covers other top-level constructor.

The algorithm recursively specializes matrix by constructors.

## 8. Usefulness algorithm intuition

A new pattern vector `q` is useful relative to matrix `P` if there exists a value matched by `q` not matched by any row in `P`.

```text
useful(P, q, types) -> bool / witness
```

For constructor `K`, specialize rows whose head can match `K`, expand wildcard rows into `K`'s arity, and recurse on payload columns.

For wildcard head, inspect which constructors are already covered; if constructor universe is complete, recurse across missing/specialized constructors.

This same machinery can produce:

- unreachable/redundant pattern diagnostics;
- non-exhaustive witness patterns.

## 9. Pseudo-code sketch

```text
useful(matrix P, vector q, type T::Ts):
  if no columns:
    return P has no empty row

  if head(q) = Constructor K(args):
    Pk = specialize(P, K)
    return useful(Pk, args ++ tail(q), payload_types(K) ++ Ts)

  if head(q) = Wildcard:
    constructors = constructors_of(T)
    heads = constructors_mentioned_in_first_column(P)

    if constructors is closed and heads covers constructors:
      for K in constructors:
        if useful(specialize(P,K), wildcards(arity(K)) ++ tail(q), ...):
          return true
      return false
    else:
      return useful(default_matrix(P), tail(q), Ts)
```

Real implementations need literals, tuples, records, or-patterns, rest patterns, guards, and open types.

## 10. Exhaustiveness result must be multi-state

Use something like:

```text
Exhaustive
NonExhaustive(witnesses)
Unknown(reason)
```

`Unknown` is required when:

- scrutinee type is dynamic/open;
- pattern feature unsupported;
- analysis dependency missing;
- budget exhausted.

Never report `Exhaustive` just because no witness was found under a bounded search.

## 11. Guards

Arm:

```text
Some(x) if x > 0 => ...
```

structurally covers only the subset satisfying guard.

Unless the guard is statically proven always true for that structural space, do not use it as complete constructor coverage.

A conservative checker can:

- use guardless structural pattern for reachability only when guard absent/true;
- treat guarded arm as potentially matching but not exhaustive;
- use prover facts later to strengthen this.

## 12. Or-patterns

Pattern:

```text
A(x) | B(x)
```

requires compatible bindings across alternatives:

- same binding set;
- compatible/equivalent bound types according to rule;
- same mutability/capture semantics.

Normalize or-patterns into matrix alternatives for usefulness while retaining source provenance for diagnostics.

## 13. Literal/range patterns

Finite literals can be reasoned about exactly:

```text
Bool: true | false
```

Integers/ranges require interval/set reasoning to prove coverage. Do not enumerate large domains.

Pattern engine can have specialized constructor spaces:

```text
FiniteTags
Intervals
OpenNominal
TupleProduct
ADTConstructors
```

Keep domain algorithms separate but composable.

## 14. Tuple and record patterns

Tuple pattern is product specialization: every component must match.

Records with unordered labels need canonical label alignment before matrix comparison.

Rest patterns create variable-width shapes. Exhaustiveness needs exact semantics for minimum/maximum length and labeled lane behavior.

This is where parser/AST pattern representation must preserve enough structure for checker algorithm.

## 15. Generic ADTs

```text
Result<T,E> = Ok(T) | Err(E)
```

Applying `Result<Int,Never>` may make `Err` uninhabited if `Err` requires a payload `E` and `Never` has no values.

A sufficiently precise exhaustiveness checker can eliminate impossible constructors after substitution:

```text
Result<Int,Never>
```

may require only `Ok(_)` if the language defines no way to construct `Err(Never)`.

But named variant existence in reflection can remain even when a particular application is uninhabited.

## 16. GADT complexity cliff

A GADT lets constructor result refine type parameters:

```text
Expr<T>
IntLit : Int -> Expr<Int>
BoolLit: Bool -> Expr<Bool>
```

Matching `IntLit` on `Expr<T>` yields equality `T ≡ Int` in that branch.

This requires:

- type equality constraints from patterns;
- skolem/existential handling;
- branch refinement;
- more sophisticated exhaustiveness modulo constraints.

Do not accidentally introduce GADT semantics by allowing arbitrary per-variant result applications without a design for these constraints.

## 17. Open inheritance patterns

For:

```text
match animal {
  Cat(...) => ...
  Dog(...) => ...
}
```

if `Animal` can have future/external subclasses, this is not globally exhaustive.

Possible result:

```text
Unknown(OpenWorld)
```

or require wildcard/default arm.

A `sealed` language construct can make hierarchy closed and enable exact exhaustiveness.

## 18. `Option` and `Result`

These canonical sums should get first-class exhaustiveness support.

For `Option<T>`:

```text
Some(_)
None
```

For `Result<T,E>`:

```text
Ok(_)
Err(_)
```

Avoid treating `None` as nullable pointer; its variant semantics provide clean coverage and narrowing.

## 19. Redundancy diagnostics

Example:

```text
match x {
  Some(_) => ...
  Some(0) => ...   # unreachable
  None => ...
}
```

Usefulness says second arm matches no new value. Diagnostic should point to earlier covering arm if possible.

Guards complicate redundancy: a guarded prior arm may not make later pattern unreachable.

## 20. Witness generation

For non-exhaustive match, produce pattern witness:

```text
missing case: None
```

or nested:

```text
missing case: Some(false)
```

Witnesses are more actionable than "match is non-exhaustive".

Keep witness construction deterministic for snapshot tests.

## 21. Complexity controls

Pattern matrices can blow up with nested or-patterns/large constructor products. Use:

- memoization by matrix/type state;
- compact constructor sets;
- interval reasoning;
- budget result `Unknown(BudgetExceeded)` rather than claiming exhaustive;
- avoid eagerly expanding open domains.

Checker completeness policy should be explicit.

## 22. Testing obligations

- every constructor covered;
- missing top-level variant;
- nested payload patterns;
- redundant arm;
- guarded arm not exhaustive;
- or-pattern binding consistency;
- generic ADT impossible variant;
- open class hierarchy not treated closed;
- sealed hierarchy exhaustive;
- tuple/record/rest patterns;
- malformed/incomplete source returns recovery/unknown without poisoning unrelated facts;
- deterministic witnesses.

## 23. Failure modes

- Enumerating runtime values.
- Treating currently known subclasses as a closed universe.
- Guards counted as full structural coverage.
- "No witness found" equated with proof.
- GADT-like variant result refinements added without equality-constraint machinery.
- Generic impossible variants ignored or overclaimed.

## 24. Competency questions

1. Why can exhaustiveness be decided without enumerating every payload value?
2. What does pattern usefulness mean?
3. Why does a guard usually not contribute to guaranteed exhaustiveness?
4. How can `Never` type arguments eliminate a generic variant?
5. Why is an open class hierarchy fundamentally different from a sealed ADT?
6. What extra machinery do GADTs require?
