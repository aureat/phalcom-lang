---
id: COLL006.C1.P6
kind: architect-implementation-plan-and-repository-audit
program: COLL006
checkpoint: COLL006.C1
scope: P6 remaining work (T2–T7) plus T1 preservation and predecessor baseline
repository: aureat/phalcom-lang
branch: feat/telos-collection-upgrade
reviewed_remote_revision: ffa2ab96578acf2f176aa2ddbd31e655590c7d29
prepared: 2026-10-11
status: ADOPTED_IMPLEMENTATION_PLAN
implementation_state: IN_PROGRESS_PARTIAL
verified_evidence: REMOTE_SOURCE_AND_RECORDED_TEST_RESULTS
execution_gate: CLEARED_TDR0092_ACCEPTED
---

# COLL006.C1.P6 — Ordered Mutable Hash Family
## Repository-grounded detailed implementation plan and architect handoff

> **Execution status (2026-10-11).** The user confirmed this repository-grounded plan as the P6 implementation plan to execute, and supplied the architect's T2 ruling approving its §3 contracts. The user separately resolved equality in §4.6 in favor of rooted start-of-call snapshots. G2 source oracles were observed failing before implementation. The architect approved exactly 29 bindings for a P6-specific TDR-0017 amendment and resolved the census disposition; the user accepted it as written, and it is now [TDR-0092](../../../decisions/accepted/0092-ordered-mutable-hash-collection-native-floor.md). The current T3–T5 implementation is partial and uncommitted. This document is the adopted detailed execution guide; `COLL006.C1.P6-detailed-implementation-plan.md` remains the numbered plan-of-record, while the checkpoint remains the sole durable status authority.

## 0. Executive contract and evidence discipline

### 0.1 Result and immediate next action

- **Verified remote feature-branch HEAD:** [`ffa2ab96578acf2f176aa2ddbd31e655590c7d29`](https://github.com/aureat/phalcom-lang/commit/ffa2ab96578acf2f176aa2ddbd31e655590c7d29), the follow-up commit `fix(COLL006): close P5 audit follow-up`. This is **one commit after** the planning request's starting revision, [`80a889f078d1f8f1656a514baff0996a176b6311`](https://github.com/aureat/phalcom-lang/commit/80a889f078d1f8f1656a514baff0996a176b6311).
- **P6:** `IN_PROGRESS / PARTIAL / FOCUSED_TESTED`. T0–T2/G2 are complete and TDR-0092 authorizes the exact native inventory. MutableBag has one focused source fixture pass; IndexedDict and OrderedSet implementation drafts are incomplete. Continue T3–T5 within the accepted 29 bindings; T6–T7 remain.
- **P5 audit correction is now committed.** The checkpoint records `core_collections`: **13 passed, 0 failed**. The old `R-MAP-REMOVE` failure was a stale test oracle; the corrected public assertion is now committed, and the earlier failure remains historical evidence. `modules_parse_package`: **4 selected, 3 passed, 1 failed**; 11/12 runtime fixtures pass. Its remaining failure is an **associated-family invocation lowering limitation**, not the repaired String/Substring or Option<Int> adapter mismatch. The 2026-10-11 checkpoint states this explicitly.
- **Execution boundary:** G2 source oracles are complete and TDR-0092 is accepted. Finalize the exact 29 admitted bindings, remove the six excluded draft bindings, and implement their source derivations under the ruling's callback/suspension constraints. Do not add other native selectors or work on P7, P8, broad P9 certification, or the associated-family compiler issue under P6 without a separate ownership decision.

### 0.2 Observed versus inferred

| Label | Meaning | Implementation permission |
|---|---|---|
| **O** | Observed directly in remote code at the reviewed branch | Can be treated as repository fact, verify live checkout before editing |
| **R** | A result recorded in the repository checkpoint or walkthrough | Historical verification evidence, not a test executed by this author |
| **F** | Fixed by accepted TDR/C1 architecture | Preserve; contradiction triggers consultation |
| **P** | Candidate contract marker retained from the consultation draft; adopted by the recorded T2 ruling unless an explicit exception is stated | Implement as specified; the G2 source-oracle requirement is satisfied |
| **M** | Mechanical implementation choice | Implementer may adapt if invariants and tests are preserved |
| **U** | Unverified in local worktree/runtime | Investigate and record before relying on it |

This review used GitHub connector reads of the remote branch and did **not** run Cargo tests or inspect the user's uncommitted filesystem worktree. Every runtime pass listed below is attributed to the repository's own checkpoint, not newly measured. File line numbers are anchored to the reviewed remote revision; local edits may move lines. The branch had diverged from `main` at inspection, so **"pre-P6" is not synonymous with "confirmed on contemporaneous main"**.

### 0.3 Governing records (read and preserve)

1. [`AGENTS.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/AGENTS.md) — status/diff preservation, narrow scope, semantic authority and stop conditions.
2. [`docs/specs/README.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/specs/README.md) — current normative specification hierarchy; historical `docs/spec/current` links do not supersede it.
3. [`TDR-0028`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/decisions/accepted/0028-collections-representation-and-literals.md), [`TDR-0033`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/decisions/accepted/0033-amend-floor-admit-collection-container-primitives.md) — native collection heap arms, minimal raw primitives and source-defined higher-level operations; TDR-0028's old bracket rule is amended by TDR-0091.
4. [`COLL006.C1-master-implementation-record.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/implementation/COLL006/COLL006.C1/COLL006.C1-master-implementation-record.md) §§3.6–3.8, 12, 16–18, 21, 23 — intended collection ontology, complexity, GC and equality.
5. [`COLL006.C1-detailed-implementation-plan.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/implementation/COLL006/COLL006.C1/COLL006.C1-detailed-implementation-plan.md) §13 — global P6 requirements, anti-regression firewall and design priorities.
6. [`COLL006.C1.P6-detailed-implementation-plan.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/implementation/COLL006/COLL006.C1/COLL006.C1.P6-detailed-implementation-plan.md) — **current implementation plan of record**; retain T0–T7 IDs and G0–G7 gates.
7. [`COLL006.C1.P6-api-contract-consultation.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/implementation/COLL006/COLL006.C1/COLL006.C1.P6-api-contract-consultation.md) — the unresolved API decision packet.
8. [`COLL006.C1-CHECKPOINT.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md) — **single durable checkpoint execution authority**, notably current P6 T0/T1 and 2026-10-11 P5 audit follow-up.
9. [`COLL006.C1.P5-handoff.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/implementation/COLL006/COLL006.C1/COLL006.C1.P5-handoff.md) and P5 associated-family invocation consultation — inherited integration boundary.
10. [`docs/workflow/implementation-record-lifecycle-convention.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/workflow/implementation-record-lifecycle-convention.md) and [`docs/workflow/luna-patch-grade-plan-schema.md`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/docs/workflow/luna-patch-grade-plan-schema.md) — artifact and gate conventions.

## 1. On-the-ground implementation audit

### 1.1 T1 shared ordered-hash lookup (O, R)

[`phalcom-core/src/primitive/ordered_hash.rs`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/src/primitive/ordered_hash.rs) defines:

```rust
pub(super) enum KeyedCollectionKind { Map, Set }

pub(super) fn locate_key(
    vm: &mut VM,
    id: ObjRef,
    key: Value,
    kind: KeyedCollectionKind,
) -> PhResult<(i64, Option<usize>)>
```

`KeyedCollectionKind::{Map,Set}` dispatches `enter_reentrant_send`, `exit_reentrant_send`, `candidate_slots`, and `candidate_key` to distinct heap accessors. `locate_key` calls the shared `send_hash`, copies bucket indices (`to_vec()`), copies candidate `Value`s before callback, preserves current numeric SameValueZero, and otherwise calls Telos `send_eq`. `with_reentrant_send` exits on both ordinary success and `PhResult::Err`. Existing `primitive/map.rs::locate_key` and `primitive/set.rs::locate` delegate to this engine.

**Caveats/requirements:**

- T1 shows no heap borrow spanning callbacks in this particular lookup path; extending to new operations must repeat that proof. A Rust local `Value` is **not automatically a proven GC root**; establish callback liveness by observing VM rooting and through forced-GC tests.
- The guard is an explicit enter/call/exit sequence, **not** an unwind-proof Rust RAII guard. T1's recorded tests cover ordinary propagated errors; do not claim panic-unwind protection. No general panic/unwind redesign is authorized in P6.
- Existing `send_hash` in `primitive/mod.rs` expects a small/inlined `Int` return via `Value::as_int()` and yields `InvalidHash` otherwise; it does **not** accept arbitrary large heap-backed Int hash results. This existing contract is not to be silently widened to support P6. Distinguish exact **multiplicity** integers from hash-bucket integers.
- The numeric SameValueZero rule is applied when the incoming or candidate key is numeric. Preserve the actual existing branch and its asymmetric user-dispatch avoidance; do not 'simplify' the condition using Rust `Eq` or default value equality.
- The central native mutable-key classifier is in `primitive/mod.rs`, rather than `ordered_hash.rs`; one semantic authority does **not** require moving its source location.

Checkpoint result (R): `shared_hash_engine_preserves_collisions_and_reentry` passed and no new Map/Set failure was observed after T1. The subsequent P5 audit follow-up records `core_collections` **13/13 passed** after fixing its public test oracle.

### 1.2 Physical Map/Set backing (O)

[`phalcom-core/src/heap/map.rs`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/src/heap/map.rs) contains:

```rust
type Entry = (Value, Value, i64);
struct MapObject {
    entries: Vec<Entry>,
    index: HashMap<i64, Vec<usize>>,
    reentrant_depth: u32,
}
```

The entry vector defines insertion order. Buckets map cached Telos hash values to live slots; collisions are disambiguated with Telos equality. `MapObject::remove_at` does **order-preserving** `Vec::remove` and repairs shifted bucket positions from cached hashes without another callback. `set_value_at` permits value overwrites while the structural lock is active; `insert_new` and `remove_at` reject a locked receiver. `Object::Map` and `Object::Set` have separate heap variants and classes but share `MapObject`; Set's associated-value field is always Nil and not semantically exposed.

**Design result:** retain Map/Set backing and behavior unchanged. The new classes may share **lookup machinery and purely structural bucket/index helpers**, not necessarily the same physical payload. `MutableBag` needs exact counts, `IndexedDict` needs explicit movement/insert positions, and `OrderedSet` can avoid a permanently unused map-value slot.

### 1.3 Existing source contracts (O)

[`collections/map.ph`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/core/universe/src/collections/map.ph): `get` returns safe lookup, strict `[_]` raises `KeyError`, `insert(value, for: key)` exposes previous association, `at(key, put: value)` and **public** `remove(key)` return `self`; replacing a value does not reorder a key; Map equality compares associations irrespective of order. Its `MapIterator` yields the labeled structural tuple `(key: K, value: V)` and accesses live source positions. The file also contains a local `Entry` class: **P9 owns the eventual general Entry migration; do not silently make P6 depend on it.**

[`collections/set.ph`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/core/universe/src/collections/set.ph): `add` and `remove` return `self`, duplicates and absent removal are idempotent; `at` is a positional observation of encounter order; Set equality is **order-insensitive**; iterator yields `T`. Neither Map nor Set should acquire IndexedDict/OrderedSet equality as a side effect.

[`collections/list.ph`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/core/universe/src/collections/list.ph): positional insertion/removal already normalize negative indices; `insert(value, at: index)` and `remove(at: index)` return `Result<..., Error>`. This is a strong compatibility **precedent**, not automatic authorization for the new public APIs.

[`collections/iterator_laws.ph`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/core/universe/src/collections/iterator_laws.ph): canonical `Iterator` owns `type Item` and `next -> Option<Self::Item>`; `Iterable` owns `type Item`, `type Iterator`, and `iterator`. `toMap` shows source-level typed entry processing and `DuplicateKeyError.new(key)`; `toSet` shows repeated iteration via `step.match`. New factories should be tested *early* against the outstanding associated-family invocation lowering residual; do not work around an unsupported compiler path by weakening static contracts.

### 1.4 Native metadata and module-loading census (O)

| Concrete owner | Verified current Map/Set precedent | P6 work |
|---|---|---|
| `phalcom-native-meta/src/universe.rs` | `UniverseKey::{Map,Set}`, class relations, path/name/from-name matches, `UNIVERSE_BINDINGS` and `UNIVERSE_TYPE_FORMS` (`Map<K,V>`, `Set<T>`) | Three canonical keys, paths, class relations, type-form binders and bindings; exhaustiveness/tests |
| `phalcom-semantic/src/core_surface/identity.rs` | `CoreDeclarationIds::{map,set}` constructed via `universe_declaration(UniverseKey::...)` | Add three canonical identity fields; no synthetic IDs or duplicate declaration authority |
| `phalcom-core/src/universe/core_classes.rs` | `make_core_class(...,"Map"/"Set",collection_class,...)`; class fields; resolve mapping; bootstrap tuple/destructuring | Add three primordial class slots, allocation and key resolution; audit all tuple/destructuring/build sites |
| `phalcom-core/src/universe/primitives.rs` | explicit imports and `primitive_static!`/`primitive_internal!` registrations for Map/Set, followed by native-surface validation | Add explicit registrations for minimal raw methods and native imports, with `NATIVE_SURFACES` parity |
| `phalcom-modules/src/builtin.rs` | `children` lists `map`,`set`; separate path-to-`include_str!` arms around lines 515 and 1041 | Add each module in **both** catalogs, not just a package export |
| `phalcom-core/core/universe/src/collections/package.ph` | explicit `expose .map`, `expose .set` | Add `.mutable_bag`, `.indexed_dict`, `.ordered_set` |
| `phalcom-core/src/typing/inspect.rs` | `class_constructor_arity` reads `UNIVERSE_TYPE_FORMS` | Reflection generic arity should follow metadata; no class-name switches |

`UNIVERSE_TYPE_FORMS` supports ordinary kind-`Type` parameters for these families; unlike `Vector<T,N>`, P6 needs no new heterogeneous static-domain binder.

### 1.5 Heap, Value, rendering and runtime exhaustiveness (O)

| Concrete owner | Verified role | Required P6 audit |
|---|---|---|
| `phalcom-core/src/heap/mod.rs` | `alloc_map`/`alloc_set` return distinct `Object` variants | Per-family allocators and re-exported storage types |
| `phalcom-core/src/heap/object.rs` | boxed `Object::Map(Box<MapObject>)`/`Object::Set(...)` | Box three new native arms; enforce common arena object-slot size budget |
| `phalcom-core/src/heap/accessors.rs` | `map`, `map_mut`, `as_map`, `set`, `set_mut`, `as_set` | Typed accessors; no downcast through unrelated arm |
| `phalcom-core/src/heap/trace.rs` | traces each Map/Set stored key/value | Per-type exact edge tracing, iterator/view ownership and callback temp rooting |
| `phalcom-core/src/value/mod.rs` | `Value::class` maps `Object::Map`/`Set` to classes; other classification paths match them | Add new class mappings and exhaustive `Object` handling without widening Value itself |
| `phalcom-core/src/value/render.rs` | raw/debug Map/Set render branches | Deterministic debug/raw forms; no accidental unbounded bag expansion; cycle handling |
| `phalcom-core/src/vm/dispatch.rs` | special Map/Set bytecode literal and `**`-expansion handling | New types must **not** be inserted into Map/Set intrinsic literal paths merely because they are keyed |
| `phalcom-repl/src/snapshot.rs` | explicit primordial root class list includes Map | Audit explicit roots/list where relevant; add only when needed by actual ownership semantics |
| `phalcom-core/src/error.rs` | `MapMutationError`, `RuntimeError::ConcurrentMutation`, `InvalidHash`, `NumericLimit` | Reuse errors; widen `ConcurrentMutation` display safely for new families without changing Map/Set messages |

`Value` is the established compact tagged representation; do **not** create `Value::MutableBag` or enlarge `Value` for P6. `Object` variant size is constrained by the arena; box large families and measure the common slot.

### 1.6 Exact integers and error constructors (O)

- [`phalcom-core/src/primitive/int.rs`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/src/primitive/int.rs) exports crate-private `expect_int_big(&Value, &mut VM) -> PhResult<BigInt>` for small and heap-backed integers.
- [`phalcom-core/src/value/mod.rs`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/src/value/mod.rs) has `normalize_bigint(BigInt, &mut Heap) -> Value`; small values stay immediate, large values allocate `Object::LargeInt`.
- [`phalcom-core/src/primitive/number.rs`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/src/primitive/number.rs) performs `BigInt` operations under a numeric bit-length policy; `NumericPolicy::standard()` currently uses `max_integer_bits: Some(8_388_608)` and other allocation/text guards. Any P6 count arithmetic must apply the **same configured VM policy**, not a new hard-coded number or host-width ceiling. Determine an existing reusable checking helper; if it is private, extract a **general numeric** helper with tests instead of duplicating policy logic.
- `phalcom-core/core/universe/src/errors/argument.ph` currently declares `ArgumentError`, `KeyError`, `IndexError`, `SliceError`, and `DuplicateKeyError`. `DuplicateKeyError.new(key)` stores the offending key. No verified native `DuplicateElementError`/`MultiplicityError` class exists: proposing one requires a decision.

### 1.7 Latest baseline — changes since the attached planning request (O, R)

The original prompt named `80a889f0`. The reviewed head `ffa2ab96` adds P5 follow-up modifications: `packages/parse/src/source/text-source.ph`, `packages/parse/src/text.ph`, the operators runtime fixture, P5 lifecycle records, a new associated-family lowering consultation, and the Map public-contract test. The checkpoint's latest P5 section records:

```text
core_collections:        13 passed, 0 failed
modules_parse_package:   4 selected, 3 passed, 1 failed
parse runtime fixtures:  11 of 12 passed
P5 G6.A:                PASS
P5 G6.B/aggregate:      BASELINE_BLOCKED
```

Its remaining failing path is token-backed `Cursor.current` / `advance.current` in `packages/parse/tests/runtime/src/core/source-test.ph`; the compiler fallback resides in `phalcom-core/src/compiler/lib/associated.rs`. **Not a P6 defect.** `Range.toList` and the underconstrained `Map.new()` language-corpus fixtures remain separately recorded as pre-T1 observations without contemporaneous-main comparison. The P4 D1/D2/D4 re-certification remains outstanding; do not reclassify it.

## 2. Fixed semantic invariants

The following are **not available to the implementer for unilateral redesign**.

| ID | Invariant | Ownership |
|---|---|---|
| INV-KEY-01 | Telos `hash` and `==` define key lookup; Rust `Value` identity or Rust `Hash` never replaces them | Shared ordered-hash engine |
| INV-KEY-02 | Same-hash collisions compare candidate keys via Telos equality | Shared ordered-hash engine |
| INV-KEY-03 | Current numeric SameValueZero path is retained | Shared ordered-hash engine |
| INV-CB-01 | No `Heap` or heap-object borrow remains live across Telos callback invocation | Every primitive and source/equality path |
| INV-CB-02 | Per-receiver callback depth enters/exits on ordinary success and error; reentrant reads remain valid; structural mutation during its own key inspection is rejected | Ordered-hash adapter/storage |
| INV-CB-03 | Callback `Err` propagates unchanged; no malformed bucket or count state survives | Every consumer |
| INV-MAP-01 | Map insertion order, public selector results, key overwrite ordering and association-based equality unchanged | Existing Map |
| INV-SET-01 | Set encounter order, idempotent add/remove and order-insensitive membership equality unchanged | Existing Set |
| INV-BAG-01 | Only positive exact Telos-Int multiplicities stored; `total == Σcounts`; multiplicity equality is order-independent | MutableBag |
| INV-IDX-01 | Every unique IndexedDict key has exactly one ordered slot; key lookup agrees with positional view; value replacement does not move key; removal/reinsertion appends unless explicit position | IndexedDict |
| INV-ORD-01 | OrderedSet members unique; order is observable/equality-sensitive; explicit position mutation; canonical name `OrderedSet` not `SetList` | OrderedSet |
| INV-GC-01 | Every reachable stored Telos `Value` and callback-visible temporary has a valid GC reachability story | Heap/tracer/native temporary roots |
| INV-KEY-04 | Mutable/content-unstable native collections cannot enter any new or existing keyed collection as keys/members | Central classifier and public entry paths |
| INV-ID-01 | Each public collection has canonical Universe declaration, class, native object identity and applied type, without a parallel metadata system | Native meta / semantic / bootstrap |
| INV-TEST-01 | Approval-derived tests express the public contract; do not derive expected results from the first implementation | Tests |

**Architecture boundary:** TDR-0033 ratifies minimal native raw operations for the original Map/Set/Tuple/Range floor. New P6 primitives must follow the same pattern (native storage/atomicity below; higher-level combinators in `.ph`), with their new primitive surface recorded and checked. Avoid an oversized general `OrderedHashObject` that obscures distinct slot/count/order invariants.

## 3. Formal public-API gate (T2/G2)

### 3.1 Adopted decision state

The architect ruling transmitted by the user approves the following tables as the P6 public contract. The user confirmed this plan as the implementation plan. The only residual decision, equality traversal under callback mutation, is resolved in §4.6 and the normative spec. The `P` labels below are retained as provenance from the consultation draft; they no longer mean unresolved proposals.

All listed selector, result, duplicate, index, error, iteration, key-admission, callback, equality and arithmetic choices are settled by the ruling and normative spec. The `from<I>` associated-type path remains a verify-first implementation probe; a compiler limitation is not permission to weaken the public contract.

### 3.2 Approved MutableBag contract

Notation: source selector spellings are consistent with existing `.ph` style; `Self == MutableBag<T>`, `I |= Iterable` with `<I as Iterable>::Item == T` where expressible. `from<I>` MUST pass an early associated-type compilation probe; otherwise stop/escalate.

| Selector / contract | Success/result | Absent/duplicate/invalid | Callback/error |
|---|---|---|---|
| class `new() -> Self` | Empty bag | N/A | Allocation policy |
| class `from<I>(_ source: I) -> Self` | Count each source element in encounter order | Repeated equal values increment | Iterable, hash, equality, numeric errors propagate, stop consuming at first failure |
| `size -> Int` | Exact **total occurrences** | Zero if empty | No key send |
| `totalCount -> Int` | Exact total occurrences | Zero if empty | No key send |
| `distinctCount -> Int` | Distinct stored keys | Zero if empty | No key send |
| `count(_ value: T) -> Int` | Exact multiplicity | Zero when absent | Telos hash/== errors propagate |
| `contains(_ value: T) -> Bool` | Multiplicity >0 | False when absent | Telos hash/== errors propagate |
| `add(_ value: T) -> Self` | Add 1 | Existing increments | Count limit / callbacks; receiver unchanged on failure |
| `add(_ value: T, count n: Int) -> Self` | Add nonnegative n | n=0 no-op; n<0 ArgumentError | Validate exact integer and numeric limit **before commit** |
| `remove(_ value: T) -> Self` | Remove up to 1 | Missing is no-op | Callback errors propagate |
| `remove(_ value: T, count n: Int) -> Self` | Remove up to n | n>available saturates to zero; n=0 no-op; n<0 ArgumentError | Validate before lookup/commit |
| `clear -> Unit` | Delete all keys/counts | Empty no-op | No user callbacks |
| `iterator -> MutableBagIterator<T>` | Lazy **expanded occurrences** in first-admission order | Count expansion must not allocate O(totalCount) | Live positional traversal; count snapshot on entry to each group |
| `entries -> MutableBagEntriesView<T>` | `(value: T, count: Int)` for each distinct member in first-admission order | Live positional view | No implicit rehash |
| `==(_ other: Self) -> Bool` | Equal multiplicity function regardless of order | Different type false | Telos key equality and lookup errors propagate |

**Exact arithmetic rule:** store only `count>0`; `sum(count) == totalCount`; use `BigInt` or canonical exact Telos Int, and enforce `vm.numeric_policy` before updating individual counts and totals. Do not cast counts to `usize`, `i64` or `f64`. Physical entry count may be `usize` but its public `Int` conversion must be checked/normalized.

### 3.3 Approved IndexedDict contract

Notation: `Self == IndexedDict<K,V>`; positional entries should use the **existing labeled tuple** `(key: K, value: V)`, not a new universal `Entry` class (P9 boundary). Class `from<I>` consumes that item type.

| Selector / contract | Success/result | Absent/duplicate/invalid | Callback/error |
|---|---|---|---|
| class `new() -> Self` | Empty | N/A | Allocation policy |
| class `from<I>(_ source: I) -> Self` | Strict unique-key ordered construction | First duplicate raises `DuplicateKeyError.new(key)`; stop consuming input | hash/==/iteration errors propagate |
| `size -> Int` | Number of entries | Zero if empty | No key send |
| `get(_ key: K) -> Option<V>` | `Some(value)` | `None` if absent | hash/== errors propagate |
| `[_ key: K] -> V` | Strict keyed lookup, even if K=Int | Missing raises KeyError | hash/== errors propagate |
| `contains(_ key: K) -> Bool` | True for admitted key | False if absent | hash/== errors propagate |
| `insert(_ value: V, for key: K) -> Option<V>` | Append new / overwrite old value at original position | New -> None; equal existing -> Some(previous); preserve stored key | hash/== errors propagate; no implicit reorder |
| `insert(_ entry: (key: K, value: V), at i: Int) -> Result<Unit, Error>` | Strict new entry at position | Equal key -> `DuplicateKeyError`; invalid i -> `IndexError` | Validation errors as Result; callback errors propagate as existing raised mechanism |
| `entryAt(_ i: Int) -> Option<(key: K, value: V)>` | Positional entry | None if outside | No hash |
| `keyAt(_ i: Int) -> Option<K>` | Positional key | None if outside | No hash |
| `valueAt(_ i: Int) -> Option<V>` | Positional value | None if outside | No hash |
| `indexOfKey(_ key: K) -> Option<Int>` | Current position | None if missing | hash/== errors propagate |
| `move(_ from: Int, to i: Int) -> Result<Unit, Error>` | Move to **final post-move index** | Same normalized index no-op; any out-of-range IndexError | No hash needed for position-only move |
| `removeAt(_ i: Int) -> Result<(key: K, value: V), Error>` | Remove and return entry | Invalid position IndexError | No hash needed; uses cached hashes |
| `remove(_ key: K) -> Self` | Remove association | Missing no-op | hash/== errors propagate |
| `clear -> Unit` | Empty | Empty no-op | No hash |
| `iterator -> IndexedDictIterator<K,V>` | Ordered labeled entries | Live positional traversal | Avoid borrow across equality callbacks |
| `==(_ other: Self) -> Bool` | Compare entry sequences by corresponding key and value | Position differs -> false | Telos equality failures propagate |

**Index normalization recommendation:** for `n` slots, normalize `i<0` as `n+i` in **exact integer arithmetic**; read/remove/move valid `[0,n)`, insert valid `[0,n]`, `move(..., to: n)` invalid for nonempty collection. Normalize destination against **pre-move** length, interpret as final post-move position. Examples: `[a,b,c,d].move(0,to:2) -> [b,c,a,d]`; `insert(...,at:size)` appends. Ensure n=0/move empty has no valid destination. No integer key may be interpreted as an index through `[_]`.

### 3.4 Approved OrderedSet contract

| Selector / contract | Success/result | Absent/duplicate/invalid | Callback/error |
|---|---|---|---|
| class `new() -> Self` | Empty | N/A | Allocation policy |
| class `from<I>(_ source: I) -> Self` | Construct in encounter order | **Strict:** first duplicate raises ArgumentError (or approved dedicated type) | Stop consuming input; propagate callback errors |
| `size -> Int` | Unique member count | Zero if empty | No key send |
| `contains(_ value: T) -> Bool` | Membership | False if missing | hash/== errors propagate |
| `add(_ value: T) -> Self` | Append if new | Existing equal value no-op, original representative/position preserved | hash/== errors propagate |
| `insert(_ value: T, at i: Int) -> Result<Unit, Error>` | Strict positional insert | Equal value elsewhere -> duplicate error; invalid i IndexError | Hash before commit, receiver atomicity |
| `at(_ i: Int) -> Option<T>` | Positional value | None outside range | No hash |
| `indexOf(_ value: T) -> Option<Int>` | Current position | None if missing | hash/== errors propagate |
| `replaceAt(_ i: Int, with value: T) -> Result<T, Error>` | Replace slot and return prior value | Existing equal at another slot -> error; equal at same slot -> no-op preserving representative; invalid i IndexError | Hash and dedup before commit |
| `move(_ from: Int, to i: Int) -> Result<Unit, Error>` | Move to final position | Same index no-op; invalid indices IndexError | Cached hash preserved; no hash callback |
| `remove(_ value: T) -> Self` | Remove matching member | Missing no-op | hash/== errors propagate |
| `removeAt(_ i: Int) -> Result<T, Error>` | Remove positional member | Invalid position IndexError | No hash |
| `clear -> Unit` | Empty | Empty no-op | No hash |
| `iterator -> OrderedSetIterator<T>` | Current order | Live positional traversal | No hash required |
| `==(_ other: Self) -> Bool` | Positionwise Telos equality | Reordered members are unequal | Telos equality errors propagate |

**Duplicate distinction:** `add` is idempotent; positional `insert` is strict; `replaceAt` never creates duplicates. IndexedDict and OrderedSet share positional normalization, not type identity or the same heap arm. `Set` remains membership/order-insensitive.

### 3.5 Approved cross-cutting rulings

| Item | Approved decision | Source-level effect |
|---|---|---|
| Inheritance and traits | `native class ... is Collection`, plus canonical `Iterable` conformance; generics invariant | Observable conformance and semantics |
| Constraints | Do **not** invent a P6-specific `Hashable` or `Comparable` trait bound; reuse Telos dynamic hash/== and mutable-key admission | No approved additional conformance rule in this implementation scope |
| Hashability | All three mutable collections are **not** admissible as keyed members; structural equality does not imply stable value hash | Required equality/hash law |
| Error policy | Reuse `KeyError`, `IndexError`, `ArgumentError`, `DuplicateKeyError`; positional validation via Result where List precedent applies; propagate thrown user callbacks unchanged | Catchability and result shape are public |
| Failure atomicity | Validation, hash/== and exact count arithmetic failures leave **receiver** unchanged; external callback side effects on other objects are not rolled back | Defines observable behavior |
| Re-entry | Preserve Map/Set allowance; new collections permit reads and reject structural mutations; bag count changes are semantic mutations and must reject during its own key inspection; IndexedDict value overwrite may follow Map | Callback behavior is observable |
| Iterator liveness | Live positional iterators; Bag snapshots a distinct entry's count when entering that group, then observes later positions live | Changes behavior under mutation |
| Display | Deterministic source-order forms; Bag renders `(value,count)`, not repeated value expansion | String output observable if specified |
| Constructor evaluation | Consume iterables in source order; fail fast and stop requesting later entries; pre-evaluated arguments are not retroactively undone | Evaluation effects observable |

**G2 pass predicate:** Approved contracts are in the normative spec, consultation, checkpoint, and plan of record; source-level oracles exist for all three public APIs and have been observed failing before production implementation. T2 architectural approval itself is complete.

## 4. Per-type storage and algorithm specification

### 4.1 Shared lookup authority and private adapter (F/M)

**Do not** fold all data models into a `MapObject` with flag-dependent behavior. Extend `primitive/ordered_hash.rs` by a narrow per-kind adapter over:

```text
collection_kind -> reentry_depth receiver
collection_kind -> candidate bucket indices (owned Vec<usize>)
collection_kind -> candidate key at slot (owned Value)
```

No calls to Telos user `hash` or `==` belong in backing data structs. Their private operations should accept **already-computed** buckets and structurally validated indices and must never send to the VM. Keep class-specific `map.rs`/`set.rs` mutations as-is; add `mutable_bag.rs`, `indexed_dict.rs`, `ordered_set.rs` alongside them. A private trait/adapter is acceptable **only** if the heap-borrow and callback boundaries remain auditable; a compact match on `KeyedCollectionKind` is also acceptable. Do not introduce a generic lifetime-bearing heap reference that survives a callback.

Required lookup state machine:

```text
is_key_admissible?           # admission for mutations; reads follow existing policy
  -> enter receiver guard
  -> Telos hash send
  -> exit guard (including Err)
  -> copy candidate slots
  -> for each candidate: copy owned key
       -> if numeric: existing SameValueZero
       -> else: enter guard -> Telos == -> exit guard
  -> return (cached bucket, Option<slot>)
```

The returned bucket is the *existing* `i64` hash bucket of Telos's runtime key engine. It must not be conflated with exact Int counts or slot extents. **Re-entry observations:** a read is allowed to recursively invoke lookup on the same receiver; guard depth must remain nonzero until outer callback exits. Structural mutation during the callback is rejected; when overriding an existing IndexedDict **value** follows Map's permitted overwrite precedent, ensure value-only overwrite cannot invalidate candidate slots. Re-examine whether equality/traversal over both receivers requires a new version/guard policy, and do not infer a public policy from the convenience of one implementation.

### 4.2 MutableBag representation (M under approved semantic invariants)

Recommended:

```rust
struct BagEntry {
    key: Value,
    count: BigInt,
    cached_hash: i64,
}
struct MutableBagObject {
    entries: Vec<BagEntry>,              // first-admission order
    buckets: HashMap<i64, Vec<usize>>,  // derived from cached_hash
    total_count: BigInt,                 // exact
    reentrant_depth: u32,
}
```

**Logical proof:** for all `i`, `entries[i].count > 0`; `total_count == Σ entries[i].count`; each live `i` belongs to exactly one `buckets[entries[i].cached_hash]`, all buckets contain only valid and unique slots, and no two keys compare equal under the approved key relation. First representative wins; last occurrence removal deletes the entry; re-insertion appends at the end for encounter iteration. Bag equality compares multiplicities without considering encounter order.

**Do not store multiplicities** in `f64`, `usize`, `i64`, a `MapObject`'s general `Value` field with implicit conversion, or a second shadow total of smaller width. `BigInt` is a reasonable Rust representation because existing Telos `Int` uses `BigInt` for large values; do not create a collection-private arbitrary precision number class. Convert public count to Telos `Int` by checked policy + `normalize_bigint` and ensure callback/GC allocation safety.

**`add(value,n)` proof sequence:**

1. Parse exact `Int` via `primitive/int.rs::expect_int_big`; reject negative, make zero a no-op only under approved T2 rule.
2. Check admitted key type; `locate_key` with no borrows across sends.
3. Copy old multiplicity (0 if absent); compute `new_count = old + n`, `new_total = total+n` in `BigInt` locals. Apply the same `vm.numeric_policy.max_integer_bits` and relevant numeric allocation checks as normal Int arithmetic. No writes yet.
4. Revalidate receiver lock/slot/version. Prepare any prospective new entry and bucket capacity, public return value, and GC roots. **Do not allocate an exposed return after committing a mutation if failure can still occur.**
5. Commit both count and total in one no-callback heap mutation; for a new key append/index exactly once. Afterward assert pure representation invariants.

**`remove(value,n)` proof sequence:** parse `n`, locate key, compute `removed=min(n,old)` and `new=old-removed`, new total, validate/prepare. If `new==0`, remove entire entry and repair shifts; else update count. Absent/over-removal semantics are T2-approved, not inventable by implementer. No stored zero, no negative total, no separate hash dispatch during index repair.

**`clear`** must reject structural mutation while own hash/== guard is nonzero, even if it invokes no key callback itself. Check the guard before clearing both entries and buckets and resetting total.

**Iteration:** avoid O(totalCount) expanded arrays. An expanded iterator stores owner, current distinct slot and exact remaining count. For mutable owner, pin/validate a documented liveness policy; if a callback can mutate the bag, copy the required key/count before sends and revalidate afterwards. An entry view yields labeled `(value: T,count: Int)` and must root the owner. Big counts may make full traversal impractical but not inexact.

### 4.3 IndexedDict representation (M)

```rust
struct IndexedEntry { key: Value, value: Value, cached_hash: i64 }
struct IndexedDictObject {
    entries: Vec<IndexedEntry>,
    buckets: HashMap<i64, Vec<usize>>,
    reentrant_depth: u32,
}
```

The entries array is the single truth for order and values. The hash index is **derived state** used for expected-O(1) key lookup. `at`/`entryAt`/`keyAt`/`valueAt` never infer meaning from an integer key. Maintain:

```text
∀ slot i: index[entries[i].cached_hash] contains exactly i
∀ bucket positions: 0 <= slot < entries.len
∀ distinct slots i,j: !key_equivalent(entries[i].key, entries[j].key)
∣buckets∣ covers all current slots without duplicates
```

**Pure positional index repair:** use cached hashes, never a callback. For first-generation correctness, a pure `build_index_from_cached_hashes(entries)` function is preferable to clever incremental bucket arithmetic. Rebuilding after an O(n) insertion/move/remove is still O(n) and preserves target asymptotics. To ensure failure-atomic receiver state, prepare the candidate reordered entry vector and derived index **before** swapping into the live object, or explicitly prove all allocation/fallible operations complete before in-place publication. One short `&mut Heap` commit may swap prevalidated structures; no VM user sends inside it.

**Key insert/overwrite:** admit key, locate existing slot, copy pre-overwrite value, prepare `Option<V>` return **before** commit if it allocates. Overwrite `.value` at the same slot; retain original stored key and hash, preserve order. New key appends and updates buckets. Same receiver reentrant value-only overwrite during a hash/== callback is currently permitted by Map's design: if adopted for IndexedDict, make outer/inner linearization explicit (the last commit wins) and protect the return-old-value observation; do not treat such an overwrite as structural index mutation.

**Insert at position:** normalize index exactly, detect duplicate by hash/==, prepare a new ordered slot vector with key placed at index, build new index from cached hashes, then commit. Duplicate at another position must not accidentally move or replace the key. A positional append at `size` is allowed under recommended contract; invalid indices and duplicate admission leave receiver unchanged.

**Move:** normalize source and final destination against pre-move length; `size` destination invalid. Rebuild purely from cached hashes. Test moving left/right, first/last, self/no-op and collisions. Moving must preserve *key and value object identity* as well as semantic association. A source-ordered `Vec::remove`/`insert` implementation is acceptable if final-index rules are mathematically demonstrated and index rebuild follows.

**RemoveAt/key remove:** copy return entry before removal; build shifted vector/index from cached hashes; commit. Removing then re-inserting a key via key-based insert appends; the prior representative is not implicitly restored. Invalid positional removal should not call hash/equality.

**Equality:** order-sensitive by corresponding `(key,value)` associations. **Numeric issue:** the existing key engine's numeric SameValueZero may differ from ordinary numeric `==` (notably NaN and signed zero); an order-sensitive keyed collection's key equality must use the approved key equivalence. Associated **values**, unlike keys, use ordinary Telos equality. All callbacks operate on the rooted start-of-call snapshots mandated by T2. No heap borrow crosses either callback.

### 4.4 OrderedSet representation (M)

```rust
struct OrderedSetEntry { value: Value, cached_hash: i64 }
struct OrderedSetObject {
    entries: Vec<OrderedSetEntry>,
    buckets: HashMap<i64, Vec<usize>>,
    reentrant_depth: u32,
}
```

Store no phantom map-associated value. Membership and `indexOf` use the common engine, returning the actual vector slot. Positional reads are direct. `add` deduplicates without changing the existing representative or position. Positional insertion rejects duplicates; `replaceAt` must search **before** mutation, reject an equal member at another index, and accept an equal member at its own index as a no-op under the proposed contract. When replacing with a new member, prepare the new cached hash and fully coherent prospective index before one atomic commit. `move` is *not* a key-admission operation: preserve cached hash and move the stored representative without sending hash/== again. Deletion compacts stably (never swap-remove) and repairs bucket slots.

Equality compares values positionwise under the collection's **key equivalence** including SameValueZero for numeric members; order matters. Verify collision-heavy and special-numeric cases independently. `Set` must remain order-insensitive.

### 4.5 Index utility design and performance (M)

A private pure helper may operate on a minimal `[(hash,slot)]` projection to rebuild `HashMap<i64,Vec<usize>>` without any `VM` argument. It may be shared across the new types (and Map/Set only if preservation proof justifies an isolated no-behavior-change improvement). It must never compute hashes or compare `Value`s; any helper that can call a Telos method is not pure and belongs at the semantic key-engine boundary.

Target complexity, **not** a license to weaken semantics:

| Operation | Expected complexity | Worst-case caveat |
|---|---:|---|
| Key/member lookup and additive admission | expected O(1) | Collisions can force O(n) Telos equality sends |
| Bag count lookup/update | expected O(1) + exact integer arithmetic | BigInt complexity depends on bit length |
| IndexedDict/OrderedSet positional read | O(1) after checked index conversion | Converting arbitrary exact Int must not truncate |
| `indexOfKey`/`indexOf` | expected O(1) | Same collision caveat |
| Ordered middle insert/remove/move | O(n) | Reindex from cached hashes; no callbacks |
| Bag expanded iteration | O(number of requested occurrences) | No O(totalCount) eager materialization |

Avoid a premature optimization with more mutable index copies or unsafe interior handles. Profile only after correctness gates. `HashMap` index is internal host infrastructure; Telos key semantics remain VM dispatch.

### 4.6 Callback, GC, and commit protocol

**Proof obligation for every operation that may run user code:**

```text
A. own the receiver ObjRef, incoming key/value, and any return Values
B. validate explicit inputs without mutation
C. establish callback-visible GC rooting (from VM frame/root or explicit guard)
D. copy candidate bucket/keys, drop all Heap borrows
E. enter receiver reentry-depth guard
F. perform Telos hash/== (or value equality)
G. exit guard even on ordinary PhResult error
H. re-access receiver and verify operation still legal
I. calculate exact counts / index edits / return structures / capacity
J. commit in one no-user-callback receiver mutation
K. check pure invariants and return prepared result
```

**Receiver atomicity (P):** When failure arises from invalid input, duplicate, `hash`/`==`, count overflow/policy, or preparatory operations, all receiver content and indices are unchanged. Side effects of user code on *other* objects are not rolled back. If user code is allowed to overwrite the current IndexedDict value during key lookup (Map precedent), distinguish *external reentrant commit* from the outer operation's receiver atomicity; define the linearization point explicitly at T2 rather than making impossible rollback promises.

**Copy-out / callback / revalidate** is mandatory. Never hold `&Heap`, `&mut Heap`, `&BagEntry`, bucket `&[usize]`, or borrowed `Value` slices across `VM::send_dynamic`. `Value: Copy` is not proof of GC rooting. Verify the runtime's native send frame rooting guarantees; add an explicit scoped root if needed. The collector's `trace_object` must visit all live `Value` keys/values and iterator/view owner ObjRefs, including while the object is being mutated or a return tuple/`Some` is allocated.

**Potential late allocation trap:** if `Result::Ok`, `Some(old_value)`, labeled tuple packaging, exception allocation, hash-map reserve or BigInt normalization can fail/trigger GC, prepare rooted results **before** publishing mutation or prove the operation cannot fail after the commit. Prefer a new candidate storage state constructed off to the side, then a no-callback swap. Avoid half-updated `entries`, `index` or `total_count`. If runtime OOM is process-fatal rather than catchable, document that narrower failure model without labeling all allocation universally infallible.

**Equality and mutation during traversal:** the user's T2 follow-up explicitly requires GC-rooted logical-content snapshots of both operands captured before the first equality callback. Callback mutations may commit to live collections, but the in-progress comparison reads only its snapshots. No epoch rejection or live traversal is permitted for equality.

### 4.7 Mutable-key admission and stable cached hashes

[`primitive/mod.rs::is_mutable_collection_key`](https://github.com/aureat/phalcom-lang/blob/ffa2ab96578acf2f176aa2ddbd31e655590c7d29/phalcom-core/src/primitive/mod.rs) already rejects native `List`, `Array`, `Buffer`, `Slice`, `Map`, `Set`, `Bytes`. Add the new `Object::{MutableBag,IndexedDict,OrderedSet}` in **that same authority**. Do not create separate Map-only, Set-only or collection-private forbidden-key lists. The rule applies on every **key admission** path, including new constructors and all existing Map/Set literal/primitive insertions. Existing lookup of previously admitted keys and error behavior must remain stable.

**Sequence crucial correction:** The original plan's T6 says to extend the classifier at cross-family closure. That would leave each newly published mutable type temporarily insertable into existing Map/Set. Instead, extend admission in the **same integration slice as each public type**, and let T6 become the full-matrix audit. Classifier tests must prove this on each type's first externally constructible gate.

Cached key hashes stay valid only under admitted stable-hash semantics. Do not attempt implicit rehash/repair of arbitrary mutable user keys during order changes; avoid assuming nominally immutable objects are stable unless their semantics support that law. General verification of custom user-defined key immutability is outside P6; preserve current runtime policy and flag stronger compiler support separately.

## 5. Dependency-ordered execution tasks, stop points and gate proofs

### T0 — Entry baseline refresh and ownership isolation (historically complete; refresh before editing)

**Files:** no source edit required. Review checkpoint/P5 handoff plus local status.

```bash
git branch --show-current
git rev-parse HEAD
git status --short
git diff --check
git diff -- docs/implementation/COLL006/COLL006.C1 \
  phalcom-core/tests/core/collections/contract.rs \
  packages/parse/src/source/text-source.ph packages/parse/src/text.ph
```

- Require branch `feat/telos-collection-upgrade`; match local revision against reviewed `ffa2ab96` or record newer head and refresh relevant facts. Do **not** reset/clean/restore unknown changes; only stage authorized files.
- Confirm actual test filters and selected case counts before relying on historical results. `core_collections` is now recorded green after the committed P5 correction; do not reopen `R-MAP-REMOVE` as an active runtime defect.
- Classify `Range.toList` and `Map.new()` language corpus failures independently; do not silently claim they reproduce on main. Determine whether the `Map.new()` test must provide `Map<Dynamic,Dynamic>.new()` to reach intended reentry, then rerun the fixture under its owner.
- Keep P5 G6.B (associated-family lowering), P4 D1/D2/D4 and P9 certification distinct.

**G0:** Worktree preservation verified, exact base revision recorded, inherited failures classified. No P6-owned regression or hidden extra changes.

### T1 — Preserve shared key-engine extraction (already implemented)

**Files:** `primitive/ordered_hash.rs`, `primitive/map.rs`, `primitive/set.rs`, `heap/map.rs`, existing `collections/contract.rs`.

Before adding new consumers, run the focused existing regression once if source changed since T1, and audit `Map`/`Set` raw/public return shapes. No new generalized storage refactor. In particular, don't change `set_value_at` lock behavior, SameValueZero, or `Map.remove(_)` to accommodate new types.

```bash
RUSTFLAGS='' cargo test -p phalcom-core --test core \
  core_collections::shared_hash_engine_preserves_collisions_and_reentry -- --nocapture
RUSTFLAGS='' cargo test -p phalcom-core --test core core_collections
```

**G1:** Shared engine preserved; actual test cases selected; no new P6-owned Map/Set failure. Historical T1 result is evidence only until current relevant edits are rerun.

### T2 — Authority/contract approval, test oracles and plan-of-record amendments (**API and 29-binding floor approvals complete; TDR-0092 accepted**)

**Owner:** architect/user decision. Implementer may prepare questions and tables; cannot approve them. Existing canonical `COLL006.C1.P6-api-contract-consultation.md` remains the packet.

1. The architect ruling received through the user approves §3; the user confirmed the repository-grounded plan as the implementation plan.
2. The user resolved equality traversal as rooted start-of-call snapshots. The IndexedDict nested-overwrite linearization follows the adopted plan and is recorded in the normative spec.
3. Approved contracts are recorded in `docs/specs/collection-families.md`, this consultation, the existing detailed plan, and the C1 checkpoint.
4. The source-level contract oracles were observed failing before implementation, completing G2. The architect approved exactly 29 bindings for amendment and resolved the catalog/audit disposition; the user accepted and recorded this as TDR-0092. Finalize only the approved surface and remove the six excluded draft bindings. Continue the approved `from<I>` compile probe without weakening the source contract.

**G2:** PASS — approved contracts and equality policy are durably recorded, and source-level oracles were observed failing before implementation. TDR-0092 separately authorizes the native floor.

### T3 — MutableBag: canonical type → exact counts → source API → GC (dependent on G2)

**T3.1 Identity and metadata (native class loadable).** Extend `UniverseKey`, class relations, from-name/name/path mappings, `UNIVERSE_BINDINGS`, one `UNIVERSE_TYPE_FORMS` binder (`T: Type`), `CoreDeclarationIds`, `core_classes.rs` class construction/resolve, source module catalog+`include_str!`, package expose, and explicit primitive registrations. Add `Object::MutableBag(Box<...>)`/allocators/accessors and value-class rendering. Audit enum/exhaustive matches. Add mutable-key rejection **now**, before publishing source creation. Exact first smoke source: `MutableBag<Int>.new()` has canonical class and correct generic arity and cannot be inserted as a Map key. Do not rely only on `cargo check`.

**T3.2 Storage and lookup.** New `heap/mutable_bag.rs` or equivalent; pure coherent `entries`/`buckets` and `BigInt` total. Extend `KeyedCollectionKind`; implement only approved native `rawCount`, `rawDistinctCount`, `rawAt`, and `rawCountAt` observations, plus atomic `rawAdd`/`rawRemove`, allocation, size, and clear. Derive public membership in source as `count(value) > 0`; do not register `rawContains`. Ensure no hash/equality clone and no borrow across VM callbacks.

**T3.3 Exact mutation.** Implement `add`, `remove` (single/bulk if approved) and `clear`; integrate `expect_int_big`, canonical bit/allocation policy and `normalize_bigint`. Check numeric bounds of *count and total* before commit. Ensure zero/negative and over-removal rules are explicit. Prebuild new bucket indices and rooted return values as needed. Test original object still usable after each rejected callback.

**T3.4 Iterable, view and equality.** Implement `MutableBagIterator<T>` (lazy expansion) and optional entry-count view per approved contract, canonical `Iterator`/`Iterable` conformance, order-insensitive multiplicity equality. Probe `from<I>` associated-type path; if it hits associated-family lowering, stop for owner consultation rather than adding unchecked dynamic API or spoofing associated type identity.

**T3.5 GC and semantics.** Trace keys plus iterator/view owner; force GC during calls; confirm removed keys can become collectable. Test immutability admission to all keyed families; custom collision keys; exact counts >2^53 and >i64 where allowed by numeric policy; numeric limit failure; deterministic mutation sequences.

**G3:** `positive stored multiplicities ∧ total == Σcounts ∧ exact counts ∧ unique keys ∧ all buckets coherent ∧ no mutable Bag key admission`; all selected T3-focused tests and typed source-compilation tests pass. No hidden `Dynamic` fallback or BigInt->float conversion.

### T4 — IndexedDict: distinct keyed/positional coordinates (dependent on G2 and shared engine)

**T4.1 Identity/native skeleton.** Add `UniverseKey::IndexedDict`, two `Type` binders (`K,V`), class metadata/bootstrapping, `Object::IndexedDict(Box<...>)`, heap accessors, correct `Value::class`/rendering/tracing, source export/loading, registrations, central mutable-key rejection. Smoke `IndexedDict<Int,String>.new()` and confirm `dict[3]` resolves *key* lookup, never slot lookup.

**T4.2 Hash-indexed key semantics.** Implement keyed `get`, `contains`, `insert(value,for:key)`, `remove(key)`, `indexOfKey`. Unchanged key overwrite position; return-old-value works even for a stored `None` as actual value (option nesting distinguishes missing key). Disallow mutable keys. Preserve user-defined `hash`/`==`, numeric SameValueZero and collision candidate sends.

**T4.3 Pure positional operations.** Implement checked exact-index conversion and approved negative normalization. `entryAt/keyAt/valueAt`, `insert(entry,at:i)`, `move(from,to:i)`, `removeAt(i)`. Require clear contracts for index `size`, `-1`, `-size`, `-size-1`, empty receiver, and duplicate keys. Use cached hashes and prepare full post-operation bucket map before commit. Confirm move right and move left produce final positional indices, not accidental pre-removal offsets.

**T4.4 Constructors, iteration, equality.** Strict source-order `from<I>` if approved; fail fast on duplicate while not consuming the next element, reuse labeled `(key:K,value:V)` without introducing P9 `Entry` changes. `IndexedDictIterator` and `Iterable` have explicit associated Item; order-sensitive structural `==`, including numeric key-equivalence and ordinary associated-value equality. Verify no implicit hash of stored keys during movement/iterator iteration.

**T4.5 Stress and GC.** Generate collisions and 100+ alternating moves/removals/inserts; check all slots and key-index lookup after every step; inspect failure atomicity under throws and reentry; trace both keys and values; check dropped values collectable after replacement.

**G4:** Key lookup and positional views are identical in meaning but not overloaded in syntax; all keys unique, all bucket/slot mappings valid after every mutation, exact order sensitive equality, callback/error/rooting proof passes, no Map/Set behavior drift.

### T5 — OrderedSet: unique ordered values and explicit movement (dependent on G2/shared engine)

**T5.1 Identity/native skeleton.** One `T: Type` binder, distinct native class/Object variant/source module/metadata, central mutable-key rejection, tracing, source export and native census.

**T5.2 Membership and positional API.** `contains`, idempotent `add`, `remove`, positional `at`, `indexOf`, strict positional `insert`, `replaceAt`, `move`, `removeAt`. All hash/equality via the shared engine; movement uses cached hashes. Reject duplicate replacement elsewhere and strict constructor duplicate; preserve original representative on an equal self-replacement/no-op if approved.

**T5.3 Equality, iteration and GC.** `Iterable<Item=T>`, ordered iteration and positionwise value equality consistent with key numeric equivalence. Trace members and iterator owner. Verify Set remains order-insensitive despite OrderedSet order sensitivity.

**T5.4 Sequence-model tests.** Compare against a slow unique-vector oracle after every operation. Cover `at(-1)`, prepend/append, move-to-self, move endpoints, invalid indices, hash collisions, callback throws, nested reads, forbidden reentrant structural mutation and GC.

**G5:** At most one equivalent element, bucket/index coherence, positionwise order-sensitive equality, no key-semantic regression, atomic unsuccessful insertion/replacement, full ownership tracing.

### T6 — Cross-family admission and integration audit (dependent on G3–G5)

**Do not defer first key admission enforcement here.** Each new family must already be rejected on first public construction. T6 reviews all consumers/paths for consistent classification:

```text
Map.rawPut / Map literals
Set.rawAdd / Set literals
MutableBag.add / MutableBag.from
IndexedDict.insert / IndexedDict.from / positional insert
OrderedSet.add / OrderedSet.from / positional insert / replacement
```

Matrix includes existing mutable families List/Array/Buffer/Slice/Map/Set/Bytes and new MutableBag/IndexedDict/OrderedSet as candidate keys or members; stable immutable Vector/Tuple/Record/etc retain existing conditional admissibility. Verify nested collections and collision/equal-key admission; do not infer permission from the static type name alone. No collection-private parallel trait or `Hashable` policy.

**G6:** Central policy enforced on every public admission path; no new mutable object is admitted to an existing Map/Set or any new keyed family; stable-immutable prior cases continue to pass. Failure messages retain existing Map/Set behavior.

### T7 — Focused stabilization and closeout (dependent on G3–G6)

1. Run affected core collection, source-language corpus discriminators, native registration/surface, metadata/class identity, module import, reflection, GC/rooting and relevant exact-integer tests **serially and selectively**.
2. Verify `Object` enum common slot size and `Value` ABI size; new arms must not widen the common arena slot beyond the established budget. Inspect new `Value::class`, equality and render match sites.
3. Verify all source APIs are exactly the approved signatures; no `Dynamic` was substituted for unsupported generic/associated-type contracts. Determine if a `from<I>` failure is the known associated-family lowering issue and escalate to its owning subsystem rather than hiding it.
4. Preserve P5 G6.B, P4 D1/D2/D4, `Range.toList` and negative generic fixture classifications as separate evidence. Report new P6 failures as new defects only when a reproducer and baseline comparison support that attribution.
5. Run scoped `rustfmt`, `git diff --check`, native surface generator `--check`, and focused compilation gates. Do not run broad workspace test, Clippy or release/perf certification as a substitute for P6 gates. P9 owns checkpoint release certification.
6. Update the **existing** P6 plan and **single** C1 checkpoint. Produce `COLL006.C1.P6-walkthrough.md` and `COLL006.C1.P6-handoff.md` with exact revision, commands, counts, scope, residuals, ownership and next plan. Do not create a second checkpoint-status document.

**G7:** Implemented public contracts match approved decisions, all P6-owned focused gates pass, no new Map/Set regression, metadata/native/GC coherence verified, unresolved external failures truthfully classified. Keep P6 verification `FOCUSED_TESTED` unless evidence justifies a different lifecycle status. **P9 is still required**.

## 6. Focused verification matrix, independent oracles and commands

### 6.1 Existing test target wiring (O)

`phalcom-core/tests/core/mod.rs` registers the existing collections contract suite with:

```rust
#[path = "collections/contract.rs"]
mod core_collections;
```

Other registered, adjacent suites include `core_array`, `core_buffer`, `core_slice`, `core_vector`, `memory_gc`, `modules_parity`, `modules_parse_package`, `native_surface_contracts`, `type_metadata_invariants`, `reflection`, and `modules_universe` (target module names; inspect exact paths in the test tree). `tests/core/collections/contract.rs` contains current `map_key_overwrite_and_remove_idempotence` and `shared_hash_engine_preserves_collisions_and_reentry`. The former's new assertions correctly demand `self` after public `Map.remove(_)`.

Recommended organization **after implementation** (filenames/test filter symbols do not yet exist):

```text
phalcom-core/tests/core/collections/
    mutable_bag.rs     -> mod core_mutable_bag
    indexed_dict.rs    -> mod core_indexed_dict
    ordered_set.rs     -> mod core_ordered_set
    ordered_hash.rs    -> mod core_ordered_hash    # only if distinct from contract.rs

phalcom-core/tests/core/mod.rs
    #[path = "collections/mutable_bag.rs"] mod core_mutable_bag;
    #[path = "collections/indexed_dict.rs"] mod core_indexed_dict;
    #[path = "collections/ordered_set.rs"] mod core_ordered_set;
```

Retain/extend `contract.rs` for shared Map/Set compatibility. Add source fixtures to the **existing** `phalcom-core/tests/fixtures/language/collections/` structure and verify language-corpus manifest registration and positive/negative lane discovery; do not add a fixture file that the test runner does not select. Use the existing `super::vm_support::run_inline(...)` helper for source-level code where appropriate. Tests of generic signatures must use the semantic/compiler production path, not direct `VM::send_dynamic` alone.

### 6.2 Command budget (existing vs prospective)

| Stage | Command | Status/requirement |
|---|---|---|
| T1 compatibility | `RUSTFLAGS='' cargo test -p phalcom-core --test core core_collections::shared_hash_engine_preserves_collisions_and_reentry -- --nocapture` | Existing filter; checkpoint records 1 passing in T1 |
| Map/Set preservation | `RUSTFLAGS='' cargo test -p phalcom-core --test core core_collections` | Existing filter; latest checkpoint records **13 passed** after P5 correction |
| Language corpus | `RUSTFLAGS='' cargo test -p phalcom-core --test language-corpus collections` | Existing filter but two pre-T1 failures; each must be independently classified |
| P5 diagnostic only | `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package` | Existing command; latest P5 result **3/4 passed**, outside P6 repair ownership |
| T3 bag | `RUSTFLAGS='' cargo test -p phalcom-core --test core core_mutable_bag` | **Prospective filter**; only run after registering tests and checking nonzero selection |
| T4 indexed | `RUSTFLAGS='' cargo test -p phalcom-core --test core core_indexed_dict` | **Prospective** |
| T5 ordered | `RUSTFLAGS='' cargo test -p phalcom-core --test core core_ordered_set` | **Prospective** |
| T6 shared | `RUSTFLAGS='' cargo test -p phalcom-core --test core core_ordered_hash` | **Prospective** or keep cases in `core_collections` |
| Native metadata | `cargo run -p phalcom-native-surface-gen -- --root . --check` | Existing generator; required after native declarations, update generated data through normal owning workflow |
| Core compilation | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-core --all-targets` | Focused cross-layer compile check at coherent gates |
| Semantic tests | `RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic coll006` | Existing semantic filter; run when canonical generic/semantic metadata affected, classify inherited failures |
| Diff check | `git diff --check` | Required for relevant scoped diff |
| Formatting | `rustfmt --check --edition 2024 <changed-rust-files>` | Use scoped files; global formatter has pre-existing drift |

Commands are **candidates until the implementer verifies actual CLI signatures and counts at the live revision**. Do not run workspace test/Clippy/release/performance certification simply because a type slice landed; no empty selection or merely compiling test target counts as execution proof. Run focused commands serially if shared target-directory contention or long builds affect reproducibility.

### 6.3 Mandatory tests by public behavior

| Contract axis | MutableBag | IndexedDict | OrderedSet |
|---|---|---|---|
| Generic class formation | `<Int>` and user class | `<Int,String>` and user key/value classes | `<Int>` and user class |
| Empty construction | zero total/distinct | zero entries | zero members |
| From iterable | duplicates accumulate, source order | duplicate key strict, source order | duplicate element strict, source order |
| Missing lookup | count=0, contains=false | get=None, strict subscript KeyError | contains=false, indexOf=None |
| Repeated admission | increment multiplicity | existing-key value overwrite preserving key slot | `add` duplicate no-op |
| Invalid new member | mutable-key rejection | mutable-key rejection | mutable-key rejection |
| Positional negative | not applicable | `-1`, `-n`, `-n-1` | `-1`, `-n`, `-n-1` |
| At upper bound | not applicable | `entryAt(size)=None`, `insert(at:size)` append | `at(size)=None`, `insert(at:size)` append |
| Invalid move | not applicable | empty/same/size/overshoot | empty/same/size/overshoot |
| Duplicate explicit insert | count increment (bag equivalent) | positional insert fails atomically | positional insert fails atomically |
| Duplicate replacement | not applicable | key-based overwrite valid; positional duplicate strict | replacement with equal at another slot fails |
| Order | first-admission iteration; ignored by == | semantic order; == sensitive | semantic order; == sensitive |
| Count exactness | >2^53, >i64 when allowed, numeric-policy bit cap | size checked from host length | size checked from host length |
| Collision | distinct equal-hash keys counted separately | distinct equal-hash associations | distinct equal-hash members |
| Numeric special keys | same existing SameValueZero path | membership/equality consistent with numeric key-equivalence | membership/equality consistent with numeric member-equivalence |
| User error | hash/== fail, state unchanged | hash/== fail, state unchanged | hash/== fail, state unchanged |
| Re-entry | read allowed; multiplicity mutation rejected | read allowed; structural reordering rejected; existing value overwrite per ruling | read allowed; add/remove/move/replace rejected where structural |
| GC | keys, temp count values, iterator/view owner | key, value, returned entry, iterator owner | members, returned prior value, iterator owner |
| Key admission | bag itself rejected across families | indexed dict itself rejected across families | ordered set itself rejected across families |
| Public/native parity | wrapper result equals approved contract | wrapper uses key/position vocabulary faithfully | wrapper result/order matches contract |

### 6.4 Independent reference models (do not share production helpers)

Model-based tests are mandatory for ordered mutations. Use intentionally simple **test-only** models which do **not** call the production index-rebuild helper or inspect its bucket state to derive expected results.

**Bag model:** `Vec<(TestKey,BigInt)>` with linear equality lookup; `total=Σcounts`. Apply random or seeded operations `add`, `remove`, `clear`, `count`; compare counts for every known key, total, distinct, iteration expansion prefix and association of first representative after each step. Model expected exceptions separately.

**IndexedDict model:** `Vec<(TestKey,Value)>` with linear key comparison, pure `Vec` insert/remove/move. Apply sequences of keyed upserts, strict positional inserts, moves, keyed/positional removes, misses, repeated replacements; assert `get`, `indexOfKey`, `entryAt`, iteration and final order **after every operation**, not only at end. Example:

```text
put A=1; put B=2; put C=3; move A -> final 2;
replace B=4; remove C; insert D at 0;
remove A; reinsert A; attempt duplicate B at 1;
expected final vector independently computed
```

**OrderedSet model:** slow unique `Vec<TestKey>`. Apply append/strict insert/replace/move/remove, assert membership, `indexOf`, positional reads, and equality under reorder after each step. Example:

```text
add A; add B; add C; move C->0;
reject duplicate B at 1; replace A with D;
remove B; reinsert B; move B to 0;
```

For callbacks, use deliberately collision-prone user-defined keys (`hash { 0 }`) with `==` dispatch counters, keys that throw in `hash` and `==`, and keys that read or attempt to mutate the target receiver in their callbacks. Test both expected semantic result and post-error receiver state. Do not rely only on happy-path primitive calls; compile `.ph` source to confirm selector labels, overload resolution, receiver side and associated types.

### 6.5 Forced-GC and rooting acceptance

For **each** new family:

1. Construct with heap-allocated, structurally comparable custom keys; hold the collection root only.
2. Trigger GC while a key's hash/equality callback allocates; check the incoming key, candidate keys, stored values and receiver remain reachable.
3. Create iterator/view and release direct collection root; verify iterator/view retains live storage as specified.
4. Remove or overwrite a contained value, drop other references, force GC; confirm the removed object is no longer kept alive by an obsolete collection edge.
5. Trigger an error during `Some`, labeled tuple, `Result`, or large Int construction (where a realistic catchable error path exists); ensure no partially committed state or unrooted return.
6. Exercise `clear`, nested read re-entry and exception propagation; verify depth guard is balanced by performing a subsequent legitimate mutation.

Use the project's existing GC test idioms from `tests/core/memory_gc.rs`, `collections/array.rs`, `collections/vector.rs` and `heap/trace.rs`. Do not invent a new collector hook or force collection in production code. Any need to change GC-wide policy or hidden roots is an **escalation**, not an implementer workaround.

### 6.6 Source, metadata and reflection acceptance

Sample source probes must compile through the normal Universe and module-loading paths; examples are **contract sketches** until G2:

```telos
let bag = MutableBag<Int>.new()
bag.add(5, count: 3)
assert bag.count(5) == 3
assert bag.totalCount == 3

let dict = IndexedDict<Int, String>.new()
dict.insert("one", for: 1)
dict.insert("two", for: 2)
assert dict[1] == "one"
assert dict.entryAt(1).isSome

let ordered = OrderedSet<String>.new()
ordered.add("a")
ordered.add("b")
assert ordered.indexOf("a").isSome
```

Do **not** treat these sketches as verified Telos test syntax; adapt exact assertion and constructor forms to the current fixture harness **without changing adopted selectors**. Test generic arity, class identity and applied-type reflection; import from `universe.collections` and through a re-exporting module; verify native surface catalogs, package exposes and loaders agree. For typed `from<I>` factories, add a separate compile probe *before* implementing body loops to discriminate between a valid associated `Iterable::Item` projection and the known associated-family invocation lowering issue.

## 7. Baseline disposition and no-regression firewall

| Residual | Status at reviewed HEAD | Required scope/ruling |
|---|---|---|
| `R-MAP-REMOVE` (`core_collections`) | **Resolved in committed P5 audit follow-up.** Public `Map.remove(_)` returns self; corrected test assertions; checkpoint records 13/13 green. Historical pre-P6 failure retained. | Do **not** change Map runtime; rerun after P6 edits. |
| `Range.toList` missing | Pre-T1 language-corpus positive-lane observation, no contemporaneous-main test comparison. | Reproduce focused fixture on clean predecessor + current branch + appropriate main; assign Range/fixture owner; don't implement Range method as P6 patch. |
| Underconstrained `Map.new()` negative fixture | Pre-T1 language-corpus negative-lane observation, fixture reaches generic inference before re-entry scenario. | Provide legitimate contextual type or `Map<Dynamic,Dynamic>.new()` under fixture owner; rerun actual intended reentry. Do not weaken generic inference. |
| `R-PARSE-TEXT` and `R-PARSE-BYTES` | **Corrected in commit `ffa2ab96`.** `TextSource.slice` materializes Substring; byte consumers explicitly handle `Option<Int>`. | No Unicode Char parser migration claimed; P5 historical record remains accurate. |
| `R-PARSE-ASSOCIATED-INVOKE` | Current parser-package blocker: token-backed Cursor associated-family invocation; 3 of 4 selected parse-package tests pass, 11/12 runtime fixtures pass. | Existing P5 compiler/associated-family consultation owns investigation. P6 should avoid expanding scope; if new `from<I>` directly reaches same capability, escalate with a minimal independent reproducer. |
| P4 D1/D2/D4 static/reflection residuals | Not newly recertified at reviewed head. | Preserve checkpoint ownership, no speculative P6 attribution or claims of pass. |
| Native surface/global formatting | Prior P6/P4 reports include focused generator success and unrelated global `cargo fmt` drift. | New native declarations require fresh generator check; use scoped rustfmt, do not blanket format repository. |

**Baseline classification algorithm:** isolate one test fixture and exact command; run against feature branch pre-change; where needed reproduce against a **clean predecessor** and **contemporaneous main** without losing the active worktree; compare failure signatures, not just exit codes; assign to owning subsystem; record selected-test count and exact revision. An inherited failure alone is not a mandatory consultation trigger if it is unaffected and already assigned. A new P6-owned regression is not excused by unrelated inherited failures.

## 8. File-by-file implementation impact inventory and sequence

The following is a **verified ownership inventory**, not a claim that every file requires modifications for every type. Perform a fresh local symbol census after G2 and add only the required paths.

| Sequence | Path/symbol | Why touched | Test/proof |
|---:|---|---|---|
| 1 | `phalcom-native-meta/src/universe.rs::UniverseKey` | Stable identifiers, names, paths, binding/catalog, class relation, type forms | Metadata identity and arity |
| 2 | `phalcom-semantic/src/core_surface/identity.rs::CoreDeclarationIds` | Canonical declaration identities | Semantic type formation |
| 3 | `phalcom-core/src/universe/core_classes.rs` | Bootstrap three primordial classes, fields, mappings, resolve | Runtime class identity |
| 4 | `phalcom-modules/src/builtin.rs` (module children + include arms) | Locate source modules | Import and re-export compilation |
| 5 | `phalcom-core/core/universe/src/collections/package.ph` | Export new public source classes | Package import test |
| 6 | `phalcom-core/src/heap/{mod,object,accessors,trace}.rs` | Heap arms, storage ownership, accessors, tracing | Layout and forced GC |
| 7 | `phalcom-core/src/heap/{mutable_bag,indexed_dict,ordered_set}.rs` (new, suggested) | Family-specific data invariants | Pure storage assertions/model tests |
| 8 | `phalcom-core/src/primitive/ordered_hash.rs` | Extend one hash/==/reentry authority | Callback reentry/collision tests |
| 9 | `phalcom-core/src/primitive/mod.rs::is_mutable_collection_key` | Central admission and primitive module declarations | Key matrix |
| 10 | `phalcom-core/src/primitive/{mutable_bag,indexed_dict,ordered_set}.rs` (new, suggested) | Minimal native operations + atomic mutation | Core runtime tests |
| 11 | `phalcom-core/src/universe/primitives.rs` | Explicit primitive installation plus native surface parity | Surface generator/census |
| 12 | `phalcom-core/core/universe/src/collections/{mutable_bag,indexed_dict,ordered_set}.ph` (new, suggested) | Approved public wrappers, iterators/views, equality/display | Full Universe compile and language fixtures |
| 13 | `phalcom-core/src/value/mod.rs`, `value/render.rs` | Class mapping, raw/display and exhaustive object handling | Identity, reflection, debug |
| 14 | `phalcom-core/src/typing/inspect.rs` and related metadata readers (**audit**, not automatic edit) | Generic arity follows Universe metadata | Applied type/reflection |
| 15 | `phalcom-core/src/vm/dispatch.rs` (**audit only**) | Existing Map/Set literal/expansion specialization | Verify new keyed classes do not accidentally become Map literal builders |
| 16 | `phalcom-repl/src/snapshot.rs` (**audit only**) | Explicit primordial roots | Confirm whether three new classes require root/snapshot list extension |
| 17 | `phalcom-core/tests/core/mod.rs`, `tests/core/collections/`, language fixtures | Register actual test targets and oracles | Nonzero test selection |
| 18 | `docs/implementation/COLL006/COLL006.C1/` existing P6/checkpoint records | T2 rulings, executed evidence, walkthrough/handoff | Lifecycle integrity |

**Primitive floor governance:** TDR-0033 explicitly approves a bounded raw-native floor for Map/Set/Tuple/Range. T2's API ruling did not authorize P6's floor; the architect therefore approved a scoped amendment, which the user accepted and recorded as [TDR-0092](../../../decisions/accepted/0092-ordered-mutable-hash-collection-native-floor.md). Finalize exactly its 29 bindings and no more. The generated catalog baseline is 429, with 458 expected after the exact 29 additions; the legacy 239-binding test is a partial runtime audit, not a global count. Keep high-level combinators in source `.ph`.

## 9. Adversarial design and failure-mode review

| Risk | Concrete mechanism | Required prevention / failing test |
|---|---|---|
| R1: Invented contract | Implementer chooses convenient `return self`/`Option`/`Result` without authority | G2 decision tables and compilation tests written before implementation |
| R2: Mixed up positional/key lookup | `IndexedDict<Int,V>[3]` accidentally reads slot 3 | Typed test with keys 1,3,7 and distinct slot order; explicit named `entryAt` |
| R3: `Vec::swap_remove` | Index deletion changes observable order | No swap removal; sequence-model oracle after each remove |
| R4: Cached bucket stale | Move/insert/remove shifts `entries` without slot index repair | Pure invariant checks and collision-heavy alternating mutations |
| R5: False duplicate | Hash collision considered equivalent without Telos `==` | Distinct custom keys returning identical hash |
| R6: Numeric equality drift | Positionwise `==` differs from hash-key SameValueZero | Numeric NaN/+0/-0 and int/float membership/equality oracles, per T2 ruling |
| R7: Callback while borrowed | VM can re-enter heap while native borrows live | Explicit copy-out/borrow scope inspection; reentrant key tests |
| R8: Reentry depth leakage | `?` propagates before guard exit | Throw during hash and equality, then mutate same collection successfully |
| R9: Partial commit | Count updated before total overflow, or entries before buckets | Prepare exact count/index/return structures then single no-callback commit; failure snapshot comparison |
| R10: Floatification | Big counts accidentally convert through f64 or host i64 | >2^53/>i64 boundary tests when policy permits |
| R11: False key safety | New mutable type omitted from classifier until T6 | Admission rejection test in T3.1/T4.1/T5.1, not only T6 |
| R12: GC invisible references | New Object trace arm misses values; native temporary only in Rust local | Forced-GC callback and removed-object liveness tests |
| R13: Generic `Dynamic` fallback | `from<I>` compilation fails, source erases type to Dynamic | Early associated-Item compile probe; stop/escalate if unsupported |
| R14: ABI slot regression | Large unboxed Object arm widens SlotMap entries | `size_of::<Value>()`, `size_of::<Object>()`/existing layout test; box payload |
| R15: Spec authority laundering | Plan or passing test claimed as accepted public contract | Written T2 ruling, indexed normative spec and C1 checkpoint |
| R16: Baseline laundering | Pre-P6 `Range.toList` or parser failure treated as P6 regression or ignored as pass | Compare predecessor/main by signature; preserve residuals |
| R17: Extra literal syntax | Reuse Map/Set VM literal bytecode for new types | No P6 compiler literal changes; separate approval required |
| R18: Unclear reentrant value overwrite | Inner callback changes same IndexedDict value while outer operation returns previous | T2 linearization rule + focused nested overwrite test |
| R19: Snapshot claim from live iterator | Source-level loop holds stale index after remove/move | Document/approve mutation-during-iteration behavior and test |
| R20: Post-commit allocation/error | A `Some`/tuple/Result construction fails after mutating storage | Prepare rooted returns before commit or prove failure impossible |

### 9.1 Mandatory stop/escalation triggers

Stop for architect consultation (not repetitive brute-force debugging) if:

- a public selector signature, duplicate behavior, result/error shape, constructor/iterator policy or numeric key-equivalence rule remains unapproved;
- semantic generic type formation requires inventing a collection-private trait, fake `Dynamic` conformance, constructor arity special case, or unpublished inference variable;
- `Iterable::Item`/associated-family invocation cannot be lowered through existing canonical semantic authority and a proposed factory depends on that path;
- runtime `Object` identity, GC root policy, `Value` representation, common arena size or module identity requires cross-system redesign;
- a heap borrow must remain live across `hash`, `==`, comparison or external source callback;
- a mutation cannot be made receiver-atomic under the accepted callback/return semantics;
- preserving Map/Set requires changing an existing accepted public method or its numeric/re-entry behavior;
- exact count arithmetic cannot reuse canonical numeric policy without altering that policy broadly;
- a new error kind or native primitive is outside the ratified floor/approved general error taxonomy;
- tests indicate a genuinely new regression in a sibling subsystem, not mechanically in the files owned by P6.

**Non-trigger:** private module names, imports, local helper refactors, exact new test filename/filter once registered, formatting of files changed by P6, or a separately documented inherited failure with no P6 behavioral change. These are implementer mechanical responsibility.

## 10. Required lifecycle amendments and evidence format

T2 approval and normative amendments were recorded 2026-10-11 in `docs/specs/collection-families.md`, the existing `COLL006.C1.P6-detailed-implementation-plan.md`, consultation, and the **single** C1 checkpoint. G2 source oracles were observed failing before implementation. The architect's exact 29-binding floor and census ruling are recorded in accepted TDR-0092 and the P6 consultation. Preserve P5 audit chronology, historical Map failure, and current associated-family baseline.

Suggested checkpoint gate record structure:

```yaml
plan: COLL006.C1.P6
gate: G3 # sample
revision: <exact-local-commit>
status: PASS | FAIL | BASELINE_BLOCKED | NOT_RUN
command: <exact-command>
tests_selected: <number>
tests_passed: <number>
tests_failed: <number>
observed_behavior: <concrete-invariant-or-failing-fixture>
classification: P6_OWNED | PRE_P6_BASELINE | PREDECESSOR_INHERITED | UNKNOWN
comparison_revision: <specific-commit-or-null>
residuals: [<exact-ticket-and-owner>]
next_action: <bounded-action>
```

When P6 implementation is complete, write:

```text
COLL006.C1.P6-walkthrough.md
COLL006.C1.P6-handoff.md
```

Walkthrough: implementation call paths, structural invariants, exact types/selectors, before/after evidence and measured limitations. Handoff: active checkpoint state, revision/worktree status, verification matrix, residual owner, next plan (P7 or P9 sequencing as governed by the C1 checkpoint), no implied broad certification. P9 is still the program's wider release/certification boundary.

## 11. Implementer agent task handoff (copy-ready)

> **Scope and authority.** Work only on `COLL006.C1.P6` in `feat/telos-collection-upgrade`. Check the current local HEAD/status/diff before touching files; preserve unrelated changes. Read `AGENTS.md`, applicable workflow rules, current C1 checkpoint, existing P6 detailed plan and API consultation, P5 handoff, TDR-0028/TDR-0033, and the source paths in this document. The checked remote revision for this handoff was `ffa2ab96578acf2f176aa2ddbd31e655590c7d29`; refresh any drift locally.
>
> **Current gate.** T1 shared Map/Set key engine and T2/G2 are complete. TDR-0092 accepts the exact 29-binding floor and census disposition. Preserve the partial uncommitted implementation; finalize only the approved inventory and implement the six source-level derivations without intervening callbacks or suspension.
>
> **Next implementation slice:** continue the partially started T3–T5 work under the approved source contracts and TDR-0092's exact 29-binding native surface. Preserve canonical identity, GC and key-admission requirements; no `Dynamic` or manual class-name special case is permitted.
>
> **Then:** implement exact-count Bag storage/operations with BigInt and canonical numeric policy, shared Telos key engine, no-Heap-borrow callback discipline, receiver-level failure atomicity, type-specific GC and focused model tests. Proceed to IndexedDict (explicit key and positional coordinate systems), OrderedSet (unique positional semantics), T6 cross-family admission audit and T7 focused stabilization. Keep Map/Set public behavior unchanged. Source wrappers own non-primitive combinators; Rust raw primitives own necessary native storage and atomicity.
>
> **Testing:** run focused tests at each gate, confirm nonzero selection, collect exact pass/fail counts and commit refs, classify inherited failures separately. The P5 audit follow-up is committed; `core_collections` is recorded 13/13 passing, while P5 G6.B remains baseline-blocked by associated-family invocation lowering. P4 D1/D2/D4 and Range language-corpus issues remain independently owned. No workspace-wide release certification under P6; P9 owns that.
>
> **Escalation:** stop for any new public semantic choice, compiler type/associated-family authority problem, GC ownership redesign, unrooted callback temporary, unverifiable mutation atomicity or accepted Map/Set contract conflict. Provide a narrow incident packet: failing test, minimal reproducer, exact source/call path, observed output and diff, hypothesized cause, attempted scoped fixes, and proposed decision question. Do not disguise a hard semantic issue with a dynamic workaround.
>
> **Closeout:** update the existing P6 plan and C1 checkpoint, create P6 walkthrough/handoff, preserve all prior lifecycle evidence, and report precisely what was and was not verified.

## 12. Final checklist for architect acceptance

- [ ] Current local branch, revision and dirty tree verified; earlier P5 audit fix not overwritten.
- [x] T2 user/architect-approved selector/error/iteration/constructor ruling entered into existing records; governing spec indexed.
- [x] G2 source-level contract oracles written and observed failing before implementation.
- [x] TDR-0092 accepts the exact 29-binding amendment and is recorded in the accepted registry before finalizing native registrations.
- [ ] Three generic nominal identities, class slots, module catalogs, package exposes, native surface metadata all consistent.
- [ ] No new native Object variant bloats common heap arena; `Value` representation unchanged.
- [ ] All new keyed operations use the shared Telos engine and present central mutable-key admission immediately.
- [ ] All callbacks run without retained heap borrows, with proven rooting and balanced guard depth on ordinary errors.
- [ ] All receiver mutations validate/prepare before commit; no index/count torn updates after exceptions.
- [ ] Bag counts are exact Telos Int; positive stored counts; total sum; configured numeric policy enforced.
- [ ] IndexedDict key lookup and positional access distinct; slot/bucket views agree through moves and replacements; order-sensitive equality.
- [ ] OrderedSet unique; strict positional duplicates, idempotent ordinary add, stable move/compaction; order-sensitive equality.
- [ ] SameValueZero key-equivalence and collision tests cover all families; Map/Set semantics unchanged.
- [ ] Forced-GC, removed-value reclamation, iterator owner, callback temp and reflection tests pass.
- [ ] Independent source-contract and model-based tests execute with nonzero selected counts; relevant native-surface generator check passes.
- [ ] `R-MAP-REMOVE` recorded as **fixed test oracle** at `ffa2ab96`; no stale active blocker.
- [ ] P5 associated-family lowering, Range.toList, underconstrained fixture and P4 residuals remain separately classified; no false claims about main.
- [ ] G3–G7 verified in the single checkpoint; P6 walkthrough and handoff completed; P9 certification still outstanding.

---

**Plan status:** T1 and T2/G2 are complete; T3–T5 are partially implemented in the uncommitted worktree. TDR-0092 accepts the exact 29-binding inventory and census ruling. Continue against that approved surface. This file is the adopted execution guide, not evidence that T3–T7 or broad runtime/release certification have completed.
