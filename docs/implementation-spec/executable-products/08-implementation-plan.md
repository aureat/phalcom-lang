# 08 — Implementation plan

Slices run strictly in order. Each ends at a green gate and a commit. A
slice that cannot reach its gate stops with an incident note in this folder
(`incident-<slug>.md`: trigger, observed code, rule involved, options).

Branch: `feat/artifact-universe-consolidation`, worktree `.worktrees/artifact`.
Stage narrow paths only. Never `git add -A`. Never create or switch branches.

## Prerequisites (ARTC)

| ARTC item | Needed by | State at anchor |
|---|---|---|
| F3 `build_package_telosc` green (today: `module 0 has empty path`) | X3 onward | RED |
| G external-aware loading and composition | X4 onward | NOT STARTED |
| H SlotMap package proof, including the package source-denial hook | X5 onward | NOT STARTED |

X0, X1 and X2 do not depend on ARTC beyond the anchor and MAY run first.

## Slice X0 — Census

Read-only. Produce `census.md` in this folder answering, with greppable
anchors:

1. Every heap object kind `capture_object` can reach from a package
   module's closure, and whether each is owned, a foreign reference, or
   foreign and unsupported (XS-C3). Run the existing source path for
   `packages/slotmap` and `packages/slotmap/tests/runtime` with a temporary
   probe; do not commit the probe.
2. Whether lowering installs any package-owned method or conformance into a
   class owned by Universe (XS-V8, `09` Q2).
3. Whether `compile_program_module_closure` reads any *initialized* value of
   another package's module (XS-C2).
4. Whether `canonical_type` digest lookups for package code ever match more
   than one pool (`09` Q3).
5. The exact `has_p7_*` family list at the time of the census (XS-W3).

Gate: every question answered or turned into an incident. A BLOCKING-IF in
`09` that fires stops the program here.

## Slice X1 — Wire

Files: `phalcom-compiler-artifact/src/executable/` (new `linked.rs` or
extend `image.rs`), `limits.rs`, `digest.rs`, new `src/product.rs`.

1. `ExecutableImageV2`, `ExecutableDependencyV2`, `ExecutableImportsV2`,
   `ImportedBindingV2`, encode/decode, structural validation XS-V1…V3,
   limits. **Commit.**
2. Schema-dispatching `ExecutableSection::decode` (XS-W8). Round-trip and
   rejection tests, including schema 3. **Commit.**
3. Product wire `ExecutableProductV1` with `DOMAIN_PRODUCT` (XP-W1…W8),
   round-trip and tamper tests. **Commit.**

Gate: codec tests green; `cargo test -p phalcom-compiler-artifact` green;
Universe digests unchanged (VER-XR1).

## Slice X2 — Capture subject refactor

Files: `phalcom-core/src/artifact/capture.rs`, `src/vm/bootstrap.rs`.

Introduce the capture subject (XS-C1) and port the Universe path to it.
No behaviour change.

Gate: VER-XR1 byte identity; VER-XR2 green. **Commit.**

## Slice X3 — Source-free package materialization primitives

Files: `phalcom-core/src/modules/materialize.rs`, `src/artifact/materialize.rs`,
`phalcom-modules` (runtime project registration).

1. Factor `materialize_program` phases 1–2 into a source-free helper
   (XM-M3). **Commit.**
2. Runtime project registration from receipts (XM-M2). **Commit.**
3. Generalize `ExecutableMaterializer` over schema (XM-M6); metadata-driven
   callable binding. **Commit.**

Gate: full `core` test target green (VER-XR4); VER-XR2 green.

## Slice X4 — Chain loading and validation

Files: `phalcom-core/src/artifact/validate.rs` (or new `validate_linked.rs`),
`src/modules/` (new `LoadedArtifactChain`).

1. `validate_linked_executable_image` (XS-V4…V12). **Commit.**
2. `LoadedArtifactChain` with XM-V1…V6 and `new_vm` (XM-M0…M8), exercised
   first with a chain of Universe only plus an empty-import fixture.
   **Commit.**

Gate: hostile cases LXH-1…LXH-16 green, each with its negative control.

## Slice X5 — Package capture and builder

Files: `phalcom-core/src/modules/package_artifact.rs`,
`src/artifact/capture.rs`.

1. Capture VM and package capture (XS-C2…C6). **Commit.**
2. `build_package_telosc` output request and extended self-check (XS-B1…B3).
   **Commit.**

Gate: VER-X1 green.

## Slice X6 — Products and run

Files: `phalcom-core/src/modules/` (product build/load/run API),
test file `phalcom-core/tests/core/modules/executable_product.rs`.

1. Product construction (XP-C1…C4) and validation (XP-V1…V4). **Commit.**
2. Run path (XR-1…XR-5) with the emission counter. **Commit.**
3. Proofs VER-X2…X6 and LXH-17…22. **Commit:**
   `test(core): prove source-free SlotMap execution from package artifacts`.

Gate: VER-X1…X6, VER-XR1…XR4, VER-XD1…XD3, LXH-1…22 green.

## Slice X7 — Minimal `telos` driver

Files: `phalcom-core/Cargo.toml` (`[[bin]] name = "telos"`,
`path = "bin/telos/main.rs"`), `phalcom-core/bin/telos/`.

Subcommands XT-C1…C3, XT-R1…R4, XT-I1, with the exit classes of XT-R3.
`phalcom` stays as it is. Add a CLI smoke test that runs the README
commands end to end in a temporary directory.

Gate: CLI smoke test green. Update ARTF001 `CHECKPOINT.md` with a short
section linking this folder and its final state.

## Effort and risk (informative)

| Slice | Risk | Why |
|---|---|---|
| X0 | Low | Read-only, but its answers can stop the program. |
| X1 | Low | New wire next to an existing pattern. |
| X2 | Medium | Universe byte identity must hold through a refactor of hot capture code. |
| X3 | **High** | `materialize_program` is shared by every source run; the runtime project model has never existed without a source root. |
| X4 | Medium–high | Import resolution touches every stable reference family. |
| X5 | High | Capture outside the Universe bootstrap is new; foreign-object classification is where surprises will be. |
| X6 | Medium | Parity against the source path, and the initialization-order rule. |
| X7 | Low | Thin driver over X6's API. |
