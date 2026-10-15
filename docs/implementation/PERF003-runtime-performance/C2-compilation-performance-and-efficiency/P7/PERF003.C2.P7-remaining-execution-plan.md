## Current execution checkpoint — resumed, 2026-10-15

The user resumed all authorized R1–R11 corrections and explicitly requested
completion, closure, and focused verification. The earlier query-closure and
callable-analysis slices remain as recorded below. This continuation embeds the
complete owner-held enum requirement signature and has focused codec,
source-fixture round-trip, and relocation evidence. It also adopts the native
catalog compatibility amendment below. Callable expression/binding/flow/
dependency facts, witness views, static and associated/type-member owners, and
query parity remain open. Export/import still reject nonempty callable-analysis
payloads. P7 closure still requires every remaining correction and acceptance
gate; do not proceed to Wave 5 or P8 before those are complete.

### P7 denotation owner-transport amendment — 2026-10-15

The existing GPT-6.1 Sol High Effort adviser returned PLAN NEEDS
CLARIFICATION plus a bounded Level 2 amendment; implementation may resume.
Use a hybrid artifact representation: TypeForm and StaticTerm reference the
shared TypeId graph; Kind references the kind/type graph; StaticDomain
references the domain graph and validates its declaration owner. Associated
exact/family and BehavioralFamily denotations are context-owned captures and
travel inline with stable owner/member/target refs, type refs, operation
shapes, captured optional targets, behavior specification, and conditional
selection. Import must preserve the denotation payload; it must not recompute
lookup, dispatch, selected members, conditional capability, or order from
declaration authority. Reuse shared codecs for associated member/family,
invocation target, operation shape, behavioral spec, and conditional selection.
Do not add a global family-selection registry.

Keep StaticApplication fail-closed until the checked application row is
retained under its actual checked-body owner. Live source shows
CheckingContext::finalize_with_normal_returns omits
checked_static_applications from CallableAnalysis, and
analyze_checked_static_application passes the target in the row body-owner
slot. Never resolve an application through the callee static definition.

Dependency order: register denotation type/kind/domain roots before closing the
shared graph; validate declaration/member/constructor/impl owners; retain and
validate checked-body application rows and owner-qualified identities; then
materialize callable/body graphs and validate all references before snapshot
publication. Required tests cover both associated variants, optional targets,
captured subsets differing from declarations, behavioral captures with and
without conditional selections, denotation-only type roots, overlapping
application ordinals across body owners, malformed/wrong-owner rejection, and
source-denied restored capture without lookup. T7.B/T8/T9/T12 and R3 evidence
are amended; no acceptance criterion is reduced.

### P7 callable binding storage-origin amendment — 2026-10-15

One Level 2 adviser consultation found that every `CallableAnalysis` retains
ambient module binding flow states copied into the body-local `BindingId`
arena. Preserve these states and attach an explicit canonical origin at
`CheckingContext::seed_linked_module_import_values`: `BodyLocal` or
`ModuleValue { symbol: SymbolId, import provenance }`. This records storage
origin without creating a second binding identity. Parameters retain their
existing canonical parameter identity; the previously approved constructor
body/declaration owner mapping remains unchanged.

The callable analysis keeps each ambient state and maps its body-local handle
to the existing module `SymbolId`. The module declaration or exact authored
import alias is represented once by module-owned authority. Copied ambient
states do not become callable-local declarations. Reads and writes retain exact
source occurrences and target the global symbol; local declarations remain
qualified by their owning callable. A missing origin for a required seeded
binding fails closed. Do not infer ambientness from name or range, drop copied
states, or compare numeric `BindingId`s across callables.

The lowering identity is `BodyLocal { callable, binding }` or
`ModuleValue { symbol }`; source ranges are site locators only. Compiler
projection, product planning, collision checks, and active-product tracking
must compare those qualified identities. Two references to one module symbol
agree across callables, while equal numeric local IDs from distinct callables
remain distinct. Extend T7.B/T8/T9/T12, the callable-analysis field manifest,
and the R3 evidence before implementation. Required discriminators cover one
module binding read by multiple methods, a shadowing local, unused ambient
copies, source/import parity, equal numeric IDs under distinct owners, and
malformed origin/conflict rejection. No acceptance criterion is reduced.

### P7 `<main>` source-owner and formal-attachment amendment — 2026-10-15

The same Level 2 adviser selected option A for top-level lexical declarations
whose checker facts belong to the synthetic module `<main>` analysis. Keep the
analysis `CallableId` unchanged and explicitly register
`CallableId → SourceOwner::Module(module)` at its producer. Generalize the
body-to-source-owner relation: ordinary body analyses map to
`SourceOwner::Callable(source_callable)`, and constructors keep their approved
declaration-owner mapping. Include registered module-body analyses in formal
attachment construction and retain the original analysis `CallableId` as the
attachment key.

For example, a top-level lambda parameter `text` has `BodyLocal` storage under
`(<main> CallableId, BindingId)` and attaches to its exact module-owned
`SourceSiteId`. `SourceOwner::Module` identifies the source namespace only; it
does not turn the lambda parameter into a `ModuleValue`. Keep storage origin,
source owner, lexical scope, body-fact identity, and exact declaration site
distinct. Fix `<main>`'s body range from the executable module statement/root
extent, not `program.preamble.range`; executable expressions and nested bodies
must be covered even with an empty preamble. Transport and relocate binding,
expression, and site references together. Do not create a normal callable
signature/declaration row for synthetic `<main>`.

Do not recognize module bodies later by selector spelling, merge a formal
expression attachment with a module AST site because their ranges match, or
apply count/order range rebasing to a module body. The relation must be
explicit, and site identity must be the source-index `SourceSiteId` resolved
from the exact declaration product. Required focused discriminators: two
top-level lambdas each with `text` parameters; each `<main>` binding is
`BodyLocal`, attaches to its own exact module site, and its read resolves to
that parameter; ordinary methods still project module reads as `ModuleValue`;
source/import attachment parity; and lowering with both empty and
documentation-bearing preambles. At the amendment decision the exact Universe
artifact test still fails during lowering at the `universe:json` top-level
`text` parameter, so later loader/source-denied stages have no evidence.

R3 evidence update: main-body binding attachment and complete callable-owner
transport are still missing; the exact Universe build discriminator is
FAILED at the `text` binding site (`MissingFormalBindingAttachment`). Passing
the earlier imported-module and qualified-module-value focused tests does not
cover this owner relation. Keep R3 and P7 open until the module-owner
discriminators, serialization parity, and full artifact/consumer gates pass.

### P7 callable-analysis owner-graph amendment — 2026-10-15

One Level 2 adviser consultation determined that the full compiler-required
`CallableAnalysis` graph is required by R3/T7.B and cannot be replaced by an
execution summary. Preserve every source-owned expression, binding, checked
resolution, CFG/flow/exit, return, dependency, diagnostic, explanation, and
constructor field. Add deterministic artifact-local ordinals qualified by
stable callable identity and export/load relocation maps; validate all
owner-qualified references before publication. Recompute body and query
fingerprints only after relocation. Keep genuine empty call-resolution and
predicate references empty; reject nonempty references until their canonical
owners are transported. Reject internal incidents for the canonical image
profile, and preserve all unsuccessful expression judgments, diagnostics,
causal sharing and exact reports.

Compiler lowering's required binding/site/target authority must be rebuilt from
the relocated callable owner graph (including the formal projection) or carried
through an equivalent owner-held attachment. Do not infer bindings from names,
drop them because the broad `source_index` is omitted, or add a second binding
authority. Amend the detailed T7.B/T8/T9/T12 contracts, the
`SNAPSHOT_FIELD_MANIFEST` explanation and this evidence matrix before
implementation. No P7 acceptance criterion is reduced.

### P7.T7.B callable-signature ownership clarification — 2026-10-15

One read-only adviser consultation resolved the wire-owner ambiguity (Level 1;
the canonical owner model remains unchanged). Extend one complete
`CallableSignatureRecordV1` and reuse it for both `SemanticWorldV1` signature
table rows and `EnumRequirementRecordV1`'s directly owned signature. Preserve
every stored `CallableSemanticSignature` and `CallableParameterSemantic` field,
including exact parameter contracts, rest lanes, inferred knowledge/provenance,
source spans, validation state, implementation, native identity, effects,
raises, return flow and lifecycle. Redundant identities must validate equal.
Share signature export/import and root-registration helpers; all type roots must
enter the existing shared `TypeExportContext` before closure. Import enum
signatures through the existing `TypeRelocation` without inserting them into
`CallableSignatureTable`.

For static native metadata, carry full symbolic values and the exact stable
native key, then compare both against the trusted native catalog bound to the
artifact's native-surface compatibility identity before borrowing static
values. Never leak decoded memory or synthesize defaults. If a valid producer
has noncatalog static values, stop that field and request a Level 2 owned
representation amendment. Required discriminators include full source-built /
imported signature parity, rest Lane/Element/Invalid, `ReturnFlowSpec::Argument`,
inferred evidence and provenance, signature-only type-root closure, and
catalog-mismatch rejection. No enum or broader P7 parity is claimed until these
tests execute.

### Explicit Dynamic call-environment amendment — 2026-10-15

The focused source discriminator confirms that `ReceiverSpecialization`
contains a canonical Dynamic owner argument while
`finalize_owner_call_specialization` incorrectly reads only `get_param`.
`Box<Dynamic>.ping()` and its `Box<Int>` control select the same callable;
the Dynamic environment has no concrete or row entry and does have
`is_dynamic(T)`, while the Int control publishes T=>Int. The Dynamic fixture
currently emits exactly the missing-owner-binding incident.

The Level 2 decision requires a typed term/Dynamic/row argument through
`CallSpecialization`, checked generic instantiation and conformance discharge,
callable-analysis wire DTO/import/export, compiler roots and call-entry
projection, runtime recipes, and runtime environments. Reuse canonical
`StaticArgument`/`RuntimeStaticArgument` categories. Dynamic remains distinct
from a proper TypeId, a missing binding, and proof evidence. Bare Dynamic
materialization returns a typed boundary, `List<T>` with `T := Dynamic` remains
a proper applied term, and Dynamic-dependent conformance creates no static
proof or runtime witness slot. Include call arguments in body/query
fingerprints. `SEMANTIC_WORLD` V2 already covers this record layout; do not
advance the container header again.

This is a bounded prerequisite for P7 R3/T7/T12, with no new language rule or
checkpoint objective. Keep the exporter incident guard. General row execution,
new Dynamic inference, automatic runtime witness acquisition, and unrelated
native erasure are outside scope and require separate escalation. Required
discriminators: exact Int/Dynamic call arguments without incidents;
cold/import parity; forwarding and nested `List<T>`; fail-closed missing
bindings; no fabricated Dynamic conformance; runtime environment identity;
fingerprint sensitivity; all four real Universe incidents absent; then rerun
the exact artifact export/load and source-denied gates.

### Native catalog compatibility amendment — 2026-10-15

The adviser found that native row key plus transported effects/raises/flow/
lifecycle does not detect changes in the canonical row's callable contract,
ABI, intrinsic, trust, or return shape. Add the trusted `catalog_fingerprint()`
value as a fixed 16-byte field to the portable compatibility identity and
container header. Advance the container schema to V2 and reject V1 explicitly;
section schemas remain unchanged. Capture the fingerprint in the source-built
compiler-image view, stamp it into the writer header, and compare it with the
current trusted catalog immediately after container parsing and before section
materialization. Preserve it in loaded identity and compare it in
`check_compatible`. Keep exact stable key and transported payload equality
checks. The fingerprint computation must be allocation-free or its requested
bytes must be included in the load transaction. Regressions: matching-catalog
success, foreign fingerprint in correctly rehashed bytes rejected before
materialization, exact payload mismatch, and explicit V1 rejection.

This is a Level 2 compatibility amendment, not a broader identity redesign.
The user's current request authorizes implementation and focused tests, so the
former compiler-only/no-tests restriction in the execution header is superseded
for this continuation. Update the affected T1/T2/T11/T12 contracts in the
detailed plan; broader P7 acceptance stays governed by the existing gates.

## Adopted Wave 4 review correction amendment — 2026-10-14

User authorization: fix all R1–R11 in the Wave 4 code review. This supersedes the earlier T10-only stop. The same existing adviser was reused; no new subagents are authorized. Adviser verdict: PLAN NEEDS MATERIAL AMENDMENT, Level 2; implementation may resume. No additional human approval is required.

FIXED allocation contract: one load transaction accounts cumulative allocator-requested bytes (including capacities, reallocations and temporary storage), excluding allocator-internal bookkeeping. No RSS bound is claimed. No temporary-storage refund during a transaction; commit caller budget only on success. Borrowed input is distinct from owned acquisition/DTO/native storage. Charge each requested allocation exactly once before construction with checked Layout/capacity and fallible reservation. No per-section reset, arbitrary estimator, global production allocator hook, or unrestricted nested serde allocation.

Dependency order and exact acceptance:

1. R6/R9–R11 boundary corrections: reader-owned ABI/package/capability validation; independent provenance checks; actual-byte owner validation before publication; exclusive unique same-directory temporary file; bounded acquisition before allocation and during growth. Correctly rehashed incompatible headers and malformed bytes preserve prior output.
2. R1/R7 allocation infrastructure: allocation-free lexical JSON inspection; typed current-schema seeds/builders with explicit capacities/string escape destinations; borrowed, preallocated validator scratch; allocation-aware native owner construction and one transaction across every layer. Counting-allocator rejection/capacity tests, cross-section cumulative overflow and rollback are required. Legacy normalization remains separate.
3. R3/R4 primary owner adapters: complete field-level persisted/derived/approved-omission manifest; callable/field/constructor/generic authority; declaration/trait/associated/type-member products; real conformance target/provenance/authorization and complete witness selections/views/associated-type plans; constants/bindings/checked definitions; required callable expression/binding/flow/target/dependency facts. Install all indexes/templates in actual owners. Never manufacture Complete or empty required products.
4. R2 coordinated export: every primary and query-only family registers stable owner/member roots before closure; explicit static roots, empty means empty; ordered semantic traversal; unreachable history omitted; one shared type/static relocation. Independent insertion-order/unreachable-history/fresh-process determinism tests.
5. R5 exhaustive typed query adapter: portable tagged owner keys, exact owner product variants and canonical current product/input fingerprint ingredients for every advertised kind. Reject missing/wrong-kind products; preserve retained root/edge census and post-materialization closure; no zeros/debug identities/skipped rows.
6. R8/final composition: loaded external module authority precedes provider resolution while retaining privacy/exposure checks. Source-denied child-process explicit imports and novel user compilation; semantic/query cold/import parity and P5/P6 gates before claiming complete acceptance.

The prior Wave 4 completion assertions are historical and invalidated by the review. Each correction records implementation separately from verification; passing existing happy-path tests does not close the above acceptance.

## Live R1–R11 requirement/evidence matrix — 2026-10-15

The initial audit baseline was the clean remote-synchronized tree at `c64dfc233d0ee2e2d9c3529ac6344c7180e5aa05`; the active worktree now contains uncommitted changes. R3 and the allocation/query rows below include latest scoped implementation and test evidence. `NOT RUN` means no successful discriminator is claimed; a suite failure during artifact construction is recorded as blocked setup, not as a result for later loader stages.

| ID / task | Canonical owner and invariant | Current implementation | Missing behavior and correction | Discriminator / command | Observed result and uncertainty |
|---|---|---|---|---|---|
| R1 / T10, T12 | `phalcom-type-meta/src/allocation.rs`; metadata codec/validator; every owned allocation is charged before requesting its layout. | `AllocationBudget` exposes explicit `Vec`, string, `Box`, `Arc`, and boxed-string helpers plus budget-aware ordered-map/set builders. Query/module/container scratch uses charged vectors; package identity strings and the directory buffer share the core transaction. | Semantic/query output owner maps, validator-owned nested values/scratch, and some materialized/error allocations remain outside the ledger. Extend sorted-vector/fallible materialization through every required native owner and verify requests independently. | Counting-allocator exact `N`/`N-1` tests over metadata, maps, nested values, errors, and full Universe load; `cargo test -p phalcom-type-meta -p phalcom-compiler-artifact`. | PASS: query-closure 512-root exact request/rollback probe; `allocation_budgeted_map` 1/1; container allocation 1/1; container codec 12/12; module DAG scratch 1/1. Full-load/native-owner requested-byte equality remains unproven. Latest semantic importer evidence: callable-signature type evidence exact-fit/N-1 rollback test PASS 1/1; artifact allocation zero-byte typed-error probe PASS 1/1. Whole-load requested-byte equality remains unproven. |
| R2 / T0, T5–T7, T11 | Semantic owner projection owns canonical roots; equivalent semantic inputs emit identical complete bytes independent of allocation/root history. | Core builder uses rooted `TypeExportContext::new`, `register_semantic_world_roots`, and static export context rather than `new_full`. Callable-analysis roots now cover expression/binding knowledge and contracts, flow summaries/exits, sequence/expansion products, associated/callable-reference/family resolutions, and for/positional iteration products. | Root registration still omits match/pattern, trait-dispatch, contextual-realization, explanation, and other owner references; cross-process/fresh-store determinism and unreachable-history pruning remain unverified. Complete one ordered root census before closure and compare independent artifacts. | Two fresh stores/processes, opposite insertion order, unreachable nodes, and different physical roots; complete artifact digest comparison. | Focused callable-analysis root filter: 15 passed. Complete artifact digest discriminator remains `NOT RUN`; canonical artifact construction still stops at the callable-analysis coverage guard. |
| R3 / T7.B, T8 | `phalcom-semantic/src/artifact/{export,import}.rs`, `snapshot.rs`, and each owning table; all P5 `RequiredSemantic` authority survives with source-built field parity. | Enum requirement records carry the full owner-held signature, roots, source order/statuses, and direct type relocation. Callable analysis additionally carries owner-local expression count, typed call-static-argument rows, source attachments, storage origins, status, unreachable-exit state, constructor-super target/range, and a budgeted frozen `FlowGraph` importer with exact adjacency reservations. The semantic reader API preserves source/frozen graph traversal and fingerprint semantics. | Callable expressions/bindings, flow summaries/exits, dispatch and resolution products, dependencies/fingerprints, diagnostics/explanations, compiler binding/site/target reader, and reconstruction into the owner snapshot remain absent. Nonempty conformance bindings and every nonempty callable-analysis payload remain fail-closed. Preserve exact judgments; reject unowned call/predicate handles and internal incidents. | Full source-built/import field parity modulo relocation, two-owner isolation, changed store numbering, malformed closure, exact compiler attachments, deterministic allocation-capped image, and source-denied consumer compilation. Flow slice discriminators: allocator-observed high-degree adjacency exact fit/N-1 rollback, malformed endpoint, source/frozen parity/fingerprint, frozen clone sharing/mutation rejection. | New flow slice: allocator probe 1/1, frozen storage 1/1, fingerprint parity 1/1, source CFG construction filter 3/3. A broader flow-graph filter selected 14 tests and failed one unrelated iteration typing assertion (13 passed); no full callable-analysis parity exists. Exact Universe image test still stops at callable-analysis export coverage. Level 2 denotation amendment recorded; implementation and focused transport tests NOT RUN. StaticApplication remains owner-blocked and fail-closed. |
| R4 / T7.B, T8 | `phalcom-semantic` conformance/witness owners; preserve actual target, provenance, generic application, selections/views, associated plans, invalidity, and completeness evidence. | Conformance-head target/source/generic/authorization fields and partial witness selection DTOs are present. | Witness `requirement_views`, conditional inherent applicability, exact failure/budget payloads, `invalid_explicit_members`, and associated-plan failures are dropped or replaced; layout fingerprint is not a proof of selected-witness parity. Persist and validate full owner facts; never synthesize Complete/default evidence. | Generic/conditional conformance, trait default, associated-type and invalid/incomplete witness source-built/load parity. | `NOT RUN`; export guard and missing witness DTO fields block a legitimate artifact. |
| R5 / T9 | `phalcom-semantic/src/artifact/query.rs`, `canonical_image.rs`, `db/*`; eligible roots and dependency evidence reconstruct exactly once with correct fingerprints and no silent loss. | Typed portable owner decoding and required-product errors are present for several supported query kinds; query DTO decode has a shared-budget path. Query closure now uses budgeted vectors, in-place unstable sort, typed duplicate/cycle/index errors, and iterative O(V+E) DFS; export and load call this path. | Module-static query adapters and evidence consumption remain incomplete; full product/root/edge/fingerprint census and native output-map charges are not proven. Complete every advertised adapter/evidence path and budget output-map materialization. | Missing/wrong-kind/duplicate owner, unknown kind, absent/self/cyclic edge, large DAG, supported-kind census, fingerprint parity; `cargo test -p phalcom-semantic --test semantic artifact`. | PASS for the focused closure discriminator: 512-node DAG, exact measured scratch debit, duplicate root, self/missing edge, multi-node cycle, dependency-evidence-only root, whole-budget and section-budget exhaustion. Full reconstruction/fingerprint parity remains unverified. |
| R6 / T2, T10–T12 | `canonical_artifact.rs`, header/container/metadata owners; reader-owned ABIs/capabilities and independently derived provenance agree across sections. | Reader compares package identity, semantic/stable/module/artifact/query/native/metadata ABIs and capability set; section digests are recomputed; builder computes source digest from canonical source provenance. | Source digest on load is only checked nonzero because source is not carried; complete source/package/provenance relationship and cross-section semantic owner agreement are not independently established by the loaded-image test. Add correctly rehashed incompatibility and provenance parity coverage. | Mutate/re-hash ABI, capability, package and provenance independently; verify precise typed errors. | Foreign native-catalog fingerprint rejection before section materialization passed with correctly rehashed V2 bytes. Other incompatible-header mutations remain unverified because the full builder stops at callable-analysis export coverage. |
| R7 / T10–T12 | `canonical_artifact.rs` transaction plus module/semantic/query owners; one cumulative whole-load ledger and independent section/stage caps cover acquisition through publication. | Core threads one `AllocationBudget` through container strings/directory, section decoders, selected module/semantic/query constructors, metadata and selected publication `Arc`s. `max_total_allocation_bytes` now independently caps the transaction; query closure uses a nested stage-limited child ledger and commits only after success. | Many nested/native output maps, relocation/indexes, formal-image construction and semantic owner scratch remain uncharged. Continue exact fallible ownership boundaries; never reset the transaction at a stage. | Individual-stage fit but combined failure; one-stage overflow with whole-load headroom; independent allocator request census over a real Universe load and retry. | PASS for query closure `N-1` rollback/section-cap independence, exact container identity/directory requests, and module DAG scratch budgeting. Full transaction remains unverified; the cumulative core load discriminator is blocked at incomplete callable-analysis export. |
| R8 / T12 | `phalcom-core/src/modules/compile.rs`, `phalcom-modules/src/resolver.rs`, and `CanonicalExternalModuleWorld`; loaded Universe authority must precede canonical source providers while preserving exposure checks. | `ModuleResolver::with_external_modules` installs no Universe source provider; `universe_location` and `load_interface` read completed external interfaces, and package exposure validation uses those interfaces. Dependency discovery invokes `resolve_import` and skips Universe traversal after it returns. | This source path is structurally source-free, but actual compiler precedence/counters and semantic authority use under explicit imports have not been verified in a child process. Add a real imported program and fail-fast provider instrumentation on actual acquisition paths. | Fresh process loads `universe.telosc`, compiles a novel imported program with source denial, and asserts all canonical source-work counters are zero. | `build_and_load_universe_telosc_compiles_source_free_inline_user_entry` is among the 6 tests blocked at export; no source-denied process evidence. |
| R9 / T11 | `canonical_artifact.rs`; writer validates actual supplied bytes and full owner relationships before replacing a prior artifact. | Writer reloads/validates actual `built.bytes` before creating a same-directory `NamedTempFile`, then syncs and persists it. | Need a built image that reaches full owner validation plus malformed cross-section/reference mutation and prior-output preservation coverage. | Existing-output preservation after actual-byte, digest-valid semantic-reference corruption, and injected failure. | `writer_preserves_previous_output_when_bytes_are_invalid` passed in the focused core run; owner-complete validation remains blocked by builder coverage. |
| R10 / T11 | `canonical_artifact.rs`; each concurrent write owns a unique exclusive same-directory temporary file and cleans only its own file. | Uses `tempfile::NamedTempFile::new_in` and `persist`; no PID-derived shared path remains. | Concurrent-writer and injected write/sync/rename cleanup behavior lacks a passing discriminator in the current tree. | Concurrent writes to one destination plus injected failures while a prior valid output exists. | No concurrency/failure-injection test run; implementation appears corrected but acceptance is open. |
| R11 / T12 | `canonical_artifact.rs`; reject oversized files before unbounded allocation and enforce the cap while reading growth. | Checks metadata length before reserving, then performs checked chunk reads against `max_artifact_bytes` and budgeted growth. | Need disk-path growth-race/independent allocation evidence; do not infer it from in-memory parsing. | Disk oversized file with tiny limit and a file-growth read discriminator. | `bounded_file_acquisition_rejects_oversized_file` passed; growth-race evidence remains absent. |

### R3 denotation transport — latest focused implementation evidence

Supersedes the earlier R3 matrix note that denotation transport tests were not
run. Type/kind/domain roots are now registered before graph closure; imported
associated exact/family and behavioral captures retain relocated type refs,
captured member order, optional targets, operation shapes, specs, and conditional
selections. Associated captures now validate both lookup-owner agreement and
primary member authority. Static-definition checked applications use the
enclosing `StaticBodyId`; two different enclosing bodies with ordinal zero are
covered. This producer correction is limited to checked static-definition
bodies: checked application rows are still not retained in `CallableAnalysis`,
and StaticApplication wire/materialization remains fail-closed.

Completed focused evidence:

```text
cargo test -p phalcom-semantic --lib denotation -- --test-threads=1                  PASS (6)
cargo test -p phalcom-semantic --lib behavioral_family_round_trip_preserves_specs_targets_and_optional_selections -- --test-threads=1  PASS (1)
cargo test -p phalcom-semantic --lib callable_denotation_import_preserves_ordinals_and_commits_only_complete_records -- --test-threads=1  PASS (1; exact fit/N-1 rollback)
cargo test -p phalcom-semantic --test semantic session_publishes_checked_static_body_with_stable_const_reads -- --test-threads=1  PASS (1)
cargo test -p phalcom-compiler-artifact --test semantic_world_callable_analysis_test -- --test-threads=1  PASS (9)
cargo check -p phalcom-compiler-artifact -p phalcom-semantic --lib                 PASS (inherited unused-import warning)
git diff --check                                                                   PASS
```

The semantic-world tests include missing-primary-member and wrong-owner
rejection. These codec/root tests do not establish complete CallableAnalysis
parity, compiler projection, source-denied loading, or P7 acceptance.

### R3 CallableAnalysis type-root closure — focused implementation evidence

`phalcom-semantic/src/artifact/roots.rs::register_callable_analysis_roots` now
registers the canonical graph roots carried by expression/binding TypeKnowledge,
binding contracts, entry/return/throw flow summaries, sequence literals,
known-invalid positional expansions, associated resolutions, callable
references, family applications, and for/positional iteration requirements.
Iteration requirements retain signature and generic-conformance roots;
hash-map substitutions/environments are normalized by TypeParameterId before
root registration. StaticApplication remains fail-closed.

Focused evidence:

```text
CARGO_INCREMENTAL=0 RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib callable_analysis -- --test-threads=1  PASS (15 selected)
rustfmt --check phalcom-semantic/src/artifact/roots.rs phalcom-semantic/src/artifact/import.rs                 PASS
git diff --check                                                                                               PASS
```

The tests prove the listed supported TypeIds are retrievable after graph
closure, including roots reached only through expression/binding/flow and
resolution products. They do not prove root completeness for the remaining
manifest fields, wire export/import, fresh-store parity, or compiler reader
reconstruction. The iteration test exercises compiler-derived conformance
evidence; exact, assumed, and derived evidence paths still need direct root
discriminators. The callable-analysis export/import completeness guards are
unchanged.

VERIFY-FIRST for the next StaticApplication slice: live `analyze_static_body_inner`
sets `current_static_body_owner` and moves checked application rows into the
owning `CheckedStaticBody`; ordinary callable finalization does not consume that
path. `OperationalTemplate` lowering retains target/arguments but drops the
`StaticApplicationId` and full signature from its wire node. Reconcile this
live owner boundary with the adopted T7.B text before adding another owner or
removing a fail-closed guard.

### P5 `RequiredSemantic` family census

The source of family classification is `phalcom_semantic::canonical_image::CANONICAL_PERSISTENCE_INVENTORY`; the field-level owner list is `phalcom-semantic/src/artifact/manifest.rs::SNAPSHOT_FIELD_MANIFEST`. Dispositions below describe current live adapters, not acceptance. `MISSING` includes a partial summary DTO when compiler-required subfields are absent.

| P5 family / compiler-required products | Current disposition | Owning adapter / unresolved consumer |
|---|---|---|
| Modules/interfaces; imports, links, topology and dependency graph | `PERSISTED` (behavior unverified) | `phalcom-modules/src/compiler_artifact.rs`; source-free import/package exposure and complete module parity remain unverified. |
| Type arena, generic binders/signatures/constraints | `PERSISTED` (coverage partial) | `phalcom-semantic/src/types/artifact.rs`, `artifact/export.rs`, `artifact/import.rs`; current roots cover emitted rows, but roots for absent owner families are not closed. |
| Static-value arena | `PERSISTED` (coverage partial) | `phalcom-semantic/src/static_value/artifact.rs`; canonical owner-root closure and full image parity remain unverified. |
| Declarations, aliases, hierarchy | `PERSISTED` (behavior unverified) | `artifact/aggregates` codecs and semantic import; field-level source-built parity has not passed. |
| Callable signatures | `PERSISTED` (partial) | `SemanticWorldV1.callable_signatures`; import currently does not restore every parameter/signature field (including persisted external labels) and has not been parity-tested. |
| Accepted callable definitions and inherent context | `MISSING` | Import synthesizes primary-declaration definitions from signatures; provenance, origin, source-member index and inherent body context are not transported. |
| Callable analyses: expression/binding facts, CFG/flow, targets, dependencies/fingerprints, analysis states/attachments | `MISSING` (partial wire) | The current row carries status, unreachable-exit state, constructor-super attachment, binding origins/formal attachments, expression count, and typed static call arguments. It still omits the rest of the body graph; export's completeness guard and import's nonempty-analysis rejection remain active. Compiler binding/site/target reads must be rebuilt from the relocated owner graph; body/query fingerprints are recomputed after relocation. |
| Field signatures, declaration surfaces, dispatch primary facts | `MISSING` (partial DTO) | `dispatch_surfaces` persists callable IDs/field signatures but importer assigns assumed `TypeId(0)` callable knowledge; no full surface parity. |
| Constructor initialization | `PERSISTED` (behavior unverified) | `SemanticWorldV1.constructor_initializations`; owner-field identity and fingerprint parity unverified. |
| Trait headers/surfaces and requirement views | `MISSING` (partial DTO) | `trait_surfaces` omits full member signature/visibility/source/default authority; conformance witness requirement views are absent. |
| Conformance heads, witness plans, derived rules and trait dispatch | `MISSING` (partial DTO) | Head DTO preserves target/provenance; witness fields are dropped/defaulted; derived conformance/trait dispatch owners are not imported. |
| Data/enum semantics, enum requirements, static case catalog/signatures | `MISSING` (partial owner DTO) | `StructuralPrimaryContractsV1` carries data/enum/variant structure. Enum requirement rows/statuses now have wire/import/export code, but their signature is only a stable callable reference and the source owner does not publish that row in `CallableSignatureTable`; source parity is blocked. Static case catalogs/signatures remain absent. |
| Associated surfaces, type members, associated static domains | `PERSISTED` (partial verification) | Explicit DTOs and import into these snapshot owners exist; current end-to-end loaded artifact is blocked before publication. |
| Static definitions/constants and module static-binding catalogs/bindings | `MISSING` | Checked templates are returned as detached `operational_templates`; published snapshot tables and module bindings are empty. |
| Secondary owner indexes | `RECONSTRUCTED_FROM_PERSISTED_AUTHORITY` (partial) | Owning callable/data/enum/declaration tables build indexes during insertion; additional debug-string keyed indexes are detached in `ValidatedSemanticParts` and are not consumed. |
| Immutable reusable query products and dependency evidence | `PERSISTED` (partial) | `QUERY_IMAGE` DTO exists; closure is unbudgeted, some eligible adapters/evidence are incomplete, and fingerprint/edge census is unverified. |
| RuntimePublic metadata | `PERSISTED` (budget/provenance unverified) | Existing metadata schema/validator is used; full allocation-request closure is missing. |
| Compiler source attachments / operational templates | `MISSING` | Typed operational wire DTO exists, but templates are not installed into actual `StaticDefinitionTable` / `StaticConstantTable`; callable facts are incomplete. |

This census keeps the exporter’s fail-closed completeness guard in place. No absent row is classified as reconstructed merely because the importer initializes a default table.

---
id: PERF003.C2.P7.EXECUTION
program: PERF003
checkpoint: PERF003.C2
phase: PERF003.C2.P7
status: IN_PROGRESS
implementation_status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
prepared_against_revision: 3ca30fca
prepared_against_working_tree: true
depends_on: [PERF003.C2.P5, PERF003.C2.P6]
autonomous: true
execution_mode: implementation-and-focused-acceptance
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
      status: superseded
      verification: user-authorized-completion-request
---

# P7 remaining execution plan

## Adopted Wave 4 prerequisite amendment — 2026-10-14

The user authorized this agent to act as principal architect, amend P7, and
implement the prerequisite corrections before T11/T12, inline with no further
subagents. This Level 2 amendment adopts the dependency corrections in
[the incident/advisory decision](PERF003.C2.P7-wave4-prerequisite-incident.md).
It preserves the original architecture and invariants. Earlier declarations of
complete T7.B–T10 takeover authority and all compiler-only test restrictions
in this companion are superseded by this section.

FIXED: the full compiler profile, owner-controlled DTOs/materialization, stable
portable identities, shared closed type/static relocation contexts, immutable
loaded queries, existing metadata schema/codec, and no source fallback or
default-filled required tables. Retain `is_source_free_artifact_complete`.

MECHANICALLY FLEXIBLE: private codec decomposition, validated insertion helpers,
and unpublished DTO fields consistent with the complete required manifest.
VERIFY-FIRST: every primary family in `SNAPSHOT_FIELD_MANIFEST` and every
compiler attachment has a complete writer and materializer before composition.

Execution order and takeover contracts:

1. T10 may be corrected independently first: content-derived streaming metadata
   preflight precedes collection decoding; counts/schema/profile/identity must
   agree with the actual bundle; charge the shared budget and reuse its real
   codec/validator. Produce a validated metadata section/bundle interface.
2. Reopen T7.B and shared graph orchestration: add explicit portable primary
   DTOs for the full manifest and operational attachments, register every root
   before closure, and emit the type graph from that same frozen context.
   Static roots use stable owner/product keys, never native allocation ordering.
   Produce complete semantic-owned snapshot materialization with fresh lineage.
3. T8: reconstruct every required owner/index family from the inserted primary
   authority. Detached declaration/data/enum maps do not fulfill this contract.
4. T9: typed portable query keys and product/aggregate references; reconstruct
   current relocated products/fingerprints and dependency-only evidence.
   Reverse-prune source-dependent reusable roots; never silently drop an edge.
5. T11/T12: compose only the completed above interfaces. Reader self-validation
   precedes output/publication. Add explicit loaded-image compiler injection,
   audit default acquisition, and use a user-only immutable-parent overlay.

Required focused stabilization: metadata round trip and forged-count/schema/
profile/identity/budget/trailing-input failures; primary-family round trip with
fresh stores; different allocation order and unreachable-node discriminator;
typed query relocation/closure/fingerprint/pruning checks; independent-process
writer determinism and atomic failure preservation; source-denied child-process
loading followed by novel user compilation with zero canonical source work and
mutable canonical query rows; scoped core/LSP and directly affected suites.
Run Cargo validation serially. Do not infer broader certification from these
gates. Checkpoint/walkthrough/handoff record actual completed evidence.

### Adopted T10 allocation clarification and user stop

The T10 content/schema correction is implemented and focused-tested. Full T10
shared-allocation acceptance is still open: current charging covers encoded
bytes only, not decoded tables and validator scratch. Do not certify full T10
or compose a complete compiler image from this subunit alone.

The same adviser disproved a proposed `2 * encoded bytes + token * row-size`
reservation: validator owner formatting retains multiple variable-length string
copies and identity indexes. No such estimate was implemented. Next implement
an owning `phalcom-type-meta` budgeted direct current-schema decode seam and
account for collection/string capacity and validator scratch before allocation;
prefer bounded/borrowed identity indexes. Keep legacy normalization on the
existing legacy path. Caller budget commits only after successful validation.

Required future discriminators: large owner strings with fixed token count,
many small/empty containers, generic-parameter/static-domain index workloads,
insufficient-budget rejection before table allocation, unchanged budget after
failure, and representative actual Universe metadata decode under the applicable
default limits. Do not raise defaults to disguise unsupported estimates.

The user explicitly requested stopping this agent after the current T10
correction and producing a detailed handoff. Subsequent prerequisite work and
T11/T12 are not performed in this continuation. No new subagents may be spawned;
the same adviser may be reused for mandatory repository-workflow consultation.

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
  <acceptance_criteria>Metadata wrapper preserves the existing schema and stable identity authority; decoding is bounded and SessionLocal metadata is rejected; native tables, strings, and validator scratch are budgeted before construction.</acceptance_criteria>
  <verify>cargo test -p phalcom-type-meta (28 passed) and cargo test -p phalcom-compiler-artifact (25 passed).</verify>
  <done>Metadata wrapper preserves the existing schema and stable identity authority; direct current-schema decoding avoids Value intermediate; allocation budgeting covers table/string/scratch memory with rollback on failure. Implemented and focused-tested.</done>
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
  <verify>cargo test -p phalcom-core --test core modules_canonical_artifact (4 passed); determinism and atomic write verified.</verify>
  <done>build_universe_telosc produces bitwise deterministic bytes across builds; write_universe_telosc publishes atomically via same-directory tempfile; self-validation ensures container integrity. Implemented and focused-tested.</done>
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
  <verify>cargo test -p phalcom-core --test core modules_canonical_artifact (inline user compilation succeeds against loaded image without Universe source acquisition).</verify>
  <done>Explicit user compilation consumes the loaded image; no source acquisition occurs on that path; loaded snapshot satisfies is_source_free_artifact_complete; error handling is fail-closed and retryable. Implemented and focused-tested.</done>
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
| G7.2 | Post-change container schema/digests/limits/corruption | Pending focused verification after V2 header/fingerprint change |
| G7.3 | Source-free external module import parity and zero canonical builds | Module code present; acceptance NOT RUN |
| G7.4 | Type/static relocation, fresh stores, equality and prefix extension | Type code present; static transport open; acceptance NOT RUN |
| G7.5 | Exhaustive semantic primary parity and required attachment transport | NOT RUN |
| G7.6 | Immutable query closure/aggregate agreement and no recomputation | NOT RUN |
| G7.7 | RuntimePublic metadata transport and no exporter/pool IDs | NOT RUN |
| G7.8 | Source-unavailable fresh-process novel user semantic compilation | NOT RUN |
| G7.9 | P5/P6 immutable-copy/local-complexity non-regression | NOT RUN for artifact path |
| G7.10 | Byte-identical fresh builds under root/allocation variation | NOT RUN |
| G7.11 | Separate load timings and materially cheaper cold startup | NOT RUN |

The original plan's named acceptance commands remain authoritative. The user has authorized implementation and verification in this continuation; select focused discriminators first and broaden only at named P7 gates. P7 COMPLETE remains blocked by every unclosed R1–R11 item and deferred acceptance.

## Flexibility and stop rules

FIXED: canonical owners, stable identity, fresh store lineage, validation before publication, immutable overlays, source-free authority, BLAKE3 framing and bounded explicit tags. MECHANICALLY FLEXIBLE: helper names/private file splits, equivalent public parameter grouping, and owner bulk insertion APIs. VERIFY-FIRST: checked template/source attachment dependencies, root graph closure, metadata canonical encoding, and genuine static/primary cycles.

Stop and consult on any required source-body regeneration, semantic solving during load, unsupported legal product, unsafe partial publication, uncertain canonical identity/GC contract, or material downstream redesign. Mechanical compiler failures allow three coherent correction cycles; do not use acceptance tests to debug an intentionally incomplete slice. Classify unrelated failures A/B/C/D and preserve inherited warnings. Do not broaden check scope merely because a check passed.
