# SC-2 — Static Verification and Repository Reconciliation Report

**Project:** Phalcom  
**Date:** 2026-09-02
**Repository:** `aureat/phalcom-lang`  
**Verified source baseline:** `main@c5916d14`
**Commit:** `fix(semantic): satisfy SC-2 final lint gate`
**Artifacts verified:**

```text
SC-2-generic-callable-application-receiver-specialization-technical-spec.md
SC-2-generic-callable-application-receiver-specialization-implementation-plan.md
```

**Verification class:** source/static planning verification plus local executable verification. The historical sections below retain their original static-reconciliation evidence; the executable completion addendum records current local results and remaining repository-wide baseline boundaries.

---

# 1. Verification result

## Overall result

**PASS — implementation planning consistency, with explicit execution caveat.**

The two SC-2 deliverables are consistent with:

- the ratified/revised generic and 04.5 semantic intent;
- the current implementation architecture at the pinned `main` commit;
- the newly landed ADT/GADT/associated lookup products;
- the current semantic test organization;
- the current separation between canonical `TypeId` and solver-local inference variables.

The deliverables intentionally change one current test policy—expected-result-only generic inference—because the current test contradicts the broader ratified 01.5/04.5 semantics and the already-documented generic-getter direction. That change is called out explicitly rather than silently reconciled.

---

# 2. Baseline verification

GitHub `main` resolved to:

```text
01e19adb86186d67212b558ba76f54f79e2b5d9f
```

with commit message:

```text
feat(core,semantic,vm): canonical native enums and associated lookup implementation
```

This is newer than the previous project audit baseline `49d74f9a7d95f695c8ff38c954eca938e6fec16f`, so the SC-2 analysis re-read all material call/inference/ADT integration seams rather than assuming the earlier audit remained exact.

GitHub combined status returned no status entries for the pinned commit. GitHub workflow-run lookup returned no workflow runs. Therefore:

```text
CI status: unavailable / none published through connected GitHub data
cargo verification: not executed in this session
```

---

# 3. Source files directly re-inspected

The following current files were read from the pinned commit and used as implementation evidence:

```text
phalcom-semantic/src/checker/call.rs
phalcom-semantic/src/checker/inference.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/associated.rs
phalcom-semantic/src/checker/declaration_signature.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/body.rs

phalcom-semantic/src/types/relation.rs
phalcom-semantic/src/types/environment.rs
phalcom-semantic/src/types/substitution.rs
phalcom-semantic/src/types/parameter.rs
phalcom-semantic/src/types/outcome.rs
phalcom-semantic/src/types/denotation.rs

phalcom-semantic/src/dispatch.rs
phalcom-semantic/src/signature.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/diagnostic.rs
```

Current tests re-inspected:

```text
phalcom-semantic/tests/semantic/foundations/mod.rs
phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs
phalcom-semantic/tests/semantic/foundations/generics_core.rs
phalcom-semantic/tests/semantic/capabilities/mod.rs
phalcom-semantic/tests/semantic/capabilities/generics.rs
phalcom-semantic/tests/semantic/adts/mod.rs
phalcom-semantic/tests/semantic/adts/constructors.rs
phalcom-semantic/tests/semantic/adts/generics.rs
phalcom-semantic/tests/semantic/incremental/mod.rs
```

---

# 4. Normative/planning sources reconciled

The SC-2 deliverables were reconciled with the following existing project documents:

```text
docs/impl/semantic/semantic-completeness/part-1/
  01.5-canonical-generic-type-semantics-and-declaration-model.md

docs/impl/semantic/semantic-completeness/part-4/
  04.5-expression-typing-generic-inference-flow-semantics-and-diagnostic-explanations.md

docs/impl/semantic/semantic-completeness/part-5/
  05-advanced-kinds-constraints-effects-and-proofs-REVISED.md

docs/impl/semantic/semantic-completeness/part-6/
  06-language-comparisons-and-design-rationale-REVISED.md

docs/impl/semantic/semantic-completeness/part-7/
  07-consolidated-implementation-plan-and-decision-register-REVISED.md

docs/spec/semantic-analyzer/
  07-generic-inference-engine.md

docs/work/deferred/
  generic-on-getter.md

docs/impl/adt-gadt-associated-lookup/part-3/
  03-associated-resolution-family-values-generic-specialization-invocation-typing-technical-spec.md
  03-associated-resolution-family-values-generic-specialization-invocation-typing-implementation-plan.md
```

The semantic intent from these documents was retained. Their historical statements about repository state were treated as observations, not timeless normative truth.

---

# 5. Important stale repository-state observations found and corrected

## 5.1 `TypeData::Infer`

Older completeness documents describe `TypeData::Infer` as active compatibility debt.

Current pinned source audit finds no production ordinary-inference `TypeData::Infer` path. Search hits are documentation only.

SC-2 therefore **does not** include a task to migrate ordinary inference variables out of `TypeStore`; that migration is already reflected in current source.

Protected SC-2 law:

```text
InferVarId != TypeId
```

## 5.2 `LocalConstraintSolver`

Older plans describe `LocalConstraintSolver` as active ordinary generic inference infrastructure.

Current pinned source search finds it only in documents/historical analysis. Ordinary call inference uses `checker/inference.rs::InferenceSession`.

SC-2 therefore extends `InferenceSession`; it does not migrate from `LocalConstraintSolver`.

## 5.3 Expected-result generic inference

Current `semantic/capabilities/generics.rs` contains a test requiring:

```text
<T>() -> T under expected Int
-> still underconstrained
```

But the broader generic semantic architecture says expected-result context participates in selection, and the generic-getter design explicitly requires result-directed inference for zero-argument generic getters.

SC-2 deliberately ratifies:

```text
<T>() -> T under expected Int
-> T := Int
-> known result, but context is not Established value evidence
```

The implementation plan explicitly requires rewriting that existing regression test rather than hiding the policy change.

---

# 6. Current architectural claims verified

## 6.1 Canonical application funnel — VERIFIED

`checker/call.rs` contains:

```text
CallableApplicationTarget
CallPremise
ApplicationArgument
ArgumentBindingPlan
apply_resolved_callable
```

Generic and non-generic resolved applications use this funnel. The SC-2 plan correctly preserves it as the application owner.

## 6.2 Session-local inference — VERIFIED

`checker/inference.rs` defines solver-local `InferenceTerm` and `InferVarId` state. Variables carry `KindId`. `InferenceOutcome` currently distinguishes:

```text
Solved
Underconstrained
Conflicting
Blocked
Cancelled
BudgetExceeded
InternalFailure
```

There is no current `Ambiguous` variant; the SC-2 plan's ambiguity work is therefore a real delta.

## 6.3 Expected-type solver terms — VERIFIED

`checker/expected.rs` supports:

```text
ExpectedType::Inference { term, origin }
```

and propagates inference-shaped expectations into collection/callable components. SC-2's contextual-closure/result-directed design uses existing machinery rather than inventing a new expectation channel.

## 6.4 Direct-only receiver specialization gap — VERIFIED

`CheckingContext::specialize_dispatch_signature` currently obtains a substitution from the actual applied receiver root, then substitutes parameters/return and `Self`.

This does not project a receiver such as:

```text
Child<Int> is Parent<List<T>>
```

into the selected declaring owner's `Parent::T` parameter identity.

The SC-2 owner-relative specialization task is justified.

## 6.5 Generic-supertype projection utility already exists elsewhere — VERIFIED

`checker/associated.rs` currently contains `project_supertype_arguments(...)`, which walks generic superclass templates with `TypeEnvironment`/`TypeView`.

The plan correctly extracts/generalizes this rather than writing a second inheritance-specialization algorithm.

## 6.6 Generic constraints are not receiver-specialized by current dispatch projection — VERIFIED

Current dispatch specialization mutates parameter/return type knowledge but leaves `CallableSignature.generics` unchanged.

Therefore a method constraint referring to a class generic parameter can remain owner-unspecialized. SC-2 explicitly closes this.

## 6.7 Compound inference subtyping gap — VERIFIED

Current `InferenceSession::subtype_terms` has explicit variable/canonical cases and otherwise falls back to `unify_terms`.

Canonical `types/relation.rs`, by contrast, already knows declaration variance and callable variance.

The plan correctly requires solver-local relation decomposition to conform to canonical relation semantics.

## 6.8 Single-upper default risk — VERIFIED

Current bound reconciliation can bind an unsolved variable to its sole upper bound when no lower bound exists.

Because `GenericWhere` subtype constraints are added to the same session before solving, a declaration restriction can in principle participate as an unintended default candidate.

The SC-2 selection-versus-restriction distinction addresses a real implementation risk.

## 6.9 Sticky argument-only underconstraint — VERIFIED

The generic call path records the argument-phase underconstrained result and later consults it even after expected-result constraints are added.

This matches the existing test policy that expected-only result inference stays underconstrained. The SC-2 plan correctly identifies the code path that must change for the ratified contextual-selection rule.

## 6.10 Materialization error loss — VERIFIED

`InferenceSession::materialize` converts some failed `apply_type_form`/exact-case operations into `UnderconstrainedInference` with an empty unsolved-variable list.

The plan's structured materialization failure domain is justified.

## 6.11 Hidden solver iteration constant — VERIFIED

`InferenceSession::solve_with_control` has a local `max_passes = 16` in addition to shared `CheckerControl` step/cancellation handling.

The plan correctly requires this to be incorporated into a named/shared convergence policy rather than silently surviving as a magic semantic cutoff.

## 6.12 Constructor positional generic guessing — VERIFIED

`checker/expression.rs::synthesize_unqualified_call` has a type-name branch that collects runtime argument types and, when their count matches declaration generic parameter count, applies them directly to the declaration type form.

The plan correctly removes this as semantic generic-constructor inference.

## 6.13 Variant constructor `Object` fallback — VERIFIED

Current associated variant invocation computes constructor parameter type with an `Object` fallback if canonical declaration type is unavailable.

The plan correctly deletes this fallback.

## 6.14 Variant generic owner products — VERIFIED

`EnumInfo` owns the enum `GenericSignature`; `VariantInfo` owns result/exact-case templates and `CaseTypeEnvironment`; `VariantConstructorSignature` owns formal payload parameters.

These are sufficient semantic inputs for residual owner generic inference without redesigning enum declaration products.

## 6.15 `Result::Ok` underconstraint requirement — VERIFIED FROM PROJECT PLAN

The existing ADT associated-invocation plan explicitly says:

```text
Result<T,E>::Ok(T)
Result::Ok(1) with no expected result
-> E remains result-relevant and unsolved
-> underconstrained
-> never Dynamic/Object/first-use default
```

SC-2 carries that rule forward.

## 6.16 Family callable generic erasure risk — VERIFIED

First-class family type members use canonical `TypeData::Callable`, which carries parameter/return types but no `GenericSignature`.

At the same time `AssociatedValueDenotation` retains behavioral receiver/target identity or associated member identity.

The plan's recommendation to recover/re-resolve the canonical generic declaration at family application is therefore grounded and avoids introducing first-class `forall` callable types.

## 6.17 Union receiver call gap — VERIFIED

Current ordinary dispatch owner resolution handles nominal/class-object/applied receiver forms, not `TypeData::Union` as one call target.

Ratified 04.5 semantics explicitly require checking every statically known union receiver arm and joining results.

SC-2 correctly owns that closure.

## 6.18 Generic getters remain deferred — VERIFIED

Current `checker/declaration_signature.rs` hardcodes getter `generics: None`; current expression dispatch also calls `synthesize_get_property` without forwarding `ExpectedType`.

SC-2 does not implement getter AST/signature changes. It only establishes the zero-value-argument expected-result inference semantics that SC-7 will later consume.

---

# 7. Proposed file-path verification

The following proposed create paths were checked against the pinned repository and do not currently exist:

```text
phalcom-semantic/src/types/specialization.rs
phalcom-semantic/tests/semantic/foundations/receiver_specialization.rs
```

The proposed test module locations follow the repository's current descriptive test organization (`foundations`, `capabilities`, `adts`, `incremental`) rather than implementation-part naming.

The incremental module registry currently includes:

```text
callable_dependencies
checker_dependencies
```

so the implementation plan's focused incremental commands use existing module names.

---

# 8. Cross-document consistency audit

## 8.1 Scope consistency — PASS

Both deliverables agree that SC-2 owns:

- receiver specialization;
- ordinary generic application;
- expected-result selection;
- constraint/admissibility semantics;
- HKT local inference;
- constructors/variants/family calls;
- union receiver calls;
- terminal outcomes;
- incremental/conformance tests.

Both keep rows/effects/proofs/generic getters out of scope.

## 8.2 Expected-result policy consistency — PASS

Both documents say:

```text
expected result may select a unique generic solution
expected result is not runtime value evidence
no expected context -> result-only generic remains underconstrained
argument-derived precise result is not overwritten by conflicting context
```

## 8.3 Constraint policy consistency — PASS

Both documents distinguish selection from declaration restriction and forbid a lone upper bound from acting as a default type argument.

## 8.4 Receiver specialization consistency — PASS

Both require:

```text
actual receiver
-> selected declaring owner
-> projected owner generic arguments
-> Self specialization
-> callable-local inference
```

and both require the associated-only superclass projection logic to become shared.

## 8.5 Runtime invariance consistency — PASS

Neither deliverable changes selectors, `CallableId`, class identity, object layout, or runtime specialization model.

## 8.6 SC-3 boundary consistency — PASS

Both keep row variables separate from ordinary type inference variables.

## 8.7 SC-7 boundary consistency — PASS

Both keep getter syntax/signature work deferred while making expected-only zero-argument generic inference a prerequisite.

---

# 9. Artifact-local validation performed

The generated markdown files were checked locally for:

- non-empty content;
- matching pinned baseline SHA;
- explicit companion document names;
- no unfinished placeholder markers;
- required SC-2 decisions present in both documents;
- implementation plan contains focused and full verification commands;
- implementation plan contains a deletion/audit ledger;
- technical specification contains non-goals and acceptance gates;
- verification report distinguishes source/static verification from executable verification.

A ZIP bundle was produced after these checks.

---

# 10. What remains unverified until implementation execution

The following cannot be claimed from source inspection alone:

1. which newly added RED tests fail in exactly the predicted location;
2. whether parser syntax for every HKT/F-bound example is accepted without fixture adjustment;
3. whether `super` source syntax currently reaches the intended semantic path;
4. whether union-call argument-once analysis needs an explicit typed-argument frame beyond `PreAnalyzed`;
5. whether canonical signature-table attachment is necessary for generic family target recovery or dispatch/denotation recovery is sufficient;
6. performance delta after owner-relative projection and richer term decomposition;
7. cargo/clippy/workspace test success after implementation;
8. whether any unrelated current test failure exists on the pinned commit.

The implementation plan handles these as characterization or conditional steps rather than pretending they are already known.

---

# 11. Final verification judgment

The deliverables are suitable as SC-2 implementation authority because they:

- rebase stale historical plans onto current `main`;
- preserve the strongest parts of the existing call/inference architecture;
- identify concrete current bypasses and semantic mismatches;
- explicitly settle expected-result-only inference;
- reconcile generic constraints as formal restrictions without defaulting;
- reuse current generic inheritance and canonical relation machinery;
- integrate the newly implemented ADT/GADT/associated semantics rather than specifying an obsolete pre-ADT model;
- provide tests-first tasks, deletion criteria, incremental gates, and full verification commands;
- do not claim implementation or executable verification that has not occurred.

**Static planning verification: PASS.**  
**SC-2 implementation status at `main@c5916d14`: Tasks 0–14 implemented and locally verified for semantic scope.**
**Repository-wide release gates:** formatting remains blocked by pre-existing repository-wide rustfmt drift; full workspace sweep was stopped after unrelated core runtime/corpus failures while the focused SC-2 and monad gates passed.

---

# 12. Executable completion addendum — 2026-09-02

## SC-2 semantic gates

All commands used `RUSTFLAGS=''` to neutralize repository-local incompatible flag injection.

```text
phalcom-semantic check --tests                              PASS
receiver_specialization                                    6 passed
generic_application                                        7 passed
union_calls                                                6 passed
capabilities::generics                                    15 passed
capabilities::higher_order                                5 passed
capabilities::type_lambdas                                1 passed
adts::constructors                                       13 passed
adts::associated                                   15 passed, 5 GATED
incremental::callable_dependencies                        12 passed
incremental::checker_dependencies                         10 passed
phalcom-semantic unit tests                                59 passed
phalcom-semantic semantic integration                     969 passed, 48 ignored
strict semantic clippy                                    PASS
```

The monad conformance request was run with the libtest spelling `--nocapture`:

```text
phalcom-core --test core monads                             35 passed
```

The user's literal `--no-capture` spelling is rejected by libtest as an unknown option; it does not represent a test failure.

## Verified SC-2 decisions

```text
expected-only inference is selection, not value evidence
one-sided bounds are not defaults
finite ambiguity differs from underconstraint
owner-relative receiver specialization is canonical
union receiver calls check all arms
cold and incremental results are equivalent
```

Task 12 retained row polymorphism and Task 13 generic-getter work outside SC-2; those remain SC-3 and SC-7 boundaries. Five associated visibility/inheritance tests and many broader ADT/match fixtures remain explicitly GATED by missing cross-module or lowering fixtures. No SC-2 test assertions were weakened.

## Task 14 audit and remaining gates

The required deletion searches found no ordinary production `TypeData::Infer`, `TypeStore::infer`, `LocalConstraintSolver`, positional constructor generic guessing, or duplicated receiver-specialization implementation. `fallback_result_type` is restricted to an independent, already-canonical family return type; dependent generic terminal outcomes remain structured and are covered by `generic_family_failure_does_not_publish_fallback_result` and generic proof-integrity tests.

`cargo fmt --all -- --check` was run and fails on repository-wide pre-existing formatting drift, including unrelated AST/core/modules/LSP files. The full `RUSTFLAGS='' cargo test --workspace` sweep was started; it reached unrelated core runtime/corpus tests with failures including `core_collections::range_literals_drive_collection_slices`, ADT runtime/conformance cases, and corpus `booleans`/`bytes_negative`, and was stopped after the slow compiler-fixture sweep continued. The focused core monad suite and all SC-2 semantic gates are green.

No dedicated SC-2 benchmark harness exposes before/after receiver-step, solver-iteration, relation-pair, TypeStore-delta, or union-arm counters. This unavailability is recorded rather than inventing thresholds or measurements.
