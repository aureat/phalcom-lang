# Symbolic Execution and Path Conditions

## Symbolic state

Replace concrete inputs with symbolic values:

```text
x = X0
path = true
```

Executing:

```text
if x > 0
```

forks:

```text
path1 = X0 > 0
path2 = X0 <= 0
```

## State components

A symbolic executor may track:

```text
symbolic locals
heap abstraction
path condition
control outcome
call stack/summary context
provenance
```

## Feasibility pruning

Ask solver whether path condition is satisfiable. `unsat` paths are unreachable and can be dropped.

## Path explosion

Every branch can double states. Mitigate via:

- merge with ITE/phi expressions;
- abstract interpretation before splitting;
- bounded path sensitivity;
- subsumption/state caching;
- summary-based calls;
- concolic/testing mode for non-proof use.

## Bounded symbolic execution

Unrolling loops/calls N times can find bugs but cannot prove absence beyond the bound. Return `Unknown` for unproven residual behavior.

## Symbolic values versus checker types

A symbolic `X0` has a type/domain constraint but is not itself a type. Example:

```text
X0 : Int
assumptions: X0 >= 0
```

Type and proposition remain separate.

## Dynamic dispatch

Symbolically resolving a send requires receiver class/type constraints. Multiple possible targets branch or join summaries. Unknown dynamic receiver may become a proof boundary unless a universal protocol contract suffices.

## Counterexamples

A satisfiable path to an assertion violation gives a model. Reconstruct user-visible inputs and path decisions; verify the model respects Phalcom runtime domains (e.g. object/class constraints), not just untyped solver integers.

---

## Deep treatment: symbolic execution as proof engine and bug finder

### Transition relation

Let symbolic state be:

```text
S = (PC, L, H, O, Ctx, Prov)
```

where `PC` is path condition, `L` maps semantic locals to symbolic terms, `H` is symbolic heap, `O` is control outcome, `Ctx` contains call/handler/home-frame context, and `Prov` tracks origins.

A symbolic step relation is:

```text
S --stmt--> {S1, S2, ...}
```

Soundness requires that every concrete execution represented by `S` is represented by at least one successor state, except successors proven infeasible. Dropping a feasible path to control explosion is under-approximation and cannot support universal `Proven` unless the dropped region is conservatively summarized.

### Path feasibility

For branch predicate `b`:

```text
S_true.PC  = PC ∧ b
S_false.PC = PC ∧ ¬b
```

A solver can prune only when `PC ∧ b` is **unsatisfiable**. Timeout/unknown means the path remains potentially feasible.

### Merging

Two states at the same program point can merge:

```text
PC = PC1 ∨ PC2
x  = ite(PC1, x1, x2)
```

Heap merging is harder and may use heap `ite`, region-wise phi nodes, or abstraction. Merge strategy trades formula size against path count. It must not invent equalities between branches.

### Subsumption

If state `S1` is covered by `S2`, exploring `S1` may be unnecessary. Conceptually:

```text
γ(S1) ⊆ γ(S2)
```

Checking this exactly may itself require SMT. Use cheap syntactic/abstract subsumption first. An unsound “looks similar” subsumption can miss bugs and invalidate proof.

### Calls

Inlining dynamic calls causes recursion/path explosion and couples proofs to implementations. Prefer summaries. Symbolic execution of a verified/trusted summary transforms state relationally:

```text
prove Pre
fresh result, heap'
assume Post
apply effects
fork throws/non-local outcomes
```

Inline only when intentionally proving the callee itself or when a bounded bug-finding mode explicitly chooses expansion.

### Loops

Three distinct modes must not be conflated:

1. **Bounded symbolic execution:** unroll N times. Good for bug finding; residual loop means `Unknown` for universal proof.
2. **Invariant-based symbolic proof:** cut loop with inductive invariant and prove initiation/preservation/exit.
3. **Abstract interpretation summary:** compute sound loop invariant/fixed point and feed resulting facts into proof.

A production engine may combine them: abstract analysis generates candidate invariants; symbolic/SMT checks them.

### Counterexample executability

A `sat` model is a logical countermodel. It becomes a Phalcom counterexample only if all encoding invariants hold:

```text
classOf(object) corresponds to constructible runtime class
ADT tags/selectors are consistent
heap fields respect object layout/model constraints
native summaries are satisfiable in actual runtime
path decisions correspond to executable sends/control behavior
```

If the encoding leaves impossible states unconstrained, report an internal modeling issue rather than a misleading user witness.

### Interaction with gradual/dynamic values

A symbolic value may have runtime-object sort with partial class constraints. Do not simply assign it integer sort because arithmetic occurs later. A runtime type check can split:

```text
success path: classOf(x) satisfies Int -> projectInt(x)
failure path: cast/type error
```

Proof after the successful check may use integer theory.

### Concolic mode

Concolic execution pairs concrete values with symbolic expressions and systematically mutates branch constraints. It is excellent for generating tests and reproducing counterexamples. It remains testing/bug-finding unless all path coverage is proved complete, which is generally not true for loops/dynamic heaps.

### Resource policy

Track budgets per callable/query:

```text
max states
max splits
max solver calls
max formula nodes
max wall time
max recursion depth in bounded mode
```

Exhaustion produces `Unknown(BudgetExceeded)` with residual path information. Never silently discard states and keep proving.

### Tests

- unknown solver feasibility result retains both branches;
- state merge preserves branch-specific values using ITE/phi;
- bounded loop with no invariant cannot return `Proven`;
- summary call invalidates written fields but preserves framed fields;
- dynamic cast creates success and failure outcomes;
- impossible solver object models are rejected during witness reconstruction;
- budget exhaustion is stable and explicit.
