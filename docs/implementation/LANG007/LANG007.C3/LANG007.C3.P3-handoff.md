---
id: LANG007.C3.P3
kind: continuation-handoff
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# LANG007.C3.P3 continuation handoff

This is a continuation handoff, not terminal P3/C3 closure. It records the
current stop point after T1-T8 implementation work and a partial T9
certification attempt.

## Current state

- T1/G1 through T8/G8 implementation work is complete. T8's authoritative
  specification ownership/search gate passes.
- T9/G9-G11 is open. Do not run T10 or claim P3/C3 completion until remaining
  acceptance coverage and baseline classifications are reconciled.
- The P3 amendments INC-C3-12, INC-C3-13, and INC-C3-14 are accepted and their
  architecture is implemented; preserve their owner-binding, inherent-impl
  type-member, CompleteTypePack projection, RecordRow, and runtime-class
  projection decisions.

## Resolved generic-owner payload failure

`associated_static_domain_payload_uses_owner_generic_parameter_kind` expects
one constructor for `class Container<T: Type> { type data State { Box(_ value:
T) } }`. The same test failed at clean predecessor `9b4ca55d`. The catalog-backed
signature builder rejected kind `Type` while the existing associated-domain
builder accepted `Type` or a static-domain kind; the checked signature was
therefore absent and the catalog omitted its constructor. T0869/T0870 define the
constructor input kind as `K`. The catalog path and fallback path now both
accept proper `Type` and static-domain kinds. The regression asserts the
published constructor payload kind is `Type`, and the full semantic ADT suite
passes (277 passed, 25 intentionally ignored).

## Verification evidence

- `cargo fmt --all -- --check` — baseline-blocked by pre-existing workspace
  formatting drift documented in the P2 handoff; repository-wide formatting
  churn was discarded.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets` — pass.
- `cargo test -p phalcom-type-meta` — 14 schema tests pass.
- `cargo test -p phalcom-ast` — 53 unit and 261 integration pass; six stale
  `associated_type_data: []` snapshots and one selector-diagnostic assertion
  fail; one ignored. The same seven failures reproduce at clean predecessor
  `9b4ca55d` (53 unit + 270 integration pass; one ignored). Generated
  `.snap.new` files were removed.
- `cargo test -q -p phalcom-semantic --test semantic semantic::adts` — 277
  pass, 25 ignored; the generic-owner payload failure is fixed.
- `cargo test -p phalcom-semantic --test semantic semantic::impls::queries` —
  54 pass, one data-component witness-plan assertion failure; that exact test
  reproduces at clean predecessor `9b4ca55d`.
- `cargo test -q -p phalcom-core --test core language::algebraic_data` — 42
  pass, two invocation-layout constructor-reference failures, 19 ignored. Both
  exact failures reproduce at clean predecessor `9b4ca55d`. The nested ADT
  fixture's second `enum` was migrated to `data` and now passes.
- The static-domain source-index test (including impl-associated domain/case
  identities) passes; the focused LSP navigation/symbol/rename test passes.
- Focused T9 checks pass for metadata identity round-trip, imported static-domain
  provider invalidation with cold parity, type-data edit cold/incremental parity,
  and rejection of a TypeOnly case as a runtime value.
- The generic-owner payload regression passes after aligning catalog-backed
  constructor signatures with the existing Type-or-static-domain payload rule.
  `cargo test -q -p phalcom-semantic --test semantic semantic::adts` passes 277
  tests, with 25 intentionally ignored.
- `associated_static_case_payload_tracks_generic_owner_kind_edit_with_cold_parity`
  passes: changing the owner's `T` kind from `Type` to `Shape` rebuilds the
  payload signature and matches cold analysis. The associated type-member
  incremental module passes 24/24.
- Universe bootstrap counters match the P3 starting revision `518f66d1`:
  106 discovered units, 37 root-reachable units, 101 executed units. The
  intrinsic facade migration adds no measured bootstrap work.
- `PATH=target/debug:$PATH npm run test:lsp:e2e` passes all 11 tests. The first
  run omitted the built LSP binary directory from PATH and logged `phalcom-lsp`
  spawn errors; the corrected run had no such errors.
- A full semantic test-binary run was started and then stopped after
  high-fanout incremental stress tests exceeded the useful gate window. It is
  incomplete, not a pass.

The shared checkpoint has the fuller gate ledger and baseline classification.

## Next actions

1. Reconcile the remaining T9 plan coverage and workspace gates; metadata,
   import/incremental, runtime-negative, bootstrap-counter, and VS Code e2e
   checks listed above are complete. The generic-owner payload acceptance
   failure is fixed and its incremental owner-kind change matches cold analysis.
2. Keep P3/C3 open until T9 and all T10 conditions pass; do not create terminal
   completion records while required T9 coverage remains open.
