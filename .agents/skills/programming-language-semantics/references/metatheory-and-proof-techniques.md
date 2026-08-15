# Metatheory and Proof Techniques

Metatheory asks whether semantic rules themselves have desirable properties. Even without mechanizing every theorem, proof-oriented thinking exposes missing cases and hidden assumptions.

## 1. Induction on syntax

Use when property follows recursive AST structure:

```text
P(literal)
P(children) => P(parent)
------------------------
forall e. P(e)
```

Useful for free variables, elaboration preservation, and simple evaluator properties.

## 2. Induction on derivations

If:

```text
Γ ⊢ e : T
```

is inductively defined, prove properties by cases on final rule. This often matches preservation proofs better than syntax induction.

## 3. Rule inversion

From derivation conclusion, infer which rule produced it and which premises hold.

Example:

```text
Γ ⊢ send(e,s,args) : T
```

may imply receiver/member compatibility premises. Inversion recovers them.

## 4. Weakening

Typical lemma:

```text
if Γ ⊢ e : T and Γ ⊆ Γ'
then Γ' ⊢ e : T
```

only if extension does not alter resolution of free names. Shadowing-sensitive contexts require precise extension relation.

## 5. Substitution

```text
Γ,x:S ⊢ e : T
Γ ⊢ v : S
----------------
Γ ⊢ e[x:=v] : T
```

Capture-avoiding substitution matters mathematically even when runtime uses environments.

## 6. Determinism proof

For small-step:

```text
if C -> C1 and C -> C2 then C1 = C2
```

Usually prove by induction/case analysis showing rules are syntax-directed/non-overlapping. This may intentionally fail for scheduler/external choices.

## 7. Progress

```text
well-typed closed e
=> value(e) or exists e'. e -> e'
```

Requires canonical-forms lemmas and explicit treatment of dynamic errors/boundaries.

## 8. Preservation

```text
Γ ⊢ e:T and e->e'
=> Γ ⊢ e':T
```

or subtype-compatible variant. Usually depends on substitution, weakening, store typing, and heap invariants.

## 9. Store typing / heap invariants

With mutable storage, maintain relation:

```text
Σ : Loc -> Type/semantic contract
```

Allocation extends it; mutation preserves it. Dynamic Phalcom may use another interpretation, but pattern shows why state complicates preservation.

## 10. Simulation proofs

For lowering:

```text
R(sourceState,targetState)
```

and prove source transitions are matched by target transitions. Stuttering permits administrative target steps. Backward simulation/refinement may be needed for nondeterminism.

## 11. Bisimulation

For two semantics expected to match mutually, show each can simulate other's observable steps. Useful conceptual standard for reference interpreter versus VM.

## 12. Logical relations

Logical relations prove contextual equivalence, representation independence, and parametricity. Powerful, but likely too expensive for initial Phalcom formalization. Keep in reserve for generics/abstraction work.

## 13. Separation logic

Heap proofs benefit from disjoint footprints:

```text
P * Q
```

meaning state can be split into disjoint parts satisfying `P` and `Q`. This may matter for serious prover development, but should not be forced into core language semantics prematurely.

## 14. Coinduction

Use for potentially infinite behavior:

- divergence;
- infinite traces;
- streams;
- long-lived concurrent systems.

Inductive derivations naturally capture finite computations; coinduction captures infinite behavior.

## 15. Preservation of lookup assumptions

Dynamic object languages need lemmas beyond lambda calculus. Example property:

```text
if hierarchy/method tables unchanged
and lookup(C,s,access)=m
then repeated lookup under same state selects m
```

Reflection invalidates premise. Cache proofs depend on explicit version/immutability assumptions.

## 16. Counterexample search

Before proving theorem, search for counterexamples involving reflective replacement, dynamic boundary, native contract lie, aliasing, yield between reads, exception ordering, and dead non-local return.

A weaker theorem with explicit operational premises is more useful than a grand theorem with hidden assumptions.

## 17. Proof obligation ledger

For each claimed guarantee record:

```text
Theorem/claim
Premises
Trusted components
Dynamic boundaries
Reflection assumptions
Concurrency assumptions
Native assumptions
Status: intuition / tested / paper proof / mechanized proof
```

This prevents "checker is sound" from becoming undefined slogan.

## 18. Mechanization discipline

A proof assistant proves formal model, not automatically intended language or implementation. Validate all links:

```text
intended semantics <-> formal rules <-> implementation
```

Conformance tests and model review remain necessary.

## 19. Competency checks

1. When is derivation induction preferable to syntax induction?
2. Why does mutable storage require extra preservation invariants?
3. What does simulation relation accomplish in compiler correctness?
4. Why can mechanized proof validate wrong language?
5. Which Phalcom features counter naive sequential soundness theorems?
