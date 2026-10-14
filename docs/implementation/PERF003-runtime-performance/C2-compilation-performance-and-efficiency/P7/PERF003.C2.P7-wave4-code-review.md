## Correction continuation status — 2026-10-14

User authorized all R1–R11 corrections, then requested a checkpoint/handoff before usage exhaustion. Corrections are PARTIAL; findings are not collectively closed. R6 ABI/package/capability guards, R9/R10 writer boundaries and R11 bounded acquisition have source changes and earlier focused evidence; R8 resolver wiring and R2 rooted export have source changes; R1 typed metadata construction/ledger is started. R3/R4/R5 and full R7 remain open. See the [current handoff](PERF003.C2.P7-handoff.md) for the exact current failure, latest-tree versus earlier test evidence, and unimplemented acceptance. Original findings/probes below are the red baseline, not a claim that unchanged old behavior remains at every edited location.

# PERF003.C2.P7 Wave 4 implementation audit and code review

Date: 2026-10-14. Review verdict: **CHANGES REQUIRED**. The claim “Wave 4 fully implemented and verified” is not supported.

Scope: live dirty worktree on `perf/universe-characterization`, HEAD `64a9b65e1d6de8b386b3181b7b4d68789099c1b8`. Reviewed T10, shared graph orchestration, T7.B/T8/T9 and T11/T12 against the adopted P7 contracts. No implementation fixes, staging, commits, new subagents, or lifecycle promotions were performed. Existing source/document edits were preserved.

## Authority and careful reading

The implementer must read these carefully before fixing findings, especially the adopted prerequisite incident and the fixed ownership/identity/semantic contracts:

- [AGENTS.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/AGENTS.md)
- [docs/workflow/implementation-record-lifecycle-convention.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/workflow/implementation-record-lifecycle-convention.md)
- [docs/workflow/shared-consultation-escalation-protocol.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/workflow/shared-consultation-escalation-protocol.md)
- [docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/CHECKPOINT.md)
- [docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-wave4-prerequisite-incident.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-wave4-prerequisite-incident.md)
- [docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-remaining-execution-plan.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-remaining-execution-plan.md)
- [docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md)
- [docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-handoff.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-handoff.md)
- [docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-walkthrough.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-walkthrough.md)
- [docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-implementation-continuation-guide.md](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-implementation-continuation-guide.md)

## Findings

### R1 [P1] Metadata allocation preflight itself allocates outside the budget

Location: [phalcom-type-meta/src/encode.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-type-meta/src/encode.rs:318).

The scanner deserializes every map key into String before comparing required_bytes with shared_budget. Top-level/header keys are not even included in string_bytes. A counting allocator probe with a 1 MiB unknown top-level key and shared_budget = 0 observed 1,048,724 allocated bytes before rejection; the budget remained zero. Budget rollback is not proof that allocations were prevented. Nested arrays also use a fixed 32-byte estimate, and scratch accounting only covers two maps despite allocations for actual_kinds, lambda_param_kinds and cloned parameter_kinds. HashMap control/alignment and Vec growth are not accounted exactly.

Required correction / discriminator: Use a bounded scanner that borrows unescaped keys and bounds/charges escape scratch, then budgeted collection/string construction with checked capacity and validator scratch accounting. Test actual allocation ceilings, including rejection paths; retain rollback checks.

### R2 [P1] Export preserves native allocation order and unreachable history

Location: [phalcom-core/src/modules/canonical_artifact.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/canonical_artifact.rs:143).

The builder selects TypeExportContext::new_full. ExportIds::new_full assigns IDs and traversal order from 0..arena.len(), rather than stable roots. The builder also closes StaticExportContext without registered roots; its empty-root branch exports the entire values arena. Equivalent semantic images with reversed allocation order or unreachable additions can therefore produce different bytes. Rebuilding twice from one cached image does not test this contract.

Required correction / discriminator: Reuse one stable-owner-rooted, closed type/static export context across all sections; omit unreachable history while retaining referenced historical identities. Add independent fresh-store/process tests with reversed allocation order and unreachable-node additions.

### R3 [P1] Loaded semantic snapshot contains empty required authority

Location: [phalcom-semantic/src/artifact/import.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/artifact/import.rs:920).

The importer initializes field_signatures, trait_headers, constructor_initializations, derived_conformance_rules, associated_surfaces, type_member_declarations, static_definitions, static_constants and module_static_bindings as empty. Imported operational_templates and three owner indexes are returned separately and discarded by the core loader, which publishes only parts.snapshot. Callable analyses are rebuilt with empty expression/binding/flow/dependency maps and Complete status. The completeness guard only tests that selected families are nonempty, allowing these holes to pass. The detailed plan explicitly requires these compiler-consumed families.

Required correction / discriminator: Finish the field-to-wire/derived/omitted manifest and restore every required family into its owning snapshot table, preserving actual callable analysis facts where consumed. Wire derived indexes into their real owners. Require per-family cold/import parity and at least field, associated/static, generic and constructor consumers.

### R4 [P1] Conformance import invents target and witness completeness

Location: [phalcom-semantic/src/artifact/import.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/artifact/import.rs:861).

ConformanceContribution.target is assigned the trait declaration rather than the conforming target declaration. source_module is similarly replaced by the trait module; generic_signature is dropped and authorized/eligible are forced true. Witness reconstruction ignores ordered_selected_callables and slot_layout_fingerprint, drops trait arguments and generic binders, supplies empty requirements/requirement_views/associated_type_plan, sets fingerprint zero and labels the result Complete. This loses canonical evidence and can change conformance lookup/dispatch for generic and associated-type implementations.

Required correction / discriminator: Persist the original conformance target, provenance, generic signature, eligibility and complete witness selection/associated-type evidence. Reconstruct and validate the real owner product; do not manufacture Complete witnesses. Add generic conformance, trait-default and associated-type round trips against cold authority.

### R5 [P1] Query reconstruction silently discards roots and substitutes zero fingerprints

Location: [phalcom-semantic/src/artifact/query.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-semantic/src/artifact/query.rs:143).

Unknown owners resolve to None and are skipped. Advertised supported kinds including module interfaces/imports, FieldSignature, TraitHeader, AssociatedSurface and TypeMemberDeclaration have no product reconstruction branch and disappear. Dependencies with unresolved keys are silently omitted. aggregate_structural_fingerprint handles only four kinds; other reconstructed products and input fingerprints receive zero. Data/Enum/TraitSurface/CallableDefinition use expect without proving that a resolved declaration/signature has that specific product, so a syntactically valid wrong-kind owner can panic. Closure validation before reconstruction does not prove parity of the reconstructed graph.

Required correction / discriminator: Reconstruct every advertised reusable kind from validated primary authority and canonical typed owner identities; reject unresolved/wrong-kind roots with typed errors. Preserve all retained edges, derive real product/input fingerprints, and validate closure after materialization. Add missing-owner, wrong-kind, supported-kind census and invalidation parity tests.

### R6 [P1] Loader accepts incompatible header identity and synthesizes capabilities

Location: [phalcom-core/src/modules/canonical_artifact.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/canonical_artifact.rs:308).

The loader copies semantic_model_version, stable_identity_schema, module_semantic_abi, semantic_artifact_abi, native_surface_schema_version and type_metadata_schema_version into compatibility identity without comparing them to reader constants. query_fingerprint_abi is not checked. It returns compiler_only_v1 capabilities regardless of header capabilities and passes meta_v1.package_digest as its own expected value. A probe rebuilt the container with valid digests but semantic_model_version, module_semantic_abi and query_fingerprint_abi each increased by 100; loading succeeded. Container integrity is not ABI compatibility. The builder also uses package_digest = [7;32] and zero source/interface/semantic/query/metadata identity digests.

Required correction / discriminator: Validate supported package identity, all relevant ABI/schema constants, actual capabilities, and package/provenance relationships before materialization. Derive expected metadata identity independently. Populate/verify compatibility digests from canonical evidence. Test well-formed incompatible containers separately from corruption.

### R7 [P1] Artifact loading resets allocation budgets between sections

Location: [phalcom-core/src/modules/canonical_artifact.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/canonical_artifact.rs:335).

MODULE_WORLD, SEMANTIC_WORLD and QUERY_IMAGE decode independently initialize their own limits/budgets. Semantic import and query reconstruction allocate further stores/maps. Only afterward does the loader create a fresh max_decoded_section_bytes budget for metadata. Total native allocation can exceed the artifact-wide cap even when every section is individually under it. Metadata preflight is then charged again by finish_loaded_universe_telosc after decode_bundle already charged it.

Required correction / discriminator: Thread one transaction-scoped budget through every section decoder, relocation/import, index/query reconstruction and metadata construction; charge each owned allocation once and publish only on success. Test aggregate overflow where each section individually fits, plus retry and exact boundary behavior.

### R8 [P1] Explicit loaded-image import path still acquires Universe source

Location: [phalcom-core/src/modules/compile.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/compile.rs:2290).

The standalone and project loaded-image paths construct ModuleResolver with FilesystemSourceProvider and call the existing discover_reachable_dependencies. That helper invokes resolve_import before checking target_unit.id.project.is_universe(). Universe resolution validates package exposure via load_package_surface_for -> load_interface -> load_parsed and obtains universe_location from the source provider. Installing external modules only later in ModuleLinker does not stop these source reads/parses. The inline print(42) test has no imports and cannot reveal this.

Required correction / discriminator: Resolve canonical Universe imports/package surfaces using the loaded module authority before contacting any provider. Keep source discovery for user/dependency modules. Prove with a fresh child process denying all Universe providers and compiling a novel user module with explicit Universe imports.

### R9 [P1] Writer can replace a valid artifact with invalid supplied bytes

Location: [phalcom-core/src/modules/canonical_artifact.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/canonical_artifact.rs:230).

BuiltUniverseTelosc has public mutable bytes and summary. The writer checks only summary.capabilities.portable_executable and then writes the bytes without container/section/cross-section validation. A probe installed a valid output, changed built.bytes to [1,2,3] while preserving its summary, and the writer successfully replaced the valid file with those bytes. build_universe_telosc self-validation only parses the container and does not use loader owners to validate its section relationships.

Required correction / discriminator: Use an opaque validated build product or independently validate the actual bytes with loader validation owners before replacement. Derive capabilities/summary from the bytes, not mutable caller claims. Test invalid bytes, valid container with invalid references and prior-output preservation.

### R10 [P2] Temporary filename collides between writes in the same process

Location: [phalcom-core/src/modules/canonical_artifact.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/canonical_artifact.rs:238).

The claimed unique temporary filename consists only of destination name and process ID. Concurrent writes to the same destination therefore use the same path, and File::create truncates an existing file. They can interleave contents, rename a still-open shared file, or remove another call’s temporary path on error. Existing tests only exercise one successful write.

Required correction / discriminator: Create a genuinely unique same-directory temporary file with exclusive creation and task-owned cleanup. Test concurrent publication and injected write/validation/rename failure with an existing destination.

### R11 [P2] Disk loader checks file size only after reading the whole file

Location: [phalcom-core/src/modules/canonical_artifact.rs](/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization/phalcom-core/src/modules/canonical_artifact.rs:256).

fs::read allocates for the complete input before TeloscContainerView::parse can apply max_artifact_bytes. An arbitrarily large file can consume memory despite a tiny configured cap. The in-memory byte-limit test does not exercise acquisition.

Required correction / discriminator: Bound file acquisition before allocation and enforce the cap while reading to handle file growth/races. Test an oversized file with a tiny cap and observe bounded allocation.

## Verification actually completed

- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_canonical_artifact`: PASS, 4 selected tests, 964 filtered, 41.61 seconds.
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta -p phalcom-compiler-artifact`: PASS, 28 type-meta tests and 25 artifact tests (10 container + 15 metadata).
- `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-compiler-artifact -p phalcom-semantic -p phalcom-core -p phalcom-lsp --lib`: PASS with an inherited unused TypeHierarchy import warning in LSP backend.rs:59; the reported “clean” check is overstated.
- `git diff --check`: PASS before review-record creation.

The new metadata test named `representative_universe_metadata_decodes_cleanly_under_default_limits` uses `real_section()`, whose fixture has one Type kind, one Unit type, and no declarations/callables. It is not real Universe metadata. The core load tests do use real canonical metadata, but they do not measure allocations. The “generic_parameters_and_static_domains” test constructs parameters only and checks the estimator, not actual scratch allocation.

The standalone review probe lives under `/tmp/perf003-wave4-review-probe`, outside repository implementation files. It instruments allocation and constructs correctly rehashed incompatible containers. Its isolated build needed an explicit mio os-ext feature; this is probe plumbing, not an active-plan defect. It uses the pinned toolchain with cleared compiler flags; its standalone dependency resolution is separate from repository-locked test evidence.

The completed probe exited 0, with assertions confirming these undesirable behaviors:

```text
zero_budget_decode: error=true, allocated_bytes=1048724, unchanged_budget=0
incompatible_abi_accepted=true
missing_query_owner_accepted=true, retained_roots=0
type_graph_changes_with_allocation_order=true
invalid_writer_input_accepted=true, replaced_prior_output=true
```

The type-order discriminator creates two fresh TypeStores with identical A/B parameter owners registered in opposite order, projects both through new_full, and compares encoded TypeGraphV1 bytes. The missing-owner discriminator supplies a CallableSignature reusable root with owner `missing-owner`; reconstruction returns success and zero retained rows. Incompatible-header mutation is followed by container reconstruction to recompute all container digests, separating compatibility validation from corruption rejection.

Probe source: [main.rs](/tmp/perf003-wave4-review-probe/src/main.rs). Completed output: [probe log](/tmp/perf003-wave4-review-probe.log). Re-run from the worktree with cleared flags and `CARGO_TARGET_DIR` pointing at its target directory using `cargo run --manifest-path /tmp/perf003-wave4-review-probe/Cargo.toml --offline`.

No broad crate/workspace release certification, source-denied child-process proof, full semantic/query parity, actual allocation-cap enforcement or cross-process determinism was established. No such PASS is claimed.

## Completion and next action

Focused tests pass, but the implementation does not satisfy the fixed acceptance contracts. Keep full Wave 4 completion open; focused test success must not be promoted to completed T10/T7.B/T8/T9/T11/T12 acceptance. The records currently contain conflicting historical and new completion claims; reconcile them against these findings after the responsible architect/implementer adopts the correction scope.

Fix allocation ownership, stable graph orchestration and primary/query parity before treating writer/loader composition as accepted. Preserve source-free boundaries, fail-closed typed validation, canonical semantic authority, stable identities and transaction rollback. Architecture-affecting fixes must follow the existing consultation protocol; this review does not independently authorize implementation or a redesign. Reuse the existing adviser when consultation is required, and do not spawn new subagents.
