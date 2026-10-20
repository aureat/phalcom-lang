# IMPLEMENTATION INCIDENT

> **Disposition — 2026-10-15:** Architectural consultation **DECIDED** by [PERF003.C2.P7-allocation-materialization-architecture-ruling-2026-10-15.md](PERF003.C2.P7-allocation-materialization-architecture-ruling-2026-10-15.md). The approved approach is budget-aware, immutable ordered materialization plus bounded scratch, preserving exact cumulative allocator-requested-byte accounting and independent section limits. This resolves the decision request in §14, **not** the implementation or verification failure: R1/R7 and P7 remain **IN_PROGRESS / PARTIAL / UNVERIFIED**. Do not claim native-map allocations accounted before the representation and its tests are implemented.

### Continuation correction — 2026-10-15

Sections 5–9 below describe the incident's pre-correction baseline and are
partially superseded by the current implementation. The container path now has
`parse_with_allocation_budget`: package identity strings and its directory
vector are charged, and duplicate/overlap validation uses bounded scans rather
than uncharged hash-set/cloned-sort scratch. The core load transaction now uses
the independent `max_total_allocation_bytes` cap. Module duplicate/index scratch
uses budgeted vectors, and runtime DAG validation uses budgeted dense CSR
scratch. The budget-aware sorted map/set builders are available for subsequent
owner migration. Exact focused probes passed for the helper, container, and
module DAG slices.

Still open: most semantic/query output BTreeMaps, nested owner values and
relocation/index allocations, formal-image construction, error paths, and the
allocator-observed full Universe load. The original path sketches remain useful
for unconverted owners but must not be read as saying container directory
allocation is still uncharged. R1/R7 and P7 remain IN_PROGRESS / PARTIAL /
UNVERIFIED.


## 1. Identity

- Program: `PERF003`
- Checkpoint: `PERF003.C2`
- Plan: `PERF003.C2.P7`
- Task/Gate: R1/R7 allocation coverage; shared-budget native materialization
- Repository: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/perf-universe-characterization`
- Branch: `perf/universe-characterization`
- Starting revision: `b579164337aff4d170414eb4f8e7c32881063a80`
- Current revision: `b579164337aff4d170414eb4f8e7c32881063a80` (working tree remains uncommitted)

## 2. Escalation trigger

Shared Consultation Protocol RED trigger 11: viable accounting choices for
native ordered maps have materially different effects on the allocation
contract and representation. The active plan requires exact cumulative
allocator-requested-byte accounting, while the user explicitly prohibits
choosing a B-tree estimate, mirroring private node layouts, changing canonical
map representations, or adding allocator interception without an approved
decision.

## 3. Required invariant

One load transaction must account cumulative allocator-requested bytes across
decoding, native relocation/materialization, indexes, scratch, errors, and
publication. Charge each requested layout before construction; keep section
limits independent from the whole-load limit; commit a child ledger only after
successful completion. Do not substitute an estimate for allocator-requested
bytes or claim that uncharged native work is bounded.

## 4. Expected architecture

`phalcom-core` owns one `AllocationBudget` for the load and passes it through
wire decoding, module materialization, semantic import, query reconstruction,
metadata decoding, and publication. A child ledger bounds each section/stage
and is committed only on success. Exact layouts for explicitly reserved
vectors and `Arc<T>` are charged at their construction sites. The effective
`docs/specs/compiler-artifacts/telosc.md` requires total allocation to be
bounded. The adopted P7 allocation clarification disallows arbitrary
estimators, allocator hooks, and per-section budget resets.

## 5. Observed behavior

The shared-budget entry points now exist and are called by the core loader.
Exact charges have started for explicit module/query vectors, selected
associated-authority vectors, and selected `Arc<T>` products. The following
native ordered-map operations still allocate without ledger charges:

- `phalcom-modules/src/compiler_artifact.rs::load_module_world_inner` builds
  `BTreeMap` state for interfaces, linked modules, topology nodes/children,
  exposed children, graph rows, and validation indexes.
- `phalcom-semantic/src/artifact/query.rs::validate_query_image_closure` uses
  `BTreeSet` scratch; query reconstruction builds `BTreeMap` entries and
  fingerprints. Resolved-import product reconstruction also builds a map.
- `phalcom-semantic/src/artifact/import.rs` starts with type-graph import and
  later constructs `BTreeMap`/map-backed semantic authority and indexes.

Other native allocations in the path also remain outside the ledger: the
container parser's directory vector, module-component owned strings, semantic
snapshot and formal-image `Arc`s, query product `Arc`s and dependency `Arc`
conversion, and many semantic import vectors/boxes/arcs. The loaded-formal-image
constructor has no budget parameter, and the shared transaction is not passed
to `TeloscContainerView::parse`.

`AllocationBudget` has no approved budget-aware `BTreeMap`/`BTreeSet`
construction seam. The pinned standard-library facts `B = 6` and node
capacity 11 do not define an approved requested-byte policy. Charging
`size_of::<(K, V)>() * len` would not describe B-tree node requests. No
allocator measurement of native materialization was run in this continuation.

## 6. Minimal reproduction

Static path (not run as an allocator-counting reproduction):

```text
load_universe_telosc_from_bytes_with_timings_and_budget
  -> TeloscContainerView::parse (directory vector, no shared-budget API)
  -> load_module_world_with_allocation_budget
  -> load_module_world_inner
  -> BTreeMap::new / insert for module authority
```

The expected behavior is that each allocator request made by native map
materialization consumes the shared transaction before construction. Current
code has no approved way to debit those node requests. A passing compile or a
decoder-only allocation test does not exercise or prove this materialization
bound.

## 7. Relevant code path

```text
phalcom-core::load_universe_telosc_from_bytes_with_timings_and_budget
  -> TeloscContainerView::parse (directory scratch remains uncharged)
  -> compiler-artifact section decoders (shared AllocationBudget)
  -> phalcom-modules::load_module_world_with_allocation_budget
       -> load_module_world_inner -> BTreeMap/BTreeSet-backed module products
  -> phalcom-semantic::validate_and_import_semantic_world_with_allocation_budget
       -> TypeStore::import_type_graph and semantic map-backed owners
  -> phalcom-semantic::reconstruct_query_image_with_allocation_budget
       -> validate_query_image_closure (BTreeSet scratch)
       -> entries/fingerprints maps
  -> metadata decode and loaded-artifact publication
       (formal-image Arc is created inside an API with no budget parameter)
```

## 8. Scoped implementation diff

- Added a direct `phalcom-type-meta` dependency to `phalcom-modules`.
- Added shared-budget wrapper APIs for module-world materialization, semantic
  world import, and query-image reconstruction; existing APIs remain thin local-
  budget wrappers.
- Threaded the existing core transaction through all three APIs.
- Charged selected exact vector reservations and `Arc<T>` layouts before
  construction. These charges are partial and do not include native map nodes,
  all identity/string clones, or all scratch.

## 9. Evidence

- `phalcom-modules/src/compiler_artifact.rs`: shared wrapper at
  `load_module_world_with_allocation_budget`; uncharged map construction begins
  at the module authority maps in `load_module_world_inner`.
- `phalcom-semantic/src/artifact/import.rs`: shared wrapper at
  `validate_and_import_semantic_world_with_allocation_budget`; associated
  authority vectors and `Arc<T>` products have explicit charges, while its
  map-backed owners remain uncharged.
- `phalcom-semantic/src/artifact/query.rs`: shared wrapper at
  `reconstruct_query_image_with_allocation_budget`; root/edge vectors and final
  `Arc<T>` have explicit charges, while closure scratch and output maps remain
  uncharged.
- `phalcom-core/src/modules/canonical_artifact.rs`: passes the same transaction
  to section decode, module materialization, semantic import, query decode and
  reconstruction, metadata decode, and final loaded-artifact publication.
  `TeloscContainerView::parse` and `CanonicalUniverseFormalImage::from_loaded_artifact`
  still allocate outside that ledger.
- PASS: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-modules --lib`.
- PASS: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic --lib`.
- PASS: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --lib`.
- PASS: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib artifact::import::tests::associated_authority_round_trips_through_type_relocation -- --exact --nocapture` — 1 selected, 1 passed.
- PASS: `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-compiler-artifact --test query_allocation_test -- --nocapture` — 1 selected, 1 passed.
- A scoped `rustfmt --edition 2024 --check` over the touched Rust files reported
  formatting differences throughout these already-modified files, beyond this
  slice. No broad formatting was applied.

## 10. Implementer's causal hypothesis

The transaction API itself is not the remaining problem. Standard-library
ordered-map allocations are hidden behind `BTreeMap`/`BTreeSet` construction and
mutation, so there is no current caller-owned point to reserve each physical
node request before it occurs. Exact accounting requires an approved allocator
or representation seam, or a revised allocation contract. The implementer
cannot decide among these because the alternatives alter the contract or
ownership/representation architecture.

## 11. Alternative hypotheses

- The standard-library map may request different node layouts as nodes split
  and occupancy changes; entry count multiplied by pair size is therefore not
  exact.
- A conservative logical per-entry debit might bound an abstract amount but
  would change the fixed allocator-requested-byte contract and would not prove
  actual requested bytes.
- Mirroring private node layouts couples accounting to implementation details
  and has not been approved.
- Replacing map owners with budget-aware collections could provide a visible
  allocation seam but changes representation and may affect canonical owner
  APIs.
- Allocator interception is explicitly disallowed by the adopted plan.

## 12. Attempts already made

- Added the shared-ledger wrappers and core call-through; corrected the
  mechanical core imports after its first compile check.
- Added charges only where the code has explicit vector capacities or uses the
  existing exact `AllocationBudget::arc` helper.
- No map estimate, private node mirror, representation replacement, or
  allocator hook was attempted, per the fixed decisions.

## 13. Why local authority is insufficient

The implementer can apply ordinary helper/signature changes but cannot change
the allocator contract or native owner representation. The plan requires exact
requested bytes, and the user explicitly leaves the map-accounting architecture
unresolved. Continuing would require choosing a materially different contract
or architecture, so this issue is stopped pending an approved decision.

## 14. Decision requested

Approve the native `BTreeMap`/`BTreeSet` accounting seam and policy that lets
the loader charge actual allocator-requested layouts before map allocation,
while preserving the fixed cumulative transaction/section semantics. State
which owner/representation changes are authorized if the standard map API
cannot expose that seam. Until that decision, leave ordered-map allocations
explicitly uncharged and keep P7 allocation closure open.
