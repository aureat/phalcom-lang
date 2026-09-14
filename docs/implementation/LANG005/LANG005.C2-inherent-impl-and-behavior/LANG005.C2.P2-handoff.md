# Handoff — LANG005.C2.P2 Variants-Only Enum Declarations and Closed Enum Behavior Migration to `impl`

## 1. Plan Status & Verification

```yaml
program: LANG005
checkpoint: LANG005.C2
plan: LANG005.C2.P2
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

## 2. Established Architecture & Invariants

1. **Variants-Only Enum AST (`INV-P2-01`)**:
   `EnumDef` has structure `pub variants: Vec<VariantDecl>`. `EnumMember`, `VariantBody`, `EnumBehaviorMember` (in AST), and `VariantDecl.body` have been eliminated from the AST. All behavior syntax inside enum bodies is rejected with `SyntaxErrorKind::EnumBehaviorUnsupported`.
2. **Canonical Impl Authority for Enums (`INV-P2-02`)**:
   Enum-root behavior is authored via `impl Enum { ... }` (supporting defaults and bodyless requirements), and case-specific behavior is authored via `impl Enum.Case { ... }` (`InherentImplTarget::ExactEnumCase(VariantId)`).
3. **No AST Fallback Bypass (`INV-P2-03`)**:
   `EnumBehaviorProduct` is derived exclusively from normalized `InherentImplContribution`s. All secondary fallback AST-walking passes in `session.rs` and `query.rs` have been removed.
4. **Compiler & Runtime Lowering**:
   Exact case methods emit `Bytecode::VariantMethod` and attach to hidden case behavior classes. Root methods attach to root behavior classes and dispatch across cases without modifying `ProductStorage` or runtime case memory layout.
5. **Universe & Shipping Source Separation (`INV-P2-04`)**:
   `Option`, `Result`, `Ordering`, and `Either` are separated into structural enums and root `impl` blocks.

## 3. Verification Summary

- **G1 (`phalcom-ast`)**: `cargo test -p phalcom-ast` — 226 passed, 0 failed, 1 ignored.
- **G2 (`phalcom-semantic::adts`)**: `cargo test -p phalcom-semantic --test semantic adts::declarations` — 14 passed, 0 failed.
- **G3 (`phalcom-semantic::impls`)**: `cargo test -p phalcom-semantic --test semantic impls` — 40 passed, 0 failed.
- **G4 (`phalcom-core::inherent_impl`)**: `cargo test -p phalcom-core --test core language::inherent_impl` — 12 passed, 0 failed.
- **G5 (`phalcom-core::algebraic_data`)**: `cargo test -p phalcom-core --test core language::algebraic_data` — 44 passed, 0 failed, 19 ignored.
- **Negative Searches**: 0 hits across all production code for `EnumMember`, `VariantBody`, and `parse_enum_behavior_member`.

## 4. Takeover Guidance for Next Plan (`LANG005.C2.P3`)

- **Objective**: Constrained & specialized inherent impl applicability (generic specializations, repeated type parameters, where-clause constraints on impl blocks).
- **Inherited Interfaces**:
  - `InherentImplTarget::Declaration(DeclarationId)`
  - `InherentImplTarget::ExactEnumCase(VariantId)`
  - `InherentImplContribution` and `InherentImplMemberContribution`
  - `build_effective_surface` and `build_enum_behavior`
- **Caution**: Keep `EnumDef` strictly structural; do not re-introduce behavior syntax into enum declarations.
