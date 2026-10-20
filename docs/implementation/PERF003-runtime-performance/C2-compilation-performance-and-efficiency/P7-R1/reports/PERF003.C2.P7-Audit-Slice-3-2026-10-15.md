# PERF003.C2.P7 — Audit Slice 3 Report

**Topic:** `.telosc` container integrity, allocation safety, compiler-artifact load transaction, deterministic portability, query-graph complexity, and fault-path verification  
**Audit date:** 2026-10-15  
**Repository:** [`aureat/phalcom-lang`](https://github.com/aureat/phalcom-lang)  
**Branch:** `perf/universe-characterization`  
**Pinned audited commit:** [`2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a`](https://github.com/aureat/phalcom-lang/commit/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a) (`feat(perf): extend artifact loading foundations`, 2026-10-15 12:25:39 UTC)  
**Report relationship:** [Slice 1](PERF003.C2.P7-Audit-Slice-1-2026-10-15.md) established the artifact end-to-end gates; [Slice 2](PERF003.C2.P7-Audit-Slice-2-2026-10-15.md) audited semantic-owner fidelity. This third slice audits *binary, resource, and algorithmic correctness*.  
**Review mode:** Read-only connector inspection of the pinned GitHub revision plus locally provided P7 handoff, execution plan and prior slice reports. **No Cargo tests, fuzz tests, memory probes, child-process compiles, repository edits, or benchmarks were run.**

> **Verdict:** **P7 NOT READY FOR CLOSURE.** In addition to the source-proven semantic-publication blockers from Slices 1–2, the active loader can allocate native module-owner structures outside the supposedly cumulative budget *before* reaching its semantic fail-closed boundary. The binary container accepts noncanonical byte sequences that share the same declared artifact digest. Query reconstruction contains additional unbudgeted owner decoding and output construction, and query export has a provable quadratic worst case. The implementation has valuable foundational safeguards; removing semantic guards alone would be unsafe.

---

## 1. Executive summary

### 1.1 Critical path

The actual load order is:

```text
load_universe_telosc(path)                         [file acquisition; shared ledger]
  -> TeloscContainerView::parse_with_allocation_budget
       -> header strings + directory (charged)
       -> section/whole-artifact digest checks
  -> validate_reader_header; section provenance digests
  -> ModuleWorldV1::decode_with_allocation_budget    [charged DTO]
  -> load_module_world_with_allocation_budget       [PARTLY charged native owners]
  -> SemanticWorldV2::decode_with_allocation_budget [unconditional accounting rejection]
  -> validate_and_import_semantic_world_with_allocation_budget
                                                       [second unconditional rejection]
  -> QueryImageV1::decode_with_allocation_budget
  -> reconstruct_query_image_with_allocation_budget [PARTLY charged native products]
  -> MetadataSectionV1::decode_with_allocation_budget
  -> decode_bundle_with_allocation_budget
  -> CanonicalUniverseFormalImage::from_loaded_artifact
  -> Arc<LoadedUniverseArtifact>
```

See [`canonical_artifact.rs` lines 265–405](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L265-L405), [`world.rs` lines 609–669](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L609-L669), [`import.rs` lines 1790–1839](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1790-L1839).

**Consequences:** A test that observes a failed full load does *not* establish correct resource-bound enforcement: it might fail only at a semantic guard, long after uncharged module allocations. Similarly, valid section digests do *not* currently establish that all bytes of the file belong to the canonical container layout.

### 1.2 Findings at a glance

| ID | Severity | Confidence | Finding | Ownership / gate |
|---|---|---|---|---|
| S3-01 | **HIGH** | Confirmed control flow | Unreferenced trailing and gap bytes are excluded from the whole-artifact digest, yet accepted by the reader. | Container canonicality / integrity |
| S3-02 | **HIGH** | Confirmed source | Reader accepts inconsistent `decoded_len` for uncompressed sections and does not impose the advertised decoded-length constraint at directory parse. | Container resource metadata |
| S3-03 | **MEDIUM–HIGH** | Confirmed source | Unknown capability bits are erased with `from_bits_truncate`, bypassing the core's otherwise strict capability equality check. | Compatibility / forward safety |
| S3-04 | **CRITICAL** | Confirmed source | Module-world materialization allocates maps, sets, strings and graph products outside the shared ledger *before* the semantic guard. | R1/R7/T10–T12 |
| S3-05 | **CRITICAL (latent after guard removal)** | Confirmed source | `TypeStore::import_type_graph` and nested type validation use native allocation and local estimation, not the load transaction. | R1/R7/T4–T5 |
| S3-06 | **HIGH (latent)** | Confirmed source | Query key JSON parsing, native product cloning, `BTreeMap` construction and `Arc` products bypass shared accounting. | R5/R7/T9 |
| S3-07 | **HIGH** | Confirmed source | Some decoder allocation failures are converted into freshly allocated `String` diagnostics rather than typed allocation errors. | Failure semantics |
| S3-08 | **HIGH** | Confirmed source / design | Query DTO decode admits counts up to generic semantic-family limits and only checks narrower query limits later. | Query preallocation / R5 |
| S3-09 | **HIGH (latent)** | Confirmed source | `dependency_evidence` is serialized but is not consumed by the query-image reconstruction path; duplicate/wrong-owner evidence lacks full import closure. | Query authority fidelity / R5 |
| S3-10 | **MEDIUM–HIGH** | Confirmed algorithm | Export's iterative reverse-prune can take quadratic work on dependency chains with omitted leaves. | Performance / T9 |
| S3-11 | **MEDIUM–HIGH** | Confirmed source | Writer retains original section buffers, clones all four, then builds another complete byte buffer, increasing peak memory and copying. | Build performance / T11 |
| S3-12 | **MEDIUM** | Confirmed source | Container reader does not demand canonical section tag order or required-section flags; noncanonical layouts pass with recomputed digests. | Determinism / canonical bytes |
| S3-13 | **MEDIUM** | Confirmed source / conditional | Atomic rename is used with file `sync_all`, but parent directory durability is not synced; power-loss durability is unproven. | Artifact publication |
| S3-14 | **HIGH** | Confirmed test gap | Current unit probes cover selected charged operations but not complete live Universe load, wrong-budget allocation requests, or strict-byte canonicality. | P7 closure gates |
| S3-15 | **RELEASE BOUNDARY** | Confirmed limitation | Loader only checks a nonzero source digest; it cannot prove freshness against changed source because source bytes are absent. | P7/P9 responsibility |

**Severity note:** “latent” means a defect remains behind a deliberate unconditional guard and must be fixed *before* lifting that guard. “Confirmed control flow” is a claim about what the checked source permits, **not** a claim that a new binary test was run.

---

## 2. Binary container and compatibility findings

### S3-01 — Trailing/gap bytes can share the same artifact digest (**HIGH**)

**Evidence:** [`container.rs` lines 146–330](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/container.rs#L146-L330). The reader verifies that each section is in bounds, outside the header/directory and nonoverlapping. The whole-artifact digest is then recomputed by hashing: header with zeroed digest field, directory bytes and **the referenced section slices in directory order**. It does not hash bytes unreferenced by those ranges, nor demand that referenced ranges exactly cover the suffix of `data`. There is no final condition such as `last_section_end == data.len()`.

**Constructive counterexample (not executed):** Take a valid `TeloscContainerBuilder::build()` result `A`; define `B = A || b"trailing junk"`, remaining within `max_artifact_bytes`. The header, directory, each referenced section, and their digest computations remain unchanged. The parser's upper-bound check passes, and its recomputed artifact digest is unchanged. Thus `A != B` but both can be accepted with the same declared artifact digest. Gaps between directory and sections or between section ranges are similar if offsets and lengths are adjusted and the directory/header digest recomputed.

**Impact:** The declared digest is not an identity for the *complete file bytes*. This matters for artifact deduplication, canonical build comparison, cache placement, replication, and any future signature over the artifact digest. This finding does **not** mean a referenced semantic section can be modified without triggering a digest mismatch.

**Required correction:** Define canonical physical layout: first section starts immediately after directory; each subsequent section starts exactly at the previous section end; final end equals `data.len()`. Keep distinct future extension points as **explicit tagged sections**, not uninterpreted padding. Alternatively, cryptographically cover the literal entire suffix and document allowed padding, but a strict no-gap layout is simpler for P7.

**Discriminator:** append one zero byte without rehashing; parser must reject it. Also construct valid rehashed directory with an interior gap and valid section digests; reject. Require exact canonical bytes for the writer's output.

### S3-02 — `decoded_len` is not bound to uncompressed section bytes (**HIGH**)

**Evidence:** [`container.rs` lines 168–240](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/container.rs#L168-L240), [`section.rs` lines 88–110](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/section.rs#L88-L110). Compression is categorically rejected in the current reader, and the builder sets `decoded_len = encoded_len`, but the reader only constrains `encoded_len` against `max_encoded_section_bytes`. It reads and stores `decoded_len` without verifying equality or comparing it with `limits.max_decoded_section_bytes`.

**Impact:** The declared decoded-size authority can disagree arbitrarily with the actual uncompressed bytes, including zero or huge values. This is not a present allocation bomb through this specific field because the reader slices by `encoded_len`; it *is* a manifest/validation contract defect and an unsafe precedent for future compression support.

**Required correction:** At the uncompressed section layer require `decoded_len == encoded_len`, `decoded_len <= limits.max_decoded_section_bytes`, and canonical noncompressed flags. Reject before checksum or native materialization. Preserve separate encoded/decoded caps for future compressed formats.

**Tests:** Rehash a tiny valid container after changing only `decoded_len` to `0`, `encoded_len+1`, or `u64::MAX`. Verify dedicated structural rejection, not eventual owner failure.

### S3-03 — Unknown capability bits silently disappear (**MEDIUM–HIGH**)

**Evidence:** [`header.rs` lines 216–226](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/header.rs#L216-L226) uses `TeloscCapabilities::from_bits_truncate(cap_bits)`. [`canonical_artifact.rs` lines 439–481](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L439-L481) insists on an exact compiler-only capability set—but only **after** decoding has erased unknown bits.

**Impact:** A header asserting a future feature bit can be read as though that bit had never been declared. Whether unknown bits would be mandatory cannot be discerned. This is an ABI/forward-compatibility failure, not an observed exploit of current standard fields.

**Required correction:** On the P7 reader, use checked `from_bits` and reject unknown bits (or preserve raw bits and explicitly classify optional versus required extensions). Retain the exact known compiler-only set comparison after decoding.

**Tests:** Flip an unused bit in an otherwise valid header, recalculate whole-artifact digest and assert rejection. Ensure this is not accidentally classified as a section digest corruption.

### S3-12 — Noncanonical tag ordering and flags accepted (**MEDIUM**)

**Evidence:** Builder sorts by section kind before writing: [`container.rs` lines 39–46](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/container.rs#L39-L46). Parser checks duplicates and overlaps but does not enforce sorted-by-tag order: [`container.rs` lines 166–263](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/container.rs#L166-L263). `REQUIRED` is set in writer section descriptors, but the parser's required-presence logic checks **presence by kind**, not that those present known sections actually carry the mandatory flag.

**Impact:** Multiple structurally different, validly rehashed encodings of the same logical sections are accepted despite the writer advertising deterministic canonical order. Canonicality is a stronger requirement than basic readability: decide whether the reader intentionally supports noncanonical encodings. For compiler cache keys, recommending strict canonical bytes is appropriate.

**Correction:** Enforce strictly increasing section tags and contiguous offsets, plus correct required flags for the four P7 mandatory sections. Clearly document how unknown **optional** tagged sections coexist with the canonical sequence.

---

## 3. Allocation transaction and materialization findings

### S3-04 — Pre-guard module-world native allocations escape the load budget (**CRITICAL / immediate**)

**Evidence chain:**

1. Core reads the module-world DTO, then directly calls `load_module_world_with_allocation_budget()` **before** semantic section decode: [`canonical_artifact.rs` lines 337–350](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L337-L350).
2. Materialization creates native `BTreeMap` and `BTreeSet` values for interfaces, linked modules, topology, exports, graph nodes, children and declarations: [`compiler_artifact.rs` lines 581–744](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-modules/src/compiler_artifact.rs#L581-L744). It `.clone()`s strings and symbols, uses `.collect()` into native maps/sets, and inserts graph products without routing each request through `AllocationBudget`.
3. Selected arrays, DAG scratch, and `Arc<T>` objects are charged: [`compiler_artifact.rs` lines 755–1019](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-modules/src/compiler_artifact.rs#L755-L1019). Charging some allocations does not charge the unaccounted ones.
4. `transaction.commit_child` therefore records only the subset explicitly seen by the budget helper: [`compiler_artifact.rs` lines 550–578](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-modules/src/compiler_artifact.rs#L550-L578).

**Impact:** `max_total_allocation_bytes` is currently **not** an upper bound on allocator-requested bytes even before the intentional semantic fails; a specially shaped module section can provoke map/string/graph allocations outside the debit. This must be treated separately from Slices 1–2's unconditional semantic guard.

**Policy-conformant repair:** Keep the current fixed allocation contract: *cumulative allocator-requested bytes*, including reallocation attempts and temporary scratch, no refunds within a successful load, no promise of RSS capping. Materialize ordered native authorities using pre-budgeted contiguous sorted entries or other owner-consumable charged storage. Where a native `BTreeMap` is unavoidable, resolve exact allocator charging before allowing this path to publish; do not approximate nodes with a uniform guessed byte multiplier. Avoid a production-wide allocator hook. Preserve lookup complexity and canonical ordering.

**Tests:** Independent per-thread/child-process counting allocator around **module materialization**, not merely DTO decode; use a module with many exports/imports/linked reads/topology children, plus one long metadata and string attribute. Assert observed allocator-requested bytes exactly equal committed debit. Exact `N` succeeds, `N−1` is rejected **before** the unmatched allocation request, and retry with fresh budget succeeds. Include an early malformed-module failure that verifies caller-ledger rollback without double charging.

### S3-05 — Type relocation and nested validation are not transaction-budgeted (**CRITICAL latent**)

**Evidence:** The native importer called by semantic materialization is `TypeStore::import_type_graph(&world.type_graph, limits)`: [`import.rs` lines 1825–1865](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1825-L1865). Its signature has **no allocation transaction** and creates a fresh store, type parameter maps, relocation vectors and uniqueness sets: [`types/artifact.rs` lines 2468–2557](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/types/artifact.rs#L2468-L2557).

`validate_scoped_roots` creates a `BTreeSet<(ScopedTypeId, Vec<usize>)>`, `context.to_vec()` per visited state, and `Vec::with_capacity(context.len()+1)`; it also tracks a **local estimated** `64 + context.len() * size_of::<usize>()` rather than charging the exact native node/vector requests: [`types/artifact.rs` lines 2580–2631](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/types/artifact.rs#L2580-L2631).

**Impact:** The constructor and validator remain incompatible with the one-load exact requested-byte contract. More importantly, byte estimates in a local `remaining_bytes` counter should not be mistaken for shared allocator accounting. This is behind the deliberate importer fail-closed guard today.

**Repair:** Introduce a fallible, transactional TypeStore/import builder and budget-aware owner indexing. Account for arrays, hash/map nodes, boxed payloads, interning, strings and all validator scratch. Reuse sorted immutable storage when compatibility permits. Do not remove `require_semantic_import_allocation_accounting()` until the full native relocation census and exact fit tests pass.

### S3-06 — Query reconstruction has unbudgeted owner and product allocations (**HIGH latent**)

**Evidence:** `resolve_query_key` calls `serde_json::from_str` to decode owner keys, without passing the transaction or allocating through `AllocationBudget`: [`query.rs` lines 119–191](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/query.rs#L119-L191). `product_and_fingerprints` uses `Arc::new`, `BTreeMap::new`, string creation, and cloned semantic owners: [`query.rs` lines 193–359](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/query.rs#L193-L359). `reconstruct_query_image_inner` charges the root tuple and edge vectors but inserts into native output `BTreeMap`s and converts edge vectors into uncharged `Arc<[DependencyEdge]>`: [`query.rs` lines 399–454](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/query.rs#L399-L454).

**Impact:** The stage commits only a fraction of the native allocation requests. For malicious owner JSON or a large supported-root set this breaches the exact-budget invariant after the semantic-import guards are resolved. An `Arc::from(Vec)` commonly allocates a new shared backing; its requested block must be accounted independently.

**Repair:** Replace unbounded `serde_json::from_str` with a budgeted stable-identity decoder (or fixed canonical binary owner format), explicitly charging any strings/container intermediates. Build query-key maps, products and dependency Arc slices through charged constructors, preserving O(log N) lookup or better. Verify *complete* output owner construction with an observed allocator. Do not sacrifice exact source and product fingerprint parity.

### S3-07 — OOM/budget errors can themselves allocate (**HIGH**)

**Evidence:** The wire module decoder maps each `AllocationError` to `invalid(&'static str)`, which returns `ArtifactError::LimitExceeded(message.into())`: [`module_world.rs` lines 29–39](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/module_world.rs#L29-L39). The semantic-world validator has the same pattern: [`world.rs` lines 1524–1536](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L1524-L1536). The materializer also converts allocation errors to `LimitExceeded(String)` via `.into()`: [`compiler_artifact.rs` lines 569–578](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-modules/src/compiler_artifact.rs#L569-L578). `ArtifactError` already has the allocation-free typed variant `Allocation(AllocationError)`: [`error.rs` lines 7–35](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/error.rs#L7-L35).

**Impact:** Error construction can make a new uncharged heap request when the requested-byte limit has just been exhausted or when the allocator is failing. It also erases the typed allocation cause and makes callers classify allocation failures via strings. This undermines deterministic fail-closed resource enforcement.

**Repair:** Preserve `ArtifactError::Allocation(error)` end to end. Format detailed, contextual human diagnostics only after leaving the restricted allocation transaction. Tests should force allocation failure and verify zero additional allocator requests until unwinding or error return.

### S3-08 — Query DTO allocates to generic semantic limits before narrower query limits (**HIGH**)

**Evidence:** [`query.rs` lines 120–186](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/query.rs#L120-L186) checks `root_count`, per-root `dep_count` and `ev_count` against `max_semantic_entries_per_family` (default **1,000,000**) and reserves their vectors. [`limits.rs` lines 25–46](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/limits.rs#L25-L46) separately defines `max_query_roots` (**100,000**) and `max_query_dependencies_per_root` (**10,000**). These narrower limits appear later in `validate_query_image_closure_with_allocation_budget`: [`semantic/artifact/query.rs` lines 25–57](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/query.rs#L25-L57).

**Impact:** Input that is already invalid under query-profile limits can force 10× the permitted root-count vector reservation (subject to the total budget) before being rejected, and per-root dependency reservations are similarly overbroad. Early, precise rejection should avoid avoidable allocation work.

**Repair:** Apply `min(max_query_roots, max_semantic_entries_per_family)` and `max_query_dependencies_per_root` in the DTO reader *before* reservation; bound evidence cardinality and total edges with checked accumulators. Keep closure validation as a separate semantic layer.

### Metadata: important positive controls and remaining limitation

[`metadata.rs` lines 165–203](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/metadata.rs#L165-L203) checks content preflight and header/profile/identity agreement before and after the current-schema typed JSON decoder. [`encode.rs` lines 38–81](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-type-meta/src/encode.rs#L38-L81) performs transactional current-schema decode. These are significant safeguards and should remain. However the owning decoder's own comment explicitly acknowledges that persistent native-owner and validator budget coverage is **partial**. Passing token/container limit tests is not proof of an exact full-load native allocation bound.

---

## 4. Query closure and deterministic algorithm findings

### S3-09 — Exported aggregate dependency evidence is ignored on import (**HIGH latent**)

**Evidence:** `export_query_image` creates `dependency_evidence` entries from `dependency_fingerprints`, retaining each stable query owner and an aggregate-owner string: [`export.rs` lines 2297–2331](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L2297-L2331). The DTO reader decodes these entries: [`compiler-artifact/query.rs` lines 145–182](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/query.rs#L145-L182). Closure validator only rejects source-dependent evidence kind; it does not establish owner reference closure or equality to the later imported map: [`artifact/query.rs` lines 25–112](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/query.rs#L25-L112). Most importantly, `reconstruct_query_image_inner` iterates supported roots and reconstructs `fingerprints` from those roots; it never consumes `image.dependency_evidence`: [`artifact/query.rs` lines 399–454](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/query.rs#L399-L454).

**Impact:** The artifact can carry aggregate dependency declarations with no ownership or semantic consequence after import. Retaining but ignoring them violates the advertised separation of reusable root products and broader aggregate dependency evidence and risks incomplete invalidation/freshness tracking.

**Repair:** Define exactly which evidence rows become the dependency-fingerprint map and which are validated against existing owner aggregate evidence. Decode each stable key exactly, reject duplicates/foreign owners, and recompute/compare fingerprints from reconstructed owner products rather than adopting arbitrary wire values. If a family is intentionally omitted, do not emit a misleading unused row; explicitly document its capability disposition.

**Tests:** Evidence-only aggregate absent from supported roots, duplicate evidence key, malformed owner key, wrong aggregate-owner key, changed owner fingerprint, and non-reusable-but-aggregate-supported evidence. Assert either exact projection or a typed rejection; no silent dropping.

### S3-10 — Query root reverse-pruning admits O(N²) work (**MEDIUM–HIGH**)

**Evidence:** [`export.rs` lines 2240–2285](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L2240-L2285). The algorithm builds `retained_keys`, then repeatedly scans all retained keys, collects keys whose dependencies were removed or absent, removes that round's failures, and repeats until no changes. On a length-N chain ending in a noncandidate/source-dependent leaf, it removes one additional upstream layer per iteration; work is Θ(N²), even though the dependency graph itself has Θ(N) edges.

**Impact:** Large compiler query graphs can generate disproportionate artifact-export CPU and temporary allocations, undermining the intended compilation-speed gain. This is independent of the semantic import guards; the query exporter is currently callable and the algorithmic behavior follows directly from its fixed-point implementation.

**Repair:** Compute candidate membership once, build reverse adjacency of candidate dependencies, initialize a queue of roots with missing dependencies, and prune each root at most once while decrementing predecessor support counters (O(V+E)). Preserve canonical emission ordering, exact dependency-index mapping, and all-or-nothing support semantics. Tests: long 10k–100k chain with one missing leaf, fan-in/fan-out graph, distinct cycles (whose retention is later rejected by closure), and large all-supported DAG. Benchmark iterations, allocation bytes and wall time.

### Determinism and source provenance

**Positive:** Shared type export uses `BTreeMap<CanonicalTypeRootKey, TypeGraphRoot>` and graph closing from sorted roots: [`types/artifact.rs` lines 76–203](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/types/artifact.rs#L76-L203). The section builder sorts by `SectionKind`. These are good deterministic foundations.

**Risk not yet independently proved:** The same code registers some roots in local order derived from native `TypeParameterId` sorting (see Slice 2), which can differ across independent allocation histories. A same-process `build() == build()` test does not prove cross-process identity if native allocations/interning order change. Require a two-process build with randomized *construction* ordering but identical semantics and compare each section digest and the final artifact bytes. Do not mark a nondeterminism defect as reproduced until that test shows a difference.

---

## 5. Build/write path and memory profile

### S3-11 — Four section-buffer clones inflate peak artifact build memory (**MEDIUM–HIGH**)

**Evidence:** [`canonical_artifact.rs` lines 128–244](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L128-L244). The writer constructs independent `module_world_bytes`, `semantic_world_bytes`, `query_image_bytes`, and `metadata_bytes`. It passes **`.clone()`** of each entire vector into `builder.add_section`, then `TeloscContainerBuilder::build()` allocates a full output buffer and copies each section into it: [`container.rs` lines 39–118](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/container.rs#L39-L118). Original buffers remain alive until summary construction.

**Impact:** The four encoded payloads can be resident approximately three times at peak (original vectors, cloned builder vectors, final combined artifact buffer), *before* accounting for wire DTO graphs and other validation buffers. Large Universe compilation may therefore spend substantial time and peak memory simply copying serialized output. This is a capacity/performance defect, not necessarily a semantic correctness failure.

**Low-risk correction:** Record section lengths first, then *move* original byte vectors into the builder rather than clone. Compute digests as needed before or during the move; summary stores lengths and final digest. Stream serialized sections into the final output in one pass or leave the builder's final single copy as a next-step optimization. Keep byte-for-byte deterministic output and independent post-build validation.

**Measurement:** Instrument peak RSS and allocator-requested bytes around source compilation, semantic export, four section encodes, builder assembly, self-validation and disk publication. Compare before/after by section size. No unmeasured speedup claim should be put into the P7 acceptance record.

### S3-13 — Atomic file replacement is not the same as power-loss durability (**MEDIUM**)

**Evidence:** [`canonical_artifact.rs` lines 249–263](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L249-L263) validates the actual bytes first, creates a `NamedTempFile` in the destination directory, writes, `sync_all()`s the temp file and `persist()`s it. The code does **not** fsync the parent directory after the rename. This is a meaningful positive upgrade from unsafe direct overwrite.

**Impact:** The replacement is designed to be atomic from a concurrent-reader perspective, but the code does not establish full crash/power-loss durability of the renamed directory entry. The distinction matters if the product promises durable artifact publication rather than safe best-effort cache replacement.

**Resolution:** For P7, document the semantics precisely. If crash durability is required, add platform-specific parent-directory synchronization where supported; otherwise make artifacts disposable/rebuildable and recover safely from a missing/corrupt post-crash cache. Tests should verify previous bytes survive pre-persist failure, concurrent loads observe a complete old or new image, and write failure leaves no orphan temp file. Do not claim power-loss durability from an ordinary unit test.

### S3-15 — Source digest is authenticated provenance, not a freshness oracle (**P7/P9 boundary**)

**Evidence:** [`canonical_artifact.rs` lines 485–546](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/canonical_artifact.rs#L485-L546): loader checks only that `source_digest` is nonzero and binds it into the authenticated header; source text is not retained, so it cannot recompute provenance from source. `canonical_source_digest` does hash source-authority map on source-built images; artifact-loaded images carry the previously recorded digest forward.

**Interpretation:** This is correct *separation of duties* for a source-free reader, not a P7 bug by itself. An external source-version/freshness authority must decide whether the persisted artifact is valid for the current Universe source snapshot. P9's full cache manager can own that policy. P7 must nevertheless prove that a valid artifact loads and compiles user code, that incompatible ABI/native metadata is rejected, and that a changed source cannot silently reuse an image when the caller *claims* it has validated freshness.

---

## 6. Tests, observability, and release evidence

### S3-14 — Existing exact-budget tests do not establish an exact full-load budget (**HIGH**)

The branch contains useful focused probes:

- [`container_allocation_test.rs` lines 1–95](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/tests/container_allocation_test.rs#L1-L95) checks the charge for package identity and directory allocation using a counting allocator.
- [`query_allocation_test.rs` lines 1–115](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/tests/query_allocation_test.rs#L1-L115) checks selected query DTO vector reservations and rollback.
- [`allocation_budgeted_map.rs` lines 1–218](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-type-meta/tests/allocation_budgeted_map.rs#L1-L218) checks explicit sorted collection helpers, helper request sizes and injected allocation failures.
- [`canonical_artifact.rs` lines 1–353](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/tests/core/modules/canonical_artifact.rs#L1-L353) specifies real build/load, limits, source-free inline compilation, atomic replacement, compatibility and corruption checks, but these cannot presently certify a real valid image: the exporter/importer still contain unconditional fail-closed stops (Slices 1–2).

**Precise test design problem:** A test asserting only `load(...).is_err()` under low budget is compatible with rejection at the *wrong* stage for *the wrong reason*. Every budget test must assert its typed error and the independent observed allocator-requested amount; the success case must reach the intended native stage. Similarly, a static parser test passing on arbitrary fake section payloads proves container structure, not semantic owner parity.

### 6.1 Required slice-3 test matrix

| Gate | Input | Required assertion |
|---|---|---|
| G3.01 complete-byte identity | Valid file plus 1 trailing byte | **Reject**; no equal-digest acceptance |
| G3.02 gap | Rehashed directory with one interior gap | **Reject** noncanonical layout |
| G3.03 uncompressed lengths | Decoded length `0`, `len+1`, `u64::MAX` | **Reject before materialization** |
| G3.04 capability extension | Unknown header bit with rehashed digest | **Reject** unknown contract |
| G3.05 directory normalization | Permuted sections; wrong mandatory flags | **Reject or explicitly classify compatibility**, never claim canonical equivalence |
| G3.06 module exact allocator | Large real module-world materialization under counting allocator | Complete observed-request equality; exact N/N−1; no allocations after N−1 rejection |
| G3.07 type exact allocator | Nested generics, lambda binders, symbolic rows, current parameter map | Same equality and rollback, independent allocation histories |
| G3.08 query exact allocator | Mixed query owners and dependency edges, typed evidence | Charge owner parsing, map nodes, Arc products and edge slabs; N/N−1 |
| G3.09 malformed owner JSON | Validly rehashed query section with malformed or large stable JSON key | Error before uncontrolled native allocation; caller ledger unchanged |
| G3.10 error budget | Force exhaustion/failure in module DTO and native module builder | `ArtifactError::Allocation` retained, no heap formatting before return |
| G3.11 pruning complexity | 10k/100k candidate chain ending at one omitted leaf | Work/time scales linearly, not quadratically |
| G3.12 evidence closure | Evidence-only, duplicate, missing/wrong owner, changed fingerprint | Reconstruct exactly or fail closed; never ignore |
| G3.13 binary determinism | Two fresh processes with differing native insertion histories | Identical section digests and whole bytes for equal semantic owners |
| G3.14 build memory | Real Universe with and without section clones | Lower measured peak and no byte changes |
| G3.15 full cold load | Fresh process, embedded source access denied | Successful source-free load and novel user/package compile; exact ledger equality |
| G3.16 failure retry | Bad/checksum/ABI/owner budget artifact then corrected artifact | No partial publication or poisoned singleton; corrected load succeeds |

### 6.2 Ordering and measurement recommendations

1. **Perform read-only parser hardening first** (S3-01/02/03/12); each can be proven on tiny synthetic containers without waiting for semantic transport.
2. **Repair exact typed allocation errors** (S3-07) and run counting-allocator tests for currently reachable module materialization (S3-04).
3. Complete semantic owner reconstruction *together with* TypeStore, static owners, validator and native-product allocation charging (S3-05, Slice 2). Preserve all unconditional guards until genuinely complete.
4. Rebuild query reconstruction, evidence handling, and budgeted output products as an integrated stage (S3-06/08/09).
5. Replace quadratic export pruning and remove redundant section clones (S3-10/11). Benchmark source-built and artifact-loaded cases.
6. Only after correctness gates are executable, test full cold-process load, source-denied standalone packages (e.g. `parse`, `similarity`, `slotmap`) and workspace-wide timings.

### 6.3 Suggested commands for implementer (not executed by auditor)

```bash
# Run in the pinned perf branch / isolated worktree after confirming HEAD and clean tree.
git rev-parse HEAD
git status --short
cargo test -p phalcom-compiler-artifact --test container_test
cargo test -p phalcom-compiler-artifact --test container_allocation_test
cargo test -p phalcom-compiler-artifact --test query_allocation_test
cargo test -p phalcom-type-meta --test allocation_budgeted_map
cargo test -p phalcom-semantic --test semantic artifact
cargo test -p phalcom-core --test core canonical_artifact

# Exact target names/filters should be checked against Cargo test discovery;
# do not treat a filter matching zero tests as a pass.
```

For peak memory, run tests as isolated child processes and record both allocator-requested bytes and peak resident bytes independently. **Allocator-requested byte budget is not an RSS ceiling.** Avoid running a global counting allocator alongside unrelated parallel test threads when interpreting exact equality.

---

## 7. Remaining requirements and implementation handoff

### Mandatory invariants before lifting the P7 load guards

1. **Byte identity:** Canonical output is a uniquely encoded, fully accounted byte sequence. No accepted trailing/gap bytes; uncompressed size fields and known capability bits are checked.
2. **Cumulative resource safety:** File acquisition (if owned), header/directory, DTOs, module/type/static/query/materialized owners, metadata, validation scratch and final publication all debit **one** ledger before each allocator request. No local cap masquerades as whole-load accounting.
3. **Typed failure:** Reject before uncharged allocation when a budget is insufficient. Preserve typed errors, no string formatting on exhausted budget. Leave the publicly visible caller ledger unchanged on failure.
4. **Canonical owner fidelity:** No fabricated semantic rows, zero fingerprints, lossy witness substitutions or missing checked-body facts; see Slice 2. A successful low-level codec must imply neither semantic correctness nor complete code-generation readiness.
5. **Reconstruction closure:** Every query dependency/evidence owner resolves against the reconstructed semantic graph; no unsupported keys or ignored required rows.
6. **Determinism:** Equal semantic owners produce identical artifact bytes across fresh processes and distinct allocation histories. A same-process repeat-only test is insufficient.
7. **Functional acceptance:** Valid `universe.telosc` is produced, atomically installed, read in a fresh source-denied process, and used to compile and run representative stand-alone packages and new code. No claim of speedup until measured against source-built baselines.

### Priority table

| Priority | Action | Dependency | Why first / ownership |
|---|---|---|---|
| **P0** | Harden strict file canonicality, length and capability decoding | None | Isolated, easy-to-prove parser correctness; independent of owner blockers |
| **P0** | Charge native module-world owners; typed allocation errors | Exact memory policy (already recorded) | Reachable now before failing semantic guard |
| **P0** | Implement real semantic import with budgeted TypeStore/static/validator paths | Slice 2 owner schema | Removes blockers without violating load contract |
| **P0** | Budget and validate query owner parsing/evidence/output maps | Complete semantic authority | Exact loading and query closure |
| **P1** | Rework reverse-pruning fixed point; remove four writer clones | Functional exporter path | Predictable runtime and memory gain |
| **P1** | Full cold-process, build-load-compile, exact-budget and parity harness | All P0 | Credible P7 closure evidence |
| **P2 / P9** | Artifact freshness/cache policy and incremental cache integration | Functional P7 | Not an excuse to defer P7 source-free usability |

### Distinguish architectural defects from intentionally incomplete work

The current allocation helpers are a coherent design, not a reason to restart P7: `AllocationBudget::vector`, `grow_vector_exact`, `boxed_str`, `arc`, and sorted-buffer builders charge allocator layouts pre-request, preserve fallible construction, and have focused tests. Container decode performs correct offset overflow and overlap checks. Query closure uses an iterative budgeted DFS. Current metadata preflight checks schema, identity, token/container/depth counts and bundle agreement. The report proposes **finishing the adoption of those primitives**, not discarding them.

Conversely, the presence of per-operation exact allocation tests does **not** certify module-world materialization, type graph relocation or full-world loading. Their accounting gaps are source-visible. Keep source-denied package acceptance as the ultimate release gate.

---

## 8. Recommended Slice 4

**Slice 4 — Loaded compiler behavioral parity, package compilation, entry ownership, native/runtime linkage, and cold/warm performance verification.** Examine source-built versus artifact-loaded `ProgramAnalyzer`/`ProgramCompiler` behavior for `Inline`, `Module`, `Package`, and `Project`; the actual `packages/parse`, `packages/similarity` and `packages/slotmap` layouts; native binding and runtime codegen; source denial; universe initialization counts; workspace test selection; correctness/performance baselines. Preserve the P7/P8/P9 scope distinction: P7 needs a usable compiler-only Universe image, not a P8 executable payload or the full P9 cache manager.

---

## 9. Audit methodology, limits, and conclusion

- Branch head was checked against the GitHub branch API at the start of Slice 3; source links target the **immutable commit**, not the moving branch.
- Read-only source inspection covered `phalcom-compiler-artifact::{container,header,section,codec,module_world,query,metadata}`, `phalcom-type-meta::{allocation,budgeted,encode}`, `phalcom-modules::compiler_artifact`, `phalcom-semantic::{types/artifact,artifact/query,artifact/export,artifact/import,canonical_image}`, and `phalcom-core::modules::canonical_artifact`, plus relevant tests and locally available P7 handoff/remaining plan.
- No executable Rust verification was possible in this audit environment: the repository is not locally cloned and the container has no direct GitHub network resolution. Observations are **source-proven** where indicated. Tests and quantitative performance outcomes are recommendations, not fabricated passes/failures.
- Container integrity gaps concern canonicality and byte identity. The full section payloads themselves are content-hashed; do not extrapolate to a demonstrated collision or silent mutation of referenced semantic data.
- Allocation-gap severity follows the existing **exact allocator-requested byte** policy, not a stronger claim that the product currently offers an RSS cap.

**Final Slice 3 disposition: `PARTIAL / FAIL-CLOSED / NOT ACCEPTABLE FOR P7 CLOSURE`.** The immediate corrective focus is (i) strict binary identity, (ii) truthful exact cumulative accounting—especially the currently reached module materializer—(iii) completion of typed semantic and query owner reconstruction under the same ledger, and (iv) independent full source-free compilation and allocation evidence. Neither test-count summaries nor a valid-looking header should be accepted as substitute proof.
