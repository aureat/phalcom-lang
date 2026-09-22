# LANG007.C2.P2 — Walkthrough: Variadic Generic Binding and Structural Type Expansion

## Overview

Plan `LANG007.C2.P2` implemented the source-level variadic generic binding, application assembly, and structural pack expansion layer on top of the canonical pack substrate established in P1.

## Scope Completed

1. **AST and Parser Support:**
   - Unified `TypePackExpansionSyntax` across `GenericArgumentSyntax::Expansion`, `TypeTupleItem::Expansion`, and `TypeCallableItem::Expansion`.
   - Parsed `*`, `**`, and `***` operators in type application, Tuple, and callable-domain positions while rejecting `...` as type pack syntax.
   - Preserved explicit empty specialization `<>` as an explicit zero-argument application node.

2. **Generic Binder Semantics and Legality:**
   - Lowered generic parameter modes to `GenericParameterMode::{Single, PositionalPack, LabeledPack, CompletePack}`.
   - Mapped binder modes to canonical kinds (`TypePack`, `LabeledTypePack`, `CompleteTypePack`) and enforced invariance.
   - Enforced deterministic lane ownership: at most one positional owner, at most one labeled owner, complete capture is terminal, no trailing fixed binders.

3. **Staged Publication & Binder-Aligned Generic Application:**
   - Implemented `resolve_generic_binder_header` to establish canonical parameter identities, kinds, modes, and variances during nominal predeclaration (resolving SC-04).
   - Application lowering consumes canonical application contracts via `scoped_generic_application_contract` and `generic_application_contract`.
   - Produced binder-aligned canonical `Applied` arguments and synthesized empty packs for saturated `<>` specializations.

4. **Scoped Pack Algebra & Aliases:**
   - Added scoped counterparts: `ScopedTypeData::{TypePack, LabeledTypePack, CompleteTypePack, TypePackConcat, LabeledTypePackConcat, TupleFromPacks, CallableFromPacks, PackProjection}`.
   - Preserved parameter modes on `TypeLambdaData` across alias lowering and partial beta reduction.

5. **Tuple and Callable-Domain Pack Expansion:**
   - Lowered Tuple pack expansions through `TypeStore::tuple_from_packs`, collapsing concrete lanes into `TypeData::Tuple` and `TypeData::Unit`.
   - Added canonical `TypeData::CallableFromPacks` and `ScopedTypeData::CallableFromPacks` for finite symbolic callable domains.
   - Concrete callable expansions collapse to standard `CallableType`.

6. **Checked Rewriting & Stable Metadata:**
   - Extended `TypeSubstitution::try_apply` and `materialize_type_inner` to recurse through `CallableFromPacks` and all C2 terms, preserving duplicate label invalidation.
   - Added `TypeNode::CallableFromPacks` in `phalcom-type-meta` with topological validation and structural fingerprinting.
   - Updated runtime metadata display in `phalcom-core`.

## Verification Evidence

- `cargo fmt --all -- --check`: PASS
- `RUSTFLAGS='' cargo check -p phalcom-ast`: PASS
- `RUSTFLAGS='' cargo check -p phalcom-type-meta`: PASS
- `RUSTFLAGS='' cargo check -p phalcom-semantic`: PASS
- `RUSTFLAGS='' cargo check -p phalcom-core`: PASS
- `cargo test -p phalcom-ast`: PASS (53 unit tests, 258 integration tests, 12 trait syntax tests)
- `cargo test -p phalcom-type-meta`: PASS (6 schema compat tests)
- `cargo test -p phalcom-semantic --test semantic callable_domain`: PASS (3 tests)
- `cargo test -p phalcom-semantic --test semantic capabilities::aliases`: PASS (7 tests)
- `cargo test -p phalcom-semantic --test semantic foundations::type_annotations`: PASS (45 tests)
- `cargo test -p phalcom-semantic --test semantic foundations::substitution`: PASS (4 tests)
- `cargo test -p phalcom-semantic --test semantic foundations::type_model`: PASS (17 tests)
- `cargo test -p phalcom-semantic --test semantic incremental::fingerprints`: PASS (34 tests)
- `cargo test -p phalcom-semantic --test semantic integration::metadata`: PASS (16 tests)
- `cargo test -p phalcom-semantic --test semantic integration::imported_resolution`: PASS (10 tests)
- `cargo test -p phalcom-semantic --test semantic incremental::type_store_revisions`: PASS (15 tests)
