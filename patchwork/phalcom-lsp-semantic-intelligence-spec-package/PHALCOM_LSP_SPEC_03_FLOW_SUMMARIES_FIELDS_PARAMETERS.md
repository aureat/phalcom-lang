# Phalcom LSP Implementation Spec 3
## Scoped Flow, Callable Summaries, Field/Parameter Facts, and Dependency Propagation

**Repository baseline:** `e2ec9e5fb6dc362786c9dd9593470feb47c91d94`  
**Depends on:** Specs 1 and 2  
**Primary crate:** `phalcom-lsp`  
**Goal:** replace separate local/summary/field/parameter/dependency AST walkers with one structured flow analysis that feeds the existing fixed-point/invalidation database.

---

# 1. Scope

This specification implements statement-level semantics around the unified expression analyzer.

It MUST consolidate the current independent walkers used for:

- local facts;
- callable return summaries;
- parameter call-site facts;
- field facts;
- callable dependency extraction.

The existing fixed-point solver and `SemanticDb` generation/invalidation machinery should remain.

---

# 2. Targeted baseline reads

| Purpose | Target |
|---|---|
| call-site collection | `phalcom-lsp/src/semantic/infer.rs:430-900` |
| local facts | `phalcom-lsp/src/semantic/infer.rs:880-1080` |
| summaries | `phalcom-lsp/src/semantic/infer.rs:1030-1270` |
| dependency extraction | `phalcom-lsp/src/semantic/infer.rs:1270-1540` |
| affected solver | `phalcom-lsp/src/semantic/infer.rs:430-650` |
| state rebuild | `phalcom-lsp/src/semantic/mod.rs:500-920` |
| fact structures | `phalcom-lsp/src/semantic/facts.rs` |
| callable summary structure | `phalcom-lsp/src/semantic/callable.rs` |
| flow helper | `phalcom-lsp/src/semantic/flow.rs` |
| block non-local return semantics | `docs/spec/current/blocks.md` |
| For AST semantics | `phalcom-ast/src/ast.rs:560-840` |

---

# 3. Replace spelling-keyed local facts

Current `LocalFacts` is keyed by `String`.

Migrate to `BindingId`.

Recommended:

```rust
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct BindingFacts {
    facts: BTreeMap<BindingId, Vec<FlowFact>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FlowFact {
    pub at: SourceRange,
    pub value: InferredValue,
}
```

Query:

```rust
pub fn value_before(&self, binding: BindingId, offset: usize)
    -> Option<&InferredValue>;
```

The occurrence resolver identifies the `BindingId`; callers do not query by name.

During compatibility migration, keep `LocalFacts` as a wrapper only if necessary. Do not leave two authoritative fact stores.

---

# 4. Flow state

Use an immutable-or-clonable state keyed by binding identity.

```rust
#[derive(Clone, Debug, Default)]
pub struct FlowState {
    pub bindings: BTreeMap<BindingId, InferredValue>,
}
```

For early implementation, cloning at structured branch boundaries is acceptable.

Do not optimize into SSA before correctness tests exist.

---

# 5. Analysis result

Recommended:

```rust
pub struct StatementFlow {
    pub normal: Option<FlowState>,
    pub returns: Vec<ReturnEvidence>,
    pub breaks: Vec<FlowState>,
    pub continues: Vec<FlowState>,
    pub throws: bool,
}

pub struct ReturnEvidence {
    pub target: CallableId,
    pub value: InferredValue,
    pub range: SourceRange,
}
```

`normal = None` means the path cannot continue normally.

---

# 6. Analysis sink / events

The unified expression analyzer from Spec 2 should emit resolved call events.

Define:

```rust
pub struct ResolvedCall {
    pub target: CallableId,
    pub site: SourceRange,
    pub args: Vec<AnalyzedArgument>,
    pub dynamic: bool,
}

pub enum AnalysisEvent {
    Call(ResolvedCall),
    FieldWrite(FieldWrite),
}
```

Or implement methods directly on an `AnalysisSink`.

From these events derive:

- callable dependencies;
- parameter facts;
- dynamic-send effects;
- optional navigation occurrence resolution.

Do not walk the AST a second time to rediscover calls.

---

# 7. Sequential statement semantics

Implement:

```rust
fn analyze_statements(
    statements: &[Statement],
    ctx: &AnalysisContext<'_>,
    state: FlowState,
    sink: &mut FlowSink,
) -> StatementFlow
```

Process statements in source order.

If `normal` becomes `None`, remaining statements can still be traversed for editor occurrence information if desired, but must not affect reachable flow facts unless deliberately marked unreachable.

---

# 8. `let` / `const`

For each binding statement:

1. analyze initializer in current state;
2. if absent mutable `let`, use the language's `None` value knowledge if canonical `None` shape is represented; otherwise `Unknown` with exact "absent initializer" provenance;
3. project destructuring shapes;
4. create/update each `BindingId`;
5. record declaration fact at exact pattern-name range.

`const` vs `let` mutability comes from binding metadata.

The semantic engine should not silently accept writes to `const`; compiler diagnostics remain authoritative, but analysis must not propagate an illegal reassignment as if valid.

---

# 9. Assignment

For:

```phalcom
x = rhs
```

1. resolve LHS occurrence to `BindingId`;
2. analyze RHS;
3. if mutable binding, update state;
4. record write fact at exact LHS range;
5. expression value follows Spec 2.

For fields:

1. resolve `FieldId`;
2. emit field-write event with RHS value;
3. do not place field in lexical binding map.

---

# 10. Destructuring

Reuse existing `ValueShape` projection.

Tuple/list destructuring should bind each target independently.

If shape is insufficient:

```text
target -> Unknown
```

but binding identity still exists.

Rest list pattern:

```text
rest -> List<element-shape>
```

when input list/tuple structure makes that safe; otherwise `List<Unknown>` or `Unknown` according to existing shape conventions.

---

# 11. Method/member analysis entry

For every source member, create an independent root flow state.

Seed all parameters in lexical scope.

Each parameter gets:

1. call-site inferred evidence if available;
2. otherwise `Unknown`;
3. later, future declared type evidence can refine/override through a separate bridge.

Never omit an unknown parameter from the state.

For instance member:

```text
self -> Instance(class)
```

For class-side:

```text
self -> ClassObject(class)
```

---

# 12. Callable return summary

Do not use the current `body_summary_value` strategy.

Instead, the member analysis produces return evidence from the real sequential traversal.

Final return summary is:

```text
join(explicit reachable returns, implicit tail result)
```

according to Phalcom member semantics.

If the language defines an empty body/unit result, represent that explicitly if `ValueShape` has/gets a Unit representation; otherwise preserve Unknown until the shape model is extended.

For constructors:

```text
return summary = exact Instance(receiver construction class semantics)
```

regardless of body return/tail, while body events/facts still run.

---

# 13. Implicit tail value

Phalcom methods/blocks have expression-tail semantics.

The flow engine must retain the last reachable expression value of a body.

Do not infer the value of a `let` statement itself as the initializer unless the language explicitly defines that statement result.

The current `body_value` does this in some paths and should be corrected.

Recommended body result:

```rust
struct BodyFlow {
    flow: StatementFlow,
    tail_value: Option<InferredValue>,
}
```

Only syntactic forms that are expression-valued should set `tail_value`.

---

# 14. Nested closure analysis

Every block gets:

- child lexical scope;
- parameter state;
- normal block result summary;
- captured binding references.

Do not mutate the outer flow state merely because a block literal is created.

Assignments to captured mutable variables become effects of invoking the block, not of constructing it.

This distinction is required for correctness.

---

# 15. Non-local `return` from blocks

Phalcom block `return` targets the home method frame.

Represent each block summary with:

```rust
pub struct BlockSummary {
    pub value: InferredValue,
    pub nonlocal_returns: Vec<ReturnEvidence>,
    pub captured_writes: Vec<CapturedWrite>,
}
```

Do not automatically union every syntactically nested block return into the method return: a block may never execute or may escape.

Propagate `nonlocal_returns` into the home callable only when analysis knows the block is invoked on that execution path.

Initial supported cases SHOULD include:

- parser-desugared `if` / `while` sacred control-flow sends with literal blocks;
- `and` / `or` lazy blocks;
- direct call of a literal/known block where resolvable.

For arbitrary higher-order calls, retain the block effect summary and conservatively avoid claiming execution unless the callable contract says it invokes the block.

This creates the correct future seam for interprocedural effect typing.

---

# 16. `for` flow

`ForStatement` is explicit AST.

Rules:

1. analyze iterable once in parent state;
2. derive element shape via `ValueShape::element_shape`;
3. enter loop body scope;
4. bind loop variable;
5. analyze body;
6. `continue` feeds loop back-edge;
7. `break` exits loop;
8. join:
   - zero-iteration state;
   - break states;
   - loop-carried state after bounded/widened iteration.

A simple first implementation can run one abstract iteration and join with entry state, repeating until stable with a small bound.

Do not let loop binding escape its scope.

---

# 17. Parsed/desugared `if` and `while`

Current parser lowers important control-flow syntax into calls/blocks.

The semantic analyzer should recognize the same sacred/control-flow selectors in a VM-free way.

Do not import compiler internals into LSP.

Create a small shared semantic recognizer, or reproduce only the stable selector semantics in `semantic/control_flow.rs`.

For known literal-block control flow:

- analyze branches under cloned states;
- join normal exits;
- collect non-local returns;
- avoid applying untaken branch writes to all paths.

This is the minimum necessary to make nested body inference meaningful.

---

# 18. Parameter facts from resolved calls

Delete the current separate `collect_call_sites_expr` once event-based collection is green.

For each `ResolvedCall`:

1. obtain target `MemberSurface.params`;
2. map arguments to parameters using canonical selector/label mapping;
3. record each non-Unknown argument shape as interprocedural evidence;
4. preserve provenance = call site;
5. merge across call sites with bounded join.

Dynamic packs/computed labels:

- only map slots whose correspondence is statically known;
- do not fabricate parameter evidence for ambiguous positions.

---

# 19. Inherited targets and parameter ownership

Parameter facts are keyed to the actual resolved callable owner.

If `Child.foo(x)` resolves to `Base.foo(_)`, record evidence under:

```text
CallableId(owner=Base, selector=foo(_), side=Instance)
```

not `Child`.

This is automatically satisfied if all call events come from Spec 2 `DispatchResolver`.

---

# 20. Field evidence

Introduce:

```rust
pub struct FieldId {
    pub owner: ClassId,
    pub name: String,
    pub side: DispatchSide,
}

pub enum FieldEvidenceKind {
    DeclarationInitializer,
    ConstructorInitialization,
    GeneralWrite,
}
```

Store:

```rust
pub struct FieldEvidence {
    pub value: InferredValue,
    pub kind: FieldEvidenceKind,
    pub site: SourceRange,
}
```

Produce these during normal flow analysis.

---

# 21. Constructor fields

When analyzing a source constructor body:

- parameter facts are present in state;
- `_owner = owner` therefore records `owner`'s actual evidence;
- nested initialization writes are traversed;
- class/instance field side is known.

This fixes the current empty-environment bug.

---

# 22. Field read policy

For editor value inference, derive a joined field value from relevant evidence.

Recommended precedence:

1. explicit future declared type evidence (future bridge);
2. stable declaration initializer + all constructor initialization evidence;
3. join general write evidence;
4. Unknown.

If general writes introduce incompatible shapes, bounded-union/widening rules apply.

Do not silently assume a constructor write dominates later mutable writes.

Inherited field lookup should resolve the defining `FieldId` through class hierarchy.

---

# 23. Dependency graph

Delete the separate `callable_dependencies` AST walk after event-based coverage is complete.

A member's dependencies are exactly the resolved call targets observed while analyzing its executable paths, including:

- explicit method calls;
- getters;
- setters;
- operators;
- subscripts;
- known callable invocation.

For dynamic/unresolved sends:

```rust
summary.effects.dynamic_send = true
```

Do not invent a callable edge.

This ensures changes to a getter such as `Account.toString` invalidate dependent summaries that use `super.toString` or `x.toString`.

---

# 24. Fixed-point solver integration

Retain the existing solver concept in `infer.rs`.

Refactor iteration inputs to:

```text
previous callable summaries
previous parameter facts
-> analyze affected members
-> new summaries + parameter contributions + dependencies
-> compare
-> repeat
```

Do not independently call five extraction functions.

Recommended single function:

```rust
fn analyze_surface(
    surface: &ModuleSurface,
    context: SolverContext<'_>,
) -> SurfaceAnalysis
```

where:

```rust
pub struct SurfaceAnalysis {
    pub summaries: Vec<CallableSummary>,
    pub parameter_facts: ParameterFacts,
    pub field_facts: FieldFacts,
    pub binding_facts: BindingFacts,
}
```

Local binding facts may be computed after/alongside solver depending on parameter evidence, but use the same flow engine.

---

# 25. Solver bottom vs Unknown

Preserve the important existing distinction between:

- no summary evidence yet in fixed-point iteration;
- a real semantic `Unknown` result.

Do not collapse solver-bottom into `Unknown`, or recursive call inference can stabilize incorrectly.

Recommended:

```rust
enum SummaryState {
    Bottom,
    Known(InferredValue),
}
```

Keep this internal to solver.

---

# 26. Invalidation

Current `SemanticDb` uses callable dependents and module graph closure.

Retain it.

After dependency extraction becomes complete:

- getter/operator/index dependencies enter the same reverse map;
- changed return summary triggers dependent module/callable recomputation;
- changed parameter fact triggers owning callable/dependents as today.

Add tests that edit:

```phalcom
Base.value { 1 }
```

to return a String and verify a dependent `Child.describe()` summary changes without full unrelated-workspace recomputation.

---

# 27. Flow widening

Avoid unbounded unions and nontermination.

Keep `MAX_SHAPE_UNION`.

For loops/recursive fixed points:

- compare stable states;
- cap iterations based on finite binding/callable slots;
- widen unstable values to Unknown after budget.

Release builds must publish coherent widened state, as current solver already does.

---

# 28. Tests

## Local/scope flow

- sequential `let`;
- reassignment;
- const illegal write ignored for propagation;
- nested shadowing;
- same name in two methods;
- closure capture;
- closure-local shadowing;
- for binding;
- destructuring.

## Callable summaries

- direct literal return;
- `let` then return;
- multiple returns joined;
- nested `if` returns;
- `while`/loop return;
- implicit tail expression;
- chained call return;
- getter/operator/subscript return;
- constructor fixed result.

## Blocks

- expression-valued block;
- literal block invoked by recognized control flow;
- non-local return from invoked block;
- escaped block return not eagerly attributed to method.

## Fields

- declaration initializer;
- constructor parameter assignment;
- multiple constructor shapes join;
- later mutable write joins;
- inherited field read.

## Parameter facts

- positional;
- labeled;
- inherited target;
- class-side target;
- getter has no params;
- setter RHS;
- subscript params;
- dynamic pack conservative behavior.

## Dependencies/invalidation

- method call;
- getter call;
- operator call;
- subscript call;
- `super` call;
- inherited call;
- edit upstream summary and verify bounded dependent recomputation.

---

# 29. Acceptance gate

Spec 3 is complete when:

1. callable summaries use sequential flow;
2. locals are keyed by `BindingId`;
3. current separate call-site/dependency walkers are no longer authoritative;
4. getter/operator/subscript dependencies are recorded automatically;
5. constructor field assignments can use parameter evidence;
6. nested control-flow returns are represented;
7. block non-local return semantics distinguish block construction from invocation;
8. fixed-point convergence/invalidation tests remain green;
9. no same-name cross-scope fact leakage remains.
