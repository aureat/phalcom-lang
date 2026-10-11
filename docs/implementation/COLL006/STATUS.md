---
id: COLL006
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# COLL006 status

COLL006 implementation is in progress within C1. P1–P4 are complete at their
bounded scope; P5 is active.

| Checkpoint | State | Active plan | Next action |
|---|---|---|---|
| `COLL006.C1` | `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED` | `COLL006.C1.P5` (in progress; G0 baseline blocked, G1–G5 and G7 passed) | continue T9 focused stabilization; G6 remains blocked by unchanged parse-package failures; retain D1/D2/D4 residuals |

## Current program truth

- `COLL006.C1.P1` is complete: neutral bracket literals, explicit heads `<...>[...]`, inference holes `_`, partial annotation elaboration, and sequence lowering foundation are implemented.
- `COLL006.C1.P2` is complete: Array runtime storage, direct construction, private spread builder, and focused semantic/runtime evidence are recorded in its walkthrough.
- `COLL006.C1.P3` is complete for its bounded scope: shared exact static integer identity, heterogeneous generic formation, Vector semantic/runtime/lowering, inference publication, and focused post-convergence evidence are recorded in its walkthrough and checkpoint ledger.
- `COLL006.C1.P4` is complete: bare brackets now default to `Array<T>` at runtime and `Vector<T, N>` in static initializers; static sequences, frozen const Arrays, List migration, and focused stabilization are recorded in its walkthrough.
- `COLL006.C1.P5` has completed T0/G0 through T6/G5 and T8/G7. T7 migrated `ListSource` to a private Array snapshot retained by Slice and reconciled P14 lifecycle records; G6 remains baseline-blocked because the parse package still fails in unchanged TextSource and operators fixtures. Workspace all-target checking passes; T9 remains active.
- `COLL006.C1-CHECKPOINT.md` is the canonical shared state document for C1.

## Ratified target semantics

```telos
let x = [1, 2, 3]
// Array<Int>

const y = [1, 2, 3]
// Vector<Int, 3>

let z: List<_> = [1, 2, 3]
// List<Int>

const a: Array<_> = [1, 2, 3]
// static Array<Int>

let b = <_>[1, 2, 3]
// Array<Int>

let c = <_, _>[1, 2, 3]
// Vector<Int, 3>
```

The family also includes:

```text
Slice<T>
Buffer<T>
Bag<T>
MutableBag<T>
Bijection<K,V>
IndexedDict<K,V>
OrderedSet<T>
Entry<K,V>
Bound<T>
Interval<T>
RangeMap<K,V>
```

## Next planning order

1. Continue P5 T9 focused stabilization and close remaining owned verification; keep G6 baseline-blocked until the parse-package failures are addressed by their owner.
2. Preserve the P14 reconciliation: private Array snapshot + read-only Slice, with no `ImmutableSequence<T>` or runtime-sized Vector snapshot.
3. Keep advanced associative/persistent collection work in later C1 plans.

P4 is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`: D4 prevents a clean aggregate `static_constants` result. D1/D2 remain suspected unrelated verification residuals pending P9 comparison. None blocks P5 execution. No release certification is claimed.
