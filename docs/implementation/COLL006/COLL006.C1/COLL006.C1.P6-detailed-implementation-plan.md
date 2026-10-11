---
id: COLL006.C1.P6
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: implementation-plan
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
depends_on:
  - COLL006.C1.P5
follows: COLL006.C1.P5
supersedes: null
repository: aureat/phalcom-lang
branch: feat/telos-collection-upgrade
recorded_against_revision: dea2cd1d83b148e71b2ce0dd442e53636b3e8917
---

# COLL006.C1.P6 — Ordered Mutable Hash Family

## 0. Executor contract

Implement the P6 ordered-hash substrate and the three mutable collection
families under the architecture already recorded by the C1 checkpoint and
master plan. Preserve existing Map/Set behavior. Adapt mechanical paths and
private helper names to the live tree. Stop for consultation before changing
ownership, semantic identity, callback/re-entry behavior, or any public
collection contract not settled by the governing records.

Use focused verification at each gate. Do not run workspace/release gates; P9
owns those. Keep P5's parse-package residuals assigned to P9.

## 1. Goal

Factor Telos hash/equality and callback re-entry into one internal authority,
then implement `MutableBag<T>`, `IndexedDict<K,V>`, and `OrderedSet<T>` while
preserving current `Map` and `Set` semantics.

## 2. Checkpoint objective

P6 is one of the plans required for `COLL006.C1`; it does not close the
checkpoint. It advances requirement 14 (reusable ordered-hash machinery) and
the mutable hash-family portion of requirements 15–20. P7–P9 remain separate.

## 3. Repository grounding

```text
repository: aureat/phalcom-lang
branch: feat/telos-collection-upgrade
revision at plan start: dea2cd1d83b148e71b2ce0dd442e53636b3e8917
predecessor: COLL006.C1.P5 COMPLETE / IMPLEMENTED / BASELINE_BLOCKED
```

Verified at plan start:

- `MapObject` is an insertion-ordered `Vec<(Value, Value, i64)>` plus hash
  buckets and a re-entry depth counter in `phalcom-core/src/heap/map.rs`.
- `Object::Map` and `Object::Set` have distinct heap accessors but reuse
  `MapObject` storage.
- `primitive/map.rs::locate_key` and `primitive/set.rs::locate` duplicate
  Telos `hash`, candidate `==`, numeric SameValueZero, and re-entry handling.
- `primitive/mod.rs::is_mutable_collection_key` is the central key-admission
  classifier and already includes P5 Array/Slice/Buffer work.
- Public Map/Set implementations live in
  `phalcom-core/core/universe/src/collections/{map,set}.ph`.
- `docs/specs/README.md` is the current specification authority index; it has
  no Map/Set or new ordered-hash collection specification. The accepted TDR
  records and C1 architecture documents supply the extant implementation
  direction, but do not fully specify the new public selector surfaces.
- `COLL006.C1.P5-handoff.md` records `R-MAP-REMOVE` as a known pre-P5
  collection-contract failure reproduced on `78cfd920`; preserve and classify
  that failure without treating it as caused by P6.

Re-read the named live paths before editing. Adapt mechanical drift locally;
architectural drift is an escalation condition.

## 4. Required reads

- `AGENTS.md` and this worktree's applicable directory guidance.
- `docs/workflow/implementation-record-lifecycle-convention.md`.
- `docs/workflow/luna-patch-grade-plan-schema.md`.
- `docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md`.
- `docs/implementation/COLL006/COLL006.C1/COLL006.C1-master-implementation-record.md`
  §§3.6–3.8, 7.5, 12, 13, 21, and 23.
- `docs/implementation/COLL006/COLL006.C1/COLL006.C1-detailed-implementation-plan.md`
  §§13.1–13.7, 21, and 23.
- `docs/implementation/COLL006/COLL006.C1/COLL006.C1.P5-handoff.md`.
- `docs/decisions/accepted/0028-collections-representation-and-literals.md`.
- `docs/decisions/accepted/0033-amend-floor-admit-collection-container-primitives.md`.
- `phalcom-core/tests/README.md` and the exact Map/Set tests/fixtures named below.
- Live source paths listed in §§6 and 11.

## 5. Authority and semantic boundary

### Fixed by accepted decisions and C1 records

- Existing Map/Set use Telos `hash` and `==`; never use Rust `Value` equality
  or identity as the semantic key relation.
- No Heap borrow may remain live across a Telos hash/equality callback.
- Structural mutation of the collection being inspected by its key callback
  remains rejected; permitted reentrant reads and error propagation remain
  intact.
- Map insertion/encounter order is preserved; Map equality is by associations
  and is order-insensitive.
- Set encounter order is preserved; Set equality is membership-based and
  order-insensitive.
- `IndexedDict` key lookup and positional access are distinct operations;
  positional order is semantic/equality-sensitive. Replacing a value does not
  move its key; remove/reinsert appends unless an explicit position is used.
- `OrderedSet` is unique, order-sensitive, and uses explicit positional
  mutation. The canonical name is `OrderedSet`, never `SetList`.
- `MutableBag` stores strictly positive exact Telos-Int multiplicities; total
  count equals their sum.
- Hash collision handling must compare candidates by Telos equality.

### Specification gap requiring consultation before new public API work

The current `docs/specs/` corpus has no owning specification for Map/Set or
the P6 collection APIs; older references to `docs/spec/current` are stale. The
P6 architecture records identify IndexedDict operations (`entryAt`, `keyAt`,
`valueAt`, `indexOfKey`, `move`, `insert(at:)`, `removeAt`) but do not fully
define signatures, return shapes, invalid-position behavior, or the complete
MutableBag/OrderedSet selector sets. Do not invent those source-visible
contracts. Complete the shared internal key-engine extraction, then prepare a
focused architecture consultation and wait for the user-provided ruling before
implementing or publishing the new APIs.

## 6. Takeover map

| Concept | Live owner | P6 invariant |
|---|---|---|
| Ordered key/value slots and collision buckets | `phalcom-core/src/heap/map.rs::MapObject` | Keep slots and cached-hash buckets coherent |
| Map key lookup and mutation | `phalcom-core/src/primitive/map.rs` | No semantic change; no Heap borrow across callbacks |
| Set key lookup and mutation | `phalcom-core/src/primitive/set.rs` | Share the same key engine; preserve Set order/equality |
| Heap object variants/accessors | `phalcom-core/src/heap/{object,accessors}.rs` | Distinct object kinds may share backing, not semantics |
| Mutable-key rejection | `phalcom-core/src/primitive/mod.rs` | Extend the central classifier only after each new type's mutability/key law is fixed |
| Existing public Map/Set APIs | `phalcom-core/core/universe/src/collections/{map,set}.ph` | Preserve selectors, return behavior, encounter order, and equality |
| Core runtime regression tests | `phalcom-core/tests/core/collections/contract.rs`, `language/mod.rs`, language fixtures | Keep in the existing responsibility tree; add focused coverage with each settled type |

## 7. Architecture

P6 proceeds in ordered slices:

1. Establish the live Map/Set behavior baseline, retaining known
   `R-MAP-REMOVE` as a pre-existing failure.
2. Extract one crate-private key lookup engine that owns hash dispatch,
   collision candidate equality, the numeric SameValueZero path, callback
   error propagation, and per-send re-entry enter/exit for Map and Set.
3. Verify the extraction against Map/Set regression behavior before adding new
   object arms or public selectors.
4. After the API consultation, add the per-type storage and source/native
   surfaces. Reuse the key engine and purpose-fit storage structures; do not
   force all collection families into one oversized universal object.
5. Add storage invariant, callback/re-entry, GC, and source-level tests at the
   lowest appropriate test tier.

The shared key engine may route through a small `Map`/`Set` backing-kind enum
because heap accessors differ even though both arms currently wrap
`MapObject`. It must copy candidate `Value`s out before calling Telos code.

## 8. Ownership boundaries

### Owns

- Map/Set key-engine extraction and focused compatibility evidence.
- MutableBag, IndexedDict, and OrderedSet runtime/native/source implementations
  after their public API contracts are approved.
- Focused invariants for count totals, key/slot consistency, uniqueness/order,
  collision handling, GC tracing, and callback re-entry.
- P6 checkpoint/handoff/walkthrough updates.

### Does not own

- P5 parser String/Substring or optional-byte repairs and P9 G6.B certification.
- Existing `R-MAP-REMOVE` repair unless separately authorized by its owning
  plan/decision; it remains a known baseline.
- Persistent `Bag`/`Bijection` (P7), Interval/RangeMap (P8), `Entry` migration,
  final property/performance/release certification (P9).
- Changes to accepted Map/Set semantics or unapproved new public selector
  semantics.

## 9. Invariants

```text
INV-01 — Telos hash and equality remain the canonical key relation.
INV-02 — No Heap borrow crosses a re-entrant Telos hash/equality send.
INV-03 — Every re-entry enter is paired with exit on success and error paths.
INV-04 — Candidate collisions are resolved by Telos equality, not hash alone.
INV-05 — Numeric keys preserve current SameValueZero behavior.
INV-06 — Existing Map/Set order, equality, duplicate, admission, callback,
         and mutation-lock behavior does not regress.
INV-07 — IndexedDict ordered slots and key index agree after every mutation.
INV-08 — OrderedSet storage and uniqueness/index data agree after every mutation.
INV-09 — MutableBag has only positive exact Telos-Int counts and
         `total == sum(counts)`.
INV-10 — New collection heap storage is fully traced and no public key is
         admitted contrary to its stable-hash law.
```

## 10. Non-goals

- Repairing unrelated or baseline failures.
- Replacing Telos callback equality with a Rust trait or identity comparison.
- Changing Map/Set observable behavior to simplify reuse.
- Designing public APIs by implementer inference when the selectors/results
  are not specified.
- Reusing P7 persistent storage or P8 ordered interval storage here.
- Workspace, Clippy, or release certification.

## 11. Expected impact

### Initial engine extraction

- `phalcom-core/src/primitive/` — add the shared ordered-hash key engine;
  route Map and Set locate paths through it.
- `phalcom-core/src/primitive/map.rs` and `set.rs` — remove duplicated lookup
  semantics without changing collection-specific mutations.
- `phalcom-core/tests/core/collections/contract.rs` and existing
  `language/collections` fixture coverage — baseline and regression evidence.
- C1 checkpoint, P6 plan, and eventually P6 walkthrough/handoff.

### Later type slices, after API ruling

- `phalcom-core/src/heap/`, `phalcom-core/src/primitive/`, native registration
  and Object tracing/accessors.
- `phalcom-core/core/universe/src/collections/`, native census, and focused
  runtime tests.

Any need to modify VM GC policy, stable IDs, callback liveness, or semantic
authority is an immediate escalation.

## 12. Decision authority

### FIXED

- Telos `hash`/`==` semantics, callback re-entry discipline, and numeric fast
  path remain centralized and unchanged for existing Map/Set.
- Map/Set order and equality semantics remain unchanged.
- The three new data models and invariants in §5 are fixed.
- Public collection methods must consume the one shared key engine.

### MECHANICALLY FLEXIBLE

- Private module/function names and whether the shared engine is in
  `primitive/ordered_hash.rs` or a semantically equivalent nearby module.
- A small private backing-kind adapter to route `Object::Map` and
  `Object::Set` accessors.
- Local helper decomposition, imports, and test harness plumbing.

### VERIFY-FIRST

- Exact fixture test filters and target registration.
- Current primitive registration and GC trace match sites for added heap arms.
- Which Map/Set regression assertions already have direct coverage.
- Exact baseline status of the known `R-MAP-REMOVE` failure on this revision.

## 13. Numbered tasks and gates

### T0 — Entry baseline

- Record branch/revision and clean starting tree.
- Run focused `core_collections` tests and one targeted Map/Set callback fixture
  discriminator; confirm each selector runs actual tests.
- Classify `R-MAP-REMOVE` separately and preserve its pre-P5 provenance.

**G0:** baseline and active worktree are recorded; no P6-owned regression is
present. Do not spend P6 effort fixing `R-MAP-REMOVE`.

### T1 — Shared key engine

- Move `hash`, candidate equality, numeric SameValueZero, and re-entry wrapping
  to one private engine used by Map and Set.
- Ensure send failures always release the re-entry depth.
- Keep entry lookup and candidate values owned before callbacks.
- Add only focused tests that expose a missing regression contract; do not
  mirror private implementation details.

**G1:** Map and Set share the same callback/key semantics; focused regressions
pass except any independently classified baseline failure; scoped diff has no
behavioral change.

### T2 — Public API ruling and amendment

- Use `COLL006.C1.P6-api-contract-consultation.md` for the unresolved selectors,
  signatures, result/error shapes, invalid positional behavior, constructor
  duplicate behavior, and multiplicity policy.
- Wait for a user-supplied architecture ruling; record any decision in the
  checkpoint and amend this plan before new public source surfaces are added.

**G2:** contracts are explicit enough to implement without inventing observable
behavior. No code for new public APIs before this gate.

### T3 — MutableBag

- Implement the consulted public contract using exact Telos Int counts.
- Test positive counts, total conservation, count overflow/error policy,
  deletion at zero, hash collision, callback failures/re-entry, and GC.

**G3:** every public mutation preserves `total == sum(counts)` and no zero or
negative entries exist.

### T4 — IndexedDict

- Implement ordered entry slots plus a key-to-slot hash index.
- Keep key lookup distinct from positional `entryAt`/`keyAt`/`valueAt`.
- Repair indexes coherently for explicit positional insert/remove/move.
- Test replacement ordering, remove/reinsert ordering, collisions, re-entry,
  and order-sensitive equality.

**G4:** key and positional views agree after each tested mutation.

### T5 — OrderedSet

- Implement ordered unique storage plus hash index and the consulted positional
  operations.
- Test duplicate-constructor policy, membership/index agreement, order-sensitive
  equality, callback re-entry, and GC.

**G5:** no duplicates or index/order divergence; order participates in `==`.

### T6 — Mutable key admission and cross-family closure

- Extend the central mutable/content-unstable key classifier to P6 types only
  where their approved mutability/hash semantics require rejection.
- Do not classify by names in one consumer or add duplicate guards.

**G6:** classifier tests cover all affected Map/Set/new collection key entry
points and preserve Vector key safety.

### T7 — Stabilize and close P6

- Run Map/Set regression, P6 collection, affected native-surface/census, GC,
  and focused formatting checks named by the final P6 plan state.
- Record unrelated/baseline failures and leave broad certification to P9.
- Update checkpoint plan ledger, invariants, evidence, residuals, active plan,
  and next action; create P6 walkthrough and handoff.

**G7:** P6-owned implementation is `IMPLEMENTED`; verification is at most
`FOCUSED_TESTED` unless an independent baseline blocker is truthfully recorded.
P9 remains the next closure/certification plan.

## 14. Verification budget

Run commands serially because the local target directories may have been
cleaned and builds can be slow.

T0/T1 candidates (confirm exact tests and filters first):

```sh
RUSTFLAGS='' cargo test -p phalcom-core --test core core_collections
RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus map_set_custom_hash_eq_no_mutation
```

For each new collection, add its named core test module and focused fixture
filter to the plan after the approved API is implemented. Do not run package,
workspace, Clippy, or release gates during BUILD; P9 owns broad certification.

## 15. Escalation triggers

Stop and seek architecture guidance if any of these occurs:

- the shared engine requires a new ownership model, stable identity, or GC
  policy;
- Map/Set compatibility tests require changing an accepted contract;
- no-Heap-borrow-across-callback cannot be maintained;
- exact Telos Int multiplicity cannot preserve overflow/total invariants;
- key and ordered-index coherence requires a cross-cutting representation
  change beyond this plan;
- approved language behavior conflicts with `docs/specs/` or checkpoint
  requirements;
- new public selector/result/error/ordering semantics remain undecided at T2.

## 16. Completion records

At plan completion, update the checkpoint's P6 ledger and active-plan state,
retain P5 G6.B/aggregate G6 as the P9 parse-package residual, then create:

```text
COLL006.C1.P6-walkthrough.md
COLL006.C1.P6-handoff.md
```
