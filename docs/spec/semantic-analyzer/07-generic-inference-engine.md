# Phalcom Semantic Analyzer Implementation Specification
## 07 — Generic Inference Engine

**Status:** Normative semantic implementation specification.

**Purpose:** Specify Phalcom's generic inference model: inference variables, kinds, constraints, solver progress, failure evidence, epistemic support, expected-result constraints, and call-result publication.

---

## 1. Inference solves type equations; it does not create authority

Generic inference has two separate responsibilities:

1. determine a substitution that satisfies the type/kind constraints;
2. determine the epistemic strength of any result derived from that substitution.

The first is mathematical/type-theoretic.

The second is evidentiary.

A unique solution does not imply an established runtime fact.

---

## 2. Inference variables

A fresh inference variable conceptually has:

```text
InferVar
├── identity
├── kind
├── current solver state
├── equivalence representative
├── bounds/constraints
└── epistemic support summary
```

The exact data structure may use dense vectors, union-find, or another representation.

### 2.1 Kinds are intrinsic

An inference variable is a value of a specific kind.

Examples:

```text
T : Type
F : Type -> Type
G : Type -> Type -> Type
```

Missing kind metadata must not silently default to `Type` unless the declaration syntax and language semantics explicitly define that default at the declaration boundary.

Inside the solver, a missing expected kind is malformed semantic input, not an ordinary `Type` variable.

---

## 3. Constraint representation

Each stored constraint preserves:

```text
relation
left/right inference terms
origin
source/call/argument identity as appropriate
```

Representative relations include equality and subtype/assignability direction.

Origins may identify:

```text
call argument
labeled argument
receiver/Self
declared where-constraint
expected result
explicit generic argument
other semantic source
```

The solver must not reconstruct provenance after the fact from whichever bound happened to be stored first.

---

## 4. Argument matching precedes constraint solving

Calls map source arguments to formal parameters before generating generic constraints.

This mapping should be deterministic and performed once.

Unsupported dynamic labels or expansion packs fail closed rather than generating speculative constraints.

This keeps constraint generation linear/indexed with respect to ordinary argument/parameter matching and prevents repeated nested matching during solver passes.

---

## 5. Declared generic constraints

Generic declaration constraints participate in inference.

For example:

```text
where T <: Number
```

is not presentation metadata. It adds a formal solver constraint with a `GenericWhere`-like origin.

The actual type forms and kinds come from canonical declaration/signature products.

---

## 6. Solver operations preserve failure reasons

Operations such as:

```text
bind variable
unify terms
check subtype terms
materialize substitution
```

must return structured failure rather than a boolean that conflates different failures.

Representative failures:

```text
OccursCheck { var, term }
KindMismatch { var/term, expected, actual }
ConflictingBounds { var, lower, upper }
UnsatisfiedConstraint { relation }
UnresolvedSelf
MaterializationBlocked { reason }
```

Exact names may differ.

---

## 7. Real conflict evidence

When the solver rejects a candidate against multiple bounds, it must report the bound that actually failed.

Example:

```text
candidate C

C <: UpperA  = true
C <: UpperB  = false
```

The conflict evidence is:

```text
lower/candidate = C
upper           = UpperB
origin          = the actual constraint or final bound reconciliation
```

It is incorrect to report `UpperA` merely because it is the first element in a vector.

If failure is discovered outside a stored constraint, the solver may report no constraint index rather than inventing one.

---

## 8. No sentinel conflict data

The following are prohibited unless they are literally the real semantic values:

```text
InferVarId(0)
Never
Unit
Object
```

They must not be inserted into failure objects simply because an API requires a field.

If no variable owns a structural failure, use a structural failure variant.

---

## 9. Solver progress and fixed point

The solver may operate in bounded passes.

Conceptually:

```text
apply constraints
      ↓
bind/merge variables
      ↓
propagate bounds/support
      ↓
repeat while changed
```

If no convergence occurs within the configured solver strategy, return an explicit blocked/budget outcome.

Do not guess a substitution solely to terminate.

The configured number of passes is an implementation policy; explicit non-success semantics are normative.

---

## 10. Epistemic support

Every inference variable or representative tracks the weakest evidence class that has influenced its solution.

Conceptually:

```text
InferenceSupport =
    Established
  | Assumed
```

This is a taint/weakening summary, not proof that the variable is solved.

Fresh variables may initialize support to `Established` as the neutral element meaning “no assumed value premise has influenced this variable yet.” An unsolved variable remains underconstrained regardless of this support value.

---

## 11. Seeding support

When a value expression generates a constraint:

```text
Known Established -> Established support
Known Assumed     -> Assumed support
Unknown           -> no usable value constraint
Dynamic           -> dynamic path, not formal generic evidence
```

Expected-result context is not value support.

Declared generic constraints also do not become “assumed runtime evidence” merely because they constrain possible substitutions.

---

## 12. Support propagation

Support follows inference-variable dependency.

When variables alias, their support joins.

When a constraint with assumed support relates a variable to a compound term containing other variables, the affected representatives must conservatively receive assumed support where the dependency can influence their solutions.

The join is monotone:

```text
Established + Established -> Established
Established + Assumed     -> Assumed
Assumed + anything        -> Assumed
```

Support never upgrades during one inference session.

---

## 13. Return-influencing variables

After solving, result strength is determined only from inference variables that occur in the callable's return inference term.

Example:

```phalcom
foo<T>(value: T) -> Int
```

`T` does not influence the return.

Even if inference of `T` depends on assumed evidence, the fixed `Int` return can remain established when the exact callable contract independently establishes it.

By contrast:

```phalcom
id<T>(value: T) -> T
```

the result depends directly on `T`, so assumed support for `T` makes the result assumed.

---

## 14. Solved generic result publication

For a successfully solved generic call:

```text
collect return variables
        ↓
classify weakest support among them
        ↓
materialize return type
        ↓
publish knowledge
```

Result classification:

```text
no return variables + exact fixed contract
    -> established fixed result using correct callable origin

return variables all Established
    -> Established(materialized type, GenericInference)

any return-influencing variable Assumed
    -> Assumed(materialized type, GenericInference)
```

The solver must not attach evidence status to canonical `TypeId`s.

---

## 15. Expected-result context

Expected result context can constrain generic selection.

For example, surrounding context may make an otherwise ambiguous generic variable solvable.

However:

```text
expected result
```

is contextual selection, not runtime value evidence.

Therefore it may help choose a valid substitution but must not by itself make the inferred result `Established`.

It also cannot rescue a solver state that is already contradictory.

---

## 16. Non-solved outcomes

Representative outcomes:

```text
Solved
Underconstrained
Conflicting
Blocked
Cancelled
BudgetExceeded
```

The call analyzer must preserve them.

### 16.1 Underconstrained

If return-relevant variables remain unsolved, the result becomes an appropriate `Unknown(UnderconstrainedTypeVariable)` or equivalent.

### 16.2 Conflicting

A conflict produces invalid call semantics and retains the real inference conflict evidence.

If the return depends on the failed variables, do not publish partial specialization.

### 16.3 Blocked/cancelled/budget

These are terminal analysis outcomes, not generic substitutions. They must propagate through call analysis.

---

## 17. Fixed-return independence under terminal failure

A generic callable may have a return that is independent of generic inference:

```phalcom
foo<T>(x: T) -> Int
```

If exact callable identity and fixed return contract remain known, a terminal inference failure concerning `T` does not erase `Int`.

The call may therefore publish:

```text
knowledge = Established(Int)
status    = Invalid / Blocked / Cancelled / BudgetExceeded
```

as appropriate to the actual terminal event.

This does not mean the call is valid. It means the return proposition is independent.

---

## 18. No partial specialization after terminal failure

For:

```phalcom
pair<T, U>(...) -> Result<T, U>
```

if solving fails terminally, the analyzer must not publish a partially substituted `Result<Int, U>` as though it were a formal result unless the language has a separately specified partial-type product.

Part 1's model instead publishes the appropriate unknown/terminal semantics.

Only an inference-independent fixed concrete return survives failure.

---

## 19. `Self` and receiver specialization

Generic or callable `Self` must derive from actual receiver/class semantics.

Missing receiver specialization does not become `Unit` or another sentinel.

It produces a structured unresolved/blocked inference result.

Constructor semantics may independently establish `Self` when exact constructor dispatch provides the required receiver class.

---

## 20. Constraint provenance and explanation

The solver retains bounded provenance sufficient to answer:

- which source/call/argument generated the failed constraint?
- which generic variable was involved?
- which bound or relation actually failed?
- was the determining evidence established or assumed?
- did expected context participate in selection?
- which variables influence the return?

It does not need to duplicate every constraint into every variable's permanent record.

---

## 21. Complexity requirements

The semantic model permits efficient implementation.

Expected shapes:

```text
argument matching                 O(args + params)
constraint insertion              O(size of referenced inference term)
representative lookup             near-constant/amortized
support join                      O(vars touched)
return support classification     O(size of return term)
```

Path compression or other union-find optimizations are implementation choices.

Correctness takes precedence over a premature solver micro-optimization.

---

## 22. External behavior guarantees

Consumers may rely on:

- actual generic kinds participating in inference;
- missing higher-kinded metadata not silently becoming `Type`;
- real failed constraint evidence being preserved;
- no canonical sentinel types representing missing solver evidence;
- inference support never upgrading assumed input to established result;
- expected-result context selecting but not evidentially establishing a solution;
- fixed-return independence;
- no partial specialization after terminal failure;
- cancellation/budget/blockage remaining distinguishable.

---

## 23. Required regression families

### Solver-level

- fresh variable creation with real kinds;
- occurs-check failure;
- kind mismatch;
- variable aliasing;
- lower/upper bound solving;
- second/later upper bound is the actual conflict;
- structural constraint failure;
- support weakening through aliases/compound terms;
- underconstrained result;
- non-convergence/blocked path.

### Call-level

- established argument -> established generic result;
- assumed argument -> assumed generic result;
- weakest return-variable support wins;
- assumed non-return variable does not weaken fixed result;
- fixed return survives conflict/block/cancel/budget when independently known;
- return-dependent conflict yields unknown rather than partial type;
- expected result selects valid substitution without fabricating support;
- expected result does not rescue contradiction;
- generic failure diagnostic points to real call/argument/constraint evidence.

---

## Source basis

This specification is derived from the Part 1 Formal Semantic Epistemic Foundation specification and its Corrections and Amendments. The amendments take precedence on generic failure evidence, inference support, suppression-cause representation, and semantic fingerprinting. Repository implementation notes were re-grounded against `aureat/phalcom-lang` `main` at `c3b82e4b88469ef9fc79aa65a03e0bed95dc908d`; such notes are non-normative and may be updated as the code evolves.
