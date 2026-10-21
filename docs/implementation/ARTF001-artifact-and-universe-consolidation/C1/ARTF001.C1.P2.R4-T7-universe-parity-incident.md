# IMPLEMENTATION INCIDENT

## 1. Identity

- Program: `ARTF001`
- Checkpoint: `ARTF001.C1`
- Plan: `ARTF001.C1.P2.R4`
- Task/Gate: `R4.T7.2` positive compiler parity; related source-image diagnostic behavior
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom`
- Branch: `feat/artifact-universe-consolidation`
- Starting revision: `4f65526d4b0cb5026ce02d53434f5b03d0167173`
- Current revision: `4f65526d4b0cb5026ce02d53434f5b03d0167173`

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
- The positive parity test fails: 12 comparator differences. The source program has 41 Universe modules while the artifact program has 8. Reported mismatches include linked initialization order/reference graph, runtime dependency sets, and constructor-initialization products.
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

The active test now compares the ordinary source compiler entry point with the artifact-backed compiler for both real SlotMap entries. The assertion remains strict. No production compiler behavior has been changed for this incident.

## 9. Evidence

- The test source selects four modules for `packages/slotmap` and eight for `packages/slotmap/tests/runtime` on both sides.
- Both compilers return `CompiledProgram` values; failure occurs in `assert_compiled_program_parity`.
- The comparator reports 12 field-level differences; the source side includes 41 Universe modules and artifact side 8.
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
