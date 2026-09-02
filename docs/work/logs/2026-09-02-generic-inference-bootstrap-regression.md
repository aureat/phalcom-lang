# Generic Inference Regression: VM Bootstrap Stack Overflow and Either Runtime Failure

Date: 2026-09-02

Status: Fixed, tested, and pushed to `main`.

Fix revision:

```text
222a943ac266d055273787d3dbe2fba3c159287e
fix(semantic): preserve generic inference progress
```

## Summary

The original Either runtime failure was not caused by Either, heap allocation,
VM size, or runtime recursion. `VM::new()` compiles and semantically analyzes
the canonical Universe, and that bootstrap exposed a non-terminating generic
inference path.

The first bad revision was:

```text
91cb91cc417ffe69f1ac359d3af8a777a98a1b5a
feat(semantic): complete generic inference outcomes
```

The failure had four related but separable parts:

1. Canonical subtype expansion could recurse without changing either term.
2. A combined canonical subtype match reversed `<:` direction in one case.
3. Deferred `var_terms` publication reported `Changed` when state was already
   identical, preventing fixed-point convergence.
4. After bootstrap was fixed, Either exposed a separate runtime prelude lookup
   problem and higher-order contextual inference loss.

## Original symptoms

The minimal reproducer was:

```text
phalcom-core/src/product.rs
empty_products_normalize_to_unit_without_heap_allocation
```

It creates a VM on a 32 MiB Rust thread stack. On the known-good baseline it
passed. On `91cb91c` and later revisions it overflowed the host Rust stack
during `VM::new()`.

The Either runtime test initially reported the same stack overflow:

```text
phalcom-core/tests/core/either/runtime.rs
either::runtime::either_runtime_surface_produces_expected_values
```

Increasing the stack did not address the cause. It only changed how much
recursive inference work could occur before failure.

## Root cause 1: non-progressing canonical subtype expansion

`InferenceSession::subtype_terms` had a combined arm equivalent to:

```rust
(InferenceTerm::Canonical(ty), other)
| (other, InferenceTerm::Canonical(ty)) => {
    let canonical = self.type_id_to_inference(*ty, &HashMap::new(), store);
    self.subtype_terms(&canonical, other, store, hier)
}
```

Caller-owned generic parameters are intentionally rigid. With an empty
substitution, a parameter such as:

```text
TypeData::Parameter(TypeParameterId(...))
```

remains:

```text
InferenceTerm::Canonical(parameter_type_id)
```

Therefore a relation like:

```text
Applied<F, ?A, ?B> <: Canonical(P)
```

could re-enter `subtype_terms` with the same canonical term forever. The
diagnostic expansion showed the rigid parameter and the unresolved applied
term immediately before the host stack overflow.

The important semantic rule is: caller parameters must stay rigid. They must
not be converted into fresh inference variables merely to make this relation
solvable.

## Root cause 2: subtype direction reversal

The same OR-pattern also treated these directional relations as if they were
interchangeable:

```text
Canonical(T) <: other
other <: Canonical(T)
```

Both paths then called:

```rust
self.subtype_terms(&canonical, other, store, hier)
```

For the second relation this changed `other <: Canonical(T)` into
`Canonical(T) <: other`. Even if recursion terminated, the solver could make
the wrong semantic decision.

## Root cause 3: false fixed-point progress

The deferred branch in `InferenceSession::unify_terms` stored an unresolved
term in `var_terms` and always returned `SolveEffect::Changed`:

```rust
self.var_terms.insert(rep, term.clone());
Ok(SolveEffect::Changed)
```

Constraint replay is expected in the fixed-point solver. Replaying the same
deferred term is not progress when:

```text
self.var_terms[rep] == term
```

Reporting `Changed` for that replay can consume the SCC/query budget even
though persistent solver state did not change. This was an independent solver
invariant violation. It was covered directly with an idempotency test rather
than hiding it by increasing `max_scc_iterations`.

## Separate Either failures after bootstrap

The stack fix made `VM::new()` succeed, but the Either runtime test still
appeared stalled. Temporary phase markers separated the phases:

```text
A: before VM::new
B: after VM::new
C: after source compilation
D: after execution
```

The roughly 55-second debug Universe bootstrap occurred between A and B. The
remaining failure was after compilation, between C and D, and surfaced as an
`UndefinedVariable("Tuple")` runtime failure.

Pattern class tests in:

```text
phalcom-core/src/compiler/lib/patterns.rs
```

were emitting ordinary `GetGlobal` references for linked prelude classes.
`emit_class_test` and `emit_required_class_check` now use
`emit_global_reference`, preserving linked prelude lookup.

The complete Either module then exposed three higher-order generic-call
failures. `apply_generic_callable_inner` was materializing expected argument
types too early, losing solver-local inference terms inside callable
expectations. A closure could receive canonical parameter data instead of a
callable term such as `Callable(Var(A), Var(B))`, causing
`Unknown(UncheckedExpression)` before solving.

`InferenceSession::term_for_expected` now recursively rewrites solved
inference variables inside compound expectations while retaining unresolved
variables. This preserves contextual information for closures without making
rigid caller parameters flexible.

## Final implementation

Production changes were limited to:

- `phalcom-semantic/src/checker/inference.rs`
  - Added recursive `term_for_expected` contextual-term rewriting.
  - Made deferred `var_terms` insertion idempotent.
  - Added non-progress protection for canonical structural unification and
    supertype projection.
  - Split canonical subtype handling into directional branches.
  - Kept rigid canonical parameters pending instead of instantiating them.
- `phalcom-semantic/src/checker/call.rs`
  - Passed rewritten inference terms into generic callable argument
    expectations.
- `phalcom-core/src/compiler/lib/patterns.rs`
  - Routed class references through linked/global reference emission.
- `phalcom-semantic/tests/semantic/foundations/inference.rs`
  - Added direct solver regression coverage.

Temporary EITHER phase markers and VM-size diagnostics were removed after
localization. No diagnostic tracing belongs in the committed Either test
helpers.

## Regression coverage

Direct tests now cover:

- `rigid_rhs_canonical_parameter_does_not_recurse_or_instantiate`
- `rigid_lhs_canonical_parameter_preserves_subtype_direction`
- `deferred_subtype_is_rechecked_after_nested_variable_solves`
- `deferred_unification_reaches_underconstrained_fixed_point`
- `fixed_point_solves_deferred_unification_after_nested_binding`
- `repeated_deferred_unification_is_idempotent`
- `expected_term_rewrites_solved_nested_variables`

These tests check termination, direction preservation, rigid-parameter
preservation, deferred obligation replay, fixed-point convergence, idempotent
state publication, and contextual higher-order inference.

## Verification

The following evidence was collected from the final fix revision:

```text
cargo test -p phalcom-semantic
```

Passed semantic unit and integration suites: 58 unit tests, 949 integration
tests, and 54 ignored tests.

```text
RUSTFLAGS='' cargo test -p phalcom-core \
  empty_products_normalize_to_unit_without_heap_allocation \
  -- --nocapture
```

Passed with the existing 32 MiB test stack. No stack-size increase was used as
the fix.

```text
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  either::runtime::either_runtime_surface_produces_expected_values \
  -- --nocapture
```

Passed.

```text
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  either:: -- --nocapture
```

Passed: 27 Either tests, 0 failures.

A broader current-core run still exposed a separate existing limitation in:

```text
language::algebraic_data_scenarios::adt_vert_07_core_option_native_representation_execution
```

It returned `GenericInferenceUnderconstrained`. That failure was not part of
the Either regression and was not claimed as fixed by this change.

## Best diagnosis sequence, looking back

1. Reproduce on a fresh worktree from current `main`; do not rely on a
   disposable bisect worktree's modifications.
2. Run the minimal `VM::new()` reproducer before changing Either or runtime
   code.
3. Compare the first bad revision with the last good revision, focusing on
   inference machinery introduced by the regression commit.
4. Instrument only phase boundaries around `VM::new`, compilation, and
   execution. This distinguishes bootstrap, semantic compilation, and runtime
   failure quickly.
5. Add a temporary non-progress assertion around canonical expansion. Print
   both terms and `TypeData`; this identifies a rigid parameter preserved as
   `Canonical(P)` instead of producing an unbounded backtrace.
6. Split directional relation cases before changing solver behavior. Check
   whether every recursive call preserves relation direction and structurally
   changes its input.
7. Test fixed-point progress directly by replaying the same unresolved term.
   This proves whether `Changed` means actual persistent mutation.
8. Once bootstrap passes, rerun the exact Either test with A/B/C/D markers.
   Do not classify a post-bootstrap failure as inference or runtime until the
   phase is known.
9. For higher-order failures, trace `ExpectedType`, `InferenceTerm`, and
   whether closure arguments produce `Known` evidence before modifying ADT
   unification.
10. Remove instrumentation, run direct inference regressions, the full
    semantic suite, the VM bootstrap reproducer, the full Either module, and
    then relevant broader tests. Record unrelated baseline failures
    separately.

## Future lessons and prevention rules

- `SolveEffect::Changed` means persistent state changed. Every map/set/table
  write path needs an equality or deduplication check before returning
  `Changed`.
- Never combine directional `<:` cases when recursive handling differs. Keep
  left-canonical and right-canonical paths explicit.
- Canonical expansion is not guaranteed to become structural. A rigid or
  atomic canonical term is a boundary; recursion must stop or return a
  pending/appropriate outcome there.
- Query budgets are safety guards, not normal convergence mechanisms. Healthy
  inference must reach a fixed point before budget exhaustion.
- Preserve solver-local inference terms through contextual typing. Converting
  compound expectations to canonical `TypeId`s too early loses constraints
  needed by closures and higher-order calls.
- Keep caller-owned generic parameters rigid. Only instantiate parameters of
  the generic callable currently being applied.
- Use minimal reproductions and phase markers before broad instrumentation.
  The first useful boundary was `VM::new`; the second was C-to-D around
  execution.
- Validate fixes against current `main`, not only the historical first-bad
  checkout. Later inference changes can alter the affected control flow.
- Treat a passing focused test as scoped evidence. Run direct solver tests,
  the complete feature module, and broader suites before declaring the issue
  closed.
