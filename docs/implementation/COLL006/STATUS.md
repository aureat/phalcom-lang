---
id: COLL006
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
---

# COLL006 status

COLL006 is defined but implementation has not started.

| Checkpoint | State | Active plan | Next action |
|---|---|---|---|
| `COLL006.C1` | `IN_PROGRESS / PARTIAL / FOCUSED_TESTED` | none | author and execute `COLL006.C1.P2` for `Array<T>` runtime/native implementation |

## Current program truth

- `COLL006.C1.P1` is complete: neutral bracket literals, explicit heads `<...>[...]`, inference holes `_`, partial annotation elaboration, and sequence lowering foundation are implemented.
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

1. Publish `COLL006.C1.P1` for the behavior-neutral literal refactor, literal heads, inference holes, and partial binding-annotation inference.
2. Plan runtime `Array<T>` and `Vector<T,N>` as separate successor plans.
3. Delay the repository-wide bare-bracket semantic cutover until both runtime families and static-sequence infrastructure are ready.
4. Keep advanced associative/persistent collection work in later C1 plans so it cannot block the sequence/literal foundation.

Publishing these lifecycle records claims no test execution, source modification, or verification promotion.
