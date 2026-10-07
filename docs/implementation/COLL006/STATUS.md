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
| `COLL006.C1` | `PROPOSED / NOT_STARTED / UNVERIFIED` | none | author `COLL006.C1.P1` for contextual bracket-literal and inference-hole foundation |

## Current program truth

- The C1 acceptance objective and target collection ontology are defined.
- No COLL006 runtime/compiler implementation is claimed by these records.
- No COLL006 verification gate has run.
- No numbered C1 implementation plan has yet been published.
- `COLL006.C1-CHECKPOINT.md` is the canonical shared state document for C1 once these records are adopted.

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
