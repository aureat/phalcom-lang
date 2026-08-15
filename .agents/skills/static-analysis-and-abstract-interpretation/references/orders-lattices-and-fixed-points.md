# Orders, Lattices, and Fixed Points

This reference is the mathematical engine behind most monotone static analyses. The purpose is not to make analysis code look formal. The purpose is to state precisely what “more information,” “merge,” “conservative,” “stable,” and “terminates” mean, then connect those claims to a Rust worklist solver.

## 1. The abstract domain starts with an order

An abstract domain `A` is a set of abstract facts plus a relation `⊑` describing their information/approximation order. Pick one convention and use it consistently in code, documentation, and tests.

For a forward may-analysis, a convenient convention is:

```text
a ⊑ b
```

means:

> `a` is at least as precise as `b`; equivalently, every concrete state represented by `a` is also represented by `b`.

If `γ : A -> P(C)` maps an abstract value to the set of concrete states it represents, then:

```text
a ⊑ b  implies  γ(a) ⊆ γ(b)
```

A runtime-class-set domain illustrates the order:

```text
{Int} ⊑ {Int, String} ⊑ AnyClass
```

The rightward values represent more possible executions and therefore less precision.

Do not automatically reuse this order for every analysis. A must-analysis may represent facts known to hold on all executions. Its implementation can still be expressed as a lattice, but the intuitive set order is often reversed. State the order explicitly instead of inferring it from container operations such as union or intersection.

### Partial-order laws

A valid partial order satisfies:

```text
reflexivity:     a ⊑ a
antisymmetry:    a ⊑ b ∧ b ⊑ a  =>  a = b   (semantic equality)
transitivity:    a ⊑ b ∧ b ⊑ c  =>  a ⊑ c
```

If a relation is symmetric in interesting cases—for example “`Int` is compatible with `Dynamic` and `Dynamic` is compatible with `Int`”—it may be a consistency/compatibility relation, not an order. Do not feed such a relation into a fixed-point algorithm that assumes a partial order.

## 2. Join is the least conservative merge

For `a, b ∈ A`, the join `a ⊔ b` is the least upper bound: the most precise abstract value that safely covers both inputs.

Formally:

```text
a ⊑ a ⊔ b
b ⊑ a ⊔ b

and for every u:
  a ⊑ u ∧ b ⊑ u  =>  a ⊔ b ⊑ u
```

A finite class-set domain may use set union:

```text
{Int} ⊔ {String} = {Int, String}
```

A constant domain can use:

```text
⊥ ⊔ Const(1)      = Const(1)
Const(1) ⊔ Const(1) = Const(1)
Const(1) ⊔ Const(2) = ⊤
```

### Join laws are implementation invariants

A canonical lattice join should satisfy:

```text
idempotence:     a ⊔ a = a
commutativity:   a ⊔ b = b ⊔ a
associativity:  (a ⊔ b) ⊔ c = a ⊔ (b ⊔ c)
```

These laws are not aesthetic. If `join` is non-commutative, branch results can depend on AST traversal order. If it is non-associative, worklist scheduling can change results. If it is non-idempotent, revisiting an unchanged predecessor can cause artificial growth and nontermination.

Phalcom's current advisory `ValueShape::join` should be reviewed with exactly this mindset. It uses bounded unions and returns `Unknown` when alternatives exceed the cap. Because `Unknown` is absorbing for joins in that domain, it is a widening-like loss of precision suitable for advisory shape inference. A future correctness type domain cannot inherit this policy merely because the enum and `join` method already exist.

## 3. Bottom and top are opposites

Under the may-analysis convention above:

```text
⊥  = no concrete states represented
⊤  = every concrete state admitted by the domain
```

Therefore:

```text
⊥ ⊑ a ⊑ ⊤
⊥ ⊔ a = a
a ⊔ ⊤ = ⊤
```

### Bottom is not “unknown”

If an `if` branch is impossible, its abstract state is `⊥`/unreachable. Joining it with a reachable branch should leave the reachable branch unchanged:

```text
join(⊥, {x = Int}) = {x = Int}
```

If instead “unknown” means `x` can be any value, joining should destroy precision:

```text
join({x = Int}, {x = ⊤}) = {x = ⊤}
```

An implementation that uses one `Unknown` variant for both cases loses the fundamental distinction between no executions and all executions.

### There may be several uncertainty states

A language analysis often needs more semantic status than lattice top alone:

```text
Unreachable              lattice bottom / no executions
Known(a)                 ordinary abstract value
Top                      all values in modeled domain
Dynamic                  language-level escape or dynamic boundary
Blocked(reason)          missing dependency / incomplete source
Ambiguous(candidates)    semantic ambiguity
BudgetExhausted(a)       conservative approximation plus reason
Inconsistent             checker/constraint contradiction
```

Not every status belongs inside the lattice. A practical design can wrap a lattice value in a status/provenance layer. The important rule is that the solver's algebra and the consumer's uncertainty policy remain explicit.

## 4. Meet and must-style reasoning

The meet `a ⊓ b` is the greatest lower bound. It is useful for intersections of possibilities, backward reasoning, and must-properties.

For a powerset domain under subset order:

```text
join = union
meet = intersection
```

For “definitely initialized variables,” a state can be represented as the set of variables definitely initialized on the current path. At a control-flow merge, a variable remains definite only if it appears in every predecessor:

```text
DefOut = DefLeft ∩ DefRight
```

This does not contradict the lattice framework. It means the domain/order was chosen for a must-property. Avoid cargo-culting “join = set union.” The abstract semantics decides the operator.

## 5. Complete lattices and fixed points

A complete lattice has joins/meets for all subsets, not only pairs. Tarski's fixed-point theorem states that a monotone function over a complete lattice has least and greatest fixed points.

For static analysis, the relevant intuition is:

```text
F(x) = x
```

means the abstract information has stabilized under the program transfer function.

A loop header can be expressed as:

```text
H = Entry ⊔ BodyBackEdge(H)
```

A recursive callable family can be expressed as a vector equation:

```text
S_f = F_f(S_f, S_g, ...)
S_g = F_g(S_f, S_g, ...)
...
```

The analysis seeks a conservative solution, commonly the least fixed point with respect to the chosen order.

## 6. Monotonicity is the solver contract

A transfer function `F : A -> A` is monotone when:

```text
a ⊑ b  =>  F(a) ⊑ F(b)
```

Intuitively: giving the analyzer less precise input must not make it claim a more precise result that excludes behavior possible under the less precise input.

### Example: monotone assignment

Suppose expressions produce class sets. For:

```text
x = y
```

transfer is:

```text
F(σ) = σ[x ↦ σ(y)]
```

If `σ1 ⊑ σ2` pointwise, then `σ1(y) ⊑ σ2(y)` and therefore the updated `x` facts preserve the order.

### Example: a dangerous non-monotone heuristic

Imagine:

```text
if receiver has <= 3 candidate classes:
    resolve methods precisely
else:
    guess String because it is common
```

Adding possibilities can suddenly produce a narrower guessed result. This is non-monotone and unsuitable inside a fixed-point solver. An LSP may run such a heuristic outside the sound lattice as a separate presentation layer, but it must not contaminate the semantic solver.

### Monotonicity and destructive replacement

Incremental solvers sometimes replace a caller's contribution rather than monotonically accumulating it. This is valid if the *global solver algorithm* is designed for replacement and re-enqueues affected dependents. Do not confuse “the stored map entry can decrease” with “the semantic transformer is necessarily invalid.” However, if using Kleene-style ascending iteration, state updates must follow the assumed order or the convergence proof changes.

The current Phalcom LSP uses contribution-indexed parameter evidence precisely so a changed caller can remove its old contribution and recompute the join for the touched slot. That is an incremental maintenance operation around a semantic join, not a claim that old evidence remains forever.

## 7. Kleene iteration

For a monotone function over an appropriate domain, least-fixed-point iteration conceptually begins at bottom:

```text
x0 = ⊥
x1 = F(x0)
x2 = F(x1)
...
```

For CFG analysis with entry facts, the equation is more often:

```text
IN[entry] = Initial
IN[B]     = ⊔ OUT[p] for p ∈ pred(B)
OUT[B]    = transfer_B(IN[B])
```

A worklist avoids recomputing blocks whose inputs have not changed.

### Why finite height matters

If the domain has no infinite ascending chain, repeated strict growth must eventually stop. For a finite set of `N` possible class IDs, an unbounded powerset domain has finite height `N + 1` even though it may have `2^N` elements. A capped union domain has an even smaller height because it jumps to top after the cap.

Intervals have infinite ascending chains:

```text
[0,0] ⊑ [0,1] ⊑ [0,2] ⊑ ...
```

Naive iteration can fail to terminate. Use widening; see [widening-narrowing-and-termination.md](widening-narrowing-and-termination.md).

## 8. Worklist algorithms

A generic forward solver:

```text
for each block B:
    in[B]  = ⊥
    out[B] = ⊥
in[entry] = initial
worklist = [entry]

while worklist not empty:
    B = pop(worklist)
    new_in = if B == entry:
                 initial ⊔ join(out[P] for P in pred(B))
             else:
                 join(out[P] for P in pred(B))
    new_out = transfer_B(new_in)

    if new_in != in[B] or new_out != out[B]:
        in[B] = new_in
        out[B] = new_out
        push successors(B)
```

In an implementation, store only the states required by consumers. Some analyses can keep `OUT` only; edge-sensitive refinement may require edge states; sparse analyses may propagate facts only along def-use edges.

### Determinism

A correct monotone solver should converge to the same least fixed point regardless of fair worklist order, assuming exact joins and no schedule-dependent budget cutoffs. Still use deterministic queues (`VecDeque` plus a membership set, stable block IDs) in tests and editor code because:

- reproducibility simplifies debugging;
- provenance samples may be bounded and order-sensitive unless normalized;
- widening points can make order affect precision even when soundness remains;
- budgets can make schedule affect fallback precision.

If precision changes with schedule, document why.

## 9. Chaotic iteration and dependency scheduling

“Chaotic iteration” means repeatedly applying local transfer functions in any fair order until stability. A worklist is a practical chaotic-iteration strategy.

The key condition is fairness: if a fact that a node depends on changes, the node must eventually be revisited. Incremental analysis turns this into dependency maintenance:

```text
changed abstract product
   -> reverse dependency edges
   -> dirty consumers
   -> worklist
```

A stale dependent is a semantic bug even if every individual transfer is correct.

## 10. Product lattices

Real analyses track multiple independent components:

```text
State = Bindings × Constants × Presence × Effects
```

If each component is a lattice, define the product order and join pointwise:

```text
(a1, a2) ⊑ (b1, b2)
    iff a1 ⊑ b1 and a2 ⊑ b2

(a1, a2) ⊔ (b1, b2)
    = (a1 ⊔ b1, a2 ⊔ b2)
```

This is usually better than a combinatorial enum such as:

```text
IntConstantSomePure
IntUnknownSomeImpure
StringConstantMaybePure
...
```

A Rust representation might be:

```rust
#[derive(Clone, Eq, PartialEq)]
struct ValueFact {
    shape: ShapeFact,
    constant: ConstantFact,
    presence: PresenceFact,
}

#[derive(Clone, Eq, PartialEq)]
struct FlowState {
    bindings: BTreeMap<BindingId, ValueFact>,
    effects: EffectFact,
}
```

The `join` implementation should delegate to each component.

## 11. Reduced products

Independent product components may contain mutually refining information. A reduced product applies a reduction function after component operations:

```text
reduce : A × B -> A × B
```

Example:

```text
Interval(x) = [0,0]
Sign(x)     = {Negative, Zero, Positive}
```

Reduction can refine `Sign(x)` to `Zero`.

For Phalcom, a future explicit bridge might allow:

```text
ShapeFact: exact instance of String
TypeFact:  String | Dynamic
```

A trusted bridge can refine the type fact to `String` if the typing semantics allows that evidence. Do not make the reduction implicit across domain boundaries; otherwise an advisory `ValueShape` can silently acquire checker authority.

## 12. Map domains and default values

Flow states often map stable semantic IDs to abstract values:

```text
σ : BindingId -> ValueFact
```

You must define the meaning of a missing key. Common choices:

1. missing = `⊥`: no information/state exists for the binding;
2. missing = `⊤`: binding exists but value is completely unknown;
3. missing = “not in scope/not initialized,” a separate status.

Do not let `BTreeMap::get(...).unwrap_or_default()` accidentally decide semantics.

For local variables, “binding does not exist in state” and “binding exists but value is unknown” may need different treatment for definite assignment and editor recovery.

## 13. Semantic equality versus representation equality

Fixed-point checks need semantic equality. Representation differences that do not change meaning must be normalized away.

For a union domain:

```text
Union([Int, String])
Union([String, Int])
```

must compare semantically equal if alternative order has no semantic meaning.

Canonicalization strategies:

- sort by stable semantic ID;
- deduplicate alternatives;
- flatten nested unions;
- collapse singletons;
- normalize top/bottom forms;
- remove dominated alternatives when the domain has subtyping/inclusion.

Do not use pointer identity as semantic equality. `Arc::ptr_eq` can measure product reuse, but two independently allocated equal summaries are still semantically equal.

The current Phalcom LSP explicitly compares callable summaries while excluding publication generation/provenance that is not an invalidation input. This is the right distinction: semantic equality controls propagation; object identity measures reuse.

## 14. Fixed points in the current Phalcom semantic engine

At the 2026-08-14 repository baseline, two concrete fixed-point-like mechanisms are important:

### Structured loop flow

The LSP `FlowAnalyzer` maintains a loop header state, analyzes condition/body, joins back edges, checks equality, and after a bounded number of iterations widens loop state. This is CURRENT advisory analysis behavior. It is not a formal language typing rule.

Conceptually:

```text
H0 = Entry
Hi+1 = join(back_edges produced from Hi)
if Hi+1 == Hi: stable
if iteration budget reached: widen(Entry, Hi, Hi+1)
```

An implementation agent modifying this code must determine whether the current exit-state collection correctly represents zero iterations, break edges, condition-false exit, body normal exit, continue back edges, and abrupt completion for the specific construct.

### Callable/parameter propagation

The current incremental solver uses a deduplicating callable worklist, contribution-indexed parameter facts, call dependency edges, semantic-summary comparison, and a derived solver budget. Changed summary/parameter facts re-enqueue dependents. If the budget is exceeded, affected facts widen to `ValueShape::Unknown` so the advisory system can publish a coherent conservative result.

This machinery is a useful implementation precedent. A future checker/prover must decide independently whether the same domain height, budget, and `Unknown` fallback preserve its stronger correctness contract.

## 15. Failure modes

### Join accidentally depends on source order

If union alternatives preserve insertion order and equality compares the vector directly, two branch traversal orders may cause repeated “changes” or different snapshots. Canonicalize.

### Transfer narrows on less precise input

This violates monotonicity and can oscillate or exclude concrete behaviors. Separate heuristics from the sound transfer.

### `⊥` represented as `Unknown`

Unreachable predecessors then destroy precision at every merge and can make impossible diagnostics appear possible.

### Infinite ascending chain without widening

Solver never terminates or hits an arbitrary cap. Add a principled widening or finite abstraction.

### Budget fallback not an upper bound

Stopping iteration and returning the last state can be unsound if later iterations could add possibilities. Widen to a state known to cover remaining behavior.

### Revision included in semantic equality

Every rebuild appears changed, causing global invalidation even when analysis facts are identical.

### Provenance included in lattice order unintentionally

Adding a provenance source can look like semantic fact growth and trigger repeated solver propagation. Usually keep explanatory provenance outside the semantic equality/order, or define a separate bounded provenance lattice intentionally.

## 16. Testing obligations

For every domain, property-test:

```text
join(a, a) == a
join(a, b) == join(b, a)
join(join(a, b), c) == join(a, join(b, c))
join(bottom, a) == a
leq(a, join(a, b))
leq(b, join(a, b))
normalize(normalize(a)) == normalize(a)
```

Where possible, test monotonicity of transfer functions:

```text
if a ⊑ b:
    assert transfer(a) ⊑ transfer(b)
```

For widening:

```text
a ⊑ widen(a, b)
b ⊑ widen(a, b)
```

and construct chains that would otherwise be infinite.

For worklists, compare at least two scheduling orders against the same expected semantic result. For incremental solvers, compare the final result after an edit sequence against clean full recomputation.

## 17. Competency questions

An implementation agent should be able to answer:

1. What exact concrete-set interpretation does `a ⊑ b` have in this domain?
2. Is a missing map entry `⊥`, `⊤`, uninitialized, or something else?
3. Why is the branch merge a join for this property?
4. If the property is “definitely initialized,” why does the apparent set operation become intersection-like?
5. What ascending chains can the domain produce?
6. Is the domain finite-height? If not, where is widening applied?
7. Which transfer functions are monotone, and can you give a counterexample to a tempting non-monotone heuristic?
8. What is semantic equality for solver convergence, and which fields are deliberately excluded?
9. Can worklist order change only performance, or also precision because of widening/budgets?
10. Why can the current `ValueShape` bounded-union policy be valid for LSP inference but not automatically for the future checker?

If these questions cannot be answered, the lattice is not yet an implementation contract.
