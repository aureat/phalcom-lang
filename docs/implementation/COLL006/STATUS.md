---
id: COLL006
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# COLL006 status

COLL006 implementation is in progress within C1. P1–P5 are complete at their
bounded scope; P6 is next.

| Checkpoint | State | Active plan | Next action |
|---|---|---|---|
| `COLL006.C1` | `IN_PROGRESS / PARTIAL / BASELINE_BLOCKED` | none; P5 complete, P6 next | plan P6; retain G6.B/aggregate G6 parser blockers and D1/D2/D4 residuals for P9 |

## Current program truth

- `COLL006.C1.P1` is complete: neutral bracket literals, explicit heads `<...>[...]`, inference holes `_`, partial annotation elaboration, and sequence lowering foundation are implemented.
- `COLL006.C1.P2` is complete: Array runtime storage, direct construction, private spread builder, and focused semantic/runtime evidence are recorded in its walkthrough.
- `COLL006.C1.P3` is complete for its bounded scope: shared exact static integer identity, heterogeneous generic formation, Vector semantic/runtime/lowering, inference publication, and focused post-convergence evidence are recorded in its walkthrough and checkpoint ledger.
- `COLL006.C1.P4` is complete: bare brackets now default to `Array<T>` at runtime and `Vector<T, N>` in static initializers; static sequences, frozen const Arrays, List migration, and focused stabilization are recorded in its walkthrough.
- `COLL006.C1.P5` is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`. The user-approved audit amendment separates G6.A independent snapshot acceptance (PASS) from G6.B complete parse-package certification (BASELINE_BLOCKED). The standalone fixture and native GC owner test pass; T9 focused regressions and workspace all-target check pass, with unrelated collection-contract and formatting baselines retained. See the P5 walkthrough/handoff and checkpoint for exact evidence.
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

1. Plan P6 using the P5 handoff. Keep G6.B/aggregate G6 baseline-blocked until the parser migration failures are repaired and the complete package command passes; retain the P9 recheck/main-comparison obligation.
2. Preserve the P14 reconciliation: private Array snapshot + read-only Slice, with no `ImmutableSequence<T>` or runtime-sized Vector snapshot.
3. Keep advanced associative/persistent collection work in later C1 plans.

P4 is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`: D4 prevents a clean aggregate `static_constants` result. D1/D2 remain suspected unrelated verification residuals pending P9 comparison. None blocks P5 execution. No release certification is claimed.
