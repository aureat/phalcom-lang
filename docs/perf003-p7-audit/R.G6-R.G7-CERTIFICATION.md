# PERF003.C2.P7 R.G6/R.G7 certification matrix

Test outcomes below were checked against the saved outputs in `logs/`. Reconciliation and implementation revisions are distinct: performance data was produced from `f2550f7193ae776f78e58d1ed187a3ae6c7efdd2`; the additional R.G7 test cases were committed at `c1524361b9606dc537671997fe5069f2aa66035b`.

## R.G6 — evidence integrity

| Gate | Revision tested | Exact command / evidence | Result | Notes |
|---|---|---|---|---|
| Artifact identity and provenance | `f2550f71` | `perf_audit_p7 generate --out target/telos/universe/universe.telosc --json target/perf003-p7-audit/ARTIFACT.json`; [ARTIFACT.json](ARTIFACT.json) | PASS | 15,936,496 bytes; BLAKE3-256 `b91ccc4e...9665bf`; source-free complete. |
| Cold sample integrity | `f2550f71` | `perf_audit_p7 run-cold-suite --samples 10`; [RAW-COLD.csv](RAW-COLD.csv), `logs/cold-suite.txt` | PASS | 80 rows, 80 PIDs, 10 rows per workload/mode, all SUCCESS. |
| Warm process isolation | `f2550f71` | `perf_audit_p7 run-warm-suite --repetitions 20`; [RAW-WARM.csv](RAW-WARM.csv), `logs/warm-suite.txt` | PASS | Two distinct PIDs; one warmup and 20 successful rows per mode. Each mode alternates two equivalent workload snippets, with the input variant recorded per row. |
| Runtime failures represented | `f2550f71` | `perf_audit_p7 run-runtime-suite --samples 10`; [RAW-RUNTIME.csv](RAW-RUNTIME.csv), `logs/runtime-suite.txt` | PASS | 20 unique PIDs; all 20 are `RUNTIME_ERROR`, with no runtime speed claim. Error details are blank. |
| Same-image serialization determinism | `f2550f71` | `cargo test -p phalcom-core --test core modules_canonical_artifact::` — `build_and_load_universe_telosc_compiles_source_free_inline_user_entry` | `logs/core-canonical-artifact.txt` | PASS; rebuilding from the same canonical program yields byte-identical output and the same artifact digest. |
| Statistics reproducible | `f2550f71` | Recompute from the three raw CSVs; conventions in [REPORT.md](REPORT.md) | PASS | Nearest-rank p95 and population standard deviation. |
| Environment and optimization profile | `f2550f71` | [ENVIRONMENT.md](ENVIRONMENT.md) | PASS | Toolchain, OS, CPU, release flags, and benchmark binary recorded. |
| Allocation and package evidence | `f2550f71` | `allocation-check`, `source-denial-check`, `slotmap-eval`; `logs/allocation-check.txt`, `logs/source-denial-check.txt`, `logs/slotmap-eval.txt`, [RAW-SLOTMAP.csv](RAW-SLOTMAP.csv) | PASS | N=373,069,634; N−1 rejected, exact retry succeeds; source-free compilation and package parity pass. |
| Unsupported memory claims | `f2550f71` | [MEMORY.md](MEMORY.md) | UNVERIFIED | Peak RSS does not establish retained memory or leak behavior. |
| **R.G6 ruling** | `f2550f71` | [REPORT.md](REPORT.md) | **PASS** | All failed runtime rows remain classified, and no failed compile is counted as successful. |

## R.G7 — required regression and corruption cases

| Required case | Revision | Exact test / command | Evidence | Result and notes |
|---|---|---|---|---|
| Missing `MODULE_WORLD`, `SEMANTIC_WORLD`, `QUERY_IMAGE`, or `METADATA` | `c1524361` | `cargo test -p phalcom-compiler-artifact each_required_compiler_section_cannot_be_omitted` | `logs/artifact-commit-c152.txt` | PASS; each omission returns `MissingRequiredSection` for that section. |
| Container digest mismatch | `c1524361` | `cargo test -p phalcom-compiler-artifact whole_artifact_digest_mismatch_is_rejected` | `logs/artifact-commit-c152.txt` | PASS; typed `ArtifactDigestMismatch`. |
| Section digest mismatch | `c1524361` | `cargo test -p phalcom-compiler-artifact section_digest_mismatch_is_rejected` | `logs/artifact-commit-c152.txt` | PASS; typed section mismatch. |
| Schema / ABI mismatch | `f2550f71` | `cargo test -p phalcom-core --test core modules_canonical_artifact::` — `loader_rejects_rehashed_incompatible_header_fields` | `logs/core-canonical-artifact.txt` | PASS; rehashed incompatible fields are rejected. |
| Native catalog fingerprint mismatch | `f2550f71` | Same core target — `foreign_native_catalog_fingerprint_is_rejected_before_section_materialization` | `logs/core-canonical-artifact.txt` | PASS; mismatch rejected before section materialization. |
| Wrong package/world provenance | `f2550f71` | `cargo test -p phalcom-compiler-artifact metadata_rejects_trailing_bytes_invalid_json_and_wrong_package`; `cargo test -p phalcom-semantic --lib canonical_image_` | `logs/compiler-artifact.txt`, `logs/semantic-canonical-image-matrix.txt` | PASS; wrong package and foreign semantic world rejected. |
| Oversized encoded section | `c1524361` | `cargo test -p phalcom-compiler-artifact section_size_limit_is_enforced` | `logs/artifact-commit-c152.txt` | PASS; typed section budget rejection. |
| Oversized decoded family | `c1524361` | Full `cargo test -p phalcom-compiler-artifact`; callable-family count/depth limits | `logs/artifact-commit-c152.txt` | PASS; representative metadata and callable families are bounded. |
| Invalid or duplicate module/import identity | `c1524361` | `cargo test -p phalcom-modules --lib module_world_rejects_invalid_and_duplicate_module_import_identities -- --nocapture` | `logs/modules-invalid-identities-commit-c152.txt` | PASS; bad module name, duplicate module path, empty alias, and duplicate import binding rejected. |
| Invalid or duplicate type/static identity | `c1524361` | `cargo test -p phalcom-semantic --lib type_graph_import_rejects_duplicate_relocated_type_identity`; full compiler-artifact suite includes `static_graph_rejects_duplicate_relocated_pack_labels` | `logs/semantic-duplicate-type-relocation-commit-c152.txt`, `logs/artifact-commit-c152.txt` | PASS; duplicate canonical type and static pack identities rejected. |
| Missing callable/dispatch/proof/semantic owner | `f2550f71` | `cargo test -p phalcom-semantic --lib artifact::`; full compiler-artifact suite | `logs/semantic-lib-artifact.txt`, `logs/compiler-artifact.txt` | PASS; complete owner links, dispatch links, proof closure, and semantic ownership are validated. |
| Unsupported or source-dependent query root | `f2550f71` | `cargo test -p phalcom-semantic --lib query_persistence_classification_is_fail_closed_for_nonstructural_products -- --nocapture` | `logs/semantic-query-fail-closed.txt` | PASS; open-world/unsupported products are not frozen. |
| Duplicate relocated query key | `f2550f71` | `cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture` — `query_closure_debits_exact_allocator_requests_and_rolls_back_on_exhaustion` | `logs/semantic-artifact-allocation.txt` | PASS; duplicate roots reject, and exhausted child closure rolls back. |
| Missing, stale, or cyclic query dependency | `f2550f71` | `cargo test -p phalcom-semantic --lib canonical_image_ -- --nocapture` — `canonical_image_rejects_foreign_stale_and_cyclic_dependencies` | `logs/semantic-canonical-image-matrix.txt` | PASS; missing/foreign, stale, and cyclic evidence fails closed. |
| Foreign semantic world / incompatible type or static prefix | `f2550f71` | Same canonical image command — `canonical_image_reader_validates_all_final_arena_and_static_roots` | `logs/semantic-canonical-image-matrix.txt` | PASS; foreign world, type prefix, and static-value root are rejected. |
| Section-local allocation exhaustion | `f2550f71` | `cargo test -p phalcom-core --test artifact-allocation -- --nocapture --test-threads=1` | `logs/core-artifact-allocation.txt` | PASS; local section failure is typed and child debit does not commit. |
| Cumulative allocation exhaustion | `f2550f71` | Same core allocation test | `logs/core-artifact-allocation.txt` | PASS; N−1 fails closed, exact N succeeds. |
| Failure immediately before publication; no partial image | `f2550f71` | `cargo test -p phalcom-core --lib loaded_artifact_prepublication_failures_are_retryable -- --nocapture` | `logs/core-prepublication-retry.txt` | PASS; injected failures at four prepublication boundaries return no loaded artifact. |
| Retry following failure | `f2550f71` | Same prepublication retry test | `logs/core-prepublication-retry.txt` | PASS; each failed load is followed by a successful retry matching control identities. |
| Source/frozen query/read parity | `f2550f71` | `cargo test -p phalcom-core --test core modules_canonical_artifact::` — `source_backed_and_artifact_backed_compiler_program_parity`; canonical image reader tests | `logs/core-canonical-artifact.txt`, `logs/semantic-canonical-image-matrix.txt` | PASS; normalized compiler products match and frozen reader authority is valid. |
| Source-free package compile and SlotMap runtime | `f2550f71` | Core canonical artifact target; `cargo run -p phalcom-core --bin phalcom -- packages/slotmap/tests/runtime` | `logs/core-canonical-artifact.txt`, `logs/slotmap-runtime.txt` | PASS; compiler source denial and runtime smoke both pass. |
| Broad workspace compile/test | `f2550f71` | `cargo test --workspace --no-run`; `cargo test --workspace` | `logs/workspace-no-run.txt`, `logs/workspace-test.txt` | BASELINE BLOCKED; 412 stale semantic fixture/API errors, also present at clean predecessor `08e2b2f5`. |
| **R.G7 ruling** | `c1524361` | Focused matrix above; broad baseline classified separately | [REPORT.md](REPORT.md) | **PASS**; the inherited broad failure does not invalidate focused P7 gates. |

## Final status

- P7 implementation: `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`.
- P7 independent certification: `ACCEPTED / RECERTIFIED`.
- R.G6: `PASS`.
- R.G7: `PASS`.
- P8 takeover: `UNBLOCKED`.
