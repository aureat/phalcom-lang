# PERF003.C2.P7.PATCH5-R baseline first-failure ledger

All entries were observed on `9cbba391a4482052dec476b327619c957a028b19`
before production changes. `D — baseline` describes when the issue arose;
items in PATCH5-R scope remain active acceptance work.

| ID | Observed failure | Classification and owner | Reproduction | Follow-up |
|---|---|---|---|---|
| F1 | Whole-load ledger reports `323,324,452` bytes; isolated allocator observes `528,785,777`, a `205,461,325`-byte delta. The exact-equality assertion fails before the test reaches N-1. | D — baseline accounting gap; active P0. Owner is not yet localized across the module, semantic, query, and publication load path. | `STRICT_ALLOCATION_GATE=1 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1` | R.T1-R.T4 must trace all charged allocations, close the owner graph, and re-run exact N/N-1 plus independent equality. |
| F2 | Source CLI rejects `for (entry in propositions)` at `packages/slotmap/src/main.ph:27:12`. | D — fixture syntax defect, owner `packages/slotmap/src/main.ph`; active R.T5. | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo run -p phalcom-core --bin phalcom -- packages/slotmap` | R.T5.1 may normalize to `for entry in propositions` after confirming the live syntax and recompiling. |
| F3 | `slotmap-eval` reports matching errors for `packages/slotmap` and claims full parity. The root has `project.toml`, while the harness uses `EntrySelection::Package`, which requires `package.ph`. | D — harness selector defect in `phalcom-core/examples/perf_audit_p7.rs::cmd_slotmap_eval`; active R.T5/R.T6. No package compilation occurred. | `target/debug/examples/perf_audit_p7 slotmap-eval --artifact target/telos/universe/universe.telosc` | Use the project selection API for the project root and assert both positive compilation results before comparing products. Keep negative parity separate. |
| F4 | Runtime test project rejects physical file `basic_test.ph` as non-canonical; `dense_test.ph` is also present under the same naming form. | D — fixture naming defect in `packages/slotmap/tests/runtime/src`; active R.T5. | `slotmap-eval` source/artifact run; both reach the same physical-name diagnostic. | Rename only after verifying that `basic_test` and `dense_test` logical imports map to canonical kebab-case physical paths. |
| F5 | Workspace `--no-run` compilation fails with E0601 because `phalcom-core/examples/perf_workloads.rs` has no `main`. | D — unrelated example target, owner `phalcom-core/examples/perf_workloads.rs`; does not undermine P7 loader correctness. | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run` | Record at R.G7. Do not repair in PATCH5-R unless new evidence ties it to changed code. |
| F6 | Artifact-mode runtime reaches VM initialization and fails because `ProductKind` has no canonical facade binding. | D — `VM_RUNTIME_BOOTSTRAP_DEFECT`, P8 boundary; owner is runtime Universe bootstrap. | `target/debug/examples/perf_audit_p7 runtime-single --mode artifact --artifact target/telos/universe/universe.telosc --sample-id 0` | Keep runtime bootstrap `OUT_OF_SCOPE`; do not use runtime execution as a PATCH5-R gate. |

## Successful baseline observations

- `cargo check --lib` passed for all four directly affected libraries.
- Focused allocation-helper, module topology/query, and semantic allocation
  tests passed.
- Artifact generation and loading completed. `source-denial-check` compiled a
  synthetic package with an empty source list, but did not deny filesystem
  access.
- One minimal inline compile succeeded through source mode and one through
  artifact mode. These single debug samples establish buildability only; they
  do not prove semantic product parity or a performance claim.

## PATCH5-R progress after the R.T0 baseline

The baseline F1 whole-load mismatch remains the starting failure. R.T1 narrowed
the active diagnostic to the complete semantic import stage, which still has an
independent allocator delta of `2,468,555` bytes (`266,069,278` observed versus
`263,600,723` charged). This does not yet identify the owning allocations or
close R.G1. The type graph substage is exact at `765,031/765,031` bytes. See the
post-baseline command results in
`PERF003.C2.P7.PATCH5-R-command-log.md`; do not rewrite the historical R.T0
measurements above.
