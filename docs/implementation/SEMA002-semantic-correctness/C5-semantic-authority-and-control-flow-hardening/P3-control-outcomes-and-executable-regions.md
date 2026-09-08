# Part 03 — Canonical Control Outcomes and Executable Regions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give `phalcom-semantic` one sound structured-control execution model by making abrupt exits context-owned, making `FlowState::reachable` the single fallthrough authority, separating executable regions from closure construction, and routing ordinary branches and `if let` through one canonical region analyzer.

**Architecture:** Keep the existing AST-driven checker and `FlowState`; do not introduce a new IR or make the CFG drive checking in this part. Add one focused `checker/control.rs` module for executable-region and branch orchestration. `check_statement` reports a small `StatementControl` outcome while `CheckingContext` owns callable exits and loop-edge recording. Executable regions run statements in lexical scope, preserve outer mutations, stop on unreachable flow, and expose a value only for a normal completion. `Expr::Block` remains closure construction and is never reused to execute control-flow bodies.

**Tech Stack:** Rust, `phalcom-semantic`, `FlowState`, `TypedExpression`, `AnalysisStatus`, `CausalInvalidity`, Part-1 `NormalReturnFact`, Part-2 field flow state, `CheckingContext`, `ExpectedType`, existing semantic capability fixtures, deterministic semantic fingerprints.

**Spec:** This plan implements Part 3 of the ratified six-part typing-correctness architecture. It depends on Part 1 — Evidence Authority and Callable Contract Certification — and Part 2 — Field Contracts and Constructor Lifecycle Correctness. The implementation source of truth is `aureat/phalcom-lang` `main` at the grounding revision below, rebased through Parts 1 and 2 before execution.

## Repository grounding

Freshly grounded against `main` at:

```text
24fc9fd98f3c3c534c4d52b613962a39b9374185
feat(semantic): add rich type diagnostics tests and polish presentation
```

Current control-flow anchors at that revision:

- `phalcom-semantic/src/checker/body.rs`
  - owns a local `can_fall_through` boolean in addition to `ctx.flow.reachable`;
  - directly collects normal return values from only the statements visible to the callable-body loop;
  - tail expressions are handled separately from ordinary statements.
- `phalcom-semantic/src/checker/statement.rs`
  - `check_statement(...) -> Option<TypeKnowledge>` at the grounding revision; Part 1 changes this to structured `NormalReturnFact` before Part 3 begins;
  - `return` reports upward to its immediate caller but does not atomically mark flow unreachable;
  - `throw`, `break`, and `continue` use separate context operations, with `break`/`continue` explicitly marking flow unreachable in `statement.rs`.
- `phalcom-semantic/src/checker/context.rs`
  - stores `throw_exit_flows` and loop frames;
  - exposes `record_throw_exit()`, `record_break()`, and `record_continue()` style operations;
  - Part 1 introduces structured normal-return facts but intentionally does not remove `body.rs`'s fallthrough architecture.
- `phalcom-semantic/src/checker/expression.rs`
  - sacred paired `ifTrue(..., ifFalse: ...)` manually forks flow, applies predicates, executes `analyze_control_block`, and joins;
  - `analyze_control_block` is a private second statement interpreter with special cases for `return`, `throw`, `break`, and `continue`;
  - `Expr::Block` is closure construction and intentionally restores `outer_flow` after checking the closure body;
  - `Expr::IfLet` incorrectly executes `then_body` and `else_body` by wrapping them in `Expr::Block`, so branch bodies inherit closure-construction semantics and outer mutations can be discarded;
  - `Expr::WhileLet` executes its body once and is intentionally deferred to Part 5 for real loop semantics.
- `phalcom-semantic/src/checker/flow/state.rs`
  - `FlowState::reachable` already exists and `join_with_hierarchy` excludes unreachable predecessors;
  - this is the correct primitive to become the sole local fallthrough authority.
- `phalcom-semantic/src/checker/flow/graph.rs`
  - already represents return/throw/break/continue/backedge structure;
  - this plan does not make it executable or authoritative.
- `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs`
  - already contains branch join, abrupt-arm, nested-return, shadowing, and unknown-arm composition coverage.
- `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`
  - already contains break/continue and closure-construction regressions;
  - precise loop semantics remain Part 5.

All exact line numbers must be re-resolved after Parts 1 and 2 land. Symbol responsibilities in this plan are normative; stale line offsets are not.

---

# 1. Dependency gates

Part 3 starts only after Parts 1 and 2 are merged and green.

Part 1 must provide the approved equivalent of:

```rust
pub struct NormalReturnFact {
    pub knowledge: TypeKnowledge,
    pub flow: FlowStateSummary,
    pub status: AnalysisStatus,
    pub causal_invalidity: CausalInvalidity,
}
```

and `BodyExitFacts` must contain real return-site flow snapshots rather than synthetic entry-flow summaries.

Part 2 must leave field state path-sensitive and joinable through ordinary `FlowState`. Part 3 must not add a control-specific field side channel.

Before implementation, run:

```sh
git status --short
git rev-parse HEAD
cargo test -p phalcom-semantic --test semantic authority -- --nocapture
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
```

Expected: Parts 1 and 2 are GREEN; existing control tests establish the baseline behavior/failures without assertion weakening.

---

# 2. Problem statement

The checker currently has several independent representations of “this path does not continue”:

```text
body.rs::can_fall_through
FlowState::reachable
Never-valued TypedExpression
analyze_control_block's local break logic
throw exit recording
loop-frame break/continue recording
Option<return-value> from check_statement
```

Each is individually understandable, but the combination allows semantic drift. The most serious consequence is that nested abrupt control is not naturally a callable-level fact: a `return` inside a branch is observed by the private block interpreter rather than by the callable body collector.

There is a second architectural confusion: an AST block literal is a value-producing closure, while a branch body is an executable lexical region. Those are not the same semantic operation.

```phalcom
let f = || {
  x = 2
}
```

constructs a closure and must not mutate `x` now.

```phalcom
if condition {
  x = 2
}
```

executes the body on the selected path and the outer mutation must participate in the post-branch flow join.

`Expr::IfLet` currently routes its bodies through `Expr::Block`, which crosses exactly this semantic boundary.

---

# 3. Required semantic laws

## Law C1 — Reachability is the sole local fallthrough authority

`FlowState::is_reachable()` answers whether sequential execution in the current region can continue. Delete `body.rs::can_fall_through` and do not replace it with another parallel boolean.

`Never` is a value-level/type-level fact, not the primary control flag.

## Law C2 — Abrupt control is recorded atomically

Executing one of:

```text
return
throw
break
continue
```

must perform both parts of the transition in one context-owned operation:

1. record the corresponding exit/edge state before termination;
2. mark the current flow unreachable.

No caller should be required to remember the second step.

## Law C3 — Callable exits are discovered regardless of nesting depth

A `return` inside an `if`, `if let`, future loop body, or nested executable region contributes to the current callable's normal-return facts exactly once.

Top-level body traversal must not be the only mechanism that discovers returns.

## Law C4 — Closure construction and control-region execution are distinct

`Expr::Block` remains closure construction. Executable branches must not create an `Expr::Block` wrapper to reuse closure analysis.

## Law C5 — Executable regions preserve outer mutation

A branch region gets a forked `FlowState`. Writes to pre-existing bindings/fields survive as path state. Lexically declared region-local bindings do not escape the region scope.

## Law C6 — Abrupt regions have no normal value

If a region terminates by `return`, `throw`, `break`, `continue`, or contradiction/unreachability, it contributes no value to an enclosing branch expression's normal value join.

## Law C7 — Normal region value follows statement semantics

For a reachable region:

- final expression statement -> that expression's `TypedExpression`;
- final declaration/assignment/ordinary statement -> established `Unit` unless that statement made flow unreachable;
- empty region -> established `Unit`.

## Law C8 — Branch joins use normal-completion predecessors only

Only branch flows that remain reachable after executing the branch body participate in the continuing flow join. Abrupt branch states have already been recorded at their proper destination.

## Law C9 — Lexical scope and flow identity remain separate

Popping a lexical scope removes name visibility but does not retroactively undo writes to an outer binding. Branch-local bindings remain in history for semantic inspection but cannot be looked up outside their scope.

## Law C10 — Part 3 does not invent predicate proof

Part 3 may continue calling the existing trusted-predicate API to preserve behavior, but it does not change predicate authority, contradiction semantics, or constant branch pruning. Those are Part 4.

## Law C11 — Part 3 does not solve loops

`break`/`continue` recording is centralized now because all abrupt control needs one ownership model. Loop topology, zero-iteration semantics, backedges, and fixed-point/widening rules remain Part 5.

## Law C12 — CFG remains descriptive

The existing `FlowGraph` may be published and tested, but body checking is not rewritten around CFG execution in this part.

---

# 4. Target architecture

## 4.1 `StatementControl`

Add a small statement transfer result in `checker/control.rs` or `checker/statement.rs` (prefer `control.rs` so the vocabulary is centralized):

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StatementControl {
    FallsThrough,
    Return,
    Throw,
    Break,
    Continue,
}

impl StatementControl {
    pub const fn is_abrupt(self) -> bool {
        !matches!(self, Self::FallsThrough)
    }
}
```

This enum is control metadata only. It does not carry the return value; `CheckingContext` owns the recorded `NormalReturnFact` from Part 1.

Do not use `Continue` to mean ordinary fallthrough. The explicit `FallsThrough` name avoids collision with the language `continue` statement.

## 4.2 Context-owned exit accumulators

After Part 1, move the final ownership of callable normal returns into `CheckingContext` beside throw exits:

```rust
pub struct CheckingContext<'a> {
    // ...
    normal_return_exits: Vec<NormalReturnFact>,
    throw_exit_flows: Vec<FlowStateSummary>,
    pub(crate) loop_frames: Vec<LoopFlowFrame>,
    // ...
}
```

Add atomic methods:

```rust
pub(crate) fn record_return_exit(&mut self, fact: NormalReturnFact) {
    self.normal_return_exits.push(fact);
    self.flow.mark_unreachable();
}

pub(crate) fn record_throw_exit_and_terminate(&mut self) {
    self.throw_exit_flows.push(self.current_flow_summary());
    self.flow.mark_unreachable();
}

pub(crate) fn record_break_and_terminate(&mut self) {
    self.record_break(); // captures current flow in innermost loop frame
    self.flow.mark_unreachable();
}

pub(crate) fn record_continue_and_terminate(&mut self) {
    self.record_continue(); // captures current flow in innermost loop frame
    self.flow.mark_unreachable();
}
```

If Parts 1/2 changed names, keep this semantic contract rather than these exact spellings.

The return fact's `flow` must be captured before `mark_unreachable()` so the exit snapshot retains bindings/fields at the return site.

## 4.3 Executable-region result

Create `checker/control.rs` with:

```rust
#[derive(Clone, Debug)]
pub(crate) struct ExecutableRegionResult {
    /// Present only when the region has a reachable normal completion.
    pub value: Option<TypedExpression>,
    /// Current region flow after execution. Unreachable for abrupt-only completion.
    pub flow: FlowState,
    pub causal_invalidity: CausalInvalidity,
}

impl ExecutableRegionResult {
    pub fn completes_normally(&self) -> bool {
        self.value.is_some() && self.flow.is_reachable()
    }
}
```

The main API:

```rust
pub(crate) fn analyze_executable_region(
    ctx: &mut CheckingContext<'_>,
    statements: &[Statement],
    range: SourceRange,
    expected: &ExpectedType,
) -> ExecutableRegionResult;
```

Contract:

1. push lexical scope;
2. execute in order while `ctx.flow.is_reachable()`;
3. analyze the final expression statement with `expected`;
4. ordinary final statements yield `Unit` only if flow remains reachable;
5. abrupt statement outcomes stop immediately;
6. pop lexical scope;
7. return the resulting flow and optional normal value;
8. never restore the entry flow automatically.

The caller owns branch forking and later joins.

## 4.4 Branch-pair orchestration

Add:

```rust
pub(crate) struct BranchPairResult {
    pub typed: TypedExpression,
    pub then_flow: FlowState,
    pub else_flow: FlowState,
}

pub(crate) fn analyze_branch_pair(
    ctx: &mut CheckingContext<'_>,
    condition: &Expr,
    condition_typed: &TypedExpression,
    then_body: &[Statement],
    then_range: SourceRange,
    else_body: Option<(&[Statement], SourceRange)>,
    whole_range: SourceRange,
    expected: &ExpectedType,
) -> BranchPairResult;
```

Part 3 behavior:

```text
entry = ctx.flow.clone()

then:
    ctx.flow = entry.clone()
    apply existing trusted true predicate if available
    execute region

else:
    ctx.flow = entry.clone()
    apply existing trusted false predicate if available
    execute region, or synthesize Unit from entry when no else

join only reachable normal-completion flows
join only values from reachable normal-completion regions
```

The branch result must propagate causal invalidity from the condition and both analyzed regions without allowing an abrupt region's value to pollute the value join.

Do not add constant-condition pruning here; Part 4 adds `ConditionTruth` to this function.

## 4.5 `if let` execution

Do not model `if let` as a type predicate yet. Part 3 fixes execution semantics only:

```text
1. analyze scrutinee exactly once
2. save entry flow
3. then path:
   - restore/fork entry
   - push region scope
   - bind pattern in that scope using scrutinee fact
   - execute then statements as an executable region
4. else path:
   - restore/fork original entry
   - execute else region, or Unit/no-op normal completion
5. join normal flows and normal values
```

Pattern binding must exist only in the success branch lexical scope. Outer mutations must survive.

The exact success/failure possibility of a refutable pattern is not solved here. Part 4 may mark impossible paths when supported by formal predicate/pattern knowledge; Part 3 simply establishes correct region mechanics.

---

# 5. File/ownership map

| Area | File | Responsibility after Part 3 |
|---|---|---|
| Statement semantic transfer | `checker/statement.rs` | Analyze one statement, invoke atomic abrupt operations, return `StatementControl` |
| Structured control orchestration | **new** `checker/control.rs` | Executable region, branch pair, `if let` region execution, normal-value selection |
| Checker state | `checker/context.rs` | Own callable exits, loop edge capture, reachability termination |
| Callable body | `checker/body.rs` | Seed callable context, sequentially check top-level statements, synthesize reachable tail/Unit, finalize context-owned exits |
| Expression synthesis | `checker/expression.rs` | Delegate structured control to `control.rs`; preserve closure construction in `Expr::Block` |
| Flow state | `checker/flow/state.rs` | Remain the sole reachability/join primitive |
| Public analysis | `checker/analysis.rs` | Consume context-owned Part-1 normal returns; no new parallel control state |
| Module exports | `checker/mod.rs` | Declare `pub(crate) mod control` |
| Fingerprinting | `db/fingerprint.rs` | Only update if final Part-3 product shape changes |
| Branch tests | `tests/semantic/capabilities/flow_branches.rs` | Existing and new executable-region regressions |
| Control-specific tests | **new** `tests/semantic/capabilities/control_regions.rs` | `if let`, nested exits, reachability, closure-vs-region laws |
| Test module wiring | `tests/semantic/capabilities/mod.rs` | Add `mod control_regions;` |

Do not move predicate transfer into `control.rs`; predicate semantics remain in `checker/flow/*` and are upgraded in Part 4.

---

# 6. Execution order

```text
Task 0  Rebase/gate on Parts 1–2
   |
Task 1  Introduce StatementControl + atomic abrupt transitions
   |
Task 2  Make context own all callable normal-return exits
   |
Task 3  Add canonical executable-region analyzer
   |
Task 4  Move paired branch execution to control.rs
   |
Task 5  Repair if-let using executable regions
   |
Task 6  Remove duplicate fallthrough/control interpreters
   |
Task 7  Composition, fingerprints, full closure gate
```

Each task must be independently reviewable and GREEN before the next one starts.

---

## Task 0: Rebase onto Parts 1 and 2 and freeze the control baseline

**Files:**
- Read: Part 1 implementation and final plan.
- Read: Part 2 implementation and final plan.
- Read: `phalcom-semantic/src/checker/body.rs`
- Read: `phalcom-semantic/src/checker/statement.rs`
- Read: `phalcom-semantic/src/checker/context.rs`
- Read: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_loops.rs`

**Interfaces:**
- Consumes: post-Part-2 `NormalReturnFact`, field flow state, return-contract validation.
- Produces: an implementation work-log note naming exact HEAD and any renamed Part-1/2 anchors.

- [ ] **Step 1: Verify clean baseline.**

```sh
git status --short
git rev-parse HEAD
git log -1 --oneline
```

Expected: no unrelated changes.

- [ ] **Step 2: Locate all duplicate control authorities.**

```sh
rg "can_fall_through|mark_unreachable\(|record_throw_exit|record_break\(|record_continue\(|analyze_control_block|Expr::Block\(Box::new\(if_let" phalcom-semantic/src/checker
```

Record the exact post-Part-2 call sites. The final Task 6 scan must eliminate the obsolete ones.

- [ ] **Step 3: Run the existing branch/loop baseline.**

```sh
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_loops -- --nocapture
```

Do not weaken currently passing abrupt-branch tests. Capture failures that specifically involve nested returns or `if let` mutation for the RED tests below.

---

## Task 1: Introduce `StatementControl` and atomic abrupt transitions

**Files:**
- Create: `phalcom-semantic/src/checker/control.rs`
- Modify: `phalcom-semantic/src/checker/mod.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Test: new unit tests in `checker/control.rs`/`checker/context.rs` as appropriate.

**Interfaces:**
- Consumes: Part-1 `NormalReturnFact`, `FlowState`, existing loop frames.
- Produces: `StatementControl`, atomic context termination methods.

- [ ] **Step 1: Add low-level RED tests for atomic termination.**

Test the context operations directly. The shape should be equivalent to:

```rust
#[test]
fn return_exit_records_flow_before_terminating_path() {
    let mut ctx = fixture_context();
    let fact = ready_return_fact(&ctx, int_knowledge(&mut ctx));

    ctx.record_return_exit(fact);

    assert_eq!(ctx.normal_return_exits().len(), 1);
    assert!(!ctx.flow.is_reachable());
    assert!(!ctx.normal_return_exits()[0].flow.bindings.is_empty());
}
```

Also add:

```text
throw records one throw exit and terminates
break captures one innermost-loop break flow and terminates
continue captures one innermost-loop continue flow and terminates
```

For break/continue outside a loop, preserve the repository's existing diagnostic/internal-failure policy; do not silently fabricate a loop frame.

- [ ] **Step 2: Add `StatementControl`.**

Implement the enum from the target architecture and wire `pub(crate) mod control;` in `checker/mod.rs`.

- [ ] **Step 3: Make abrupt context operations atomic.**

Move all `mark_unreachable()` calls associated with abrupt statements into the context operation itself. Capture the flow snapshot before terminating.

- [ ] **Step 4: Change `check_statement` to return `StatementControl`.**

For ordinary statements:

```rust
StatementControl::FallsThrough
```

For `return`:

1. analyze return expression and contract relation exactly as Part 1 requires;
2. build `NormalReturnFact`;
3. call `ctx.record_return_exit(fact)`;
4. return `StatementControl::Return`.

For throw/break/continue, invoke the atomic context operation and return the matching variant.

Do not return a normal return fact to the caller anymore; ownership has moved into `CheckingContext`.

- [ ] **Step 5: Compile-migrate statement callers without semantic refactoring yet.**

Temporarily adapt `body.rs`, `expression.rs`, and loop callers to inspect `StatementControl`. Do not yet delete `analyze_control_block` or `can_fall_through`; those are removed after the canonical replacement exists.

- [ ] **Step 6: Run focused tests.**

```sh
cargo test -p phalcom-semantic checker::context -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
```

- [ ] **Step 7: Commit.**

```sh
git add phalcom-semantic/src/checker/control.rs \
        phalcom-semantic/src/checker/mod.rs \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/checker/statement.rs \
        phalcom-semantic/src/checker/body.rs \
        phalcom-semantic/src/checker/expression.rs
git commit -m "refactor(semantic): centralize abrupt statement outcomes"
```

---

## Task 2: Make `CheckingContext` the sole owner of callable normal-return exits

**Files:**
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/body.rs`
- Modify: `phalcom-semantic/src/checker/analysis.rs` only if finalization signatures need adjustment.
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs`
- Test: new `control_regions.rs`.

**Interfaces:**
- Consumes: atomic `record_return_exit`, Part-1 `NormalReturnFact`.
- Produces: finalization that drains context-owned exits; no body-local normal-return accumulator.

- [ ] **Step 1: Add RED nested-return test.**

Create `tests/semantic/capabilities/control_regions.rs`, wire it in `capabilities/mod.rs`, and add:

```rust
#[test]
fn nested_return_is_recorded_once_as_callable_exit() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    if flag {
      return 1
    }
    return 2
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(run.exits.normal_returns.len(), 2, "each source return path must be recorded exactly once");
    assert!(run.exits.normal_returns.iter().all(|exit| exit.knowledge.ty() == Some(f.ty("Int"))));
}
```

If Part 4 later proves constant/impossible paths, this test remains valid because `flag` is not constant.

- [ ] **Step 2: Add nested throw/return separation test.**

```rust
#[test]
fn nested_throw_is_not_misclassified_as_normal_return() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    if flag { throw "bad" }
    return 1
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    assert_eq!(run.exits.throws.len(), 1);
    assert_eq!(run.exits.normal_returns.len(), 1);
}
```

- [ ] **Step 3: Remove body-local return collection.**

Delete the local `normal_returns` vector introduced by Part 1. `body.rs` must no longer discover explicit returns by receiving values from `check_statement`.

- [ ] **Step 4: Add context access/drain used only at finalization.**

Prefer a consuming method:

```rust
fn take_normal_return_exits(&mut self) -> Vec<NormalReturnFact> {
    std::mem::take(&mut self.normal_return_exits)
}
```

Do not expose public mutation of the vector.

- [ ] **Step 5: Preserve implicit tail/Unit returns through the same context operation.**

A callable's reachable tail expression is a normal exit even without `return` syntax. Create its `NormalReturnFact` and call `record_return_exit`.

An empty/reachable callable body and a reachable non-expression tail likewise record established `Unit` through the same path.

- [ ] **Step 6: Finalize from context-owned exits.**

`finalize`/`finalize_with...` consumes `normal_return_exits` and `throw_exit_flows` directly.

There must be no second normal-return collection in `body.rs` or `analysis.rs`.

- [ ] **Step 7: Run.**

```sh
cargo test -p phalcom-semantic --test semantic nested_return_is_recorded_once_as_callable_exit -- --nocapture
cargo test -p phalcom-semantic --test semantic nested_throw_is_not_misclassified_as_normal_return -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
```

- [ ] **Step 8: Commit.**

```sh
git add phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/checker/body.rs \
        phalcom-semantic/src/checker/analysis.rs \
        phalcom-semantic/tests/semantic/capabilities/control_regions.rs \
        phalcom-semantic/tests/semantic/capabilities/mod.rs
git commit -m "refactor(semantic): record callable exits in checking context"
```

---

## Task 3: Add the canonical executable-region analyzer

**Files:**
- Modify: `phalcom-semantic/src/checker/control.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs` only for reusable helpers if necessary.
- Test: `phalcom-semantic/tests/semantic/capabilities/control_regions.rs`

**Interfaces:**
- Consumes: `StatementControl`, `CheckingContext`, `ExpectedType`, `TypedExpression`.
- Produces: `ExecutableRegionResult`, `analyze_executable_region`.

- [ ] **Step 1: Add RED empty/tail/unit region unit tests.**

Test the region helper through source integration where practical:

```phalcom
if flag { }
if flag { 1 } else { 2 }
if flag { let x = 1 } else { let x = 2 }
```

Assertions:

```text
empty reachable region -> Unit
reachable final expression -> expression value
reachable final let -> Unit
```

- [ ] **Step 2: Add RED abrupt-region test.**

```rust
#[test]
fn abrupt_region_has_no_normal_value() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = if flag { return 1 } else { "ok" }
    x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_binding_established(run, "x", f.ty("String"));
}
```

This test may already pass through legacy code; it remains a required invariant test after refactoring.

- [ ] **Step 3: Implement `ExecutableRegionResult`.**

Use the target struct. `value == None` is permitted only when no reachable normal completion remains.

- [ ] **Step 4: Implement `analyze_executable_region`.**

Pseudocode:

```rust
pub(crate) fn analyze_executable_region(...) -> ExecutableRegionResult {
    ctx.push_scope();
    let mut last = None;
    let mut causal = CausalInvalidity::Clean;

    for (index, statement) in statements.iter().enumerate() {
        if !ctx.flow.is_reachable() {
            break;
        }

        let is_tail = index + 1 == statements.len();
        match statement {
            Statement::Expr { expr, .. } if is_tail => {
                let typed = analyze_expression(ctx, expr, expected);
                causal = causal.join(typed.causal_invalidity);
                if ctx.flow.is_reachable() && typed.knowledge.ty() != Some(ctx.store.never()) {
                    last = Some(typed);
                }
            }
            _ => {
                let control = check_statement(ctx, statement);
                if control.is_abrupt() || !ctx.flow.is_reachable() {
                    last = None;
                    break;
                }
                if is_tail {
                    last = Some(TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, range));
                }
            }
        }
    }

    if statements.is_empty() && ctx.flow.is_reachable() {
        last = Some(TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, range));
    }

    let flow = ctx.flow.clone();
    ctx.pop_scope();
    ExecutableRegionResult { value: last.filter(|_| flow.is_reachable()), flow, causal_invalidity: causal }
}
```

Adjust causal propagation to include statement-owned invalidity already recorded in flow/bindings; do not invent a second diagnostic-cause accumulator if existing typed/context state supplies it.

- [ ] **Step 5: Ensure unreachable trailing statements are not semantically executed.**

Add a source regression:

```phalcom
if flag {
  return 1
  let impossible = mystery()
}
```

Assert no binding named `impossible` is published for the callable and no `UnresolvedName` diagnostic is emitted for `mystery()` from that unreachable trailing statement.

This intentionally chooses “do not semantically execute dead trailing statements” for Part 3. Dedicated unreachable-code diagnostics can be a later feature.

- [ ] **Step 6: Run and commit.**

```sh
cargo test -p phalcom-semantic --test semantic control_regions -- --nocapture
git add phalcom-semantic/src/checker/control.rs phalcom-semantic/tests/semantic/capabilities/control_regions.rs
git commit -m "feat(semantic): add executable control regions"
```

---

## Task 4: Move paired branch execution onto the canonical control module

**Files:**
- Modify: `phalcom-semantic/src/checker/control.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/flow_branches.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/control_regions.rs`

**Interfaces:**
- Consumes: `analyze_executable_region`, existing `extract_trusted_predicate`, `CheckingContext::apply_flow_predicate`, `join_type_knowledge`.
- Produces: `analyze_branch_pair`.

- [ ] **Step 1: Strengthen RED/guard tests around outer mutation.**

Add:

```rust
#[test]
fn executable_branch_mutation_survives_region_scope() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ flag: Bool) {
    let x = 1
    if flag { x = "changed" } else { x = 2 }
    let observed = x
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_union_members(
        f.binding(run, "observed").current.ty().expect("joined mutation"),
        &[f.ty("Int"), f.ty("String")],
    );
}
```

Existing `branch_local_shadow_does_not_mutate_outer_binding_flow` remains mandatory.

- [ ] **Step 2: Implement `analyze_branch_pair`.**

Use a single entry `FlowState` fork. Run then/else with `analyze_executable_region`.

For a missing else:

```rust
let else_result = ExecutableRegionResult {
    value: Some(TypedExpression::established(ctx.store.unit(), EvidenceOrigin::Flow, whole_range)),
    flow: entry.clone(),
    causal_invalidity: CausalInvalidity::Clean,
};
```

Apply the existing trusted predicate before each branch, preserving current Part-3 behavior. Part 4 changes the transfer result/authority.

- [ ] **Step 3: Join only normal flows.**

Build:

```rust
let normal_flows = [then_result, else_result]
    .iter()
    .filter(|result| result.completes_normally())
    .map(|result| result.flow.clone())
    .collect::<Vec<_>>();
```

If no normal flow remains, set `ctx.flow = FlowState::unreachable()` and produce `Never` for the enclosing expression.

If one normal flow remains, install it directly or pass it through `join_flow_states` consistently.

If multiple remain, use `ctx.join_flow_states`.

- [ ] **Step 4: Join only normal values.**

Do not call `join_type_knowledge` over both branch values unconditionally as the current `ifTrue` code does.

```rust
let values = results.iter().filter_map(|result| {
    result.completes_normally().then(|| result.value.as_ref()?.knowledge.clone())
});
```

No values -> `Never`.

One/more -> canonical `join_type_knowledge`.

- [ ] **Step 5: Preserve explanation data.**

The existing `BranchJoin` explanation must record reachability accurately. Its explanation node's own status must be derived from the joined knowledge, not hard-coded `Established`.

Use:

```rust
knowledge.status().unwrap_or(EvidenceStatus::Assumed)
```

and preserve branch explanation parents.

- [ ] **Step 6: Replace sacred `ifTrue` manual orchestration.**

In `expression.rs`, the `ifTrue` + `ifFalse` sacred control case should delegate branch execution to `control::analyze_branch_pair`.

Delete the duplicated fork/join/value logic from that match arm.

- [ ] **Step 7: Run focused branch suite.**

```sh
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
cargo test -p phalcom-semantic --test semantic control_regions -- --nocapture
```

- [ ] **Step 8: Commit.**

```sh
git add phalcom-semantic/src/checker/control.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/capabilities/flow_branches.rs \
        phalcom-semantic/tests/semantic/capabilities/control_regions.rs
git commit -m "refactor(semantic): route branches through executable regions"
```

---

## Task 5: Repair `if let` by executing branch regions instead of constructing closures

**Files:**
- Modify: `phalcom-semantic/src/checker/control.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/control_regions.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/patterns.rs` where existing pattern helpers are exercised.

**Interfaces:**
- Consumes: `bind_pattern`, `analyze_executable_region`, `join_flow_states`, `TypedExpression.fact()`.
- Produces: `analyze_if_let` helper or equivalent branch orchestration with a success-scope hook.

- [ ] **Step 1: Add RED outer-mutation test.**

```rust
#[test]
fn if_let_success_region_preserves_outer_mutation() {
    let f = Fixture::new(
        r#"
class Probe {
  @class
  run(_ input: Int) {
    let observed = 0
    if let value = input {
      observed = "matched"
    } else {
      observed = 2
    }
    let result = observed
  }
}
"#,
    );
    let run = f.callable("Probe", "run", DispatchSide::Class);
    f.assert_union_members(
        f.binding(run, "result").current.ty().expect("if-let outer mutation join"),
        &[f.ty("Int"), f.ty("String")],
    );
}
```

Use a parser-valid refutable pattern fixture if simple name binding is irrefutable in the final grammar; the semantic assertion is the outer mutation, not pattern refutability.

- [ ] **Step 2: Add RED scope test.**

```phalcom
if let value = input {
  let inside = value
}
let outside = value
```

Assert `inside` resolves to the branch binding and `outside` is unresolved. The pattern binding must not leak.

- [ ] **Step 3: Add RED branch-value test.**

```phalcom
let x = if let value = input {
  1
} else {
  "none"
}
```

Expected: `Int | String` when both paths are conservatively reachable in Part 3.

- [ ] **Step 4: Analyze the scrutinee exactly once.**

`Expr::IfLet` must call `analyze_expression` on `if_let.value` once before branch forking. Do not reanalyze per branch.

- [ ] **Step 5: Execute the success region with pattern binding in its lexical scope.**

Do **not** construct:

```rust
Expr::Block(Box::new(if_let.then_body.clone()))
```

Instead add a control helper that can run a prelude immediately after `push_scope()` and before region statements, or manually follow the same canonical region steps without duplicating the statement loop.

Recommended small extension:

```rust
pub(crate) fn analyze_executable_region_with_prelude(
    ctx: &mut CheckingContext<'_>,
    statements: &[Statement],
    range: SourceRange,
    expected: &ExpectedType,
    prelude: impl FnOnce(&mut CheckingContext<'_>),
) -> ExecutableRegionResult
```

and let `analyze_executable_region` call it with an empty prelude.

The prelude is only for lexical binding setup such as `if let`; it is not a general callback framework.

- [ ] **Step 6: Execute else/no-else from the original flow.**

The failure branch must not inherit the success pattern binding or success mutations.

No else -> normal `Unit` value with the unmodified entry flow.

- [ ] **Step 7: Join using the same normal-flow/value rules as ordinary branches.**

Extract the common join helper from `analyze_branch_pair` if necessary:

```rust
fn join_branch_results(
    ctx: &mut CheckingContext<'_>,
    results: &[ExecutableRegionResult],
    range: SourceRange,
) -> TypedExpression
```

Do not duplicate join semantics for `if let`.

- [ ] **Step 8: Run.**

```sh
cargo test -p phalcom-semantic --test semantic control_regions -- --nocapture
cargo test -p phalcom-semantic --test semantic patterns -- --nocapture
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
```

- [ ] **Step 9: Commit.**

```sh
git add phalcom-semantic/src/checker/control.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/capabilities/control_regions.rs \
        phalcom-semantic/tests/semantic/capabilities/patterns.rs
git commit -m "fix(semantic): execute if-let bodies as control regions"
```

---

## Task 6: Remove duplicate fallthrough and control-block interpreters

**Files:**
- Modify: `phalcom-semantic/src/checker/body.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Modify: `phalcom-semantic/src/checker/statement.rs`
- Test: all Part-3 focused suites.

**Interfaces:**
- Consumes: context-owned exits, `StatementControl`, executable regions.
- Produces: one reachability authority and one executable-region statement loop.

- [ ] **Step 1: Delete `body.rs::can_fall_through`.**

Rewrite top-level body traversal around:

```rust
if !ctx.flow.is_reachable() {
    break;
}
```

When a statement is abrupt, the atomic context operation has already terminated the flow.

- [ ] **Step 2: Delete `analyze_control_block`.**

All callers must use `checker/control.rs`.

- [ ] **Step 3: Remove caller-side abrupt `mark_unreachable()` calls.**

Run:

```sh
rg "mark_unreachable\(" phalcom-semantic/src/checker
```

Allowed after Part 3:

```text
context-owned atomic abrupt operations
Part-4 predicate contradiction code when it lands
carefully justified direct control primitives
```

`statement.rs`, branch orchestration, and legacy block interpreters must not redundantly terminate paths.

- [ ] **Step 4: Remove return-value signalling from statement callers.**

Run:

```sh
rg "check_statement\([^\n]*\).*Some|let .* = check_statement|if let Some\(.*check_statement" phalcom-semantic/src/checker
```

There should be no normal-return collection based on `check_statement` return payloads.

- [ ] **Step 5: Confirm closure semantics are unchanged.**

Keep the existing behavior in `Expr::Block` that restores `outer_flow` after checking the closure body.

Run:

```sh
cargo test -p phalcom-semantic --test semantic captured_block_write_is_not_applied_until_execution_is_proven -- --nocapture
```

- [ ] **Step 6: Run focused closure.**

```sh
cargo test -p phalcom-semantic --test semantic flow_branches -- --nocapture
cargo test -p phalcom-semantic --test semantic control_regions -- --nocapture
cargo test -p phalcom-semantic --test semantic callable_publication -- --nocapture
cargo test -p phalcom-semantic --test semantic fields -- --nocapture
```

- [ ] **Step 7: Commit.**

```sh
git add phalcom-semantic/src/checker/body.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/src/checker/statement.rs
git commit -m "refactor(semantic): make flow reachability canonical"
```

---

## Task 7: Composition, semantic fingerprints, and final Part-3 closure gate

**Files:**
- Modify: `phalcom-semantic/src/db/fingerprint.rs` only if Part-3 product fields changed from the Part-1/2 shape.
- Modify: `phalcom-semantic/tests/semantic/capabilities/control_regions.rs`
- Modify: `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` if that ledger tracks these capabilities.

**Interfaces:**
- Consumes: complete Part-3 architecture.
- Produces: regression suite that proves nested exits, branch values, outer mutations, and closure isolation compose.

- [ ] **Step 1: Add a composed branch/return/field test.**

```rust
#[test]
fn branch_return_and_field_mutation_keep_separate_exit_facts() {
    let f = Fixture::new(
        r#"
class Box {
  value: Int = 0

  update(_ flag: Bool) -> Int {
    if flag {
      value = 1
      return value
    } else {
      value = 2
    }
    value
  }
}
"#,
    );
    let update = f.callable("Box", "update", DispatchSide::Instance);
    assert_eq!(update.exits.normal_returns.len(), 2);
    assert!(update.exits.normal_returns.iter().all(|exit| exit.knowledge.ty() == Some(f.ty("Int"))));
    assert!(update.exits.normal_returns.iter().all(|exit| {
        exit.flow.fields.values().any(|field| field.current.ty() == Some(f.ty("Int")))
    }));
}
```

This directly validates the Part-1/2/3 composition boundary.

- [ ] **Step 2: Add nested control + shadowing composition.**

```phalcom
let x = 0
if outer {
  let x = "shadow"
  if inner { return 1 }
} else {
  x = 2
}
let y = x
```

Expected: outer `x`/`y` remain `Int`; shadow is distinct; nested return recorded once.

- [ ] **Step 3: Verify semantic fingerprints.**

If no analysis product field changed in Part 3, do not churn `db/fingerprint.rs`.

If `StatementControl` and executable-region results are ephemeral checker state only, they must not be fingerprinted.

If context-owned exits changed serialized `BodyExitFacts` shape beyond Part 1, hash the exact semantic fields and exclude allocator-local IDs as existing fingerprint code does.

- [ ] **Step 4: Full semantic test gate.**

```sh
cargo fmt --all -- --check
cargo test -p phalcom-semantic --test semantic -- --nocapture
cargo test -p phalcom-semantic
cargo clippy -p phalcom-semantic --all-targets -- -D warnings
```

Expected: GREEN. Do not accept “new tests pass but old tests fail.”

- [ ] **Step 5: Architecture scan.**

```sh
rg "can_fall_through|analyze_control_block|Expr::Block\(Box::new\(if_let" phalcom-semantic/src/checker
```

Expected: no matches.

```sh
rg "mark_unreachable\(" phalcom-semantic/src/checker
```

Expected: only context-owned abrupt termination and other explicitly justified canonical control primitives.

- [ ] **Step 6: Commit final tests/docs.**

```sh
git add phalcom-semantic/tests/semantic/capabilities/control_regions.rs \
        phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md \
        phalcom-semantic/src/db/fingerprint.rs
git commit -m "test(semantic): close executable control region invariants"
```

---

# 7. Acceptance test matrix

The following cases are mandatory before Part 3 is complete.

| Scenario | Required outcome |
|---|---|
| top-level `return 1` | one normal return fact; current path unreachable |
| return inside one branch | return recorded exactly once even though nested |
| throw inside branch | throw exit recorded; no normal value from arm |
| break inside loop body | innermost loop break flow captured before path termination |
| continue inside loop body | innermost loop continue flow captured before path termination |
| statement after abrupt exit | not semantically executed |
| branch final expression | contributes value when branch completes normally |
| branch final `let` | contributes Unit when branch completes normally |
| abrupt branch + normal branch | only normal branch contributes expression value |
| both abrupt branches | expression result `Never`; continuing flow unreachable |
| branch outer assignment | survives into branch flow and post-join |
| branch-local shadow | does not replace outer binding identity |
| closure body outer assignment | does not execute at closure construction |
| `if let` success mutation | participates in outer post-branch join |
| `if let` pattern binding | visible only in success-region lexical scope |
| `if let` branch values | joined as executed region values, not closure types |
| nested branch return + field state | return flow snapshot contains actual field state |

---

# 8. Required negative assertions

Do not test only positive type results. Add explicit assertions that the checker does **not** do the following:

```text
- no nested return is lost because it was below the body loop
- no nested return is recorded twice
- no abrupt branch contributes a value to a normal branch join
- no caller must manually mark unreachable after return/throw/break/continue
- no if-let body is analyzed through Expr::Block closure construction
- no branch-local declaration becomes lexically visible after the branch
- no closure construction mutates outer flow
- no second fallthrough boolean exists beside FlowState::reachable
```

---

# 9. Explicit non-goals

Part 3 must not expand into:

- `while let` fixed-point execution;
- `for`/`whileTrue` loop correctness beyond using atomic break/continue recording;
- branch contradiction proof;
- negative type filtering fixes;
- equality/nil predicate trust;
- `if true`/`if false` pruning;
- full pattern exhaustiveness/refutability analysis;
- CFG-driven interpretation;
- SSA;
- a general effect system;
- non-local block return runtime semantics redesign;
- diagnostics for unreachable source statements.

These boundaries are intentional. Part 3 establishes control ownership and executable-region semantics; Part 4 establishes path proof; Part 5 establishes loop topology and fixed points.

---

# 10. Definition of done

Part 3 is complete only when all of the following are true:

1. `FlowState::reachable` is the only checker-local fallthrough authority.
2. `return`, `throw`, `break`, and `continue` record their destination fact and terminate the path atomically.
3. Callable normal returns are context-owned and discovered at any nesting depth.
4. `checker/control.rs` is the one statement-loop implementation for executable branch regions.
5. `Expr::Block` remains closure construction and is not reused for branch execution.
6. ordinary paired branches delegate execution/join orchestration to `control.rs`.
7. `if let` executes real control regions and preserves outer mutations.
8. abrupt branches do not participate in normal value/flow joins.
9. existing branch, callable publication, field, and closure tests remain GREEN.
10. no CFG/SSA rewrite or loop fixed-point work was smuggled into the change.

The architectural result should be simple to state:

```text
statement syntax
    ↓
check_statement
    ↓
StatementControl + context-owned side effects
    ↓
FlowState::reachable

structured branch syntax
    ↓
checker/control.rs executable region
    ↓
normal flow/value or abrupt termination
    ↓
canonical reachable join
```

That becomes the stable control substrate consumed by Parts 4 and 5.
