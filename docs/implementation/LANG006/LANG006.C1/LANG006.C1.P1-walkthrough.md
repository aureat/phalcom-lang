---
id: LANG006.C1.P1
kind: implementation-plan-walkthrough
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
base_revision: bf4e12ba
working_tree: uncommitted scoped implementation changes
---

# LANG006.C1.P1 — Implementation Walkthrough

## Outcome

The C1 declaration-modifier/access-domain vertical is implemented across the
AST, semantic surfaces, compiler/runtime dispatch, native metadata, Universe
source, reflection-facing presentation, LSP, syntax grammar, current
specification authority, and decision registry. The implementation is
The implementation is focused-tested. Verification is `FOCUSED_TESTED` because
the active owner checks and workspace compilation pass; two unrelated broader
tests remain red in the shared checkout and the repository-wide formatter has
pre-existing drift.

## Architecture implemented

- `phalcom_common::modifier::Visibility` is the shared visibility product;
  `ImplementationKind` is the shared Phalcom/native/foreign product.
- Source modifiers normalize once. Selector and field spelling no longer
  derives visibility or implementation origin.
- Runtime native descriptors carry Universe declaration origin, so internal
  native calls use project identity rather than a blanket boolean privilege.
- Direct, reflective, dynamic, cached, semantic, and native paths retain the
  same access relation. Compiler-generated protocol privilege remains a narrow
  compiler seam and is not ordinary member authorization.
- Universe provenance (`is_universe` / Universe project identity) remains
  independent of ordinary `Internal` access.

## Main implementation surfaces

- `phalcom-ast`: retired authored implementation token/AST variants removed;
  `_$` is rejected as invalid source, ordinary fields/selectors remain ordinary,
  standardized dunders remain available, and legacy modifier attributes no
  longer parse as built-in semantic attributes.
- `phalcom-core`: structured visibility drives compilation and access checks;
  native source members are declaration anchors; native installation preserves
  Universe origin; selector-prefix authority, legacy attribute expanders, and
  old compiler error variants were removed.
- `phalcom-semantic`: native source/conformance/presentation products consume
  `ImplementationKind` and structured visibility.
- `phalcom-native-decl` and generated native surface: ordinary selector names
  and exact visibility metadata are preserved; the generated surface contains
  334 primitive records.
- `phalcom-lsp`: seven declaration keywords are tokenized as keywords,
  completion uses canonical access context, and hover presents structured
  visibility plus implementation origin.
- Universe source: all canonical legacy modifier attributes and `_$` selectors
  were migrated to explicit modifiers and ordinary names; standardized dunder
  protocol members remain dunders.

## Migration/protocol exceptions

The remaining `__name__`-style Universe members are standardized reflection or
package protocol dunders, not general implementation fields. Remaining
`compiler_internal`/`emit_invoke(..., internal)` and Universe provenance checks
are explicitly compiler/bootstrap protocol seams; they do not authorize
ordinary source `Internal` members. No new dunder semantics were introduced.

## Specification and decision records

- Added authoritative [`docs/specs/source-declarations.md`](../../../specs/source-declarations.md).
- Added accepted [`TDR-0082`](../../../decisions/accepted/0082-declaration-modifiers-and-identity-based-access-domains.md), superseding the relevant TDR-0076 implementation-namespace clauses.
- Updated the specification index/current mirror pointer, declaration grammar,
  class/allocator examples, and TextMate grammar. Historical attribute drafts
  remain historical and now point at the current authority.

## Verification evidence

| CV | Evidence | Result |
| --- | --- | --- |
| CV-01 | `cargo test -p phalcom-ast --lib` | PASS, 52 tests |
| CV-02 | `parser::tests::lang006_declaration_modifiers_and_ast_storage` | PASS |
| CV-03 | `parser::tests::lang006_modifier_negative_diagnostics` | PASS |
| CV-04 | AST modifier storage, Universe field census, explicit source migration | PASS focused |
| CV-05 | Runtime/semantic access implementation and workspace compile | IMPLEMENTED; no standalone module-only discriminator run |
| CV-06 | Runtime/semantic access implementation and workspace compile | IMPLEMENTED; no standalone package-only discriminator run |
| CV-07 | Bootstrap/native origin checks and structured runtime authorization | PASS focused; no standalone cross-project fixture run |
| CV-08 | `compiler_declaration_dispatch::{private_member...,protected_member...,reflection_cannot_bypass_private_visibility}` | PASS previously recorded |
| CV-09 | Reflection/dynamic dispatch paths compile and private reflection regression | PASS focused |
| CV-10 | `modules_universe::boot_01_bootstrap_measurement_separates_catalog_closure_and_execution` | PASS |
| CV-11 | `native_surface_contracts::system_native_metadata_matches_language_contracts`; reflection census | PASS |
| CV-12 | Exact Universe bootstrap test above | PASS |
| CV-13 | `compiler_declaration_dispatch::primitive_and_intrinsic_metadata_are_universe_only`; semantic native conformance | PASS focused |
| CV-14 | AST foreign body/target negative diagnostics | PASS |
| CV-15 | Reflection census plus standardized dunder source/runtime preservation | PASS focused |
| CV-16 | `compiler_declaration_dispatch::ordinary_source_rejects_retired_selector_namespace` | PASS (`Invalid token`) |
| CV-17 | `cargo test -p phalcom-lsp --lib` | PASS, 72 tests |
| CV-18 | Incremental semantic surface remains compiled; dedicated cache parity not rerun | DEFERRED |
| CV-19 | Canonical source/native/reflection census tests | PASS |
| CV-20 | Affected crates and workspace all-target compile | PASS; standalone ownership fixture deferred |

Additional completed checks:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-common -p phalcom-ast -p phalcom-modules -p phalcom-semantic -p phalcom-native-meta -p phalcom-native-surface -p phalcom-core -p phalcom-lsp
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --workspace --all-targets
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-native-decl --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::native_conformance -- --nocapture
git diff --check
```

## Baseline/deferred issues

- `phalcom-semantic` incremental caller identity assertion failed in an
  unrelated query test; the source has no relevant modifier/native change.
- `phalcom-core` VM walk ordering test failed because `fold(initial,using)` is
  not understood by the existing runtime path; it is outside the migration
  surface.
- `cargo fmt --all -- --check` reports repository-wide pre-existing formatting
  drift, including untouched files. No broad formatting rewrite was applied.
- Full package test suites and workspace tests were not run; focused evidence
  plus workspace all-target compilation was the proportionate gate for this
  dirty shared checkout.
