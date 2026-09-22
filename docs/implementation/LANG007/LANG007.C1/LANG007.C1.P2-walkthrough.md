---
id: LANG007.C1.P2
category: LANG
program: LANG007
checkpoint: LANG007.C1
kind: implementation-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
branch: main
start_revision: 33ee08f28988e8483259a0870f1bd247c0022547
final_revision: 33ee08f28988e8483259a0870f1bd247c0022547
prepared: 2026-09-22
---

# LANG007.C1.P2 — Walkthrough

## 1. Executive summary

`LANG007.C1.P2` converged Phalcom's decision registry, normative language specifications, compiler attribute handling, and source-like presentation on the accepted `intrinsic` keyword model. The obsolete source attribute `@intrinsic` was retired as a builtin attribute and removed from live source generation, while `intrinsic` was ratified as the canonical fourth declaration modifier and implementation kind (`ImplementationKind::Intrinsic`). All normative specifications were aligned with the completed P1 type-level domain and structural product calculus without accidental promotion of deferred generic or GAT features.

## 2. Implemented changes

### 2.1 Decision registry & TDR ratification

- Created `docs/decisions/accepted/0089-intrinsic-is-a-first-class-implementation-keyword.md` (TDR-0089).
  - Ratified `intrinsic` as a lexical source keyword and `ImplementationKind::Intrinsic`.
  - Retired `@intrinsic` as a builtin source attribute.
  - Amended historical source-form phrasing in TDR-0082 and TDR-0088 while preserving original decision text.
- Synchronized `docs/decisions/README.md`:
  - Updated accepted decision count from 82 to 89.
  - Added table rows for TDR-0083 through TDR-0089.

### 2.2 Production source and presentation cleanup

- `phalcom-ast/src/ast.rs`:
  - Removed `BuiltinAttr::Intrinsic` variant.
  - Removed `BuiltinAttr::Intrinsic` from `BuiltinAttr::ALL`.
  - Removed name mapping and parsing for `"intrinsic"` as a builtin attribute.
- `phalcom-core/src/compiler/attributes.rs`:
  - Removed `BuiltinAttr::Intrinsic` expander registration from `AttributeRegistry`.
  - Removed `BuiltinAttr::Intrinsic` from class target handling.
- `phalcom-semantic/src/core_surface/presentation.rs`:
  - Removed obsolete `@intrinsic(...)` attribute emission from `render_virtual_source`.
  - Rendered source implementation modifiers directly from `ImplementationKind` (`Phalcom` -> `""`, `Native` -> `"native "`, `Foreign` -> `"foreign "`, `Intrinsic` -> `"intrinsic "`).
  - Disambiguated internal `NativeIntrinsicId` in markdown presentation (`intrinsic-id:BoolNot`).

### 2.3 Normative specification convergence

- `docs/specs/source-declarations.md`:
  - Added `intrinsic` to reserved declaration keywords and `ImplementationKind::Intrinsic`.
  - Specified four-way mutually exclusive implementation modifier model.
  - Documented retirement of `@intrinsic` and preservation of `@primitive`.
- `docs/specs/type-members.md`:
  - Updated syntax examples to `intrinsic type ...`.
  - Defined intrinsic type members as pure compiler/semantic normalization with zero VM dispatch.
  - Aligned canonical integer indexing identity: `T[I]` and `T::At<I>` denote one Subscript member identity, with `At` as the named presentation alias.
  - Confirmed unparameterized trait-associated boundary (no GATs).
- `docs/specs/universe.md`:
  - Aligned Universe metadata and intrinsic type member ownership.
- `docs/specs/type-level-domains.md` & `docs/specs/structural-products.md`:
  - Verified domain separation (`TypePack`, `LabeledTypePack`, `RecordRow`, `Type`).
  - Confirmed closed structural calculus, negative indexing, symbolic open-row mapping, and zero-product distinctions.

## 3. Deviations from plan

None. All steps and verification gates executed in accordance with the patch-grade plan.

## 4. Consultations & amendments

- **AMEND-002:** Recorded adoption of TDR-0089 and retirement of `@intrinsic` in `LANG007.C1-CHECKPOINT.md`.

## 5. Verification evidence

### 5.1 Executed tests

1. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-ast --test integration intrinsic`
   - `parser::intrinsic_attribute_is_not_builtin`: `ok`
   - `parser::parses_intrinsic_declarations`: `ok`
   - `parser::parses_intrinsic_fields_and_methods`: `ok`
2. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance`
   - `test_native_enum_merge_preserves_declarations`: `ok`
   - `test_presentation_ir_and_virtual_source`: `ok`
   - `test_native_surface_conformance`: `ok`
3. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::type_members`
   - 6 tests: `ok`
4. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-ast` (`PASS`)
5. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic` (`PASS`)
6. `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core` (`PASS`)
7. `git diff --check` (`PASS`)

### 5.2 Authority and deletion searches

- Confirmed zero live `@intrinsic` usage in normative specifications or production compiler handling.
- Confirmed keyword preservation for `TokenKind::Intrinsic` and `ImplementationKind::Intrinsic`.
- Confirmed preservation of internal `NativeIntrinsicId` metadata for primitive operations (`BoolAnd`, `BoolOr`, `BoolNot`).
- Confirmed representation independence of normative specs (no raw Rust AST/store IDs as language law).

### 5.3 Intentionally deferred verification

- Workspace-wide tests (`cargo test --workspace --all-targets`)
- Workspace clippy (`cargo clippy --workspace --all-targets`)
- Full language corpus & LSP integration tests
- Release certification

## 6. Residual / baseline issues

Two pre-existing baseline trait dependency test failures (`BASELINE-001`, `BASELINE-002`) remain recorded in the C1 checkpoint and reproduce independently of LANG007.

## 7. Verification classification

`status: COMPLETE`  
`completion: IMPLEMENTED`  
`verification: FOCUSED_TESTED`  

## 8. Next action

Proceed to `LANG007.C1.P3` (Architecture Baseline and Acceptance Matrix).
