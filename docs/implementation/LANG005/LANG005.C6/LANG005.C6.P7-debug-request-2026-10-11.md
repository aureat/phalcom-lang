# LANG005.C6.P7 Debug Request — Missing Callable Analyses on Package Compilation

Date: 2026-10-11
Status: immediate cause confirmed; underlying semantic update failure still unknown
Scope: read-only diagnosis and solution proposal; no implementation is authorized by this packet.

## 1. Identity

- Program: `LANG005`
- Checkpoint: `LANG005.C6`
- Plan/task: `LANG005.C6.P7.T4` — exported generic Family recipe transport; current failure is upstream of runtime transport
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `main`
- Starting revision: `6e5287af46e7bde7a13fb2cecd37a10f42d6ee80` (`docs: clarify numeric and collection specs`)
- Working tree: dirty before this packet; 12 tracked files modified and two unrelated untracked documents. No existing work was changed for this packet.
- Delivery state: no commit or push reported.

## 2. Escalation trigger

The current package-compile trace contradicts the positive-path assumption needed to validate the P7 T4 runtime change: `ProgramCompiler` receives a semantic snapshot with no callable analyses, so it cannot lower the expected call recipe or Family application. This crosses the semantic publication/compiler-consumer boundary and requires identifying the owning layer before further VM edits.

## 3. Required invariants

From the P7 plan and the C6 checkpoint:

- The semantic snapshot remains the canonical source for call specialization and Family-application facts; compiler lowering must consume those facts rather than reconstruct them.
- A call-site recipe stays attached to the Family application and is validated only after live Family selection reaches the captured target method.
- Preserve canonical callable identity, live receiver lookup, argument order, caller evidence, existing Family errors, and ordinary `Function.call(***)` behavior for closure calls.
- Do not add runtime proof search, a numerics-specific inference path, a duplicate inference source, a `Dynamic` fallback, or fabricated conformance evidence.
- B2 remains unapplied until a positive metadata path is established.

## 4. Expected architecture

For this package path, source analysis should publish a `CallableAnalysis` for the entry module body containing the `stats.mean(values.iterator)` call facts. `ProgramCompiler` should consume those facts to lower both:

1. a generic call-environment recipe whose canonical selector is `mean(_)`; and
2. the matching behavioral Family application at the same call site.

Only then can the runtime path exercise post-selection validation and materialization for the selected `Descriptive.mean(_)` method.

## 5. Observed behavior

### Supplied current run

Command:

```sh
RUSTFLAGS='' PHALCOM_CALL_TRACE=1 cargo test -p phalcom-core --test core audit_probes::math::public_statistics_family_applies_its_recipe_after_live_member_selection -- --nocapture
```

The trace prints `analyses=0` for every linked Universe and project module, including `proj#2:main`, and reports:

```text
metadata export callables=1421
```

The test then panics at `phalcom-core/tests/core/audit_probes/math.rs:1002` with:

```text
generic recipe for the public mean call
```

The test fails while looking up the recipe in the compiled entry module. It has not reached `validate_specialized_call`, Family execution, or the test's later negative control.

### User-reported investigation

- Before the exporter wiring correction, `MetadataExporter` received no callable signature table and emitted `semantic_metadata.callables = 0`.
- That wiring was corrected; the semantic callable-signature table now contains 1,421 entries and the supplied run reports the same export count.
- Despite the populated signature table, `SemanticSnapshot.callable_analyses` is empty on this package path.
- Fix A and B1 timing/VM plumbing are reported implemented; `cargo check -p phalcom-core` passes for the current work.
- B1's positive metadata path remains unresolved. B2 has not been applied because its fail-closed behavior would not establish the missing positive path.

These statements are user-supplied findings, not independently reproduced by this packet.

## 6. Minimal reproduction and expected result

The existing test is at `phalcom-core/tests/core/audit_probes/math.rs:986`. Its package source imports `numerics.statistics`, builds a `List<Float>` with `1.0` and `3.0`, and evaluates:

```phalcom
let mean = stats.mean(values.iterator)
```

Expected before runtime: the compiled entry module has a `mean(_)` entry in `lowering.call_environment_recipes` and a behavioral Family application at the same source range. Actual: the recipe lookup fails because the lowering maps are empty.

The command above was supplied by the user with its output. It was not rerun during preparation of this packet.

## 7. Relevant code path

Inspect this path end to end, verifying each live call and data product:

```text
ProgramAnalyzer::discover_and_analyze
  -> analyze_workspace(SemanticWorkspaceInput)
  -> SemanticWorkspaceSession update and callable body analysis
  -> callable_analyses in the published SemanticSnapshot
  -> ProgramCompiler's loop over snapshot.callable_analyses
  -> semantic_lowering call recipe / Family application products
  -> entry_module.lowering maps inspected by the test
```

Useful live locations:

- Package discovery and snapshot creation: `phalcom-core/src/modules/compile.rs`, `ProgramAnalyzer::discover_and_analyze`.
- Lowering's dependence on snapshot analyses and metadata export wiring: `phalcom-core/src/modules/compile.rs`, `ProgramCompiler`.
- Analysis construction, filtering, and publication: `phalcom-semantic/src/session.rs`, especially the update path ending in `SemanticSnapshot::new_with_callable_analyses`.
- Callable signature export: `phalcom-semantic/src/metadata/export.rs`, `MetadataExporter`.
- Runtime binding/materialization changes already present in the working tree: `phalcom-core/src/modules/materialize.rs` and VM dispatch files.
- Reproducer and failing assertion: `phalcom-core/tests/core/audit_probes/math.rs:986-1004`.

## 8. Scoped current changes and preservation boundary

The working tree already contains edits in compiler, runtime, metadata, and semantic files. In particular, `compile.rs`, `materialize.rs`, `metadata/export.rs`, and `session.rs` are modified. The user identifies Fix A and B1 as in-progress work; the supplied trace instrumentation is present in the dirty tree. Preserve all current modifications and unrelated untracked documents. Do not reset, clean, broadly format, or stage the checkout.

Do not infer from `cargo check -p phalcom-core` that call recipes or Family applications were successfully published: the supplied failing test directly shows they were absent in this run.

## 9. Evidence classification

### Directly visible in the supplied run

- Every printed module has zero callable analyses; the total is also printed as zero.
- Metadata export reports 1,421 callables.
- The selected test fails at recipe lookup before runtime dispatch.

### Directly inspected in the current tree

- The test first looks for a `mean(_)` call-environment recipe, then looks for a Family application at the same range.
- The package analyzer passes the linked program and discovered parsed sources to `analyze_workspace`.
- Compiler lowering iterates `analyzed.semantic.callable_analyses` to derive call-site products.
- The P7 plan is `IN_PROGRESS / PARTIAL / UNVERIFIED`; no P7 handoff or walkthrough is present.

### Prior implementation record that must be reconciled

The P7 checkpoint's 2026-10-11 execution note says an earlier positive route through `stats.mean(values.iterator)` reached the Family method and computed a valid value, while its negative control failed. The supplied current trace stops before recipe lowering. Determine whether the earlier note describes another intermediate tree/run, or whether the current uncommitted package/semantic changes regressed analysis publication. Do not treat the earlier note as proof of current behavior.

## 10. Causal hypotheses to test, not conclusions

1. The package path may create callable signatures from interface/declaration products while failing to schedule or retain body-analysis queries for the source modules.
2. A module/source identity or ownership mismatch may prevent analyses from being keyed or selected under the same `ModuleId`s used by lowering.
3. A semantic-work scheduling, blocked/budget/error fallback, or final-retention condition may publish signatures while dropping analyses.
4. The recent package-source seeding/discovery or uncommitted compiler/semantic edits may have changed the work set relative to the earlier P7 positive run.

The debugger should distinguish these possibilities using the live tree and existing trace output; do not patch the VM or assume a specific hypothesis.

## 11. Attempts already made

- **Exporter wiring (Fix A):** passed the semantic callable-signature table to `MetadataExporter`. Prediction: exported callable metadata would no longer be empty. Observed: 1,421 callables are now reported, but call analyses remain zero and the recipe is absent.
- **B1 timing/VM plumbing:** user reports the plumbing is implemented and `cargo check -p phalcom-core` passes. Prediction: once the semantic recipe reaches lowering, the runtime will apply it after live Family selection. Observed: this does not establish the positive metadata path; the current test stops before runtime.
- **B2:** deliberately not applied. The positive recipe path is a prerequisite for judging the fail-closed behavior.

## 12. Why a local runtime correction is insufficient

The current failure occurs before a runtime recipe exists. A dispatch or post-initialization change cannot produce canonical expression analysis that was not published to the compiler. The owning correction may be in semantic body analysis, package input/work scheduling, snapshot publication/filtering, or their interface; identify that cause before proposing a patch. Any change to semantic authority or the snapshot/compiler contract must be called out explicitly.

## 13. Debugging agent request

Perform a read-only, goal-backward diagnosis. Use the supplied command output and current modified tree as evidence; preserve the user's working tree. Do not edit source, change the plan/checkpoint, apply B2, commit, or push. Do not rerun the supplied test merely to reproduce its already-provided output.

Return a concise root-cause report that answers:

1. At what exact stage do callable analyses become empty or disappear, while callable signatures remain populated?
2. Which live condition, path, identity, scheduling decision, or failure branch causes that result for the `compile_numerics_package` path?
3. Is the result a regression from the earlier P7 positive run? Reconcile the earlier checkpoint note with the current failure using revision and dirty-tree evidence.
4. What is the smallest correct owner-layer solution? Name the concrete file/symbol/data flow and explain why it restores canonical analysis publication without duplicating semantic facts or adding a consumer workaround.
5. What focused discriminator would prove the correction, and what observable values should it assert (`callable_analyses`, recipe count/selector, Family application site, and runtime negative control)? Do not run it in this diagnostic-only task.
6. Does the proposed correction cross any mandatory consultation boundary in the repository's escalation protocol?

Separate observed facts, causal conclusion, and any remaining uncertainty. If the live evidence is insufficient to name one exact fix, state the narrowest missing discriminator rather than guessing.

## 14. Read-only diagnosis result

The debugging agent confirmed the immediate cause and the limit of the available evidence:

- `ProgramAnalyzer::discover_and_analyze` calls `phalcom_semantic::analyze_workspace`.
- `analyze_workspace` creates a new `SemanticWorkspaceSession`, calls `session.update(input)`, then returns the snapshot without an error channel.
- On the first update, `SemanticWorkspaceSession::update_with_delta` catches any `QueryOutcome` error with `unwrap_or_else`. With no last-known-good snapshot, it builds a fallback from the baseline callable-signature table and an empty `callable_analyses` map.
- Therefore the `1,421` callable metadata records can come from the fallback's baseline signatures. They do not prove the package's semantic body analysis succeeded. The fallback explains the empty call recipes and Family applications.
- A successful first full-workspace update selects current modules for semantic work and inserts a `<main>` analysis for each source module. The all-zero analysis trace therefore points to fallback publication rather than a successful update that simply omitted the entry body.

The exact `QueryOutcome` that triggers the fallback is discarded by the convenience API and cannot be recovered from the supplied trace. The underlying semantic query failure is not yet identified, so the final behavioral fix remains undetermined.

### Smallest next correction and discriminator

For package compilation, call the existing fallible `SemanticWorkspaceSession::update_with_budget_and_cancel` path and map its `QueryOutcome` into the existing `ProgramCompileError::SemanticUpdate`. `phalcom-core/src/modules/compile_session.rs` already follows the same fail-visible policy for persistent compilation. This exposes the real semantic failure without changing semantic ownership or adding a compiler-side reconstruction.

The next discriminator is the exact result of that update: successful current-snapshot publication or the concrete `QueryOutcome`. The `1,421` export count is not that discriminator. After the surfaced cause is addressed, verify the entry `<main>` analysis, the `mean(_)` recipe, the same-range behavioral Family application, successful execution returning `2.0`, and failure of the altered-dispatch negative control. This discriminator was not run as part of this diagnosis.

### Prior positive-run reconciliation

The P7 checkpoint's earlier positive-path note is from commit `3ad129dce9a920362235ad071e8dbd168ba7f0b4` (2026-10-11 14:51:55 +03). Current `HEAD` is `6e5287af46e7bde7a13fb2cecd37a10f42d6ee80` (14:58:47 +03). The dirty `compile.rs` diff adds source seeding for Universe collection/scalar modules to package discovery; that change was absent at the earlier revision. The evidence establishes that the current and earlier runs were on different trees, but does not establish that the source seeding caused the semantic update error.

### Escalation boundary

Surfacing the discarded error through the existing fallible API and existing error variant does not itself require architectural consultation. If fixing the surfaced outcome requires changing semantic scheduling, publication authority, or the compiler/semantic contract, stop and consult under the repository escalation protocol before implementation.
