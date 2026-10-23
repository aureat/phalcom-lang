# 07 — Verification

Test file for the proofs: `phalcom-core/tests/core/modules/executable_product.rs`
(new; register it in `phalcom-core/tests/core/mod.rs`). Codec-only tests
live in `phalcom-compiler-artifact/tests/`.

Every proof that claims "source-free" runs in a fresh child process with
`PHALCOM_DENY_SOURCE_BOOTSTRAP=1`, using the parent/child pattern of
`artifact_executable_bootstraps_and_runs_slotmap_without_universe_compilation`
and `fresh_process_artifact_executable_runner`.

## 1. SlotMap proofs

| ID | What | Pass condition |
|---|---|---|
| VER-X1 | Build `slotmap.telosc` with an executable section over a saved `universe.telosc`, twice, in two fresh source-denied processes. | Both builds succeed; bytes are identical; `canonical_universe_formal_image_build_count() == 0` throughout; header has `PORTABLE_EXECUTABLE`; section schema is 2; `dependencies` is exactly `[Universe]` with the loaded Universe's digest; `imports` is non-empty and contains no SlotMap-owned identity. |
| VER-X2 | Build product `bin:slotmap=slotmap.main` and run it from `universe.telosc` and `slotmap.telosc` in a fresh source-denied process. | Run succeeds. Standard output equals the source path's output for the same entry (`compile_user_entry(EntrySelection::Project(packages/slotmap))` then `run_compiled`). |
| VER-X3 | During the VER-X2 run, count emission and source access. | `artifact_run_emission_count() == 0`; formal-image build count `== 0`; no package source read (assert through the package source-denial hook ARTC slice H places; if H has not placed one, this slice places it). |
| VER-X4 | Three-artifact chain: build `slotmap-runtime-tests.telosc` (`packages/slotmap/tests/runtime`) over `slotmap.telosc` over `universe.telosc`; build product for `slotmap_tests.main`; run it. | Run succeeds with the same output as the existing source-compiled proof (VER-R2). The product's `initialization` equals the package-module subsequence of the source path's `CompiledProgram::initialization_order`, and does **not** contain `slotmap.main` (D-XL-9). |
| VER-X5 | Product determinism. | VER-X2's product built twice in fresh processes is byte-identical. |
| VER-X6 | `ARTC VER-H6` closes. | The `#[ignore]` placeholder that ARTC slice H adds is replaced by VER-X2 and VER-X4; the ignore is removed. |

## 2. Regression gates

| ID | Gate |
|---|---|
| VER-XR1 | Universe artifact bytes are unchanged by this program (ARTC VER-R1 baseline digests). |
| VER-XR2 | ARTC VER-R2 (`artifact_executable_bootstraps_and_runs_slotmap_without_universe_compilation`) still passes. |
| VER-XR3 | Compiler-only package artifacts still build, load and serve as a compilation base (ARTC VER-H1…H5). |
| VER-XR4 | `materialize_program`, refactored by XM-M3, still passes the full `core` test target. |

## 3. Negative controls

Each proof above records one negative control in its commit message, as
ARTC `07` requires: break the thing the test claims to check and show the
test fails. Required controls:

- VER-X3: temporarily call the emitter once during run; the counter test
  must fail.
- VER-X4: put `slotmap.main` into the product's `initialization`; XR-1 must
  reject the product, or, with XR-1 disabled, the output must differ.
- VER-X1: drop one import entry from the built section; the builder
  self-check (XS-B3) must fail.

## 4. Hostile matrix

Each case is a crafted artifact or product, loaded through the production
path, that MUST be rejected with the named error family before any heap
allocation for the offending artifact.

| ID | Case | Expected |
|---|---|---|
| LXH-1 | Package artifact with a schema 1 executable section | XS-V1 |
| LXH-2 | Universe artifact with a schema 2 section | XS-V1 |
| LXH-3 | Unsorted or duplicated import vector | XS-V2 |
| LXH-4 | External reference missing from `imports` | XS-V6 |
| LXH-5 | Unused import entry | XS-V6 |
| LXH-6 | Module row owned by another project | XS-V4 |
| LXH-7 | Executable module set differs from the module world's owned rows | XS-V5 |
| LXH-8 | Linked reads differ from the module world's | XS-V5 |
| LXH-9 | Method recipe whose holder is a Universe class | XS-V8 |
| LXH-10 | Class recipe for a Universe declaration | XS-V8 |
| LXH-11 | Non-empty `eager_modules` or `native_runtime_roles` | XS-V9 / XS-V2 |
| LXH-12 | Dependency digest differs from the supplied chain artifact | XM-V2 |
| LXH-13 | Native ABI digest differs from Universe's | XM-V3 |
| LXH-14 | Import resolves only in a later chain artifact | XM-V4 |
| LXH-15 | Import resolves nowhere | XM-V4 |
| LXH-16 | Compiler-only package in a product chain | XM-V6 |
| LXH-17 | Product digest tampered | XP-V1 |
| LXH-18 | Product chain entry not supplied, or supplied file has another digest | XP-V2 |
| LXH-19 | Product entry not owned by root | XP-V3 |
| LXH-20 | Product `initialization` out of order or containing a Universe module | XR-1 |
| LXH-21 | Product schema 2 (reserved) | XP-W7 |
| LXH-22 | Profile other than `Debug` | XS-V11 |

ARTC's LDH-13 ("Package artifact with `PORTABLE_EXECUTABLE` → existing
capability rejection") is rewritten by this program: a package artifact
with `PORTABLE_EXECUTABLE` and a schema 2 section is accepted by the
executable-aware loader; LDH-13 becomes LXH-1 (schema 1 in a package). A
compiler-only consumer (one that loads a package only as a compilation
base) MUST still accept such an artifact and ignore its executable section.

## 5. Determinism

| ID | Check |
|---|---|
| VER-XD1 | VER-X1 and VER-X5 byte identity across fresh processes. |
| VER-XD2 | Building from two different checkout paths of the same source produces identical bytes (catches host paths in source maps, XS-W6). |
| VER-XD3 | Import tables and dependency lists do not depend on hash-map iteration order. `std` hash maps are seeded per process, so VER-XD1's two fresh processes already perturb iteration order; in addition, assert sortedness (XS-V2) on the built bytes. |
