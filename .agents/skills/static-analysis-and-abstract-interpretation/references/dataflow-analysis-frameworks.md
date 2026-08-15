# Dataflow Analysis Frameworks

Dataflow analysis turns local semantic transfer rules into whole-program-point facts over a control-flow graph or an equivalent structured-flow representation. This reference explains the algebra, solver algorithms, edge sensitivity, may/must distinctions, and the point at which a source-recursive analysis should become a shared CFG-based infrastructure.

## 1. Dataflow problems are systems of equations

For a forward analysis over basic block `B`:

```text
IN[B]  = ⊔ { OUT[P] | P ∈ pred(B) }
OUT[B] = F_B(IN[B])
```

where:

- `pred(B)` is the set of predecessor blocks;
- `IN[B]` is the abstract state immediately before `B`;
- `F_B` is the abstract transfer function for the block;
- `OUT[B]` is the state after the block;
- `⊔` is the merge operator induced by the abstract domain.

For a backward analysis:

```text
OUT[B] = ⊔ { IN[S] | S ∈ succ(B) }
IN[B]  = F_B(OUT[B])
```

Dataflow analysis is therefore not “walk the AST and remember facts.” A recursive AST traversal is an implementation technique only if it computes the same equations for the language's control flow.

## 2. Forward versus backward problems

### Forward examples

Information moves in execution direction:

- reaching definitions;
- constant propagation;
- runtime shape/value approximation;
- definite assignment in some formulations;
- points-to propagation;
- taint propagation from sources;
- may-effects accumulated along paths.

### Backward examples

Information moves opposite execution:

- liveness;
- available cleanup obligations;
- some demand analyses;
- weakest-precondition-like requirement propagation.

Direction is a property of the semantic question. Do not force all analyses into the existing forward-flow engine if the natural formulation is backward.

## 3. May versus must analysis

A may-analysis asks whether a property can hold on at least one execution reaching a point. A must-analysis asks whether it holds on every relevant execution.

### May examples

```text
receiver may be String
call may throw
field may be written
value may escape
```

At a branch merge, possible alternatives accumulate.

### Must examples

```text
binding definitely initialized
a resource definitely closed
x definitely refined to Some on all continuing paths
method definitely cannot yield under trusted summaries
```

At a merge, a fact survives only if every reachable predecessor establishes it.

### Worked diamond

```text
if cond {
  x = 1
} else {
  x = "s"
}
```

Possible shape after merge:

```text
MayShape(x) = {Int, String}
```

Definite property “x is Int”:

```text
MustInt(x) = false
```

Definite property “x was assigned on every path”:

```text
MustAssigned(x) = true
```

The same CFG supports multiple analyses with different domains and merge operators.

## 4. Reachability must participate in merges

Unreachable predecessors contribute bottom, not unknown.

```text
if cond {
  return
} else {
  x = 1
}
# only the else path continues
```

If the true branch has no normal successor, its state should not destroy the refinement/assignment facts of the only continuing branch.

In a structured flow representation, this often means:

```text
normal: Option<FlowState>
```

where `None` means no normal continuation. That pattern is CURRENT in Phalcom's LSP `StatementFlow`. It is an implementation representation of reachability—not a replacement for reasoning about all abrupt successors.

## 5. Worklist solving

A standard forward worklist algorithm:

```text
initialize IN/OUT to bottom
seed entry
queue entry

while queue not empty:
    B = pop()
    merged = join(OUT[P] for P in pred(B))
    new_out = transfer_B(merged)

    if new_out != OUT[B]:
        OUT[B] = new_out
        enqueue successors(B)
```

For edge-sensitive facts, calculate successor-specific edge states:

```text
edge_state[B -> S] = refine(B, S, OUT[B])
IN[S] = join(edge_state[P -> S] for P in pred(S))
```

### Rust representation sketch

```rust
struct BlockId(u32);

struct DataflowState<D> {
    in_state: Vec<D>,
    out_state: Vec<D>,
    queued: BitSet<BlockId>,
    queue: VecDeque<BlockId>,
}
```

For a small semantic engine, `BTreeMap<BlockId, D>` is easier to debug. For dense stable block IDs and hot analyses, indexed `Vec`/bitsets are preferable. Representation choice does not change the semantic equations.

## 6. Fairness, determinism, and scheduling

A worklist schedule is fair if a block whose input may have changed is eventually processed. For monotone finite-height analysis without widening, any fair schedule reaches the same least fixed point.

Use deterministic scheduling anyway:

- test snapshots remain stable;
- iteration counts are reproducible;
- bounded provenance samples do not vary unexpectedly;
- widening points can otherwise produce schedule-dependent precision;
- budget-limited analyses should degrade predictably.

Phalcom's current callable worklist uses `VecDeque` plus a `BTreeSet` for deduplicated deterministic insertion order. Similar design is appropriate for CFG block worklists.

## 7. Gen/kill bitvector frameworks

Classic dataflow problems can be expressed:

```text
OUT[B] = GEN[B] ∪ (IN[B] - KILL[B])
```

Examples include reaching definitions and variants of available expressions/liveness.

Bitvector frameworks are still highly relevant for Phalcom when facts are sets of stable semantic IDs:

- live bindings;
- definitely assigned IDs;
- reachable definitions;
- escaping allocation IDs;
- effect categories;
- dirty dependencies.

Do not use heavyweight per-value abstract states for a problem naturally represented by bitsets.

## 8. Monotone frameworks

A monotone framework consists of:

```text
CFG
lattice L
initial/boundary value
transfer functions F_B : L -> L
merge operator
```

with monotone transfers. Finite-height or widening ensures termination.

### Distributivity

A transfer is distributive over join if:

```text
F(a ⊔ b) = F(a) ⊔ F(b)
```

Distributivity is stronger than monotonicity. Some classical frameworks exploit it for maximal precision within the abstraction. Rich value/heap analyses often remain monotone but are not distributive.

Do not assume distributivity when proving solver equivalence or considering IFDS-style algorithms.

## 9. MOP versus MFP

The meet-over-all-paths (or join-over-all-paths depending on convention) solution conceptually applies transfer along every concrete CFG path and then merges:

```text
MOP[p] = ⊔ { F_path(initial) | path reaches p }
```

The maximal fixed-point/least fixed-point solution computed by a monotone dataflow solver is often called MFP in classic literature. For distributive frameworks, MFP equals MOP; for merely monotone frameworks, fixed-point solutions may be less precise.

This distinction matters when someone claims a dataflow solver is “path exact.” It usually is not. It computes an abstraction whose joins can lose path correlations.

## 10. Edge-sensitive branch refinement

A condition often has different transfer on its two edges.

Suppose:

```text
x.isExactly(String).ifTrue || {
  x.uppercase()
}
```

If `isExactly` is a trusted runtime-class test, analysis can refine:

```text
true edge:  Shape(x) = intersect(Shape(x), {String})
false edge: Shape(x) = remove(Shape(x), {String})
```

The refinement belongs to the branch edge, not globally to the condition expression. If the true branch returns, the false/normal edge may be the only state reaching the join and its refinement can survive.

The current Phalcom structured flow already recognizes trusted `is`/`isExactly` tests and produces branch-specific state refinement. Future analyses should preserve the principle while deciding whether to centralize such control semantics in a CFG/HIR.

## 11. Exceptional and abrupt control edges

A sound CFG/dataflow model includes every control transfer relevant to the property:

```text
normal successor
return exit
throw/exceptional successor
break target
continue target
non-local return target
cleanup/finally/defer edge if language has one
fiber cancellation edge if introduced
```

If an operation can invoke user code, that user code may itself throw, return non-locally, mutate state, or yield. Whether those effects need explicit CFG edges or summarized effects depends on the analysis.

### Example: definite cleanup

If a resource must be closed, ignoring exceptional exits can falsely conclude that every path closes it. The normal CFG may be correct for completion but insufficient for a checker/prover/lint with abrupt-exit obligations.

## 12. Structured flow versus explicit CFG

Phalcom's CURRENT LSP analysis uses structured statement flow, with a result that can carry normal state, returns, breaks, continues, throws, and a tail value. This is a reasonable design for a source-oriented advisory analyzer because control constructs are recognized during recursive traversal, source ranges remain direct, and malformed/incomplete source can be handled without a separate lowering pipeline.

An explicit shared CFG becomes justified when several consumers need the same control facts or structured recursion becomes duplicated/fragile.

### Strong signs that a CFG/HIR is warranted

- linter, checker, prover, and optimizer each implement their own loop/branch reachability;
- dominance/post-dominance is needed;
- liveness or sparse def-use becomes important;
- exception/non-local-return edges need uniform modeling;
- proof/path analysis needs stable program-point identities;
- optimization requires value versions or SSA;
- incremental analysis would benefit from block/callable-granular dependencies;
- source constructs desugar to the same semantic control pattern and repeated handling becomes inconsistent.

Do not introduce multiple incompatible CFGs. Prefer one semantic control representation with consumer-specific domains.

## 13. Block granularity and program points

Facts can be associated with:

- basic-block entry/exit;
- statement before/after points;
- expression points;
- edges;
- semantic operations in a lowered HIR.

Choose the coarsest granularity that supports the consumers and diagnostics.

For LSP hover on a local before a particular assignment, per-binding source-order facts may be sufficient. For definite assignment or prover conditions, stable before/after program-point IDs are more robust.

Program points should have stable semantic identity within a source generation and source provenance for rendering. Byte offsets alone are poor durable IDs across edits.

## 14. Sparse dataflow

Dense CFG analysis propagates state through every block. Sparse analysis uses def-use or SSA edges so facts flow only where relevant.

Sparse Conditional Constant Propagation (SCCP) combines:

- executable-edge discovery;
- SSA value lattice propagation.

This can be dramatically faster for optimizer-oriented constant analysis. It is unnecessary for early LSP shape inference if source-flow traversal already meets latency budgets.

Move to sparse forms when profiling shows repeated dense propagation or when SSA already exists for other reasons.

## 15. Interprocedural dataflow

A CFG solver handles one callable unless extended with call/return edges. Whole-program interprocedural analysis has several approaches:

1. summarize callees and apply summaries at call sites;
2. build a supergraph with call/return edges;
3. use specialized frameworks such as IFDS/IDE for suitable distributive problems;
4. context-sensitive summary solving.

Phalcom's dynamic dispatch and higher-order blocks make generic supergraph construction expensive and uncertain. Summary-based analysis is the practical default; see [interprocedural-analysis-and-call-graphs.md](interprocedural-analysis-and-call-graphs.md).

## 16. IFDS/IDE: use only when the problem fits

IFDS solves interprocedural, finite, distributive subset problems efficiently through graph reachability. IDE extends to environments mapping facts to values.

Good candidate properties:

- taint facts with finite fact sets;
- some typestate/dataflow properties;
- reachability of semantic facts.

Poor candidate without reformulation:

- rich unbounded interval/polyhedral domains;
- arbitrary dynamic heap abstractions;
- non-distributive joins of complex Phalcom `ValueShape` states.

Do not introduce IFDS because it is sophisticated. First prove the problem matches its assumptions.

## 17. Loop equations

For a simple loop:

```text
Entry -> Header -> Body -> Header
                  \-> Exit
```

a forward may equation is:

```text
IN[Header] = EntryState ⊔ OUT[BodyBackEdge]
```

The zero-iteration possibility is represented through the initial entry contribution and/or condition-false edge to exit.

For `continue`, the state goes to the loop's continuation/header point. For `break`, it contributes to the loop exit. For `return`/`throw`, it does not contribute normal loop exit unless the language semantics says so.

A correct structured implementation must compute equivalent merges.

## 18. Phalcom current flow mechanics

At repository baseline `b5477b74…`, CURRENT LSP flow includes:

- `FlowState` keyed by lexical `BindingId`;
- `StatementFlow.normal: Option<FlowState>` for normal reachability;
- collected `returns`, `breaks`, `continues`, and `throws`;
- control-expression recognition for `ifTrue`, `ifFalse`, `ifTrue:ifFalse`, `whileTrue`, and short-circuit boolean forms;
- trusted type-test refinement for `is`/`isExactly`;
- bounded loop iteration with equality check and widening;
- `for` loop fixed-point-style iteration;
- event emission for calls/field writes;
- block-effect extraction and higher-order parameter invocation tracking.

This is substantial current machinery. An agent should first decide whether a requested analysis can share/extend it or whether requirements such as common dominance, backward flow, exceptional edges, or proof program points justify a reusable CFG/HIR.

## 19. Failure modes

### Last-visited branch wins

An AST visitor mutates one shared map through both branches and retains whichever arm ran second. Result depends on traversal order. Fix: branch-local states plus domain merge.

### Ignoring zero iterations

Loop exit includes only body output. Fix: include entry/condition-false path and solve header equation.

### Treating unreachable as unknown

A terminating branch weakens facts at merge. Fix: represent absence of normal successor separately/bottom.

### Missing exceptional edge

A must-property appears established because throw paths are ignored. Fix: include/summarize abrupt exits relevant to property.

### Applying branch refinement before evaluating condition effects

If the condition invokes user code that mutates relevant state, refinement may be based on stale state. Preserve actual evaluation order and effects.

### Duplicated control semantics

Checker and LSP disagree because each recognizes a different set of control selectors. Fix: shared semantic lowering/control model or shared control-semantic helpers.

### Dataflow state keyed by source name

Shadowed bindings collide. Fix: use `BindingId` or equivalent semantic identity.

## 20. Testing obligations

### Canonical CFG fixtures

Test at least:

```text
straight line
diamond
one branch terminates
nested diamonds
zero-iteration loop
loop needing >1 iteration for stability
break
continue
nested loops
return in loop
throw path
short-circuit boolean
block non-local return
```

### May/must differentiation

Use the same CFG with two analyses and verify different merge behavior.

### Traversal/schedule invariance

Where theory says results should be schedule-independent, randomize worklist ordering in tests and compare semantic facts.

### Incremental equivalence

Edit a predecessor, add/remove a branch, change a loop body, then assert incremental result equals clean full analysis.

### Malformed source

For editor-oriented flow, test incomplete constructs. Recovery facts must not fabricate valid complete-program semantics; unaffected scopes/functions should remain queryable.

## 21. Review questions

1. What are nodes and edges in the analysis control representation?
2. Which abrupt transfers exist, and which are intentionally summarized?
3. Is this a forward or backward problem?
4. Is it may or must?
5. What is the merge operator and why?
6. How is unreachable represented?
7. Which predicates refine edges?
8. Can evaluating the condition invoke user code or mutate facts?
9. Where do loops form fixed-point equations?
10. Does structured flow still compute the required equations, or is a shared CFG now justified?
11. Can the analysis become sparse over stable def-use IDs?
12. Does incremental invalidation dirty exactly the blocks/callables whose equation inputs changed?

A correct answer should connect control semantics to algebra, not merely point to a visitor method.
