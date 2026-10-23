# 07 — Verification: SlotMap proof, hostile matrix, determinism (slices H, I)

## 1. SlotMap package artifact proof (slice H)

Fixture inputs:

- `packages/slotmap` (manifest: name `slotmap`, version `0.1.1`, entry
  `slotmap.main`);
- `packages/slotmap/tests/runtime` (project `slotmap-runtime-tests`, depends
  on `slotmap = { path = "../.." }`);
- a Universe artifact built once per test binary and reused through the
  existing `PHALCOM_TEST_FULL_TELOSC_PATH` convention.

Place the tests in `phalcom-core/tests/core/modules/` in a new file
`package_artifact.rs`, registered in `tests/core/mod.rs`.

| ID | Test | Pass condition |
|---|---|---|
| VER-H1 | Build `slotmap.telosc` over the loaded Universe artifact in a fresh process with `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`. | Build succeeds; `canonical_universe_formal_image_build_count() == 0` throughout; header identity is `Package{slotmap, 0.1.1}`; capabilities exclude `PORTABLE_EXECUTABLE`. |
| VER-H2 | Inspect the built module world. | Every local row's project is the publishing identity; `external_modules` is non-empty and every entry is `Builtin{universe,…}`; `external_projects == [Builtin universe]`; no `SourceArtifact` anywhere (EXP-P3). |
| VER-H3 | Compose Universe + `slotmap.telosc` through `compose_artifact_semantic_base` with a test resolver issuing the receipt (stable identity → built digest → `ResolvedProjectId` of the loaded `packages/slotmap` project). | Composition succeeds. `generation_for_module(slotmap.main)` is the package generation, and `ExternalModuleAuthoritySet::origin` assigns slotmap modules to origin 1. |
| VER-H4 | Semantic parity. Check `slotmap-runtime-tests` (a) against Universe artifact + slotmap **source** (today's path), and (b) against the composed base with slotmap **source denied**. | Same diagnostics (use `assert_semantic_diagnostics_parity`). For every slotmap export used by the tests, the declared types/signatures read from the session are equal. Under (b), a read of any slotmap source file fails the test. |
| VER-H5 | Negative parity. Introduce a type error in a scratch copy of the runtime tests that calls a slotmap API wrongly. | Both paths report the same diagnostic code and span. |
| VER-H6 | Execution (deferred). | Not required in this program; see `09` Q1. Record it as an `#[ignore]` test with that reference so the gap stays visible. |

**Source denial for a package (VER-H4b).** Add the source-denial hook as a
test-only knob mirroring `PHALCOM_DENY_SOURCE_BOOTSTRAP`. One option is an
env var `PHALCOM_DENY_PACKAGE_SOURCE=<namespace>`, checked where project
sources are read for parsing. The implementer chooses the exact hook site.
The test must prove that the hook fires: run once **without** the artifact
(source path) under denial and assert failure. That is the negative control
for the hook.

## 2. Hostile matrix (slice I)

Each case mutates a valid `slotmap.telosc` (or its decoded V5 world, then
re-encodes and re-digests where the case needs a well-formed container) and
asserts the specific error. Reuse the hostile-matrix helpers in
`phalcom-core/tests/core/modules/canonical_artifact.rs` where they fit.

| ID | Mutation | Expected rejection |
|---|---|---|
| LDH-1 | External index out of range in a `linked_reads` entry. | MW5 range error naming "external". |
| LDH-2 | Unsorted / duplicate `external_modules`. | MW5-V1. |
| LDH-3 | External module whose project is not in `external_projects`. | MW5-V3. |
| LDH-4 | External module in the package's own project. | MW5-V4. |
| LDH-5 | Unreferenced external table entry. | MW5-V5. |
| LDH-6 | External module that exists in no loaded artifact (valid path in an unknown Universe module). | LD-M2 step 3. |
| LDH-7 | Semantic owner referencing a non-existent Universe declaration. | LD-C2 / LD-C3. |
| LDH-8 | External binding read of a name Universe does not export. | LD-M4. |
| LDH-9 | External availability witness with no matching Universe proof. | LD-W1. |
| LDH-10 | Package module row smuggled into a second package artifact (two artifacts, same module). | Foreign-owner guard or `from_worlds` duplicate ownership. |
| LDH-11 | Non-empty `bootstrap_modules` in a package world. | MW5-V7. |
| LDH-12 | Receipt digest differs from the built digest. | `PackageAuthorityRegistrationError::ArtifactDigestMismatch`. |
| LDH-13 | Package artifact with `PORTABLE_EXECUTABLE`. | Existing capability rejection. |
| LDH-14 | Package A references package B's module, and B is absent from the composition. | LD-M2 step 3 (or planner error); no partial publication. |

Each hostile case also asserts **no partial publication**: after the failed
composition, the receiving `ProjectUniverse` and stores equal their
pre-attempt values (ARCH-I8).

## 3. Determinism (slice I)

| ID | Test |
|---|---|
| VER-D1 | Build `slotmap.telosc` twice in one process: byte-identical. |
| VER-D2 | Copy `packages/slotmap` to two temp directories with different absolute paths, then build both: byte-identical (ARCH-I7). This test is the detector for path leakage through `logical_uri` or `for_query` fingerprints. |
| VER-D3 | Edit only a comment in `slot-map.ph`: `source_digest` and the package identity fingerprint change (EXP-P2), and the module world's structural content is unchanged apart from identity. |

## 4. Regression gates

| ID | Gate |
|---|---|
| VER-R1 | Universe artifact bytes are identical before and after the whole program. Record the anchor digest in this folder (`baseline-digests.md`) during slice 0, and compare after every slice that touches export or the module world. |
| VER-R2 | Existing source-free SlotMap runtime proof `artifact_executable_bootstraps_and_runs_slotmap_without_universe_compilation` passes. |
| VER-R3 | `scripts/test.sh core` and the `phalcom-semantic`, `phalcom-modules` and `phalcom-compiler-artifact` test targets pass. Compare failures against the slice-0 baseline, not against zero. |
| VER-R4 | `cargo clippy --workspace` adds no new warnings in touched files; `cargo doc` has no missing-docs warnings for new public items. |

## 5. Negative controls (method rule)

Every new test that proves a rejection or a parity MUST be shown to fail
once without the code under test (revert the check locally, observe the
failure, restore). Note in the commit message which control was run. This
follows the project's lesson that green-but-vacuous tests shipped before.
