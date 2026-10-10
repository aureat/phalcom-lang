---
id: COLL006
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006 status

COLL006 implementation is in progress within C1. P1–P4 are complete at their
bounded scope; P5 is the next proposed plan.

| Checkpoint | State | Active plan | Next action |
|---|---|---|---|
| `COLL006.C1` | `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED` | `COLL006.C1.P5` (proposed) | begin Slice/Buffer work from the completed P4 handoff; retain D4 static Record-key baseline |

## Current program truth

- `COLL006.C1.P1` is complete: neutral bracket literals, explicit heads `<...>[...]`, inference holes `_`, partial annotation elaboration, and sequence lowering foundation are implemented.
- `COLL006.C1.P2` is complete: Array runtime storage, direct construction, private spread builder, and focused semantic/runtime evidence are recorded in its walkthrough.
- `COLL006.C1.P3` is complete for its bounded scope: shared exact static integer identity, heterogeneous generic formation, Vector semantic/runtime/lowering, inference publication, and focused post-convergence evidence are recorded in its walkthrough and checkpoint ledger.
- `COLL006.C1.P4` is complete: bare brackets now default to `Array<T>` at runtime and `Vector<T, N>` in static initializers; static sequences, frozen const Arrays, List migration, and focused stabilization are recorded in its walkthrough.
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

1. Prepare P5 from the completed P4 handoff and checkpoint state; establish Slice/Buffer contiguous-view and bounded-storage semantics.
2. Coordinate parser-snapshot exposure with `LANG007.C4.P14` as recorded in the checkpoint.
3. Keep advanced associative/persistent collection work in later C1 plans.

P4 is complete/implemented with focused evidence, but the checkpoint and program remain `BASELINE_BLOCKED` by the inherited three computed-Record-key failures recorded as D4. No release certification is claimed.
