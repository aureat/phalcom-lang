# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `ARTF001`
- Checkpoint: `ARTF001.C1`
- Plan: `ARTF001.C1.P2.R4`
- Task/Gate: `R4.T7.2` positive compiler parity; related source-image diagnostic behavior
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `feat/artifact-universe-consolidation`
- Starting revision: `4f65526d4b0cb5026ce02d53434f5b03d0167173`
- Current revision at packet creation: `4f65526d4b0cb5026ce02d53434f5b03d0167173`

## 2. Escalation trigger

Mandatory trigger 11: two plausible fixes have different architectural consequences. Trigger 16 also applies because the source-built formal image's semantic diagnostics, ordinary source compilation, and artifact compilation currently produce different observable paths.

## 3. Required invariant

- Preserve P2's single current-main semantic authority and current canonical identity.
- Preserve `P2-INV-07`: default `ProgramCompiler` behavior remains on the canonical source path.
- Do not weaken the P2.R4 positive parity requirement or suppress semantic diagnostics to make parity pass.
- Keep P2 compiler/runtime compatibility separate from P3 source-free VM execution.

## 4. Expected architecture

P2.R4.T7 requires the same real package entry to compile through the ordinary source-backed compiler and the loaded compiler artifact, then satisfy the existing compiled-program parity contract. The source-built formal image is used for the earlier explicit image path; artifact loading reconstructs semantic/query authority without carrying source diagnostics. P2 does not own source-free VM bootstrap.

## 5. Observed behavior

- `ProgramCompiler::compile_entry_selection` compiles both `packages/slotmap` and `packages/slotmap/tests/runtime` from source.
- `LoadedUniverseArtifact::compile_user_entry` compiles both entries from the loaded compiler-only artifact.
- The positive parity test fails: 12 comparator differences. The source-side linked graph has 41 Universe modules while the artifact-side linked graph has 10. Reported mismatches include linked initialization order and graph/module sets, runtime dependency sets, and constructor-initialization products.
- The earlier source side using `compile_entry_selection_with_image` failed before comparison because the source-built `CanonicalUniverseFormalImage` snapshot contains semantic diagnostics; `compile_analyzed` rejects snapshots with errors.
- The ordinary source CLI completes SlotMap runtime tests. The artifact-compiled SlotMap test program also runs on ordinary `VM::run_compiled`.
- Semantic diagnostics are not serialized in the compiler artifact. Their absence after artifact import is not evidence that the source diagnostics were resolved.

## 6. Minimal reproduction

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core source_backed_and_artifact_backed_compiler_program_parity -- --nocapture
```

The test builds and loads the current compiler artifact, compiles the real SlotMap package and its runtime-test package through both entry points, and fails the positive comparison with the differences listed above. The command completed with a failing test after approximately 171 seconds.

## 7. Relevant code path

```text
ProgramCompiler::compile_entry_selection
  -> source-backed package/Universe resolution
  -> source CompiledProgram

LoadedUniverseArtifact::compile_user_entry
  -> artifact-backed canonical semantic image
  -> artifact CompiledProgram

compile_entry_selection_with_image
  -> CanonicalUniverseFormalImage source snapshot
  -> compile_analyzed rejects semantic errors
```

Key files:

- `phalcom-core/tests/core/modules/canonical_artifact.rs`
- `phalcom-core/src/modules/canonical_artifact.rs`
- `phalcom-core/src/compile.rs`
- `phalcom-semantic/src/canonical_image.rs`

## 8. Scoped implementation diff

The positive test compares the ordinary source compiler entry point with the artifact-backed compiler for both real SlotMap entries. The assertion remains strict. The negative test source route was changed to the ordinary source entry point in `15917229f`; its focused rerun is pending. No production compiler behavior has been changed for this incident.

## 9. Evidence

- The test source selects four modules for `packages/slotmap` and eight for `packages/slotmap/tests/runtime` on both sides.
- Both compilers return `CompiledProgram` values; failure occurs in `assert_compiled_program_parity`.
- The exact rerun at `7cff51b4b` reports these 12 comparator differences:
  1. linked initialization order;
  2. reference graph nodes;
  3. semantic graph nodes;
  4. runtime graph nodes;
  5. linked module set (`only_in_src` contains the 41-module Universe source set; `only_in_art` is empty);
  6. constructor-initialization products for package `<root>`;
  7. runtime dependencies for `dense_slot_map`;
  8. constructor-initialization products for `dense_slot_map`;
  9. runtime dependencies for `main`;
  10. constructor-initialization products for `main`;
  11. runtime dependencies for `slot_map`;
  12. constructor-initialization products for `slot_map`.
- The artifact-side runtime dependencies add Universe modules. `dense_slot_map` adds `collections.iterator_laws`, `collections.list`, `errors.error`, `object.object`, `option.option`, and `scalar.number`; `main` adds `collections.tuple`, `concurrency.fiber`, `errors.error`, `option.option`, and `scalar.string`; `slot_map` adds `collections.iterator_laws`, `collections.list`, `errors.error`, `object.object`, `option.option`, `scalar.bool`, `scalar.number`, and `scalar.string`.
- The live code confirms two path inputs without establishing which output is wrong: source analysis seeds source-authored Universe support and finalizes with `canonical_universe_program().availability_after_bootstrap()`; artifact analysis discovers through the imported external module world and supplies `AvailabilityContract::default()`.
- Focused artifact-runtime execution passes on ordinary `VM::run_compiled`; source CLI execution also passes.
- The source-image compiler path returns semantic errors from multiple Universe modules, including unresolved/unsaturated type annotations, generic inference failures, call-shape or argument mismatches, constructor-super requirements, and unsupported enum/associated-type forms. Their status against the accepted spec and their relationship to the parity mismatch have not been classified.

## 10. Implementer's causal hypothesis

The mismatch may indicate that artifact closure omits source-side implicit Universe dependencies or compiler products. The diagnostic-bearing formal image path may also include a wider or differently validated source set than the ordinary compiler path.

## 11. Alternative hypotheses

- Ordinary source bootstrap may include extra implicit modules that artifact bootstrap is intentionally not meant to compile, requiring the parity contract to compare the relevant semantic projection rather than all module/link products.
- One or more comparator fields may represent path-specific but semantically irrelevant bootstrap detail; changing that oracle would still require an explicit architectural ruling.
- The source diagnostics may be accepted implementation debt, actual Universe source/checker defects, or artifacts of the image-building mode. Current evidence does not distinguish these.

## 12. Attempts already made

- First attempt: use `compile_entry_selection_with_image` for the source side. Prediction: compare both paths against the same source-built canonical image. Result: compilation stopped on canonical Universe semantic diagnostics.
- Second attempt: use the ordinary source entry point, which is P2's default source-backed compiler route. Prediction: compare the real default compiler with artifact-backed compilation without the prebuilt image's diagnostic set. Result: both compile, but strict parity fails with 12 differences. No third architecture-changing attempt was made.

## 13. Why local authority is insufficient

Choosing whether to expand artifact closure, narrow/normalize the parity projection, or change source-image diagnostic handling affects semantic authority, dependency closure, compiler/runtime contracts, and P3's assumptions. These options cannot be selected as a mechanical test fix.

## 14. Decision requested

Determine the intended positive-parity boundary and source-image diagnostic disposition for P2.R4: should artifact loading reproduce the source compiler's 41-module linked/dependency/constructor products, or should the parity contract exclude source-bootstrap-only products while retaining semantic equivalence for the SlotMap program? Also determine whether diagnostics retained by `CanonicalUniverseFormalImage` are expected source debt, a checker/compiler defect, or a formal-image construction error, and identify the narrowest evidence needed to classify them.

## 15. Advisory decision — 2026-10-21

### Root cause and confidence

The parity cause is not established. Two path differences are confirmed:

- Ordinary source compilation creates a fresh `SemanticWorkspaceSession` and obtains availability from the source-backed canonical Universe program.
- Artifact compilation creates a session from the imported image and uses `AvailabilityContract::default()` to avoid source bootstrap.
- `CanonicalUniverseFormalImage::build` analyzes the Universe source set and retains its semantic diagnostics. Canonical bootstrap uses `compile_analyzed_for_canonical_bootstrap` to project that source surface despite diagnostics, while ordinary compilation with an explicit image uses `compile_analyzed_with_options`, which rejects snapshots with errors.

The additional source-side modules may be required entry closure or broader source bootstrap. Current evidence does not decide which side produces the incorrect products. Confidence is high about the observed failures and path differences, low about the root cause.

### Architecture verdict

**PLAN NEEDS CLARIFICATION before parity-boundary narrowing or a shared availability-contract change.** Keep positive parity and the strict comparator. Do not expand artifact closure or change source availability until the exact 12 field differences identify which products are missing or extra. If a defect can be fixed while preserving the comparator and phase boundary, no plan amendment is required.

### Required evidence and next action

1. Produce a field-by-field report for all 12 differences, including module identities and affected products.
2. Trace those products through source imports, artifact closure, and `AvailabilityContract`.
3. For diagnostic parity, compile the same invalid input through the ordinary source and artifact-backed entry points and compare normalized entry-local diagnostics. The explicit source-image path is not a useful control while it rejects existing Universe diagnostics first.
4. Catalog each Universe diagnostic by module, code, range, and triggering construct; compare with accepted specification rules, existing tests, and the relevant predecessor revision before classifying it as inherited debt or a defect.

Do not suppress diagnostics, filter diagnostic-bearing modules, drop parity fields, or treat runtime smoke as parity evidence. Independent T6/T8 work may continue; T7 parity changes remain stopped until the difference report resolves the architecture choice.

## 16. Latest continuation evidence — 2026-10-21

### Entry-local negative diagnostic parity

The focused test
`negative_parity_constructor_resolution_failure` completed after 168.10s and
failed its entry-local diagnostic comparison. The ordinary source compiler
reported one diagnostic at `<root>`, range `93..106`:

```text
ConstructorSuperInvalid: no direct-parent constructor matches this `super.new(...)` call
```

Artifact-backed compilation reported that same diagnostic and an additional
diagnostic at `<root>`, range `31..39`:

```text
IncompatibleOverride: override `new()` is incompatible with the inherited contract from `Class`
```

This confirms a second semantic-product mismatch for the same invalid input.
The output does not establish which route owns the incorrect result. No
diagnostic was suppressed or normalized away.

### Positive parity causal trace

The live source path confirms these inputs:

- `ProgramAnalyzer::discover_and_analyze` seeds source-authored Universe
  support modules from `collections`, `scalar`, `text`, and `option.unit`.
- Its finalizer obtains availability from
  `canonical_universe_program().availability_after_bootstrap()`.
- `discover_image_closure` uses the imported image's external module world and
  passes `AvailabilityContract::default()` so artifact compilation does not
  consult the source bootstrap singleton.
- `AvailabilityContract::satisfies` requires an exact target and product with
  a proof available by the requirement's timing. The source and artifact paths
  therefore do not currently provide equivalent availability inputs.

This explains why the paths can create different runtime edges. It does not
prove that the artifact-side edges are wrong, and it does not by itself explain
the differing constructor-initialization products.

The supplied follow-up analysis describes the source/artifact linked Universe
counts as 42/0. The earlier captured parity output prints only set differences
(`only_in_src` and `only_in_art`), which are insufficient to derive both full
set sizes. A fresh strict parity run is in progress with the comparator now
printing the exact source and artifact constructor rows for each differing
user module. Use that output to reconcile the module counts, record exact
`ModuleId` dispositions, and trace each constructor product before changing
compiler behavior or the parity boundary.

### Current disposition

- Positive parity remains failed; comparator and compiler behavior remain
  unchanged.
- Entry-local negative diagnostic parity is also failed by one extra
  artifact-side `IncompatibleOverride` diagnostic.
- Canonical Universe source diagnostics remain a separate unclassified census.
- Ignored `Result` values in `phalcom-semantic/src/impls.rs` are a separate
  semantic-correctness audit item, not an established cause of either parity
  failure.
- Do not change availability, normalize constructor products, or narrow the
  parity comparator until the structural dump is reviewed and the architecture
  decision is recorded.
