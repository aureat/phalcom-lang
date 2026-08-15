# Control Flow, Semantic IR, Dataflow, and Fixed Points

## 1. Why explicit control flow matters

Flow-sensitive semantics is not “walk statements in order and mutate a map.” Branches, loops, exceptions, `break`/`continue`, returns, non-local returns, closures, and future concurrency create multiple incoming states and cycles. Once multiple analyses need these distinctions, use an explicit control-flow graph (CFG) or a structured representation with equivalent, rigorously defined join/back-edge behavior.

A CFG for a callable is `G = (B, E, entry, exits)` where `B` is a set of basic blocks and `E` directed control edges. Program points identify positions before/after operations where facts can be queried.

## 2. Forward dataflow equations

For a forward may-analysis:

```text
IN[B]  = ⊔ { OUT[P] | P ∈ pred(B) }
OUT[B] = F_B(IN[B])
```

`⊔` is the least upper bound (join) in the abstract domain and `F_B` the block transfer function. To use standard fixed-point iteration safely:

- the abstract states form a partial order `⊑`;
- joins are well-defined;
- transfer functions are monotone: `a ⊑ b => F(a) ⊑ F(b)`;
- either the domain has finite ascending chains or widening/budgets intentionally force termination.

A worklist algorithm:

```text
for B: IN[B] = ⊥; OUT[B] = ⊥
IN[entry] = initial_state
worklist = [entry]

while worklist not empty:
    B = pop(worklist)
    new_in  = join(OUT[P] for P in pred(B))  // entry also receives initial_state
    new_out = transfer_B(new_in)
    if new_out != OUT[B]:
        IN[B] = new_in
        OUT[B] = new_out
        push successors(B)
```

Do not stop after an arbitrary fixed number of rounds and call the result solved. If a budget is needed for editor latency, return a distinct budget-exhausted/approximated state and widen conservatively.

## 3. Bottom, unknown, and unreachable are different

`⊥` usually represents no reachable concrete states. `Unknown` often means reachable but analysis lacks useful value knowledge. They must not be equal.

Example:

```phalcom
return 1
x = mystery()
```

The state before `x = ...` is unreachable (`⊥`), not a reachable state where `x` has unknown shape. This distinction affects diagnostics, joins, proof obligations, and dead-code analysis.

A binding map domain may use:

```text
State = Unreachable | Reachable(Map<BindingId, AbstractValue>)
```

rather than encoding unreachable as “empty map.”

## 4. Worked loop example

Consider:

```phalcom
let x = 0
while cond() {
  x = next(x)
}
use(x)
```

The loop header receives both preheader and back-edge states:

```text
IN[header] = OUT[preheader] ⊔ OUT[body_backedge]
```

If the domain tracks exact integers and `next` increments, the ascending chain `0, {0,1}, {0,1,2}, ...` may not terminate. A bounded domain must widen, e.g. exact constant -> integer-like/runtime class shape -> unknown/numeric interval depending on analysis.

For current Phalcom `ValueShape`, a bounded union can widen to `Unknown` after its cap. That is an implementation precision policy, not a proof that all values are possible, and not a formal type rule.

## 5. May versus must analyses

The lattice operation depends on the question.

- **May analysis:** “may this binding have shape A?” joins alternatives by union.
- **Must analysis:** “is this binding definitely assigned?” joins by intersection of definitely assigned sets.

Example definite-assignment domain:

```text
D = powerset(Bindings)
order: reverse? choose convention carefully
join at branch merge: intersection
transfer assignment x: S -> S ∪ {x}
```

Do not reuse the same `join` because both analyses happen to use sets.

## 6. Conditions and refinements

For a condition `c`, construct true/false successor transfers:

```text
T_true(c, state)
T_false(c, state)
```

A trusted nominal type test might refine receiver candidates differently on each edge. A truthiness test may refine known booleans/null-like values according to Phalcom semantics. Refinement must be justified by the actual dynamic predicate; syntactic resemblance to another language is insufficient.

Path sensitivity is expensive. Prefer branch-local refinements merged at dominator/join points rather than preserving arbitrary path formulas in the base semantic engine. The prover can own richer path conditions.

## 7. Abrupt completion as control edges

A statement result should distinguish normal continuation and abrupt exits. A convenient conceptual type:

```text
FlowResult = {
  normal: Option<State>,
  returns: Vec<(TargetCallable, State, Value)>,
  breaks: Vec<(LoopId, State)>,
  continues: Vec<(LoopId, State)>,
  throws: ThrowSummary,
  nonlocal_returns: ...
}
```

**CURRENT:** `flow.rs` already has `StatementFlow` with normal state, returns, breaks, continues, throws, and tail value, plus block effects for non-local returns/captured writes. This structured model is an important current bridge toward explicit CFG semantics.

When CFG becomes shared infrastructure, represent abrupt outcomes as edges to dedicated exit/handler blocks rather than ad hoc booleans where precision matters.

## 8. Closures and execution timing

A literal block body should not be merged into current flow merely because it appears syntactically. Construction creates a closure/capture fact; invocation executes body effects.

For a block passed to a known callable, effect summaries can model “parameter `i` is invoked” and propagate block effects. **CURRENT:** Phalcom callable summary effects already include dynamic-send and invoked-parameter information, and `BlockEffects` tracks non-local returns, captured writes, invoked parameters, and dynamic sends. This is a strong foundation, but it remains an approximation whose soundness boundaries must be documented.

## 9. Exceptions and future effect edges

A boolean `throws` is sufficient only for analyses that ask “can abrupt throw occur?” A checker/prover/effect system may require richer thrown-class/effect facts and handler edges.

Do not prematurely model every send as a precise exception set. First define Phalcom's normative error/exception semantics and dynamic-send boundary. A conservative call may have `MayThrow`/`MayInvokeUnknownCode` effects.

## 10. Building the CFG

Useful block boundaries include:

- entry and normal exit;
- branch targets and merge points;
- loop header/body/latch/exit;
- before/after potentially abrupt operations if handlers/refinements need them;
- return/non-local return exits;
- pattern match success/failure;
- short-circuit operands;
- possibly yield/suspend points for future fiber-aware analyses.

Preserve source origins on operations and edges. A CFG that loses authored ranges is hostile to diagnostics and refactoring.

## 11. Dominance and SSA: optional, not automatic

Dominators answer whether every path to `B` passes through `A`. They aid reaching definitions, refinement validity, and SSA construction. SSA can simplify def-use and optimization but is not required for basic semantic analysis.

Do not introduce SSA solely to implement hover. Consider it when optimizer/prover/dataflow consumers repeatedly need versioned definitions and phi-like joins. Mutable captured variables and reflective storage make full-language SSA modeling more complex; memory SSA/alias abstraction may be necessary for heap state.

## 12. Verification properties

Tests should include:

- branch joins with different shapes;
- unreachable branches after return/throw;
- loop requiring more than one iteration to stabilize;
- widening threshold behavior;
- nested `break`/`continue` ownership;
- short-circuit evaluation order;
- closure construction versus invocation;
- captured write joined after known invocation;
- non-local return destination;
- recursion through summaries;
- malformed body represented without CFG panic.

Metamorphic property: incremental and full CFG/dataflow analysis of identical final source yield equal observable facts.

## 13. Review questions

1. What is the domain and partial order?
2. What exactly do `⊥` and top/unknown mean?
3. Is this a may or must analysis?
4. Are transfers monotone?
5. What guarantees termination around cycles?
6. Where does widening lose precision, and is that loss surfaced?
7. Are normal and abrupt completion separated?
8. Are closure bodies analyzed conditionally on invocation?
9. Does the representation preserve enough source/provenance for diagnostics?
10. Is an explicit CFG now simpler than several structured visitors reimplementing the same joins?
