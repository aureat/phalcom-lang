# Walkthrough — LANG005.C2.P4 Exact-Case Conditional Dispatch Precedence

```yaml
plan: LANG005.C2.P4
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

## Result

Closed DEF-006 and restored the C2 predecessor gate required by
`LANG005.C3.P1`. An applicable exact-enum-case member now wins over a
same-selector enum-root requirement/default, and the selected case behavior
remains executable for nested sends through root-typed payloads.

## Implementation

- `phalcom-semantic/src/checker/context.rs` now checks the applicable exact
  case before returning ordinary enum-root dispatch results and publishes the
  existing `ConditionalDispatchSelection` product.
- `phalcom-semantic/src/impls.rs` and `phalcom-semantic/src/editor.rs` apply
  the same exact-case overlay for receiver-effective lookup and preserve the
  case-owned callable identity.
- `InherentImplContribution` and `ConditionalInherentMember` now preserve
  whether an impl is genuinely receiver-dependent. Exact-case identity
  domains for unconditional/covering implementations stay semantically
  indexed but remain eligible for `VariantMethod` installation; specialized
  or constrained domains remain conditional and isolated from shared classes.
- `phalcom-core/src/modules/semantic_lowering.rs` consumes that semantic flag
  instead of treating every exact-case index entry as runtime-conditional.

No VM applicability solver, new runtime dispatch registry, or alternate
identity source was introduced.

## Verification

- `semantic::impls` — 41 passed, 0 failed.
- `core language::inherent_impl` — 15 passed, 0 failed.
- C1 consolidation: product tests 6/6, type-environment tests 2/2,
  compiler-lowering Record test 1/1, data E2E test 1/1, semantic generic
  specialization test 1/1, and product-optimizer tests 16/16.

The full workspace/release gate was not run.
