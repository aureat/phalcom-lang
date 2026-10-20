# PERF003.C2.P7 — Audit Slice 1 Report

**Date:** 2026-10-15  
**Repository:** [`aureat/phalcom-lang`](https://github.com/aureat/phalcom-lang)  
**Branch:** `perf/universe-characterization`  
**Immutable audited source revision:** [`2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a`](https://github.com/aureat/phalcom-lang/commit/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a) (pushed at 2026-10-15 12:25:39 UTC).  
**Previous pushed revision:** `c64dfc233d0ee2e2d9c3529ac6344c7180e5aa05`.  
**Lifecycle assessment:** `IN_PROGRESS / PARTIAL / UNVERIFIED`; P8 takeover **not authorized**.  
**Audit mode:** Read-only static source inspection through the live GitHub connection, comparative reading of the supplied P7 handoff, remaining execution plan, walkthrough, continuation guide, and checkpoint. **No Cargo tests or benchmarks were executed by this auditor.** Recorded passes/failures in implementation records are evidence reported by prior implementers, not independently rerun evidence.

> **Slice boundary:** This first slice audits repository baseline and the end-to-end P7 compiler-artifact viability chain: exporter, semantic-world decoder/validator, semantic materializer, completeness gate, initial compiler entry modes, directly connected owner fidelity, relevant allocation/performance risks, and acceptance-test coverage. It is **not** an exhaustive audit of the entire TypeStore codec, parser, module linker, native metadata codec, static-value graph, frozen graph implementation, query dependency closure, or all branch-wide test targets. Those require subsequent audit slices.

## 1. Executive conclusion

**At this revision, P7 cannot build, validate, load, and publish a complete usable `universe.telosc` using its normal public API.** The failure is structural, not merely a shortage of test coverage. A real canonical snapshot has callable analyses and other required owner families, but `export_semantic_world` rejects them. Even if export were completed, the semantic-world budgeted decoder **always** fails at a deliberately unconditional `UnaccountedBoundary`; independently, semantic import contains its own unconditional `UnaccountedBoundary`. And even if these safety stops were replaced with proper accounting, the current importer materializes incomplete/defaulted semantic owner products and an empty callable-analysis table, contradicting the completeness predicate on the imported snapshot.

The latest commit is substantial and valuable: it adds tagged call specializations and denotation transport foundations, exact-budget primitives and tests, broader root registration, frozen flow-graph import, provenance/ABI changes, and compiler integration work. **Implementation volume is not acceptance evidence.** Preserve the fail-closed barriers until their underlying contracts are implemented; bypassing them would transform a known incomplete artifact into plausible but semantically wrong compiler authority.

The user's intended closure is a compiler that (1) produces a valid canonical Universe compilation artifact, (2) loads it in a fresh process without Universe source acquisition, (3) compiles real user projects and packages through a small overlay, including capabilities exercised by `parse`, `similarity`, and `slotmap`, and (4) demonstrates P5/P6 locality and measurable cold/warm speed improvements. P7 is compiler-image persistence, **not** P8's separately planned portable-executable image; it need not smuggle VM-ready executable Universe content into `.telosc` to satisfy its compiler-image contract. It must, however, produce compiler outputs usable by the existing execution pipeline and validate meaningful package code, rather than only `print(42)`.

## 2. Baseline, confidence and evidence classification

The remote head at the start of this audit was `2939e9d` with parent `c64dfc23`. GitHub's compare API reports **one additional commit modifying approximately 80 files**, including 1,400+ added lines in semantic export, 1,700+ in semantic import, 1,000+ in root registration, 1,200+ in semantic-world DTOs, significant tests and corrections, and a checked-in Rust compiler ICE report. The attached handoff identifies `c64dfc23` as its HEAD and describes inherited uncommitted changes. The bulk of that material is now present in the pushed `2939e9d` source tree. **A remote GitHub read cannot establish the current local worktree's cleanliness or any newer unpushed modifications.** Do not silently treat the handoff's earlier dirty-worktree warning as a claim about today's local status.

| Evidence level | Meaning in this report |
|---|---|
| **CONFIRMED—source** | Verified directly in the pinned pushed Rust source or repository tree; it does not imply a runtime reproduction was executed. |
| **CONFIRMED—logical** | Direct consequence of an inspected unconditional branch or contradictory constructor/predicate. |
| **REPORTED—test** | A prior implementer recorded a passing/failing test; not rerun by this audit. |
| **RISK / UNVERIFIED** | Plausible performance, safety, or behavior issue needing a discriminating test or broader call-path study. |

### Direct code-chain map

```text
build_universe_telosc(view)
  -> register_semantic_world_roots / close_graph
  -> export_semantic_world
       -> validate_semantic_world_export_coverage  [stops real callable/other owner families]
  -> encode SEMANTIC_WORLD / QUERY_IMAGE / METADATA
  -> assemble .telosc

load_universe_telosc[_from_bytes]
  -> parse container, check reader compatibility and section provenance
  -> decode MODULE_WORLD, materialize external modules
  -> SemanticWorldV2::decode_with_allocation_budget
       -> validate_with_allocation_budget
            -> require_semantic_validation_allocation_accounting [always errors]
  -> validate_and_import_semantic_world_with_allocation_budget
       -> require_semantic_import_allocation_accounting     [always errors independently]
       -> validate_and_import_semantic_world_inner          [not reached by public load]
            -> incomplete/default semantic owner tables
            -> has_source_free_artifact_contents           [cannot pass current empty tables]
  -> reconstruct QUERY_IMAGE, METADATA, formal image, publish Arc

write_universe_telosc(path, built)
  -> load_universe_telosc_from_bytes(built.bytes)            [currently also blocked]
  -> only then create unique same-directory temp + persist
```

Source: [`canonical_artifact.rs#L129-L157`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L129-L157), [`canonical_artifact.rs#L249-L265`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L249-L265), [`canonical_artifact.rs#L318-L355`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L318-L355).

## 3. Findings register, ordered by closure impact

| ID | Severity | Evidence | Finding / impact | Closure ownership |
|---|---|---|---|---|
| **S1-01** | **BLOCKER** | CONFIRMED—logical | Budgeted semantic-world decode unconditionally fails validation. | `phalcom-compiler-artifact` semantic DTO validator; R1/R7 |
| **S1-02** | **BLOCKER** | CONFIRMED—logical | Independently, semantic import unconditionally fails before materialization. | `phalcom-semantic` import; R1/R7 |
| **S1-03** | **BLOCKER** | CONFIRMED—source | Artifact exporter rejects all nonempty canonical callable analysis tables and many other real primary owner families. | semantic export; R3/R4/T7.B |
| **S1-04** | **BLOCKER** | CONFIRMED—logical | Actual import constructs empty required snapshot products, yet completeness requires them nonempty. | semantic import, snapshot, module products; T7.B/T8 |
| **S1-05** | **HIGH** | CONFIRMED—source | Importer fabricates or erases semantic facts in callable definition, traits, surfaces, and witness plans. | semantic owner-specific codecs; R3/R4 |
| **S1-06** | **HIGH** | CONFIRMED—source | Field manifest advertises wire coverage not implemented by the current callable-analysis DTO; export/import are deliberately non-parity. | manifest, DTO, roots, consumer reconstruction |
| **S1-07** | **HIGH** | CONFIRMED—logical | Full native allocation accounting is not optional polish: current source identifies uncharged validation and materialization paths, and the loader must refuse to cross them. | DTO validator, TypeStore, semantic owner tables, publication; R1/R7 |
| **S1-08** | **HIGH** | CONFIRMED—source + unverified test | Core integration suite cannot yet certify loading, writing, deterministic bytes, or source-free compilation; several tests require build success. | core artifact tests; T11/T12/T13–T15 |
| **S1-09** | **HIGH** | CONFIRMED—source | Existing integration's sole positive user compilation example is an import-free inline literal; the closure target requires real project/package import and consumer proof. | core test harness, package fixtures; R8/G7.3/G7.8 |
| **S1-10** | **MEDIUM** | CONFIRMED—repo tree | `packages/similarity/project.toml` declares `similarity.main`, while its checked-in `src` directory has only `traits.ph`. This is a fixture/packaging inconsistency to reconcile before declaring package acceptance. | similarity package / test fixture |
| **S1-11** | **MEDIUM** | RISK / UNVERIFIED | No independent end-to-end measurement of allocation requests, artifact-load footprint, determinism across fresh processes, or actual speedup has been established. | performance/acceptance harness; G7.7–G7.11 |
| **S1-12** | **LOW** | CONFIRMED—repo tree | A generated Rust nightly internal-compiler-error dump was committed in repository root; not evidence of a Telos semantic defect but a reproducibility/hygiene item. | build environment / repository hygiene |

### S1-01 — Unconditional failure in semantic-world transactional decode

**Evidence.** [`phalcom-compiler-artifact/src/semantic/world.rs#L609-L630`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L609-L630) calls `validate_with_allocation_budget` before committing the decoded-section child transaction. [`world.rs#L654-L668`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L654-L668) invokes `require_semantic_validation_allocation_accounting()` before its normal validation body. [`world.rs#L1533-L1536`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L1533-L1536) implements that prerequisite as an unconditional `Err(AllocationError::UnaccountedBoundary.into())`.

**Consequence.** No valid `SemanticWorldV2`, including a minimal otherwise-valid wire image, can complete the public budgeted decoder. The normal core load path unconditionally calls it. Successful isolated `SemanticWorldV2::validate` or `encode` tests cannot contradict this because those use a different path, outside the load transaction. The stop is a correct fail-closed response to missing accounting; the defect to fix is the still-unaccounted validator allocation boundary, **not** the presence of the guard.

**Required correction.** Inventory every `validate_inner` delegate's allocations: scratch identity/duplicate indexes, sorted owners, nested validations, temporary strings, vectors, maps/sets and error construction. Use charged, fallible storage (or borrow validated wire slices, where safe), with checked layouts and monotonic requested-byte debits. Ensure stage/whole budgets are genuinely independent. Replace the unconditional stop only when counting-allocator tests verify exact requests over representative real and malicious payloads. Test exact-fit, `N−1`, aggregate overflow, no commit on failure and a successful validated semantic world.

### S1-02 — Second unconditional failure in semantic materialization

**Evidence.** [`phalcom-semantic/src/artifact/import.rs#L1794-L1827`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1794-L1827) enters `validate_and_import_semantic_world_with_allocation_budget`, invokes `require_semantic_import_allocation_accounting()` **before** creating the stage child transaction, and unconditionally returns `UnaccountedBoundary`. The following inner implementation calls `TypeStore::import_type_graph`, structural and static owner materializers and owning-table rebuilds without a complete shared budget seam: [`import.rs#L1829-L1860`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1829-L1860).

**Consequence.** Fixing only the decoder in S1-01 yields another deterministic failure, not a usable artifact. Disabling only the import guard would violate the adopted one-transaction allocation guarantee. Although some specialized helpers already have exact accounting (TypeKnowledge, tagged denotations, frozen CFG), the whole graph—TypeStore allocations, native owner indexes, map nodes, strings, `Arc`s, scratch, and published structures—remains outside certified coverage.

**Required correction.** Plan the full materialization allocation census **before** the final owner rewrite, because owner storage and budget-safe reconstruction are interdependent. Avoid inserting into unbudgeted `BTreeMap`/`HashMap` and then measuring estimates. Where necessary, adapt frozen imported owners to budgeted sorted storage or use a proven fallible native-store charging seam; ensure existing read APIs and parent-overlay locality survive. Measure actual allocator requests independently rather than asserting that `AllocationBudget::requested` equals itself.

### S1-03 — Publisher explicitly refuses real required semantic authority

**Evidence.** [`phalcom-semantic/src/artifact/export.rs#L1841-L1848`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L1841-L1848) calls `validate_semantic_world_export_coverage` as the first export step. [`export.rs#L2189-L2236`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L2189-L2236) rejects any nonempty callable-analysis table and separately rejects nonempty static case catalogs/signatures, static definitions/constants, module static bindings, derived conformance rules and trait dispatch. It also rejects witness plans that carry nonempty requirement views, invalid explicit members, certain conditional selections, associated failures or non-Complete status.

**Consequence.** This is consistent with the reported core test that reaches this guard after proving no internal callable incidents. It **does not** establish that source-free loading is almost complete; the producer does not yet have a lossless portable representation for genuine canonical compiler authority. These rejected families are not optional if their consumers use them.

**Required correction.** Complete field-level owner transport, including `CallableAnalysis` expression/binding judgments, CFG/flow/exit state, match/pattern and dispatch resolutions, calls and generic evidence, dependencies, diagnostics/explanations, formal compiler attachments; all static/derived/trait-dispatch authority; exact witness selections, views, invalidity and completeness. Resolve the existing checked-static-body `StaticApplication` owner discrepancy at the real `CheckedStaticBody` boundary before new wire design. **Never remove the coverage guard ahead of its parity tests.**

### S1-04 — Imported snapshot cannot satisfy its own completeness gate

**Evidence.** In [`import.rs#L2229-L2303`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2229-L2303), the snapshot is assembled with an empty callable-analysis table, new empty derived-conformance and trait-dispatch indexes, empty static case tables/definitions/constants/bindings, and `ModuleQueryProducts::empty()`. The importer subsequently checks `has_source_free_artifact_contents()` before promoting the snapshot to Complete. [`snapshot.rs#L773-L823`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/snapshot.rs#L773-L823) requires nonempty callable analyses plus genuine body facts, derived/trait/static families, and both `module_products.unlinked` and `.linked` for the canonical image. [`canonical_image.rs#L788-L806`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/canonical_image.rs#L788-L806) checks completeness again before making the image visible.

**Consequence.** Even if the two allocation guards and the export guard vanished, the loader's present constructor still could not produce a complete published canonical snapshot. In particular, separately loaded `CanonicalExternalModuleWorld` does not automatically populate the snapshot's required `module_products`; no later assignment in `from_loaded_artifact` repairs those fields. This is a producer/consumer ownership mismatch to repair explicitly, not a reason to weaken the predicate or synthesize fake entries.

**Required correction.** Build a source-free full snapshot from exact primary authorities. Attach module/query products through the accepted module owner and semantic snapshot relationship; do not duplicate an independent module authority. Reconstruct all indexes from primary tables, validate every required product/reference and its consumer-facing identity, and only then promote status. Test the intended complete image and deliberately absent individual required owners.

### S1-05 — Current importer contains real lossy placeholders

This finding is **confirmed even though guards presently prevent normal publication**. Those guards cannot be removed until the following code is replaced:

1. **Callable definition origin/context:** [`import.rs#L1867-L1887`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1867-L1887) reconstructs every signature into `EffectiveCallableDefinition` using `PrimaryDeclaration`, `source_member_index: 0`, `inherent_body_context: None`. These are not justified by an actual owner-held accepted-definition record.
2. **Trait member contracts:** [`import.rs#L1896-L1971`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1896-L1971) creates associated-type requirements with `KindId(0)` and default source spans, reconstructs required/default members only when a signature lookup happens to succeed, assigns Public visibility and synthetic source locations, and loses original default-source authority. Partial lookup must not silently skip a required member.
3. **Dispatch callable type knowledge:** [`import.rs#L1974-L2006`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1974-L2006) fills callable entries with assumed `TypeId(0)` instead of relocated exact callable knowledge. This changes semantic evidence and can alter lowering/inference even if stable callable IDs are correct.
4. **Witness/conformance semantics:** [`import.rs#L2030-L2170`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2030-L2170) replaces real invalidity, requirement views, diagnostics, failure reports and completeness detail with empty/default or generalized synthetic values. The export guard rejects a portion of these loss cases, but not all; a general producer-to-loaded-consumer field parity audit is necessary.

**Impact.** A snapshot can look structurally plausible while carrying wrong specialization, visibility, conformance, witness, or compiler lowering facts. These issues are correctness blockers—not mere fidelity improvements. Replace per-owner defaults with first-class wire fields or accepted, independently provable derivations; do not manually force `Complete`, authorization, unknown/assumed types or zeroed evidence.

### S1-06 — Callable manifest and wire record disagree

The explicit [`CALLABLE_ANALYSIS_FIELD_MANIFEST`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/manifest.rs#L29-L67) marks expression/binding facts, pattern/match resolution, entry/exits, return validation, diagnostics/explanations and dependencies as `Wire`. The live [`CallableAnalysisRecordV1`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L298-L327) only carries the body identity/range/status, graph, termination, constructor attachment, expression count, selected denotations/call-specialization rows and binding origin/formal attachments. The partial export constructor is visible at [`export.rs#L1140-L1162`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L1140-L1162); the importer deliberately rejects every nonempty analysis at [`import.rs#L630-L637`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L630-L637).

The manifest is a **target contract, not a census of implemented transport**. Carry separate columns for `required owner field`, `wire disposition`, `producer`, `wire DTO`, `validation`, `materializer`, `compiler reader`, and `passing parity discriminator`; do not count a manifest row as completed because it says `Wire`. `roots.rs` now contains meaningful type-root work, but roots alone do not materialize an owner graph.

### S1-07 — Allocation accounting and memory shape remain release-relevant

The latest work appropriately introduces typed allocation failures, `AllocationBudget` helpers, sorted-vector containers, budgeted module scratch and frozen CFG import. The two unconditional blockers nevertheless confirm that total requested bytes are **not** fully accounted for. Further, the unreachable present importer makes ordinary `HashMap`/`BTreeMap` containers, strings, owner-index vectors, primary table insertions, clones, type relocation and publication `Arc`s without a complete charged native construction path: see [`import.rs#L1867-L1888`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1867-L1888), [`import.rs#L2042-L2173`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2042-L2173), [`import.rs#L2218-L2304`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2218-L2304).

**Performance risk—not measured in this audit:** reconstructing multiple equivalent owner indexes or retaining detatched debug-string-index tables alongside actual owner maps can inflate both peak memory and cold-load latency. Investigate underlying table ownership and shared immutable slices rather than assuming every map clone is a full deep copy; `im`/`Arc`-backed stores may share structure. Do not prematurely label a specific clone O(N) without profiling.

For a real Universe artifact, measure requested allocator bytes from file acquisition through parsing, materialization, cross-owner validation, reconstructed queries and publication; independently measure steady-state retained bytes, peak RSS and user compilation latency. **These are three different metrics.** The adopted allocation guarantee is on cumulative allocator-requested bytes, not a claim of bounded RSS.

### S1-08 — Existing tests are not independent closure evidence

[`phalcom-core/tests/core/modules/canonical_artifact.rs#L8-L112`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/tests/core/modules/canonical_artifact.rs#L8-L112) defines an ambitious build, second-build determinism, writer, disk/memory reload and inline compilation test. But it starts with `build_universe_telosc(...).expect(...)`, and the canonical export guard stops that call. The supplied handoff reports this precise focused test **failed** at export; this audit did not independently rerun it. Other tests (`corrupted_artifact`, `portable-executable writer rejection`, load timing, cumulative budget and rehashed-header matrix) also construct a full artifact before reaching most of their intended assertions: [`canonical_artifact.rs#L115-L166`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/tests/core/modules/canonical_artifact.rs#L115-L166), [`canonical_artifact.rs#L191-L260`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/tests/core/modules/canonical_artifact.rs#L191-L260), [`canonical_artifact.rs#L296-L355`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/tests/core/modules/canonical_artifact.rs#L296-L355).

Conversely, the synthetic foreign-catalog-header test does not require a genuine image and can pass independently; the invalid-bytes writer-preservation test can also pass without any valid published artifact. These are valuable negative-path regressions, but **not substitutes for** owner-complete publication and loader acceptance. Once a full artifact can build, rerun actual selectors and record counts/exits; add byte-level rehash and mutation to target the intended stage rather than merely obtaining `is_err()` upstream.

### S1-09 — Package/end-user acceptance is currently under-tested

The compiler exposes the important explicit-image boundary: [`LoadedUniverseArtifact::compile_user_entry`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L110-L126) calls `compile_entry_selection_with_image`. The analyzer handles `EntrySelection::Project`, standalone `Package`, `Module`, and `Inline`: [`compile.rs#L2305-L2469`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/compile.rs#L2305-L2469). Project/package mode reaches `discover_and_analyze_with_image`, whose resolver is seeded from the loaded external module authority: [`compile.rs#L2590-L2638`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/compile.rs#L2590-L2638). This is the right architecture but an **unverified real consumer boundary**.

The current full artifact test compiles only `EntrySelection::Inline("print(42);")`, which has no explicit import, no package manifest, no dependency graph with user modules, no generic data structure, no contextual enum variant, no trait/associated type use, and no `parse`/`similarity`/`slotmap` functionality. It also runs in the same process as the canonical image construction. To certify the requested end state, use a fresh process and an actual loaded `.telosc` with Universe source providers explicitly denied, compile real project fixtures and novel user modules, and assert both semantic/behavioral parity and **zero** canonical reparses, relinks, body analyses and presentation rebuilds. The standard runtime pipeline should also execute at least a compiled package smoke test if executable code is the declared product; this does not require adding P8 portable executable sections.

### S1-10 — Existing target-package fixture discrepancy

At pinned HEAD, `packages/parse/project.toml` declares entry `parse.main` and has `src/main.ph`; `packages/slotmap/project.toml` declares `slotmap.main` and has `src/main.ph`. [`packages/similarity/project.toml`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/packages/similarity/project.toml) declares `entry = "similarity.main"`, while its checked-in `src` directory lists only `traits.ph`. **The manifest/source mismatch is verified; the exact project-resolution failure has not been executed**. Resolve whether similarity is an intentionally library-only project with a stale entry or a missing executable module before using it as an executable-project acceptance case. A thin consuming fixture importing `similarity.traits` may be more appropriate if the library is meant to be non-executable.

Also distinguish existing project-style `packages/*/project.toml` layouts from the analyzer's flat `EntrySelection::Package` convention, which requires `package.ph` and `main.ph` at the specified package directory root. Do not test a project-style library under the wrong entry mode and diagnose the resulting layout error as a `.telosc` bug.

### S1-11 — No attributable cold/warm speedup demonstrated yet

The P7 checkpoint aims to remove the Universe-dominated ~45-second one-shot compilation floor and reach a small user-overlay cost. A passing synthetic codec, graph allocation probe or one in-process compilation cannot establish that. Required evidence, in order: successful source-built baseline; deterministic artifact build; fresh-process load and source-denied compile; correct canonical source-work cardinalities; cold load time and footprint; warm repeated overlay compile speed and locality; then package-level performance comparisons. Keep artifact build time, artifact load time, loaded-image retained memory, first user overlay and warm overlay separate in measurements. Avoid marketing the aspirational `<200 ms` optimized target as an achieved benchmark.

### S1-12 — Checked-in Rust compiler ICE artifact

The latest commit includes [`rustc-ice-2026-10-15T09_58_51-58574.txt`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/rustc-ice-2026-10-15T09_58_51-58574.txt), indicating a Rust 1.99.0-nightly incremental verification panic while checking an `Arc<CallableSignatureTable>` Send obligation in `canonical_image::IMAGE`. This **does not prove a Telos defect**, but warrants pinning the toolchain/build flags for reproducibility, investigating whether clean `CARGO_INCREMENTAL=0` builds pass, and removing transient crash logs from the production tree under normal repository hygiene procedures. No Rust build was rerun here.

## 4. What genuinely appears implemented versus what remains unverified

| Boundary | Observed current source | Audited conclusion |
|---|---|---|
| Container + native-catalog V2 identity | Explicit schema, fingerprint, reader comparisons; negative test for foreign catalog | **Implemented in source; broad compatibility/parity unverified.** |
| Module-world source-free loader and resolver seed | `load_module_world_with_allocation_budget`; `with_external_modules` during project discovery | **Structural integration exists; explicit-import fresh-process parity unverified.** |
| Shared type/static export roots | Canonical root registration + closure calls; enlarged field-specific roots | **Partial roots; missing full callable/other-family census.** |
| CallableAnalysis DTO/export | Graph and selected denotations, origins, call-specialization static args | **Partial representation; all normal analyses rejected.** |
| CallableAnalysis import | Budgeted flow/denotation helpers exist; complete analysis table not materialized | **Not implemented end to end.** |
| Trait/dispatch/conformance materialization | Tables constructed, but with documented defaults and missing fields | **Not semantically lossless.** |
| Static and module binding primary owners | DTO/building blocks exist; imported snapshot uses empty required owners | **Not source-free complete.** |
| Whole-load allocation ledger | Container/module/selected semantic/flow work charged | **Partial; two explicit hard stops.** |
| Query reconstruction | Typed decoder/reconstruction call and targeted closure tests exist | **End-to-end loaded owner/fingerprint parity not demonstrated.** |
| Writer/atomic publication | Same-directory `NamedTempFile`, validate actual bytes before write | **Source implementation exists; no genuine complete image can currently pass validation.** |
| Compiler integration | Loaded-image `Project`, `Package`, `Module` and `Inline` paths | **Promising seam, no completed package/explicit-import artifact acceptance.** |

## 5. Implementer execution route — what to do next, without unsafe shortcuts

The following ordering is deliberately **dependency-driven**; it avoids spending time on end-to-end suites that currently cannot reach their assertions.

### Phase A — establish immutable truth and a compiled owner manifest

1. Pin and identify the actual local worktree HEAD/diff against `2939e9d`; preserve any new dirty/unpushed work. Read the current branch versions of `CHECKPOINT.md`, the detailed P7 plan, the allocation incident and repository `AGENTS.md` before editing.
2. Replace a binary `Wire` label as a completion indicator with a **field-by-field proof matrix** for every `CallableAnalysis` field and each `RequiredSemantic` owner. Required columns: native owner and producer; compiler consumer; intended on-wire owner/ordinal; DTO field; export/root registration; validation; budgeted import; installed snapshot/index; parity test and result; source-free consumer exercise. Retain the accepted `CheckedStaticBody` owner for checked static-definition applications unless an authorized owner change is recorded.
3. Inventory the exact allocators of `SemanticWorldV2::validate_inner`, `TypeStore::import_type_graph`, and all semantic native imports. Associate each requested byte with one transaction stage. Prefer one coherent owner seam to mass refactoring of unrelated stores.

### Phase B — implement complete semantic owner transport and its allocation-aware materializer together

4. Finish callable analyses as **owner-qualified complete graphs**, not summaries: exact expressions, bindings/origins/site attachments, evaluated denotations, call and trait resolution, match/pattern, iteration/generic evidence, CFG/exits/return facts, dependencies, diagnostics and causal explanations. Use the shared type/static relocation, no new checker or source lookup. Make ordinary compiler lowering read those restored structures.
5. Implement and validate all missing static/derived/trait/conformance owner fields and attached `module_products`. Replace `PrimaryDeclaration/0/None`, `KindId(0)`, assumed `TypeId(0)`, omitted witness views, generic failure placeholders, fabricated empty tables and synthetic `Complete` with exact field transport or explicitly approved deterministic reconstruction.
6. Make validation and materialization budget-visible and fallible before disabling the two `UnaccountedBoundary` stops. Keep decoder and importer fail-closed until counting-allocator probes pass. Verify each reconstituted table is owned by the intended canonical snapshot/module owner and that immutable parent overlays do not perform deep canonical copies.

### Phase C — certify usable image and package outcomes

7. **Smallest real positive gate:** canonical snapshot exports every required owner, `build_universe_telosc` succeeds, and disk/memory load rebuilds a `SemanticSnapshot` satisfying `is_source_free_artifact_complete()`. Assert exact owner presence and absence of internal incidents; distinguish a genuine complete graph from arbitrary nonempty rows.
8. **Required semantic gate:** compare source-built and fresh-imported type/parameter/static, callable/field/constructor, traits, data/enums, witnesses, associated and static products at field/identity/behavior level after relocation. Include two owner-local ordinal-zero collisions and wrong-owner corruptions; use canonical Universe data, not only synthetic mini-worlds.
9. **Required cold-process gate:** load disk artifact with Universe source acquisition forbidden; compile a **novel explicit-import project**; observe zero Universe parse/link/semantic-body work. Test `parse` and `slotmap` project entry paths, and resolve the similarity manifest / use a consuming-project fixture. Cover language facilities exercised by those libraries, including generics, traits, enums, advanced calls and flow.
10. **Required hardening gate:** deterministic bytes from independent fresh stores/processes and different insertion/unreachable histories; native-catalog/provenance rehashed incompatibility; whole-load allocation exact fit and aggregate overflow; repeated/parallel writes and previous-output preservation; lower-bound and scaling benchmarks; P5/P6 locality/nonregression. Run only specific targets that exercise the changed seam first, then acceptance suites after the last material code change.

**Guard-removal rule:** An individual guard may be removed only when its own safety contract is actually implemented and tested. Even then, the next independent guard may still fail by design. Never remove all guards as a shortcut to see green output; never weaken completeness or silently use source fallback.

## 6. Suggested discriminating test specifications

| Test name / scope | Stimulus | Must demonstrate | Should fail before corresponding repair? |
|---|---|---|---|
| `semantic_world_budgeted_validation_accepts_real_canonical_owner_graph` | Valid canonical SEMANTIC_WORLD DTO | Decoder passes, every validator request charged, exact `N` passes, `N−1` errors without debit | **Yes**: unconditional S1-01 stop. |
| `semantic_import_budgeted_materialization_relocates_real_snapshot` | Valid world, fresh type/static stores | Import succeeds without extra unaccounted allocator requests, no source acquisition | **Yes**: S1-02 stop. |
| `callable_owner_graph_round_trip_consumer_parity` | Canonical method + synthetic `<main>`, shadowing and module import, rich flow/match, repeated body-local ordinals | Every stored field restored to correct body owner; lowering read parity | **Yes**: S1-03/04/06. |
| `trait_dispatch_exact_type_evidence_after_load` | Generic trait default and class/instance member | Restored callable knowledge equals source evidence; never `TypeId(0)` fallback | **Yes**: S1-05. |
| `conformance_witness_full_status_and_view_parity` | Conditional/incomplete/invalid and associated type witnesses | Views, invalidity, failures, selectors and provenance survive | **Yes**: S1-05. |
| `loaded_image_has_real_module_and_static_products` | Canonical Universe artifact | Snapshot's exact module/static/derived products populated, validated and consumed | **Yes**: S1-04. |
| `source_denied_project_uses_loaded_universe_imports` | New project with explicit Universe imports, cold child process, poisoned providers | Package compile succeeds, imported lookup works, canonical source-work counters all zero | **Yes/no**: blocked today; must select and execute after upstream repair. |
| `parse_slotmap_similarity_compiler_project_regressions` | Project entry/consuming fixtures based on real packages | Semantic+lowering parity, no need to modify dependencies to get a pass | **Blocked today**, with similarity layout issue needing fixture decision. |
| `cross_stage_allocation_observation_is_exact` | Entire disk load; separately stage-fitting but cumulatively over-budget | Counted allocator-request bytes match debited ledger; rollback; bounded growth | **Yes** for uncharged owner paths. |
| `fresh_process_canonical_artifact_determinism` | Two independent stores/processes, reversed insertion, unreachable historical values | Byte-for-byte identical `.telosc` sections and digest | **Unknown** until a genuine image exists. |

No assertion should pass merely because `is_err()` catches an unrelated earlier stage. Assert typed error class **and the intended execution frontier**. Zero selected tests are not a passing gate.

## 7. P7 versus P8/P9: important scope discipline

P7 must persist a **complete source-independent compiler image**, permitting ordinary compiler and backend work for standalone user packages without rebuilding Universe from source. It may store semantics and checked operational authority needed by compiler consumers. It does **not** need to store the separately specified portable executable VM image, which belongs to P8; a `PORTABLE_EXECUTABLE` capability in P7 is forbidden by its writer. General automatic artifact discovery, freshness invalidation and incremental artifact rebuilding belong to later cache/distribution work (not a substitute for P7's working explicit artifact APIs). Do not shift P7 completion to a future cache phase: successful explicit build→write→cold load→real package compile is required **now**.

## 8. Follow-on audit slices

**Slice 2 — semantic authority deep dive:** Exact `CallableAnalysis` field inventory; owner-local ordinal stability; `CheckedStaticBody` application ownership; `<main>` source attachments, storage origins and lowering; trait, witness, derived, static/associated/type-member owners; field-accurate parity counterexamples. This is the highest-value next source-level audit because it blocks valid production artifacts.

**Slice 3 — artifact integrity and memory:** Container limits, native metadata/decoder scratch, type/static graph cycles and relocation, cumulative allocation/requested-byte accounting, graph memory sharing, malicious inputs, publication atomicity and races, deterministic codec IDs, possible denial-of-service/performance hazards.

**Slice 4 — source-free runtime/compiler integration and acceptance:** Provider precedence, project/package modes, import exposure/privacy, query/fingerprint reconstruction, actual `parse`/`similarity`/`slotmap` fixtures, CLI/child-process denial harness, P5/P6 locality, cold/warm benchmarks, failing workspace tests and P7 release matrix.

## 9. Explicit limits and disposition

This report identifies confirmed source defects and logical blockers, but **it is not a clean-tree Cargo certification**. No local worktree inspection, compilation, allocator instrumentation, child-process source-denial harness, package execution, native codec fuzzing or end-to-end benchmark was performed in this audit. The attached reports contain focused PASS records for selected codecs and CFG/allocation primitives, as well as a reported failed canonical artifact test; those remain inherited test reports, not tests rerun here. The existing source gives no defensible basis to claim a working persisted artifact or realized performance gain.

**P7 decision at Audit Slice 1:** **OPEN — BLOCKED FOR FULL ARTIFACT PUBLICATION AND COLD LOAD**. Continue implementation against the pinned current branch; preserve the safety guards; finish complete primary semantic transport and exact allocation accounting before promoting any G7.x acceptance state. **P8 handoff remains blocked.**
