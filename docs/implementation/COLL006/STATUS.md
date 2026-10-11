---
id: COLL006
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# COLL006 status

COLL006 implementation is in progress within C1. P1–P6 are complete at their
bounded scope. P7 is active: T0–T4/G2–G4 have recorded focused evidence, and
the consolidated main checkout now contains Bag implementation files whose
T5 completion and verification still need to be reconciled. Bijection remains
outstanding.

| Checkpoint | State | Active plan | Next action |
|---|---|---|---|
| `COLL006.C1` | `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED` | `COLL006.C1.P7` | reconcile the present Bag implementation against P7 T5, then continue Bijection; retain P5 G6.B/aggregate G6 and D1/D2/D4 residuals for P9 |

## Current program truth

- `COLL006.C1.P1` is complete: neutral bracket literals, explicit heads `<...>[...]`, inference holes `_`, partial annotation elaboration, and sequence lowering foundation are implemented.
- `COLL006.C1.P2` is complete: Array runtime storage, direct construction, private spread builder, and focused semantic/runtime evidence are recorded in its walkthrough.
- `COLL006.C1.P3` is complete for its bounded scope: shared exact static integer identity, heterogeneous generic formation, Vector semantic/runtime/lowering, inference publication, and focused post-convergence evidence are recorded in its walkthrough and checkpoint ledger.
- `COLL006.C1.P4` is complete: bare brackets now default to `Array<T>` at runtime and `Vector<T, N>` in static initializers; static sequences, frozen const Arrays, List migration, and focused stabilization are recorded in its walkthrough.
- `COLL006.C1.P5` is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`. The user-approved audit amendment separates G6.A independent snapshot acceptance (PASS) from G6.B complete parse-package certification (BASELINE_BLOCKED). The standalone fixture and native GC owner test pass; T9 focused regressions and workspace all-target check pass, with unrelated collection-contract and formatting baselines retained. See the P5 walkthrough/handoff and checkpoint for exact evidence.
- `COLL006.C1.P6` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`, as recorded in the checkpoint and P6 walkthrough.
- `COLL006.C1.P7` is `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` for the previously verified T0–T4/G2–G4 slices. Bag implementation files are present in the consolidated main tree without a T5 completion claim; Bijection remains outstanding. No runtime tests were run during consolidation; library-only checks passed, while the core test-target compile remains blocked by the unchanged `phalcom-core/src/vm/send.rs:1960` baseline call.
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

1. Reconcile and complete P7 T5 against the Bag implementation now present in the consolidated tree, then implement Bijection under P7.
2. Keep P5 G6.B/aggregate G6 baseline-blocked until the parser migration failures are repaired and the complete package command passes; retain the P9 recheck/main-comparison obligation.
3. Preserve the P14 reconciliation: private Array snapshot + read-only Slice, with no `ImmutableSequence<T>` or runtime-sized Vector snapshot.
4. Keep advanced associative/persistent collection work in later C1 plans.

P4 is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`: D4 prevents a clean aggregate `static_constants` result. D1/D2 remain suspected unrelated verification residuals pending P9 comparison. None blocks P5 execution. No release certification is claimed.
