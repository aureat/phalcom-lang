# Inference, Constraints, Unification, and Bidirectional Typing

## Purpose

Use this reference when an agent is asked to "infer the type". That phrase is dangerously underspecified.

At least three distinct Phalcom activities can be called inference:

1. advisory runtime/value-shape inference for semantic tooling;
2. correctness-oriented static type synthesis/checking;
3. generic type-argument inference for a particular type application/send.

They may share semantic identities and evidence. They do not have the same guarantees or failure policy.

## 1. Inference is solving a relation, not guessing a class

A minimal type inference pipeline is:

```text
resolved syntax + expected context
            ↓
      generate constraints
            ↓
  solve equalities/bounds/obligations
            ↓
 validate restrictions + substitute
            ↓
 canonical type OR explicit non-solution state
```

If an implementation contains logic like:

```text
match ast_kind {
  IntLiteral => "Int",
  Call => callee_name_guess,
  _ => "Unknown"
}
```

it may be useful shape inference, but it is not a correctness-oriented type solver.

## 2. Metavariables are not language type parameters

Distinguish:

```text
TypeParamId      source/declaration binder, reflectively meaningful
InferenceVarId   solver-local metavariable, temporary and not reflectable
```

Given generic method:

```text
identity<T>(x: T) -> T
```

a call `identity(42)` creates a fresh inference variable `?α` representing the instantiation of declaration parameter `T` for this call.

Do not reuse the reflective `T` object as mutable solver state.

## 3. Equality unification

For a first-order type language without subtyping, unification solves equations.

Example constraints:

```text
?α = Int
List<?β> = List<String>
```

The most general unifier is:

```text
?α ↦ Int
?β ↦ String
```

### Core algorithm

For terms `s` and `t`:

```text
unify(s,t):
  s = prune(s)
  t = prune(t)

  if s == t: success
  if s is var α: bind(α,t)
  if t is var α: bind(α,s)
  if s = F(s1..sn) and t = F(t1..tn):
      unify(si,ti) for each i
  otherwise: conflict
```

### Occurs check

Before binding:

```text
?α := List<?α>
```

check whether `?α` occurs in the candidate term. If yes, ordinary finite-tree unification rejects it.

Why: otherwise repeatedly expanding substitution gives an infinite type:

```text
?α = List<List<List<...>>>
```

If recursive types are supported, represent them through explicit recursive descriptors/fixed points; do not disable the occurs check casually.

## 4. Union-find versus structured terms

Union-find is excellent for equivalence classes of variables, but generic type expressions still require structured decomposition.

A practical solver may use:

```text
InferenceVar table:
  parent/rank
  optional bound TypeTerm
  lower bounds
  upper bounds
  provenance
```

and canonical `TypeId`s for finalized structures.

Do not intern unresolved solver nodes into the permanent type interner.

## 5. Subtyping constraints

With subtyping, constraints become inequalities:

```text
Int <: ?α
?α <: Number
```

The variable has:

```text
lower bounds L = {Int}
upper bounds U = {Number}
```

A valid solution `S` must satisfy:

```text
∀L_i. L_i <: S
∀U_j. S <: U_j
```

Choosing `S` is a language policy. Possible choices include:

- least admissible supertype of lowers that satisfies uppers;
- greatest admissible subtype of uppers that satisfies lowers;
- explicit union of lowers;
- require uniqueness/principal solution;
- reject underconstrained/ambiguous cases.

Do not hard-code `Any` because it satisfies upper-bound convenience. That can erase programmer errors.

## 6. Constraint provenance

Every generated constraint should know why it exists.

Conceptual form:

```text
Constraint {
  relation: Subtype(actual, expected),
  origin: SourceSpan,
  cause: Argument { call, parameter_index },
  parent: Option<ConstraintId>
}
```

Conflict:

```text
Int <: ?T
?T <: String
```

can then diagnose:

```text
cannot infer T
  lower requirement: Int
    from argument `x` at ...
  upper requirement: String
    from declared bound/expected result at ...
```

Do not discard provenance after solving and attempt to reconstruct it from AST later.

## 7. Bidirectional typing

Bidirectional typing reduces global search by assigning modes to syntax.

### Synthesis

```text
Γ ⊢ e ⇒ T
```

Compute `T` from `e`.

Typical synthesizing forms:

- literals;
- resolved names;
- annotated expressions;
- many message sends once receiver/member contract is known;
- explicit generic applications.

### Checking

```text
Γ ⊢ e ⇐ T
```

Use expected `T` to validate and guide `e`.

Typical checking-friendly forms:

- blocks with unannotated parameters;
- empty collection literals;
- ambiguous sum/variant constructors;
- polymorphic values requiring instantiation;
- expressions with contextual numeric/literal adaptation if Phalcom chooses it.

### Subsumption bridge

```text
Γ ⊢ e ⇒ S     S <: T
────────────────────
Γ ⊢ e ⇐ T
```

Keep the bridge explicit. Do not silently use assignability/consistency where strict subtyping was intended.

## 8. Worked bidirectional example: block

Expected type:

```text
(Int) -> String
```

Source:

```text
|x| { x.toString }
```

Checking proceeds:

1. expected callable gives parameter `x : Int`;
2. resolve/synthesize `x.toString` under `x : Int`;
3. check body result against `String`;
4. produce no need to guess block parameter type.

Without expected type, synthesis may fail or require an explicit parameter annotation. That is not necessarily a language defect; it is a local inference boundary.

## 9. Generic call inference

Consider:

```text
pair<T>(a: T, b: T) -> Pair<T,T>
```

Call:

```text
pair(cat, dog)
```

Assume:

```text
Cat <: Animal
Dog <: Animal
```

Algorithm:

1. instantiate declared `T` with fresh `?α`;
2. parameter templates become `?α`, `?α`;
3. each actual argument creates a lower-bound constraint:

```text
Cat <: ?α
Dog <: ?α
```

4. solve `?α` according to join policy; if explicit union types are the LUB, candidate may be `Cat | Dog`; if language prefers nearest nominal supertype, candidate may be `Animal`;
5. validate declared bound/finite constraints;
6. substitute into result `Pair<?α,?α>`;
7. canonicalize final type.

The "right" inferred answer is not determined by type theory alone. It depends on Phalcom's declared inference policy.

## 10. Expected result constraints

Context can constrain generic inference:

```text
const x: Box<Number> = makeBox(1)
```

An expected result can generate constraints from:

```text
result_template(?α) <: Box<Number>
```

Whether result context participates in inference is a deliberate design choice. It improves inference but can make error messages less local or change inferred types when surrounding code changes.

If enabled, document directionality and precedence.

## 11. Variance-aware decomposition

Constraint:

```text
F<A> <: F<B>
```

can be decomposed based on `F`'s parameter variance.

For covariant position:

```text
A <: B
```

For contravariant:

```text
B <: A
```

For invariant:

```text
A ≡ B
```

Nested variance composes. A solver should call the same variance/polarity machinery used by generic subtyping, not duplicate ad-hoc rules.

## 12. Finite exact constraints versus upper bounds

Phalcom's current generic design distinguishes:

```text
T : Bound          # upper bound
T in (A, B, C)     # finite exact constraint set
```

These are different solver domains.

Upper bound:

```text
solution(T) <: Bound
```

Finite constraints:

```text
solution(T) ≡ A or ≡ B or ≡ C
```

A solver must not interpret the finite set as union `A | B | C` unless a later specification explicitly defines that promotion behavior.

## 13. Underconstrained, ambiguous, and inconsistent

These outcomes differ.

### Underconstrained

```text
make<T>() -> T
const x = make()
```

No constraints choose `T`.

### Ambiguous

Two incomparable admissible solutions satisfy all constraints and policy has no tie-breaker.

### Inconsistent

No solution satisfies constraints:

```text
Int <: ?T
?T <: String
```

assuming `Int` is not a subtype of `String` and no admissible intermediate exists.

### Blocked

The solver cannot decide because imported metadata or a dependency is unresolved.

Do not collapse these into `Dynamic` or one analyzer `Unknown`.

## 14. Principal types

A principal type is a most-general typing from which all other valid typings can be obtained by substitution/instantiation under the system's relation.

Hindley-Milner has principal typings under strong assumptions:

- parametric polymorphism;
- equality-based unification;
- no unrestricted subtyping/overloading;
- disciplined generalization.

A Phalcom system with nominal/structural subtyping, unions/intersections, gradual typing, reflection, generic bounds, and contextual checking may not have principal types for all expressions.

Do not promise "the most general type" unless the language defines and the solver proves a principal-solution property for that fragment.

## 15. Generalization and instantiation

HM-style let polymorphism uses:

```text
Gen(Γ, T) = ∀α1...αn. T
```

where quantified variables are free in `T` but not `Γ`.

At use sites, instantiate quantified variables with fresh metavariables.

Phalcom's reflective, declaration-owned generic parameters are not automatically HM let-polymorphism. If local implicit generalization is ever proposed, it needs explicit semantics for:

- mutable captured state;
- value restriction or equivalent;
- reflection of inferred quantifiers;
- method/class generic ownership;
- dynamic boundaries.

Do not introduce hidden let-generalization just because unification infrastructure exists.

## 16. Recursion and SCCs

Unannotated mutually recursive methods can generate cyclic result constraints:

```text
A.foo -> B.bar -> A.foo
```

Options:

1. require explicit result annotations on recursive SCCs;
2. seed and iterate a monotone type approximation to a fixed point;
3. use recursive type equations where semantically justified.

If iterating, define:

```text
initial seed
order
join
termination/widening
precision loss
```

A correctness checker should not copy an LSP's bounded union widening if that changes acceptance.

## 17. Solver worklist architecture

Conceptual structures:

```text
InferenceVarId
ConstraintId
ObligationId

VarState {
  lower: Set<TypeTerm>,
  upper: Set<TypeTerm>,
  equality: Option<TypeTerm>,
  status,
}

worklist: Queue<ObligationId>
```

Each obligation can produce more obligations. Use canonical semantic IDs for finalized types and stable source IDs/spans for provenance.

A worklist should terminate through a decreasing/finite measure or explicit SCC/fixed-point strategy, not by "max 1000 iterations then accept".

## 18. Incremental inference

A cached inference result depends on:

- resolved binding/member identities;
- declared annotations and generic signatures;
- expected type where used;
- callable summaries/return sites;
- subtype/conformance relation generations;
- module visibility;
- checker mode.

Changing one generic bound should invalidate calls that depend on that signature, not every file and not zero files.

Incremental recomputation must satisfy a key metamorphic property:

```text
facts_after_incremental_edit == facts_after_clean_full_analysis
```

modulo permitted diagnostic ordering/presentation differences.

## 19. Diagnostics as unsatisfied constraints

Prefer reporting a small causal core over dumping all solver state.

For conflict:

```text
expected String
  because parameter `name` is declared String
found Number
  because argument `x` has inferred Number
    because return sites of foo() join to Number
```

This requires causal edges from type facts to constraints.

## 20. Failure modes

- Using `Any`/`Dynamic` for underconstrained variables.
- Omitting occurs check because recursive types may exist someday.
- Letting inference variables escape into reflection/cache identity.
- Applying expected-result inference without documenting context sensitivity.
- Treating finite exact constraints as ordinary upper bounds.
- Using one fixed iteration cap as recursive inference semantics.
- Reusing LSP `ValueShape` union caps in checker type inference.
- Reporting the final contradictory types without the constraints that caused them.

## 21. Testing obligations

Include:

- first-order unification success/failure;
- occurs-check rejection;
- nested generic substitution;
- lower/upper bound solving;
- invariant/covariant/contravariant decomposition;
- expected-type influence;
- underconstrained versus ambiguous versus inconsistent outcomes;
- recursive SCC policy;
- shadowed type parameters;
- finite constraint exactness;
- dynamic boundary policy;
- incremental/full equivalence.

Property tests can generate small type terms and verify that a solver substitution actually satisfies every emitted constraint.

## 22. Competency questions

1. Why is `TypeParamId` different from `InferenceVarId`?
2. What does the occurs check prevent?
3. Given lower bounds `{Cat, Dog}`, what extra language rule is needed before choosing `Animal`?
4. Why can expected-result inference make diagnostics/context sensitivity harder?
5. How does variance change decomposition of `F<A> <: F<B>`?
6. Why is `T in (A,B)` not automatically the same as `T <: A | B`?
7. What distinguishes underconstrained, ambiguous, inconsistent, and blocked inference?
