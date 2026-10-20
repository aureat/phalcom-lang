---
id: PERF003.C2.P7.PATCH5-R
kind: implementation-plan
title: "Corrective PATCH5-R — Compiler-Only Universe Artifact Closure"
program: PERF003
checkpoint: PERF003.C2
parent_phase: PERF003.C2.P7
supersedes_for_remaining_work: PERF003.C2.P7.PATCH5
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
branch: perf/universe-characterization
prepared_against_remote_revision: 9cbba391a4482052dec476b327619c957a028b19
prior_implementation_commit: c09ee934ff05db2a9f36f977cc6c5e0abfbac9ee
prepared_date: 2026-10-17
execution_mode: patch-grade
implementation_agent: Luna
scope: compiler-only-universe-artifact
prohibitions:
  - Do not implement P8 executable Universe or VM bootstrap.
  - Do not relax the exact allocator-requested byte contract.
  - Do not synthesize absent semantic identities, proofs, query products, or fingerprints.
  - Do not use source-built Universe fallback in artifact-backed compilation.
  - Do not weaken, skip, or relabel failing tests to declare closure.
  - Do not claim successful package parity when both paths produce errors.
  - Do not claim physical source denial, RSS leak freedom, or unmeasured speedups.
---

# PERF003.C2.P7.PATCH5-R — Luna Patch-Grade Corrective Implementation Plan

> **Mission:** Finish all remaining work from original `PERF003.C2.P7.PATCH5`. The result must be a *fully imported, immutable, source-free compiler-only Universe artifact* that a fresh compiler can use to successfully compile an independent package. Exact cumulative allocation accounting, canonical semantic/query correctness, fork semantics, failure atomicity, regression coverage, and reproduced evidence are mandatory. **Tasks are done only when their acceptance gates pass.**

## 0. Decision, target outcome, and proof policy

### 0.1 The goal is an operational compiler artifact, not a collection of implemented types

Successful closure means the following complete path works **without rebuilding the Universe from source in the artifact-backed compiler**:

```text
Compile packages/universe from source
  -> build a canonical compiler image
  -> deterministically write universe.telosc
  -> start a fresh compiler process
  -> validate artifact ABI/digests/limits/capabilities
  -> import MODULE_WORLD + SEMANTIC_WORLD + QUERY_IMAGE + METADATA
  -> construct source-free CanonicalUniverseFormalImage
  -> extend frozen canonical stores with local compilation state
  -> compile packages/slotmap successfully
  -> verify source-built vs artifact-backed semantic equivalence
  -> independently prove exact whole-load requested-byte accounting
  -> demonstrate fail-closed invalid-artifact and budget behavior
  -> publish reproducible evidence
```

**Binary acceptance:** all mandatory `R.G0`–`R.G7` gates must pass. A buildable API, a successful import of a small fixture, a test that only compares identical errors, and a ledger self-report are **not** substitutes.

### 0.2 Current auditable repository baseline

| Property | State as of 2026-10-17 | Evidence confidence |
|---|---|---|
| Repository | `aureat/phalcom-lang` | GitHub source read |
| Branch | `perf/universe-characterization` | GitHub branch read |
| Remote HEAD | `9cbba391a4482052dec476b327619c957a028b19` | Verified |
| Previous implementation | `c09ee934ff05db2a9f36f977cc6c5e0abfbac9ee` | Verified commit |
| Remote commits after previous audit | 0 | Verified compare: identical |
| Original PATCH5 plan/handoff | Present | Verified |
| T0/T1 allocator infrastructure | Present | Source-observed; complete runtime correctness unverified |
| T2/T3 module frozen authority | Present | Source-observed; whole-load proof unverified |
| T4/T5 frozen type and static-value prefixes | Present | Source-observed; active full importer not wired |
| Original T6–T12 | Incomplete or not accepted | Specific defects detailed below |
| Local macOS worktree | Not directly observed | **Must verify** |
| Fresh Cargo results on current HEAD | Not executed in this planning audit | **Must run** |

The implementer must re-anchor if `HEAD` or the remote moves. Prior reports describe an older revision and are **historical characterization**, not closure proof of PATCH5-R.

Primary references:

- `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/PERF003.C2.P7.PATCH5-Luna-Patch-Grade-Implementation-Plan.md`
- `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/PERF003.C2.P7.PATCH5-handoff.md`
- `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-allocation-materialization-architecture-ruling-2026-10-15.md`
- `docs/implementation/PERF003-runtime-performance/C2-compilation-performance-and-efficiency/P7/PERF003.C2.P7-allocation-map-accounting-incident-2026-10-15.md`
- `docs/perf003-p7-audit/{REPORT,FINDINGS,FAILURES,MEMORY,COMMANDS,ENVIRONMENT}.md`
- `phalcom-core/src/modules/canonical_artifact.rs`

### 0.3 Mandatory requirements and their gates

| ID | Non-negotiable outcome | Proof gate |
|---|---|---|
| M1 | Complete real Universe artifact loads into a fresh compiler | G3, G5 |
| M2 | All required semantic owners, types, statics, query products and metadata imported | G1–G3 |
| M3 | Every requested allocator layout in the defined successful load is charged **before** allocation | G1, G2, G4 |
| M4 | Exact cumulative budget `N` succeeds | G4 |
| M5 | `N-1` yields the **correct typed** allocation error | G4 |
| M6 | Loaded canonical prefixes are immutable and extend through local overlays | G1, G3, G5 |
| M7 | Query keys, products, fingerprints and dependencies preserve authoritative semantics | G2, G5 |
| M8 | Artifact-backed compilation does not rebuild the Universe from source | G3, G5 |
| M9 | A nontrivial independent package **successfully** compiles through both paths | G5 |
| M10 | Invalid and resource-exhausted loads reject without partial publication | G1–G4, G7 |
| M11 | Relevant workspace functionality and tests do not regress | G7 |
| M12 | Benchmarks, evidence, and lifecycle assertions accurately reflect executed outcomes | G6–G7 |

### 0.4 Status vocabulary

Use **`PASS`**, **`FAIL`**, **`BLOCKED`**, **`UNVERIFIED`**, and **`OUT_OF_SCOPE`** consistently. `PASS` requires an executed command/test or explicit recorded proof. Do not treat `EXPECTED_ERROR`, `COMPILE_ERROR`, or `test --no-run` as successful package compilation. Record first causal owner for any blocker.

Distinguish two decisions:

- **Allocation-hardening closure:** exact accounting, import correctness, and publication gates complete.
- **PATCH5-R / P7 functional closure:** allocation-hardening closure **plus** real successful source/artifact package compilation, regression verification and evidence closure.

The original PATCH5 permitted allocation-only closure with a documented baseline SlotMap blocker. That is **not sufficient** for the stated P7 functional end state. It may be accurately reported as partial closure, never full PATCH5-R `COMPLETE`.

---

## 1. Fixed architecture: representation, ownership, and load transaction

### 1.1 Compiler-only artifact layout

```text
Built canonical Universe compiler image (source-backed)
    |
    +-- MODULE_WORLD       -> portable modules/interfaces/topology/linked authority
    +-- SEMANTIC_WORLD     -> portable types/statics/structural and callable authority
    +-- QUERY_IMAGE        -> portable eligible reusable roots and dependency evidence
    +-- METADATA           -> validated public runtime/type metadata
    |
    v
universe.telosc: deterministic, validated, ABI-bound compiler-only container
    |
    v
Fresh load with ONE AllocationBudget
    |
    +-- frozen CanonicalExternalModuleWorld and nested ordered authority
    +-- frozen canonical TypeStore/TypeLambda/StaticValueStore prefixes
    +-- imported semantic owner tables and SemanticSnapshot
    +-- frozen reusable CanonicalSemanticQueryImage + dependency fingerprints
    +-- validated metadata bundle
    |
    v
budgeted CanonicalUniverseFormalImage publication
    |
    v
LoadedUniverseArtifact { formal_image, metadata, ... }
    |
    v
ProgramCompiler::compile_entry_selection_with_image(...)
    |
    v
local semantic/query session extends canonical immutable authority
```

Retain the current `TeloscCapabilities::compiler_only_v1()` shape; `portable_executable` must remain `false`. No runtime executable section is introduced.

### 1.2 One cumulative budget, explicit child transactions

```text
ONE successful load
  -> ONE whole-load AllocationBudget
  -> stage-local children with independent section/stage limits
  -> precharge exact allocator-requested Layout before requests
  -> account for replacement allocations during growth
  -> account for temporary scratch and final retained owners
  -> commit child only on successful stage completion
  -> never refund successful scratch
  -> publish only after final cross-component validation
```

**Count allocator-requested bytes**, not RSS, usable allocator capacity, logical entry size, wire bytes, or guessed B-tree node sizes. Undercharge and overcharge both violate the contract.

For successful loads with identical measurement scopes:

```text
N = production whole-load ledger requested bytes
observed = independent test-only GlobalAlloc requested bytes
required: observed == N
required: load(max_total_allocation_bytes = N) succeeds
required: load(max_total_allocation_bytes = N-1) fails with typed exhaustion
```

For **failed** child transactions, external budget commit is rolled back by design. Do **not** incorrectly expect the rolled-back parent ledger to equal the total physical allocator requests made during the failed attempt. Instead prove precharge discipline at each request, typed rejection, no parent commit, and no publication; separately observe/verify failed-stage requests if practical.

Define accounting behavior for zero-sized allocations, `alloc`, `alloc_zeroed`, `realloc(new_size)`, fallible reserves, intermediate buffers, and error construction. Keep isolated counting-allocator hooks in tests only.

### 1.3 Source/Frozen physical duality; same semantic read authority

Source construction may retain `BTreeMap`, `BTreeSet`, `im::Vector` and `im::HashMap`. Loaded immutable authority must use explicitly fallible, budgetable dense/sorted storage behind a **common read contract**. Existing `ModuleAuthorityMap` / `ModuleAuthoritySet` and `BudgetedSortedMap` / `BudgetedSortedSet` are the pattern.

Preferred design for query entries (illustrative; name and placement flexible):

```rust
pub(crate) enum CanonicalQueryEntries {
    Source(BTreeMap<QueryKey, CanonicalQueryEntry>),
    Frozen(Arc<BudgetedSortedMap<QueryKey, CanonicalQueryEntry>>),
}

impl CanonicalQueryEntries {
    // get, iter, len, is_empty; no mutation of Frozen authority.
}
```

Do **not** use a generic repository-wide collection migration as a PATCH5-R shortcut. Do **not** construct a native B-tree and immediately convert it into a frozen map: the B-tree node allocations remain opaque. Sort an already budgeted contiguous entry buffer in-place and reject duplicate keys adjacent to one another.

### 1.4 Frozen canonical prefix + mutable local overlay

The loaded TypeStore, TypeLambdaArena, and StaticValueStore must support:

- A valid canonical prefix with stable semantic meaning.
- Forks that reuse this prefix without deep reinterning.
- Local inserts in independent session overlays.
- Canonical lookup and interning across both frozen and local storage.
- No ability for local mutation to overwrite, delete, or shadow canonical semantic facts.
- Correct lineage/relocation checks in `extends_canonical_prefix` and compatible query readers.

Raw process-local store IDs need not match across independent processes. Semantic identity, canonical shape, relocation meaning, and fingerprints must match.

### 1.5 Query authority is proof-carrying, not reconstructible by guesswork

Respect `canonical_query_persistence`: structural reusable roots may be loaded; open-world and unsupported families may not be promoted to reusable authority. Preserve exact fingerprints, dependency observations, owner provenance, typed products and query-world/prefix checks. Never synthesize a missing query product or zero fingerprint merely to keep the importer running.

### 1.6 Loaded module product views should not deep-copy canonical modules

`ModuleQueryProducts::from_external_modules` currently creates native maps and clones entire module interfaces for a loaded image. Prefer a read-only canonical-view-backed representation or minimal budgeted indexes referencing existing authority. Preserve source-built behavior through a variant/read adapter if necessary. Do not duplicate full canonical module graphs solely to satisfy historical container types.

### 1.7 Scope exclusions

No `EXECUTABLE` section, VM bytecode/prototype serialization, `VM::try_new_with_universe_artifact`, `ProductKind` runtime bootstrap repair, runtime source denial, automatic CLI/cache discovery, incremental Universe artifact rebuilding, or P8 executable bootstrap. P7 must be usable through existing library APIs; adding CLI flags is not a PATCH5-R prerequisite.

---

## 2. Confirmed live source gaps / corrective take-over map

| Gap | Live source site | Required correction |
|---|---|---|
| Ineffective accounting guard returns `Ok(())` | `phalcom-semantic/src/artifact/import.rs::require_semantic_import_allocation_accounting` | Remove after actual closure; don't pretend it enforces an invariant |
| Non-budget type import in active semantic path | `validate_and_import_semantic_world_inner` -> `TypeStore::import_type_graph` | Switch to implemented budget-aware frozen prefix import |
| Non-budget static import in active semantic path | `validate_and_import_semantic_world_inner` -> `import_static_value_graph` | Switch to implemented budget-aware static prefix import with budgeted closure adapters |
| Structural contracts and many semantic owner families | `phalcom-semantic/src/artifact/import.rs` | Audit/transitively budget each retained owner and scratch |
| JSON owner decoding, `format!`, `DeserializeOwned` | `phalcom-semantic/src/artifact/query.rs::decode_owner` | Borrowed/bounded decoding + budgeted identity materialization, same wire ABI |
| No budget passed into `product_and_fingerprints` | `phalcom-semantic/src/artifact/query.rs` | Budget nested products, clones, Arc wrappers, native maps and owned strings |
| Native query entry/fingerprint maps | `reconstruct_query_image_inner` | Frozen budgeted ordered storage/read adapters |
| Native query census and extra BTreeSet DAG validation | `CanonicalSemanticQueryImage::from_loaded_artifact_entries` | Budget/source-frozen backing, reuse budgeted graph validation, no repeated opaque scratch |
| Deep module product duplication | `phalcom-semantic/src/snapshot.rs::ModuleQueryProducts::from_external_modules` | Reference canonical module authority or build minimal frozen indexes |
| Unbudgeted formal image construction | `CanonicalUniverseFormalImage::from_loaded_artifact` | Add budgeted artifact constructor and atomic publication |
| Core loader calls old publication | `phalcom-core/src/modules/canonical_artifact.rs` | Pass whole-load transaction through finalization |
| String-producing errors in charged path | Core/import/query `e.to_string()`, `format!`, `into()` | Preserve typed nonallocating exhaustion; budget other error ownership if allocates |
| Conditional independent equality | `phalcom-core/tests/artifact_allocation.rs` | Unconditional equality assertion |
| `N-1` accepts any error | Same | Require typed allocation exhaustion |
| SlotMap `for` syntax and physical test filenames | `packages/slotmap/...` | Minimal source-valid fixture repairs |
| Harness checks identical errors instead of successful packages | `phalcom-core/examples/perf_audit_p7.rs::cmd_slotmap_eval` | Require both `Ok`, then compare stable compilation observables |
| Harness CSV/determinism/RSS/reporting gaps | `phalcom-core/examples/perf_audit_p7.rs` | Fix and regenerate evidence against final revision |

This table is **not** a complete transitive allocation ledger. T1–T3 must classify every reachable owned allocation in the complete successful load path. Existing T0–T5 infrastructure should be preserved, reviewed, and reused rather than replaced wholesale.

---

## 3. Execution protocol

### 3.1 Task ordering

```text
R.T0 Baseline
  -> R.T1 Semantic owner closure [G1]
  -> R.T2 Query closure [G2]
  -> R.T3 Atomic final image publication [G3]
  -> R.T4 Independent full-load byte equality [G4]
  -> R.T5 Successful independent package parity [G5]
  -> R.T6 Correct measurement and evidence [G6]
  -> R.T7 Full regression, closure report and P8 boundary [G7]
```

T5 SlotMap fixture normalization and T6 harness mechanical repairs may run in parallel with T1–T3 **as isolated commits**, but final parity/benchmark results are valid only against the completed integrated artifact loader. One integration owner must control shared branch merges.

### 3.2 Each task's required report

For every task provide: changed files, short causal explanation, exact commands and exit codes, test count/results, evidence paths, remaining known defects, and explicit `PASS/FAIL/BLOCKED/UNVERIFIED` gate decision. An implemented function without its discriminator is `IN_PROGRESS`, never `COMPLETE`.

### 3.3 Review checkpoints

1. **Checkpoint A:** Semantic import, canonical store lineage/fork behavior, independent semantic requested-byte equality.
2. **Checkpoint B:** Query import + formal image + whole-load allocator equality, corruption and publication atomicity.
3. **Checkpoint C:** Source/artifact successful compilation, full regressions, accurate measured evidence, P7 closure decision.

---

## 4. R.T0 — Baseline, buildability, and failed-discriminator capture

**Priority:** P0. **Dependencies:** none. **Gate:** `R.G0`.

### R.T0.1 Inspect Git state without silently rewriting history

```bash
cd /Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization

git fetch origin
git branch --show-current
git rev-parse HEAD
git rev-parse origin/perf/universe-characterization
git status --short
git log -8 --oneline
```

Record exact local/remote revision and status. If branch advanced, analyze relevant diff and update the plan's source line anchors. Do not overwrite uncommitted changes, do not switch branches, and do not force-push.

### R.T0.2 Build/test-target baseline

Run sequentially and capture logs:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --lib \
  -p phalcom-type-meta -p phalcom-modules -p phalcom-semantic -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta --test allocation_helpers
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test topology --test query
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture
STRICT_ALLOCATION_GATE=1 RUSTFLAGS='' RUSTC_WRAPPER='' \
  cargo test -p phalcom-core --test artifact_allocation -- --nocapture --test-threads=1
```

If a target name is not registered under a crate's `Cargo.toml` (for example because `autotests=false`), discover the real target and record the corrected command. Do not quietly skip coverage.

### R.T0.3 Capture current package/compiler and artifact behavior

- Record `target/debug/phalcom packages/slotmap` source-backed result where supported.
- Run or create a programmatic source/artifact compile comparison using `LoadedUniverseArtifact::compile_user_entry`.
- Record whether failures are package syntax, discovery, semantic, source-independent compiler, artifact-specific, or VM bootstrap.
- Execute original harness `allocation-check`, `source-denial-check`, and `slotmap-eval` where buildable; document their proof limitations explicitly.

### Gate R.G0 — PASS conditions

- Baseline revision and worktree state known.
- Relevant build/test commands attempted and actual results recorded.
- First strict whole-load accounting mismatch or success captured independently.
- Every pre-existing compiler/fixture/runtime blocker classified by owner.
- No test assertions weakened as a baseline convenience.

**Artifacts:** `PATCH5-R-baseline.md`, command log, and first-failure ledger. Expected failing tests are acceptable *as baseline evidence*; inaccurate or absent evidence is not.

---

## 5. R.T1 — Semantic-world owner closure (original T6)

**Priority:** P0, critical path. **Dependencies:** R.T0. **Gate:** `R.G1`.

**Primary files:**

```text
phalcom-semantic/src/artifact/import.rs
phalcom-semantic/src/types/artifact.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/type_lambda.rs
phalcom-semantic/src/static_value/artifact.rs
phalcom-semantic/src/static_value/store.rs
phalcom-semantic/src/snapshot.rs    # only if reached by import/materialization
```

Touch owner modules (`parent_map`, `signature`, `traits`, `dispatch`, `impls`, `associated`, `static_definition`, `static_constant`, `module_bindings`, `derived_conformance`) only when reachable from artifact import. Do not refactor unrelated source compilation structures.

### R.T1.1 Wire existing budget-aware frozen imports into the active path

Current `validate_and_import_semantic_world_inner` calls `TypeStore::import_type_graph` and `import_static_value_graph` without the parent stage ledger. Replace with the existing budget-aware import APIs, adapting live signatures:

```rust
let (mut type_store, type_relocation) =
    TypeStore::import_type_graph_with_allocation_budget(
        &world.type_graph,
        limits,
        allocations,
    )?;

let (static_values, static_relocation) =
    import_static_value_graph_with_allocation_budget(
        &world.static_values,
        &type_store,
        &type_relocation,
        &structural_contracts.data_table,
        &structural_contracts.enum_table,
        /* budget-aware declaration/variant resolvers */,
        limits,
        allocations,
    )?;
```

Do not copy this illustration blindly: check the live callback/closure signature and construct budget-aware stable declaration/variant identities. The semantic import transaction must remain the one passed by core, not a new unlimited local budget.

### R.T1.2 Audit the complete imported semantic owner graph

Trace each call from `validate_and_import_semantic_world_inner` through:

- primary structural contracts and aliases, hierarchy, data/enum declarations;
- callable definitions, signatures, control/proof/explanation/diagnostic authority;
- trait headers, trait surfaces, dispatch buckets, witnesses and conformance indexes;
- field signatures, conditional inherent members/domains, derived conformance;
- constructor initialization, associated families and type members;
- static case catalogs/signatures, static definitions/constants;
- module static bindings, parent/owner indexes;
- `SemanticSnapshot` and any retained map/index/`Arc` owner.

For each `BTreeMap`, `BTreeSet`, `im::Vector`, `im::HashMap`, `Arc::new/from`, `Vec` growth, `Box`, `String`, `format!`, `clone`, `collect`, `to_owned`, and owned stable-identity conversion, classify:

```text
A source-only: not invoked on charged load path
B allocation-free: prove no requested allocation occurs
C charged scratch: explicitly budgeted, including growth and nested ownership
D retained frozen owner: explicitly budgeted contiguous/sorted representation
E shared charged owner: reuse or Arc::clone with no new heap allocation
F unresolved: gate remains FAIL; escalate before moving on
```

A field's container reservation does not pay for strings or `Arc`s nested in its elements. Sorting must be in-place and duplicate detection must preserve semantic rejection, privacy, ordering and fingerprint behavior.

### R.T1.3 Preserve canonical interners and overlays

Exercise all frozen prefix families: types, kinds, rows, type parameters, family/variant types, static domains/constructors, scoped types, lambdas, static strings/symbols, tuple/sequence/record/data/variant/pack values. Insertion and lookup must resolve canonical or local instances correctly, without duplicate canonical identities or lost semantics.

Preserve:

```text
baseline frozen prefix -> fork A -> local U
                       -> fork B -> U absent, canonical T still present
```

Explicitly test structurally equivalent local insertions, foreign store lineage rejection, copied snapshots, and `extends_canonical_prefix` in both positive and negative cases.

### R.T1.4 Typed errors and transactional behavior

Remove ineffective `require_semantic_import_allocation_accounting() -> Ok(())` **only after** completing accounting. Do not replace it with another vacuous guard. Exhaustion/overflow must be typed without allocating diagnostic strings after no budget remains. A failed stage must not commit its child ledger or expose a partial snapshot.

### R.T1.5 Required tests/proofs

Add a **complete semantic-world import** independent allocator discriminator, not just a type-graph or callable-flow microtest. Build all input artifacts outside the measured interval; use representative real Universe semantic data and focused adversarial fixtures.

| Proof | Required result |
|---|---|
| Real `SemanticWorldV3` import via budget-aware entry | Success |
| Observed semantic import allocator requests vs child debit | Exact equality on successful import |
| Exact semantic stage budget `N` | Success |
| Stage budget `N-1` | Typed exhaustion |
| Failed semantic import | No parent child commit / no published snapshot |
| Type/static relocation completeness | All valid references resolve; invalid references rejected |
| Source vs imported owner families | Semantic identity, ordering and fingerprints agree |
| Frozen canonical fork and local extension | Works; no canonical mutation/reinterning |
| Missing callable/trait/dispatch/proof authority | Fail closed; no invented product |
| Relevant semantic unit/integration tests | Pass |

**Suggested commands:**

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib artifact::
```

Add and run the new complete-semantic-import target as needed; do not claim the current callable-flow/query-scratch test provides that coverage.

### Gate R.G1

**PASS iff** complete semantic import succeeds, all loaded owner families are classified/budgeted, independent successful-import allocator delta is zero, exact N/N−1 behaves correctly, malformed semantic world rejects, and canonical identity/fork/overlay parity passes. **Stop** if an existing semantic consumer requires mutable frozen authority; report exact consumer and architectural options instead of hidden native-map reconstruction.

---

## 6. R.T2 — Query-key decoding, product reconstruction and frozen query authority (original T7)

**Priority:** P0, critical path. **Dependencies:** R.G1. **Gate:** `R.G2`.

**Primary files:**

```text
phalcom-semantic/src/artifact/query.rs
phalcom-semantic/src/canonical_image.rs
phalcom-semantic/src/module_product.rs
phalcom-semantic/src/db/product.rs and query-read owners, only as needed
```

### R.T2.1 Make portable query-owner decoding allocation-transparent

**Known issue:** `decode_owner<T: DeserializeOwned>` builds a formatted tag prefix, performs `serde_json::from_str` into potentially owned structures, and formats errors without the ledger. `resolve_query_key` then performs owned stable-identity reconstruction.

Add a bounded/budgeted import API conceptually:

```rust
fn resolve_query_key_with_allocation_budget(
    key: &PortableQueryKeyV1,
    allocations: &mut AllocationBudget,
) -> Result<QueryKey, CanonicalImageError>;
```

Prefer `Deserialize<'de>` into borrowed intermediate views for JSON owner keys, then explicitly budget the owned stable identity. Where borrowed parsing cannot cover a required structure, write a bounded, schema-compatible parser or an independently verified fallible accounting-aware construction path. **Do not change `PortableQueryKeyV1` wire schema for convenience.** Ensure malformed JSON/tag/selector yields typed or legitimately budgeted errors.

Cover every supported portable query key kind: modules, declarations, callables, fields, type-member selectors, and module static bindings. Avoid `format!`/`to_string` in charged paths unless proven budgeted.

### R.T2.2 Budget all nested semantic product construction

`product_and_fingerprints(snapshot, key, modules)` currently has no budget parameter and constructs `Arc`, native maps, strings, cloned declarations/surfaces/signatures, enum variant arrays, and module binding catalogs. Thread `allocations` through every owning helper and nested product. Examples:

- linked-interface clones and owners;
- `ResolvedImportsProduct`'s paths and maps;
- declaration shell / callable signature / callable definition / field signature owners;
- trait/associated/data/enum products;
- enum variant vectors/`Arc` slices;
- module static-binding catalogs and hierarchy-edge products;
- outer `Arc<SemanticProduct>`.

Prefer shared immutable authority and `Arc::clone` when semantics allow. If a product must own a new map, use budgeted sorted construction; do not build an opaque B-tree and charge a logical estimate. **Preserve existing product and input fingerprint algorithms.** Recompute for *validation*, not to manufacture a replacement for missing persisted proof.

### R.T2.3 Freeze query entry and fingerprint indexes

Replace load-created `BTreeMap<QueryKey, CanonicalQueryEntry>` and `BTreeMap<QueryKey, ProductFingerprint>` with explicit budgeted frozen sorted storage behind minimal read adapters. Keep source-built query image mutable/native backing where appropriate. All `CanonicalQueryReader`, `CanonicalSemanticQueryImage::entries`, `entry`, `len`, and aggregate fingerprint consumers must read both forms identically. Preserve order and duplicate rejection.

Budget dependency edges and conversion to owned `Arc<[DependencyEdge]>` with exact helpers; include each cloned `QueryKey` and nested key storage.

### R.T2.4 Close query-image constructor's hidden allocations

`CanonicalSemanticQueryImage::from_loaded_artifact_entries` currently creates a census with native maps and a `BTreeSet` plus repeated leaf-removal scans. The imported constructor must have a budget-aware path. It may use an empty nonallocating census where allowable, a frozen census/index, or prebudgeted arrays. Reuse/revalidate already checked dependency information with a bounded linear-time DFS/topological worklist, not opaque B-tree scratch or repeated global rescans.

Keep independent validation of relocated dependency identities and the observed product fingerprint. No acceptance of cycles, missing dependencies, duplicate relocated keys, source-dependent roots or unsupported families.

### R.T2.5 Required tests/proofs

| Proof | Required result |
|---|---|
| All supported persisted portable roots imported | Every root accounted; no omitted root |
| Query key relocation | Stable semantic identity; duplicates rejected |
| Input/product/dependency fingerprint equivalence | Exact stable equality with source authority |
| Query reader from loaded image | Correct typed products and canonical origin |
| Invalid query owner tag/JSON/selector | Fail closed |
| Unsupported root, missing owner, cycle, missing/stale edge | Fail closed |
| Foreign semantic world / incompatible type/static prefix | Rejected |
| Successful independent reconstruction allocator observer | Delta = 0 |
| Query stage budget N/N−1 | Exact success / typed exhaustion |
| Read parity Source vs Frozen | Same order/get/len/fingerprint |
| Scratch complexity | Bounded DFS/worklist; no per-leaf full rescan |

Use `phalcom-semantic --lib artifact::`, query-focused tests and the semantic allocator test; add a complete reconstruction discriminator as a new isolated test if current tests cover only closure-validation scratch.

### Gate R.G2

**PASS iff** all supported roots reconstruct, all required product and dependency evidence is preserved, no absent owner is synthesized, no unsupported family is promoted, reader parity holds and independent successful-query-import requested-byte equality is exact. Do not proceed to publication with a known accounting delta.

---

## 7. R.T3 — Budgeted formal-image publication and failure atomicity (original T8)

**Priority:** P0, critical path. **Dependencies:** R.G2. **Gate:** `R.G3`.

**Primary files:**

```text
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/canonical_image.rs
phalcom-core/src/modules/canonical_artifact.rs
```

### R.T3.1 Stop duplicating loaded module authority

`ModuleQueryProducts::from_external_modules` currently clones unlinked and linked interfaces and populates B-trees for resolved/reverse imports. Provide a loaded-path representation that references already validated `CanonicalExternalModuleWorld` or retains only minimal budgeted indexes. Preserve source-side `ModuleQueryProducts` APIs via Source/FrozenBacking or a narrow read facade. Do not copy all interfaces into a B-tree just to present the historical query interface.

Verify `ModuleQueryProducts` consumer operations, reverse-import semantics, module visibility/privacy and topology ordering. Keep source authority and artifact authority semantically identical.

### R.T3.2 Add a budgeted loaded-only finalizer

Current `CanonicalUniverseFormalImage::from_loaded_artifact` clones `SemanticSnapshot`, builds `ModuleQueryProducts`, constructs `Arc`s and retains native dependency fingerprints without receiving a budget. Introduce:

```rust
CanonicalUniverseFormalImage::from_loaded_artifact_with_allocation_budget(
    world, modules, snapshot, queries, dependency_fingerprints, allocations,
)
```

Name flexible; preserve source-built constructors. Charge only actual new allocations (`Arc::clone` alone is not a new allocator request). Eliminate / explicitly budget allocation inside status changes, snapshots, query census, module query products, fingerprint owner construction, diagnostic/error paths and the `Arc<CanonicalUniverseFormalImage>`.

### R.T3.3 Thread the exact parent transaction through core

In `phalcom-core/src/modules/canonical_artifact.rs`, the current path loads module/semantic/query sections with the caller budget, then invokes unbudgeted `CanonicalUniverseFormalImage::from_loaded_artifact`. Replace this last call with the new budgeted constructor. Preserve metadata decoding/publication, `Arc<LoadedUniverseArtifact>` debit, stage timing semantics and header/capability validation.

Preserve the `LoadedUniverseArtifact` public compile APIs and compiler-only capabilities.

### R.T3.4 Validation, source-free invariant and publication failure cases

Must prove:

```rust
formal.sources().is_empty();
formal.snapshot().is_source_free_artifact_complete();
formal.reader().is_ok();
```

Additionally require no advisory module/callable products in the loaded image, compatible semantic world/prefix lineage, correct aggregate fingerprints, no source-backed query fallback, and no mutable canonical authority.

Construct failure discriminators after: module import; semantic import; query import; metadata validation; and immediately before final image `Arc` publication. On failure, publish **nothing**, do not commit a failed child and permit clean retry. Preserve typed allocation errors rather than allocating an owned error string after exhaustion.

### Gate R.G3

**PASS iff** a full real artifact yields a complete source-free formal image, `reader()` succeeds, all final owned requests are budgeted, no duplicate opaque module authority exists, and failed publication is atomic and retryable.

---

## 8. R.T4 — Make independent whole-load exact allocation a non-optional hard gate (original T9)

**Priority:** P0. **Dependencies:** R.G3. **Gate:** `R.G4`.

**Primary files:**

```text
phalcom-core/tests/artifact_allocation.rs
phalcom-core/src/modules/canonical_artifact.rs
phalcom-type-meta/src/allocation.rs
phalcom-compiler-artifact/tests/container_allocation_test.rs
phalcom-compiler-artifact/tests/query_allocation_test.rs
```

### R.T4.1 Unconditional independent allocator assertion

Remove the `STRICT_ALLOCATION_GATE` conditional. The regular dedicated test must assert:

```rust
assert_eq!(independently_observed_requested, ledger_requested);
```

Use real canonical artifact bytes built *outside* measurement; scope the observer around exactly the successful `load_universe_telosc_from_bytes` operation, disable before formatting or dropping the returned artifact. Avoid contention and asynchronous allocator noise; isolate the global allocator test target and use `--test-threads=1` as necessary.

### R.T4.2 Tight N/N−1 discriminator

Discover required `N` from a successful load under sufficient limits (not from an old hardcoded value). Assert success with exact `N` and **match** the intended typed allocation-exhaustion variant at `N-1`; `is_err()` alone is not proof. A reference to `322,547,484` bytes in old evidence is not a golden assertion.

### R.T4.3 Distinguish whole-load and local limits

Prove:

1. Section/stage local limit may fail while global headroom remains.
2. Whole-load cumulative limit may fail while each individual stage has enough local limit.
3. Failed child does not commit its debit to parent.
4. Invalid artifact and exhaustion cannot publish partial image.
5. After failure a retry at sufficient budget succeeds.
6. Empty/small/large vectors, nested strings/identities/Arc slices and scratch are accounted.
7. File-acquisition buffer, if measured via `load_universe_telosc(path)`, is included in that **separate** disk-load contract, not the caller-owned in-memory bytes contract.

### R.T4.4 Resource-accounting test semantics

For successful loads require independent byte equality. For allocation failures check precharge/failure atomicity and typed behavior without falsely comparing rolled-back parent debit to already attempted physical allocation. Verify both undercharges and double charges. Charge `realloc` as requested new layout rather than only net delta, consistent with adopted policy. Document any standard allocator zero-size request convention and account for it consistently.

### Gate R.G4 and commands

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-type-meta --test allocation_helpers
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact_allocation -- --nocapture --test-threads=1
```

**PASS iff** on a complete real artifact:

```text
load(N) = SUCCESS
load(N-1) = TYPED ALLOCATION EXHAUSTION
observed_allocator_requested_bytes - ledger_requested_bytes = 0
section/local limit and cumulative limit tests = PASS
failed child parent commit = NONE
partial publication = NONE
retry = SUCCESS
```

Do not tolerate any nonzero delta or suppress an assertion to advance the checkpoint.

---

## 9. R.T5 — Successful source/artifact compiler parity on real packages (original T10)

**Priority:** P0. **Dependencies:** R.G3 (and final acceptance depends on R.G4). **Gate:** `R.G5`.

**Primary files:**

```text
packages/slotmap/src/main.ph
packages/slotmap/tests/runtime/src/basic_test.ph  # rename physical file only as needed
packages/slotmap/tests/runtime/src/dense_test.ph  # rename physical file only as needed
packages/slotmap/tests/runtime/src/main.ph        # only for proven resolver adjustments
phalcom-core/examples/perf_audit_p7.rs
phalcom-core/tests/                         # programmatic parity test target
```

### R.T5.1 Normalize proven fixture errors only

Change `for (entry in propositions)` to valid `for entry in propositions` if still present. Rename `basic_test.ph` / `dense_test.ph` to `basic-test.ph` / `dense-test.ph` to meet physical naming convention. Verify logical module import mapping before touching import names. Preserve tests and package API/semantics.

If a new failure exposes inference, constructor accessibility, coverage, callable resolution or formal match-lowering defects, identify owner; do not rewrite the package to hide a compiler bug. VM `ProductKind` bootstrap is not part of this compiler parity gate.

### R.T5.2 Two independent compiler image paths

**Source-backed baseline:** Compile the canonical Universe from source, then compile the same package with `ProgramCompiler::compile_entry_selection_with_image` using the source image.

**Artifact-backed:** In a **fresh process**, load the persisted `universe.telosc` and call `LoadedUniverseArtifact::compile_user_entry` (or options variant) for the same package. Instrument the path to establish no source-backed Universe initialization/fallback; `image.sources().is_empty()` alone proves source-free authority, not filesystem denial.

For each workload assert **both** compilation calls produce `Ok`. Only then compare stable semantic and compiler observables. Raw per-process `TypeStoreId` and pointer identities need not match.

Required valid workloads:

- Minimal standalone program.
- Generic/trait or dispatch-bearing program.
- Structural type or type/static-value program.
- `packages/slotmap`.
- `packages/slotmap/tests/runtime` **compilation** (VM execution separate).
- Negative semantic program for diagnostic parity (not counted as successful compile).

### R.T5.3 Compare meaningful outputs

Compare as accessible in current APIs:

- successful completion and compiled module topology;
- module import and name resolution;
- declarations/callable identities and selected dispatch targets;
- type inference and generic specializations;
- trait/conformance resolution and field/member signatures;
- match/coverage semantics where present;
- stable diagnostics for negative fixtures;
- stable semantic/query fingerprints where defined;
- generated compile products or normalized outputs, avoiding incidental process-local IDs.

A test that compares only matching error strings is **insufficient**. In particular, the current `cmd_slotmap_eval` consumes both `Result`s with `.err()` and can report “100% parity” when both compilers failed. Replace with `expect`/`?` success assertions before comparing positive products. Negative parity belongs to a separate test.

### R.T5.4 Canonical reuse and fork discriminator

Prove: no source bootstrap in artifact mode; canonical query reader returns imported products; forked TypeStore/StaticValueStore extends the loaded prefix; local inserts do not mutate the prefix or leak into independent forks. Measure reused canonical query hits or otherwise inspect proof of imported product use, not just successful recomputation.

### R.T5.5 Correct failure classification

Use one of:

```text
FIXTURE_DEFECT
BASELINE_COMPILER_DEFECT
ARTIFACT_SPECIFIC_DEFECT
UNSUPPORTED_LANGUAGE_FEATURE
VM_RUNTIME_BOOTSTRAP_DEFECT (outside P7 compiler acceptance)
```

A shared baseline compiler defect may permit **documented allocation-hardening completion**, but **blocks PATCH5-R functional closure** until a valid real package succeeds through both compiler paths. At minimum keep the SlotMap blocker explicit; a passing substitute package must not be misrepresented as SlotMap passing.

### Gate R.G5

**PASS iff** positive source compilation **and** artifact-backed compilation both succeed for the agreed real package workloads, semantic outputs are equivalent, loaded canonical query/prefix authority is actually reused, and there is no Universe source recompilation in the artifact-backed process. VM execution success is not required for compiler-only P7.

---

## 10. R.T6 — Repair performance harness and make claims traceable (original T11)

**Priority:** P1, may proceed mechanically in parallel. **Dependencies for final measurement:** R.G4, R.G5. **Gate:** `R.G6`.

**Primary:** `phalcom-core/examples/perf_audit_p7.rs`.

### R.T6.1 CSV and child-process integrity

Implement one escaping helper for comma, CR/LF and double quote. All CSV records, including diagnostic/output columns, must be valid one-logical-sample-per-row. Each launched child must yield a validated row; missing rows, child panic, nonzero exit, unexpected sample count and unparsable output are explicit failures rather than silently missing observations.

### R.T6.2 Correct digest and determinism language

The artifact digest is a domain-separated **BLAKE3-256** digest, not SHA-256. Distinguish:

- **Same-image serialization determinism:** same frozen image encoded twice -> byte-identical artifact.
- **Cross-process full-build determinism:** process A source-builds artifact A, independent process B source-builds artifact B -> byte-identical bytes and section/header digests (required for an unqualified cross-process determinism claim).

Record any failure instead of calling determinism verified. Never share in-process canonical `OnceLock` between supposedly independent full builds.

### R.T6.3 Valid measurement boundaries and classifications

Capture separately: file read, container validation, section validation, relocation/query construction, final publication, user compilation, optional VM bootstrap and execution. `ru_maxrss` measures **peak** RSS; it does not measure current retained memory or prove leak freedom. Source-free authority (zero source units + compilation) is not equivalent to physical filesystem source-denial without its own discriminator.

A compile-error workload may be measured as time to diagnostic; it is not a successful-package speedup sample. Runtime `ProductKind` bootstrap failures must be labeled and must not be folded into successful VM execution timing.

### R.T6.4 Statistics from raw samples

Generate from raw data: sample count, successes, failures, min, mean, median, p95 (document percentile convention), max, standard deviation and peak RSS. Record exact git revision, artifact digest, toolchain, OS/CPU, optimization profile and command lines. Never manually change summary numbers without updating raw data.

### R.T6.5 Re-measure completed PATCH5-R

Rebuild release harness and regenerate:

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --release -p phalcom-core --example perf_audit_p7
./target/release/examples/perf_audit_p7 generate \
  --out target/telos/universe/universe.telosc \
  --json target/perf003-p7-audit/ARTIFACT.json
./target/release/examples/perf_audit_p7 run-cold-suite \
  --artifact target/telos/universe/universe.telosc \
  --samples 10 --csv target/perf003-p7-audit/RAW-COLD.csv
./target/release/examples/perf_audit_p7 run-warm-suite \
  --artifact target/telos/universe/universe.telosc \
  --repetitions 20 --csv target/perf003-p7-audit/RAW-WARM.csv
./target/release/examples/perf_audit_p7 slotmap-eval \
  --artifact target/telos/universe/universe.telosc
./target/release/examples/perf_audit_p7 allocation-check \
  --artifact target/telos/universe/universe.telosc
./target/release/examples/perf_audit_p7 source-denial-check \
  --artifact target/telos/universe/universe.telosc
```

Adjust if subcommand names change and record the actual final commands. Run runtime baseline separately and label VM bootstrap as blocked if it still fails; P8 work is excluded.

### R.T6.6 Non-regression investigation thresholds

Investigate (do not automatically reject correctness repair) if compared to a measured comparable baseline:

- artifact-load median worsens by >15%;
- artifact-backed peak RSS worsens by >15%;
- warm compile median worsens by >15%;
- artifact size changes >5% without semantic payload change.

Old requested-byte `N` changing is expected when missing charges are fixed. Compare actual allocator requests and RSS, not just the ledger value.

### Gate R.G6

**PASS iff** CSV rows are structurally valid, raw row counts reconcile with child executions, no failed compile is reported as a successful workload, correct digest/statistical terminology is used, claims link to executed evidence and final measurements describe final PATCH5-R revision. Workspace-speedup and memory-leak claims remain `UNVERIFIED` absent separate proof.

---

## 11. R.T7 — Full regression, adversarial tests, evidence and closure (original T12)

**Priority:** P0 correctness / P1 reporting. **Dependencies:** R.G0–R.G6. **Gate:** `R.G7`.

### R.T7.1 Mandatory failure/corruption matrix

Run or add discriminators for:

| Case | Expected result |
|---|---|
| Missing `MODULE_WORLD`, `SEMANTIC_WORLD`, `QUERY_IMAGE`, `METADATA` | Typed capability/validation rejection |
| Container digest mismatch | Rejected |
| Section digest mismatch | Rejected |
| Schema/ABI mismatch | Rejected |
| Native catalog fingerprint mismatch | Rejected |
| Incorrect World/package provenance | Rejected |
| Oversized encoded section and oversized decoded family | Rejected under appropriate limit |
| Invalid/duplicate module owner/import identity | Rejected |
| Invalid/duplicate type/static relocation | Rejected |
| Missing callable/dispatch/proof/semantic owner | Rejected; never synthesized |
| Unsupported/source-dependent query root | Rejected |
| Duplicate relocated query key | Rejected |
| Missing/stale/cyclic query dependency | Rejected |
| Foreign semantic world or incompatible canonical prefix | Rejected |
| Section-local and cumulative allocator exhaustion | Typed errors |
| Failure immediately before final publication | No partial image published |
| Retry after failed load | Success with sufficient limits |
| Source/frozen module query/read parity | Exact semantic equivalence |

Preserve all pre-existing corruption and compatibility tests. Add focused tests where the current suite lacks the only discriminator of an invariant.

### R.T7.2 Required integration/build/regression commands

```bash
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check --lib \
  -p phalcom-type-meta -p phalcom-modules -p phalcom-semantic -p phalcom-core
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-modules --test topology --test query
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test artifact-allocation -- --nocapture
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib artifact::
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test artifact_allocation -- --nocapture --test-threads=1
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-core --test core modules_canonical_artifact::
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace --no-run
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test --workspace
```

Use registered targets if naming differs. Also explicitly run canonical-image integration tests and the real package compilation discriminator; do not assume generic `cargo test --workspace` includes external Telos package fixtures. Record every nonzero exit with its owning layer and whether it pre-existed at R.T0.

### R.T7.3 Evidence bundle refresh

Refresh relevant documents under `docs/perf003-p7-audit/`:

```text
REPORT.md
FINDINGS.md
FAILURES.md
MEMORY.md
COMMANDS.md
ENVIRONMENT.md
ARTIFACT.json
RAW-COLD.csv
RAW-WARM.csv
RAW-RUNTIME.csv
RAW-TESTING.csv
```

Include machine-readable outputs where practical: allocation requested/observed/delta, N/N−1 error category, package compile matrix and fingerprints, query root counts, deterministic artifact digest, process count and outcome for every benchmark row.

Create `PERF003.C2.P7.PATCH5-R-walkthrough.md` alongside existing P7 implementation records. Link exact commands/logs/revisions and explicitly separate `PASS`, `FAIL`, `BLOCKED`, `UNVERIFIED`, and `OUT_OF_SCOPE` claims. Old measurements at `e4503086`/`a134a065` must not be presented as final measurements at the new revision.

### R.T7.4 Final lifecycle and commit discipline

- Update PATCH5-R status and parent P7 checkpoint truthfully.
- Commit bounded reviewable waves; preserve architectural/test provenance.
- Avoid unrelated formatting or compiler fixes except causal blockers of the acceptance gates.
- Push the final branch; verify local HEAD equals remote, worktree clean.
- Include the complete acceptance matrix and remaining P8 limitations in walkthrough.
- Declare overall `COMPLETE` only when **all** required gates pass.

### Gate R.G7

**PASS iff** relevant regression and negative-case tests passed or their documented baseline owner does not undermine P7 correctness, no PATCH5-R regression remains, all P7 functional gates G0–G6 pass, evidence is reproducible, and the worktree is clean/pushed. Do not use a known baseline *package compilation* failure as a waiver of R.G5.

---

## 12. Consolidated mandatory acceptance matrix (one-page closure checklist)

All mandatory cells must be filled with a command/evidence link, revision and `PASS` state before full closure.

| Gate | Proof objective | Required acceptance |
|---|---|---|
| R.G0 | Accurate baseline | Revision, clean/diff state, executed baseline, first blockers |
| R.G1 | Complete semantic import | Full world + frozen type/static owners; zero successful-import allocation delta; fork parity |
| R.G2 | Complete query reconstruction | All eligible roots/products/edges/fingerprints; zero reconstruction allocation delta |
| R.G3 | Correct final image | `sources().is_empty()`; source-free complete; reader valid; atomic publish |
| R.G4 | Exact cumulative load | `observed == ledger`; N succeeds; N−1 typed exhaustion; stage limits/rollback |
| R.G5 | Operational compiler artifact | Fresh-process artifact mode compiles valid independent package; source parity; reuse without Universe source compilation |
| R.G6 | Evidence integrity | Valid CSV, correct digest, truthful successful/failed classification, final revision benchmarks |
| R.G7 | Patch closure | Corruption tests, regression suites, walkthrough, clean pushed branch, no unresolved P0/P1 relevant failure |

### Additional non-optional correctness assertions

- All relevant authority/dependencies are loaded rather than guessed or recomputed through source fallback.
- Canonical type/static prefixes remain immutable through independent session forks.
- No source-dependent query family is incorrectly treated as reusable canonical authority.
- Invalid portable query JSON/owner selectors and malformed relocation fail closed.
- `N-1` failure must not be counted if caused by unrelated malformed artifact or semantic error.
- No allocation observed in the successful measured load may be silently excluded or approximated.
- Failure of full package compilation is never counted as successful source/artifact parity.

### Separate classifications

```text
Allocation-hardening closure:
  PASS only after G1 + G2 + G3 + G4 and relevant atomicity tests.

PATCH5-R closure / P7 functional eligibility:
  PASS only after G0 + G1 + G2 + G3 + G4 + G5 + G6 + G7.

P8 executable VM bootstrap:
  OUT_OF_SCOPE, regardless of P7 compiler-image PASS.
```

---

## 13. Escalation and prohibited shortcuts

Escalate with a concrete incident packet when any of these occurs:

1. A production consumer requires mutable access to frozen loaded authority.
2. Exact requested-byte accounting cannot be implemented with explicitly allocated layouts and fallible construction.
3. Frozen prefix + overlay semantics cannot preserve interning, type identity or static-value lineage.
4. A persisted query product depends on unavailable or open-world semantic authority.
5. New representation would change canonical iteration order, fingerprints, privacy or dispatch semantics.
6. Final publication cannot remain atomic, or a failure pollutes shared global cache.
7. A baseline compiler bug blocks successful valid-package compilation and cannot be fixed without scope/architecture review.
8. Existing source-freedom assumptions require a hidden Universe source fallback.
9. A proposed fix needs wire-format/ABI expansion or changes semantic authority rules.

Incident packet: revision, first failing command, minimal reproducer, causal call path, invariant violated, alternatives with consequences, recommended ruling, and test proving the selected design. Do not introduce a workaround while waiting for architecture decision.

**Rejected shortcuts:** estimated B-tree/im node bytes; global allocator interception in production; conditional strict-gate assertions; ignoring `N-1` error type; native map reconstitution behind a read facade; `Arc::new` wrapper charging while nested clone allocations are untracked; synthetic fingerprints/owners; source fallback to make a fresh compiler work; broad API rewrite; silently modifying SlotMap semantics; equal compilation-error strings treated as successful package parity; retiring corruption tests; claiming leak-freedom from peak RSS.

---

## 14. Implementation commit and evidence strategy

Preferred reviewable commits (names flexible, preserve causal separation):

```text
1. test(perf): anchor PATCH5-R failing gates and evidence
2. fix(semantic): wire budgeted frozen semantic import and owner closure
3. fix(artifact): reconstruct query owners/products with exact budgeting
4. fix(artifact): publish source-free formal image atomically and budgeted
5. test(perf): make full-load requested-byte equality unconditional
6. fix(slotmap): normalize valid package fixtures and prove compiler parity
7. fix(perf): repair benchmark harness and deterministic evidence
8. docs(perf): finalize PATCH5-R walkthrough and P7 closure decision
```

At every commit, run focused relevant tests. At integration checkpoints, run broader checks. Do not postpone strict equality to a final cosmetic assertion; its failure is diagnostic information for the actual allocation ownership gap.

### Required final report template

```markdown
# PERF003.C2.P7.PATCH5-R — Implementation Walkthrough

## Identity
- Branch / starting HEAD / final HEAD / remote / worktree status:

## Gate Results
| Gate | Status | Evidence | Remaining blocker |
|---|---|---|---|
| G0 | | | |
| G1 | | | |
| G2 | | | |
| G3 | | | |
| G4 | | | |
| G5 | | | |
| G6 | | | |
| G7 | | | |

## Architecture and Changes
- Frozen/source backing adaptations:
- Allocation owner ledger changes:
- Query identity/product changes:
- Publication/failure-atomicity changes:

## Independent Allocation Proof
- Measurement boundary:
- Ledger requested N:
- Independently observed requested bytes:
- Delta:
- Exact N result:
- N-1 typed error:
- Section and whole-budget limits:
- Failure rollback and retry:

## Canonical Semantic and Query Proof
- Type/static relocation:
- Fork and overlay tests:
- Query roots/products/fingerprints:
- Missing authority / corruption rejection:

## Compiler Functional Proof
- Source package compile result:
- Artifact package compile result:
- Fresh-process source-free behavior:
- Semantic parity:
- SlotMap status:
- VM runtime boundary (P8):

## Regression and Performance Evidence
- All commands and exit codes:
- Benchmark raw rows and summaries:
- Artifact identity/digest and size:
- Memory measurement semantics:
- Measured/nonmeasured claims:

## Closure Decisions
- Allocation hardening: PASS/FAIL
- PATCH5-R: PASS/FAIL
- P7 compiler-image closure: ELIGIBLE/BLOCKED
- P8 implementation: AUTHORIZED/BLOCKED
- Exact remaining follow-ups and owners:
```

---

## 15. P8 preparation boundary and exit decision

PATCH5-R does **not** implement P8. Once complete, it must provide P8 with:

- An immutable compiler-only `universe.telosc` image with proven source-free compiler authority.
- A validated `MODULE_WORLD`/`SEMANTIC_WORLD`/`QUERY_IMAGE`/`METADATA` ABI contract.
- Stable canonical relocation and query read interfaces.
- Bounded and exact allocation ownership through publication.
- A fresh-compiler package compilation discriminator.
- Reproducible baseline metrics and manifest of explicitly deferred VM requirements.

P8 design preparation may proceed in parallel, including executable-image schema, VM relocation, runtime type/`ProductKind` identity architecture ruling, initialization order, executable parity tests and performance baselines. **P8 implementation is not authorized by mere PATCH5 infrastructure progress.** It requires formal P7 closure.

### Final decision rule

```text
If any R.G0–R.G7 mandatory gate is FAIL, BLOCKED or UNVERIFIED:
    PATCH5-R = IN_PROGRESS or BLOCKED
    P7 full compiler-image closure = NOT ESTABLISHED
    P8 implementation = BLOCKED

If all R.G0–R.G7 mandatory gates PASS:
    PATCH5-R = COMPLETE
    P7 compiler-only image = CLOSURE ELIGIBLE
    P8 may begin against its stable compiler-artifact baseline
```

**North-star discriminator:** a new compiler process loads `universe.telosc`, reuses canonical authority without analyzing Universe sources, extends frozen prefixes for its own user program, and successfully compiles `packages/slotmap` with source-equivalent semantic results—while independent instrumentation proves every allocator-requested byte of artifact loading was budgeted exactly. Nothing less is full corrective closure.
