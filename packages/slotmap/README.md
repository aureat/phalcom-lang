# slotmap

A pure-Telos implementation of generational slot maps.

`slotmap` provides stable, typed handles without pointers, unsafe code, native
storage intrinsics, or runtime-specific representation tricks.

## Package surface

- `SlotKey<Tag>` — stable handle = slot index + generation.
- `SlotMap<Tag, T>` — sparse stable-slot storage.
- `DenseSlotMap<Tag, T>` — compact value storage behind stable slots.
- Canonical `Iterable` / `Iterator` conformances for both maps.
- `insertWithKey` for values that store their own handle.
- `retain`, `clear`, checked indexing, `containsKey`, `get`, `remove`.

## Core invariant

A key is live iff all three facts hold:

1. its internal slot index exists;
2. the slot is occupied;
3. the internal key generation equals the slot generation.

Removal increments the generation before placing the slot on the freelist.
Reusing that slot therefore cannot revive the old key.

Unlike fixed-width generation counters, this implementation uses Telos `Int`.
Generations intentionally do not wrap. Index and generation are implementation
state; callers should treat `SlotKey<Tag>` as an opaque handle.

## Typed key domains

```telos
from slotmap.slot_map import SlotKey, SlotMap

data Philosophers()
data Propositions()

type PhilosopherId = SlotKey<Philosophers>
type PropositionId = SlotKey<Propositions>

let philosophers = SlotMap<Philosophers, String>()
let propositions = SlotMap<Propositions, String>()

let socrates: PhilosopherId = philosophers.insert("Socrates")
let mortality: PropositionId =
  propositions.insert("All humans are mortal")

// Static type error:
// propositions.get(socrates)
```

The singleton `Tag` type prevents accidental cross-arena key use without requiring
macros or generated nominal wrapper types. Marker declarations use explicit empty
product syntax such as `data Philosophers()`.

## Sparse SlotMap

`SlotMap` stores values directly inside stable slots.

```text
free
 │
 ▼
[5] Vacant(next = 2)
                │
                ▼
       [2] Vacant(next = None)

[0] Occupied(value, generation = 0)
[1] Occupied(value, generation = 3)
```

The freelist is intrusive: vacant slots themselves carry the next free index.

Complexity:

| operation | complexity |
|---|---:|
| insert | O(1) amortized |
| get / contains | O(1) |
| remove | O(1) |
| iteration | O(number of stable slots) |
| clear | O(number of stable slots) |

## DenseSlotMap

`DenseSlotMap` separates stable-key space from dense-value space:

```text
stable slots                    dense storage

slot 0 -> dense 2 ────────────> key0   valueC
slot 1 -> vacant
slot 2 -> dense 0 ────────────> key2   valueA
slot 3 -> dense 1 ────────────> key3   valueB
```

Removal swap-moves the last dense entry into the removed dense position and
repairs the moved key's stable slot. User-visible keys do not change.

Complexity:

| operation | complexity |
|---|---:|
| insert | O(1) amortized |
| get / contains | O(1) |
| remove | O(1) |
| iteration | O(len) |
| clear | O(number of stable slots) |

## `insertWithKey`

```telos
data Entities()
type EntityId = SlotKey<Entities>

data Entity(
  id: EntityId,
  name: String
)

let entities = SlotMap<Entities, Entity>()

let socrates = entities.insertWithKey |id| {
  Entity(id: id, name: "Socrates")
}
```

The callback is evaluated before allocator state is committed. If construction
raises, the map remains structurally unchanged.

## Clear semantics

`clear` deliberately retains slots and advances generations for occupied slots.
Simply replacing the slot list with an empty list would be incorrect: a later
insert at index zero with generation zero could resurrect a key issued before
the clear.

## Layout

```text
slotmap/
├── project.toml
├── README.md
├── src/
│   ├── package.ph
│   ├── main.ph
│   ├── slot_map.ph
│   └── dense_slot_map.ph
├── examples/
│   ├── self_key.ph
│   └── typed_domains.ph
└── tests/
    └── runtime/
        ├── project.toml
        └── src/
            ├── package.ph
            ├── support.ph
            ├── basic_test.ph
            ├── dense_test.ph
            └── main.ph
```

## Verification status

The package is written against the current Phalcom/Telos repository conventions
observed on `main`: `project.toml`, `.ph` modules, `src/package.ph`, `@variant`
data constructors, `@constructor call`, `List` indexed mutation, and canonical
`Iterable` / `Iterator` conformances.

The execution environment used to produce this archive does not contain a local
Phalcom compiler checkout/binary, so the archive has not been compiler-executed
here. The runtime tests are included specifically to make integration validation
direct and mechanical when placed in the repository.
