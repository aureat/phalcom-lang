# TDR-0092 — Admit the ordered mutable hash collection native floor

- Ratified: 2026-10-11
- Status: Accepted
- Date: 2026-10-11
- Supersedes: none
- Amends: [TDR-0017](./0017-freeze-vm-blessed-primitive-floor.md)
- Related: [TDR-0028](./0028-collections-representation-and-literals.md), [TDR-0033](./0033-amend-floor-admit-collection-container-primitives.md), [TDR-0063](./0063-admit-bytes-native-octet-buffer.md), and [`collection-families.md`](../../specs/collection-families.md)
- Implementation record: [`COLL006.C1.P6-native-floor-consultation.md`](../../implementation/COLL006/COLL006.C1/COLL006.C1.P6-native-floor-consultation.md)

The user accepted this amendment as written on 2026-10-11. It is the scoped
P6 amendment to TDR-0017 and authorizes only the exact 29 bindings listed
below.

## Context

TDR-0017 freezes the VM-blessed native primitive floor and requires an explicit
amendment to admit new bindings. TDR-0028 establishes the collection
architecture: native collection arms may receive narrowly justified storage
operations while higher-level behavior stays in Telos source. TDR-0033
applies that policy to Map, Set, Tuple, and Range; it does not admit the
distinct `MutableBag<T>`, `IndexedDict<K,V>`, or `OrderedSet<T>` bindings.

The P6 public API contracts and rooted start-of-call equality snapshot policy
are specified separately in `collection-families.md`. This decision amends
only the native mechanism floor required to implement those contracts. A
private or source-`internal` native selector is still a floor binding when it
is implemented and registered by the VM.

TDR-0063 supplies the applicable bulk-operation precedent: callback-free
container operations may be native, while block-taking traversal and
combinators remain source implementations.

## Decision

### 1. Native admission

Admit exactly **29 new native method bindings** for `MutableBag<T>`,
`IndexedDict<K,V>`, and `OrderedSet<T>`: nine, ten, and ten respectively. The
exact owner, side, and selector inventory is:

#### MutableBag — 9 bindings

| Side | Selector |
|---|---|
| Class | `new()` |
| Instance | `rawSize` |
| Instance | `rawDistinctCount` |
| Instance | `rawCount(_)` |
| Instance | `rawAdd(_,_)` |
| Instance | `rawRemove(_,_)` |
| Instance | `rawAt(_)` |
| Instance | `rawCountAt(_)` |
| Instance | `rawClear` |

#### IndexedDict — 10 bindings

| Side | Selector |
|---|---|
| Class | `new()` |
| Instance | `rawSize` |
| Instance | `rawPut(_,_)` |
| Instance | `rawKeyAt(_)` |
| Instance | `rawValueAt(_)` |
| Instance | `rawInsertAt(_,_,_)` |
| Instance | `rawMove(_,_)` |
| Instance | `rawRemoveAt(_)` |
| Instance | `rawIndexOfKey(_)` |
| Instance | `rawClear` |

#### OrderedSet — 10 bindings

| Side | Selector |
|---|---|
| Class | `new()` |
| Instance | `rawSize` |
| Instance | `rawAdd(_)` |
| Instance | `rawAt(_)` |
| Instance | `rawIndexOf(_)` |
| Instance | `rawInsertAt(_,_)` |
| Instance | `rawReplaceAt(_,_)` |
| Instance | `rawMove(_,_)` |
| Instance | `rawRemoveAt(_)` |
| Instance | `rawClear` |

No other P6 native selector is admitted by this amendment.

### 2. Scope of native behavior

Native implementation is limited to allocation, representation observation,
keyed lookup, coordinated structural mutation, and callback-free native bulk
operations required to maintain these representations. Collection iteration,
equality and equality loops, constructor iteration, index normalization,
public result/error adaptation, and higher-order operations remain Telos source
implementations.

The removed draft bindings are intentionally not admitted:

| Owner | Not admitted | Required source derivation |
|---|---|---|
| MutableBag | `rawContains(_)` | `rawCount(value) > 0` |
| IndexedDict | `rawGet(_)`, `rawHas(_)`, `rawRemove(_)` | Lookup and value observation; lookup for presence; lookup followed by positional removal |
| OrderedSet | `rawContains(_)`, `rawRemove(_)` | Index lookup; index lookup followed by positional removal |

For IndexedDict, source-derived `get` must preserve its approved `Option<V>`
contract, including when `V` is itself optional. `rawPut` remains one atomic
native operation. Derived removal must have no user callback, suspension, or
user-controlled mutation opportunity between lookup and positional removal;
this condition is part of P6 verification. OrderedSet ordinary `add` remains
idempotent, while positional insertion remains strict and rejects duplicates.

### 3. Shared hash and callback authority

Every keyed operation continues to use the shared
`phalcom-core/src/primitive/ordered_hash.rs` machinery. User-defined `hash` and
`==` remain ordinary Telos dispatches. Existing numeric SameValueZero,
mutable-key rejection, callback reentrancy protection, reentrant-read
behavior, and callback-error propagation remain mandatory. No live heap borrow
may cross a Telos callback, and callback guards must be released on success
and error paths.

Each native mutation must leave the complete collection invariant valid at
its return or failure boundary. Callback, validation, or numeric-policy errors
must not leave partially changed entries, buckets, positional indices,
multiplicity totals, or retained values.

The previously approved rooted start-of-call snapshot policy for equality is
unchanged. Equality remains in Telos source; this amendment admits no native
equality selector.

### 4. `rawClear`

The three `rawClear` bindings are admitted as callback-free container bulk
operations under TDR-0063. Each must clear entries and all associated indices
together, release retained values correctly, invoke no user hash/equality
callback, respect the existing structural-mutation guard, and leave a valid
empty receiver.

### 5. Census and verification

The machine-readable native descriptor inventory is
`phalcom-native-surface/src/generated.rs`, generated from authored declarations
by `phalcom-native-surface-gen`. At the checked committed P6 base it contains
429 records. With exactly these 29 additions and no unrelated catalog changes,
the expected generated catalog count is 458. This catalog count does not by
itself certify runtime installation or governance of historical entries.

The test
`phalcom-core/tests/core/object_model/invariants.rs::floor_census_matches_installed_bindings`
is an explicitly enumerated partial runtime audit. Its base assertion of 239
does not count every native owner and must not be described as the global
runtime floor. P6 must add the three new class identities and exactly these 29
selectors to that audit, investigate and document its existing class-coverage
omissions, and ensure runtime native-surface validation covers all three P6
classes. P6 need not repair unrelated omissions; any such discrepancies remain
separately classified. The exact scope and result of the legacy audit must be
reported as partial.

The historical paths `docs/spec/current/core/floor-census.md` and
`docs/forge/units/U-COLLTYPES/plan.md` are retired references and are not
deliverables. Do not recreate them. Before P6 G3–G5 pass, verify the exact
declaration inventory, generated catalog, runtime registration parity, and
updated P6 rows in the partial executable audit. The expected catalog delta is
29 from the committed baseline, with any unrelated changes accounted for.

### 6. Non-admission

This amendment does not authorize native collection combinators, block-taking
traversal, source-level equality loops, generalized sorting, new hash/equality
semantics, or additional convenience selectors. It does not alter T2 public
contracts, the rooted-snapshot equality decision, Map/Set behavior, P5's
associated-family blocker, or the broader release-certification boundary.

## Consequences

- P6 can complete its three new native collection representations within the
  exact 29-binding floor under this accepted amendment.
- The six removed draft selectors must be implemented by source composition
  under the callback and suspension constraints above.
- P6 owns focused verification of its registration and partial runtime audit
  rows. Full global runtime-floor certification is not claimed here.
- The generated catalog is expected to move from 429 to 458 records if the
  only additions are these 29 bindings.
- P7, P8, P9, and broad release certification remain governed by their own
  plans.
