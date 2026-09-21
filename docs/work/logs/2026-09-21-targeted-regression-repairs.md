# 2026-09-21 Targeted Regression Repairs

## Scope

This record covers the parser snapshots, VM walk test, and seven targeted regressions repaired in this session:

- AST snapshots for calls and source fields;
- VM walk test setup for selector-shaped method names;
- product expansion lowering for anonymous and exact products;
- native error origin wrapping for guest re-entry and family-pattern failures;
- ADT singleton and nullary pattern exhaustiveness;
- canonical time-surface callable analysis.

## AST Snapshots

Three parser snapshots had drifted with intentional AST representation changes. Ordinary method calls now print `invocation_authority: Ordinary`, and `__result` in an `@ensures` source expression now prints `FieldKind::Source`.

The snapshots were updated to describe the current AST output.

Changed:

- `phalcom-ast/tests/snapshots/integration__parser__multiple_statements_semicolon_separated.snap`;
- `phalcom-ast/tests/snapshots/integration__parser__for_loop_over_list.snap`;
- `phalcom-ast/tests/snapshots/integration__parser__ensures_attribute_on_getter.snap`.

## VM Walk Setup

The stack-walk ordering test attempted to reach a callback through `List#fold(initial,using)`, which is unavailable on the runtime surface loaded by that unit test. The outer send raised before the callback block was entered, leaving only the main frame and the `total(_)` frame.

The test now enters the block through a locally declared `Cart#callCallback(_,_)` method. It still verifies oldest-first ordering and that the block reports `total(_)` as its lexical enclosing selector.

Changed: `phalcom-core/src/vm/walk.rs`.

## Product Expansion Lowering

`compile_positional_expansion` treated missing `positional_expansions` evidence as a compiler error in every fallback branch. Semantic analysis deliberately omits iterator-lowering evidence for statically known `Tuple` and `Unit` values because the VM uses direct positional lanes for those cases. The generic dynamic fallback is still compiled after that fast-path probe, even when it is unreachable for a tuple or unit source.

The compiler now preserves semantic-authoritative lowering when it exists, retains the fail-closed path for sources explicitly proved non-iterable, and otherwise emits the established dynamic iteration fallback. This removes the spurious `MissingPositionalExpansionLowering` error without weakening the non-iterable proof boundary.

Changed: `phalcom-core/src/compiler/lib/expr.rs`.

## Native Error Origin

Primitive dispatch unconditionally wrapped primitive errors in `PhError::NativeOrigin`. That wrapping was incorrect for a selector-pattern mismatch produced before native dispatch owned the call, and for `DepthExceeded` returned from guest execution re-entered by a native primitive while guest frames remained active.

Primitive error handling now centralizes this distinction. Direct native failures retain native provenance. `SelectorPatternMismatch` remains direct. `DepthExceeded` remains direct when it escapes active guest frames, while native-only recursion control continues to retain native provenance.

Changed: `phalcom-core/src/vm/send.rs`.

## ADT Exact-Case Flow

The semantic checker invalidated mutable-binding exact-case knowledge after every call. A statically resolved data or variant constructor is not an opaque mutation boundary. In the singleton/nullary ADT scenario, an unrelated `State::Ready()` constructor call erased the known exact case of `singleton`, causing a false `MatchNonExhaustive` diagnostic.

Resolved data and variant constructor calls now preserve exact-case flow facts. Native, structural, abstract, callable-value, and ordinary calls retain conservative invalidation.

Changed: `phalcom-semantic/src/checker/call.rs`.

## Canonical Time Callable Analysis

Canonical source-owned `data` and `enum` declarations, including `Duration`, could be resolved from a user body but did not publish ready hierarchy products. Resolving `Duration` in synthetic `Probe.run` therefore recorded an unresolved `HierarchyEdge` dependency. Dependency validation discarded the body analysis and fell back to the base snapshot, so the synthetic callable was absent from `callable_analyses`.

Semantic bootstrap now registers source-owned canonical data and enum declarations with their implicit `Object` hierarchy and publishes hierarchy products for all bootstrapped declarations, including roots without a superclass.

Changed: `phalcom-semantic/src/session.rs`.

## Validation

The following commands passed serially after all corrections:

```sh
RUSTFLAGS='' cargo test -p phalcom-ast --test integration
RUSTFLAGS='' cargo test -p phalcom-core vm::walk --lib
RUSTFLAGS='' cargo test -p phalcom-core --test core compiler_product_optimization::unprovable_anonymous_product_uses_dynamic_pack_route -- --exact
RUSTFLAGS='' cargo test -p phalcom-core --test core compiler_product_optimization::tuple_and_record_exactness_ignores_representation_and_record_presentation_order -- --exact
RUSTFLAGS='' cargo test -p phalcom-core --test core execution_depth_limits::native_reentry_is_bounded -- --exact
RUSTFLAGS='' cargo test -p phalcom-core --test core execution_depth_limits::traceback_survives_a_frame_that_executed_nothing -- --exact
RUSTFLAGS='' cargo test -p phalcom-core --test core execution_family_runtime::family_pattern_mismatch_returns_typed_error_before_dispatch -- --exact
RUSTFLAGS='' cargo test -p phalcom-core --test core language::algebraic_data_scenarios::adt_run_03_singleton_and_nullary_patterns_select_separately -- --exact
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic semantic::capabilities::time::monotonic_time_surface_publishes_expected_types -- --exact
```

Each selected test passed with zero failures.
