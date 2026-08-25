# Complex Semantic Analysis Scenarios Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add one semantic integration test file with ten deliberately cross-cutting Phalcom programs that measure whether the analyzer preserves precise facts, conservative uncertainty, control-flow reachability, dispatch identity, and incremental dependencies.

**Architecture:** Create `phalcom-semantic/tests/semantic_complex_analysis.rs`. Reuse the existing single-module analysis helper pattern from `semantic_authority_composition.rs` for source-local scenarios, and use `SemanticWorkspaceSession` for module and revision scenarios. Assertions target `CallableAnalysis`, `BodyExitFacts`, `BindingState`, `ExpressionAnalysis`, flow-graph nodes, diagnostics, dependency edges, and snapshot reuse rather than UI output.

**Tech Stack:** Rust integration tests, `phalcom_semantic::analyze_single_module`, `SemanticWorkspaceSession`, `TypeKnowledge`, `EvidenceAuthority`, `CallableId`, `DispatchSide`, flow graphs, and snapshot dependency/invalidation APIs.

**Spec:** Current task request plus the return-summary contract: explicit returns contribute their expressions; tail expressions contribute values; tail `let`/`const` contributes `Unit`; abrupt paths contribute no normal return; unannotated callable summaries join reachable normal returns.

## Global Constraints

- Keep this as a new integration file; do not enlarge `semantic_authority_composition.rs`.
- Test semantic products directly before adding any runtime/compiler assertion.
- A dynamic or reflective construct must remain `Dynamic`, `Unknown`, or opaque where proof is unavailable; never assert a concrete type merely because one use site happens to suggest it.
- Every test must identify its semantic tier: flow, dispatch, callable summary, module, or incremental analysis.
- Use deterministic source text and stable callable lookup helpers.
- Tests that exercise cross-module state must use resolved `ModuleId` values and verify dependency behavior, not only absence of diagnostics.

---

### Task 1: Create shared test harness

**Files:**
- Create: `phalcom-semantic/tests/semantic_complex_analysis.rs`
- Reference: `phalcom-semantic/tests/semantic_authority_composition.rs`

- [ ] **Step 1: Add analysis and lookup helpers.**

Add helpers equivalent to the existing authority-composition test:

```rust
fn analyze(source_text: &str) -> (ModuleId, Arc<str>, Analysis);
fn callable_analysis<'a>(analysis: &'a Analysis, id: &CallableId) -> &'a CallableAnalysis;
fn zero_arg_callable(module: &ModuleId, owner: &str, name: &str, side: DispatchSide) -> CallableId;
fn binding<'a>(analysis: &'a CallableAnalysis, name: &str) -> &'a BindingState;
```

Add a second helper for resolved workspace modules:

```rust
fn single_module_input(module: ModuleId, source: &str, revision: u64) -> ModuleInput;
```

- [ ] **Step 2: Run the new target before adding scenarios.**

Run:

```sh
cargo test -p phalcom-semantic --test semantic_complex_analysis
```

Expected: the target compiles and reports zero tests before scenario tests are added.

---

### Task 2: Branch joins and flow-sensitive refinement

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Add `branch_join_keeps_union_and_refines_each_arm`.**

Use a method with an `Object` parameter:

```phalcom
class Probe {
  @class choose(_ value: Object) {
    if (value.is(Int)) {
      return value
    } else {
      return "fallback"
    }
  }
}
```

Assert that the callable is complete, both normal paths are represented, the summary is a known `Int | String` union, and no arm is incorrectly treated as `Dynamic` or `Never`. Inspect the `value` occurrences in each branch to confirm the positive arm narrows to `Int` while the merge does not retain the arm-only refinement.

- [ ] **Step 2: Run the focused test.**

Run:

```sh
cargo test -p phalcom-semantic --test semantic_complex_analysis branch_join_keeps_union_and_refines_each_arm -- --nocapture
```

Expected: PASS if branch transfer and join are implemented; otherwise the failure must identify whether narrowing, union construction, or merge reachability is missing.

---

### Task 3: Loop fixed point, `break`, and `continue`

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Add `loop_fixpoint_preserves_mutated_integer_and_abrupt_edges`.**

Use a `while` loop containing mutation and `continue`, followed by a `for` loop containing `break`:

```phalcom
class Probe {
  @class run(_ limit: Int) -> Int {
    let total = 0
    let i = 0
    while (i < limit) {
      i = i + 1
      if (i == 2) { continue }
      total = total + i
    }
    for item in [1, 2, 3] {
      if (item == 2) { break }
      total = total + item
    }
    total
  }
}
```

Assert complete analysis, `Int` knowledge for `total`, a `LoopHeader` and `BackEdge` in the flow graph, and a normal `Int` summary. Assert that `continue` and `break` do not erase the reachable tail or manufacture an extra normal return.

- [ ] **Step 2: Run the focused test.**

Run the same target with the test filter `loop_fixpoint_preserves_mutated_integer_and_abrupt_edges`.

---

### Task 4: Closure capture and non-local return context

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Add `escaped_closure_keeps_capture_and_non_local_return_isolated`.**

Analyze both a closure that captures a local and a closure containing a non-local `return`:

```phalcom
class Maker {
  make(_ seed: Int) {
    return || { seed + 1 }
  }

  makeReturningBlock() {
    return || { return 1 }
  }
}
```

Assert that `make` records a callable/block result rather than `Unit`, that the block captures `seed`, and that `makeReturningBlock` does not receive a spurious return-annotation mismatch from the nested `return`. Verify the outer callable remains complete and its normal summary is not polluted by the nested block’s separate control context.

- [ ] **Step 2: Run the focused test.**

Run the target with `escaped_closure_keeps_capture_and_non_local_return_isolated`.

---

### Task 5: Higher-order callback summary propagation

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Add `higher_order_block_call_propagates_captured_result`.**

Use a local block, capture a parameter, and invoke it through `call()`:

```phalcom
class Probe {
  @class apply(_ value: Int) {
    const increment = || { value + 1 }
    increment.call()
  }
}
```

Assert that the block’s captured parameter remains `Int`, the call expression is a known `Int` with proven/contract evidence where available, and `apply` receives an inferred `Int` summary. This separates block construction from block execution and catches analyzers that infer only the closure object type.

- [ ] **Step 2: Run the focused test.**

Run the target with `higher_order_block_call_propagates_captured_result`.

---

### Task 6: Inheritance, `super`, and class/instance-side separation

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Add `super_dispatch_preserves_side_and_inherited_constructor_identity`.**

Use both instance-side and class-side overrides:

```phalcom
class Base {
  @constructor
  new() {}

  value(_ n: Int) -> Int { n }

  @class
  label() -> String { "base" }
}

class Derived is Base {
  value(_ n: Int) -> Int { super.value(n) + 1 }

  @class
  label() -> String { super.label() }
}

class Probe {
  @class run() {
    const object = Derived.new()
    const number = object.value(4)
    const text = Derived.label()
  }
}
```

Assert that `object` is `Derived`, `number` is `Int`, and `text` is `String`. Inspect call denotations/dependencies to verify instance `super.value` does not resolve through the class side and class `super.label` does not resolve through the instance side.

- [ ] **Step 2: Run the focused test.**

Run the target with `super_dispatch_preserves_side_and_inherited_constructor_identity`.

---

### Task 7: Field initializer, constructor write, and later mutation

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Add `field_facts_survive_constructor_and_general_writes`.**

Use a typed field, constructor initialization, a mutating method, and a read method:

```phalcom
class Counter {
  _value: Int = 0

  @constructor
  new(_ initial: Int) { _value = initial }

  increment() -> Int {
    _value = _value + 1
    _value
  }

  read() -> Int { _value }
}

class Probe {
  @class run() {
    const counter = Counter.new(4)
    const after = counter.increment()
    counter.read()
  }
}
```

Assert field/parameter facts remain `Int` across initializer, constructor assignment, general assignment, and reads. Confirm the mutation invalidates only dependent refinements and does not make unrelated local facts dynamic.

- [ ] **Step 2: Run the focused test.**

Run the target with `field_facts_survive_constructor_and_general_writes`.

---

### Task 8: Reflection and dynamic pack conservatism

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`
- Reference: `phalcom-core/tests/lang/reflection/reflection_perform_user_method_with_args.ph`

- [ ] **Step 1: Add `reflective_dynamic_pack_stays_conservative_but_keeps_known_independent_facts`.**

Use a user-defined method and reflective `perform` with a dynamic outgoing pack:

```phalcom
class Adder {
  add(_ left: Int, _ right: Int) -> Int { left + right }
}

class Probe {
  @class run(_ target: Object, *args) {
    const known = 42
    const reflected = target.perform(Symbol.new("add(_,_)"), ***args)
    known
  }
}
```

Assert that the reflective call is marked dynamic/opaque rather than falsely proven as `Int`, while the independent `known` binding remains exact `Int`. Record whether the callable status is complete-with-boundary or blocked; do not accept a false diagnostic caused solely by the dynamic send.

- [ ] **Step 2: Run the focused test.**

Run the target with `reflective_dynamic_pack_stays_conservative_but_keeps_known_independent_facts`.

---

### Task 9: Cross-module import/export and dispatch dependency

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`
- Reference: `phalcom-semantic/tests/workspace.rs`

- [ ] **Step 1: Add `cross_module_exported_constructor_and_method_feed_client_summary`.**

Create resolved modules `app.api` and `app.client`:

```phalcom
// app.api
class Service {
  @constructor
  new() {}

  @class serve() -> Int { 7 }
}
export Service
```

```phalcom
// app.client
import app.api.Service

class Client {
  @class run() { Service.serve() }
}
export Client
```

Assert exported identity resolution, client-to-API dependency edges, and an inferred `Int` result in `Client.run`. Also assert that the API declaration does not leak unrelated names into the client scope.

- [ ] **Step 2: Run the focused test.**

Run the target with `cross_module_exported_constructor_and_method_feed_client_summary`.

---

### Task 10: Collection shapes, destructuring, and rest bindings

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Add `collection_and_destructure_facts_preserve_element_shapes`.**

Use nested collection literals, tuple/list destructuring, and a rest binding:

```phalcom
class Probe {
  @class
  run() {
    const source = [1, 2, 3]
    const [head, *tail] = source
    const pair = (head, tail)
    const record = #{ first: head, remaining: tail }
    record
  }
}
```

Assert that `head` is known as `Int`, `tail` retains a list/pack shape rather than collapsing to `Dynamic`, and the tuple/record expressions preserve their field or positional structure. Verify that rest capture does not erase the independently known type of `head`.

- [ ] **Step 2: Run the focused test.**

Run the target with `collection_and_destructure_facts_preserve_element_shapes`.

---

### Task 11: Incremental edit, removal, re-addition, and deterministic recovery

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Add `dependency_edit_remove_readd_recomputes_only_affected_summary`.**

Drive one `SemanticWorkspaceSession` through four revisions:

1. `Api.value() -> Int` and `Client.read() { Api.value() }`.
2. Change only the API body while preserving `Int`; assert API recomputes and client analysis is reused.
3. Change API return surface to `String`; assert dependent client recomputes and its inferred result changes.
4. Remove and re-add the API declaration; assert stale callable/declaration facts disappear, then recover deterministically when the declaration returns.

Assert recomputed/reused counts, `Arc` identity for unaffected products, reverse dependency edges, diagnostics, and stable snapshot output across two identical revision sequences. This is the highest-value incremental scenario because it tests semantic result correctness and cache ownership together.

- [ ] **Step 2: Run the focused test.**

Run the target with `dependency_edit_remove_readd_recomputes_only_affected_summary`.

---

### Task 12: Full verification and failure classification

**Files:**
- Modify: `phalcom-semantic/tests/semantic_complex_analysis.rs`

- [ ] **Step 1: Run the complete new integration file.**

```sh
cargo test -p phalcom-semantic --test semantic_complex_analysis -- --nocapture
```

- [ ] **Step 2: Classify failures by semantic tier.**

For each failing scenario, record whether failure is parser coverage, source collection, local transfer/join, dispatch identity, callable-summary convergence, dynamic-boundary policy, module resolution, or invalidation. Do not weaken expected assertions to make a test pass.

- [ ] **Step 3: Run neighboring regression suites.**

```sh
cargo test -p phalcom-semantic --test semantic_authority_composition --test callable_dependency_invalidation --test product_stability_invalidation
cargo test -p phalcom-core --test invariants
```

- [ ] **Step 4: Update graphify after code changes.**

Run:

```sh
graphify update .
```

- [ ] **Step 5: Commit the new integration test separately.**

```sh
git add phalcom-semantic/tests/semantic_complex_analysis.rs
git commit -m "test semantic analyzer complex scenarios"
```

## One Fully Worked Complex Example

The first implementation should begin with this scenario because it combines refinement, mutation, looping, inheritance, `super`, and return-summary behavior in one analyzable program:

```phalcom
class Base {
  @constructor
  new() {}

  value(_ n: Int) -> Int { n }
}

class Derived is Base {
  value(_ n: Int) -> Int {
    let total = n
    for i in [1, 2, 3] {
      if (i == 2) { continue }
      total = total + i
    }
    super.value(total)
  }
}

class Probe {
  @class
  run(_ input: Object) {
    if (input.is(Int)) {
      return Derived.new().value(input)
    } else {
      throw input
    }
  }
}
```

Expected semantic result:

- `input` is `Object` at entry and `Int` in the positive branch.
- `total` remains `Int` through loop back-edge, `continue`, mutation, and tail expression.
- `Derived.new()` is specialized to `Derived` through inherited constructor behavior.
- `super.value(total)` resolves to the instance-side `Base.value` and returns `Int`.
- The `else` path contributes no normal return because it throws.
- `Probe.run` therefore has one normal return value, `Int`, rather than `Int | Object`, `Unit`, or `Never`.
- The flow graph contains branch and loop structure; the callable records dependencies on the constructor and superclass method.

This example is intentionally diagnostic. If it fails, the assertion should identify whether the problem is branch refinement, loop-state joining, mutation invalidation, constructor self specialization, instance-side `super` dispatch, or normal-return summary construction.
