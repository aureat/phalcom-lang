---
id: LANG007.C4.P5
kind: implementation-walkthrough
title: "Product/Sum Reflection and Compiler-Derived Structural Capabilities — Walkthrough"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
recorded: 2026-10-12
branch: main
---

# LANG007.C4.P5 — Implementation Walkthrough

## Summary

`LANG007.C4.P5` implements the normative compiler-derived structural capabilities (`Product`, `OrderedProduct`, `UnorderedProduct`, `Sum`), universal member projection, and domain-native reflection for products and sums without fabricating fake `TypeId`s or synthetic `ImplId`s.

The implementation preserves the normative semantic layer separation:
- Structural classification authority remains `phalcom-semantic/src/structural.rs`.
- Canonical structural reflection definitions live in `phalcom-core/core/universe/src/reflection/structural.ph` and are re-exported in `reflection/package.ph`.
- Generic conformance proving is handled entirely by `evaluate_conformance_goal` in `generic_evidence.rs` via `CompilerDerivedConformanceEvidence`.
- Domain-neutral projection representations (`MemberProjection`, `MemberProjectionSubject`, `MemberDeclarationIdentity`, `MemberProjectionArgument`, `StaticMemberProjectionOutcome`) are housed in `phalcom-semantic/src/member_projection.rs`.
- Universal evaluation of structural reflection members is centralized in `phalcom-semantic/src/structural_reflection.rs` through `StructuralReflectionProjector`.

---

## Tasks Completed

### Task 1: Extend TypeMemberDomain with Value-Type Domain and Adapter
- Added `TypeMemberDomain::ValueType(TypeId)` in `phalcom-semantic/src/types/type_member.rs`.
- Implemented total conversions to and from `StaticValueDomain`:
  - `StaticValueDomain::ValueType(ty)` <-> `TypeMemberDomain::ValueType(ty)`
  - Preserved standard kinds (`KindId::TYPE`, `KindId::RECORD_ROW`, etc.).
- Verified by unit tests in `tests/semantic/foundations/static_domains.rs`.

### Task 2: Domain-Neutral Member Projection Representation
- Implemented `MemberProjection`, `MemberProjectionSubject`, `MemberDeclarationIdentity`, `MemberProjectionArgument`, and `StaticMemberProjectionOutcome` in `phalcom-semantic/src/member_projection.rs`.
- `StaticMemberProjectionOutcome` supports:
  - `ReducedValue(StaticValueRef)`: directly returns reduced static value store refs.
  - `Stuck(MemberProjection)`: preserves unresolved / stuck projections without fabricating fake types.
  - `Invalid(Box<str>)`: malformed domain or argument errors.

### Task 3: Trait Associated Type Requirement Result Domains
- Extended AST `TraitMember` in `phalcom-ast/src/ast.rs` and parser in `phalcom-ast/src/parser.rs` to parse optional `: Domain` on `type Shape: ProductShape`.
- Updated `TraitAssociatedTypeRequirement` in `phalcom-semantic/src/traits.rs` to retain `result_domain: StaticValueDomain`.
- Added test coverage in `phalcom-ast/tests/trait_syntax.rs` and `phalcom-semantic/tests/semantic/capabilities/traits.rs`.

### Task 4: Author Canonical Structural Declarations in Universe
- Authored `phalcom-core/core/universe/src/reflection/structural.ph`:
  - Sealed traits: `Product`, `OrderedProduct`, `UnorderedProduct`, `Sum`.
  - Intrinsic data declarations: `ProductKind`, `ProductShape`, `OrderedTail<Rest>`, `OrderedShape`, `Presence`, `FieldInfo`, `ProductField`, `UnorderedTail`, `UnorderedShape`, `FieldLookup`, `UnorderedShapeView`, `Variant`.
  - Re-exported all declarations in `universe.reflection`.
- Added `StructuralUniverseIds` in `phalcom-semantic/src/core_surface/identity.rs`.

### Task 5: Generic Conformance Proof Authority for Structural Traits
- Updated `evaluate_conformance_goal` in `generic_evidence.rs` to recognize canonical structural traits and query `StructuralClassifier`.
- Proves structural conformance via `CompilerDerivedConformanceEvidence` without fabricating `ImplId`s.
- Explicit source `impl`s targeting canonical structural traits are rejected with `DiagnosticCode::ConformanceCompilerDerivedTrait` in `phalcom-semantic/src/impls.rs`.
- Verified in `tests/semantic/capabilities/structural.rs`.

### Tasks 6–9: Structural Reflection Projector & Intrinsic Operations
- Registered `IntrinsicTypeOwner::UnorderedShape` and intrinsic operations:
  - `UnorderedShapeLookup`: takes a key argument and returns `FieldLookup` (`Required`, `Optional`, `Absent`, `Unknown`).
  - `UnorderedShapeView`: returns `UnorderedShapeView` (`Field(first, rest)` or `Empty(tail)`).
  - `UnorderedShapeTail`: returns `UnorderedTail` (`Closed`, `Typed`, `Unconstrained`, `Parameter`, `Projection`).
- Implemented `StructuralReflectionProjector` in `phalcom-semantic/src/structural_reflection.rs`:
  - `project_product_kind`: Ordered vs Unordered `ProductKind`.
  - `project_product_shape`: maps to `ProductShape::Ordered` / `Unordered`.
  - `project_ordered_product_member`: `OrderedShape`, `Entries`, `PositionalEntries`, `LabeledEntries`.
  - `project_unordered_product_member`: `UnorderedShape`.
  - `project_sum_variants`: declaration-ordered pack of `Variant(name, shape)` for enum/data cases.
  - `project_unordered_shape_member`: evaluates `Lookup`, `View`, `Tail`.
  - `project_member`: universal dispatch entry point.

### Task 10: Generic and Stuck Projections
- Generic type parameters and unresolved structural projections evaluate to `StaticMemberProjectionOutcome::Stuck(MemberProjection)` rather than erroring or fabricating `TypeId`s.

---

## Verification Evidence

1. **Compilation Check:**
   - `RUSTFLAGS='' cargo check -p phalcom-semantic` passed with 0 errors and 0 warnings.
2. **AST / Parser Tests:**
   - `cargo test -p phalcom-ast --test trait_syntax` passed (14/14 tests pass).
3. **Static Domains & Type Members:**
   - `cargo test -p phalcom-semantic --test semantic foundations::static_domains` passed (5/5 tests pass).
   - `cargo test -p phalcom-semantic --test semantic foundations::type_members` passed (14/14 tests pass).
4. **Structural Classification Tests:**
   - `cargo test -p phalcom-semantic --test semantic foundations::structural_classification` passed (3/3 tests pass).
5. **Structural Capabilities & Reflection (Gates G5, G6, G8, G9, G10):**
   - `cargo test -p phalcom-semantic --test semantic capabilities::structural` passed (15 passed, 1 pre-existing baseline failure `contextual_product_components_propagate_to_nested_literals`).
   - Covered gates:
     - G5: Compiler-derived structural matrix and rejection of explicit source impls.
     - G6: Ordered product reflection (`Product::Kind`, `Product::Shape`, `OrderedProduct::Entries`).
     - G8: Unordered product reflection (`Product::Kind`, `Product::Shape`, `UnorderedShape::Lookup`, `View`, `Tail`).
     - G9: Sum variant reflection preserving declaration order and exact variant payload shapes.
     - G10: Generic stuck projections returning `StaticMemberProjectionOutcome::Stuck`.
