# Checkpoint Record — LANG005.C1 Data, Enums, Anonymous Products, and Representation Convergence

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

## C1.P3 completed state — 2026-09-13

### Lifecycle

```yaml
plan: LANG005.C1.P3
status: COMPLETED
completion: COMPLETE
verification: FOCUSED_TESTED
```

### Implemented in LANG005.C1.P3

- **C0 & C1 Takeover & Core Infrastructure:**
  - Shared `ProductShape` and `RuntimeAnonymousProductDescriptor` registries wired into the VM.
  - `TupleObject` and `RecordObject` converted to packed `ProductStorage` with descriptor-directed access.
  - Added static anonymous product build bytecodes `BuildStaticTuple` and `BuildStaticRecord`.
  - Normalized empty products to `Unit` across dynamic and static construction paths without allocating empty heap objects.
  - Enforced representation-independent `===` / `VM::semantic_same` across Tuple, Record, and nested mixed transparent products.

- **C2 Runtime Semantics & Interoperability:**
  - Standardized Tuple and Record view APIs (`TupleView`, `RecordView`) resolving physical layouts and presentation orders through VM metadata.
  - Verified GC tracing, argument unpacking, and collection/hashing interop over converted product storage.

- **C3 Anonymous-Product Scalar Replacement & Optimizer Generalization:**
  - Generalized `VirtualProductKind` to cover `Data`, `Tuple`, `Record`, and `Variant`.
  - Unified `NestedData` into `NestedProduct` across `VirtualComponentPlan` and path indexing methods.
  - Generalized `ProductPlanner` to identify static Tuple and Record literal candidates from lowering specs.
  - Unified eligibility and allocation sinking rules across `data`, `Tuple`, and `Record`.
  - Implemented source-order evaluation and contiguous leaf slot reservation for virtual Tuple and Record construction and projection.
  - Materialized transparent virtual products on whole-value reads sinking to canonical static product construction.

- **C4 Runtime Generic, Dynamic, and Reification Closure:**
  - Added `RuntimeTypeRecipe` (`Closed`, `Template`) and `RuntimeTypeEnvironmentId` / `RuntimeTypeEnvironmentRegistry`.
  - Attached `type_environment` to `CallFrame` and propagated it through frame creation and lexical `BlockObject` captures.
  - Implemented lazy `instantiate_type_recipe` with cycle protection across nominal, applied, union, tuple, record, and callable types.

- **C5 Verification & State Closure:**
  - All unit and integration test suites pass focused verification gates.

### Verification evidence

- `RUSTFLAGS='' cargo test -p phalcom-core --lib compiler::lib::product_opt` — 11 passed.
- `RUSTFLAGS='' cargo test -p phalcom-core --lib typing::environment::tests` — 2 passed.
- `RUSTFLAGS='' cargo test -p phalcom-core --test core product` — 4 passed (1 ignored pre-existing).
- `RUSTFLAGS='' cargo test -p phalcom-core --test core outgoing_packs` — 28 passed.
