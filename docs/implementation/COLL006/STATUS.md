---
id: COLL006
status: IN_PROGRESS
completion: PARTIAL
verification: FOCUSED_TESTED
---

# COLL006 status

COLL006 implementation is in progress within C1. P3 is paused at the user's request.

| Checkpoint | State | Active plan | Next action |
|---|---|---|---|
| `COLL006.C1` | `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` | `COLL006.C1.P3` (paused) | resume inline, complete Vector runtime verification and remaining P3 requirements |

## Current program truth

- `COLL006.C1.P1` is complete: neutral bracket literals, explicit heads `<...>[...]`, inference holes `_`, partial annotation elaboration, and sequence lowering foundation are implemented.
- `COLL006.C1.P2` is complete: Array runtime storage, direct construction, private spread builder, and focused semantic/runtime evidence are recorded in its walkthrough.
- `COLL006.C1.P3` is partial: shared exact static integer identity and heterogeneous generic formation are implemented; initial Vector semantic/runtime/lowering code exists. The 22-test COLL006 semantic filter passed. The runtime test command was aborted during compilation, and the remaining requirements are listed in the P3 handoff.
- Bare bracket default remains `List<T>` during P1/P2/P3; P4 owns the atomic cutover to `Array<T>` runtime and `Vector<T, N>` static defaults.
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

## Immediate planning order

1. Resume and finish `COLL006.C1.P3` using its partial walkthrough and handoff.
2. Start P4 only after the Vector runtime family and P3 acceptance gates are stable.
3. Delay the repository-wide bare-bracket semantic cutover until both runtime families and static-sequence infrastructure are ready.
4. Keep advanced associative/persistent collection work in later C1 plans so it cannot block the sequence/literal foundation.

Program/checkpoint verification remains `FOCUSED_TESTED`; P3 implementation completion remains `PARTIAL`. No release certification is claimed.
