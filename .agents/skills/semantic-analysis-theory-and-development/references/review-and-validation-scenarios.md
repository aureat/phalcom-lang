# Review and Validation Scenarios

Use these as pressure tests for an implementation agent. The expected answer is not merely a conclusion; the agent should identify the governing domain, invariant, dependency, and test.

## Scenario 1: “Unknown fixes the join”

A branch yields `String` on one path and a missing-import expression on the other. A patch joins them to `ValueShape::Unknown` and discards the missing dependency.

**Expected reasoning:** shape widening may be acceptable for an advisory shape projection, but missing dependency is a distinct recovery/availability fact. Preserve `Blocked(module)` provenance so recomputation occurs when the dependency appears and diagnostics do not misstate dynamic uncertainty.

**Reject if:** the agent says “Unknown is conservative, therefore correct” without discussing dependency invalidation and information domain.

## Scenario 2: Loop solved in three passes

A worklist implementation stops after three loop iterations to guarantee editor latency and returns the current union.

**Expected reasoning:** this is a budgeted approximation, not a fixed point. Define lattice/order, show why convergence is or is not guaranteed, add widening if needed, and surface `BudgetExceeded/Widened` separately. For sound consumers, ensure the final state over-approximates all remaining iterations.

**Test:** loop whose abstract chain needs more than three steps; compare with slow reference solver.

## Scenario 3: `ValueShape::Instance(Foo)` becomes type `Foo`

A checker patch converts every current `ValueShape` directly into a formal type.

**Expected reasoning:** reject domain collapse. Runtime-shape evidence is advisory/current and has bounded-union/heuristic policies not defined as Phalcom type judgments. Use shared IDs/provenance, but introduce explicit type representation/constraints and a documented bridge for any trusted evidence.

## Scenario 4: Type annotations select methods

A proposal allows two methods with the same Phalcom selector but different parameter annotations, selecting based on inferred argument types.

**Expected reasoning:** this changes dispatch/selector semantics, reflection, method identity, caches, dynamic calls, and compatibility. Current architecture/spec direction keeps types non-dispatching. Treat as a major language-design proposal, not checker implementation detail.

## Scenario 5: Unique target optimization under reflection

The analyzer sees receiver `Foo` and one method `bar`, so optimizer removes runtime fallback lookup.

**Expected reasoning:** source-visible uniqueness is insufficient if runtime method tables can mutate or subclasses/dynamic classes appear. Require closed-world assumption, class/method version guard/deoptimization, or proof. Advisory LSP navigation can still use the unique source candidate.

## Scenario 6: Closure capture after reassignment

```phalcom
let x = 1
let f = || { x }
x = "s"
f()
```

A patch records closure result as `Int` because `x` was `Int` at closure construction.

**Expected reasoning:** determine Phalcom capture semantics. If closure captures mutable storage, construction does not snapshot abstract value. Analyze `f()` with captured binding/cell state and effects; escaping/unknown invocation may require widening.

## Scenario 7: Literal block executed during analysis

```phalcom
consume(|| { global = 1 })
```

The analyzer applies the block write immediately because the argument syntax is a literal block.

**Expected reasoning:** constructing a block is not invocation. Only apply effects if `consume`'s summary/contract establishes that the relevant parameter is invoked, and account for conditional/multiple invocation semantics.

## Scenario 8: Fiber suspension and refinement

A future checker proves `shared.field is String`, then a fiber yields and later reads `shared.field` as `String` without revalidation.

**Expected reasoning:** if other execution contexts can mutate shared state while suspended, the refinement may be invalid. The effect/concurrency model must mark suspension/interference; kill/refine facts accordingly or require isolation/guarding.

## Scenario 9: Module cycle

```text
A imports B
B imports A
```

A patch rejects every cycle during import graph construction.

**Expected reasoning:** separate declaration resolution from runtime initialization. An SCC may permit staged declaration/interface indexing even if some initialization cycles are illegal. Consult normative module semantics before rejecting the graph categorically.

## Scenario 10: Cache with no invalidation rule

A completion cache is keyed by `(receiver_name, prefix)` and kept until process restart.

**Expected reasoning:** reject. Key lacks semantic receiver identity, module/generation, dispatch side, class hierarchy/method-table dependencies, visibility context, and source revision. Specify key/value/dependencies/validity/invalidation/concurrency/memory bound before implementation.

## Scenario 11: Source offsets as identity

`BindingId` is replaced with `(file_uri, declaration_start_offset)` to simplify navigation.

**Expected reasoning:** source insertion before declaration changes identity and invalidates unrelated facts; offsets are locations. Keep semantic IDs and map them to current source ranges. Snapshot-local recovery node IDs may use positions internally but must not masquerade as durable logical identity.

## Scenario 12: Delete caller but parameter stays widened

Caller A supplied `String`, caller B supplied `Number`; parameter summary is `String | Number`. Caller A is deleted, but summary remains union because the engine only joins new evidence.

**Expected reasoning:** classic retraction bug. Store contributions by source/caller, remove A's contribution, recompute touched slot to `Number`, and propagate changed summary to dependents. Current Phalcom `ParameterContributions` already demonstrates this pattern.

## Scenario 13: Solver timeout means property holds

An SMT query times out and the tool suppresses the warning because no counterexample was found.

**Expected reasoning:** timeout is `Unknown`, never `Proved`. Preserve proof outcome and trusted assumptions separately. Testing/unrolling/no-counterexample likewise cannot establish proof.

## Scenario 14: Incomplete selector in completion

User types `user.upd` and completion wants methods beginning `upd`. A resolver reports “no method selector `upd`” and emits an error.

**Expected reasoning:** completion-prefix search is not normative dispatch resolution. Resolve receiver, search member surface by prefix/family, preserve incomplete-source status, and avoid claiming a complete call target until selector syntax is complete.

## Scenario 15: Duplicate declaration during editing

Two temporary local declarations have the same name. Resolver picks first declaration by `BTreeMap` order.

**Expected reasoning:** if language forbids duplicates, represent ambiguity/error; do not make map order semantics. Navigation/refactoring should avoid arbitrary target selection.

## Scenario 16: `super` as superclass cast

HIR lowers `super.foo()` to `Cast(self, Superclass).foo()`.

**Expected reasoning:** reject. `super` changes lookup start, not actual receiver identity. Lowering must carry lexical lookup context while preserving `self` receiver.

## Scenario 17: Native function assumed pure

A Rust primitive has no Phalcom source body, so the effect analyzer gives it empty effects.

**Expected reasoning:** missing implementation visibility implies opaque/conservative effects, not purity. Add a semantic/native contract with conformance tests if precision is required.

## Scenario 18: One global semantic lock

Every hover takes a write lock because lazy inference updates a shared `HashMap`.

**Expected reasoning:** prefer immutable published snapshots and private/memoized query state with bounded synchronization. Semantic truth should not mutate under ordinary read queries. Measure query latency and lock contention.

## Scenario 19: Union cap silently changes checker result

Runtime-shape union grows past eight alternatives and widens to `Unknown`; checker interprets this as `Dynamic` and accepts a call.

**Expected reasoning:** two independent errors: bounded `ValueShape` is advisory, and widening is not explicit programmer-selected `Dynamic`. Preserve `Widened`/domain provenance; formal checker uses its own union/type policy.

## Scenario 20: Incremental result differs from clean rebuild

After rename + import edit + undo, hover works but references omits one use. A clean restart fixes it.

**Expected reasoning:** stale dependency/identity bug. Minimize the edit sequence, compare each semantic product generation, inspect forward/reverse dependencies and contribution retraction, and add the sequence as an incremental/full equivalence regression.

## Scenario 21: Feature review — pattern-based narrowing

A proposal adds a pattern whose success binds `x` and narrows receiver type/shape.

**Expected review path:** specify parser/source representation in parser skill; binding introductions and success/failure scope here; lower match operation and CFG edges; define abstract transfer on each edge; type system defines type refinement; prover defines logical pattern condition; LSP completion uses success-edge facts; tests cover failure path where binding must not exist.

## Scenario 22: Feature review — reflective method addition

A new reflection API can install a method at runtime.

**Expected review path:** update runtime/dispatch normative spec; semantic engine marks affected dispatch facts open-world/dynamic; optimizer uses version guard or avoids devirtualization; checker defines whether reflected method installation changes typed guarantees; occurrence/navigation may still index source-authored methods; incrementality source graph alone is insufficient to model executing runtime mutation.

## Scenario 23: Performance proposal — intern every string globally

**Expected reasoning:** ask whether string hash/clone cost is measured hot path, define normalization, process/workspace lifetime, reclamation/memory bound, and whether IDs survive generations. Prefer targeted selector/name interning if justified; do not trade editor-session memory leak for hypothetical speed.

## Scenario 24: Review checklist outcome

A strong agent reviewing any scenario should explicitly produce:

```text
status: CURRENT / NORMATIVE / PROPOSED / ...
semantic question
identity involved
runtime rule being modeled
analysis domain and trust
control-flow/interprocedural consequence
dependency + invalidation consequence
provenance/diagnostic consequence
consumer consequences
test exposing the distinction
future compatibility risk
```

If an answer jumps directly to Rust structs without this chain, the skill has not been applied correctly.
