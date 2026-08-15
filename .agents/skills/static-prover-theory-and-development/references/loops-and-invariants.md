# Loops and Invariants

## Why unrolling is insufficient

Checking 10 iterations proves only those 10. A loop proof needs an inductive invariant or another sound acceleration.

## Invariant obligations

For loop:

```text
while b { C }
```

with invariant `I`:

1. initiation: pre-state implies `I`;
2. preservation: `I ∧ b` and body imply `I'`;
3. exit: `I ∧ ¬b` implies desired postcondition.

## Finding invariants

Sources:

- user-supplied invariant contracts;
- abstract interpretation (intervals, equalities, types, shapes);
- template-based inference;
- Houdini-style candidate elimination;
- loop summaries from common patterns.

Do not promise automatic invariant inference for arbitrary loops.

## Loop-modified variables

Compute modifies set. Variables/fields not modified can be framed unchanged. Modified state gets fresh symbolic versions at loop head.

## Havoc + assume model

A common modular VC encoding:

1. assert invariant before loop;
2. havoc loop-modified variables;
3. assume invariant;
4. branch on condition;
5. verify one body iteration preserves invariant;
6. on exit assume `!condition` and invariant.

## Termination

Total correctness requires a variant `V` over a well-founded order:

```text
V >= 0
V decreases each iteration
```

Phalcom static prover can initially target partial correctness unless termination checking is explicitly required.

## Break/continue

- `continue` must establish invariant before back-edge;
- `break` exits with invariant/path facts that hold at that point;
- throws/returns leave through their own obligations.

## Iterator/for loops

Desugar semantically or give dedicated loop rules using iterable protocol summaries. Avoid assuming finite iteration unless boundedness is proven/contracted.

---

## Deep treatment: invariants as inductive summaries

### Invariant meaning

An invariant `I` is not merely “a fact often true in the loop.” It is an inductive set of states containing every reachable loop-head state. If `ReachHead` is the concrete set:

```text
ReachHead ⊆ γ(I)
```

for an abstract/logical invariant representation. To prove this, establish initiation and preservation.

### SSA cut-point encoding

Suppose loop modifies locals `i,sum` and field `self.count`. At loop head, create fresh symbolic versions:

```text
i_h, sum_h, H_h
```

The invariant constrains those fresh versions. The preservation obligation executes one body iteration from arbitrary states satisfying `I ∧ guard`, not merely from one concrete trace.

This is why `havoc + assume I` is sound: it forgets how the loop head was reached while retaining exactly the invariant summary.

### Computes-modifies soundness

A loop modifies set must include:

- direct local assignments;
- direct field/index writes;
- writes through aliases;
- writes of called methods in the loop;
- module/global effects;
- reflection/dispatch mutations if relevant.

Under-approximation preserves facts that might actually change and can create false proofs. Over-approximation is sound but weakens precision.

### Deriving invariants from abstract interpretation

Suppose interval analysis reaches fixed point:

```text
0 <= i <= n
sum >= 0
```

These facts can become candidate logical invariants only if the abstract analysis is sound for the same semantics. The prover may then validate them as explicit initiation/preservation obligations. This creates a strong architecture:

```text
abstract analysis proposes
VC engine checks
SMT proves arithmetic consequences
```

Do not assume an IDE-only heuristic narrowing is sound enough to seed proof.

### Houdini candidate elimination

Given candidates `{I1,...,Ik}`, Houdini-style inference starts by assuming all candidates and repeatedly removes any candidate not preserved until a fixed point. It discovers an inductive subset but cannot invent predicates outside the candidate language.

Conceptually:

```text
C0 = all candidates
Cn+1 = { I in Cn | body preserves I assuming Cn }
stop when Cn+1 = Cn
```

This is useful for invariants generated from types, intervals, bounds, equalities, and user hints.

### Relational invariants

Intervals alone cannot prove properties such as:

```text
sum == 2 * i
```

Relational domains/templates or SMT candidate checking are needed. The skill should teach the agent to identify the necessary invariant shape from the postcondition rather than merely increase solver timeout.

### Arrays/collections

A loop over collection contents may need quantified invariants:

```text
∀j < i. processed(xs[j])
```

These are substantially harder. Early Phalcom prover stages should prefer length/index safety and summary contracts unless quantified reasoning is explicitly in scope.

### `break`

A break path may not imply `¬guard`. Example:

```text
while true {
  if found { break }
}
```

Post-loop facts come from break conditions and any other exits, not the standard guard-false rule alone. Model break as a separate exit continuation.

### `continue`

Every continue back-edge must re-establish the invariant. A common bug is proving preservation only at the lexical end of the body.

### Abrupt exits

A `return`, throw, or non-local return inside a loop does not need to re-establish the loop invariant solely for the next iteration, but it must satisfy the corresponding enclosing outcome obligations. Cleanup/finally may run first.

### Termination

For variant `V` over natural numbers:

```text
I ∧ guard => V >= 0
I ∧ guard ∧ body => V' < V
```

More general well-founded orders may be needed for tuples/recursive structures. If first prover targets partial correctness, record termination as `NotChecked` rather than pretending loops terminate.

### Review example

```phalcom
sumTo(n) {
  let i = 0
  let s = 0
  while i < n {
    s = s + i
    i = i + 1
  }
  return s
}
```

To prove exact closed form requires invariant roughly:

```text
0 <= i <= n
2*s = i*(i-1)
```

The nonlinear multiplication may move the obligation into a harder theory. A prover limited to linear arithmetic should return Unknown/unsupported rather than weakening the claim.
