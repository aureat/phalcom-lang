# Pyrefly constraint solving and fixed points

## Scope

This document reconstructs Pyrefly's solving machinery at implementation level. It separates two cooperating mechanisms:

1. **Answer solving** calculates the type or fact for a binding, tracks dependencies, detects recursive binding SCCs, and publishes answers.
2. **Type-relation solving** checks subset/subtyping, unifies solver variables, accumulates bounds, handles overload/protocol transactions, and asks the semantic database for facts through TypeOrder.

Pyrefly's performance comes from composing these mechanisms. The answer solver does not reimplement every type relation, and the subset solver does not own module lookup or binding-graph traversal.

## Evidence boundary

Pyrefly observations are pinned to commit 43467e64e36550f232a18e89f24fda79b1020b6b, inspected 2026-08-22.

Primary source files:

- [answers_solver.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/alt/answers_solver.rs) — binding calculation stack, SCCs, placeholders, iterations, demotion, and answer commits.
- [answers.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/alt/answers.rs) — answer table, answer slots, indexes, traces, and solutions.
- [solver.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/solver/solver.rs) — inference variables, bounds, snapshots, pinning, and subset cache.
- [subset.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/solver/subset.rs) — structural/nominal subset algorithm and recursive relation cache.
- [type_order.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/pyrefly/lib/solver/type_order.rs) — boundary from relation solving into module/class/protocol facts.
- [calculation.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_graph/src/calculation.rs) — lower-level cached calculation cell and same-thread cycle behavior.
- [types.rs](https://github.com/facebook/pyrefly/blob/43467e64e36550f232a18e89f24fda79b1020b6b/crates/pyrefly_types/src/types.rs) — Type::Var, type forms, and recursive type operations.

Relevant local Phalcom seeds:

- phalcom-semantic/src/types/constraint.rs
- phalcom-semantic/src/types/relation.rs
- phalcom-semantic/src/types/evidence.rs
- phalcom-semantic/src/types/store.rs
- phalcom-semantic/src/checker/
- phalcom-lsp/src/semantic/infer.rs

All Phalcom statements below are recommendations unless marked CURRENT or EXPERIMENTAL.

## The real execution path

At module level, Pyrefly builds Bindings and Answers during the Answers step, then turns those answers into final Solutions during the Solutions step. The concrete sequence is:

~~~text
step_answers
  -> create Solver
  -> Bindings::new(AST, exports, solver, lookup, configuration)
  -> Answers::new(bindings, solver, index/trace policy)

step_solutions
  -> Answers::solve(lookup, bindings, errors, stdlib, uniques, limits)
  -> demand binding calculations
  -> AnswersSolver drives calculations
  -> SCC-local answers converge
  -> final answers and side effects commit
  -> Solutions expose stable consumer-facing facts
~~~

The solver is demand-driven inside a module, but the module itself is solved as a complete unit. Pyrefly therefore avoids both extremes: it does not build one globally persistent query for every identifier, and it does not eagerly evaluate every semantic fact.

## Layer 1: calculation identity

The alternate answer solver uses CalcId, conceptually:

~~~text
CalcId = (module bindings identity, binding-table index)
~~~

A binding-table index points to a semantic key/value entry. The key identifies the operation: definition, use, anonymous statement, export, class field, function, type alias, yield, or another solver-relevant fact.

This identity is cheaper than carrying AST nodes or names through the solver:

- dependency edges are compact IDs;
- the calculation stack stores IDs;
- SCC membership is a set of IDs;
- answer tables index by IDs;
- source ranges are recovered only when diagnostics need them;
- equivalent solver requests can share a calculation slot.

The source architecture separates the key from the value. A use key may refer to another binding key; an export key may refer across a module boundary; an anonymous statement can be checked without retaining a result for later name lookup.

### Phalcom transfer

Phalcom should introduce an equivalent identity with explicit ownership:

~~~rust
struct CalcId {
    module: ModuleId,
    binding: BindingId,
}
~~~

If a dense generation-local index is added, wrap it:

~~~rust
struct BindingCalcIndex {
    snapshot: SemanticGeneration,
    index: u32,
}
~~~

Do not use a source byte offset as the only durable identity. Offsets are occurrence coordinates, not stable semantic ownership across reparsing and edits.

## Layer 2: calculation stack and cycle detection

The answer solver keeps a calculation stack and a reverse position map. When a calculation requests another binding:

~~~text
request(target)
  if target has a final answer in the current/previous table:
      return the answer
  if target is not on the active stack:
      push target and calculate
  else:
      identify stack segment from the prior target to the top
      create or merge an SCC
      return a recursive placeholder or prior answer
~~~

The stack is not merely a recursion guard. A set can say that a cycle exists; the stack position says which calculations must be solved together.

The reverse position map avoids scanning the entire stack for every recursive lookup. It turns a back-edge into an SCC segment cheaply. The map must be updated whenever frames are pushed, popped, or merged into an SCC.

### Phalcom rule

Do not implement recursion as one global depth counter. A depth limit cannot distinguish repeated non-recursive lookup, self-recursion, mutual recursion, SCC membership growth, or a query blocked on an external module. Use a per-solver stack and typed calculation IDs.

## Layer 3: SCC state machine

Pyrefly tracks node states similar to:

~~~text
Fresh
InProgress
HasPlaceholder(Var)
Done { errors, traces }
~~~

SCC-level state tracks:

~~~text
members                  calculation IDs
detected_at              stable cycle anchor
owner                    which driver owns iterative solving
iteration                current fixed-point round
current_answers          answers produced in this round
previous_answers         answers from prior round
has_changed              semantic answer-change flag
recursion_breaks         nodes where a prior/placeholder answer was used
needs_demotion           membership grew and a cold restart is required
~~~

This is more precise than “recursive equals unknown.” A recursive region can converge to a literal, nominal type, callable summary, or stable union.

### Why Done carries side effects separately

A node's answer and its errors/traces have different lifetime concerns:

- the answer is semantic state;
- an error may be suppressed or reported only after the SCC commits;
- a trace is useful for demand/debug output but should not escape from an abandoned attempt.

Pyrefly keeps these side effects attached to SCC-local node state until final commit. This avoids stale diagnostics from speculative or superseded calculations.

Phalcom should use the same separation:

~~~rust
struct PendingFact {
    value: TypeTerm,
    dependencies: SmallVec<[DependencyKey; 4]>,
    evidence: EvidenceSet,
    diagnostics: Vec<Diagnostic>,
    trace: Option<TraceFragment>,
}

struct SccNodeState {
    status: SccNodeStatus,
    pending: Option<PendingFact>,
}
~~~

## Layer 4: recursive placeholders

When a binding is re-entered during its own calculation, Pyrefly creates a Type::Var placeholder. The placeholder is a finite name for an unfinished recursive answer.

For the architecture example:

~~~text
x = 1
while test():
    x = x
print(x)
~~~

The binding graph contains a phi-like equation:

~~~text
x_after_loop = phi(Literal[1], x_after_loop)
~~~

Calculation proceeds conceptually:

~~~text
solve(x_after_loop)
  -> solve(Literal[1]) = Literal[1]
  -> solve(x_after_loop) re-enters
  -> return ?1
  -> record ?1 = Literal[1] | ?1
  -> take the upper reachable bound of ?1
  -> resolve ?1 to Literal[1]
  -> simplify x_after_loop to Literal[1]
~~~

The placeholder has an owner and a relation to the recursive binding. It is not a global unknown value.

### Phalcom representation

Phalcom has InferVarId in its experimental type identity layer. Use it as a solver identity, not as a canonical persistent type:

~~~rust
enum RecursiveAnswer {
    Placeholder(InferVarId),
    Previous(TypeId),
    Widened(TypeId, WideningReason),
}

struct RecursiveVar {
    id: InferVarId,
    owner: CalcId,
    origin: SourceOrigin,
}
~~~

The checker must preserve recursive status and evidence. Dynamic is not an appropriate default for a recursive placeholder.

## Layer 5: current and previous answers

Every warm SCC iteration has two answer generations:

~~~text
previous = answers from iteration n - 1
current  = answers being built in iteration n
~~~

On a recursive read:

- use a previous answer when one exists;
- otherwise create a fresh placeholder;
- record a recursion break;
- after the round, compare current and previous answers semantically.

Pyrefly does not publish every tentative answer to the global answer table. SCC-local state isolates provisional values until the fixed point is stable enough to commit. This prevents one member from observing a half-updated answer from another member.

Phalcom equivalent:

~~~rust
struct SccIteration {
    current: IndexMap<BindingId, TypeTerm>,
    previous: Option<Arc<IndexMap<BindingId, TypeTerm>>>,
    changed: bool,
    recursion_breaks: SmallSet<BindingId>,
}
~~~

Changed must use semantic equality, not pointer identity. It should ignore publication generation and irrelevant diagnostic provenance while preserving meaningful type, flow, and evidence differences.

## Layer 6: SCC membership expansion and demotion

A cycle can discover another active calculation that belongs to the same recursive region. Pyrefly merges SCCs or absorbs stack members. Existing tentative answers are then based on incomplete membership.

Its algorithm is:

~~~text
Phase 0: discover SCC members
iteration 1: cold solve all members
if membership expands:
    mark NeedsDemotion
    discard iteration answers
    restart cold with expanded membership
else if answers changed:
    advance current -> previous
    reset node states to Fresh
    warm iteration
else:
    converge and commit
~~~

At the pinned revision, the iterative SCC solver uses a maximum of five fixpoint iterations and ten demotion restarts. Exceeding the iteration limit reports a type error while committing the last approximate answers; exceeding the demotion limit is treated as an internal infinite-membership expansion failure.

These are Pyrefly policies, not universal type-theory rules. Phalcom should distinguish semantic non-convergence, resource budget exhaustion, implementation invariant violation, and cancellation.

Recommended Phalcom result:

~~~rust
enum FixpointStatus {
    Converged,
    Widened { reason: WideningReason },
    BudgetExceeded { budget: SolverBudget },
    RecursiveUnresolved { cycle: CycleId },
    Cancelled,
    InternalInvariantFailure,
}
~~~

Normal checker behavior should convert non-convergence into an explicit analysis result, not panic.

## Layer 7: batch commit

When an SCC converges, Pyrefly commits answers in a batch. The final commit pairs:

~~~text
CalcId
  + canonical answer
  + deferred errors
  + deferred traces
  + dependency information
~~~

The batch boundary prevents abandoned or intermediate calculations from leaking diagnostics. First-answer-wins behavior prevents duplicate stack frames from overwriting the canonical answer for one SCC member.

Phalcom should publish a complete fact:

~~~rust
struct CommittedFact {
    value: TypeId,
    evidence: EvidenceSet,
    dependencies: SmallVec<[DependencyKey; 4]>,
    diagnostics: Arc<[Diagnostic]>,
    semantic_revision: SemanticRevision,
}
~~~

Publish the fact at a query/snapshot boundary. Do not publish a TypeId first and attach evidence later.

## Layer 8: solver variables

The Solver owns mutable inference-variable state. The important separation is:

~~~text
Type                 persistent/structural semantic value
Var                  solver identity
VariableNode         unification/graph node
Variable             answer, quantified state, partial state, bounds, residual
Bounds               lower and upper obligations
Solver               variable store, caches, policy, type heap
~~~

The implementation supports:

- fresh variables;
- answer variables;
- partial/first-use variables;
- quantified variables;
- lower and upper bounds;
- unification and variable merging;
- snapshots/restoration for speculative overload or branch checks;
- sanitization/freezing before values cross calculation boundaries;
- subset and consistency checks;
- bounded expansion of unresolved variables.

The performance consequence is important: a type tree is not copied for every new constraint. A variable accumulates bounds in a compact mutable node; a canonical type is materialized at a boundary.

## Bound accumulation

A simplified bound equation is:

~~~text
lower bounds:  L1 <: T, L2 <: T
upper bounds:  T <: U1, T <: U2

candidate lower = join(L1, L2, ...)
candidate upper = meet(U1, U2, ...)
~~~

Pyrefly also handles gradual types, callable residuals, quantified variables, partial containers, and special forms. It filters broad Any bounds when a more precise bound exists so Any does not contaminate every result.

Phalcom must specify:

- whether lower-bound join is a type LUB or a separate inference operation;
- whether Dynamic and Unknown are retained or filtered;
- whether Never means bottom or missing information;
- whether an underconstrained variable remains a variable;
- how recursive bounds are guarded;
- which relation removes redundant bounds.

Do not implement solve-bounds as an arbitrary “union everything” function.

## Speculative solving: snapshots and rollback

Overload pruning and quantified finishing need to ask whether a candidate works without permanently mutating solver state. Pyrefly snapshots:

- selected variable states;
- instantiation errors;
- subset cache;
- protocol-cycle assumptions;
- residual/deferred-variable context.

Then it probes the candidate. A failed probe restores the complete snapshot. This is a transaction, not merely a clone of one substitution map.

Phalcom call checking should expose the same semantic shape:

~~~rust
enum ProbeResult {
    Accept { delta: SolverDelta },
    Reject { reason: ConstraintFailure },
    Unknown { reason: UnknownReason },
}

struct SolverCheckpoint {
    variables: VarSnapshot,
    relation_cache: RelationCacheSnapshot,
    errors: ErrorSnapshot,
    effects: EffectSnapshot,
}
~~~

The first implementation may clone a small solver state. Undo logs or persistent maps are later optimizations.

## Subset solver and relation cache

Subset is the algorithmic relation engine. Its query-local cache is conceptually keyed by:

~~~text
(got type, wanted type, subset context)
~~~

Entries are:

~~~text
InProgress
Ok
Err(error)
~~~

InProgress breaks recursive protocol and alias checks coinductively. Successful entries persist for the query so sibling paths do not repeat the same structural comparison. On failure, entries created during the failed computation can be rolled back because they may depend on an assumption that later failed.

This is a major efficiency mechanism. A recursion stack detects a cycle but still recomputes repeated pairs on sibling paths. A persistent query-local relation cache prevents exponential repetition while preserving rollback soundness.

Phalcom rules:

- query-local cache may contain recursive assumptions;
- cross-query cache may contain only revision-stable, inference-variable-free relations;
- cache keys include semantic policy and relevant surface revision;
- Unknown and blocked results retain their dependency reason.

## TypeOrder: relation solving does not own lookup

A Type value does not contain enough information to determine every relation. The subset solver asks TypeOrder for superclass/MRO information, protocol membership, required members, metaclass facts, variance, constructor callability, typed-dict fields, aliases, special class behavior, and module answers.

TypeOrder is a narrow wrapper around the answer solver. The relation algorithm does not navigate module state directly.

Phalcom equivalent:

~~~rust
trait SemanticOrder {
    fn nominal_parents(&self, ty: TypeId) -> RelationResult<Arc<[TypeId]>>;
    fn member(&self, receiver: TypeId, selector: Selector) -> DispatchResult;
    fn callable(&self, ty: TypeId) -> CallableFacts;
    fn variance(&self, kind: KindId) -> Variance;
    fn protocol_members(&self, protocol: DeclarationId) -> MemberSet;
    fn constructor(&self, class: DeclarationId) -> CallableFacts;
}
~~~

This boundary preserves Phalcom semantics: message sends, class/instance side, family selectors, reflection, native contracts, and open-world dispatch.

## Termination and gas

Pyrefly uses gas/budget checks in subset solving and additional limits around recursive type expansion and SCC iterations. The important mechanism is that expensive recursive work checks a budget at the semantic relation boundary.

Phalcom should make budgets explicit:

~~~rust
struct SolverBudget {
    relation_steps: u32,
    worklist_steps: u32,
    scc_iterations: u32,
    type_depth: u32,
    union_members: u32,
    allocations: usize,
}
~~~

Budget exhaustion must preserve the failed proof obligation. It must not yield a trusted type merely because the solver stopped.

## Why this solver is fast

The performance chain is:

1. CalcId avoids carrying syntax through the solver.
2. Binding answers are demanded, not recursively expanded everywhere.
3. Same-thread cycles become placeholders.
4. SCCs solve related recursive calculations together.
5. Previous answers warm-start later iterations.
6. SCC-local answers prevent partial global visibility.
7. Semantic equality decides convergence without comparing irrelevant metadata.
8. Subset relation pairs are memoized within a query.
9. Speculative branches are reversible.
10. Budgets cap pathological graphs.
11. Stable IDs reduce hashing and allocation cost.
12. Module-level scheduling avoids a global fine-grained dependency engine.

Removing one boundary causes a recognizable failure: recomputation, stale answers, infinite recursion, memory growth, or unsound speculative state.

## What Phalcom should transfer

### Transfer directly as architecture

- typed calculation IDs;
- stack-based SCC discovery;
- placeholders for recursive facts;
- current/previous answer generations;
- SCC-local batch commit;
- query-local coinductive relation cache;
- speculative solver snapshots;
- narrow semantic-order interface;
- explicit budgets and convergence outcomes;
- semantic equality separate from provenance and publication identity.

### Adapt

- Type::Var becomes InferVarId plus canonical TypeId;
- Python binding keys become Phalcom definitions, uses, sends, members, and exports;
- Python subset/protocol rules become Phalcom subtype, conformance, assignability, and dispatch relations;
- Python Any fallback becomes Phalcom Dynamic/Unknown policy;
- Python module SCCs must include reflection, method mutation, family dispatch, and native contracts;
- Python TypeOrder becomes a Phalcom semantic-order service over declaration surfaces and dispatch facts.

### Do not copy blindly

- Python's type lattice;
- Python overload/protocol rules;
- Any as a universal completion state;
- panic on demotion overflow;
- byte offsets as durable identity;
- unsafe publication before safe Phalcom reference tests exist;
- fixed-point limits as language semantics.

## Phalcom implementation sequence

1. Add CalcId, QueryKey, and typed binding indexes.
2. Extend TypeConstraint with origin, dependency, and relation metadata.
3. Implement query-local relation cache with InProgress/Ok/Err.
4. Implement solver-variable checkpoints and rollback for speculative calls.
5. Add recursive placeholders and RecursiveFixpoint.
6. Add SCC stack discovery and batch answer publication.
7. Add warm iterations and semantic convergence checks.
8. Add SemanticOrder over TypeHierarchy, declaration surfaces, and dispatch.
9. Add budgets and metrics before optimizing storage.
10. Differential-test clean and incremental solving.

## Required tests and metrics

Unit tests:

- self-recursive binding;
- mutually recursive bindings;
- SCC membership expansion and restart;
- previous-answer warm start;
- non-convergence budget;
- recursive relation cache;
- cache rollback after failed speculative check;
- variable snapshot/restore;
- first-writer-wins publication;
- semantic equality excluding generation/provenance.

Metamorphic tests:

- reorder independent constraints;
- reorder union members;
- duplicate equivalent constraints;
- insert an irrelevant binding;
- split a module without changing exports;
- clean solve versus incremental edit sequence;
- serial versus parallel scheduling.

Metrics:

- calculations demanded;
- calculation hits/misses;
- SCC count and membership;
- SCC iterations and demotion restarts;
- placeholders created/resolved;
- subset-cache hit rate;
- speculative probes committed/rolled back;
- constraint worklist steps;
- budget exhaustion;
- type-node allocations;
- semantic-order lookup time.

## Conclusion

Pyrefly does not achieve speed from an abstract “clever constraint solver.” It controls the entire solver execution lifecycle: compact calculation identity, demand-driven work, stack-to-SCC conversion, placeholder semantics, warm fixed points, rollbackable speculative solving, relation memoization, bounded recursion, and atomic result publication. Phalcom should implement those mechanics as first-class semantic-engine structures.
