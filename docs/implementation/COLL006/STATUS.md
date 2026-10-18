---
id: COLL006
status: IN_PROGRESS
completion: PARTIAL
verification: BASELINE_BLOCKED
---

# COLL006 status

COLL006 implementation is in progress within C1. P1–P7 are complete at their
bounded scopes. P8 is active, with T8/G8 integration and T9 lifecycle closeout
still outstanding. P9 has not passed G0 because P8 remains its hard
predecessor. P5's parse-package gate is closed at its declared integration
boundary; the wider C1 checkpoint remains open.

| Checkpoint | State | Active plan | Next action |
|---|---|---|---|
| `COLL006.C1` | `IN_PROGRESS / PARTIAL / UNVERIFIED` | `COLL006.C1.P8` | classify or resolve P8 T8/G8 residuals and complete P8 T9; P5 G6.B passes, while wider C1 residuals remain |

## Current program truth

- `COLL006.C1.P1` is complete: neutral bracket literals, explicit heads `<...>[...]`, inference holes `_`, partial annotation elaboration, and sequence lowering foundation are implemented.
- `COLL006.C1.P2` is complete: Array runtime storage, direct construction, private spread builder, and focused semantic/runtime evidence are recorded in its walkthrough.
- `COLL006.C1.P3` is complete for its bounded scope: shared exact static integer identity, heterogeneous generic formation, Vector semantic/runtime/lowering, inference publication, and focused post-convergence evidence are recorded in its walkthrough and checkpoint ledger.
- `COLL006.C1.P4` is complete: bare brackets now default to `Array<T>` at runtime and `Vector<T, N>` in static initializers; static sequences, frozen const Arrays, List migration, and focused stabilization are recorded in its walkthrough.
- `COLL006.C1.P5` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`. G6.A and G6.B pass; the complete parse-package gate passes 4/4 Rust integration checks and 13/13 runtime fixtures. Workspace tests, Clippy, and release certification were not run. See the P5 walkthrough/handoff and checkpoint for exact evidence.
- `COLL006.C1.P6` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`, as recorded in the checkpoint and P6 walkthrough.
- `COLL006.C1.P7` is `COMPLETE / IMPLEMENTED / FOCUSED_TESTED`; see the checkpoint ledger and P7 walkthrough for its accepted scope and evidence.
- `COLL006.C1.P8` remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; current T8/G8 evidence and T9 closeout remain outstanding. Its residuals are recorded in the P8 checkpoint continuation and P9 residual ledger.
- `COLL006.C1.P9` remains `IN_PROGRESS / PARTIAL / UNVERIFIED`; G0 is blocked pending P8 G8 and P8 lifecycle closeout. The P5 parse-package residual is resolved and must not be reported as an active blocker.
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

1. Resolve or baseline-classify P8 T8/G8 residuals and complete P8 T9.
2. Resume P9 T0/G0 only after its P8 predecessor gate and handoff are complete.
3. Preserve the P14 reconciliation: private Array snapshot + read-only Slice, with no `ImmutableSequence<T>` or runtime-sized Vector snapshot.
4. Keep dynamic associated-family invocation separate from the completed static parser-package path and assign it an independent plan if pursued.

P4 is `COMPLETE / IMPLEMENTED / BASELINE_BLOCKED`: D4 prevents a clean aggregate `static_constants` result. D1/D2 remain suspected unrelated verification residuals pending P9 comparison. None blocks P5 execution. No release certification is claimed.
