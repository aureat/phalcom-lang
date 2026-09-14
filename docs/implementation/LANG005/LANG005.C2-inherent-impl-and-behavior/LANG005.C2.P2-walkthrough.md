# Walkthrough — LANG005.C2.P2 Variants-Only Enum Declarations and Closed Enum Behavior Migration to `impl`

## 1. Overview & Objectives

Plan `LANG005.C2.P2` completed the migration of enum behavior from legacy in-enum member syntax into first-class inherent `impl` blocks (`impl Target { ... }` and `impl Target.Case { ... }`), enforcing that enum declarations are strictly structural variant declarations.

Key goals achieved:
- **Variants-Only Enum AST**: `EnumDef` now holds only `pub variants: Vec<VariantDecl>`. Legacy `EnumMember`, `VariantBody`, and `VariantDecl.body` structures have been completely removed from production AST.
- **Fail-Fast Syntax Rejection**: Any attempt to declare behavior (`fn`, methods, getters, setters, `@class`, `@static`, or index operators) directly inside an `enum` or `variant` block triggers `SyntaxErrorKind::EnumBehaviorUnsupported` (`syntax.enum.behavior_unsupported`).
- **Canonical Impl Resolution for Enums**: Root enum behavior is authored via `impl Enum { ... }` (supporting bodyful defaults and bodyless requirements), and case-specific behavior is authored via `impl Enum.Case { ... }`.
- **Pure Inherent Impl Behavioral Derivation**: Semantic analysis (`build_enum_behavior`, `session.rs`, `query.rs`) constructs `EnumBehaviorProduct` exclusively from normalized `InherentImplContribution`s. All secondary fallback AST-walking passes have been removed.
- **Compiler Lowering & VM Dispatch**: Exact case methods emit `Bytecode::VariantMethod` and attach to hidden case behavior classes. Root methods attach to root behavior classes and dispatch across cases without modifying `ProductStorage` or runtime case memory layout.
- **Standard Library & Fixture Migration**: `Option`, `Result`, `Ordering`, and `Either` were separated into structural enum declarations and corresponding `impl` blocks.

---

## 2. Changes Summary

### AST & Parser (`phalcom-ast`)
- `phalcom-ast/src/ast.rs`:
  - Updated `EnumDef` to `pub variants: Vec<VariantDecl>`.
  - Removed `EnumMember`, `VariantBody`, and `VariantDecl.body`.
- `phalcom-ast/src/error.rs`:
  - Added `SyntaxErrorKind::EnumBehaviorUnsupported` (`syntax.enum.behavior_unsupported`).
- `phalcom-ast/src/parser.rs`:
  - `parse_enum_body` parses only variant declarations.
  - Rejects `@class`, `@static`, methods, getters, setters, index operators with `EnumBehaviorUnsupported`.
  - Rejects braced variant bodies with `EnumBehaviorUnsupported`.
- `phalcom-ast/src/selector.rs`:
  - Replaced enum-specific behavior selector helpers with unified `selector_from_behavior_member`.

### Semantic Layer (`phalcom-semantic`)
- `phalcom-semantic/src/impls.rs`:
  - Added `InherentImplTarget::ExactEnumCase(VariantId)` target resolution.
  - Validated case existence, constructor signature compatibility, and type parameter coverage.
  - Derived `is_requirement` for bodyless members in enum-root impls.
- `phalcom-semantic/src/checker/enum_behavior.rs`:
  - Removed legacy Section 2 AST extraction.
  - `build_enum_behavior` now constructs root defaults, root requirements, and case implementations exclusively from `InherentImplContribution`s.
- `phalcom-semantic/src/session.rs` & `phalcom-semantic/src/db/query.rs`:
  - Removed all AST fallback scans in callable resolution and matching.
  - Canonicalized enum callable definitions under `QueryKey::CallableDefinition`.
- `phalcom-semantic/src/core_surface/source.rs`, `source_index/builder.rs`, `source_index/occurrence.rs`, `semantic_shard.rs`:
  - Migrated declaration extraction, occurrence indexing, and structure shard fingerprints to inspect `enum_def.variants` and index behavior from `Statement::Impl`.

### Core & Compiler Layer (`phalcom-core`)
- `phalcom-core/src/modules/semantic_lowering.rs`:
  - Extended `InherentImplLoweringTarget` to support `ExactEnumCase(VariantId)`.
  - Emits `InherentImplLoweringSpec` for enum roots and exact cases.
- `phalcom-core/src/compiler/lib/impl_decl.rs` & `enum_decl.rs`:
  - Lowering compiles exact case behavior into `Bytecode::VariantMethod`.
  - Standalone enum compilation synthesizes specs over `enum_def.variants`.
- `phalcom-core/src/native/source.rs`:
  - `index_enum` registers Option variants; `index_impl` registers native methods.

### Fixtures & Universe
- `phalcom-core/core/universe/src/option/option.ph`: Separated into `enum Option<T>` and `impl<T> Option<T>`.
- `phalcom-core/core/universe/src/errors/result.ph`: Separated into `enum Result<T, E>` and `impl<T, E> Result<T, E>`.
- `phalcom-core/core/universe/src/object/ordering.ph`: Separated into `enum Ordering` and `impl Ordering`.
- `examples/either.ph` & `tests/core/typing_integration/sources/either.ph`: Separated into `enum Either<L, R>` and `impl<L, R> Either<L, R>`.

---

## 3. Verification Matrix (CV-01 through CV-30)

| ID | Description | Status | Evidence |
|---|---|---|---|
| CV-01 | Variants-only `EnumDef` AST | PASS | `phalcom-ast/src/ast.rs`, `phalcom-ast/tests/enum_syntax.rs` |
| CV-02 | Rejection of methods in enum body | PASS | `enum_with_nested_method_is_rejected` in `enum_syntax.rs` |
| CV-03 | Rejection of nested variant case body | PASS | `enum_with_nested_variant_body_is_rejected` in `enum_syntax.rs` |
| CV-04 | Rejection of class-side methods in enum | PASS | `enum_with_class_method_is_rejected` in `enum_syntax.rs` |
| CV-05 | Rejection of index operator in enum | PASS | `enum_with_index_operator_is_rejected` in `enum_syntax.rs` |
| CV-06 | Exact enum case impl target resolution | PASS | `test_exact_enum_case_target_success` in `semantic::impls::targets` |
| CV-07 | Rejection of non-existent enum case target | PASS | `test_specialized_repeated_target_rejected` in `semantic::impls::targets` |
| CV-08 | Exact case impl type parameter bijection | PASS | `test_generic_covering_target_success_and_bijection_permutation` in `semantic::impls::targets` |
| CV-09 | Root enum impl defaults and requirements | PASS | `test_enum_root_inherent_impl_body` in `semantic::impls::bodies` |
| CV-10 | Enum requirement verification on cases | PASS | `adt_req_01_02_04_requirements_report_satisfied_missing_and_incompatible_cases` |
| CV-11 | Exact case override of root defaults | PASS | `exact_enum_case_impl_dispatches_with_defaults_overrides_and_payloads` |
| CV-12 | Removal of legacy AST behavior extraction | PASS | Zero AST fallback in `enum_behavior.rs` and `query.rs` |
| CV-13 | Source indexing of `impl Target.Case` | PASS | `semantic::impls::queries` tests |
| CV-14 | Shard structural fingerprint ignores impl bodies | PASS | `semantic_shard.rs` unit tests & queries |
| CV-15 | Shard header fingerprint tracks enum variants | PASS | `declaration_header_fingerprint` in `semantic_shard.rs` |
| CV-16 | Native source indexer separates enum and impl | PASS | `native/source.rs` `index_enum` + `index_impl` |
| CV-17 | Semantic lowering of `ExactEnumCase` | PASS | `semantic_lowering.rs` `ExactEnumCase` lowering |
| CV-18 | Compiler emits `VariantMethod` bytecode | PASS | `exact_case_impl_emits_variant_method_bytecode` in `language::inherent_impl` |
| CV-19 | Runtime case dispatch with payload access | PASS | `exact_enum_case_impl_dispatches_with_defaults_overrides_and_payloads` |
| CV-20 | Root impl method dispatch across all variants | PASS | `enum_root_impl_method_is_available_on_each_case` |
| CV-21 | Inherent impl preserves case memory layout | PASS | `impl_members_do_not_add_storage_or_change_product_layouts` |
| CV-22 | Option universe migration to `impl` | PASS | `Option` in `option.ph`, `adt_vert_07_core_option_native_representation_execution` |
| CV-23 | Result universe migration to `impl` | PASS | `Result` in `result.ph`, `adt_vert_08_core_result_error_variant_execution` |
| CV-24 | Ordering universe migration to `impl` | PASS | `Ordering` in `ordering.ph`, `adt_vert_09_core_ordering_four_state_execution` |
| CV-25 | Either fixture migration to `impl` | PASS | `either.ph`, `adt_vert_01_generic_result_crosses_constructor_match_and_runtime` |
| CV-26 | AST integration test suite pass | PASS | 226/226 passed in `cargo test -p phalcom-ast` |
| CV-27 | Semantic ADT & Impl test suites pass | PASS | 14/14 `adts::declarations`, 40/40 `semantic::impls` passed |
| CV-28 | Core inherent impl test suite pass | PASS | 12/12 passed in `language::inherent_impl` |
| CV-29 | Core algebraic data test suite pass | PASS | 44/44 active passed in `language::algebraic_data` |
| CV-30 | Zero production legacy syntax hits | PASS | 0 hits for `EnumMember`, `VariantBody`, `parse_enum_behavior_member` |

---

## 4. Verification Gates (G1–G5) Summary

1. **G1 (`phalcom-ast`)**:
   `cargo test -p phalcom-ast` -> **226 passed, 0 failed, 1 ignored**.
2. **G2 (`phalcom-semantic::adts`)**:
   `cargo test -p phalcom-semantic --test semantic adts::declarations` -> **14 passed, 0 failed**.
3. **G3 (`phalcom-semantic::impls`)**:
   `cargo test -p phalcom-semantic --test semantic impls` -> **40 passed, 0 failed**.
4. **G4 (`phalcom-core::inherent_impl`)**:
   `cargo test -p phalcom-core --test core language::inherent_impl` -> **12 passed, 0 failed**.
5. **G5 (`phalcom-core::algebraic_data`)**:
   `cargo test -p phalcom-core --test core language::algebraic_data` -> **44 passed, 0 failed, 19 ignored**.

## 5. Conclusion & Lifecycle State

Plan `LANG005.C2.P2` is **COMPLETE** (`IMPLEMENTED` + `FOCUSED_TESTED`).
All tasks T1–T7 are completed, with zero legacy enum behavior syntax in production and 100% focused test suite pass across AST, semantic, and core runtime layers.
