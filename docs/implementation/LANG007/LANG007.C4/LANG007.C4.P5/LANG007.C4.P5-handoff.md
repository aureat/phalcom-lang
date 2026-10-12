---
id: LANG007.C4.P5
kind: implementation-handoff
title: "Product/Sum Reflection and Compiler-Derived Structural Capabilities — Handoff"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
recorded: 2026-10-12
updated: 2026-10-12
branch: main
---

# LANG007.C4.P5 — Handoff

`LANG007.C4.P5` is complete (`IMPLEMENTED / FOCUSED_TESTED`).
This handoff records the established interfaces, invariants, and instructions for downstream plans (`LANG007.C4.P6`, `LANG007.C4.P7`, `LANG007.C4.P8`).

## Read First

1. [C4 Checkpoint Record](LANG007.C4-CHECKPOINT.md) for checkpoint-wide authority and ledger.
2. [P5 Implementation Plan](LANG007.C4.P5-implementation-plan.md) for the architecture and task breakdown.
3. [P5 Walkthrough](LANG007.C4.P5-walkthrough.md) for verification evidence and changes made.
4. Normative structural reflection declarations in `phalcom-core/core/universe/src/reflection/structural.ph`.

## Established Takeover Interfaces and Invariants

1. **Lower Structural Authority:**
   - `phalcom-semantic/src/structural.rs` remains the canonical classifier for `StructuralClassification` (`OrderedProduct`, `UnorderedProduct`, `Sum`).
   - Nominal data with named fields classify as `UnorderedProduct(UnorderedProductForm::Data { owner })`.
   - Nominal data with positional/empty fields classify as `OrderedProduct(OrderedProductForm::Data { owner })`.

2. **Domain-Neutral Member Projection:**
   - Located in `phalcom-semantic/src/member_projection.rs`.
   - `StaticMemberProjectionOutcome` cleanly distinguishes `ReducedValue(StaticValueRef)` from `Stuck(MemberProjection)` and `Invalid(Box<str>)`.
   - Evaluator does not fabricate fake `TypeId`s in `TypeStore` for values or reflection terms.

3. **Compiler-Derived Structural Capabilities:**
   - Trait goals for `Product`, `OrderedProduct`, `UnorderedProduct`, and `Sum` evaluate via `evaluate_conformance_goal` in `generic_evidence.rs`.
   - Evidence is represented by `GenericConformanceEvidence::CompilerDerived(CompilerDerivedConformanceEvidence)`. No synthetic `ImplId`s are fabricated.
   - Explicit user `impl`s targeting canonical structural traits are sealed and rejected with `DiagnosticCode::ConformanceCompilerDerivedTrait`.

4. **Universal Structural Reflection Projector:**
   - Centralized in `phalcom-semantic/src/structural_reflection.rs` (`StructuralReflectionProjector`).
   - Evaluates:
     - `Product::Kind`: returns `ProductKind::Ordered` or `ProductKind::Unordered`.
     - `Product::Shape`: returns `ProductShape::Ordered(...)` or `ProductShape::Unordered(...)`.
     - `OrderedProduct` members: `OrderedShape`, `Entries`, `PositionalEntries`, `LabeledEntries`.
     - `UnorderedProduct` member: `UnorderedShape`.
     - `Sum` member: `Variants` (positional pack of `Variant(name, shape)` preserving declaration order).
     - `UnorderedShape` intrinsic operations: `Lookup`, `View`, `Tail`.
   - Projections on generic/unresolved subjects return `StaticMemberProjectionOutcome::Stuck`.

5. **Static Value Store Integration:**
   - Reduced values are ordinary static values stored in `StaticValueStore` (`Data`, `Variant`, `Pack`, `Record`, `Symbol`, `Type`).

## Downstream Consumers

- **P6 (`LANG007.C4.P6`):** Consumes `UnorderedShape::View` (`Field(first, rest)` stepping to `Empty(tail)`) and `Sum::Variants` for static `for`, matching, and proof predicates.
- **P7 (`LANG007.C4.P7`):** Consumes `StructuralReflectionProjector` and `StaticMemberProjectionOutcome::Stuck` for recursive generic static computation and memoization.
- **P8 (`LANG007.C4.P8`):** Preserves static value nodes during cross-module static serialization.
