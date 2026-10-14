---
id: PERF003.C2.P5
program: PERF003
checkpoint: PERF003.C2
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
---

# PERF003.C2.P5 — Canonical Publication and Persistent Image Boundary Walkthrough

Final certified completion record on `perf/universe-characterization`, 2026-10-14.
This plan establishes the explicit, immutable canonical compiler-image publication boundary
required before the Universe can be persisted as `universe.telosc` in `PERF003.C2.P7`.

The continuation replaces a Universe-wide dependency bypass with an exact,
image-owned structural fingerprint index. Canonical dependency evidence stays
separate from frozen-query readiness, with local-state precedence and no mutable
canonical query copies. Query freeze rejects structural rows changed by final
return settlement and their dependent closure. Existing generic hierarchy
fingerprints preserve validated superclass source syntax.

Parent overlays now cover dispatch surfaces/conditional members/type
registrations, data, enums/variants, associated surfaces, enum requirements,
and type-member declarations. The snapshot shares dispatch's surface map, and
the resolver retains associated/type-member views without flattening parents.
First-update factories now use empty local overlays. Core map consumers were
adapted mechanically to the new merged iteration API.

Imported alias formation now fingerprints the resolved alias result before
cache reuse. Cycle quarantine removes the affected alias component and its
transitive alias dependents, cached declaration/static products and retained
forms. Unaffected canonical/local aliases survive; the original alias-shell
republication assertion still passes. Three global-empty alias fixture
assumptions were corrected to exact invalid identity checks under the adviser
ruling, retaining cycle diagnostics and declaration-absence checks.

Adviser rulings clarified aggregate dependency authority, final-content freeze
eligibility, affected-component alias quarantine, and foreign inherent target
ownership. Independent focused review found a computed-key readiness consumer,
which now uses exact dependency evidence; no remaining findings were reported.

Focused evidence: 13 recomputation, 18 workspace, seven alias capability,
12 canonical-image library, 10 parent library and four canonical-image
integration tests passed, plus one computed Record-key signature regression
(65 selected tests total). Core with observability and LSP checks passed.
CHECKPOINT.md records the exact targets, timings and limits. New tests cover
canonical mutation/collision safety, inherited dispatch and removal, exact
aggregate dependency recording/missing-key/mismatched-fingerprint rejection,
stale freeze closure, canonical sharing and alias cycle recovery.

P5 remains PARTIAL. Residual canonical lookup maps, persistence descriptors,
query eligibility extraction, observability closure, P4-dependent metadata
composition and final certification remain open. No `.telosc` is emitted and
cold source-built acquisition remains. Broad release suites, full-workspace
formatting and performance certification were deliberately deferred. The
updated baseline-failure-fix handoff contains the next bounded work.

## Residual lookup and query-classification increment (`eeca76f3`)

Alias, static-definition and static-constant tables now parent-read the image's
immutable products on first local update; local removals do not touch the
parent, and collisions are rejected. Retained alias forms share the published
type-alias table instead of copying every canonical form. Resolver static
definitions retain the parented table. Accepted callable definitions now use
`ParentMap` across snapshot publication and local revision cloning.

Query family eligibility is an exhaustive, exported `QueryKind` classification
used by `CanonicalSemanticQueryImage::freeze`. The validated Ready row, current
origin, matching final aggregate and closed acyclic dependency requirements
remain unchanged. This is a T9 classification extraction, not a serialized
query-image implementation.

Test-first canonical replacement coverage detected alias, static constant and
accepted callable entry copying and passes after correction. It also verifies
static-definition sharing, local-only entry counts, parent-safe removal and
collision rejection. Focused post-change evidence: 13 recomputation tests,
seven alias tests, 18 workspace tests, 13 canonical-image library tests, one
final image replacement integration test and one exact callable/body edit test
passed. Core observability check passed after the final source change. The
test harness has a pre-existing unused import warning in `workspace.rs`.
Broad release/format and performance certification were deferred.

T6 Slice 4 remains partial: static-case/associated static products, module
static bindings and other copied maps still need ownership conversion. T7–T8,
T11–T13 remain open; P4 gates T10/final P5 acceptance. No `.telosc` persistence
or certified zero-copy claim across all families follows from this increment.

## Module, constructor and trait parent continuation (`ca3c3c03`)

Module binding facts now flow through a canonical-parent `ParentMap` in the
checker and snapshot. Constructor paths, trait headers and trait surfaces
likewise retain canonical parents; focused three-revision replacement tests
prove pointer sharing. Parent-only map iteration/count and constructor/trait
local/parent counts are exposed for T7. Canonical-image integration passed
four selected; workspace integration 18; the exact module-value dependency
test, parent-map tests and core observability check passed. Static-case maps
have no canonical entries in this image; conformance aggregates still clone.

P4 prerequisites were explicitly resumed by the user. PR-A Record/Pack
native-facade coverage passed. The canonical compiler singleton still fails
at `FixedFields` metadata export because v12 cannot encode the non-Symbol
remainder. The approved PR-B graph normalization requires loader-supplied
canonical Symbol identity for old bundles lacking a Symbol node; standalone
decode must return a typed missing-context error for affected bundles. No
schema change shipped here. P5 remains PARTIAL / FOCUSED_TESTED.

## Layered conformance continuation

Explicit conformance heads and guarded derived rules now use immutable
canonical parents with local-only mutable secondary indexes. Trait-local
derived-rule access preserves indexed and iterable reads through a borrowed
two-slice view. Conformance exact candidates and coherence cross-pairs are
merged from the two indexes without cloning parent buckets into local state.
Focused tests passed for canonical/local overlap against brute force, original
index partition parity, derived-rule incremental/cold parity and condition
invalidations, actual image replacement sharing, and a core check. Witness
plans are empty in the current canonical image; a general parented witness
contract, full persistence capabilities and metadata composition remain outstanding.

The overlapping GitHub-only continuation below was unverified at publication;
its inventory and useful tests were retained during branch integration.

## P5 conformance-index parent continuation (`b99e0507`; follow-up structural assertions)

- `ConformanceIndex` now keeps a shared immutable canonical parent and mutable
  local-only contribution/secondary-index buckets. The first image-backed
  workspace update uses `ConformanceIndex::with_parent`; retained revisions
  clone only local contributions and the parent handle. Read-through retains
  exact `ImplId` lookup and deterministic merged iteration.
- Indexed exact conformance queries combine canonical and local trait/family
  candidates without materializing a merged conformance table. Overlap
  detection retains canonical/canonical and local/local checks and includes
  canonical/local candidate pairs, so local heads cannot evade coherence
  checking against canonical heads.
- Canonical insert collisions are rejected. `remove` and `remove_source`
  affect only local contributions. Added focused unit coverage for cross-layer
  query/coherence, removal safety, collision rejection and local counts.
  Expanded the existing three-revision image-session integration regression to
  assert zero local conformance entries and pointer-identical canonical reads.
- The preceding `791a0c0d` introduced a typed persistence-family inventory
  and an in-memory capability census (P5.T8 partial). This is a planning
  contract, **not** a portable DTO schema, byte encoder or portable digest.
- **Verification status:** Both source increments and the new regression
  assertions are committed, but no Rust check/test, formatter, or benchmark
  was executed in this GitHub-only continuation. No GitHub Actions workflow
  runs were returned for the branch. Existing `9143262b` test evidence does
  not certify either later increment.
- **Remaining:** witness-plan maps and derived-rule buckets still clone
  canonical entries; T7 read hardening, full T8 compatibility/provenance
  descriptor, T10 canonical RuntimePublic metadata composition and T11–T13
  certification remain. P4's `FixedFields` blocker is not assumed resolved.
  No `.telosc` was written, and P5 must remain PARTIAL.

Recommended first verification commands after branch synchronization:

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --lib conformance_index_partition_tests
RUSTFLAGS='' cargo test -p phalcom-semantic --lib canonical_image
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::canonical_image
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::traits
RUSTFLAGS='' cargo check -p phalcom-core --features perf-observability
```

## Derived-rule image parent and bounded-read refinement (`e3f1add8`, `2819af8f`)

The derived rule index now preserves canonical rule buckets as one immutable
parent and stores only local rule additions/replacements in mutable buckets.
A borrowed `DerivedRuleTraitView` exposes `len`, `get`, indexing and
iteration over canonical + local source rules, without constructing a merged
`Vec`. A single parent layer is enforced to support nonallocating iteration.
`ConformanceIndex::iter` uses the same one-level read model.

A new integration assertion checks, for three successive image-backed local
revisions, that derived rules remain parent-owned and resolve to the exact
canonical `Arc`-backed objects. Nested-parent rejection has a dedicated unit
regression. Local derived-rule re-publication removes prior local trait-bucket
membership before inserting the replacement; canonical collisions remain
forbidden.

Persistence classification was strengthened: source-semantic attachments
required by downstream compilation are not designated rebuildable while P7
has no proof they can be recreated with Universe source unavailable.

**Verification gap:** GitHub commits are published but no Rust checks,
tests, formatting commands, CI workflows, performance benchmarks or cold
compiler tests have been run for these source changes. The last actually
focused-tested revision is `9143262b`. Do not claim complete G6 zero-copy:
`conformance_witness_plans` remains a cloned `BTreeMap`, and P4's
canonical metadata authority is still gated per the recorded incident.

Minimum required verification before attempting the witness-plan migration:

```sh
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo test -p phalcom-semantic --lib conformance_index_partition_tests
RUSTFLAGS='' cargo test -p phalcom-semantic --lib canonical_parent_tests
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic integration::canonical_image
RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic incremental::derived_conformance
RUSTFLAGS='' cargo check -p phalcom-core --features perf-observability
```

## Reconciled branch verification

The concurrent remote inventory and conformance commits were merged with the
locally tested layered conformance implementation. The resulting index retains
stable-ID sorted, allocation-free read iteration and parent/local coherence
parity without copying canonical buckets; the borrowed derived-rule view also
supports `get` and indexing. Duplicate assertions in the three-revision image
regression were removed. The remote `CanonicalImageCapabilities` and 22-family
inventory remain, with source attachments conservatively required.

After reconciliation, semantic `cargo check`, two conformance partition tests,
14 canonical-image library tests, one derived parent test, four canonical-image
integration tests, one derived incremental/cold parity test and the core
observability build check passed. The earlier remote "unverified" statements
above describe the GitHub-only commits before integration; this paragraph
records the integrated evidence.

## Final Certification & Completion (P5.T10 – P5.T13)

Following the completion and certification of `PERF003.C2.P4` and the resolution
of the trait evidence slot lowering incident:

1. **P5.T10 Canonical Compiler Image View Boundary:**
   - Implemented `CanonicalCompatibilityIdentity` and `CanonicalCompilerImageView` in
     `phalcom-core::modules::canonical_universe`.
   - Directly wired the semantic `CanonicalUniverseFormalImage` into `CanonicalUniverseProgram`.
   - Composes the immutable formal image, authoritative `RuntimePublic` metadata bundle,
     and compatibility identity (`semantic_world`, `semantic_model_version`,
     `native_surface_schema_version`, `metadata_schema_version`, fingerprints, extent).
   - Validated via unit test `canonical_compiler_image_view_composes_formal_image_metadata_and_compatibility`.

2. **P5.T11 Observability Cardinality Verification:**
   - Added `CanonicalAggregateEntriesCopied` and `CanonicalParentHandlesAttached` to
     `telos_observability::PerfCounter`.
   - Instrumented `SemanticWorkspaceSession::from_image` to record 0 copied entries
     and 10 attached canonical parent handles.
   - Guarded via automated integration test `from_image_zero_canonical_entries_copied_cardinality_guard`.

3. **P5 Verification Matrix:**

| Gate | Target / Test Filter | Result | Time |
|---|---|---|---|
| Image View Composition | `cargo test -p phalcom-core --lib modules::canonical_universe::tests::canonical_compiler_image_view_` | **PASS** (1 selected) | 42.90 s |
| Cardinality Guards | `cargo test -p phalcom-semantic --test semantic from_image_zero_canonical_entries_copied_cardinality_guard --features perf-observability` | **PASS** (1 selected) | 40.72 s |
| Canonical Program Tests | `cargo test -p phalcom-core --lib modules::canonical_universe::tests` | **PASS** (7 selected) | 41.67 s |
| Canonical Formal Image Library | `cargo test -p phalcom-semantic --lib canonical_image` | **PASS** (15 selected) | 40.76 s |
| Canonical Image Integration | `cargo test -p phalcom-semantic --test semantic integration::canonical_image` | **PASS** (5 selected) | 131.79 s |
| Parent Map Integrity | `cargo test -p phalcom-semantic --lib parent_map` | **PASS** (2 selected) | 0.00 s |
| Module Compilation Suite | `cargo test -p phalcom-core --lib modules::compile::tests` | **PASS** (3 selected) | 42.41 s |
| Runtime Reflection Display | `cargo test -p phalcom-core --test core applied_record_row_metadata_display_preserves_field_types_and_tail` | **PASS** (1 selected) | 0.01 s |
| Complete Typing Suite | `cargo test -p phalcom-core --lib typing::` | **PASS** (13 selected) | 0.01 s |
| Type Meta Schema Compat | `cargo test -p phalcom-type-meta --test schema_compat` | **PASS** (23 selected) | 0.00 s |
| Core Observability Build | `cargo check -p phalcom-core --features perf-observability` | **PASS** (clean) | 10.53 s |
| Semantic Observability Build | `cargo check -p phalcom-semantic --features perf-observability` | **PASS** (clean) | 3.80 s |
| Scoped Diff Hygiene | `git diff --check` | **PASS** (clean) | 0.00 s |
