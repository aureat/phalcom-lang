# Contracts and Axiomatic Semantics

Phalcom's `@requires`, `@ensures`, and `@invariant` direction naturally connects to Hoare-style reasoning. Contracts describe obligations over semantic states; they do not replace dynamic semantics that gives calls, mutation, exceptions, and fibers meaning.

## 1. Hoare triples

```text
{P} c {Q}
```

means: if `P` holds and `c` terminates normally under modeled semantics, `Q` holds afterward. This is ordinarily partial correctness. Total correctness additionally proves termination.

## 2. State predicates

Predicates can refer to:

```text
self and arguments
locals/fields/globals
ghost/model state if specified
old(pre-state) values
result
```

Keep source-visible values distinct from prover-only ghost state.

## 3. Method contract

```text
requires P(self, args, σ)
ensures  Q(self, args, result, σ, σ')
```

Exceptional postconditions may be separate:

```text
throws E when R(...)
```

or encoded in an outcome relation.

## 4. Assignment rule

Classic assignment:

```text
{Q[x := e]} x := e {Q}
```

With heap fields and aliasing, substitution becomes more complex; weakest-precondition or separation-logic reasoning may be preferable.

## 5. Sequence

```text
{P} c1 {R}    {R} c2 {Q}
--------------------------
{P} c1;c2 {Q}
```

Abrupt outcomes require extra rules; normal-postcondition-only calculus cannot reason fully about throws/non-local returns.

## 6. Conditionals

```text
{P ∧ b} c1 {Q}
{P ∧ ¬b} c2 {Q}
-----------------
{P} if b then c1 else c2 {Q}
```

If Phalcom surface conditionals are message/block based, prover may lower to core conditional only after semantic equivalence is established.

## 7. Loops

Loop invariant `I` must satisfy:

```text
P => I
{I ∧ condition} body {I}
I ∧ ¬condition => Q
```

Termination additionally needs a well-founded variant that decreases. Finite unrolling is bug finding, not proof of arbitrary iterations.

## 8. Object invariants

An object/class invariant needs a policy for when it must hold:

- after construction;
- entry/exit of public methods;
- before/after callbacks;
- at visible yield points;
- perhaps not during internal mutation sequence.

Yielding while invariant is temporarily broken is dangerous if another fiber can observe the object.

## 9. Behavioral subtyping

Substitutability usually implies:

```text
subclass method must not require more from caller
subclass method must guarantee at least super contract
```

Exact Phalcom rules need separate type/contract design, but this is semantic motivation behind precondition weakening and postcondition strengthening.

## 10. Frame conditions

A postcondition is difficult to use without knowing what may change. Frame conditions/effect summaries can say:

```text
modifies self._balance
preserves other state in specified footprint
```

Dynamic aliasing makes footprint specification significant.

## 11. Call rule

A method call can use callee contract instead of inlining body:

```text
prove callee precondition
havoc state callee may modify
assume callee postcondition
continue caller proof
```

This depends on sound effect/frame summaries and dynamic dispatch target assumptions.

## 12. Dynamic dispatch contracts

For receiver with multiple possible runtime classes, caller must rely on contract guaranteed by all possible selected implementations, typically an interface/protocol/base contract. Per-subclass stronger guarantees may require refinement proving exact target.

## 13. Runtime checking versus static proof

Contracts may be runtime checked, statically proved/erased, both, or documentation-only. Mode behavior must be explicit.

Removing runtime check is valid only if proof assumptions remain true at native, reflection, dynamic, and concurrency boundaries.

## 14. Competency checks

1. Why does Hoare triple not define message dispatch?
2. What changes partial correctness into total correctness?
3. When must object invariant hold around `yield`?
4. Why can override not strengthen caller-facing precondition under substitutability?
5. What trust conditions are needed before erasing runtime contract check?
