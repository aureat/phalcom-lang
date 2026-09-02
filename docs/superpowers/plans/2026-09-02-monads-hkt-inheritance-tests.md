# Monads HKT / Generic-Inheritance Conformance Package Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build `phalcom-core/tests/core/monads`, a user-level Phalcom conformance package that stress-tests higher-kinded parameters, type lambdas, generic inheritance, inherited generic-method specialization, constructor inference, proof/explanation paths, diagnostics, and runtime execution.

**Architecture:** The package mirrors the successful `either` conformance package: ordinary Phalcom source defines the functional hierarchy and probes; Rust tests inspect canonical semantic products and inference explanations; runtime tests compile and execute the same language-level abstractions. Explicit constructor specialization is the stable backbone (`EitherMonad<E> is Monad<<X> =>> Either<E, X>>`), while synthesized constructor abstraction from `F<A> ~ Either<E, A>` is isolated as a deliberate gap detector.

**Tech Stack:** Phalcom source fixtures, `phalcom_ast`, `phalcom_semantic`, `phalcom_core`, Rust integration tests, VM execution.

**Spec:** Conversation-approved design for the `monads` package on branch `tests/monads-hkt-inheritance`.

## Global Constraints

- Package path is exactly `phalcom-core/tests/core/monads`.
- Higher-kinded parameter syntax is `F: Type -> Type`, never `F<_>`.
- Inheritance syntax is `class Child is Parent { ... }`.
- Type-lambda syntax follows the implemented parser: `<X> =>> ...`.
- Runtime success alone is insufficient: successful cases must also assert canonical types and, where inference occurs, explanation/proof provenance.
- Negative cases must assert deterministic semantic diagnostics and must not silently degrade to `Dynamic` or unrelated `Unknown` states.
- Production compiler changes are out of scope unless a named `MON-*` test exposes a genuine implementation defect; any such repair must be minimal and test-driven.

---

## File Structure

- Create `phalcom-core/tests/core/monads/mod.rs` — package module registry.
- Create `phalcom-core/tests/core/monads/README.md` — executable-law catalog and intended proof obligations.
- Create `phalcom-core/tests/core/monads/monads.ph` — `Either`, `Functor`, `Applicative`, `Monad`, `EitherMonad`, `OptionMonad`/simple unary-constructor probes where current core APIs permit them, and reusable algorithms.
- Create `phalcom-core/tests/core/monads/semantic_probes.ph` — positive semantic/inference probes only.
- Create `phalcom-core/tests/core/monads/runtime_probes.ph` — executable values exported for Rust VM assertions.
- Create `phalcom-core/tests/core/monads/support.rs` — fixture, type matcher, lambda/kind/superclass/inference-trace helpers, VM helpers.
- Create `phalcom-core/tests/core/monads/kinds.rs` — HKT kind/bound conformance.
- Create `phalcom-core/tests/core/monads/type_lambdas.rs` — captured outer parameter, canonical lambda, and beta-reduced application integration tests.
- Create `phalcom-core/tests/core/monads/inheritance.rs` — one-hop/multi-hop generic-superclass projection tests.
- Create `phalcom-core/tests/core/monads/inherited_methods.rs` — inherited method owner projection + method-generic specialization tests.
- Create `phalcom-core/tests/core/monads/inference.rs` — argument/receiver/callable/nested constructor constraint solving.
- Create `phalcom-core/tests/core/monads/composition.rs` — `sequence`, Kleisli, and `traverse` semantic tests.
- Create `phalcom-core/tests/core/monads/rejection.rs` — wrong-kind, conflicting-constructor, and underconstrained cases.
- Create `phalcom-core/tests/core/monads/runtime.rs` — VM assertions for `map`, `pure`, `map2`, `flatMap`, Kleisli, and traversal behavior.
- Modify `phalcom-core/tests/core/mod.rs` — register `monads`.

---

### Task 1: Register the package and establish parser/type-formation RED tests

**Files:**
- Create: `phalcom-core/tests/core/monads/mod.rs`
- Create: `phalcom-core/tests/core/monads/README.md`
- Create: `phalcom-core/tests/core/monads/monads.ph`
- Create: `phalcom-core/tests/core/monads/support.rs`
- Create: `phalcom-core/tests/core/monads/kinds.rs`
- Modify: `phalcom-core/tests/core/mod.rs`

**Produces:** a compilable Rust test module and first laws:
- `MON-KIND-01`: `F: Type -> Type` is published as an arrow-kinded declaration parameter.
- `MON-KIND-02`: unary constructors satisfy the bound when used as `F`.
- `MON-KIND-03`: proper types and wrong-arity constructors do not satisfy the bound.

- [ ] Write the Rust tests first, with the smallest Phalcom declarations needed to exercise `class Functor<F: Type -> Type>` and `F<A>`.
- [ ] Run `cargo test -p phalcom-core --test core monads::kinds -- --nocapture` and verify RED for missing package/source support rather than a syntax typo.
- [ ] Add the minimal fixture/source support and module registration.
- [ ] Re-run the targeted tests and require GREEN for positive kind formation; keep negative kind cases in `rejection.rs` rather than weakening positive fixtures.
- [ ] Commit the checkpoint.

### Task 2: Type-lambda capture and canonical application integration

**Files:**
- Modify: `phalcom-core/tests/core/monads/monads.ph`
- Create: `phalcom-core/tests/core/monads/type_lambdas.rs`
- Modify: `phalcom-core/tests/core/monads/support.rs`

**Produces:**
- `MON-LAMBDA-01`: `<X> =>> Either<E, X>` has kind `Type -> Type` inside `EitherMonad<E>`.
- `MON-LAMBDA-02`: `E` remains a free outer declaration parameter while `X` is lambda-bound.
- `MON-LAMBDA-03`: specializing `E = String` yields a constructor canonically equivalent to `<X> =>> Either<String, X>`.
- `MON-LAMBDA-04`: applying that constructor to `Int` beta-reduces to `Either<String, Int>`.

- [ ] Add tests that inspect `TypeData::Lambda`, `TypeLambdaArena`, and `ScopedTypeData`; do not merely compare formatted strings.
- [ ] Run the targeted tests and verify RED if the package cannot expose the needed superclass/type-lambda product.
- [ ] Add only the support helpers needed to inspect canonical lambda/kind data.
- [ ] Re-run targeted tests to GREEN.
- [ ] Commit the checkpoint.

### Task 3: Generic superclass projection through the full hierarchy

**Files:**
- Modify: `phalcom-core/tests/core/monads/monads.ph`
- Create: `phalcom-core/tests/core/monads/inheritance.rs`
- Modify: `phalcom-core/tests/core/monads/support.rs`

**Produces:**
- `MON-INHERIT-01`: `Applicative<F> is Functor<F>` preserves the exact constructor parameter.
- `MON-INHERIT-02`: `Monad<F> is Applicative<F>` preserves it again.
- `MON-INHERIT-03`: `EitherMonad<String>` projects to `Monad<<X> =>> Either<String, X>>`.
- `MON-INHERIT-04`: multi-hop projection reaches `Functor<<X> =>> Either<String, X>>` with no unspecialized `E`.
- `MON-INHERIT-05`: a second concrete subclass hop preserves substitution composition.

- [ ] Write superclass-projection assertions before adding all hierarchy bodies.
- [ ] Run targeted tests and verify RED on missing/wrong projection.
- [ ] Add the minimal hierarchy declarations/bodies needed for semantic publication.
- [ ] Re-run to GREEN and assert canonical `TypeId` identity where the store guarantees interning.
- [ ] Commit the checkpoint.

### Task 4: Inherited generic methods and proof paths

**Files:**
- Modify: `phalcom-core/tests/core/monads/semantic_probes.ph`
- Create: `phalcom-core/tests/core/monads/inherited_methods.rs`
- Modify: `phalcom-core/tests/core/monads/support.rs`

**Produces:**
- `MON-CALL-01`: inherited `Functor.map` on `EitherMonad<String>` has effective shape `(Either<String, A>, (A) -> B) -> Either<String, B>`.
- `MON-CALL-02`: class-level `F` specialization and method-level `A/B` ownership remain distinct.
- `MON-CALL-03`: calling inherited `map` solves `A` from the value argument and `B` from the closure result.
- `MON-CALL-04`: explanation trace records receiver selection and generic constraints/solutions rather than only a final inferred binding.
- `MON-CALL-05`: methods introduced on `Applicative` and `Monad` specialize through the same receiver.

- [ ] Add semantic probe calls and Rust assertions for result binding + explanation trace.
- [ ] Run targeted tests and verify RED for the intended specialization/proof gap.
- [ ] Extend support only with reusable owner/projection/trace matchers.
- [ ] Re-run to GREEN.
- [ ] Commit the checkpoint.

### Task 5: Constructor constraint solving from independent evidence

**Files:**
- Modify: `phalcom-core/tests/core/monads/semantic_probes.ph`
- Create: `phalcom-core/tests/core/monads/inference.rs`

**Produces:**
- `MON-SOLVE-01`: `Monad<F>` receiver evidence fixes `F` for a generic algorithm.
- `MON-SOLVE-02`: `F<A>` value evidence agrees with receiver-derived `F` and solves `A`.
- `MON-SOLVE-03`: `(A) -> F<B>` closure-return evidence agrees and solves `B`.
- `MON-SOLVE-04`: nested `List<F<A>>` evidence is recursively decomposed.
- `MON-SOLVE-05`: repeated uses in one call converge on one canonical constructor solution.
- `MON-SOLVE-06` gap detector: `F<A> ~ Either<String, Int>` may synthesize `F = <X> =>> Either<String, X>`; if unsupported, the test must expose that specific underconstrained/unsupported capability without contaminating the stable hierarchy tests.

- [ ] Write each solver test independently and name the exact evidence origin it expects.
- [ ] Run targeted tests and classify failures as stable-path defect vs isolated gap-detector limitation.
- [ ] Do not weaken solver assertions to “no errors”; require result type and trace evidence.
- [ ] Commit the checkpoint.

### Task 6: Negative diagnostics

**Files:**
- Create: `phalcom-core/tests/core/monads/rejection.rs`
- Optionally create focused source snippets under `phalcom-core/tests/core/monads/invalid/` if inline sources become unreadable.

**Produces:**
- `MON-REJECT-01`: `Int` cannot satisfy `F: Type -> Type`.
- `MON-REJECT-02`: unsaturated/wrong-arity `Either` cannot satisfy unary `F` without a type lambda/partial constructor form.
- `MON-REJECT-03`: `EitherMonad<String>` plus `Option<Int>` evidence yields a deterministic constructor conflict.
- `MON-REJECT-04`: differing fixed constructor parts (`Either<String, A>` vs `Either<Error, B>`) cannot solve one `F` where synthesis is attempted.
- `MON-REJECT-05`: unconstrained `F: Type -> Type` remains explicitly underconstrained; it does not become `Dynamic`.

- [ ] Write rejection tests first and assert exact diagnostic codes/status where the semantic API exposes them.
- [ ] Run and verify each fails for the intended reason before any compiler repair.
- [ ] If a compiler defect is exposed, repair only that named law and re-run the positive suites too.
- [ ] Commit the checkpoint.

### Task 7: Composition algorithms (`sequence`, Kleisli, `traverse`) semantic proofs

**Files:**
- Modify: `phalcom-core/tests/core/monads/monads.ph`
- Modify: `phalcom-core/tests/core/monads/semantic_probes.ph`
- Create: `phalcom-core/tests/core/monads/composition.rs`

**Produces:**
- `MON-COMP-01`: `sequence` propagates `F` beneath `List<F<A>>` and returns `F<List<A>>`.
- `MON-COMP-02`: Kleisli composition propagates `F` beneath callable return positions and returns `(A) -> F<C>`.
- `MON-COMP-03`: `traverse` reconciles receiver-derived `F`, `List<A>`, `(A) -> F<B>`, and `F<List<B>>`.
- `MON-COMP-04`: `EitherMonad<ParseError>` traversal yields `Either<ParseError, List<Int>>` as the canonical result type.
- `MON-COMP-05`: explanation traces contain independent evidence that converges on the same `F` solution.

- [ ] Write semantic tests before implementing/expanding algorithm bodies.
- [ ] Run targeted tests to RED.
- [ ] Add minimal correct Phalcom algorithm bodies.
- [ ] Re-run to GREEN and inspect proof/explanation assertions.
- [ ] Commit the checkpoint.

### Task 8: Runtime execution conformance

**Files:**
- Create: `phalcom-core/tests/core/monads/runtime_probes.ph`
- Create: `phalcom-core/tests/core/monads/runtime.rs`
- Modify: `phalcom-core/tests/core/monads/support.rs`

**Produces:**
- `MON-RUNTIME-01`: inherited `map` transforms `Right` and preserves `Left`.
- `MON-RUNTIME-02`: `pure` constructs the successful branch.
- `MON-RUNTIME-03`: `map2` combines two successes and preserves the first failure path.
- `MON-RUNTIME-04`: `flatMap` chains success and short-circuits failure.
- `MON-RUNTIME-05`: Kleisli composition executes the typed composition correctly.
- `MON-RUNTIME-06`: `traverse` returns the expected successful values and short-circuits on parse/validation failure.

- [ ] Add VM assertions first for exported primitive observations (`Bool`, `Int`, etc.), not heap-layout internals.
- [ ] Run targeted runtime tests and verify RED.
- [ ] Add the runtime probes/algorithm bodies needed to satisfy them.
- [ ] Re-run runtime tests to GREEN.
- [ ] Re-run semantic suites to prove runtime-oriented edits did not weaken types.
- [ ] Commit the checkpoint.

### Task 9: Full verification and package self-review

**Files:**
- Review all files under `phalcom-core/tests/core/monads/`.

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo test -p phalcom-core --test core monads -- --nocapture`.
- [ ] Run `cargo test -p phalcom-core --test core either -- --nocapture` to guard the neighboring generic conformance suite.
- [ ] Run the broader relevant semantic tests for type lambdas/generic inference if package failures led to compiler changes.
- [ ] Verify every success law has at least one canonical-type assertion and every inference law has an explanation/provenance assertion.
- [ ] Verify runtime laws assert observable values independently of semantic assertions.
- [ ] Verify gap-detector tests are clearly isolated from stable conformance tests and cannot make unrelated runtime tests cascade.
- [ ] Compare branch against `main` and review the diff for accidental unrelated changes.
