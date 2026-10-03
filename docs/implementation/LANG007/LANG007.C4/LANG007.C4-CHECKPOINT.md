# LANG007.C4 — Structural Static Upgrade

**Status:** DESIGN_RATIFIED / DOCUMENTATION_MIGRATED / IMPLEMENTATION_NOT_STARTED  
**Decision:** [TDR-0090](../../../decisions/accepted/0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md)

## Authoritative specifications

C4 semantics are owned by:

- [Bindings and Fields](../../../specs/bindings-and-fields.md)
- [Static Evaluation](../../../specs/static-evaluation.md)
- [Type-Level and Static Domains](../../../specs/type-level-domains.md)
- [Type Members](../../../specs/type-members.md)
- [Structural Products](../../../specs/structural-products.md)
- [Structural Sums](../../../specs/structural-sums.md)
- [Arguments and Parameters](../../../specs/callables/arguments.md)
- [VM Products](../../../specs/vm/products.md)

Historical C1–C3 implementation plans remain evidence for the current implementation
substrate but do not override these C4 specifications.

## Ratified C4 surface

```telos
let x = runtimeExpression
let mut y = runtimeExpression
let mut pending: T
const z = staticExpression
```

`let` is runtime immutable, `let mut` runtime mutable, and `const` compile time.
An uninitialized `let mut` is a storage state; reading it traps. Local flow analysis
proves definite initialization and removes checks where possible.

Fields are explicit and immutable by default; `mut` marks mutable fields. Immutable
fields are initialized during the constructor lifecycle.

Static computation uses parameterized `type` declarations and ordinary Telos
semantic control. C4 has no const functions and no compile-time VM.

`Pack` is the first-class variadic product used for rest capture and spreading.
Product/Sum reflection is ordinary static data built from compiler-derived sealed
structural capabilities.

## Implementation invariants

1. static evaluation lives in `phalcom-semantic`;
2. no arbitrary runtime dispatch is available to static evaluation;
3. semantic `while` is explicit; `whileTrue` has no static meaning;
4. runtime/static matching share the existing pattern/GADT authority;
5. Pack is flat/slice/virtual where profitable and fixed calls do not allocate Pack;
6. unordered FieldKey identity is stable and independent of runtime IDs;
7. reflection is shallow with respect to recursive Type references;
8. normalized residual IR is semantic, not AST/evaluator state;
9. exported const value changes participate in incremental fingerprints;
10. runtime use of const materializes a frozen result and never reruns its initializer.

## Documentation migration

TDR-0052 is superseded. The public TypePack/RecordFields ontology is retired in favor
of Pack/Product/UnorderedShape source semantics; optimized internal terms may remain.

Implementation planning must re-audit the live repository before each C4 plan lands.
