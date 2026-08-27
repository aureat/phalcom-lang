# Semantic Capability Gap Closure Baseline

Date: 2026-08-27

Post-Technical-03 baseline:

```text
586238b08e2797e594d3e0bde569fef6da412a66 semantic: land Technical 03 proof integrity
```

The supplied gap-closure plan was grounded at `1ffa7c1d`, but the checkout
already contains the completed Technical 03 delivery. All subsequent work
uses the post-Technical-03 callable and inference APIs.

## Verification

Technical 03 remains green:

- inference foundations: 17 passed;
- generic proof-integrity regressions: 8 passed;
- generic capability suite: 19 passed.

Gap probes at this baseline:

| Probe | Baseline result |
| --- | --- |
| `refined_branch_with_abrupt_else_publishes_only_normal_value` | Fails: branch expression remains `Assumed(Object)` instead of established `Int`. |
| `higher_order_block_call_propagates_captured_result` | Fails when run ignored: `.call()` produces `Unknown(DynamicMessageSend)`. |
| `field_facts_survive_constructor_and_general_writes` | Fails when run ignored: field read remains `Assumed` instead of `Established`. |
| `collection_and_destructure_facts_preserve_element_shapes` | Fails when run ignored: `tail` binding is absent. |
| `dependency_edit_remove_readd_recomputes_affected_summary_deterministically` | Fails when run ignored: reused caller loses `Arc` identity. |
| `imported_binding_use_resolves_to_exported_declaration_not_local_import_site` | Separate failure: read resolves to local import binding instead of external declaration. |

## Post-Spec-03 overlap

Technical 03 owns generic argument binding, proof-state publication, directed
inference constraints, cancellation/budget control, and fixed-return fallback
in `checker/call.rs`, `checker/inference.rs`, `checker/context.rs`, and
`types/evidence.rs`. Gap work must reuse those APIs. The remaining affected
seams are branch transfer/execution, structural callable-value dispatch,
pattern decomposition, field-flow/lifecycle publication, incremental refresh,
and source-index import identity.

## Scope boundary

Reflection semantics and unrelated LSP work remain outside this plan. Existing
repository formatting/Clippy baselines are tracked separately and are not
treated as capability evidence.
