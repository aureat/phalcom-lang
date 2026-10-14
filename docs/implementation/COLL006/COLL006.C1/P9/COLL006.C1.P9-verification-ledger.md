---
id: COLL006.C1.P9
program: COLL006
checkpoint: COLL006.C1
kind: verification-ledger
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# COLL006.C1.P9 — Verification Ledger

## T0 — Takeover and rebaseline

### Starting anchor

- Branch: `feat/telos-collections`
- Starting `HEAD`: `06017339c36d4f8b06016b1d89aa9d2996673de9`
- Contemporaneous `origin/main`: `b00d75b3ac9f50d37cf9abfddfde00917e099031`
- Worktree: `/Users/altunhasanli/dev/phalcom/phalcom/.worktrees/collections`
- The target worktree initially had the copied P9 plan as its only untracked task file and no source modifications.
- P9 T0 made two mechanical P8 corrections in `range_map.ph`: `_RangeMapNode` became `RangeMapNode`, and the duplicate `@class new()` wrapper was removed. The immutable fields and `@constructor` APIs remain unchanged.

### Executed evidence

| Discriminator | Result | What it establishes |
|---|---|---|
| `RUSTFLAGS='' cargo test -p phalcom-core --test core core_range_map::range_map_construction_and_lookup -- --nocapture` on `feat/telos-collections` | **PASS**, 1 test; 127.34s | RangeMap's corrected Universe module bootstraps; basic construction, lookup, and old-root checks pass. This does not prove the full P8 contract. |
| `RUSTFLAGS='' cargo test -p phalcom-core --test core modules_parse_package` on `feat/telos-collections` after the two corrections | **FAIL**, 0/4 Rust tests; 9/12 runtime fixtures pass, 3 fail on `ListSource`'s immutable `_items` reinitialization at `packages/parse/src/source/list-source.ph:33`; 447.98s | The prior RangeMap parse and duplicate-selector errors are gone. The Cursor associated-invocation assertion remains masked by the earlier `ListSource` error. |
| Same `modules_parse_package` command on current `main` | **FAIL**, 0/4 Rust tests; 12/12 runtime fixtures fail: 9 legacy-metadata export errors and the same 3 `ListSource` errors; 153.02s | Current main has additional post-branch metadata-export drift. The `ListSource` failure reproduces with the same source and diagnostic. The 447.98s vs 153.02s test-time difference is recorded as an observation only; cause is not established. |
| `RUSTFLAGS='' cargo run -p phalcom-native-surface-gen -- --root . --check` | **PASS**, 488 primitive declarations | Native surface artifact matches the current source tree. |
| `RUSTFLAGS='' cargo test -p phalcom-core --lib value_is_exactly_sixteen_bytes` | **PASS**, 1 test | `Value` remains 16 bytes. |
| `RUSTFLAGS='' cargo test -p phalcom-core --lib object_slot_size_is_unaltered` | **PASS**, 1 test | `Object` remains 48 bytes. |

### Constructor authority

`docs/specs/objects/classes.md` says `@constructor` provides the ordinary class-side factory and instance initializer. `RangeMap` keeps its `@constructor new()` and overloaded `@constructor new(root:, size:)`; the redundant `@class new()` had the same zero-argument selector and is removed.

### Not established

- P8 T3/T5's planned path-copy split/join/range-splice is missing: `removing` materializes all entries and reinserts survivors; `overlaying` calls `removing` then `inserting`. Its O(log n + k) contract is not established.
- P8 G7's multi-version GC, endpoint/value reachability, callback allocation/GC, randomized oracle, and lookup/overlay complexity gates have no recorded evidence in the current P8 handoff. The current RangeMap tests also do not contain these discriminators.
- The P8 handoff does not enumerate residuals, although P9 T0 requires them.
- The exact associated `Cursor.current` / `advance.current` failure was not reached by the current parser-package run. Its prior compiler consultation remains unresolved.
- P4 D1/D2/D4 and the P7 numeric hash-law matrix were not rerun during this takeover.

### Gate state

T0 is **PARTIAL**. G0 is **BLOCKED** because P8 T3/T5 and its
hard-predecessor evidence are incomplete. The plan-order decision is resolved:
resume P8 T3/T5 split/join/range-splice first, then T7/G7, G8, and a current
residual handoff; no P9 amendment is needed. No P9 production implementation
has started. See `COLL006.C1.P9-T0-G0-takeover-incident.md` and checkpoint
rulings `COLL006.C1.A12` / `COLL006.C1.A13`.

## P8 predecessor revalidation — 2026-10-13

The earlier T0 findings above are historical and are superseded for P8's
split/join, G7, and residual-handoff state by the current checkpoint continuation.
The original P9 T0 starting anchor remains `06017339c36d4f8b06016b1d89aa9d2996673de9`; the P8 continuation is recorded as work against that anchor. No P9 production implementation has started.

- RangeMap focused suite: PASS, 15/15 after T6 mutation-boundary normalization.
- P6/P7 `core_collections`, `core_bag`, and `core_bijection`: PASS (16 passed / 1 ignored, 5/5, and 6/6 respectively).
- `cargo check -p phalcom-core --all-targets`: PASS.
- T8 remains incomplete: `language::data_e2e` is 30/32, `memory_gc` is 13/19, the exact numeric Range generic-spread test fails, and `cargo fmt --all -- --check` reported repository-wide formatting diffs. Scoped `range_map.rs` formatting and `git diff --check` pass.
- These T8 failures have not been compared against an equivalent P8 baseline and remain `AMBIGUOUS`; see the P8 checkpoint evidence and P9 residual ledger for exact failure names.

T0 remains **PARTIAL** and G0 **BLOCKED** until P8 satisfies G8 and completes its lifecycle handoff. No P9 production work is authorized by the current gate state.
