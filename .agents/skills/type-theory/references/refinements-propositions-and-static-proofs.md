# Refinements, Propositions, Occurrence Typing, and Static Proof Boundaries

## Purpose

This reference explains how ordinary types interact with path-sensitive propositions and contracts without collapsing the type checker into the prover.

Phalcom's future `@requires`, `@ensures`, and invariants make this boundary important: a checker may know `x : Int`, flow analysis may know `x > 0`, and a prover may establish a stronger theorem. Those facts should cooperate while remaining distinct semantic domains.

## 1. Refinement type idea

A refinement narrows a base type with predicate:

```text
{x : Int | x > 0}
```

Set interpretation:

```text
[[{x:T | P(x)}]] = { v ∈ [[T]] | P(v) }
```

Formation requires `P` to be a meaningful proposition over values of `T` under the chosen logic.

Phalcom need not expose full refinement-type syntax to exploit refinement facts internally.

## 2. Separate canonical type from proposition environment

Instead of materializing a new canonical type for every branch fact:

```text
Γ(x) = Option<String>
Φ = { x is Some }
```

The branch-specific usable view can be derived from `(Γ, Φ)`.

Benefits:

- canonical type identity remains stable;
- propositions can include value inequalities/equalities not representable as types;
- prover facts can expire at mutation points;
- diagnostics can explain the refinement source.

## 3. Occurrence/flow typing

Condition:

```text
if x is Cat { ... }
```

can refine true branch:

```text
Γ(x) = Animal
Φ_true += runtime_class(x) = Cat
usable_type(x) = Cat
```

False branch may refine to `Animal \ Cat` only if the type system can represent or safely approximate the complement.

For a closed union:

```text
x : Cat | Dog
```

false branch can become `Dog` after exact `is Cat` test.

## 4. Trusted refinement predicates

Not every Boolean-returning method is a type predicate.

```text
x.looksLikeCat()
```

cannot refine `x` unless semantics/trusted contract says the predicate implies class/type membership.

Trusted sources may include:

- built-in runtime class test;
- exhaustive constructor pattern;
- exact `Option` variant test;
- explicitly declared/proven type-guard attribute;
- verified contract.

This prevents unsound user-defined Boolean methods from becoming proof axioms.

## 5. Mutation invalidates refinements

Suppose:

```text
if field is Some {
  call_user_code()
  use field.value
}
```

If `call_user_code()` can mutate `field`, the refinement may no longer hold.

A flow engine needs stability/alias/effect reasoning:

- local immutable binding: refinement stable;
- mutable local not reassigned on path: potentially stable;
- field reachable through alias/user call: may be invalidated;
- concurrent/shared state: suspension/calls may invalidate.

Do not retain smart-cast facts across mutation points without proof.

## 6. Contracts as logical obligations

Method contract:

```text
requires P(args,self)
ensures Q(args,self,result)
```

At call site:

```text
prove P(actuals,receiver)
```

Inside/after method verification:

```text
assume P at entry
prove Q at each normal return
```

This is proof machinery, not ordinary subtype checking unless predicates are deliberately embedded into refinement types.

## 7. Hoare-logic bridge

A command contract:

```text
{P} C {Q}
```

means if `P` holds before `C` and `C` terminates normally, `Q` holds after.

Type checking can discharge simple obligations structurally; stronger value predicates may go to static prover.

Example:

```text
requires n > 0
result: Int
ensures result >= 0
```

Type checker proves `n : Int`; prover reasons about inequalities.

## 8. Proof status is separate

Use:

```text
Proven
Disproven(counterexample/model)
Unknown(reason)
```

`Unknown` includes solver timeout/unsupported logic. It is not success and not necessarily a type error.

A program can be type-correct while a stronger contract remains unproven.

## 9. Contradictions and bottom

If path propositions imply false:

```text
Φ ⊢ False
```

then path is unreachable. Analysis can mark control state bottom and suppress diagnostics in dead path.

But contradiction must be soundly proven. Failure to understand `Φ` is not contradiction.

## 10. Type unions as propositions—limited analogy

Under set interpretation:

```text
A | B  resembles membership disjunction
A & B  resembles membership conjunction
```

But proposition logic is richer:

```text
x > 0
x == y
field(self,#state) == #ready
```

Do not encode every proposition as a type union/intersection. Keep a logical fact domain.

## 11. Refinement subtyping

A refinement can be subtype of its base:

```text
{x:T | P} <: T
```

To prove:

```text
{x:T | P} <: {x:T | Q}
```

need logical implication:

```text
P => Q
```

This connects subtype checking to a theorem prover and can make type checking expensive/undecidable if predicates are unrestricted.

Phalcom should adopt full refinement subtyping only with a carefully bounded logic/solver architecture.

## 12. Decidability and budget

General Phalcom code as predicates is undecidable. Options:

- restricted decidable predicate language;
- SMT-supported fragment;
- user invariants;
- bounded symbolic execution returning `Unknown`;
- runtime contract checks for unproven obligations.

Never make ordinary editor typing depend on unbounded theorem proving.

## 13. Native and reflection trust

A native method's declared `ensures` is only as trustworthy as native authority policy.

A reflective call can bypass statically known target contracts.

Prover needs a trust model:

```text
TrustedAxiom
VerifiedContract
RuntimeCheckedContract
UntrustedDeclaration
DynamicUnknown
```

Do not assume every annotation/contract is proven true merely because it exists.

## 14. Provenance

Refinement facts should carry cause:

```text
Fact: x is Some
Origin: pattern arm at span ...
DependsOn: binding x version 12
InvalidatedBy: assignment/call effect touching x
```

This supports diagnostics and incremental invalidation.

## 15. LSP presentation

An IDE can show:

```text
x: Option<String>
refined here to: Some<String>
because: pattern `Some(name)`
```

without changing source annotation or reflective type identity.

This is a good example of "one semantic truth, many views".

## 16. Testing obligations

- trusted runtime class narrowing;
- false-branch union subtraction where closed;
- user Boolean method does not refine without guard contract;
- mutation invalidates field refinement;
- immutable local keeps refinement;
- contradiction creates unreachable state only when proven;
- prover timeout returns `Unknown`, not `Proven`;
- native untrusted contract not assumed;
- diagnostic provenance identifies guard/pattern source.

## 17. Failure modes

- Every `Bool` method becomes a type guard.
- Refinement facts stored permanently in canonical `TypeId`.
- Field smart cast survives arbitrary user call.
- Solver timeout treated as proof.
- Type correctness equated with contract proof.
- Native declarations treated as automatically verified postconditions.

## 18. Competency questions

1. Why keep path propositions separate from canonical types?
2. What invalidates a refinement on mutable state?
3. When does `{x:T|P} <: {x:T|Q}` require theorem proving?
4. Why is proof `Unknown` not a type `Unknown`?
5. What predicates may safely refine types?
