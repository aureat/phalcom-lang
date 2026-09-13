# Implementation State — LANG005.C1.P2 Representation-Aware Product Optimizer

## C1.P2 base revision
- Remote: `aureat/phalcom-lang`
- Branch: `main`
- Base commit: verified P1 completed state.

## P1 takeover interface map

| P2 concept | Committed P1 symbol/path |
|---|---|
| Data component id | `phalcom_semantic::identity::DataComponentId` |
| Data constructor id | `phalcom_semantic::identity::DataConstructorId` |
| Data declaration lowering spec | `phalcom_core::modules::semantic_lowering::DataDeclarationLoweringSpec` |
| Data component lowering spec | `phalcom_core::modules::semantic_lowering::DataComponentLoweringSpec` |
| Data construction lowering spec | `phalcom_core::modules::semantic_lowering::DataConstructionLoweringSpec` |
| Product layout spec | `phalcom_core::product::ProductLayoutSpec` |
| Product storage | `phalcom_core::product::ProductStorage` |
| Construct data bytecode | `phalcom_core::bytecode::Bytecode::ConstructData` |
| Load data singleton bytecode | `phalcom_core::bytecode::Bytecode::LoadDataSingleton` |
| Get data component bytecode | `phalcom_core::bytecode::Bytecode::GetDataComponent` |
| General enum lowering spec | `phalcom_core::modules::semantic_lowering::EnumLoweringSpec` |
| General variant lowering spec | `phalcom_core::modules::semantic_lowering::VariantLoweringSpec` |
| Construct variant bytecode | `phalcom_core::bytecode::Bytecode::ConstructVariant` |
| Load variant singleton bytecode | `phalcom_core::bytecode::Bytecode::LoadVariantSingleton` |

## Architectural amendments applied
- **Canonical BindingId optimizer identity:** Local candidates and occurrences are identified via `BindingId` instead of lexical `(range, name)` reconstruction.
- **Source-order constructor evaluation:** Constructor arguments evaluate exactly once in source order, storing into logical component slots.
- **Loop-repeat allocation sinking restriction:** Whole-value uses inside loops where loop depth > binding creation loop depth bail out to canonical P1 eager materialization.
- **Pure eligibility boundary:** Eligibility decisions are pure and separate from emission.
- **Value frame slots:** Virtual leaf components live as standard `Value` frame slots (GC roots, unboxed native scalar registers are out of scope).
- **Strict enum proof:** General enum cases are virtualized only when provably unobservable as whole values; zero enum rematerialization.

## Checkpoint status
- [x] C0 — Takeover, optimizer mode, proof planner (Complete)
- [x] C1 — Virtual slot substrate, projection, and rematerialization primitives (Complete)
- [x] C2 — Positive-arity data scalar replacement and allocation sinking (Complete)
- [x] C3 — Recursive nested data virtualization and bounded profitability (Complete)
- [x] C4 — Exact general-enum virtualization across resolved matches (Complete)
- [x] C5 — Compiler integration, qualification, performance evidence, and delivery (Complete)

## Established invariants
1. With `ProductOptimizationMode::Disabled`, bytecode emission is byte-for-byte canonical P1.
2. Production compilation defaults to `ProductOptimizationMode::Enabled`.
3. Optimizer planner never creates secondary name resolution or type inference.
4. Transient optimizer plans are never persisted in the semantic DB.

## Active incident
None.
