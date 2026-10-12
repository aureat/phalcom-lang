# COLL006.C1.P6 — Native Primitive Floor Consultation

## 1. Identity and disposition

- Program/checkpoint/plan: `COLL006.C1.P6`
- Gate: T3–T5 native registration / G3–G5
- Repository/branch: `aureat/phalcom-lang`, `feat/telos-collection-upgrade`
- Reviewed base revision: `ffa2ab96578acf2f176aa2ddbd31e655590c7d29`
- Architectural ruling: received from the user on 2026-10-11
- Disposition: **Accepted as TDR-0092 on 2026-10-11; implementation may resume within the exact 29-binding inventory.**

The architect resolved the required selector inventory, census disposition,
and resumption condition. This consultation is therefore closed as an
architectural question. The proposed TDR remains unaccepted; no binding is
authorized by this consultation alone.

The accepted decision is
[`TDR-0092`](../../../decisions/accepted/0092-ordered-mutable-hash-collection-native-floor.md).
The user accepted the architect-approved text as written; it was assigned the
next canonical number and entered in the accepted registry.

## 2. Governing authority

| Decision | Binding requirement | P6 consequence |
|---|---|---|
| [TDR-0017](../../../decisions/accepted/0017-freeze-vm-blessed-primitive-floor.md) | Native methods may enter the frozen floor only through an amending decision. | P6 needs a dedicated accepted amendment. |
| [TDR-0028](../../../decisions/accepted/0028-collections-representation-and-literals.md) | Native collection arms receive narrowly justified raw-storage primitives; higher-level combinators remain in Telos source. | The P6 representation fits the architecture, but this does not itself admit its bindings. |
| [TDR-0033](../../../decisions/accepted/0033-amend-floor-admit-collection-container-primitives.md) | Explicitly admits 21 bindings for Map, Set, Tuple, and Range. | It does not automatically admit MutableBag, IndexedDict, or OrderedSet. |
| [TDR-0063](../../../decisions/accepted/0063-admit-bytes-native-octet-buffer.md) | Native callback-free container bulk operations are admissible; block-taking combinators remain in source. | Supports the three narrowly scoped `rawClear` bindings. |

A private or `internal` selector is still a native-floor binding when the VM
implements and registers it.

## 3. Exact inventory approved for amendment

The architect reduced the 35-binding draft by six redundant selectors. The
following **29 bindings** are the exact inventory for the proposed amendment.

### 3.1 MutableBag — 9 bindings

| Binding | Ruling | Scope |
|---|---|---|
| `new()` | APPROVE | Allocate native bag storage. |
| `rawSize` | APPROVE | Read total multiplicity. |
| `rawDistinctCount` | APPROVE | Read distinct-entry count. |
| `rawCount(_)` | APPROVE | Hash-based multiplicity lookup. |
| `rawAdd(_,_)` | APPROVE | Atomically update multiplicity and total count. |
| `rawRemove(_,_)` | APPROVE | Atomically remove up to the requested multiplicity and repair the index. |
| `rawAt(_)` | APPROVE | Observe a distinct value by position. |
| `rawCountAt(_)` | APPROVE | Observe its exact multiplicity by position. |
| `rawClear` | APPROVE | Clear entries and indices without repeated callbacks. |

`rawContains(_)` is removed; source derives membership as `rawCount(value) > 0`.
`rawAdd` and `rawRemove` remain native because a multi-send update could expose
inconsistent entry counts, totals, hash indices, or entry lifetime.

### 3.2 IndexedDict — 10 bindings

| Binding | Ruling | Scope |
|---|---|---|
| `new()` | APPROVE | Allocate native ordered dictionary storage. |
| `rawSize` | APPROVE | Read entry count. |
| `rawPut(_,_)` | APPROVE | Atomic key insertion/replacement without unintended movement. |
| `rawKeyAt(_)` | APPROVE | Observe key by position. |
| `rawValueAt(_)` | APPROVE | Observe value by position. |
| `rawInsertAt(_,_,_)` | APPROVE | Atomic duplicate-checked positional insertion. |
| `rawMove(_,_)` | APPROVE | Move an entry and repair indices atomically. |
| `rawRemoveAt(_)` | APPROVE | Remove by position and repair indices atomically. |
| `rawIndexOfKey(_)` | APPROVE | Hash-based lookup returning an entry position. |
| `rawClear` | APPROVE | Clear entries and hash index together. |

`rawGet(_)` is removed; source composes `rawIndexOfKey(_)` and
`rawValueAt(_)`, preserving the public `Option<V>` contract even when `V` is
itself optional. `rawHas(_)` is removed; source derives it from
`rawIndexOfKey(_)`. `rawRemove(_)` is removed; source composes key lookup with
`rawRemoveAt(_)`.

`rawPut` must remain one native atomic operation. The derived remove sequence
is valid only when no user callback, suspension, or user-controlled mutation
opportunity occurs between lookup and positional removal. This is a required
source-level test condition.

### 3.3 OrderedSet — 10 bindings

| Binding | Ruling | Scope |
|---|---|---|
| `new()` | APPROVE | Allocate native ordered-set storage. |
| `rawSize` | APPROVE | Read unique-element count. |
| `rawAdd(_)` | APPROVE | Atomic idempotent insertion. |
| `rawAt(_)` | APPROVE | Observe an element by position. |
| `rawIndexOf(_)` | APPROVE | Hash-based element-to-position lookup. |
| `rawInsertAt(_,_)` | APPROVE | Atomic strict positional insertion with duplicate rejection. |
| `rawReplaceAt(_,_)` | APPROVE | Atomic positional replacement with uniqueness enforcement. |
| `rawMove(_,_)` | APPROVE | Move an existing element and repair indices. |
| `rawRemoveAt(_)` | APPROVE | Remove by position and repair the index. |
| `rawClear` | APPROVE | Clear the native representation. |

`rawContains(_)` is removed; source derives it from `rawIndexOf(_)`.
`rawRemove(_)` is removed; source composes `rawIndexOf(_)` and
`rawRemoveAt(_)`, with no intervening callback or suspension. `rawAdd(_)`
remains separate from strict positional insertion because ordinary add is
idempotent and must not expose a duplicate error after repeating user-defined
hash/equality callbacks. `rawReplaceAt` remains atomic.

### 3.4 Count and removed draft bindings

| Owner | Draft | Approved | Removed |
|---|---:|---:|---|
| MutableBag | 10 | 9 | `rawContains(_)` |
| IndexedDict | 13 | 10 | `rawGet(_)`, `rawHas(_)`, `rawRemove(_)` |
| OrderedSet | 12 | 10 | `rawContains(_)`, `rawRemove(_)` |
| **Total** | **35** | **29** | **6** |

This is an approved inventory for a proposed amendment, not an accepted TDR.
No other convenience selector is admitted.

## 4. Required architecture and boundaries

The amendment authorizes native mechanisms, not alternative collection
semantics.

- Keyed operations continue through the shared
  `phalcom-core/src/primitive/ordered_hash.rs` authority.
- User `hash` and `==` remain ordinary Telos dispatches. Preserve numeric
  SameValueZero, mutable-key rejection, callback reentrancy protection, and
  error-safe lock release.
- No live heap borrow may cross a user callback. Root values needed across
  callbacks and keep the receiver valid on every success and failure path.
- Each native mutation maintains the full representation invariant at its
  return or failure boundary: slots, buckets, positions, totals, and retained
  values cannot be partially updated.
- The previously approved rooted start-of-call snapshot policy for equality
  remains unchanged. Equality stays in source; no native equality binding is
  admitted.
- Iteration, equality loops, constructor iteration, index normalization,
  public result/error adaptation, and higher-order operations remain Telos
  source implementations.
- `rawClear` invokes no user hash/equality callback, clears entries and all
  related indices together, releases retained values, and leaves a valid empty
  receiver. It rejects structural mutation when the existing callback guard
  prohibits it.

The draft `OrderedSet` source module is still absent in the current worktree;
the registered module catalog alone does not close that source requirement.

## 5. Census ruling and required scope

### 5.1 Generated native-surface catalog

The canonical machine-readable descriptor inventory is generated at
`phalcom-native-surface/src/generated.rs` from authored native declarations by
`phalcom-native-surface-gen`.

At the architect's checked committed revision, the catalog contains 429
records. Adding exactly the 29 approved bindings yields an expected final
catalog count of **458**, assuming there are no unrelated additions or
removals. This is a descriptor count, not a verified count of runtime-installed
bindings and not proof that every historical descriptor has separate
governance approval.

The current uncommitted worktree's generated catalog has 439 records. That is
partial draft state and is not a final or accepted count.

### 5.2 Legacy executable floor audit

`phalcom-core/tests/core/object_model/invariants.rs::floor_census_matches_installed_bindings`
is a useful, explicitly enumerated **partial runtime audit**. At the reviewed
base revision it asserts 239 expected/live bindings, but omits several current
native owners, including Vector, Slice, and Buffer. Do not describe 239 as the
global runtime floor count, and do not turn P6 into a broad repair of every
omission.

P6 must add the three new collection classes and exactly their 29 approved
selectors to this explicit audit, investigate and document its existing class
coverage omissions, and ensure the runtime validator covers all three P6 class
identities. The class-coverage discrepancy in `validate_native_surface` must be
checked; the architect did not independently reproduce a runtime failure from
it. Keep unrelated omissions separately classified. If the existing audited
rows are unchanged, their explicit row count grows by 29, but that value still
describes only this partial audit, never the global native floor.

### 5.3 Retired historical path

`docs/spec/current/core/floor-census.md` and
`docs/forge/units/U-COLLTYPES/plan.md` do not exist on the checked branch.
Decision: retire these stale path requirements. Do not recreate either path.
Use the generated catalog for descriptor inventory and the legacy executable
test for its explicitly enumerated runtime audit.

### 5.4 P6 census evidence

Before G3–G5 pass, P6 must:

1. Regenerate and check the catalog, showing the exact 29-binding inventory and
   expected count of 458 relative to the 429-record committed baseline, with
   unrelated changes accounted for explicitly.
2. Verify owner/side/selector details, not just the count.
3. Extend the explicit runtime audit with the three new classes and all 29
   approved selectors; document that the audit is partial.
4. Verify runtime registration parity for these three class identities.
5. Record any broader class-coverage discrepancy as separate work rather than
   silently broadening P6.

## 6. Verification requirements

| Gate | Required evidence |
|---|---|
| Declaration generation | Exact 29 approved declarations; no removed selector remains registered |
| Generated catalog | `cargo run -p phalcom-native-surface-gen -- --check` passes; expected count reconciled to baseline 429 and final 458 |
| Registration parity | The three P6 owners and selectors match generated descriptors and runtime registration |
| Legacy floor audit | Three P6 owners and all approved selectors are enumerated; audit is explicitly labeled partial |
| MutableBag | Exact count arithmetic, duplicates, saturating removal, reentrancy, GC |
| IndexedDict | Position/index coherence, replacement stability, duplicate rejection, nested Option values, callback-free derived removal |
| OrderedSet | Uniqueness, order preservation, atomic replacement and movement, callback-free derived removal |
| Compatibility | Existing Map/Set collision and reentrancy regressions remain green |

The missing full runtime floor count remains unverified. The P5 associated-family
invocation blocker and P4 D1/D2/D4 residuals remain separate from P6.

## 7. Acceptance and implementation sequence

Formal acceptance is complete: TDR-0092 records the exact 29-selector
amendment, and the decision registry and C1 checkpoint have been updated. The
existing uncommitted implementation is preserved. Continue in this order:

1. Remove the six unapproved draft bindings; implement their source-level
   equivalents under the stated callback/suspension constraints.
2. Complete source wrappers and runtime registration for T3–T5, including the
   missing OrderedSet source module.
3. Regenerate the native catalog and complete the P6 census work.
4. Run the focused gates and continue P6.

No additional architectural consultation is needed while implementation stays
within these 29 bindings. Re-consult only if a removed operation cannot be
derived without violating its public contract, another native operation proves
necessary, the shared hash/callback architecture must change, approved equality
contracts conflict with implementation evidence, or a materially different
storage representation is required. Routine source wrappers, Rust fixes,
tests, and selector spelling consistent with the exact inventory are
implementation work.

```yaml
consultation: COLL006.C1.P6-native-primitive-floor
architecture: SCOPED_AMENDMENT_ACCEPTED
amendment_status: ACCEPTED_TDR_0092
inventory:
  total: 29
  mutable_bag: 9
  indexed_dict: 10
  ordered_set: 10
generated_catalog:
  baseline: 429
  expected_after_p6: 458
  current_worktree_draft: 439
legacy_runtime_audit:
  base_assertion: 239
  scope: PARTIAL_EXPLICIT_AUDIT
historical_floor_census_path: RETIRED
may_resume_after_formal_acceptance: true
```
