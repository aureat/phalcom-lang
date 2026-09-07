# SC-1 Final Verification Ledger

Status: implementation evidence is substantial, but SC-1 is not release-complete. This ledger records the current evidence and the remaining acceptance gates. It was prepared in Task 19 from the shared, intentionally dirty checkout; unrelated changes were not reset, cleaned, or staged.

## Scope and dirty-tree boundary

The current checkout contains SC-1 semantic changes mixed with unrelated or parallel work. The visible overlap includes `.codex/`, `docs/.obsidian/workspace.json`, `docs/spec/primitives/Untitled.md`, core compiler/module files, and semantic files outside the narrow Task 17–18 edits. The complete status must be reviewed with `git status --short`; no claim below relies on a clean-tree comparison.

Task 19 changed documentation only. It does not commit or push anything.

## Tasks 0–19 evidence

| Task | Evidence | Status |
|---|---|---|
| 0 | HEAD, dirty overlap, and the three serial baseline focused checks were recorded before implementation. Shared checkout intentionally retained. | Complete; baseline evidence only |
| 1 | Explicit `TypeFormationOutcome` conversion and exhaustive consumers. `RUSTFLAGS='' cargo test -p phalcom-semantic type_annotations` passed in later gates. | Implemented; focused-tested |
| 2 | `KindResolution` propagates invalid syntax without `KindId::TYPE` recovery; recovered-invalid-kind regression covered by type-annotation tests. | Implemented; focused-tested |
| 3 | `TypeLevelBinding` carries type forms versus record rows; row binders avoid `parameter_form`; ordinary row use is rejected. | Implemented; generic/type-annotation evidence |
| 4 | Source type lambdas lower to scoped bound terms with capture-safe nesting/beta behavior; no whole-body `Free` shortcut remains. | Implemented; focused evidence |
| 5 | Generic signatures resolve into pending state, validate variance/constraints, and publish only after all operands are ready. | Implemented; generic and semantic check passed |
| 6 | Declaration headers publish one coherent generic realization; fingerprints use structural kind/signature/supertype metadata, with no raw-ID fallback. | Implemented; workspace/fingerprint evidence |
| 7 | `TypeFormationSite` carries explicit owner and dispatch side for `Self`; ambient declaration lookup was removed from lowering. | Implemented; Self-focused evidence |
| 8 | Instance-side declaration bindings are domain-aware; class-side members do not receive ambient instance generics; method generics overlay. | Implemented; declaration/generic evidence |
| 9 | Written-but-failed superclasses remain distinct from absent superclasses and block ready publication; canonical generic substitution is preserved. | Implemented; workspace/core focused evidence |
| 10 | Transparent aliases have module/session/index surfaces, scoped generic lowering, dependency tracking, imported resolution, and cycle rejection. Alias suite: 13 passed, 5 ignored/gated. | Implemented; focused-tested |
| 11 | `Expr::TypeForm` uses dedicated denotation lowering and handles all outcomes explicitly. Type-form suite: 10 passed. | Implemented; focused-tested |
| 12 | Open record tails produce an explicit unsupported outcome and are not erased into an equivalent closed record. | Implemented; type-annotation evidence |
| 13 | Declaration/class-object fabrication fallbacks were removed; failure categories remain explicit. Deletion searches are clean for prohibited patterns. | Implemented; search/focused evidence |
| 14 | Alias/generic dependencies and structural fingerprints participate in incremental products; alias and incremental cold/parity checks passed. Incremental suite: 110 passed, 5 ignored. | Implemented; focused-tested |
| 15 | Law coverage includes alpha equivalence, capture avoidance, HKT/partial application, row-binder rejection, substitution, and publication checks. | Implemented; focused evidence |
| 16 | Durable metadata/read-model path carries SC-1 publication facts; hostile publication cases reject invalid/blocked formation. | Implemented; metadata evidence |
| 17 | Body checking reuses published callable signatures rather than re-lowering them. `declaration_authority`: 7 passed; `generic`: 86 passed, 4 ignored; semantic check passed. | Implemented; focused-tested |
| 18 | Scoped formatting/diff checks, package tests, workspace check, focused suites, deletion searches, clippy, workspace tests, and incremental checks were attempted serially. Residual red gates are listed below. | Verification complete; release gates red |
| 19 | This ledger records residual failures, evidence classification, deletion gates, and a final commit grouping. | Complete; documentation only |

## Focused verification that passed

Commands were run with `RUSTFLAGS=''` for Cargo because repository flag injection can select incompatible compiler flags.

```text
RUSTFLAGS='' cargo test -p phalcom-modules
RUSTFLAGS='' cargo check --workspace
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic alias
RUSTFLAGS='' cargo test -p phalcom-semantic type_form
RUSTFLAGS='' cargo test -p phalcom-semantic type_annotations
RUSTFLAGS='' cargo test -p phalcom-semantic workspace
RUSTFLAGS='' cargo test -p phalcom-semantic incremental
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic contradictory_branch_does_not_publish_bindings_or_diagnostics
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic refined_branch_with_abrupt_else_publishes_only_normal_value
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic builtin_type_test_dispatch_does_not_require_source_signature_product
RUSTFLAGS='' cargo test -p phalcom-semantic --lib checker::loop_analysis
RUSTFLAGS='' cargo test -p phalcom-core --test core language::corpus::compile_errors
```

The scoped `rustfmt --edition 2024 --check` and `git diff --check` runs for touched SC-1 Rust/fixture files passed. Graphify was updated after the implementation changes and will be updated again after this ledger is added.

## Full-gate residuals

These failures are reproducible from the current shared checkout. They are recorded rather than hidden or weakened.

| Gate | Reproduction and result | Classification |
|---|---|---|
| Full formatting | `cargo fmt --all -- --check` → failure with broad pre-existing formatting drift across core/AST/semantic files. | Outside Task 19; cannot safely rewrite unrelated dirty files. Scoped SC-1 formatting passes. |
| Semantic package | `RUSTFLAGS='' cargo test -p phalcom-semantic` → 856 passed, 2 failed, 100 ignored after the residual fixes. The fail-fast, record-row, checker-smoke, and both SC-1 `for`-loop publication tests now pass; remaining failures are `semantic::foundations::expression_engine::test_keyword_argument_mismatch_detected` and `semantic::support::regressions::union_expectation_rejects_wrong_structural_members`. | Broad baseline/parallel semantic residuals; fail-fast, record-row, checker-smoke, and SC-1 flow publication failures are resolved. |
| Publishability filter | `RUSTFLAGS='' cargo test -p phalcom-semantic publish` → 52 passed, 0 failed after classifying synthetic `Object.is`/`Object.is!` as bootstrap callables. A regression in `incremental::checker_dependencies` proves these built-in dispatch reads do not require source signature products. | Fixed in this residual unit; ordinary source callable dependencies remain query-owned. |
| Workspace tests | `RUSTFLAGS='' cargo test --workspace` → 365 passed, 34 failed, 33 ignored. Failures include core range/ADT, language corpus arithmetic/bindings/classes/collections/control-flow/functions/indexing/iteration/runtime/streams, missing golden examples, module-linking duplicate selector, standalone project reflection, and universe/core-new fixtures. | Broad pre-existing or parallel core/module/fixture failures; not introduced by Task 19. The focused sealed-superclass compile-error fixture was repaired and passed. |
| Workspace clippy | `RUSTFLAGS='' cargo clippy --workspace --all-targets --all-features -- -D warnings` → SC-owned type-complexity/borrow issues were fixed; remaining lints are in unrelated semantic ADT tests and core match/module/VM files (`variants.rs`, `constructors.rs`, `exhaustiveness.rs`, `match_expr.rs`, `builtin_materialize.rs`, `semantic_lowering.rs`, `vm/adt.rs`). | Outside SC-1 Task 19; exact remaining locations preserved for follow-up. |
| Workspace type metadata | Included in workspace failure: `type_metadata_invariants::reified_nominal_types_preserve_canonical_identity` fails `resolved_int.is_some()`. | Baseline/parallel identity setup failure; no Task 19 code path. |

“Baseline/outside” means the failure is in an unrelated assertion or file and remained after the narrow SC-1 verification fixes. Because this checkout is intentionally shared and dirty, this is not a claim that a pristine historical checkout has been reconstructed.

### Residual debugging record

- `checker::context::tests::fail_fast_policy_panics_only_after_recording_incident`: initial unit-test reproduction failed because the default Cranelift test binary could not catch this panic. The assertion was moved to the existing integration regression module: the parent verifies incident state before/after contained recording, then runs an ignored child process that exercises `FailFast` and checks the recorded incident in the panic payload. `RUSTFLAGS='' cargo test -p phalcom-semantic fail_fast_policy_panics_only_after_recording_incident -- --nocapture` now passes (one parent pass, one intentionally ignored child). Production behavior is unchanged.
- The two flow failures shared one semantic cause: synthetic `Object.is`/`Object.is!` dispatch was recorded as a source-owned callable-signature dependency even though its bootstrap surface has no source signature product. `is_builtin_type_test_callable` now excludes only those exact built-in selectors from query-product dependency recording; ordinary source callable dependencies remain unchanged. Both original flow tests and `builtin_type_test_dispatch_does_not_require_source_signature_product` pass.
- `semantic::capabilities::flow_loops::break_and_continue_preserve_loop_exit_and_backedge_facts` and `loop_fixpoint_preserves_mutated_integer_and_abrupt_edges`: `for` dispatch inspected canonical `Iterable`, but body publication captured `DeclarationSurface(collections.iterable::Iterable)` even though it is immutable bootstrap input and has no revision-current source product. Canonical Universe declarations are now excluded from declaration-surface/callable-signature dependency capture while declaration-shell dependencies remain tracked. Both original loop tests and `for_loop_callable_body_publishes_with_bootstrap_iteration_protocol` pass; full package reached 856 passed, 2 failed.

## Deletion and invariant ledger

The following searches were run against the final implementation tree:

```text
rg "KindSyntax::Invalid" phalcom-semantic/src/types/annotation.rs
rg "tail: _" phalcom-semantic/src/types/annotation.rs
rg "ScopedTypeData::Free\(body" phalcom-semantic/src/types
rg "unwrap_or_else.*nominal" phalcom-semantic/src
rg "resolve_type_parameter" phalcom-semantic/src
rg "UnannotatedDeclaration" phalcom-semantic/src/types
rg "Statement::TypeAlias" phalcom-modules phalcom-semantic
```

Results: invalid kind syntax has explicit invalid outcomes; no ignored tail, whole-body-free lambda, resolved-declaration nominal fabrication, or stale TypeId-only binding path remains. Remaining `resolve_type_parameter` and `UnannotatedDeclaration` references are legitimate compatibility/absence paths, not type-formation failure recovery. Type aliases occur in module interface/session/index paths. `Expr::TypeForm` calls dedicated type-form lowering and explicitly handles every outcome. `RecordRow` binders remain separate from `TypeData::Parameter`.

## Cohesive commit organization

Do not commit from Task 19. After review and once the shared checkout is separated, stage only files belonging to each unit. Recommended grouping follows dependency order:

1. outcome algebra, invalid-kind lowering, type-level bindings, row safety, and scoped source lambdas;
2. atomic generic signatures, declaration headers/fingerprints, `Self` sites, declaration-side scope, and superclass publication;
3. transparent aliases, alias cycles/dependencies/indexing, type-form values, open-tail handoff, and fabrication-fallback removal;
4. durable metadata/export, incremental invalidation/cold parity, canonical signature reuse, and law regressions;
5. this documentation/deletion ledger and any separately reviewed verification fixtures.

Each group needs its own focused tests and `git diff --check`. No unrelated `.codex`, core, module, fixture, or documentation changes belong in an SC-1 commit merely because they are dirty in the shared checkout.

## Final acceptance items still unproven

SC-1 must not be marked release-complete until all of these are resolved or explicitly waived by the owner:

- full `cargo fmt --all -- --check` on a suitably separated tree;
- full `RUSTFLAGS='' cargo test -p phalcom-semantic`;
- full `RUSTFLAGS='' cargo test --workspace`;
- CI-equivalent clippy with no unrelated lint residuals;
- final publishability walk over all declaration, alias, callable, and generic products;
- final cold/incremental equivalence run from a separated, reproducible tree;
- review of the remaining compatibility `resolve_type_parameter` and `UnannotatedDeclaration` uses against their exact allowed absence semantics;
- owner review of this ledger and selective staging/commit split.

The focused SC-1 suites provide strong implementation evidence, but the red broad gates mean this ledger deliberately makes no release-complete claim.
