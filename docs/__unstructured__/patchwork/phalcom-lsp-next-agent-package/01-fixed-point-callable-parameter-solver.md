# Implementation Spec 1 — Correct Fixed-Point Callable and Parameter Solving

**Repository:** `aureat/phalcom-lang`  
**Baseline commit:** `8f41ee4a7029f0617930cb01348454a111d072fb` (`checkpoint: commit live semantic workspace changes`)  
**Primary package:** `phalcom-lsp`  
**Order:** implement this spec before Specs 2 and 3.

## 1. Goal

Finish the semantic fixed-point layer so callable summaries and parameter facts are mathematically stable, module-qualified, monotone, and correct across forwarding chains and multiple caller modules.

The current baseline is materially ahead of the older handoff: it already retains `Arc<Program>` snapshots, records `CallableSummary.dependencies`, loops to a fixed point, and exposes `return_for_callable`. Do **not** rewrite that foundation from scratch. Fix the remaining correctness defects and extract the solver into a form that Spec 2 can run over an affected dependency slice rather than the whole workspace.

## 2. Authoritative invariants

Preserve these rules:

1. Semantic callable identity is `CallableId = ClassId(module + class) + selector + DispatchSide`.
2. Selector text alone is never semantic callable identity.
3. `ValueShape::Unknown` means loss/absence of useful runtime knowledge. It is not `Any`.
4. Unions are bounded by `MAX_SHAPE_UNION` and widen to `Unknown` above the bound.
5. Parameter facts are admitted only from resolved call targets.
6. Facts from multiple call sites **join**. They never overwrite one another because of file ordering.
7. Recursive and mutually recursive graphs terminate.
8. A fixed point must propagate inferred parameter knowledge through forwarding call sites, not only into method return bodies.
9. A published generation is coherent: summaries, parameter facts, local facts, and fields must be derived from the same solver result.

## 3. Read only these files first

Do not scan the repository. Read these exact anchors:

1. `phalcom-lsp/src/semantic/facts.rs`
   - `MAX_SHAPE_UNION`
   - `ValueShape::join`
   - `ValueShape::bounded_union`
   - `InferredValue::join`
   - `ParameterFacts::{record,get,iter}`
2. `phalcom-lsp/src/semantic/callable.rs`
   - `CallableSummary`
   - `SummaryEffects`
3. `phalcom-lsp/src/semantic/infer.rs`
   - `infer_expr_with_returns`
   - `parameter_facts_for_program`
   - `collect_call_sites`
   - `collect_call_sites_expr`
   - `summaries_for_surface`
   - `callable_dependencies`
   - `body_value`
4. `phalcom-lsp/src/semantic/mod.rs`
   - `SemanticState`
   - `SemanticDb::update_file`
   - `SemanticDb::return_for_callable`
   - `rebuild_state` (baseline starts at approximately line 493)
5. Tests at the bottom of `phalcom-lsp/src/semantic/mod.rs`.
6. Apply/read `phalcom-lsp-regression-tests.patch` supplied with these specs; the two interprocedural fixtures are acceptance tests for this unit.

Do not read `backend.rs`, VS Code files, compiler/VM files, or the legacy `WorkspaceIndex` for this unit.

## 4. Baseline defects that must be fixed

### 4.1 Cross-module parameter contributions overwrite instead of join

Baseline `rebuild_state` contains this shape:

```rust
let mut next_parameters = BTreeMap::new();
for (module, program, surface) in &inputs {
    // ...
    let facts = infer::parameter_facts_for_program(/* ... */);
    next_parameters.extend(facts.iter().map(|(key, value)| (key.clone(), value.clone())));
}
```

`BTreeMap::extend` is last-writer-wins. If `consumer_a.ph` calls `Service.consume(Cat.new())` and `consumer_b.ph` calls `Service.consume(Dog.new())`, one `(Service.consume(_), value)` fact replaces the other. The result is dependent on module iteration order.

This is a correctness bug, not merely an optimization issue.

### 4.2 Parameter inference does not seed method environments with already inferred parameters

`summaries_for_surface` seeds the method environment from `parameter_fact`, but the call-site walker used by `parameter_facts_for_program` starts each member with an empty `member_environment`.

Therefore this chain does not fully propagate:

```phalcom
class Relay {
  sink(_ value) { value }
  forward(_ value) { sink(value) }
}
```

If another module calls `forward(Product.new())`, `forward.value` can become `Product`, but `collect_call_sites` still sees `value` as unknown while inspecting `sink(value)`. `sink.value` therefore never receives the propagated fact.

### 4.3 The solver has a silent arbitrary round cap

The baseline uses:

```rust
const MAX_SOLVER_ROUNDS: usize = 64;
for _ in 0..MAX_SOLVER_ROUNDS {
    // ...
    if !summaries_changed && !parameters_changed {
        break;
    }
}
```

If round 64 is reached without convergence, the current code silently publishes the last partial state. A semantic engine must not silently publish a non-fixed point.

### 4.4 The solver rebuild logic is embedded in `SemanticDb`

`rebuild_state` currently performs extraction, iteration, local facts, fields, and publication in one function. Spec 2 needs to solve only an affected dependency slice. Extract the pure solving step now so the incremental layer can reuse it.

## 5. Required data/API changes

### 5.1 Add joining support to `ParameterFacts`

Edit `phalcom-lsp/src/semantic/facts.rs`.

Add this exact public-within-crate behavior:

```rust
impl ParameterFacts {
    /// Joins every contribution from `other` into this aggregate.
    pub fn merge_from(&mut self, other: &Self) {
        for ((callable, name), value) in other.iter() {
            self.record(callable.clone(), name.clone(), value.clone());
        }
    }
}
```

Do not expose the internal `params` map just to make aggregation easier.

### 5.2 Seed parameter facts during call-site extraction

Change `parameter_facts_for_program` in `semantic/infer.rs` to accept a parameter lookup closure:

```rust
pub fn parameter_facts_for_program(
    program: &Program,
    surface: &ModuleSurface,
    module: &ModuleId,
    known_class: impl Fn(&str) -> Option<ClassId> + Copy,
    is_constructor: impl Fn(&ClassId, &str) -> bool + Copy,
    callable_return: impl Fn(&CallableId) -> Option<InferredValue> + Copy,
    parameter_fact: impl Fn(&CallableId, &str) -> Option<InferredValue> + Copy,
    resolve_member: impl Fn(&ClassId, &str) -> Option<MemberSurface> + Copy,
) -> ParameterFacts
```

Propagate `parameter_fact` through the internal `collect_call_sites*` helpers.

When entering each class member, resolve the exact `MemberSurface` for that AST member and seed `member_environment` before walking its body:

```rust
let selector = crate::selectors::class_member_selector(member);
let Some(member_surface) = class_surface.members.get(&selector) else {
    continue;
};

let mut member_environment = BTreeMap::new();
for param in &member_surface.params {
    if let Some(value) = parameter_fact(&member_surface.callable, &param.name) {
        member_environment.insert(param.name.clone(), value);
    }
}
```

Then walk that member body using `member_environment`.

Do not key this lookup by selector globally. The owner is the current `ClassSurface`, so the lookup is class-qualified.

### 5.3 Introduce a pure solver result

In `semantic/callable.rs`, add a crate-private result structure:

```rust
#[derive(Clone, Debug, Default)]
pub(crate) struct SolverResult {
    pub summaries: std::collections::BTreeMap<CallableId, CallableSummary>,
    pub parameter_facts: super::facts::ParameterFacts,
}
```

If imports become noisy, the structure may live in `semantic/infer.rs` instead. Do not make it part of a public crate API.

### 5.4 Extract the fixed-point function from `rebuild_state`

Create in `semantic/infer.rs`:

```rust
pub(crate) fn solve_workspace_callables(
    inputs: &[(ModuleId, std::sync::Arc<Program>, ModuleSurface)],
    classes: &BTreeMap<ClassId, ClassSurface>,
    graph: &ModuleGraph,
    generation: SemanticGeneration,
) -> SolverResult
```

If `ModuleGraph` visibility creates a cycle in module imports, pass resolution closures from `mod.rs` instead. The important constraint is that the function is deterministic and does not mutate `SemanticState` while solving.

Spec 2 may later generalize `inputs` to an affected subset. Keep the solver independent of `RwLock` and `SemanticDb`.

## 6. Fixed-point algorithm

Use the following sequence.

### 6.1 Initial state

Start with no inferred source-callable summaries and no parameter contributions:

```rust
let mut summaries = BTreeMap::<CallableId, CallableSummary>::new();
let mut parameters = ParameterFacts::default();
```

Native/core contract knowledge can still be queried through existing core resolution paths. Do not seed source callables by selector.

### 6.2 One solver iteration

For each iteration:

1. Clone/snapshot the previous summaries and parameter facts.
2. Re-extract per-module call-site parameter facts using:
   - previous callable returns;
   - previous parameter facts to seed member environments;
   - exact receiver/member resolution.
3. **Join** all module contributions using `ParameterFacts::merge_from`.
4. Recompute callable summaries using the newly joined parameter facts and previous callable returns.
5. Compare semantic content, not allocation identity.
6. Stop only when both summary and parameter maps are unchanged.

The key call becomes:

```rust
let parameter_fact = |id: &CallableId, name: &str| {
    previous_parameters.get(id, name).cloned()
};

let facts = parameter_facts_for_program(
    program,
    surface,
    module,
    known_class,
    is_constructor,
    callable_return,
    parameter_fact,
    resolve_member,
);
next_parameters.merge_from(&facts);
```

### 6.3 Termination guard

The semantic lattice is intentionally bounded. Still retain a defensive guard against implementation mistakes, but it must never silently publish a partial fixed point.

Use a derived step budget rather than `64`:

```rust
let callable_count: usize = inputs.iter()
    .map(|(_, _, surface)| surface.classes.values().map(|c| c.members.len()).sum::<usize>())
    .sum();
let slot_count: usize = inputs.iter()
    .map(|(_, _, surface)| {
        surface.classes.values()
            .flat_map(|c| c.members.values())
            .map(|m| m.params.len())
            .sum::<usize>()
    })
    .sum();
let max_rounds = (callable_count + slot_count).max(1) * (MAX_SHAPE_UNION + 2);
```

If the budget is exceeded:

- in tests/debug builds: `panic!` with a message naming the still-changing callable/parameter keys;
- in release: conservatively replace the still-changing solver values with `Unknown`, perform one final stabilization pass, and return a fixed state.

Do not return an arbitrary round-64 snapshot.

## 7. Solver-bottom versus semantic `Unknown`

Do not change `ValueShape::Unknown` into a bottom value. It is already used as the conservative top/loss-of-knowledge result (`Unknown.join(T) == Unknown`).

However, recursive solving needs to distinguish:

- **not solved yet**; and
- **solved but unknown**.

Use absence from the summary map as solver-bottom. Do not insert `Some(Unknown)` for a source callable before it has been evaluated.

When a resolved call targets a source callable whose summary is absent, summary extraction must treat that edge as “no evidence yet”, not as a permanent semantic `Unknown` that poisons every join.

Implement this only in the summary-extraction path; editor-facing `infer_expression` may continue returning `Unknown` for unresolved knowledge.

Recommended internal helper:

```rust
type SummaryEvidence = Option<InferredValue>; // None = solver bottom / no evidence yet
```

Add a solver-specific expression path where needed:

```rust
fn infer_summary_expr(/* same semantic context */) -> SummaryEvidence
```

Rules:

- literal, known constructor, known field/parameter => `Some(value)`;
- resolved source call with a current summary => `Some(summary.returns)`;
- resolved source call without a current summary => `None`;
- genuinely dynamic/unresolved call => `Some(Unknown)`;
- joining `None` with `Some(v)` => `Some(v)`;
- joining two `Some` values => `InferredValue::join`.

After the fixed point, any callable that still has no evidence gets a published `Unknown` summary.

This distinction is required for recursive SCCs with concrete base evidence.

## 8. Callable dependency behavior

Keep `CallableSummary.dependencies` populated by `callable_dependencies`. Ensure the dependency list is:

- exact `CallableId`s;
- sorted/deduplicated (the current `BTreeSet` extraction is acceptable);
- class-side/instance-side correct;
- inherited target owner correct.

Do not record an unresolved selector as a dependency on every matching workspace method.

`SummaryEffects.dynamic_send` should become `true` whenever a call cannot be resolved to a bounded semantic target. Do not use this flag to fabricate a dependency.

## 9. Tests to add in this unit

### 9.1 Existing tests that must remain

Do not weaken:

- direct constructor return;
- recursive call terminates at unknown;
- mutual recursion terminates at unknown;
- explicit multiple returns create bounded union;
- imported callable returns/parameters propagate.

### 9.2 Add unit tests in `semantic/mod.rs` or `semantic/infer.rs`

Add these exact laws:

1. `parameter_facts_join_across_modules`
   - two caller modules send `Cat` and `Dog` to one provider parameter;
   - provider parameter is `Cat | Dog`.
2. `parameter_fact_flows_into_nested_forwarding_call`
   - `forward(value) -> sink(value)`;
   - external `Product` call infers `sink.value == Product` and `forward()` return `Product`.
3. `three_step_return_forwarding_converges`
   - `a() -> b() -> c() -> Product.new()`.
4. `recursive_scc_with_concrete_evidence_converges`
   - use valid current Phalcom control-flow syntax;
   - one recursive path plus one concrete return path;
   - result must retain the concrete result rather than becoming stuck at solver-bottom.
5. `nine_incompatible_return_shapes_widen_to_unknown`
   - boundary is exactly `MAX_SHAPE_UNION == 8` at this baseline.
6. `same_selector_different_classes_have_independent_summaries`
   - `A.value()` and `B.value()` return different shapes;
   - lookup by their `CallableId` returns the correct shape.

### 9.3 Apply the supplied RPC/fixture patch

The supplied `phalcom-lsp-regression-tests.patch` adds two high-value public tests for this spec:

- `parameter_facts_from_multiple_consumer_modules_join_instead_of_overwriting`;
- `inferred_parameter_facts_propagate_through_forwarding_calls`.

Do not edit those expected outcomes to accommodate current behavior.

## 10. Implementation sequence

Follow this exact order:

1. Add `ParameterFacts::merge_from` + unit test.
2. Add `parameter_fact` callback to `parameter_facts_for_program` and internal walkers.
3. Seed each method environment from its exact `MemberSurface.params`.
4. Change the current global parameter aggregation from `BTreeMap::extend` to joining aggregation.
5. Run the two interprocedural regression tests; make them green before refactoring.
6. Extract the pure fixed-point solver.
7. Introduce solver-bottom/no-evidence behavior for recursive source-call edges.
8. Replace silent fixed-round publication with explicit convergence/defensive widening.
9. Add recursion/forwarding/widening unit tests.
10. Run all focused commands below.

## 11. Commands

```sh
cargo test -p phalcom-lsp semantic::
cargo test -p phalcom-lsp --test integration parameter_facts_from_multiple_consumer_modules_join_instead_of_overwriting
cargo test -p phalcom-lsp --test integration inferred_parameter_facts_propagate_through_forwarding_calls
cargo test -p phalcom-lsp --test integration workspace_semantics
cargo test -p phalcom-lsp
```

Then:

```sh
cargo clippy -p phalcom-lsp --all-targets -- -D warnings
```

Do not use a broad workspace test as a substitute for the focused semantic tests.

## 12. Completion criteria

This unit is complete only when:

- no cross-module parameter contribution can overwrite another;
- inferred parameters can drive further call-site inference;
- forwarding chains converge;
- recursive SCCs terminate deterministically;
- source-callable lookup remains `CallableId`-qualified;
- solver-bottom is not conflated with semantic `Unknown`;
- solver publication cannot silently stop at an arbitrary iteration cap;
- existing LSP tests remain green.
