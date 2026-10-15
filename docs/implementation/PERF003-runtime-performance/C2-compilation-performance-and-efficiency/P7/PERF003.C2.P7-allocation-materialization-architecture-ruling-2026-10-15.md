# PERF003.C2.P7 — Native allocation/materialization architecture ruling

**Date:** 2026-10-15  
**Decision:** APPROVED architecture; implementation NOT STARTED by this ruling  
**Incident:** [PERF003.C2.P7-allocation-map-accounting-incident-2026-10-15.md](PERF003.C2.P7-allocation-map-accounting-incident-2026-10-15.md)  
**Audited committed base:** `9e0a86a1249276099602b65716b7bdc5681fd7dd`  
**Plan/gates:** PERF003.C2.P7, R1/R7; G7.2, G7.7, G7.8, G7.11 and final P7 acceptance  
**Status:** Architectural blocker RESOLVED BY DECISION; allocation correctness PARTIAL/UNVERIFIED; P7 remains IN_PROGRESS.

## 1. Decision and non-negotiable contract

Maintain the adopted **one whole-load transaction accounting cumulative allocator-requested bytes**, including replacement layouts on growth, temporary storage, native relocation/indexing, decoding, error construction where allocation occurs, and publication. A successful load cannot exceed the configured whole-load limit. Each section/stage also retains an independent local limit. Debit each requested `Layout` before the allocation request; retain zero refund for scratch during a successful transaction. Child ledgers commit only on success; a failed load publishes no artifact and does not commit externally visible budget state. Count requested layouts, **not** allocator usable capacity, resident memory, logical key/value bytes, or container entry counts.

For **immutable artifact-owned ordered authority**, authorize introducing a **budgeted sorted-vector map/set representation behind source-independent read interfaces** and migrating only the loaded/native product boundaries that need it. For **temporary validation/index scratch**, use bounded, explicitly reserved vectors, bitsets or dense-index worklists wherever possible. Do not build `std::collections::BTreeMap`/`BTreeSet` in the charged load path and treat its unobserved node requests as covered. Where ordered structures are genuinely required, construct them through an allocator-visible and *fallible* interface with proved precharge, otherwise keep this gate open.

This decision **does not** authorize a global collection refactor, wire-format change, change in semantic/query authority, source-backed fallback, weakening the byte contract, changing the canonical ordering, replacing persistent maps indiscriminately, or claiming that instrumented wrappers alone constitute correctness.

## 2. Why the current approach cannot be completed by bookkeeping

`phalcom-type-meta/src/allocation.rs` has explicit-layout helpers for `Vec`, `Box<T>`, strings and `Arc<T>`. Standard-library ordered maps allocate private B-tree nodes on insertion and splits; their public stable API does not expose requested node layouts before each request. Estimating `size_of::<(K,V)>() * len`, hard-coding node capacities/layouts, or charging an artificial per-entry amount is not actual requested-byte accounting. Calling a custom allocator on an alternate B-tree does not by itself establish *fallible* propagation: if `insert` aborts on allocation failure, it cannot satisfy the loader's typed resource-limit rejection contract.

Stable `BTreeMap` custom allocator construction remains unavailable in the referenced Rust API. The third-party `arena-btreemap` port is **not selected** in P7: its dependency/unsafe surface, fallible mutation behavior, absence of a matching B-tree set, API migration and performance must first be independently audited. Nightly-only allocator APIs and production global allocator hooks are not authorized. Test-only counting instrumentation is permitted **solely as verification**, not as a production accounting mechanism.

## 3. Implementation contract for ordered materialization

Introduce an internal `BudgetedSortedMap<K,V>` / `BudgetedSortedSet<K>` (name flexible) or an equivalent read-only collection. The storage is a single explicitly budgeted `Vec<(K,V)>` / `Vec<K>`, without an uncharged final conversion. Construction must:

1. Validate item and encoded limits *before* reserving. Compute `Layout::array::<T>(count)` by checked arithmetic. Debit the load's child transaction before calling `try_reserve_exact`; for each growth, debit the full newly requested layout, not merely the delta.
2. Append through a non-allocating path after sufficient explicit reservation. Clone or decode *owned* keys/values only through separate budgeted helpers; reserving the pair buffer does not pay for nested strings, `Box`, `Arc`, `Vec`, maps, or identity clones.
3. Sort **in-place with `sort_unstable_by`**, avoiding the heap scratch possible with stable sorting. Check duplicates/ordering with an adjacent scan; preserve each existing family’s required duplicate rejection and semantic order. Do not silently introduce last-wins or first-wins behavior.
4. Seal the vector and expose the read operations needed by consumers (`get`, `contains_key`, `iter`, `keys`, `values`, `len`, bounded `range`) through a narrow adapter. Binary search is acceptable for frozen authority. If a consumer genuinely needs insertion/entry mutation, keep a separately accounted builder or redesign that stage; do not reconstitute a hidden native B-tree.
5. Preserve stable owner identity, fingerprints, privacy and exposure semantics. Loaded-versus-source-built forms may have different physical backing only **behind a common canonical read contract**, with parity tests. Do not change general source-built collection representations as an incidental P7 repair.
6. Account for `Vec` ownership conversions: no unexamined `into_boxed_slice`, `Arc::from`, `collect`, `format!`, `clone`, or `to_owned` on the untrusted load path. Either precharge a verified requested layout or retain the allocation as an explicit unresolved gap.

A canonical flat map is a *physical representation* choice only. It does not change the semantic ownership, revision model, query authority, persisted `.telosc` schema or canonical iteration order. Audit binary-search latency and P5/P6 locality relative to existing source-built maps before adopting it for high-frequency readers.

## 4. Concrete file-by-file correction order

**A. Budget foundations — `phalcom-type-meta/src/allocation.rs`.** Add checked allocation helpers for borrowed-key arrays, dense worklists, owned strings/identities, exact nested slice/box/arc construction, and staged sorted vectors. Distinguish the requested new layout on growth from the allocator's *usable* block size; debit growth requests before mutating buffers. Verify failed reservation leaves the caller-visible budget unchanged. Keep section child ledgers and parent commit semantics explicit.

**B. Container — `phalcom-compiler-artifact` container parser and `phalcom-core/src/modules/canonical_artifact.rs`.** Add `TeloscContainerView::parse_with_allocation_budget` and charge the directory/scratch on the same transaction before parsing/materialization. Existing borrowed public parser wrappers may create local budgets, but the artifact load entry point must not. Separate the whole-load limit from section/metadata limits in load options; reusing `max_decoded_section_bytes` as the whole-load budget without an independently configurable policy is not adequate proof of independent limits.

**C. Modules — `phalcom-modules/src/compiler_artifact.rs` and canonical-product read boundaries.** Replace load-time B-trees for the interfaces/linked modules/topology/graphs and validation indexes with sorted-vector or dense-index builders. Provide an explicit adaptation path for `CanonicalExternalModuleWorld`, `LinkedProgram`, `ModuleTopology` and nested interface-owned maps/sets; a top-level adapter does not account for their nested allocations. Budget component-owned strings, cloned names, import surfaces and metadata. Preserve module/runtime DAG and source-independent fingerprint verification.

**D. Query — `phalcom-semantic/src/artifact/query.rs`.** Rework `validate_query_image_closure` to accept the ledger and avoid the `BTreeSet`-plus-leaf-rescan algorithm. Duplicate keys can be checked with a sorted, budgeted vector of borrowed keys/indices. Validate DAG closure using bounded three-color iterative DFS or a dense adjacency/indegree worklist, not repeated global scans; budget stack, colors and any adjacency buffers. Replace reconstructed query maps/fingerprint maps with sealed ordered products behind the canonical query-read interface. Account for per-dependency arcs, serialized owner deserialization and error paths. Keep product/aggregate parity and exact dependency evidence.

**E. Semantic — `phalcom-semantic/src/artifact/import.rs` and owning import functions.** Thread the same ledger into `TypeStore::import_type_graph`, static-value/primary-contract import, snapshot/index constructors and their reachable allocators. Use explicit stage-owned scratch, fallible constructors, and budgeted frozen ordered authority where possible. Prioritize allocations from untrusted counts and high-cardinality owner maps. Budget `TypeStore` clones, semantic snapshot and formal-image `Arc` construction, including `CanonicalUniverseFormalImage::from_loaded_artifact`.

**F. Metadata/publication.** Prove each request is charged **once** (avoid both missing charges and duplicate logical charges), across lexical preflight, decoding, validation, native table construction and publication. Account for final arcs only when they actually request storage. No status promotion until allocator-observed proof covers the whole path.

## 5. Required discriminating tests

1. **Requested-byte equivalence:** in an isolated, single-threaded, test-only counting-allocator harness, compare cumulative native allocation requests against the ledger for representative full loads, across small/large/split-heavy maps, scratch-heavy query DAGs, malformed data and successful publication. Exclude intentional test-harness allocations by scope; keep the test-only hook out of production.
2. **Tight boundary:** for valid nonempty sections, budget `N` succeeds and `N-1` fails with the appropriate typed allocation error; test section-local exhaustion while whole-load headroom remains, and cumulative whole-load exhaustion while individual stages fit.
3. **No undercharging/no double charging:** assert independent requested-layout measurements, including vectors that grow, nested strings, arcs, errors and temporary buffers. A charge based on the implementation ledger's own reported debit is not independent evidence.
4. **Failure atomicity:** on exhausted/overflow/invalid inputs, parent budget is not committed; no partial artifact/cache publication; retry from fresh or retained previous valid state works.
5. **Semantic parity:** compare every reconstructed module/semantic/query owner, dependency edge, fingerprint, canonical order, privacy/exposure decision and runtime DAG against source-built authority. Include large randomized insertion orders, duplicates, and disconnected/cyclic graphs.
6. **Performance:** measure memory requested, allocation count, relocation wall time and hot lookup latency. Verify P5/P6 local-only compilation counters and P7 cold-load improvement, not just zero panics.
7. **Final acceptance remains separate:** source-denied fresh-process novel user compilation, cross-process byte determinism and the existing G7.1–G7.11 suite are still required.

## 6. Rejected shortcuts

- `size_of::<(K,V)>() * entries`, private std B-tree node mirroring, guessed overhead factors, capacities inferred from existing maps, or charging only encoded DTO bytes.
- Arbitrary maps/sets assembled with `collect()` or `insert()` after charging only the outer object.
- Swapping `std::BTreeMap` for a custom-allocator collection without fallible out-of-budget semantics and without carrying the budget into nested ownership.
- Global allocator interception in production, nightly-only allocator features or catch-unwind around allocator errors.
- Incrementally resetting a whole-load ledger for each section, ignoring scratch because it is freed, or falsely marking the gate closed on `cargo check` alone.

## 7. Ownership, rollout and stop conditions

**Proceed in bounded stages:** (1) foundational budget helpers plus counter probes, (2) container and query scratch, (3) module map/owner adapter, (4) semantic/type/static relocation and snapshot, (5) metadata/publication, (6) exhaustive full-load verification and performance/parity acceptance. Each merged stage must preserve the current source-free image contract and explicitly enumerate remaining uncharged allocations. Do not begin a wholesale migration of runtime maps or general collection APIs.

The current committed shared-budget plumbing and selected explicit-layout charges are useful intermediate work. They are **not** a proof of a bounded native load. The architectural consultation is concluded by this ruling, but **R1/R7 and PERF003.C2.P7 remain OPEN / PARTIAL / UNVERIFIED** until the implementation and independently measured tests satisfy the contract. Do not promote P8 takeover on the basis of this decision alone.
