---
id: PERF003.C2.P7.EXECUTION
program: PERF003
checkpoint: PERF003.C2
phase: PERF003.C2.P7
status: PLANNED
implementation_status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
prepared_against_revision: 3ca30fca
prepared_against_working_tree: true
depends_on: [PERF003.C2.P5, PERF003.C2.P6]
autonomous: true
execution_mode: compiler-only-build
files_modified:
  - phalcom-compiler-artifact/src/semantic/
  - phalcom-compiler-artifact/src/query.rs
  - phalcom-compiler-artifact/src/metadata.rs
  - phalcom-semantic/src/artifact/
  - phalcom-semantic/src/types/artifact.rs
  - phalcom-semantic/src/static_value/
  - phalcom-semantic/src/canonical_image.rs
  - phalcom-semantic/src/snapshot.rs
  - phalcom-core/src/modules/
must_haves:
  truths:
    - Required semantic families and compiler attachments have exhaustive portable dispositions.
    - Loaded module, type, static, query and metadata authority uses fresh process stores.
    - User compilation consumes one immutable loaded image without Universe source construction.
    - P6 owner indexes and P5 immutable-parent overlay boundaries survive loading.
    - Encoding is deterministic across physical roots and allocation histories.
  prohibitions:
    - statement: Load must not parse, link, analyze, rebuild witnesses, export metadata or lower Universe.
      status: resolved
      verification: backstop
    - statement: No raw snapshots, parsed modules, process store IDs or runtime handles enter wire payloads.
      status: resolved
      verification: backstop
    - statement: No tests, runtime probes or certification commands execute under the current user policy.
      status: resolved
      verification: explicit
---

# P7 remaining execution plan

## Contract and current state

This companion refines execution of the existing `PERF003.C2.P7-telosc-persisted-semantic-compiler-image-detailed-implementation-plan.md`; it does not replace its architecture or renumber its tasks. Consume the shared execution research and latest handoff first. T1–T5 have code, with T4/T5 integration and deterministic export still incomplete. T6–T12 have no complete implementation. T13/T14 acceptance is deferred. There is no usable `universe.telosc` yet.

Approved invariants: artifact owns explicit bounded wire DTOs; modules owns completed module products; semantic owns type/static/primary/query reconstruction; core composes metadata and public artifact APIs. Required sections are MODULE_WORLD, SEMANTIC_WORLD, QUERY_IMAGE and RuntimePublic METADATA. Stable owners and graph indices replace allocation coordinates. No source provider is installed on the loaded image. Error paths publish nothing and remain retryable.

P7's first complete compiler tracer is T12. Earlier waves supply genuine dependencies, not stub Universe products. It must remain usable without P8 executable content; the VM execution API separately requires PORTABLE_EXECUTABLE. P9 automatic discovery/cache/distribution remains excluded.

## Execution waves and ownership

| Wave | Tasks | Result | Scheduling |
|---|---|---|---|
| 1 | T0 refinement + T5 integration | Complete persistence manifest and deterministic shared export context | Sequential; no re-audit of established T1–T4 code |
| 2 | T7.A structural products, T6, T7.B remaining aggregates | Type-validated static values and full primary authority | Structural products precede ADT static import |
| 3 | T8, T9; T10 | Indexed immutable authority, query image, metadata payload | T10 may be implemented independently; Cargo checks stay serial |
| 4 | T11, T12 | Deterministic writer and explicit source-free compiler path | Writer/loader share the same manifest/maps |
| 5 | T13/T14 coverage and T15 | Deferred gate harnesses and truthful handoff | No acceptance execution without policy change |

Scheduling independence does not authorize subagents. A single implementer can execute every wave. When splitting a long continuation, stop at a completed owner interface and record the next task in CHECKPOINT.md.

## Loader construction order

1. Bounded container/section decode and independently defined reader compatibility check.
2. Complete source-free module world and stable owner map.
3. Fresh TypeStore with a checked TypeRelocation and immutable graph identity.
4. Type-only declaration shells, hierarchy/aliases, constructor/data/enum/variant contracts and normalized generic constraints.
5. Fresh StaticValueStore through ordinary validation, using those primary contracts.
6. Remaining signatures/surfaces, witness and trait products, static definitions/constants, checked operational templates, module bindings and backend analyses.
7. Owner indexes, artifact query selection and current-process dependency fingerprints.
8. Existing metadata decoding/validation and cross-section owner/type/capability checks.
9. Atomic construction of loaded snapshot/formal/query roots and core LoadedUniverseArtifact.

All intermediate builders remain private. A genuine graph cycle uses validated IDs/shells and a final completeness pass. Never expose incomplete snapshots or install artifact products as mutable local DB Ready rows.

## Artifacts this phase produces

Proposed names may adapt mechanically: TypeExportContext and StaticExportContext with lineage checks; StaticValueGraphV1 and StaticRelocation; SemanticWorldV1 and per-family DTOs; SemanticExportContext/ValidatedSemanticParts; canonical artifact profile query support manifest; QueryImageV1; metadata payload wrapper; CanonicalUniverseFormalImage::from_loaded_artifact; LoadedUniverseArtifact; build_universe_telosc, write_universe_telosc, load_universe_telosc; explicit ProgramCompiler loaded-image entry API; per-stage load measurements. New modules live at the owners named in task file lists. T11/T12 expose no executable VM entry point before P8 supplies it.

## P7.T0/T5 — Freeze coverage and shared graph projection

<task id="P7.T0/T5" type="auto" wave="1">
  <name>Freeze coverage and shared graph projection</name>
  <depends_on>Inherited T1–T5 code</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>canonical_image.rs persistence inventory; snapshot.rs; types/artifact.rs; original P7.T0/T5</read_first>
  <files>phalcom-semantic/src/types/artifact.rs; phalcom-semantic/src/artifact/manifest.rs (new); phalcom-compiler-artifact/src/semantic/type_store.rs</files>
  <action>Create an exhaustive field-to-wire/derived/omitted manifest for SemanticSnapshot and all CanonicalProductFamily variants, with each compiler reader named. Required CompilerSourceAttachments cannot be reclassified just because source is absent. Enumerate checked-static-body and constraint syntax consumers. Expose a single guarded export context for type/kind/parameter/row/family/variant/domain/constructor/scoped/lambda references, shared by all sections. Collect canonical roots from required primary products and selected queries, ordered by family tag and stable owner/product key; include current parameter slots as stable-keyed roots. Walk each root in deterministic depth-first field/child order and assign per-arena IDs on first encounter, emitting explicit forward references where needed. Native IDs serve only as private memo keys, never as root order or wire IDs. Preserve distinct referenced historical versions even when payloads compare equal; never coalesce them by content hash. Close the graph before bytes are emitted. Retain referenced historical parameter identities and current-slot/variance selection; omit unreachable arena history. Preserve ordered scoped unions. Integrate existing module transport without reparsing. New QueryKind/product variants must force exhaustive disposition.</action>
  <acceptance_criteria>Coverage manifest has no undisposed required field; foreign-lineage lookup fails; source attachment consumers have a source-independent transport contract; all consumers use one graph map.</acceptance_criteria>
  <verify>At completed seam only: RUSTFLAGS=&#x27;&#x27; RUSTC_WRAPPER=&#x27;&#x27; cargo check -p phalcom-semantic --lib. No tests.</verify>
  <done>Coverage manifest has no undisposed required field; foreign-lineage lookup fails; source attachment consumers have a source-independent transport contract; all consumers use one graph map. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T7.A — Load structural primary contracts before static ADTs

<task id="P7.T7.A" type="auto" wave="2">
  <name>Load structural primary contracts before static ADTs</name>
  <depends_on>P7.T0/T5</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>data_semantics.rs; enum_semantics.rs; signature.rs; types/parameter.rs; static_value/store.rs::intern_data/intern_variant</read_first>
  <files>phalcom-compiler-artifact/src/semantic/aggregates.rs (new); phalcom-semantic/src/artifact/import.rs (new); relevant owner-table insertion APIs</files>
  <action>Define explicit declaration, alias, hierarchy, generic signature/constraint, constructor, DataInfo, EnumInfo and VariantInfo DTOs. Relocate their type references and stable component/case identities, preserve declared order, and construct a private structural authority used by T6. Validate constructor ownership, variant membership, shape templates, result kinds and type references without running formation or solving. Defer any static-valued payload to a typed pending reference, require final resolution before publication; never fill missing fields with defaults. Record exact pending-reference dependencies for final cross-validation.</action>
  <acceptance_criteria>Existing static ADT factories can consume reconstructed contracts; no struct field or genuine dependency cycle is bypassed.</acceptance_criteria>
  <verify>Batch with T6/T7 completion: cargo check -p phalcom-semantic --lib with flags/wrapper cleared.</verify>
  <done>Existing static ADT factories can consume reconstructed contracts; no struct field or genuine dependency cycle is bypassed. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T6 — Transport and relocate every static-value family

<task id="P7.T6" type="auto" wave="2">
  <name>Transport and relocate every static-value family</name>
  <depends_on>P7.T7.A; shared type export context</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>static_value/store.rs; static_value/store.rs materialization descriptors; original P7.T6</read_first>
  <files>phalcom-compiler-artifact/src/semantic/static_value.rs (new); phalcom-semantic/src/static_value/artifact.rs (new); static_value/store.rs</files>
  <action>Add per-arena static value/string/symbol/tuple/sequence/record/data/variant/pack-backing/pack DTO IDs and guarded export/import maps. Preserve arbitrary integer values, float bits, structural field order, Array/Vector family, result types and shared pack backing/slice bounds. Import into a fresh lineage through existing factories, including DataInfo/VariantInfo validation. Restore interners with ordinary equality/hash behavior. Validate children, type kinds, exact cases, duplicate keys, pack ranges, cumulative memory and graph depth/cycles. Use iterative traversal or explicitly bounded recursion. Do not flatten shared packs or persist StaticValueStoreId.</action>
  <acceptance_criteria>All StaticValueData and structural arena variants have exhaustive adapters; references relocate to fresh stores and bounded validation precedes publication.</acceptance_criteria>
  <verify>One compiler check after the coherent T6/T7 seam; no runtime or round-trip tests.</verify>
  <done>All StaticValueData and structural arena variants have exhaustive adapters; references relocate to fresh stores and bounded validation precedes publication. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T7.B — Complete semantic primary payload and checked attachments

<task id="P7.T7.B" type="auto" wave="2">
  <name>Complete semantic primary payload and checked attachments</name>
  <depends_on>P7.T6; P7.T7.A</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>snapshot.rs; impls.rs; static_definition.rs; static_eval/{machine,application}.rs; checker/analysis.rs; original P7.T7</read_first>
  <files>phalcom-compiler-artifact/src/semantic/*.rs; phalcom-semantic/src/artifact/{export,import,validate}.rs (new); owning primary tables</files>
  <action>Encode every required manifest row: signatures/fields/accepted definitions and inherent context; surfaces/dispatch; trait headers/surfaces, conformance heads, witness plans, derived rules; ADTs/requirements; type members/associated/static domains and cases; constants/definitions/module bindings; backend callable analyses and compiler-facing fact/site attachments. Normalize constraints and checked static operational templates with stable targets, relocated operands, local slot recipes and retained evaluation order; do not persist raw ParsedModuleUnit or rerun checking. Keep source locations logical/capability-separated. If a required checked template still needs raw syntax or new semantic reasoning, consult before modifying the evaluator contract; record it as a blocked family rather than emit a partial complete image. Validate all pending static/type/owner edges; construct only a complete immutable snapshot.</action>
  <acceptance_criteria>Every required primary family and compiler attachment is accounted for, with no missing-default fallback, duplicated witness construction or analysis during import.</acceptance_criteria>
  <verify>RUSTFLAGS=&#x27;&#x27; RUSTC_WRAPPER=&#x27;&#x27; cargo check -p phalcom-semantic --lib at complete family batch only.</verify>
  <done>Every required primary family and compiler attachment is accounted for, with no missing-default fallback, duplicated witness construction or analysis during import. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T8 — Reconstruct indexes through their owning tables

<task id="P7.T8" type="auto" wave="3">
  <name>Reconstruct indexes through their owning tables</name>
  <depends_on>P7.T7.B</depends_on>
  <reversibility rating="reversible">Private implementation can change within the owning boundary.</reversibility>
  <read_first>P6 plan/handoff; CallableAnalysisTable; DataSemanticTable; EnumSemanticTable; trait requirement and inherent-domain owners</read_first>
  <files>phalcom-semantic/src/artifact/import.rs; owner insertion/bulk constructors; snapshot.rs</files>
  <action>Populate module indexes for analyses/definitions/declarations/data/enums during primary insertion. Restore reverse target occurrence lookup for compiler-required sites, direct inherent impl domains, derived-conformance get_rule index and trait requirement layouts. Derive slot layout only from validated member order and verify its fingerprint; never reconstruct witnesses. A bounded single O(N) pass is allowed; repeated per-module whole-world scans are forbidden. Preserve immutable parent tables and direct exact-dependency reads.</action>
  <acceptance_criteria>Loaded owner tables expose P6 local iterators/slot/domain lookup with all required secondary entries; no competing central index database.</acceptance_criteria>
  <verify>Combine with T9 compiler check; P6 workload counters remain deferred.</verify>
  <done>Loaded owner tables expose P6 local iterators/slot/domain lookup with all required secondary entries; no competing central index database. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T9 — Restore eligible queries and separate dependency evidence

<task id="P7.T9" type="auto" wave="3">
  <name>Restore eligible queries and separate dependency evidence</name>
  <depends_on>P7.T8; shared primary maps</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>canonical_image.rs::canonical_query_persistence; db/{key,product,fingerprint}.rs; original P7.T9</read_first>
  <files>phalcom-compiler-artifact/src/query.rs (new); phalcom-semantic/src/artifact/query.rs (new); canonical_image.rs</files>
  <action>Define exhaustive profile support for each QueryKind, portable keys, aggregate references and dedicated query-only DTOs. Omit ParsedModule and reverse-prune reusable roots requiring it. Restore aggregate-backed dependency evidence separately, including unsupported reusable-query kinds needed by user-overlay invalidation. Recompute ProductFingerprint/InputFingerprint as applicable from relocated authority and remap observed dependency fingerprints; do not trust old u64 fingerprints across processes. Validate product/aggregate agreement, canonical origin, closure and DAG bounds. Persist no open-world negative proof. Read through the immutable canonical query reader without inserting local Ready rows.</action>
  <acceptance_criteria>Query eligibility remains singly owned; supported roots and dependency-only evidence are distinct; no source/AST dependency is needed to read canonical compiler facts.</acceptance_criteria>
  <verify>RUSTFLAGS=&#x27;&#x27; RUSTC_WRAPPER=&#x27;&#x27; cargo check -p phalcom-semantic --lib.</verify>
  <done>Query eligibility remains singly owned; supported roots and dependency-only evidence are distinct; no source/AST dependency is needed to read canonical compiler facts. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T10 — Transport existing RuntimePublic metadata

<task id="P7.T10" type="auto" wave="3">
  <name>Transport existing RuntimePublic metadata</name>
  <depends_on>Existing canonical compiler-image view; T0 manifest</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>phalcom-type-meta/src/{bundle,encode,validate}.rs; modules/canonical_universe.rs::CanonicalCompilerImageView</read_first>
  <files>phalcom-compiler-artifact/src/metadata.rs (new); phalcom-core/src/modules/canonical_artifact.rs (new)</files>
  <action>Use the already-owned canonical metadata bundle once. Reuse its existing encoding/decoder/validator behind a bounded METADATA wrapper rather than inventing another semantic metadata schema. Canonicalize unordered inner encoding collections deterministically without changing ordered pools/indices. Enforce total metadata bytes, token/container counts and nesting before allocating decoder collections; use bounded preflight if the existing decoder only validates after deserialization. Charge the shared artifact allocation budget rather than resetting it for each section. Require RuntimePublic and portable V1Standard identities; check outer/inner versions, package identities, digest/provenance relationships and resolved cross-pool references. Retain Arc&lt;SemanticMetadataBundle&gt;; runtime pool allocation belongs to P8. No MetadataExporter call during load or repeated exporter call during write.</action>
  <acceptance_criteria>Metadata wrapper preserves the existing schema and stable identity authority; decoding is bounded and SessionLocal metadata is rejected.</acceptance_criteria>
  <verify>Batch with T11/T12: cargo check -p phalcom-core --lib with flags/wrapper cleared.</verify>
  <done>Metadata wrapper preserves the existing schema and stable identity authority; decoding is bounded and SessionLocal metadata is rejected. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T11 — Build a deterministic compiler artifact and publish atomically

<task id="P7.T11" type="auto" wave="4">
  <name>Build a deterministic compiler artifact and publish atomically</name>
  <depends_on>T6–T10; T1–T5 adapters</depends_on>
  <reversibility rating="one-way">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>canonical_universe.rs; artifact header/identity/container; telosc.md; original P7.T11</read_first>
  <files>phalcom-core/src/modules/canonical_artifact.rs; phalcom-core/src/modules/mod.rs; artifact crate header/validation integration</files>
  <action>Expose build_universe_telosc(options) returning validated bytes and a structured summary; expose write_universe_telosc(path, artifact) for atomic file publication. Compiler-only build declares no executable capability. Convert the core in-process compatibility view into portable package/ABI/digest identity. Build each payload with one export context and canonical order, compute domain-separated BLAKE3 section and whole-artifact digests with the latter field masked. Validate cross-section references using the same loader validation owners before output. Create a unique same-directory temporary file, write/flush, validate, atomically replace; failures preserve prior output and clean task-owned temp files. Report schemas, capabilities, digests, section sizes and entity counts. P8 augments this pipeline with executable capture.</action>
  <acceptance_criteria>No allocation ID or physical root contributes to canonical bytes; failed builds cannot replace a valid output; format capabilities truthfully match payloads.</acceptance_criteria>
  <verify>One core compiler check at the end of T12; atomic-write/determinism execution deferred.</verify>
  <done>No allocation ID or physical root contributes to canonical bytes; failed builds cannot replace a valid output; format capabilities truthfully match payloads. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T12 — Load and inject the immutable compiler image explicitly

<task id="P7.T12" type="tracer" wave="4">
  <name>Load and inject the immutable compiler image explicitly</name>
  <depends_on>P7.T11</depends_on>
  <reversibility rating="costly">Cross-layer contracts or persistent payloads require coordinated compatibility changes; previously approved wire decisions are not re-gated.</reversibility>
  <read_first>ProgramCompiler analysis entry seams; SemanticWorkspaceSession::from_image; canonical_image.rs; original P7.T12</read_first>
  <files>phalcom-core/src/modules/canonical_artifact.rs; modules/compile.rs; phalcom-semantic/src/artifact/import.rs; canonical_image.rs</files>
  <action>Expose load_universe_telosc(path, limits) and an internal bounded bytes entry point. Return Arc&lt;LoadedUniverseArtifact&gt; containing private validated formal/module/query/metadata authority, portable identity, capabilities and optional executable slot reserved for P8. Compare reader ABI constants without canonical source acquisition; validate all sections before publication. Add CanonicalUniverseFormalImage::from_loaded_artifact and ProgramCompiler entry APIs taking the loaded image explicitly. Feed the same Arc into the user-only workspace overlay. Audit every default-image acquisition on that path; do not call source-backed OnceLock initialization. Loaded images own fresh lineage/query prefixes and no source provider. Error variants preserve I/O/schema/digest/budget/compatibility/module/type/static/semantic/query/metadata/capability causes. Failed loads publish nothing and are retryable.</action>
  <acceptance_criteria>Explicit user compilation can consume the loaded image; compiler-only files load without VM execution; no source acquisition or mutable canonical query rows occur on that call path.</acceptance_criteria>
  <verify>RUSTFLAGS=&#x27;&#x27; RUSTC_WRAPPER=&#x27;&#x27; cargo check -p phalcom-core -p phalcom-lsp --lib. No source-free proof executed.</verify>
  <done>Explicit user compilation can consume the loaded image; compiler-only files load without VM execution; no source acquisition or mutable canonical query rows occur on that call path. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T13/T14 — Prepare and defer acceptance evidence

<task id="P7.T13/T14" type="auto" wave="5">
  <name>Prepare and defer acceptance evidence</name>
  <depends_on>P7.T12 code; user check restriction unchanged</depends_on>
  <reversibility rating="reversible">Private implementation can change within the owning boundary.</reversibility>
  <read_first>Original P7 §§23–24; existing semantic integration/canonical_image tests; core test placement READMEs</read_first>
  <files>Future tests at artifact/container, semantic/integration/canonical_image, and core artifact integration owners; telos-observability owner if counters need extension</files>
  <action>Prepare focused coverage for static/type/semantic/query/metadata parity, malformed refs/limits/compatibility, deterministic fresh-process builds under different roots/allocation histories and failure retry. Plan a child process that denies every Universe provider/embedded-source access, loads the image and semantically compiles a novel user module. Require zero Universe parse/interface/link/update/body/witness/export/lowering counts and P5/P6 structural parity. Define separate timings for file read, container validation, section validation, relocation and publication. Existing CLI parser check is not semantic proof. These are deferred acceptance tasks: do not execute tests, child processes or benchmarks under the current policy.</action>
  <acceptance_criteria>Acceptance matrix and harness ownership cover every original gate without claiming PASS; absent evidence remains explicitly deferred.</acceptance_criteria>
  <verify>DO NOT RUN. If test harness code is added later, targeted compiler-only test-target checking is permitted; execution requires a changed user instruction.</verify>
  <done>Acceptance matrix and harness ownership cover every original gate without claiming PASS; absent evidence remains explicitly deferred. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## P7.T15 — Record implementation separately from certification

<task id="P7.T15" type="auto" wave="5">
  <name>Record implementation separately from certification</name>
  <depends_on>P7.T12; deferred gate disposition</depends_on>
  <reversibility rating="reversible">Private implementation can change within the owning boundary.</reversibility>
  <read_first>CHECKPOINT.md; implementation-record lifecycle convention</read_first>
  <files>CHECKPOINT.md; P7/PERF003.C2.P7-{walkthrough,handoff}.md</files>
  <action>Update meaningful established APIs, unresolved required families, exact compiler commands/results, and deferred acceptance. P7 source-free compiler gate must pass before COMPLETE. Under compiler-only continuation, implementation may become IMPLEMENTED only when required code is finished, but status stays IN_PROGRESS and verification UNVERIFIED while behavioral acceptance is absent. Never promote P6 inherited focused evidence to P7. P8 BUILD may start after the complete P7 compiler interface and compiler check are available; deferred acceptance blocks lifecycle promotion, not subsequent code work. Preserve unrelated dirty work; do not commit/push without user instruction.</action>
  <acceptance_criteria>One checkpoint state and coherent takeover records identify the next task and exact evidence gaps.</acceptance_criteria>
  <verify>Prose-only record update: no Cargo command.</verify>
  <done>One checkpoint state and coherent takeover records identify the next task and exact evidence gaps. Behavioral evidence remains UNVERIFIED until deferred gates execute.</done>
</task>

## Deferred gate matrix

These are the original detailed plan's exact gate identifiers, retained without renumbering.

| Original gate | Evidence | Current disposition |
|---|---|---|
| G7.0 | P5 persistence/metadata/immutable-copy contract and P6 owner indexes | Inherited predecessor evidence; final wire census still required |
| G7.1 | Portable identities and all compatibility ABIs | Code present; end-to-end acceptance NOT RUN |
| G7.2 | Post-change container schema/digests/limits/corruption | NOT RUN after container changes |
| G7.3 | Source-free external module import parity and zero canonical builds | Module code present; acceptance NOT RUN |
| G7.4 | Type/static relocation, fresh stores, equality and prefix extension | Type code present; static transport open; acceptance NOT RUN |
| G7.5 | Exhaustive semantic primary parity and required attachment transport | NOT RUN |
| G7.6 | Immutable query closure/aggregate agreement and no recomputation | NOT RUN |
| G7.7 | RuntimePublic metadata transport and no exporter/pool IDs | NOT RUN |
| G7.8 | Source-unavailable fresh-process novel user semantic compilation | NOT RUN |
| G7.9 | P5/P6 immutable-copy/local-complexity non-regression | NOT RUN for artifact path |
| G7.10 | Byte-identical fresh builds under root/allocation variation | NOT RUN |
| G7.11 | Separate load timings and materially cheaper cold startup | NOT RUN |

The original plan's named acceptance commands remain authoritative, subject to the user's current DO NOT RUN override. Define actual new harness targets/filters when implementing them and confirm real test selection only once execution is authorized. P7 COMPLETE is blocked by deferred acceptance; P8 code can proceed after P7's implemented interfaces are ready while both plans remain UNVERIFIED.

## Flexibility and stop rules

FIXED: canonical owners, stable identity, fresh store lineage, validation before publication, immutable overlays, source-free authority, BLAKE3 framing and bounded explicit tags. MECHANICALLY FLEXIBLE: helper names/private file splits, equivalent public parameter grouping, and owner bulk insertion APIs. VERIFY-FIRST: checked template/source attachment dependencies, root graph closure, metadata canonical encoding, and genuine static/primary cycles.

Stop and consult on any required source-body regeneration, semantic solving during load, unsupported legal product, unsafe partial publication, uncertain canonical identity/GC contract, or material downstream redesign. Mechanical compiler failures allow three coherent correction cycles; do not use acceptance tests to debug an intentionally incomplete slice. Classify unrelated failures A/B/C/D and preserve inherited warnings. Do not broaden check scope merely because a check passed.
