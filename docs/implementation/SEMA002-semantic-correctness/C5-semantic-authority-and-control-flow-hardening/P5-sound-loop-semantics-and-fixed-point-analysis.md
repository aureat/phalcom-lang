# Part 05 — Sound Loop Semantics and Fixed-Point Analysis Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make every Phalcom loop form use one sound cyclic-flow model in which normal body completion and `continue` feed the next loop header, `break` and condition failure feed the post-loop exit, `return`/`throw` escape the loop entirely, and recursive flow facts reach a bounded semantic fixed point or conservatively become `Unknown(RecursiveFixpoint)`.

**Architecture:** Build the loop engine on top of Parts 1–4 rather than adding another control-flow system. `checker/control.rs` remains the owner of executable regions and condition splitting; a new focused `checker/loop_analysis.rs` owns cyclic topology and convergence; `checker/flow/state.rs` owns semantic fixed-point projections and widening; `CheckingContext` supplies isolated speculative flow probes so convergence iterations cannot duplicate diagnostics, explanation nodes, expression products, or callable exits. The existing `FlowGraph` remains a published structural product and topology oracle, not the checker execution engine.

**Tech Stack:** Rust, `phalcom-semantic`, `phalcom-ast`, canonical `TypeStore`, `TypeKnowledge` / evidence authority, path-sensitive `FlowState`, existing `FlowGraph`, semantic test `Fixture`, incremental `SemanticWorkspaceSession`, Cargo test/check/fmt.

**Spec:** This is Part 05 of the six-part semantic hardening series. It implements the loop/control-flow closure implied by `docs/impl/semantic/semantic-correctness/part-4/2026-08-27-semantic-capability-gap-closure-implementation-plan.md` and depends on the completed contracts/interfaces from Parts 01–04: evidence authority and callable certification, field lifecycle validity, canonical executable regions/abrupt exits, and trusted predicate/condition splitting.

## Global Constraints

- Repository implementation source of truth for this plan: `main` at `24fc9fd98f3c3c534c4d52b613962a39b9374185` before Parts 01–04 are applied.
- Rebase this plan onto the actual post-Part-04 HEAD before implementation; do not force old line numbers or duplicate abstractions that Parts 01–04 have already introduced.
- `phalcom-semantic` remains the sole owner of static semantics. Do not move loop typing, flow proof, or fixed-point logic into `phalcom-lsp`.
- Preserve the formal/advisory boundary. Advisory shapes never seed, strengthen, or terminate formal loop fixed points.
- Soundness precedes precision. A loop that cannot stabilize within the bounded solver must publish conservative unknown formal knowledge, never a guessed concrete type.
- The checker remains AST/executable-region driven in this part. Do not convert the existing `FlowGraph` into an execution IR.
- All loop forms share the same cyclic-flow engine. Do not implement independent `whileTrue`, `while let`, and `for` join algorithms.
- Do not use raw `FlowState ==` as the convergence predicate. Versions, explanation IDs, and provenance are transport/publication metadata and are not semantic fixed-point dimensions.
- Speculative iterations must not publish diagnostics, explanation nodes, expression analyses, dependencies, return exits, throw exits, or source-index facts into the parent analysis.
- Speculative iterations may intern canonical types into the one active `TypeStore`; interning must be idempotent and must never create a second type-store domain.
- `continue` is a backedge input, never a direct post-loop exit.
- Normal body completion is a backedge input, never a direct post-loop exit.
- `break` is a post-loop exit, never a backedge.
- `return` and `throw` are callable exits, not loop exits or backedges.
- `while let` evaluates its scrutinee on every iteration and scopes successful pattern bindings to that iteration.
- `for` evaluates each iterable lane once at the preheader and then repeats cursor/element flow; do not repeatedly execute source iterable expressions during fixed-point probing.
- Do not add value-range analysis, cardinality proof, general abstract interpretation, loop unrolling, termination proving, or a general constant evaluator in this part.
- Every semantic change starts with a failing law-level test and ends with focused + full regression verification.

---

## 1. Fresh Repository Grounding

This plan was rebuilt from source, not from the failed prior artifact.

At the grounding commit, the checker already contains several useful pieces, but they are not orchestrated into a sound loop analysis:

1. `phalcom-semantic/src/checker/flow/state.rs`
   - `FlowState::join_with_hierarchy(...)` ignores unreachable predecessor states and conservatively joins current binding/field knowledge.
   - `FlowState::widen_loop_state_with_hierarchy(...)` already exists and preserves binding contracts while joining changing facts.
   - `FlowState` derives `Eq`/`PartialEq`, but its equality includes `BindingState.version`, `FieldState.version`, fact explanation IDs, and `TypeKnowledge` provenance. Those fields make raw equality unsuitable for convergence.
   - `UnknownReason::RecursiveFixpoint` already exists in `types/evidence.rs`; there is no need to invent a second loop-unknown category.

2. `phalcom-semantic/src/checker/context.rs`
   - `LoopFlowFrame` currently stores `continues: Vec<FlowState>` and `breaks: Vec<FlowState>`.
   - `record_continue()` and `record_break()` snapshot current flow.
   - `CheckerControl` can be cloned while sharing the same budget and cancellation token.
   - resolver/hierarchy wrappers record semantic dependencies.

3. `phalcom-semantic/src/checker/statement.rs`
   - current `for` analysis executes the body once and then joins `[preheader, body_flow, continues..., breaks...]` as peers.
   - this is topologically wrong: normal completion/continue belong at the next header; preheader/condition failure/break belong at exit.

4. `phalcom-semantic/src/checker/expression.rs`
   - current `Expr::WhileLet` evaluates the scrutinee once, executes the body once, and returns `Unit`.
   - current sacred `whileTrue` structured path executes the body once and joins preheader/body/continue/break as peers.
   - the current `whileTrue` path recognizes a literal block receiver, but constructing/analyzing that block as a closure does not execute the condition block on each iteration.

5. `phalcom-semantic/src/checker/flow/graph.rs`
   - the published CFG already models the intended topology more accurately:
     - body tail → `BackEdge`
     - `continue` → header
     - `break` → exit join
     - condition false → exit join
   - therefore the CFG is valuable as an independent structural oracle for tests.
   - it must not become a second execution engine in this part.

6. `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
   - already tests same-type loop writes, zero-iteration/body joins, break/continue, closure creation, and a composed loop case.
   - those tests are necessary but insufficient because some expected final types can accidentally pass under the wrong topology.

### Required predecessor interfaces

Before Task 0 passes, the post-Part-04 tree must expose equivalent capabilities to these names/contracts:

```rust
// Part 03: executable control region.
pub(crate) struct ExecutableRegionResult {
    pub value: Option<TypedExpression>,
    pub flow: FlowState,
    pub causal_invalidity: CausalInvalidity,
}

pub(crate) fn analyze_executable_region(
    ctx: &mut CheckingContext<'_>,
    statements: &[Statement],
    range: SourceRange,
    expected: &ExpectedType,
) -> ExecutableRegionResult;

// Part 03: abrupt control is recorded atomically and terminates current flow.
impl CheckingContext<'_> {
    pub(crate) fn record_return_exit(&mut self, fact: NormalReturnFact);
    pub(crate) fn record_throw_exit_and_terminate(&mut self);
    pub(crate) fn record_break_and_terminate(&mut self);
    pub(crate) fn record_continue_and_terminate(&mut self);
}

// Part 04: condition truth + trusted predicate transfer.
pub(crate) enum ConditionTruth {
    AlwaysTrue,
    AlwaysFalse,
    Unknown,
}

pub(crate) enum PredicateTransfer {
    Unchanged,
    Refined(AppliedFlowRefinement),
    Contradiction { binding: BindingId, prior: TypeKnowledge },
}
```

If Part 04 uses different but equivalent final names, adapt this plan once at Task 0 and use the post-Part-04 names consistently thereafter.

---

## 2. Normative Loop Laws

These are acceptance laws, not implementation suggestions.

### L1 — Zero-iteration law

Unless the checker has an established proof that the loop must enter, ordinary post-loop flow includes a path in which the body executes zero times.

```phalcom
let x = 1
while condition {
    x = "later"
}
let y = x
```

`y` may be `Int | String`; it may not be established as `String`.

### L2 — Backedge law

Only reachable normal body completion and reachable `continue` exits feed the next loop header.

### L3 — Exit law

Only reachable condition-failure / iteration-exhaustion states and reachable `break` exits feed ordinary post-loop flow.

### L4 — Escaping abrupt law

`return` and `throw` are recorded at callable scope and never enter a loop header or ordinary loop exit join.

### L5 — Dead suffix law

Statements after `break`, `continue`, `return`, or `throw` on the same path cannot contribute binding types, field facts, diagnostics dependent on execution, or loop topology.

### L6 — Nested-target law

`break`/`continue` target the innermost active loop frame. An inner `break` cannot exit an outer loop; an inner `continue` cannot become an outer backedge.

### L7 — Stable-header law

A cyclic flow fact is publishable only from a stable semantic header state or from a conservative exhaustion state. A single traversal is not a proof of a loop invariant.

### L8 — Semantic convergence law

Convergence compares semantic state:

```text
binding contract/current type + authority/consistency/causal state
field contract/current/initialization/validity/causal state
predicate identity
reachability
```

and ignores:

```text
version counters
explanation IDs
source/provenance list duplication
iteration-local expression IDs
```

### L9 — Authority monotonicity through cycles

Loop widening/joining cannot upgrade Assumed evidence to Established. If any reachable contributor required by a loop fact is Assumed, the stable known result is at most Assumed.

### L10 — Unknown visibility law

A reachable unknown contributor cannot be discarded because another iteration has a known type. Recursive uncertainty remains formally visible.

### L11 — Exhaustion law

If a semantic state continues changing after the configured bound, only unstable facts are weakened to `Unknown(RecursiveFixpoint)`. Stable contracts and independent stable facts remain intact.

### L12 — Probe isolation law

The number of speculative fixed-point iterations must not change the final number or identity of published diagnostics, explanation nodes, expression products, callable exits, or semantic dependencies.

### L13 — Canonical TypeStore law

All iterations operate against the active canonical `TypeStore`. Probe type interning is allowed only because canonical interning is monotonic/idempotent; a probe must not materialize types in a clone and return their IDs.

### L14 — `while let` re-evaluation law

The `while let` scrutinee is evaluated at the header on every logical iteration. Successful pattern bindings exist only on the body path and are recreated per iteration.

### L15 — `for` preheader law

Each source iterable lane is evaluated exactly once before iteration. Element/cursor state repeats; source iterable expressions do not.

### L16 — Predicate-invariant law

A predicate fact survives a loop header only if it is valid on every reachable header predecessor. Facts that hold on one backedge but not on the entry/other backedge are removed by intersection.

### L17 — Field-cycle law

Fields use the same cyclic topology as bindings. Part 02's initialization/contract-validity dimensions must survive widening without conflating “assigned” with “contract validated.”

### L18 — Determinism law

Re-running clean analysis, changing only the fixed-point iteration cap above the convergence point, or reaching the same stable header via a different predecessor enumeration order must produce semantically equivalent formal products.

---

## 3. Target Architecture

### 3.1 File ownership

```text
checker/control.rs
  executable regions
  branch/condition split
  Part-04 ConditionTruth + predicate application
  no fixed-point loop iteration policy

checker/loop_analysis.rs          [new]
  loop topology
  bounded fixed-point driver
  loop-step probe/final-pass orchestration
  convergence/exhaustion result types
  no syntax-specific AST lowering

checker/flow/state.rs
  semantic fixed-point projection/key
  widening/joining primitives
  unstable-fact weakening
  no AST traversal

checker/context.rs
  isolated flow-probe child context
  loop-frame capture
  shared budget/cancellation
  no loop algorithm

checker/expression.rs
  syntax adapters:
    sacred whileTrue
    while let

checker/statement.rs
  syntax adapter:
    for

checker/flow/graph.rs
  structural CFG publication only
  topology parity tests

checker/analysis.rs
  optional published convergence metadata only if later consumers need it;
  do not publish iteration-internal states
```

### 3.2 New semantic fixed-point key

Add a projection that strips publication-only metadata.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum KnowledgeFixpointKey {
    Known {
        ty: TypeId,
        status: EvidenceStatus,
        origin: EvidenceOrigin,
    },
    Unknown(UnknownReason),
    Dynamic(DynamicReason),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BindingFixpointKey {
    pub contract: Option<BindingContract>,
    pub current: KnowledgeFixpointKey,
    pub denotation: Option<SemanticDenotation>,
    pub consistency: BindingConsistency,
    pub causal_invalidity: CausalInvalidity,
    pub mutable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FieldFixpointKey {
    pub contract: KnowledgeFixpointKey,
    pub current: KnowledgeFixpointKey,
    pub initialization: FieldInitialization,
    // Include the Part-02 validity state under its actual final name.
    pub validity: FieldContractValidity,
    pub causal_invalidity: CausalInvalidity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FlowFixpointKey {
    pub reachable: bool,
    pub bindings: BTreeMap<BindingId, BindingFixpointKey>,
    pub fields: BTreeMap<FieldId, FieldFixpointKey>,
    pub predicates: BTreeSet<FlowPredicate>,
}
```

`EvidenceOrigin` remains in the key because a change from an assumption sourced from a declaration to a flow-derived established/assumed fact is semantically observable. Provenance range arrays do not.

Add:

```rust
impl FlowState {
    pub(crate) fn fixpoint_key(&self) -> FlowFixpointKey;
}
```

### 3.3 Isolated probe API

Add a child-analysis boundary in `CheckingContext`:

```rust
pub(crate) struct FlowProbeResult<T> {
    pub value: T,
    pub flow: FlowState,
}

impl CheckingContext<'_> {
    pub(crate) fn run_flow_probe<T>(
        &mut self,
        entry: FlowState,
        run: impl FnOnce(&mut CheckingContext<'_>) -> T,
    ) -> FlowProbeResult<T>;
}
```

The child context must:

- use the same mutable `TypeStore`;
- use the same underlying hierarchy/resolver semantics;
- borrow the same dispatch snapshot and lazily detach only inside the child if local declarations require mutation;
- clone lexical scopes so existing `BindingId`s resolve exactly as in the parent;
- clone current class/side/callable/return contract;
- attach the same field signatures;
- share `CheckerControl` so budget/cancellation apply across probes;
- start with fresh diagnostics, explanations, expression analysis, callable exits, dependency sets, and loop frames;
- discard every child-local product except the explicitly returned value and final `FlowState`.

Add private accessors:

```rust
impl TrackingTypeResolver<'_> {
    pub(crate) fn inner(&self) -> &dyn TypeResolver;
}

impl TrackingTypeHierarchy<'_> {
    pub(crate) fn inner(&self) -> &dyn TypeHierarchy;
}
```

This prevents speculative child reads from polluting the parent's recorded dependency set.

### 3.4 Condition split interface

Part 04 branch logic must expose a reusable condition split:

```rust
#[derive(Clone, Debug)]
pub(crate) struct ConditionFlowSplit {
    pub condition: TypedExpression,
    pub when_true: FlowState,
    pub when_false: FlowState,
    pub causal_invalidity: CausalInvalidity,
}

pub(crate) fn analyze_condition_split(
    ctx: &mut CheckingContext<'_>,
    condition: &Expr,
) -> ConditionFlowSplit;
```

This function owns:

- one evaluation of the condition expression;
- `ConditionTruth`;
- trusted predicate extraction;
- true/false refinement;
- contradiction → unreachable conversion.

Branch and loop code both call it.

### 3.5 Fixed-point engine interface

Create `checker/loop_analysis.rs`:

```rust
pub(crate) const MAX_LOOP_FIXPOINT_ITERATIONS: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LoopConvergence {
    Stable { iterations: u8 },
    Exhausted { iterations: u8 },
}

#[derive(Clone, Debug)]
pub(crate) struct LoopStepResult {
    pub normal_backedge: Option<FlowState>,
    pub continues: Vec<FlowState>,
    pub breaks: Vec<FlowState>,
}

impl LoopStepResult {
    pub(crate) fn backedge_states(&self) -> impl Iterator<Item = &FlowState>;
}

#[derive(Clone, Debug)]
pub(crate) struct LoopFixpoint {
    pub header: FlowState,
    pub convergence: LoopConvergence,
}
```

The generic solver consumes:

```rust
pub(crate) fn solve_loop_header(
    ctx: &mut CheckingContext<'_>,
    entry: &FlowState,
    probe_iteration: impl FnMut(&mut CheckingContext<'_>, &FlowState) -> LoopStepResult,
) -> Result<LoopFixpoint, FlowInvariantFailure>;
```

Algorithm:

```text
header := entry
repeat up to MAX_LOOP_FIXPOINT_ITERATIONS:
    step := isolated probe(header)
    backedges := reachable(normal completion + continues)
    if no backedges:
        stable: no cyclic state reaches header
    candidate := join(entry + backedges)
    next := widen(header, candidate)
    if next.fixpoint_key == header.fixpoint_key:
        stable
    header := next

if bound exhausted:
    run one more probe/header candidate
    weaken only dimensions whose fixed-point keys still differ
    mark those current facts Unknown(RecursiveFixpoint)
    return Exhausted
```

The fixed-point solver does **not** include break/false-exit states in the header.

### 3.6 Exhaustion weakening

Add:

```rust
impl FlowState {
    pub(crate) fn weaken_unstable_fixpoint_facts(
        previous: &FlowState,
        next: &FlowState,
    ) -> FlowState;
}
```

Rules:

- persistent contracts remain unchanged;
- mutability remains unchanged;
- stable bindings retain their exact current knowledge/status/consistency;
- unstable binding current knowledge becomes `Unknown(RecursiveFixpoint)`;
- unstable denotation becomes `None`;
- unstable consistency becomes `Blocked(BlockReason::RecursiveFixpoint)` when a persistent contract exists;
- field current/validity dimensions that are still changing are conservatively weakened under Part-02 rules;
- invariant predicate facts remain; changing facts are discarded;
- reachability is not fabricated;
- poisoned/internal-failure states terminate analysis rather than becoming fixpoint unknowns.

### 3.7 Final-pass rule

After solving the header, execute exactly one real iteration/condition pass in the parent context at the stable/conservative header. This pass alone publishes:

- expression analyses;
- diagnostics;
- explanation DAG nodes;
- return/throw exits;
- semantic dependencies;
- final loop-frame `break`/`continue` captures.

Post-loop state is:

```text
join(reachable condition-false/iteration-exhaustion exit,
     reachable break exits)
```

Never include:

```text
normal body completion
continue states
```

directly in the post-loop join.

---

## 4. File Change Map

| File | Responsibility in Part 05 |
| --- | --- |
| `phalcom-semantic/src/checker/mod.rs` | register/export `loop_analysis` internally |
| `phalcom-semantic/src/checker/loop_analysis.rs` | **create** bounded fixed-point engine and topology types |
| `phalcom-semantic/src/checker/control.rs` | expose reusable condition split from Part 04; keep executable-region semantics |
| `phalcom-semantic/src/checker/context.rs` | isolated probe child context; raw resolver/hierarchy accessors; loop-frame helpers |
| `phalcom-semantic/src/checker/flow/state.rs` | semantic convergence keys; exhaustion weakening; fixed-point unit tests |
| `phalcom-semantic/src/checker/expression.rs` | replace one-pass `whileTrue` / `while let` adapters with canonical loop engine |
| `phalcom-semantic/src/checker/statement.rs` | replace one-pass `for` adapter with canonical loop engine |
| `phalcom-semantic/src/checker/flow/graph.rs` | no execution changes; only parity fixes if tests expose structural mismatch |
| `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs` | main real-source loop law suite |
| `phalcom-semantic/tests/semantic/foundations/flow_graph.rs` | structural topology parity |
| `phalcom-semantic/tests/semantic/capabilities/fields.rs` | field-through-loop composition |
| `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs` | predicate/branch/loop composition where appropriate |
| `phalcom-semantic/tests/semantic/incremental/fingerprints.rs` | clean/incremental deterministic semantic fingerprints |
| `phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs` | TypeStore stability/idempotent probe interning |

---

# 5. Implementation Tasks

## Task 0 — Rebase Gate and Soundness RED Matrix

**Files:**
- Modify: `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/mod.rs` only if a new focused test module is added
- Read/verify: post-Part-04 `checker/control.rs`, `checker/context.rs`, `checker/analysis.rs`, `checker/flow/transfer.rs`

**Interfaces:**
- Consumes: completed Part-01 through Part-04 APIs.
- Produces: failing tests that distinguish correct loop topology from the current accidental joins.

- [ ] **Step 1: Record the implementation base**

Run:

```bash
git rev-parse HEAD
git status --short
cargo test -p phalcom-semantic semantic::capabilities::flow_loops -- --nocapture
```

Record the actual post-Part-04 SHA at the top of the implementation work log. Do not edit this plan's historical grounding SHA.

- [ ] **Step 2: Verify predecessor contracts compile**

Run:

```bash
rg -n "analyze_executable_region|ConditionFlowSplit|ConditionTruth|record_continue_and_terminate|NormalReturnFact" phalcom-semantic/src/checker
```

Expected: the post-Part-04 equivalents exist. If names differ, mechanically update the names used by this plan before implementation; do not create compatibility aliases solely for this plan.

- [ ] **Step 3: Add RED test — `continue` is not a direct exit**

Add to `flow_loops.rs`:

```rust
#[test]
fn continue_state_feeds_header_not_direct_post_loop_exit() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ keepGoing: Bool, _ skip: Bool) {
    let x = 1
    while keepGoing {
      if skip {
        x = "continued"
        continue
      }
      x = 2.5
      break
    }
    let observed = x
  }
}
"#,
    );

    let run = f.callable("Probe", "run", DispatchSide::Class);
    let int_ty = f.ty("Int");
    let string_ty = f.ty("String");
    let float_ty = f.ty("Float");
    let observed = f.binding(run, "observed").current.ty().expect("post-loop knowledge");
    f.assert_union_members(observed, &[int_ty, string_ty, float_ty]);

    // Topology assertion is the key regression: Continue must lead to a
    // LoopHeader, never directly to the loop-exit Join.
    f.assert_continue_edges_target_loop_headers(run);
}
```

If `Fixture` does not yet have `assert_continue_edges_target_loop_headers`, add it in `tests/semantic/support/fixture.rs` using the callable's `flow_graph`.

- [ ] **Step 4: Add RED test — dead suffix after `continue` does not contribute**

```rust
#[test]
fn statement_after_continue_never_contributes_loop_fact() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
    while flag {
      x = "seen"
      continue
      x = true
    }
    let y = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    let y = f.binding(run, "y").current.ty().expect("post-loop type");
    f.assert_union_members(y, &[f.ty("Int"), f.ty("String")]);
    assert!(!f.union_contains(y, f.ty("Bool")));
}
```

- [ ] **Step 5: Add RED test — dead suffix after `break` does not contribute**

Use the same shape with `break` before `x = true`. Expected post-loop members: `Int | String`, never `Bool`.

- [ ] **Step 6: Add RED test — `while let` is cyclic, not one-shot**

Use an outer mutable binding and a scrutinee whose type remains valid across repetitions. Assert the outer binding's post-loop fact includes both preheader and body-updated knowledge and that the CFG contains a `BackEdge`.

- [ ] **Step 7: Add RED test — fixed-point publication is iteration-count independent**

Create a helper fixture source whose header needs more than one semantic step to stabilize. Analyze it twice with test-only loop caps `4` and `8` (the cap injection is implemented in Task 4). Assert equal:
- final binding knowledge;
- diagnostics;
- callable normal-return summary;
- explanation **semantic content**, not raw IDs.

Initially mark the helper invocation behind the missing test hook so compilation fails for the intended missing API.

- [ ] **Step 8: Run the RED set**

```bash
cargo test -p phalcom-semantic semantic::capabilities::flow_loops -- --nocapture
```

Expected: new topology/fixed-point tests fail for current one-pass/join-all behavior.

- [ ] **Step 9: Commit tests only**

```bash
git add phalcom-semantic/tests/semantic/capabilities/flow_loops.rs \
        phalcom-semantic/tests/semantic/support/fixture.rs
git commit -m "test(semantic): pin sound loop topology laws"
```

---

## Task 1 — Semantic Fixed-Point Projection

**Files:**
- Modify: `phalcom-semantic/src/checker/flow/state.rs`
- Test: inline unit tests in `checker/flow/state.rs`

**Interfaces:**
- Consumes: Part-02 final `FieldState` validity/causal fields.
- Produces:
  - `KnowledgeFixpointKey`
  - `BindingFixpointKey`
  - `FieldFixpointKey`
  - `FlowFixpointKey`
  - `FlowState::fixpoint_key()`
  - `FlowState::weaken_unstable_fixpoint_facts(...)`

- [ ] **Step 1: Add failing equality-separation tests**

Write a unit test proving two states with the same semantic facts but different `version` counters and explanation IDs produce equal fixed-point keys.

```rust
#[test]
fn fixpoint_key_ignores_versions_and_explanation_ids() {
    // Construct two equivalent binding states.
    // Change only version and explanation.
    assert_ne!(left, right);
    assert_eq!(left.fixpoint_key(), right.fixpoint_key());
}
```

- [ ] **Step 2: Add failing provenance-insensitivity test**

Construct two `TypeKnowledge::Known` values with identical `ty/status/origin` but different source ranges. Assert their `KnowledgeFixpointKey`s are equal.

- [ ] **Step 3: Add failing semantic-change tests**

Assert fixed-point keys differ when any of these changes:
- type;
- Established ↔ Assumed;
- origin;
- binding consistency;
- causal invalidity;
- mutability;
- field initialization;
- Part-02 field validity;
- reachability;
- predicate identity.

- [ ] **Step 4: Implement fixed-point key structs**

Add the structs from §3.2. Implement `From<&TypeKnowledge> for KnowledgeFixpointKey`.

For fact sets, expose a predicate-only iterator:

```rust
impl FactSet {
    pub(crate) fn predicate_keys(&self) -> impl Iterator<Item = &FlowPredicate> {
        self.facts.keys()
    }
}
```

- [ ] **Step 5: Implement `FlowState::fixpoint_key()`**

Build ordered `BTreeMap`/`BTreeSet` projections. Never copy explanation IDs or versions into the key.

- [ ] **Step 6: Add failing exhaustion-weaken tests**

Test:
- stable binding remains exact;
- unstable binding becomes `Unknown(RecursiveFixpoint)`;
- persistent contract remains;
- unstable denotation clears;
- field contract remains;
- changing predicate fact disappears;
- unrelated stable predicate remains.

- [ ] **Step 7: Implement `weaken_unstable_fixpoint_facts`**

Compare the previous and next fixed-point keys dimension-by-dimension rather than replacing the entire state with unknown. Preserve stable independent evidence.

- [ ] **Step 8: Run focused tests**

```bash
cargo test -p phalcom-semantic checker::flow::state -- --nocapture
cargo check -p phalcom-semantic
```

Expected: PASS.

- [ ] **Step 9: Commit**

```bash
git add phalcom-semantic/src/checker/flow/state.rs
git commit -m "feat(semantic): add loop fixed-point state projection"
```

---

## Task 2 — Isolated Flow Probe Context

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/flow_probe.rs` **create**
- Modify: `phalcom-semantic/tests/semantic/foundations/mod.rs`

**Interfaces:**
- Consumes: `CheckingContext`, `CheckerControl`, `DispatchAccess`, tracking wrappers.
- Produces:
  - `TrackingTypeResolver::inner()`
  - `TrackingTypeHierarchy::inner()`
  - `FlowProbeResult<T>`
  - `CheckingContext::run_flow_probe(...)`

- [ ] **Step 1: Add RED probe-isolation test**

Construct a `CheckingContext`, execute a probe containing:
- a type mismatch diagnostic;
- a method call;
- a literal expression;
- a `return`.

Assert after the probe:
- parent diagnostics unchanged;
- parent explanation arena length unchanged;
- parent expression index unchanged;
- parent normal-return exits unchanged;
- parent dependency sets unchanged;
- parent `flow` unchanged;
- returned probe flow reflects the transfer.

- [ ] **Step 2: Add RED shared-budget test**

Use a small `QueryBudget`; perform enough probe work to consume it; assert subsequent parent analysis observes the shared budget exhaustion. This proves probes cannot escape bounded analysis.

- [ ] **Step 3: Add raw-wrapper accessors**

```rust
impl TrackingTypeResolver<'_> {
    pub(crate) fn inner(&self) -> &dyn TypeResolver {
        self.inner
    }
}

impl TrackingTypeHierarchy<'_> {
    pub(crate) fn inner(&self) -> &dyn TypeHierarchy {
        self.inner
    }
}
```

- [ ] **Step 4: Implement `run_flow_probe` with a child context**

The child must be constructed using `new_with_dispatch_ref_and_control` against:
- `self.store` reborrow;
- `self.hierarchy.inner()`;
- `self.resolver.inner()`;
- `self.declarations`;
- `self.dispatch.get()`;
- cloned `self.control`.

Then copy semantic execution context:

```rust
probe.current_class = self.current_class.clone();
probe.current_side = self.current_side;
probe.current_callable = self.current_callable.clone();
probe.expected_return = self.expected_return.clone();
probe.scopes = self.scopes.clone();
probe.flow = entry;
probe.body_id = self.body_id;
```

Attach field signatures via the existing context API. Do not copy parent publication vectors/arenas into the child.

- [ ] **Step 5: Preserve source binding identity**

Add a probe test reading and writing an existing local by name. Assert the child resolves the same `BindingId` and the returned flow updates that binding without mutating the parent flow.

- [ ] **Step 6: Verify local declaration isolation**

Probe a nested/local declaration path that forces `DispatchAccess::Borrowed` to detach. Assert the parent dispatch surface does not gain the probe-local declaration.

- [ ] **Step 7: Run focused tests**

```bash
cargo test -p phalcom-semantic semantic::foundations::flow_probe -- --nocapture
cargo check -p phalcom-semantic
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/tests/semantic/foundations/flow_probe.rs \
        phalcom-semantic/tests/semantic/foundations/mod.rs
git commit -m "feat(semantic): isolate speculative flow probes"
```

---

## Task 3 — Reusable Condition Split

**Files:**
- Modify: `phalcom-semantic/src/checker/control.rs`
- Modify: branch callers in `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs`

**Interfaces:**
- Consumes: Part-04 `ConditionTruth`, trusted predicates, `PredicateTransfer`.
- Produces:
  - `ConditionFlowSplit`
  - `analyze_condition_split(...)`

- [ ] **Step 1: Add RED test that condition evaluation happens once**

Use a condition expression with a semantically visible callable dependency/expression product. Assert an `if` creates one condition expression analysis, not two.

This protects the extraction from accidentally evaluating once per branch.

- [ ] **Step 2: Extract condition analysis from branch-pair code**

Implement the interface from §3.4. Start from the caller's current flow, evaluate the expression once, fork it, then apply true/false condition facts.

- [ ] **Step 3: Preserve constant-condition behavior**

`AlwaysTrue` returns an unreachable false state. `AlwaysFalse` returns an unreachable true state.

- [ ] **Step 4: Preserve predicate contradiction behavior**

A trusted contradiction marks only the contradicted fork unreachable. It does not emit a fake type-mismatch diagnostic merely because a branch is impossible.

- [ ] **Step 5: Route branch analysis through the extracted split**

Remove duplicated condition/refinement logic from `analyze_branch_pair`.

- [ ] **Step 6: Run branch suite**

```bash
cargo test -p phalcom-semantic semantic::capabilities::flow_branches -- --nocapture
cargo test -p phalcom-semantic semantic::foundations::flow_graph -- --nocapture
```

- [ ] **Step 7: Commit**

```bash
git add phalcom-semantic/src/checker/control.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_branches.rs
git commit -m "refactor(semantic): share canonical condition flow split"
```

---

## Task 4 — Bounded Loop Fixed-Point Engine

**Files:**
- Create: `phalcom-semantic/src/checker/loop_analysis.rs`
- Modify: `phalcom-semantic/src/checker/mod.rs`
- Modify: `phalcom-semantic/src/checker/flow/state.rs`
- Test: inline unit tests in `loop_analysis.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`

**Interfaces:**
- Consumes:
  - `CheckingContext::run_flow_probe`
  - `FlowState::fixpoint_key`
  - `FlowState::widen_loop_state_with_hierarchy`
  - `FlowState::weaken_unstable_fixpoint_facts`
- Produces:
  - `MAX_LOOP_FIXPOINT_ITERATIONS`
  - `LoopConvergence`
  - `LoopStepResult`
  - `LoopFixpoint`
  - `solve_loop_header(...)`

- [ ] **Step 1: Add unit RED test for a two-step convergence**

Use a synthetic probe closure that changes an `Int` binding to `String` on the first cycle. Assert the solver returns a stable `Int | String` header and needs more than one probe.

- [ ] **Step 2: Add RED test proving `breaks` are ignored by header solving**

Return a `break` state containing a distinct type that no backedge has. Assert that type never enters the solved header.

- [ ] **Step 3: Add RED test proving `continues` do feed the header**

Return a continue state with a distinct type and assert it appears in the stable header.

- [ ] **Step 4: Implement the fixed-point loop**

Use `ctx.run_flow_probe(header.clone(), ...)` for each speculative iteration.

Join only reachable normal/continue backedges. Use hierarchy-aware join/widening. Compare `fixpoint_key()`.

- [ ] **Step 5: Add bounded-exhaustion test**

Create a test-only probe that changes a semantic type every iteration. Use an internal test hook:

```rust
fn solve_loop_header_with_limit(..., limit: usize, ...)
```

Keep the production wrapper fixed at `MAX_LOOP_FIXPOINT_ITERATIONS`.

Assert exhaustion returns `LoopConvergence::Exhausted` and unstable facts become `Unknown(RecursiveFixpoint)`.

- [ ] **Step 6: Handle no-backedge loops**

If body/condition analysis yields no reachable backedge, the entry header is sufficient; do not fabricate another iteration.

- [ ] **Step 7: Propagate cancellation/budget/internal failures**

If a probe reaches cancelled, budget-exceeded, or poisoned/internal-failure state, terminate through existing checker status machinery rather than converting operational failures to `RecursiveFixpoint`.

- [ ] **Step 8: Run tests**

```bash
cargo test -p phalcom-semantic checker::loop_analysis -- --nocapture
cargo test -p phalcom-semantic checker::flow::state -- --nocapture
cargo check -p phalcom-semantic
```

- [ ] **Step 9: Commit**

```bash
git add phalcom-semantic/src/checker/loop_analysis.rs \
        phalcom-semantic/src/checker/mod.rs \
        phalcom-semantic/src/checker/flow/state.rs
git commit -m "feat(semantic): add bounded loop fixed-point engine"
```

---

## Task 5 — Sacred `whileTrue` Through the Canonical Loop Engine

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/flow_graph.rs`

**Interfaces:**
- Consumes: Part-03 executable regions, Task-03 condition split concepts, Task-04 fixed point.
- Produces: correct structured semantics for parser-lowered/sacred `whileTrue`.

- [ ] **Step 1: Add RED test proving condition block executes**

Use:

```phalcom
let checks = 0
while (checks < limit) {
    checks = checks + 1
}
```

Assert the condition reads the header's evolving `checks` fact and the final checker products include condition/body expression analyses from the final pass.

The exact parser-produced `whileTrue` shape should be used through source parsing; do not manufacture the AST manually.

- [ ] **Step 2: Add RED constant-false test**

```phalcom
let x = 1
while false {
    x = "unreachable"
}
let y = x
```

Expected: `y` remains established `Int`; body write does not contribute.

- [ ] **Step 3: Add RED break-exit test**

A `break` path with `x = "break"` must contribute to post-loop flow; a body-tail type reachable only on a backedge must not be treated as a direct exit.

- [ ] **Step 4: Replace the one-pass `whileTrue` join**

In `synthesize_control_method_call`, keep the existing sacred recognition gate, but route the recognized loop to a helper that:
1. captures preheader flow;
2. solves header by probe execution;
3. executes the condition block at the solved header;
4. applies Bool/truth splitting under Part-04 semantics;
5. executes body only on true flow;
6. captures continue/break through the active loop frame;
7. computes post-loop from false flow + breaks only.

- [ ] **Step 5: Do not treat closure construction as condition execution**

The literal block receiver remains syntactically a closure value, but the sacred inlined control path executes its block body as an executable region. Remove any reliance on the already-created closure's callable type as proof of the condition result.

- [ ] **Step 6: Validate condition type**

The condition executable region must be checked against canonical Bool expectation using the same semantic type identity rules as other condition contexts. An unknown/invalid condition cannot silently become true/false.

- [ ] **Step 7: Run focused tests**

```bash
cargo test -p phalcom-semantic semantic::capabilities::flow_loops -- --nocapture
cargo test -p phalcom-semantic semantic::foundations::flow_graph -- --nocapture
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_loops.rs \
        phalcom-semantic/tests/semantic/foundations/flow_graph.rs
git commit -m "fix(semantic): make while loops use cyclic flow"
```

---

## Task 6 — `while let` Through the Canonical Loop Engine

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/patterns.rs`

**Interfaces:**
- Consumes: Task-04 fixed point; Part-03 executable regions; existing pattern binding/decomposition.
- Produces: per-iteration scrutinee evaluation and successful-pattern body scope.

- [ ] **Step 1: Add RED test for per-iteration outer mutation**

Use a `while let` body that changes an outer binding from one established type to another legal type. Assert post-loop knowledge includes the preheader + stable backedge facts.

- [ ] **Step 2: Add RED test that pattern bindings do not leak**

Bind `item` only in the `while let` pattern. Assert no outer binding named `item` appears after the loop and source binding identities remain distinct.

- [ ] **Step 3: Add RED `continue` test inside `while let`**

A write before `continue` must reach the next header. A write after `continue` must not.

- [ ] **Step 4: Add RED `break` test inside `while let`**

A write before `break` contributes to post-loop state, not header state.

- [ ] **Step 5: Implement one iteration probe**

At each probe header:
1. evaluate `while_let.value`;
2. fork success/failure pattern paths using the existing refutable-pattern semantics introduced by Part 03/04;
3. on success, push iteration scope, bind pattern from the scrutinee fact, execute body region, pop scope;
4. collect normal completion + continues as backedges;
5. keep pattern-failure flow as loop exit;
6. keep breaks separate.

Do not evaluate the scrutinee once before calling the solver.

- [ ] **Step 6: Execute one final real pass**

Repeat the same operation in the parent at the solved header so diagnostics/explanations/pattern sites are published once.

- [ ] **Step 7: Verify variant/list/tuple pattern composition**

Run:

```bash
cargo test -p phalcom-semantic semantic::capabilities::patterns -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::flow_loops -- --nocapture
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_loops.rs \
        phalcom-semantic/tests/semantic/capabilities/patterns.rs
git commit -m "fix(semantic): make while-let analysis iterative"
```

---

## Task 7 — `for` Through the Canonical Loop Engine

**Files:**
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/iteration.rs`

**Interfaces:**
- Consumes: existing iterable protocol resolution and Task-04 solver.
- Produces: preheader-once iterable evaluation + cyclic body flow.

- [ ] **Step 1: Add RED single-evaluation test**

Use an iterable expression that produces one call-resolution/expression product. Assert the source iterable is analyzed once even when header solving takes multiple iterations.

Do not count internal probe expression IDs because probes are intentionally unpublished; assert the final public product contains one source iterable analysis.

- [ ] **Step 2: Add RED zero-iteration test**

```phalcom
let x = 1
for item in values {
    x = "body"
}
let y = x
```

Without an established non-empty cardinality proof, post-loop `y` must retain the preheader possibility.

- [ ] **Step 3: Add RED continue/break topology test**

Assert:
- normal completion and continue flow to header;
- break flows to exit;
- `return`/`throw` in body are excluded from both.

- [ ] **Step 4: Refactor iterable lane setup into preheader data**

Keep existing `resolve_iteration_element_application` behavior. Evaluate each `for_stmt.lanes[*].iter` exactly once left-to-right and retain:
- element `ValueSemanticFact`;
- iteration causal invalidity;
- explanation parent needed by the final real pattern bind.

- [ ] **Step 5: Probe body iterations without re-evaluating iterable source**

Each probe:
- starts from candidate header flow;
- pushes iteration scope;
- binds lane patterns from precomputed element semantic facts;
- binds ordinal/index facts under existing semantics;
- executes body;
- collects normal/continue backedges and break exits.

- [ ] **Step 6: Preserve lockstep lane semantics**

For multiple lanes, do not infer that one lane's type arguments or cardinality replace another's. Keep each element fact independent and join only flow state, not lane identity.

- [ ] **Step 7: Execute final pass and compute exit**

Post-loop = zero/exhaustion path + breaks. The zero/exhaustion path begins from the stable header; it is not the original preheader after backedge refinement.

- [ ] **Step 8: Run focused tests**

```bash
cargo test -p phalcom-semantic semantic::capabilities::iteration -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::flow_loops -- --nocapture
```

- [ ] **Step 9: Commit**

```bash
git add phalcom-semantic/src/checker/statement.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_loops.rs \
        phalcom-semantic/tests/semantic/capabilities/iteration.rs
git commit -m "fix(semantic): route for loops through fixed-point flow"
```

---

## Task 8 — Nested Loops, Predicates, Fields, and Abrupt Exits

**Files:**
- Modify: `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/fields.rs`
- Modify: `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs`
- Modify implementation only where these composed tests expose a violation

**Interfaces:**
- Consumes: complete loop engine + Parts 02–04.
- Produces: composition proof that the engine works across semantic dimensions.

- [ ] **Step 1: Test nested loop target ownership**

Source:

```phalcom
while outer {
    while inner {
        x = "inner-break"
        break
    }

    if skip {
        x = 2.5
        continue
    }

    x = true
    break
}
```

Assert inner `break` exits only inner loop; outer `continue` is an outer backedge; outer `break` is the outer exit.

- [ ] **Step 2: Test predicate fact intersection at header**

Start with a union-typed binding. Narrow it on only one backedge. Assert the next stable header does not retain a predicate fact that is absent from the entry/other predecessor.

- [ ] **Step 3: Test contradiction-pruned backedge**

Under Part-04 established contradiction semantics, an unreachable refined branch must not feed the header.

- [ ] **Step 4: Test field mutation through a loop**

Use a constructor/method with a Part-02 field contract. Assert:
- loop assignment affects field current knowledge through the fixed point;
- definite initialization follows reachable topology;
- validity is not upgraded merely because a loop wrote the field;
- invalid write remains causally invalid and cannot publish established lifecycle knowledge.

- [ ] **Step 5: Test return/throw exclusion**

Put distinct types before `return` and `throw`. Assert they appear in callable exit products as appropriate but not in post-loop binding joins.

- [ ] **Step 6: Test all-abrupt loop body**

If every entered body path returns or throws and the condition can be false, only the false/zero path reaches after the loop. If an established always-true condition has no break and all body paths escape callable scope, post-loop flow is unreachable.

- [ ] **Step 7: Run composition suites**

```bash
cargo test -p phalcom-semantic semantic::capabilities::flow_loops -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::flow_branches -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::fields -- --nocapture
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-semantic/tests/semantic/capabilities/flow_loops.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_branches.rs \
        phalcom-semantic/tests/semantic/capabilities/fields.rs \
        phalcom-semantic/src/checker
git commit -m "test(semantic): close composed cyclic flow laws"
```

---

## Task 9 — CFG Topology Parity Without CFG Execution

**Files:**
- Modify: `phalcom-semantic/tests/semantic/foundations/flow_graph.rs`
- Modify: `phalcom-semantic/src/checker/flow/graph.rs` only if a structural mismatch is exposed

**Interfaces:**
- Consumes: published `CallableAnalysis.flow_graph`.
- Produces: structural parity assertions between AST checker destinations and CFG edge kinds.

- [ ] **Step 1: Add helper assertions**

In test support or the flow-graph suite, add:
- every `Continue` edge targets a `LoopHeader`;
- every `BackEdge` targets a `LoopHeader`;
- every `Break` edge targets a loop exit `Join`;
- `Return`/`Throw` edges do not target loop headers;
- nested loop edges target the nearest structurally enclosing header/join.

- [ ] **Step 2: Add one source fixture per loop form**

Cover:
- parser-lowered `while`;
- `while let`;
- `for`;
- nested loops.

- [ ] **Step 3: Compare semantic destinations, not node numbering**

Do not assert raw `FlowNodeId` numbers. Node allocation order is presentation metadata. Assert node kinds, edge kinds, ranges, and structural predecessor/successor relationships.

- [ ] **Step 4: Fix only structural CFG bugs**

If a mismatch exists, patch `flow/graph.rs` to match the now-canonical checker topology. Do not make the checker consume the CFG.

- [ ] **Step 5: Run**

```bash
cargo test -p phalcom-semantic semantic::foundations::flow_graph -- --nocapture
```

- [ ] **Step 6: Commit**

```bash
git add phalcom-semantic/src/checker/flow/graph.rs \
        phalcom-semantic/tests/semantic/foundations/flow_graph.rs
git commit -m "test(semantic): enforce loop cfg topology parity"
```

---

## Task 10 — Probe Determinism, TypeStore Stability, and Incremental Equivalence

**Files:**
- Modify: `phalcom-semantic/tests/semantic/incremental/fingerprints.rs`
- Modify: `phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs`
- Modify: `phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs`
- Modify: `phalcom-semantic/src/db/fingerprint.rs` only if final semantic loop products are missing from fingerprints

**Interfaces:**
- Consumes: final loop analysis.
- Produces: proof that speculative iterations do not destabilize immutable semantic products.

- [ ] **Step 1: Add diagnostic/explanation determinism test**

Analyze the same loop source twice from clean sessions. Assert:
- same diagnostic codes/ranges/messages;
- same explanation semantic steps/parent structure after normalizing arena-local IDs;
- same callable return knowledge;
- same binding/field formal facts.

- [ ] **Step 2: Add cap-above-convergence test**

With test-only solver limit injection, analyze a loop known to stabilize in ≤3 iterations using limits `4` and `8`. Assert final semantic products are equal.

- [ ] **Step 3: Add TypeStore idempotence test**

Within one session:
1. analyze a loop that interns a union/applied type during probing;
2. record `store.type_count()`;
3. reanalyze an unchanged revision or equivalent body;
4. assert the count does not grow solely because probes repeated;
5. assert `TypeStoreId` remains unchanged.

- [ ] **Step 4: Add incremental body-edit test**

Edit only a loop body such that its stable header changes. Assert:
- affected callable recomputes;
- unaffected callable reuses;
- downstream dependency invalidates only if published semantic fingerprint changes;
- clean recomputation and incremental recomputation agree.

- [ ] **Step 5: Fingerprint convergence only if public**

Do **not** fingerprint raw iteration count unless it is intentionally a user/query-visible semantic product. Fingerprint final header-derived formal products and statuses, not solver work history.

- [ ] **Step 6: Run**

```bash
cargo test -p phalcom-semantic semantic::incremental::fingerprints -- --nocapture
cargo test -p phalcom-semantic semantic::incremental::type_store_revisions -- --nocapture
cargo test -p phalcom-semantic semantic::incremental::callable_dependencies -- --nocapture
```

- [ ] **Step 7: Commit**

```bash
git add phalcom-semantic/tests/semantic/incremental \
        phalcom-semantic/src/db/fingerprint.rs
git commit -m "test(semantic): prove loop analysis deterministic incrementally"
```

---

## Task 11 — Part 05 Closure Gate

**Files:**
- Modify: `phalcom-semantic/tests/semantic/capabilities/BASELINE_LEDGER.md`
- Modify: `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` if the current ledger records these law-level additions
- No production changes unless a gate exposes a defect

**Interfaces:**
- Consumes: all Part-05 tasks.
- Produces: a clean handoff to Part 06.

- [ ] **Step 1: Run formatting**

```bash
cargo fmt --all -- --check
```

- [ ] **Step 2: Run semantic compile**

```bash
cargo check -p phalcom-semantic
```

- [ ] **Step 3: Run focused semantic suites**

```bash
cargo test -p phalcom-semantic semantic::capabilities::flow_loops -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::flow_branches -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::fields -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::patterns -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::iteration -- --nocapture
cargo test -p phalcom-semantic semantic::foundations::flow_graph -- --nocapture
```

- [ ] **Step 4: Run the whole semantic crate**

```bash
cargo test -p phalcom-semantic
```

- [ ] **Step 5: Run LSP boundary regression**

Even though Part 05 does not implement LSP semantics:

```bash
cargo check -p phalcom-lsp --lib
cargo test -p phalcom-lsp --test semantic_boundary
```

Expected: no second semantic world appears.

- [ ] **Step 6: Update capability ledgers**

Add explicit entries for:
- fixed-point convergence;
- fixed-point exhaustion;
- continue/backedge separation;
- break/exit separation;
- `while let` repeated scrutinee;
- `for` preheader-once semantics;
- probe publication isolation;
- loop incremental determinism.

Do not mark a capability complete based solely on CFG shape.

- [ ] **Step 7: Run the authority grep**

```bash
rg -n "loop_states|push_loop_frame|pop_loop_frame|record_continue|record_break|widen_loop_state" phalcom-semantic/src/checker
```

Expected after closure:
- syntax adapters do not each build ad-hoc loop join vectors;
- fixed-point orchestration is centralized in `loop_analysis.rs`;
- loop-frame capture remains context-owned;
- state widening remains `flow/state.rs`.

- [ ] **Step 8: Commit closure**

```bash
git add phalcom-semantic/tests/semantic
git commit -m "docs(semantic): record sound loop closure"
```

---

# 6. Detailed Acceptance Matrix

| Scenario | Header contributors | Post-loop contributors | Required result |
| --- | --- | --- | --- |
| body never entered | entry | false/exhaustion | entry facts preserved |
| normal body tail | entry + tail | false/exhaustion | tail feeds future header only |
| `continue` | entry + continue | false/exhaustion | continue never direct exit |
| `break` | entry + other backedges | false/exhaustion + break | break never header |
| `return` | entry + remaining backedges | false/exhaustion + breaks | return only callable exit |
| `throw` | entry + remaining backedges | false/exhaustion + breaks | throw only callable exit |
| unknown backedge | entry + unknown | false/exhaustion | unknown remains visible |
| assumed backedge | entry + assumed | false/exhaustion | no authority upgrade |
| predicate on one backedge | entry + refined lane | false/exhaustion | non-invariant predicate removed |
| field write | entry + field state | false/exhaustion | Part-02 validity preserved |
| nested inner break | inner entry | inner exit only | outer loop remains active |
| nested outer continue | outer backedge | outer false/break exits | inner state cannot retarget it |
| non-convergent | bounded candidates | conservative final exit | unstable facts `RecursiveFixpoint` |
| constant false | entry only | entry | body unreachable |
| always true + no break | entry/backedges | none | post-loop unreachable unless callable escape |

---

# 7. Diagnostics and Explanation Expectations

Part 05 must not add “loop did not converge” as a source error merely because formal precision reached the analysis bound. `Unknown(RecursiveFixpoint)` is an epistemic result.

A budget/cancellation condition remains operational:

```text
BudgetExceeded
Cancelled
```

and must not be relabeled as recursive uncertainty.

Diagnostics inside a loop body must be published once from the final semantic pass. Example:

```phalcom
while flag {
    let x: Int = "bad"
}
```

One source mismatch → one diagnostic cause, regardless of whether the header converged in one, two, or eight probes.

Explanations may include a final flow-join/fixed-point derivation node if useful, but must not expose discarded speculative iterations as separate user-facing proof chains.

Recommended final explanation step, only if explanation UX needs it:

```rust
ExplanationStep::LoopFixedPoint {
    result: TypeKnowledge,
    exhausted: bool,
}
```

Do not add this solely to count solver iterations.

---

# 8. Non-Goals

Part 05 does not:

- prove termination;
- infer integer intervals or collection cardinality;
- unroll loops using literal iteration counts;
- execute the published CFG;
- replace `FlowState` with SSA;
- redesign `TypeStore`;
- redesign predicate syntax;
- redesign `for` cursor protocol;
- add new language syntax;
- add advisory feedback into formal loop analysis;
- expose speculative solver state to hover/LSP;
- establish a type merely because a loop syntactically assigns it;
- infer “loop runs at least once” without an established proof.

---

# 9. Part 05 Completion Criteria

Part 05 is complete only when all are true:

1. Every supported loop syntax reaches one canonical fixed-point engine.
2. No syntax adapter constructs `[entry, body, continues, breaks]` as a flat post-loop join.
3. `continue` and normal completion feed headers only.
4. `break` and condition failure/exhaustion feed ordinary loop exits only.
5. return/throw exits are callable-owned and excluded from loop joins.
6. `while let` evaluates the scrutinee per logical iteration.
7. `for` evaluates source iterable expressions once.
8. convergence uses semantic fixed-point keys, not raw state equality.
9. exhaustion weakens only unstable facts to `Unknown(RecursiveFixpoint)`.
10. probes cannot duplicate diagnostics/explanations/expression products/dependencies.
11. probe interning remains in the canonical TypeStore and is idempotent.
12. field validity/initialization and binding authority remain sound through cycles.
13. CFG structural topology agrees with checker destinations.
14. clean and incremental results are semantically equivalent.
15. the entire `phalcom-semantic` suite passes.

---

# 10. Handoff Contract to Part 06

Part 06 may assume:

- evidence authority cannot be strengthened by joins/relations;
- callable return publication is certified;
- field lifecycle separates initialization from validity;
- abrupt control and executable regions are canonical;
- trusted predicates preserve authority and contradictions prune only proven-impossible paths;
- loops have sound cyclic topology and bounded fixed-point behavior;
- any remaining unsoundness should therefore be attributable to a feature-specific semantic shortcut, identity mistake, or incomplete operation implementation rather than branch/loop architecture.

Part 06 must not reopen loop topology unless a closure test demonstrates a concrete violation.
