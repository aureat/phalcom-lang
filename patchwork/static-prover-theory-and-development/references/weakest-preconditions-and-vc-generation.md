# Weakest Preconditions and Verification Conditions

## Weakest precondition

`wp(C, Q)` is the weakest condition on the pre-state sufficient to guarantee postcondition `Q` after executing `C` under the chosen correctness model.

For assignment:

```text
wp(x := e, Q) = Q[e/x]
```

For sequence:

```text
wp(C1; C2, Q) = wp(C1, wp(C2, Q))
```

For conditional:

```text
wp(if b then C1 else C2, Q)
= (b => wp(C1,Q)) ∧ (!b => wp(C2,Q))
```

## Assertions and assumptions

```text
wp(assert P, Q) = P ∧ Q
wp(assume P, Q) = P => Q
```

depending on exact semantics/encoding conventions.

## Verification condition generation

For a method with precondition `Pre` and postcondition `Post`:

```text
VC = Pre => wp(body, Post)
```

Prove VC valid. SMT approach commonly asks solver whether `¬VC` is satisfiable:

```text
unsat -> Proven
sat   -> Disproven/model
unknown -> Unknown
```

## SSA helps

Transforming mutable locals to SSA avoids repeated substitution explosion:

```text
x0 = input
x1 = x0 + 1
```

VCs become logical equations over versions.

## Path merging

Naively expanding conditionals duplicates formulas exponentially. Use SSA phi/ITE expressions, predicate sharing and DAG/hash-consed proposition IR.

## Exceptional/control outcomes

Generate separate postconditions/continuations for:

```text
normal
return
throw
break/continue
non-local return
yield if verification spans suspension
```

A continuation-style WP representation handles this cleanly.

## Source provenance

Each generated proposition/obligation should retain source and semantic origin so unsatisfied constraints can explain which expression created them.

## Simplification

Before SMT:

- constant fold propositions;
- apply known type facts;
- eliminate unreachable branches;
- canonicalize equality/order expressions;
- propagate SSA equalities;
- use abstract-analysis intervals/presence facts.

Simplification reduces solver time and diagnostic noise.

---

## Deep treatment: a compositional VC generator

### Continuation-parametric WP

Classic `wp(C,Q)` assumes one normal continuation. Phalcom benefits from continuation-parametric WP:

```text
WP[C](K) : Proposition
```

where `K` contains postconditions for distinct outcomes. Conceptually:

```text
K = {
  normal(v,H)   -> Pn,
  return(v,H)   -> Pr,
  throw(e,H)    -> Pt,
  nonlocal(h,v,H) -> Pnl,
  suspend(s,H)  -> Ps
}
```

This prevents later ad hoc patches for exceptions or non-local returns.

### Core rules

For a pure local assignment in SSA-like form:

```text
WP[x := e](K) = WP[e](λv,H. K.normal with x' = v)
```

For `assert P`:

```text
WP[assert P](K) = P ∧ K.normal(unit,H)
```

For `assume P`:

```text
WP[assume P](K) = P => K.normal(unit,H)
```

For sequence:

```text
WP[C1 ; C2](K) = WP[C1](K where normal = λ_,H. WP[C2](K) at H)
```

For conditional with pure logical guard `b`:

```text
WP[if b then Ct else Cf](K)
= (b => WP[Ct](K)) ∧ (¬b => WP[Cf](K))
```

If evaluating `b` can have effects or throw, first lower the evaluation semantics so `b` is a pure logical value at the branch point.

### Why source expressions cannot always appear directly in formulas

Consider:

```phalcom
if account.isOpen { ... }
```

`isOpen` may be a message send. It cannot simply become solver predicate `isOpen(account)` unless a summary proves the call pure enough and relates its result to a logical predicate. Correct lowering is:

```text
evaluate send -> result b, heap H1, possible throw/effects
branch on b in H1
```

The logical term only appears after the language evaluation has been accounted for.

### SSA and heap SSA

Mutable locals are easiest to reason about when versioned:

```text
x0 = input
x1 = x0 + 1
x2 = ite(cond, x1, x0)
```

Heap state similarly receives versions:

```text
H1 = store(H0, self, f, v)
```

A call with broad effects may create `H2` constrained only by the callee frame/postcondition. Avoid explicit substitution of giant heap expressions through every postcondition; use named versions/equalities.

### Call rule in VC generation

Suppose callee summary has `Pre`, `Post`, `MayWrite`, and exceptional summary `Exc`. At a call:

```text
1. emit obligation Pre(actuals,H0)
2. create fresh result r and H1
3. constrain unchanged framed locations between H0 and H1
4. assume Post(actuals,r,H0,H1) on normal path
5. route possible exceptional outcomes through K.throw/etc
```

If `Post` is not verified/trusted according to current proof mode, it cannot be assumed as theorem.

### Loops via cut points

Direct recursive WP of `while` does not terminate. Introduce invariant `I` and modified set `M`:

```text
initiation VC: current_state => I

preservation VC:
  havoc M
  assume I
  assume guard
  execute body
  prove I at each back-edge/continue

exit continuation:
  havoc M
  assume I
  assume ¬guard
  continue after loop
```

This is not arbitrary havoc: only state the loop may modify should be forgotten. An under-approximated modifies set is unsound.

### Break and continue

`continue` targets the invariant-preservation continuation. `break` targets the loop exit continuation but may carry stronger path facts than ordinary guard-false exit. The post-loop state is the join/disjunction of all exits.

### Formula DAGs

Naive WP expansion can blow up exponentially:

```text
if b1 ...
if b2 ...
...
```

Use interned DAG propositions, named let/SSA bindings, or solver `ite` terms. Keep formula size metrics. A simplifier should preserve sharing rather than stringify-and-reparse terms.

### Quantifier introduction

Do not introduce quantifiers just because a contract mentions collections. Prefer library summaries with quantifier-free facts where adequate:

```text
0 <= index < length(seq)
length(append(seq,x)) = length(seq)+1
```

Universal collection properties may require quantifiers, induction, or specialized theories and should be recognized as a harder proof class with explicit budgets.

### Correctness invariant of the generator

For each lowering rule, the intended theorem is roughly:

```text
If σ satisfies WP[C](K), then every concrete execution of C from σ
that produces outcome o reaches a state satisfying K(o).
```

The implementation may not formally prove this theorem initially, but tests and code structure should mirror it. Each new statement/control construct must define its effect on this invariant.

### Worked example

```phalcom
@requires(x >= 0)
@ensures(result >= 1)
f(x) {
  if x == 0 { return 1 }
  return x + 1
}
```

Ignoring call effects because operations are assumed trusted integer primitives, normal-return VC becomes:

```text
x >= 0 =>
  ((x == 0) => 1 >= 1)
  ∧
  ((x != 0) => x + 1 >= 1)
```

The second branch needs integer arithmetic plus `x >= 0 ∧ x != 0`, hence `x >= 1` over integers.

### Failure modes

- Applying branch rule to an effectful guard without evaluating it first.
- Treating all returns as fall-through.
- Forgetting heap version updates in `old`-based postconditions.
- Havocing too little state at calls/loops.
- Assuming unverified callee ensures.
- Letting simplification use heuristic/non-sound facts.
- Exponential formula duplication from tree construction.

### Tests

Snapshot normalized VCs for tiny constructs. Add mutation tests that delete one branch premise, one heap update, or one call precondition and ensure the test suite detects the unsoundness.
