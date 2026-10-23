# 08 — Implementation plan

Slices run strictly in order. Each slice ends at a green gate and a commit.
A slice that cannot reach its gate stops with an incident note in this folder
(`incident-<slug>.md`: trigger, observed code, rule involved, options). The
implementer does not improvise around a rule.

Branch: `feat/artifact-universe-consolidation`, worktree `.worktrees/artifact`.
Stage narrow paths only. Never `git add -A`. Never create or switch branches.

## Slice 0 — Baseline and test-target recovery

Scope: make A–D verifiable and record the regression baseline.

1. `cargo test -p phalcom-semantic --lib --no-run`. At the anchor this fails
   with 3 errors in `session/publication_tests.rs`, `static_value/artifact.rs`
   and `types/artifact.rs` (reported by the previous agent; not yet
   reproduced). Reproduce them. Check whether they predate `48435bb71` by
   compiling the test target at `141e01442` in a throwaway worktree
   (`git worktree add /tmp/... 141e01442`, then remove it). Record the result
   in `baseline-failures.md`.
2. Fix the compile errors with the smallest change that preserves each
   test's intent.
3. Run the `projection.rs` unit tests and the existing Universe artifact
   tests. Record pass/fail in `baseline-failures.md`.
4. Build the Universe compiler artifact (`build_universe_telosc`) and the
   full Universe artifact. Record their digests in `baseline-digests.md` for
   VER-R1.

Gate: semantic lib-test target compiles; A–D unit tests pass; baselines
recorded.
Commit: `test(semantic): restore lib-test target and record artifact baselines`.

## Slice E — `ModuleWorldV5`

Spec: `03`. Files: `phalcom-compiler-artifact/src/module_world.rs`,
`container.rs` (schema accept), then the consumers in MW5-M1.

Steps:

1. Types, wire, validate, fingerprint, lift (MW5-T*, MW5-V*, MW5-F*, MW5-L1)
   with crate-local tests. **Commit.**
2. Schema-dispatching decode (MW5-L2); container accepts 5. **Commit.**
3. Migrate consumers to `&ModuleWorldV5` (MW5-M1). External arms return
   errors per MW5-M2 until G implements them. The Universe builder projects
   V4, lifts to V5 for export, and writes V4 bytes. **Commit per crate**, each
   compiling.

Gate: MW5 tests green; VER-R1 byte-identical; VER-R2 green.

## Slice F1 — Export identity

Spec: `04`. Files: `phalcom-semantic/src/metadata/stable_identity.rs`,
`phalcom-semantic/src/artifact/{projection,export,roots,operational}.rs`,
`artifact/export/*.rs`, `types/artifact.rs`, `static_value/artifact.rs`.

Steps:

1. `StableIdentityContext::publishing` + EXP-T1. **Commit.**
2. Thread the context through `ArtifactSemanticProjection` and the
   type/static export contexts. Switch every conversion to `*_with_context`
   (EXP-C2). This is mechanical. Run the `rg` check from EXP-C2 at the end.
   **Commit.**
3. `package_identity_fingerprint`, package `source_digest` (EXP-P2, EXP-P4) +
   EXP-T2. **Commit.**
4. EXP-T3 fixture test. **Commit.**

Gate: EXP-T1…T4 green; VER-R1 byte-identical.

## Slice F2 — Package compiler image

Spec: `05` §1–2. Files: new `phalcom-core/src/modules/package_image.rs`
(module rustdoc required), `phalcom-semantic/src/canonical_image.rs`
(scoped freeze).

Steps:

1. Scoped `freeze` (PB-I3); Universe passes `whole_image()`; VER-R1. **Commit.**
2. Capture `PackageCompilerImage` from a session over the Universe artifact
   (PB-B1…B3, PB-I1, PB-I2, PB-I4, PB-I5). Test: capture slotmap; assert
   scope = linked key set = the 4 slotmap modules; zero Universe bootstrap.
   **Commit.**

Gate: the capture test is green under `PHALCOM_DENY_SOURCE_BOOTSTRAP=1`.

## Slice F3 — Package module world and builder

Spec: `05` §3–4. Files: `phalcom-modules/src/compiler_artifact.rs`
(`project_package_module_world_v5`), new
`phalcom-core/src/modules/package_artifact.rs` (`build_package_telosc`).

Steps:

1. `project_package_module_world_v5` (PB-M*) with a unit test on the
   captured slotmap image. **Commit.**
2. `build_package_telosc` without the load self-check (PB-H1…H4, PB-H6).
   Test: VER-H1, VER-H2. **Commit.**

Gate: VER-H1, VER-H2 green; VER-R1 byte-identical.

## Slice G — External-aware loading

Spec: `06`. Files: `phalcom-modules/src/compiler_artifact.rs` (loader,
staging), `phalcom-semantic/src/artifact/identity.rs` (catalog),
`phalcom-core/src/modules/telosc_compiler_wire.rs` (composition, decode).

Steps:

1. V5 loader + external table resolution (LD-M*). **Commit.**
2. Cross-artifact witnesses (LD-W*). **Commit.**
3. Catalog external existence (LD-C*). **Commit.**
4. Prefix authority in composition (LD-K*). **Commit.**
5. Enable the builder load self-check (PB-H5). **Commit.**

Gate: VER-H3 green; the builder self-check is green.

## Slice H — SlotMap proof

Spec: `07` §1. File: `phalcom-core/tests/core/modules/package_artifact.rs`.

Gate: VER-H1…H5 green, with negative controls recorded; VER-H6 present as
`#[ignore]`.
Commit: `test(core): prove source-free SlotMap package artifact`.

## Slice I — Hostile and determinism matrix

Spec: `07` §2–3. Gate: LDH-1…14 and VER-D1…D3 green, each with its negative
control. VER-R1…R4 green. Update `CHECKPOINT.md` (ARTF001) with a short
section linking this folder and its final state.

## Effort and risk (informative)

| Slice | Risk | Why |
|---|---|---|
| 0 | Low–medium | Compile errors of unknown origin. |
| E | Medium | Broad mechanical type migration; the Universe byte-identity gate keeps it honest. |
| F1 | Medium | About 260 call-site rewrites; mechanical, but touches the hottest export code. |
| F2 | **High** | The capture path for a non-Universe image does not exist yet; scoped freeze touches census classification. |
| F3 | Medium | New projection; the two-pass external indexing must be deterministic. |
| G | Medium–high | Cross-artifact witness matching and prefix authority thread through several transaction layers. |
| H/I | Medium | Package source-denial hook placement. |
