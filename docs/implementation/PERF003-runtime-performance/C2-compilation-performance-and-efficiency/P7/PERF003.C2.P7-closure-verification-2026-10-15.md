# PERF003.C2.P7 full-closure verification — 2026-10-15

**Verdict: P7 IS NOT CLOSED. The COMPLETE / IMPLEMENTED / all-R1–R11-resolved claim is contradicted by live implementation and new executed probes. Do not accept this record set as a verified P8 takeover.**

Scope: read-only implementation/acceptance review in `perf/universe-characterization`, HEAD `64a9b65e1d6de8b386b3181b7b4d68789099c1b8`, with substantial uncommitted implementation. Only this verification record was added; no source, tests, checkpoint, handoff or plan statuses were repaired. No agents spawned or consulted. Passing focused suites are real, but do not prove the contracts attributed to them.

## 1. Required P7 gates were reclassified without satisfying their contract [P1]

[docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md:2988) requires a fresh process with canonical source access fail-fast, novel user compilation, and zero canonical parse/interface/link/update/body/witness/export counters. G7.9 requires zero-copy and P6 locality counters; G7.10 requires byte identity across fresh processes; G7.11 requires materially faster load and retained locality, not merely nonzero duration.

The detailed plan explicitly states at line 3301 that these properties define P7 completion, and at line 3303 that an in-process round trip does not finish P7. G7.3/G7.8/G7.9 therefore cannot be dismissed as broad workspace/release certification. No adopted amendment removing these obligations was found in the active correction plan; its correction section still requires them.

The new walkthrough labels G7.3/G7.8/G7.9 DEFERRED while declaring completion. Its G7.5/G7.6 PASS claims are falsified below; G7.10/G7.11 PASS are not supported by the cited tests. The existing determinism test builds twice from the same cached canonical program. The user program is import-free `print(42)` in the source-built process. No denied-provider child process or locality-counter assertions occur in that test.

## 2. Exhaustive semantic parity is false [P1; R3/R4 remain open]

A new probe built the real canonical Universe artifact, loaded it, then compared source-built and loaded snapshot family counts:

| Required authority | Source-built | Loaded |
|---|---:|---:|
| Callable expression facts | 19,939 | 0 |
| Witness requirement views | 1,342 | 0 |
| Static definitions | 13 | 0 |
| Static constants | 11 | 0 |
| Type-member declarations | 17 | 0 |
| Associated surfaces | 281 | 0 |

[phalcom-semantic/src/artifact/import.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/artifact/import.rs:780) still manufactures callable analyses with empty expression/binding/flow/target/dependency facts, zero dependency fingerprints and Complete status. [phalcom-semantic/src/artifact/import.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/artifact/import.rs:1175) still explicitly initializes required associated/static/type-member families to empty tables. Operational templates and owner indexes remain detached return fields, while core publishes only `parts.snapshot`.

Conformance target/source/generic/authorization transport has improved. Nevertheless, [phalcom-semantic/src/artifact/import.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/artifact/import.rs:1067) still drops every requirement view, copies a process-produced witness fingerprint verbatim, forces `invalid_explicit_members = false`, and starts associated plans from empty. Inherent selection applicability/conditional_impl are omitted. Non-Complete status payloads are replaced with generic reasons, empty failures and zero budget reports (lines 1047–1055). Dispatch still substitutes assumed TypeId(0) facts (line 897). These are material losses, not optional presentation data.

The core test checks that selected tables have at least one entry and finds one populated/Complete witness. It never compares these lost families or their facts. Those assertions cannot establish G7.5 or close R3/R4.

## 3. Query integrity is still permissive [P1; R5 remains open]

[phalcom-semantic/src/artifact/query.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/artifact/query.rs:143) retains the original missing-owner `continue`, unsupported-product `continue`, zero product-fingerprint fallback, zero input fingerprint, debug-string owner maps and conditional edge omission. Wrong-kind products still use `expect` without proving the kind. Several advertised reusable query kinds have no reconstruction branch.

Executed discriminator: one CallableSignature root with owner `missing-owner` is accepted, and reconstruction returns zero roots. This directly contradicts required wire/materialized census and fail-closed owner validation. G7.6 cannot be marked PASS from the existing same-process round trip.

## 4. Cumulative allocation claim is false [P1; R1/R7 remain open]

Executed counting-allocator probe over `QueryImageV1::decode_with_budget`:

```text
query_decoded=true, budget=500012, allocated=2800000, remaining=0
query_ignores_per_section_decoded_limit=true
```

The probe encoded 50,000 small syntactically accepted roots, supplied only their 500,012-byte encoded size as budget, and observed 2,800,000 requested allocation bytes during successful decoding. These duplicate-owner rows are not proposed as a valid final semantic graph; the decoder must respect its allocation budget even if later closure validation rejects their contents.

[phalcom-compiler-artifact/src/query.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-compiler-artifact/src/query.rs:132) allocates native root/dependency vectors before accounting. Line 172 charges only encoded bytes after construction. A supplied per-section decoded limit of one byte is also ignored when the external remaining budget is larger. Merely sharing a variable named remaining_budget does not enforce allocator-requested bytes.

[phalcom-core/src/modules/canonical_artifact.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/canonical_artifact.rs:312) calls native module and semantic reconstruction without passing an allocation ledger; query reconstruction similarly has no ledger. Persistent maps, stores, indexes, cloned identities and scratch therefore remain outside the claimed end-to-end bound. DTO Box<T> construction in module_world.rs is also uncharged.

Content preflight is still serde-backed. A successful `inspect_metadata_json` call over a JSON string containing 200,000 escaped characters requested **524,280 allocation bytes**. The new lexical header probe is an improvement, but preceding content preflight still allocates without transaction accounting. Validator error paths still clone/format untrusted identities. The new borrowed indexes also call stable `sort_by` at validate.rs:324 and :938 without reserving/accounting its possible heap scratch; replacing HashMaps did not alone establish scratch-allocation closure.

Metadata charging is also duplicated: `MetadataSectionV1::decode_with_budget` charges the owned encoded payload at metadata.rs:272–273, then `decode_bundle` calls validate_and_charge_metadata_preflight on that same owned payload at line 155, charging it again without a second payload allocation. Removing the charge from finish_loaded_universe_telosc did not eliminate this earlier duplication.

The new cumulative test only asserts that some error occurs under a smaller budget. It does not isolate the reason, measure allocations, or prove that independently valid sections individually fit their native allocation bounds. The metadata exact-boundary test computes total_charge from the decoder's own reported debit; it is not independent allocator measurement, contrary to the walkthrough wording.

## 5. Portable provenance is still placeholder/self-checked [P1; R6 partly open]

[phalcom-core/src/modules/canonical_artifact.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/canonical_artifact.rs:174) still assigns metadata package_digest = [7;32]. Lines 195–199 set source/interface/semantic/query/metadata identity digests to ZERO. Runtime inspection confirmed zero source and semantic digests in the built header. Core supplies meta_v1.package_digest as its own expected value when decoding metadata (line 368), so package/provenance agreement is not independently established.

Per-section and whole-container digests and the new reader-owned ABI checks are useful real corrections. They do not supply these missing compatibility/provenance relationships. Do not mark all portable identity/provenance acceptance complete.

## 6. Lifecycle records are internally contradictory [P2]

[docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md:9) still declares active P7, latest completed P6, PARTIAL/UNVERIFIED in front matter, while the body declares active P8/latest completed P7 and the normalized plan ledger marks P7 COMPLETE at line 134. The remaining execution plan still declares IN_PROGRESS/PARTIAL/UNVERIFIED and lists required proof gates NOT RUN. The new prose did not finalize a consistent lifecycle state.

These inconsistencies are secondary to the demonstrated code failures. Harmonizing the documents alone cannot close P7.

## 7. Gate disposition supported by this verification

| Gate | Actual defensible disposition |
|---|---|
| G7.0 persistence/index inheritance | Existing P5/P6 evidence does not certify artifact reconstruction |
| G7.1 portable identity/ABI | ABI regressions pass; provenance/fingerprint requirements incomplete |
| G7.2 bounded container | Container tests pass; resource-bounded decoding is refuted |
| G7.3 loaded module/import parity | Required acceptance unrun; not a dispensable release gate |
| G7.4 type/static relocation | Narrow parameter checks and in-process load pass; exhaustive parity unproven |
| G7.5 semantic primary parity | FAIL: real source-built authority is lost on import |
| G7.6 query image | FAIL: missing owner silently disappears; zero/default/skipped products remain |
| G7.7 metadata | Focused transport tests pass; allocation/provenance invariants incomplete |
| G7.8 source-denied fresh-process compile | Required proof unrun |
| G7.9 P5/P6 artifact-path non-regression | Required structural/locality proof unrun; not equivalent to broad workspace testing |
| G7.10 fresh-process determinism | Cited test repeats one cached image; required variation unverified |
| G7.11 performance | Positive phase durations do not demonstrate speedup or P6 locality |

## 8. Executed verification and limits

- Repository-pinned `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core canonical_artifact`: **8 PASS**, 964 filtered, 41.82 seconds. Log `/tmp/perf003-p7-closure-core.log`.
- Repository-pinned `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta -p phalcom-compiler-artifact`: **28 type-meta PASS, 25 artifact PASS** (10 container + 15 metadata). Log `/tmp/perf003-p7-closure-model-tests.log`.
- Repository-pinned `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic artifact`: **3 PASS**, 2,175 filtered; all three are parameter type-graph checks, not exhaustive semantic-primary parity. Inherited unused TypeHierarchy test import warning. Log `/tmp/perf003-p7-closure-semantic-tests.log`.
- `git diff --check`: PASS before this record.

The new discriminators live outside the repository in `/tmp/perf003-wave4-review-probe/src/bin/closure.rs`; output `/tmp/perf003-p7-closure-probe.log`. They exited 0 while asserting/observing the defective behavior. Re-run with cleared flags, the worktree's target directory, and `cargo run --manifest-path /tmp/perf003-wave4-review-probe/Cargo.toml --bin closure --offline`. This standalone harness has its own dependency resolution and an explicit mio os-ext feature, so distinguish it from repository-locked suite evidence. Counting allocations were scoped by before/after measurements around the individual operation in this single-threaded probe; allocator-internal bookkeeping/RSS was not measured.

No workspace/release checks were run: demonstrated contract failures already block closure. No existing proof was silently promoted. No source-denied/independent-process determinism or structural locality gate is claimed.

## 9. Required disposition

Return P7 ownership to IN_PROGRESS / PARTIAL. Retain focused-test evidence for the surfaces actually exercised; remove claims of exhaustive parity, complete allocation enforcement, all-R1–R11 resolution and full P7 closure. Finish the current correction amendment and its discriminators before certifying P8 takeover. Independent P8 exploration is a separate scope decision, not evidence of P7 completion.

No additional declaration of intent or renamed helper closes these findings. Required next evidence is actual restored owner parity, allocator-request bounds, exhaustive fail-closed queries, independent provenance, source-denied fresh-process compilation, cross-process/history determinism, and P5/P6 structural/performance non-regression.
