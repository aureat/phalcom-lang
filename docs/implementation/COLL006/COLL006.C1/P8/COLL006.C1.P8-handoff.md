---
id: COLL006.C1.P8
program: COLL006
checkpoint: COLL006.C1
kind: handoff
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006.C1.P8 — Handoff to P9

This handoff's original completion claim and P9 T0 revalidation are superseded
by the 2026-10-12 continuation. The current state is **IN_PROGRESS / PARTIAL /
FOCUSED_TESTED**: source-owned split/join/range-splice and G5/G7 focused
evidence pass. The adviser ruled the P8 architecture sound with no amendment;
the T6 normalization correction coalesces touching equal-valued neighbors in
insertion and overlay, while removal preserves gaps. The RangeMap filter passed
15/15 after that change, including complexity and callback regressions. The
Interval filter passed 7/7 before the RangeMap-only T6 correction. T8 now has
passing P6/P7 collection suites and a passing core all-target check, but is not
green: `language::data_e2e` passed 30/32, `memory_gc` passed 13/19, the exact
numeric Range generic-spread regression failed, and repository-wide formatting
reports diffs in untouched files. These failures are recorded as ambiguous
causal residuals in the checkpoint and P9 residual ledger; no branch-baseline
comparison established ownership. `git diff --check` passes. T9 closeout remains
pending G8; P9 G0 stays blocked. This continuation started on
`feat/telos-collections` at `06017339c36d4f8b06016b1d89aa9d2996673de9`; the
branch tip containing this handoff and implementation is the current state
anchor. The C1 checkpoint record
(`docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md`) remains
the single durable shared state authority.

Takeover corrected two RangeMap bootstrap defects: `_RangeMapNode` was renamed
to parser-supported `RangeMapNode`, and a redundant `@class new()` selector was
removed while retaining the `@constructor new()` overloads. The follow-up also
added source-owned AVL split/join helpers, precise boundary splicing, empty
overlay identity, and spec-defined Bound/Interval equality and hashing. See the
checkpoint's P8 continuation record for exact focused evidence and remaining
G8 work.

---

## 1. Stable Takeover Interfaces

The successor plan (`COLL006.C1.P9` — Whole-C1 Convergence, API Consistency, `Entry<K,V>` Migration, Diagnostics, and Release Certification) and future plans inherit the following stable authorities:

1. **Native Catalog and Floor (Floor Preserved)**:
   - Zero new native bindings added (Route A, source-first persistent AVL in Telos).
   - Native surface count remains exactly **488 declarations** (`cargo run -p phalcom-native-surface-gen -- --root . --check`).
   - `Object` heap slot size remains **48 bytes**.

2. **Generic Interval Domain Foundation (`interval.ph`)**:
   - `Bound<T>` with variants `Unbounded`, `Included(_ endpoint: T)`, `Excluded(_ endpoint: T)`.
   - `Interval<T>(_ lower: Bound<T>, _ upper: Bound<T>)` with factory constructors (`closed`, `open`, `halfOpenRight`, `halfOpenLeft`, `atLeast`, `greaterThan`, `atMost`, `lessThan`, `all`, `singleton`).
   - Canonical boundary algebra kernel: `compareDomain`, `compareLower`, `compareUpper`, `isValid`, `isEmpty`, `contains`, `overlaps`, `intersection`, `touchesWithoutGap`, `unionIfContiguous`, `remaindersAfterRemoval`, `toString`.
   - Wired in `prelude.rs` as `Bound` and `Interval`.

3. **Error Hierarchy (`argument.ph`, `package.ph`, `prelude.rs`)**:
   - `InvalidIntervalError is ArgumentError`
   - `RangeOverlapError is ArgumentError`
   - Registered in prelude for unqualified access.

4. **Persistent Ordered Substrate & `RangeMap<K, V>` (`range_map.ph`)**:
   - `RangeMapNode<K, V>`: internal immutable persistent AVL tree node with pure rotations, rebalancing, and $O(\log n)$ point lookup.
   - `RangeMap<K, V> is Collection`:
     - Immutable mapping from pairwise-disjoint intervals to values.
     - Strict insertion (`inserting`) rejecting invalid, empty, or overlapping intervals.
     - Coverage subtraction (`removing`) with interior remainder splitting.
     - Range overlay (`overlaying`) replacing mapped coverage with new values.
     - Canonical normalization at mutation boundaries; `entries` and the iterator traverse the already-normalized in-order tree.
     - Extensional equality (`==`) and deterministic hash fold (`hash`).
     - Lazy iterator (`RangeMapIterator<K, V>`).
   - Registered in `collections/package.ph`, `phalcom-modules/src/builtin.rs`, and canonical `prelude.rs`.

5. **Test Suites**:
   - `phalcom-core/tests/core/collections/interval.rs`: capability probes and Bound/Interval unit tests.
   - `phalcom-core/tests/core/collections/range_map.rs`: 8 test suites covering construction, strict insertion, unbounded point lookup, coverage removal, overlay, coalescing, equality/hash, and iteration.

---

## 2. P9 Transition & Next Actions

- `COLL006.C1.P9` owns:
  - Whole-C1 collection API convergence.
  - `Entry<K, V>` migration boundary across mappings.
  - Final release-level certification and checkpoint closure.
